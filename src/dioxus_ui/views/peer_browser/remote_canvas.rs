use super::super::super::super::prelude::*;

#[component]
pub(super) fn RemoteCanvas(original: bool, scale: u16) -> Element {
    let state = use_context::<super::super::super::super::state::AppState>();
    let session = state.session;
    let status = state.ui_state.read();
    let id = dioxus::native::use_wgpu(move || {
        session
            .peek()
            .as_ref()
            .and_then(|session| session.ui_handler.native_video())
            .unwrap_or_default()
    });
    let scale = scale.min(crate::dioxus_ui::views::remote_window::maximum_scale(
        status.width,
        status.height,
        crate::dioxus_ui::native::window().scale_factor(),
    ));
    let width = status.width.max(1) as u32 * scale as u32 / 100;
    let height = status.height.max(1) as u32 * scale as u32 / 100;
    rsx! {
        canvas {
            id: "remote-desktop", "src": "{id}", aria_label: "Remote desktop", tabindex: "0",
            style: "--remote-aspect: {status.width.max(1)} / {status.height.max(1)}; --remote-width: {width}px; --remote-height: {height}px",
            width: "{status.width}", height: "{status.height}",
            class: if original { "m-auto block h-[var(--remote-height)] w-[var(--remote-width)] max-h-none max-w-none shrink-0 aspect-[var(--remote-aspect)] bg-[#090a0b]" } else { "m-auto block h-auto max-h-full w-auto max-w-full shrink-0 aspect-[var(--remote-aspect)] bg-[#090a0b] object-contain" },
        }
    }
}
