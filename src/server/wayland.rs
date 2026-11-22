use super::*;
#[cfg(feature = "drm")]
use hbb_common::anyhow;

use crate::platform::linux::is_x11;

lazy_static::lazy_static! {
    static ref ACTIVE_DISPLAY_COUNT: RwLock<usize> = RwLock::new(0);
}

pub fn init() {}

pub(super) fn increment_active_display_count() -> usize {
    let mut count = ACTIVE_DISPLAY_COUNT.write().unwrap();
    *count += 1;
    *count
}

pub(super) fn decrement_active_display_count() -> usize {
    let mut count = ACTIVE_DISPLAY_COUNT.write().unwrap();
    if *count > 0 {
        *count -= 1;
    }
    *count
}

/// Uinput desktop rect from the DRM display list, for a login screen where no compositor can be
/// asked. `(minx, maxx, miny, maxy)`, in delivered-orientation physical pixels (a rotated
/// output counts transposed, matching its frames): no compositor here applied a scale, so
/// unlike `desktop_rect_of` there is no logical size to handle.
#[cfg(feature = "drm")]
fn drm_desktop_rect_for_uinput() -> Option<(i32, i32, i32, i32)> {
    let displays = super::drm_capturer::get_display_infos()?;
    if displays.is_empty() {
        return None;
    }
    let minx = displays.iter().map(|d| d.x).min()?;
    let miny = displays.iter().map(|d| d.y).min()?;
    let maxx = displays.iter().map(|d| d.x + d.width).max()?;
    let maxy = displays.iter().map(|d| d.y + d.height).max()?;
    if maxx <= minx || maxy <= miny {
        return None;
    }
    Some((minx, maxx, miny, maxy))
}

