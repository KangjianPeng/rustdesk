use super::prelude::*;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod cursor;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod cursor_texture;
mod frame;
mod input;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod remote_state;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(in crate::dioxus_ui) use remote_state::{ChatMessage, RemoteUiState};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod video;

#[cfg(test)]
pub(crate) use frame::encode_frame;
#[cfg(any(target_os = "android", target_os = "ios"))]
use frame::frame_worker;
use frame::{FramePacket, FramePipeline};
#[cfg(test)]
pub(crate) use input::remote_point;
pub(crate) use input::{send_keyboard_event, send_mouse_event, send_touch_event, send_wheel_event};

#[derive(Clone, Default)]
pub(crate) struct DioxusHandler {
    state: Option<SyncSignal<DioxusUiState>>,
    frames: Option<Arc<FramePipeline>>,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub(in crate::dioxus_ui) cursors: Arc<Mutex<cursor::RemoteCursorState>>,
}

lazy_static::lazy_static! {
    static ref CURRENT_SESSION: Mutex<Option<Session<DioxusHandler>>> = Mutex::new(None);
}

pub(crate) fn current_session() -> Option<Session<DioxusHandler>> {
    CURRENT_SESSION
        .lock()
        .ok()
        .and_then(|session| session.clone())
}

pub(super) fn set_current_session(session: Option<Session<DioxusHandler>>) {
    if let Ok(mut current) = CURRENT_SESSION.lock() {
        *current = session;
    }
}

impl DioxusHandler {
    fn new(state: SyncSignal<DioxusUiState>) -> Self {
        let frames = Arc::new(FramePipeline::default());
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let worker_frames = Arc::downgrade(&frames);
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let worker_state = state;
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let frames = match std::thread::Builder::new()
            .name("rustdesk-dioxus-render".to_owned())
            .spawn(move || frame_worker(worker_frames, worker_state))
        {
            Ok(_) => Some(frames),
            Err(error) => {
                log::error!("Failed to start Dioxus frame renderer: {error}");
                None
            }
        };
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let frames = Some(frames);
        Self {
            state: Some(state),
            frames,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            cursors: Default::default(),
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub(in crate::dioxus_ui) fn native_video(&self) -> Option<frame::NativeVideo> {
        self.frames
            .as_ref()
            .map(|frames| frame::NativeVideo::new(frames.clone()))
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub(in crate::dioxus_ui) fn snapshot(&self) -> Option<DioxusUiState> {
        self.state?.try_peek().ok().map(|state| state.clone())
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn update_dimensions(&self, width: i32, height: i32) {
        if self.state.is_some_and(|state| {
            state
                .try_peek()
                .is_ok_and(|state| state.width == width && state.height == height)
        }) {
            return;
        }
        self.update(|state| {
            state.width = width;
            state.height = height;
        });
    }

    fn update(&self, update: impl FnOnce(&mut DioxusUiState)) {
        if let Some(mut state) = self.state {
            if let Ok(mut state) = state.try_write() {
                update(&mut state);
            }
        }
    }
}

impl InvokeUiSession for DioxusHandler {
    fn set_cursor_data(&self, _cd: CursorData) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            self.cursors.lock().unwrap().store(_cd);
            crate::dioxus_ui::native::request_redraw();
        }
    }

    fn set_cursor_id(&self, _id: String) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            self.cursors.lock().unwrap().select(&_id);
            crate::dioxus_ui::native::request_redraw();
        }
    }

