use super::super::super::super::prelude::*;

#[component]
pub(super) fn RemoteCanvas(original: bool, scale: u16) -> Element {
    let state = use_context::<super::super::super::super::state::AppState>();
    let status = state.ui_state.read();
    let width = status.width.max(1) as u32 * scale as u32 / 100;
    let height = status.height.max(1) as u32 * scale as u32 / 100;
    rsx! {
        canvas {
            id: "remote-desktop", aria_label: "Remote desktop", tabindex: "0",
            style: "--remote-aspect: {status.width.max(1)} / {status.height.max(1)}; --remote-width: {width}px; --remote-height: {height}px",
            width: "{status.width.max(1)}", height: "{status.height.max(1)}",
            class: if original { "m-auto block h-[var(--remote-height)] w-[var(--remote-width)] max-h-none max-w-none shrink-0 aspect-[var(--remote-aspect)] bg-[#090a0b] touch-none select-none" } else { "m-auto block h-auto max-h-full w-auto max-w-full shrink-0 aspect-[var(--remote-aspect)] bg-[#090a0b] object-contain touch-none select-none" },
            onmounted: move |_| {
                let mut eval = document::eval(include_str!("remote_canvas.js"));
                if let Err(error) = eval.send(crate::dioxus_ui::desktop::session_url()) {
                    log::error!("Cannot initialize remote canvas: {error}");
                    return;
                }
                spawn(async move {
                    loop {
                        match eval.recv::<crate::dioxus_ui::session::BrowserInput>().await {
                            Ok(input) => {
                                if let Some(active) = state.session.peek().as_ref() {
                                    crate::dioxus_ui::session::send_browser_input(input, active);
                                }
                            }
                            Err(error) => {
                                log::trace!("Remote canvas closed: {error}");
                                break;
                            }
                        }
                    }
                });
            },
        }
    }
}