/// Set the uinput absolute-pointer range to the whole logical desktop so the compositor maps
/// injected coordinates 1:1 instead of stretching a single-monitor range across all outputs. On a
/// multi-monitor host the injected pointer otherwise lands on the wrong output, and the hardware
/// cursor may never appear on the captured CRTC. Reads the layout from the Wayland outputs.
///
/// The update performs three things:
///
/// - drops the cached Wayland layout first, because it can predate compositor changes made while no
///   session was active (rustdesk#15601), and on the hotplug path it is stale by definition;
/// - bounds the IPC wait, because `uinput::client::set_resolution` reads its reply with no timeout of
///   its own, so a hung uinput socket would otherwise block every video-service start on this branch
///   and wedge the hotplug worker inside `rt.block_on`, leaving `UINPUT_REFRESH_BUSY` latched true so
///   that every later hotplug refresh is silently skipped for the process lifetime;
/// - records the applied rect and snapshots the per-display layout baseline, which is what arms the
///   #15601 drift remap. Without it the remap never activates on the DRM path at all.
///
#[cfg(feature = "drm")]
pub(super) async fn update_uinput_resolution() {
    if !crate::input_service::wayland_use_uinput() {
        return;
    }
    // Compositor first at a login screen too: a greeter runs one, and the hbb_common socket probe
    // reaches it with no environment variables. The DRM union is used only when no compositor
    // layout is available; DRM has no origins, so its union rect
    // mis-maps the pointer whenever the compositor arranged the outputs side by side.
    //
    // Off the executor: the compositor query can block for the socket probe deadline, and this
    // runs on current-thread runtimes (session init and the hotplug worker). The layout baseline
    // is computed in the SAME task: a failed lookup is not cached, so asking for the rects
    // afterwards would rerun the whole socket probe synchronously.
    //
    // The generation read before the rect is computed labels the range for the DRM cursor
    // calibration; the raw DRM union below is no compositor layout and is labelled with none.
    let gen = scrap::wayland::display::wayland_snapshot_generation();
    let (rect, layout) = match hbb_common::tokio::task::spawn_blocking(|| {
        scrap::wayland::display::clear_wayland_displays_cache();
        match scrap::wayland::display::get_desktop_rect_for_uinput() {
            // The lookup above just cached the displays, so the rects come from that snapshot.
            Some(rect) => Some((
                rect,
                scrap::wayland::display::get_display_rects_for_uinput(),
            )),
            // Raw DRM union: there is no compositor layout to baseline. Empty keeps the #15601
            // remap inactive, which is right when the origins are unknown anyway.
            None => drm_desktop_rect_for_uinput().map(|rect| (rect, Vec::new())),
        }
    })
    .await
    {
        Ok(Some(pair)) => pair,
        Ok(None) => {
            log::warn!("Failed to get desktop rect for uinput");
            return;
        }
        Err(err) => {
            log::warn!("The desktop rect probe task failed: {err}");
            return;
        }
    };
    let gen = super::display_service::input_map_label(gen, !layout.is_empty());
    // Re-snapshot the baseline on every call: this runs at session init and after every hotplug, and
    // the baseline is what the client's coordinates are measured against.
    let snapshot_layout = move || {
        super::display_service::set_wayland_layout_baseline(layout.clone());
    };
    // Reprogram the device only when the range actually changes. A display stuck in a rebuild loop
    // calls this about once a second, and reapplying an identical range is an IPC roundtrip plus a
    // uinput device reconfiguration under a user who may be at the console.
    let job = move |rt: &tokio::runtime::Runtime| {
        if super::display_service::wayland_uinput_rect() == Some(rect) {
            super::display_service::note_input_map_ready(gen);
            snapshot_layout();
            return;
        }
        let (minx, maxx, miny, maxy) = rect;
        log::info!("update mouse resolution: ({minx}, {maxx}), ({miny}, {maxy})");
        super::display_service::note_input_map_unknown();
        match rt.block_on(async {
            timeout(
                3_000,
                input_service::update_mouse_resolution(minx, maxx, miny, maxy),
            )
            .await
        }) {
            // Record the rect only after a successful apply, so a transient failure is retried on
            // the next call instead of being remembered as applied.
            Ok(Ok(())) => {
                super::display_service::set_wayland_uinput_rect(rect);
                super::display_service::note_input_map_adopted(gen);
                snapshot_layout();
            }
            Ok(Err(err)) => log::error!("Failed to update mouse resolution: {}", err),
            Err(err) => log::error!("Failed to update mouse resolution: {}", err),
        }
    };
    if super::display_service::run_uinput_apply(job).await.is_err() {
        hbb_common::throttled_log!(
            super::display_service::UINPUT_APPLY_LOG_INTERVAL,
            error,
            "Failed to update mouse resolution: the uinput apply thread is gone"
        );
    }
}

#[tokio::main(flavor = "current_thread")]
pub(super) async fn ensure_inited() -> ResultType<()> {
    // The DRM/KMS backend owns the Wayland capture stream. It is initialized by the root service
    // and the per-user capturer connects over IPC, so there is no portal session to initialize.
    if is_x11() {
        return Ok(());
    }
    #[cfg(feature = "drm")]
    {
        if !super::drm_capturer::is_available() {
            bail!("Wayland DRM/KMS capture is unavailable")
        }
        update_uinput_resolution().await;
        return Ok(());
    }
    #[cfg(not(feature = "drm"))]
    bail!("Wayland DRM/KMS capture requires the `drm` feature")
}

pub(super) fn is_inited() -> Option<Message> {
    if is_x11() {
        None
    } else {
        #[cfg(feature = "drm")]
        {
            if super::drm_capturer::is_available_cached() {
                return None;
            }
            let mut msg_out = Message::new();
            msg_out.set_message_box(MessageBox {
                msgtype: "nook-nocancel-hasclose".to_owned(),
                title: "Wayland".to_owned(),
                text: "RustDesk could not initialize the Wayland DRM/KMS capture backend."
                    .to_owned(),
                link: "".to_owned(),
                ..Default::default()
            });
            return Some(msg_out);
        }
        #[cfg(not(feature = "drm"))]
        {
            let mut msg_out = Message::new();
            msg_out.set_message_box(MessageBox {
                msgtype: "nook-nocancel-hasclose".to_owned(),
                title: "Wayland".to_owned(),
                text: "RustDesk requires the Wayland DRM/KMS capture backend.".to_owned(),
                link: "".to_owned(),
                ..Default::default()
            });
            Some(msg_out)
        }
    }
}

