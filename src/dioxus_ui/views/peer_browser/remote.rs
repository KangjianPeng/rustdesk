use super::super::super::prelude::*;

#[cfg(not(any(target_os = "android", target_os = "ios", target_os = "linux")))]
#[path = "remote_canvas.rs"]
mod remote_canvas;
#[cfg(target_os = "linux")]
#[path = "remote_surface.rs"]
mod remote_surface;

#[component]
pub(in crate::dioxus_ui::views) fn RemoteDesktop(
    #[props(default)] standalone: bool,
    #[props(default = "adaptive".to_owned())] view_style: String,
    #[props(default = 100)] custom_scale: u16,
) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let mut session = state.session;
    let ui_state = state.ui_state;
    let status = ui_state.read().clone();
    let mouse_down_state = status.clone();
    let mouse_up_state = status.clone();
    let mouse_move_state = status.clone();
    let touch_start_state = status.clone();
    let touch_move_state = status.clone();
    let touch_end_state = status.clone();
    let touch_cancel_state = status.clone();
    rsx! {
        div { class: "flex h-full min-h-0 min-w-0 flex-1 flex-col bg-[#090a0b]",
            class: if standalone { "absolute inset-0" } else { "" },
            if !standalone {
            header { class: "flex min-h-10 shrink-0 items-center gap-3 border-b border-[#303238] bg-rd-surface px-3 text-sm text-[#e8e9eb]",
                span { class: "min-w-0 flex-1 overflow-hidden text-ellipsis whitespace-nowrap", if status.peer_name.is_empty() { {tr("Remote desktop")} } else { "{status.peer_name}" } }
                if !status.message.is_empty() {
                    span { class: "hidden max-w-[35%] overflow-hidden text-ellipsis whitespace-nowrap text-xs text-rd-muted min-[841px]:inline", "{status.message}" }
                }
                Button {
                    css_class: "h-7 rounded-md bg-[#8e4650] px-3 text-xs text-white hover:bg-[#a95662]",
                    variant: ButtonVariant::Destructive,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        if let Some(active) = session.read().as_ref() {
                            active.close();
                        }
                        set_current_session(None);
                        session.set(None);
                        update_ui_state(ui_state, |state| {
                            state.connected = false;
                            state.status = "Disconnected".to_owned();
                            state.message.clear();
                            state.frame_data_uri = Arc::from("");
                        });
                    },
                    {tr("Disconnect")}
                }
            }
            }
            div { class: if standalone { "flex min-h-0 flex-1 overflow-auto" } else { "flex min-h-0 flex-1 items-center justify-center overflow-auto p-3" },
                if cfg!(not(any(target_os = "android", target_os = "ios"))) {
                    {desktop_canvas(standalone && matches!(view_style.as_str(), "original" | "custom"), if standalone && view_style == "custom" { custom_scale } else { 100 })}
                } else if status.frame_data_uri.is_empty() {
                    div { class: "m-auto flex flex-col items-center gap-2 text-sm text-rd-muted",
                        span { class: "h-5 w-5 animate-spin rounded-full border-2 border-[#5d91e8] border-t-transparent" }
                        span { {tr("Connected, waiting for image...")} }
                    }
                } else {
                    img {
                        class: if standalone && view_style == "original" { "m-auto block max-h-none max-w-none shrink-0 bg-[#090a0b] [touch-action:none] [user-select:none]" } else if standalone { "m-auto block h-auto max-h-full w-auto max-w-full shrink-0 bg-[#090a0b] object-contain [touch-action:none] [user-select:none]" } else { "block h-auto max-h-full w-auto max-w-full rounded-[3px] border border-[#414348] bg-[#090a0b] object-contain [touch-action:none] [user-select:none]" },
                        src: "{status.frame_data_uri}",
                        width: "{status.width}",
                        height: "{status.height}",
                        alt: "Remote desktop",
                        tabindex: "0",
                        onmounted: move |event| async move {
                            if let Ok(rect) = event.get_client_rect().await {
                                update_ui_state(ui_state, |state| {
                                    state.viewport_left = rect.origin.x;
                                    state.viewport_top = rect.origin.y;
                                    state.viewport_width = rect.width();
                                    state.viewport_height = rect.height();
                                });
                            }
                        },
                        onresize: move |event| {
                            if let Ok(size) = event.get_border_box_size() {
                                update_ui_state(ui_state, |state| {
                                    state.viewport_width = size.width;
                                    state.viewport_height = size.height;
                                });
                            }
                        },
                        onmousedown: move |event| if let Some(active) = session.read().as_ref() { send_mouse_event(&event.data, 1, active, &mouse_down_state); },
                        onmouseup: move |event| if let Some(active) = session.read().as_ref() { send_mouse_event(&event.data, 2, active, &mouse_up_state); },
                        onmousemove: move |event| if let Some(active) = session.read().as_ref() { send_mouse_event(&event.data, 0, active, &mouse_move_state); },
                        onwheel: move |event| {
                            event.prevent_default();
                            if let Some(active) = session.read().as_ref() { send_wheel_event(&event.data, active); }
                        },
                        onkeydown: move |event| {
                            event.prevent_default();
                            if let Some(active) = session.read().as_ref() { send_keyboard_event(&event.data, true, active); }
                        },
                        onkeyup: move |event| {
                            event.prevent_default();
                            if let Some(active) = session.read().as_ref() { send_keyboard_event(&event.data, false, active); }
                        },
                        ontouchstart: move |event| {
                            event.prevent_default();
                            if let Some(active) = session.read().as_ref() { send_touch_event(&event.data, 1, active, &touch_start_state); }
                        },
                        ontouchmove: move |event| {
                            event.prevent_default();
                            if let Some(active) = session.read().as_ref() { send_touch_event(&event.data, 0, active, &touch_move_state); }
                        },
                        ontouchend: move |event| {
                            event.prevent_default();
                            if let Some(active) = session.read().as_ref() { send_touch_event(&event.data, 2, active, &touch_end_state); }
                        },
                        ontouchcancel: move |event| {
                            event.prevent_default();
                            if let Some(active) = session.read().as_ref() { send_touch_event(&event.data, 2, active, &touch_cancel_state); }
                        },
                    }
                }
            }
            if !standalone && status.displays.len() > 1 {
                div { class: "flex shrink-0 flex-wrap gap-[7px] border-t border-[#303238] bg-rd-surface p-2",
                    for (index, display) in status.displays.iter().enumerate() {
                        Button {
                            css_class: if status.current_display == index as i32 { "min-h-[30px] rounded border border-[#6696df] bg-[#36588e] px-2.5 text-[11px] text-white" } else { "min-h-[30px] rounded border border-[#4b4d52] bg-[#303135] px-2.5 text-[11px] text-[#c8c9cb]" },
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            disabled: !display.online,
                            onclick: move |_| if let Some(active) = session.read().as_ref() { active.switch_display(index as i32); },
                            "Display {index + 1} · {display.width} × {display.height}"
                        }
                    }
                }
            }
        }
    }
}

fn desktop_canvas(original: bool, scale: u16) -> Element {
    #[cfg(target_os = "linux")]
    return rsx! { remote_surface::RemoteSurface { original, scale } };
    #[cfg(not(any(target_os = "android", target_os = "ios", target_os = "linux")))]
    return rsx! { remote_canvas::RemoteCanvas { original, scale } };
    #[cfg(any(target_os = "android", target_os = "ios"))]
    rsx! {}
}
