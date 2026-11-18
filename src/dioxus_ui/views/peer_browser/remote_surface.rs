use super::super::super::super::prelude::*;
use crate::dioxus_ui::desktop::{NativeVideoSurface, Viewport};

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum SurfaceEvent {
    Layout { viewport: Viewport },
    Input(crate::dioxus_ui::session::BrowserInput),
}

#[component]
pub(super) fn RemoteSurface(original: bool, scale: u16) -> Element {
    let state = use_context::<crate::dioxus_ui::state::AppState>();
    let status = state.ui_state.read();
    let width = status.width.max(1) as u32 * scale as u32 / 100;
    let height = status.height.max(1) as u32 * scale as u32 / 100;
    let mut failure = use_signal(String::new);
    rsx! {
        div {
            id: "remote-native-surface", role: "application", aria_label: "Remote desktop", tabindex: "0",
            "data-width": "{status.width.max(1)}", "data-height": "{status.height.max(1)}", "data-original": "{original}",
            style: "--remote-width:{width}px;--remote-height:{height}px",
            class: if original { "m-auto block h-[var(--remote-height)] w-[var(--remote-width)] shrink-0 touch-none select-none" } else { "m-auto block h-full min-h-0 w-full min-w-0 shrink-0 touch-none select-none" },
            onmounted: move |_| {
                let Some(active) = state.session.peek().as_ref().cloned() else { return; };
                let surface = match NativeVideoSurface::mount(active.ui_handler.clone()) {
                    Ok(surface) => surface,
                    Err(error) => {
                        log::error!("Cannot mount native video: {error}");
                        failure.set(error.to_string());
                        return;
                    }
                };
                let mut eval = document::eval(include_str!("remote_surface.js"));
                if let Err(error) = eval.send(crate::dioxus_ui::desktop::session_url()) {
                    failure.set(error.to_string());
                    return;
                }
                spawn(async move {
                    while let Ok(event) = eval.recv::<SurfaceEvent>().await {
                        match event {
                            SurfaceEvent::Layout { viewport } => surface.set_viewport(viewport),
                            SurfaceEvent::Input(input) => crate::dioxus_ui::session::send_browser_input(input,&active),
                        }
                    }
                });
            },
            if !failure().is_empty() { div { class: "m-auto p-4 text-sm text-destructive", "{failure}" } }
        }
    }
}