pub(super) async fn get_displays_and_primary() -> ResultType<(Vec<DisplayInfo>, usize)> {
    #[cfg(feature = "drm")]
    {
        // This function runs once per login (update_get_sync_displays_on_login is its only
        // caller), and login is the moment the client is PROMISED a display list -- so refresh
        // that list over a live `_drm` handshake first. The service wakes sleeping displays and
        // answers with the settled truth, which is what makes an unattended box with an idled,
        // DISABLED panel connectable at all: the cached list would either omit the panel (probed
        // while asleep) or advertise a display with no scanout behind it (probed while awake), and
        // either way the wake then firing inside the capture handshake would change the list the
        // client had already been given. Properly async, so the executor is never blocked; on any
        // failure the cache serves as before.
        super::drm_capturer::refresh_displays_for_login().await;
        let snapshot = hbb_common::tokio::task::spawn_blocking(
            super::drm_capturer::get_display_infos_and_primary,
        )
        .await
        .map_err(|err| anyhow::anyhow!("Wayland display probe task failed: {err}"))?;
        if let Some(snapshot) = snapshot {
            return Ok(snapshot);
        }
        bail!("Wayland DRM/KMS capture returned no displays")
    }

    #[cfg(not(feature = "drm"))]
    bail!("Wayland DRM/KMS capture requires the `drm` feature")
}

pub fn clear() {
    if is_x11() {
        return;
    }
    // DRM augments its geometry from the compositor's Wayland outputs, which scrap caches
    // process-wide. Invalidate that cache on teardown so the next session reads fresh geometry.
    #[cfg(feature = "drm")]
    if super::drm_capturer::is_available_cached() {
        scrap::wayland::display::clear_wayland_displays_cache();
    }
    // NOTE: intentionally do NOT reset the DRM probe cache here. `clear()` runs on every capturer
    // teardown (which happens on each video-service restart), and re-probing `_drm` from the async
    // enumeration path blocks the executor long enough to trip "deadline has elapsed" and spiral
    // into a restart loop. DRM availability is fixed at service start, so the cache stays valid.
}

/// Close the Wayland RemoteDesktop portal session after the last remote client disconnects.
/// The portal still owns the input channel even though video capture now uses DRM/KMS.
pub(super) fn try_close_portal_session() {
    static CLOSER: Mutex<Option<std::thread::JoinHandle<()>>> = Mutex::new(None);
    if matches!(scrap::wayland::pipewire::RDP_SESSION_INFO.try_lock(), Ok(info) if info.is_none()) {
        return;
    }
    let mut closer = CLOSER.lock().unwrap();
    if matches!(&*closer, Some(close) if !close.is_finished()) {
        return;
    }
    match std::thread::Builder::new()
        .name("portal-session-close".into())
        .spawn(|| {
            if !AUTHED_CONNS
                .lock()
                .unwrap()
                .iter()
                .any(|c| c.conn_type == AuthConnType::Remote)
            {
                scrap::wayland::pipewire::try_close_session();
            }
        }) {
        Ok(close) => *closer = Some(close),
        Err(err) => hbb_common::throttled_log!(
            std::time::Duration::from_secs(60),
            error,
            "Failed to spawn the portal session close: {}",
            err
        ),
    }
}

pub(super) fn get_capturer_for_display(
    _display_idx: usize,
) -> ResultType<super::video_service::CapturerInfo> {
    if is_x11() {
        bail!("Do not call this function if not wayland");
    }
    #[cfg(feature = "drm")]
    {
        if !super::drm_capturer::is_available() {
            bail!("Wayland DRM/KMS capture is unavailable");
        }
        return super::drm_capturer::get_capturer_info(_display_idx);
    }

    #[cfg(not(feature = "drm"))]
    bail!("Wayland DRM/KMS capture requires the `drm` feature")
}