    fn set_cursor_position(&self, _cp: CursorPosition) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            self.cursors.lock().unwrap().position = Some((_cp.x, _cp.y));
            crate::dioxus_ui::native::request_redraw();
        }
    }

    fn set_display(&self, _x: i32, _y: i32, w: i32, h: i32, _cursor_embedded: bool, _scale: f64) {
        self.update(|state| {
            state.width = w;
            state.height = h;
        });
    }

    fn switch_display(&self, display: &SwitchDisplay) {
        self.update(|state| {
            state.current_display = display.display;
            state.width = display.width;
            state.height = display.height;
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let Some(info) = state.displays.get_mut(display.display.max(0) as usize) {
                info.x = display.x;
                info.y = display.y;
                info.width = display.width;
                info.height = display.height;
                info.cursor_embedded = display.cursor_embedded;
            }
            state.viewport_width = 0.0;
            state.viewport_height = 0.0;
        });
    }

    fn set_peer_info(&self, peer_info: &PeerInfo) {
        self.update(|state| {
            state.peer_name = peer_info.username.clone();
            state.displays = peer_info.displays.clone();
            state.current_display = peer_info.current_display;
        });
    }

    fn set_displays(&self, displays: &Vec<DisplayInfo>) {
        self.update(|state| state.displays = displays.clone());
    }

    fn set_platform_additions(&self, _data: &str) {}

    fn on_connected(&self, conn_type: ConnType) {
        if conn_type == ConnType::DEFAULT_CONN {
            crate::keyboard::client::start_grab_loop();
        }
        self.update(|state| {
            state.status = "Connected".to_owned();
            state.message.clear();
            state.connected = true;
            state.retry_available = false;
            state.password_required = false;
            state.two_factor_required = false;
        });
    }

    fn update_privacy_mode(&self) {}

    fn set_permission(&self, name: &str, value: bool) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        self.update(|state| {
            Arc::make_mut(&mut state.remote)
                .permissions
                .insert(name.to_owned(), value);
        });
    }

    fn close_success(&self) {
        self.update(|state| {
            state.message.clear();
        });
    }

    fn update_quality_status(&self, status: QualityStatus) {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        if let Some(fps) = status.fps.values().next() {
            self.update(|state| state.message = format!("{fps} FPS"));
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        self.update(|state| {
            let remote = Arc::make_mut(&mut state.remote);
            if let Some(value) = status.speed {
                remote.speed = value;
            }
            remote.fps.extend(status.fps);
            if let Some(value) = status.delay {
                remote.delay = Some(value);
            }
            if let Some(value) = status.target_bitrate {
                remote.bitrate = Some(value);
            }
            if let Some(value) = status.codec_format {
                remote.codec = format!("{value:?}");
            }
            if let Some(value) = status.chroma {
                remote.chroma = value;
            }
        });
    }

    fn set_connection_type(&self, is_secured: bool, direct: bool, stream_type: &str) {
        self.update(|state| {
            let connection = format!(
                "{} · {} · {}",
                if is_secured { "Secure" } else { "Not secure" },
                if direct { "Direct" } else { "Relay" },
                stream_type
            );
            #[cfg(any(target_os = "android", target_os = "ios"))]
            {
                state.message = connection;
            }
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                Arc::make_mut(&mut state.remote).connection = connection;
            }
        });
    }

    fn set_fingerprint(&self, _fingerprint: String) {}

    fn job_error(&self, _id: i32, _err: String, _file_num: i32) {}

    fn job_done(&self, _id: i32, _file_num: i32) {}

    fn clear_all_jobs(&self) {}

    fn new_message(&self, msg: String) {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        self.update(|state| state.message = msg);
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        self.update(|state| {
            Arc::make_mut(&mut Arc::make_mut(&mut state.remote).chat).push(ChatMessage {
                outgoing: false,
                text: msg,
            });
        });
    }

    fn update_transfer_list(&self) {}

    fn load_last_job(&self, _cnt: i32, _job_json: &str, _auto_start: bool) {}

    fn update_folder_files(
        &self,
        _id: i32,
        _entries: &Vec<FileEntry>,
        _path: String,
        _is_local: bool,
        _only_count: bool,
    ) {
    }

    fn confirm_delete_files(&self, _id: i32, _i: i32, _name: String) {}

    fn override_file_confirm(
        &self,
        _id: i32,
        _file_num: i32,
        _to: String,
        _is_upload: bool,
        _is_identical: bool,
    ) {
    }

    fn update_block_input_state(&self, _on: bool) {}

    fn job_progress(&self, _id: i32, _file_num: i32, _speed: f64, _finished_size: f64) {}

    fn adapt_size(&self) {}

    fn on_rgba(&self, display: usize, rgba: &mut scrap::ImageRgb) {
        if let Some(state) = self.state {
            let Ok(state) = state.try_peek() else {
                return;
            };
            let current_display = state.current_display.max(0) as usize;
            if display != current_display {
                return;
            }
        }
        if let Some(frames) = &self.frames {
            let recycled = frames
                .recycled
                .lock()
                .ok()
                .and_then(|mut buffers| buffers.pop());
            let raw = std::mem::replace(&mut rgba.raw, recycled.unwrap_or_default());
            let packet = FramePacket {
                width: rgba.w,
                height: rgba.h,
                format: rgba.fmt,
                align: rgba.align(),
                raw,
            };
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                self.update_dimensions(rgba.w as i32, rgba.h as i32);
                frames.submit(packet);
            }
            #[cfg(any(target_os = "android", target_os = "ios"))]
            if let Ok(mut pending) = frames.pending.lock() {
                if let Some(previous) = pending.replace(packet) {
                    if let Ok(mut recycled) = frames.recycled.lock() {
                        recycled.push(previous.raw);
                    }
                }
                frames.wake.notify_one();
            }
            return;
        }
        self.update(|state| {
            state.width = rgba.w as i32;
            state.height = rgba.h as i32;
        });
    }

    fn msgbox(&self, msgtype: &str, title: &str, text: &str, _link: &str, retry: bool) {
        self.update(|state| {
            state.message = if text.is_empty() {
                title.to_owned()
            } else {
                format!("{title}: {text}")
            };
            match msgtype {
                "input-password" | "re-input-password" => {
                    state.password_required = true;
                    state.two_factor_required = false;
                }
                "input-2fa" => {
                    state.password_required = false;
                    state.two_factor_required = true;
                }
                "error" | "custom-error" => {
                    state.status = "Connection error".to_owned();
                    state.connected = false;
                    state.retry_available = retry;
                }
                _ => {}
            }
        });
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn clipboard(&self, _content: String) {}

    fn cancel_msgbox(&self, _tag: &str) {}

    fn switch_back(&self, _id: &str) {}

    fn portable_service_running(&self, _running: bool) {}

    fn on_voice_call_started(&self) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        self.update(|state| {
            Arc::make_mut(&mut state.remote).voice_call = "Voice call connected".to_owned()
        });
    }

    fn on_voice_call_closed(&self, _reason: &str) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        self.update(|state| Arc::make_mut(&mut state.remote).voice_call.clear());
    }

    fn on_voice_call_waiting(&self) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        self.update(|state| {
            Arc::make_mut(&mut state.remote).voice_call =
                "Waiting for voice call response".to_owned()
        });
    }

    fn on_voice_call_incoming(&self) {}

    fn get_rgba(&self, _display: usize) -> *const u8 {
        std::ptr::null()
    }

    fn next_rgba(&self, _display: usize) {}

    fn set_multiple_windows_session(&self, _sessions: Vec<WindowsSession>) {}

    fn set_current_display(&self, disp_idx: i32) {
        self.update(|state| state.current_display = disp_idx);
    }

    fn update_record_status(&self, start: bool) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        self.update(|state| Arc::make_mut(&mut state.remote).recording = start);
    }

    fn printer_request(&self, _id: i32, _path: String) {}

    fn handle_screenshot_resp(&self, _sid: String, _msg: String) {}

    fn handle_terminal_response(&self, _response: TerminalResponse) {}
}
pub(super) fn new_session(
    peer_id: String,
    password: String,
    args: Vec<String>,
    state: SyncSignal<DioxusUiState>,
) -> Session<DioxusHandler> {
    let force_relay = args.iter().any(|arg| arg == "--relay");
    let conn_type = match args.first().map(String::as_str) {
        Some("--file-transfer") => ConnType::FILE_TRANSFER,
        Some("--view-camera") => ConnType::VIEW_CAMERA,
        Some("--port-forward") => ConnType::PORT_FORWARD,
        Some("--terminal") => ConnType::TERMINAL,
        Some("--rdp") => ConnType::RDP,
        _ => ConnType::DEFAULT_CONN,
    };
    let session = Session {
        password,
        args,
        ui_handler: DioxusHandler::new(state),
        server_keyboard_enabled: Arc::new(std::sync::RwLock::new(true)),
        server_file_transfer_enabled: Arc::new(std::sync::RwLock::new(true)),
        server_clipboard_enabled: Arc::new(std::sync::RwLock::new(true)),
        reconnect_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        ..Default::default()
    };
    match session.lc.write() {
        Ok(mut lc) => lc.initialize(peer_id, conn_type, None, force_relay, None, None, None),
        Err(error) => log::error!("Failed to initialize Dioxus session: {error}"),
    }
    session
}

pub(super) fn start_session(
    mut session: Signal<Option<Session<DioxusHandler>>>,
    peer_id: String,
    password: String,
    args: Vec<String>,
    state: SyncSignal<DioxusUiState>,
) {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if args
        .first()
        .is_none_or(|arg| matches!(arg.as_str(), "--connect" | "--view-camera" | "--relay"))
    {
        super::runtime::open_remote_window(peer_id, password, args, state);
        return;
    }
    if peer_id.trim().is_empty() || session.read().is_some() {
        return;
    }
    let peer_id = normalize_remote_id(peer_id.trim());
    remember_remote_id(&peer_id);
    let new_session = new_session(peer_id, password, args, state);
    set_current_session(Some(new_session.clone()));
    session.set(Some(new_session.clone()));
    update_ui_state(state, |state| {
        state.status = "Connecting".to_owned();
        state.message.clear();
        state.connected = false;
    });
    new_session.reconnect(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_callbacks_after_window_close_do_not_panic() {
        fn closing_view() -> Element {
            let state = use_signal_sync(DioxusUiState::default);
            let handler = use_context::<Arc<Mutex<Option<DioxusHandler>>>>();
            use_hook(move || {
                *handler.lock().unwrap() = Some(DioxusHandler {
                    state: Some(state),
                    ..Default::default()
                });
            });
            rsx! { div {} }
        }

        let handler = Arc::new(Mutex::new(None::<DioxusHandler>));
        let mut dom = VirtualDom::new(closing_view);
        dom.provide_root_context(handler.clone());
        dom.rebuild_to_vec();
        drop(dom);

        let handler = handler.lock().unwrap().take().unwrap();
        handler.on_rgba(0, &mut scrap::ImageRgb::new(ImageFormat::ARGB, 1));
        handler.close_success();
    }

    #[test]
    fn first_video_frame_preserves_connected_status() {
        fn status_view() -> Element {
            let state = use_signal_sync(DioxusUiState::default);
            use_hook(|| {
                let handler = DioxusHandler {
                    state: Some(state),
                    ..Default::default()
                };
                handler.on_connected(ConnType::VIEW_CAMERA);
                handler.close_success();
            });
            rsx! { p { "{state.read().status}" } }
        }

        let edits = VirtualDom::new(status_view).rebuild_to_vec();
        assert!(edits.edits.iter().any(|edit| matches!(
            edit,
            dioxus::dioxus_core::Mutation::CreateTextNode { value, .. } if value == "Connected"
        )));
    }
}
