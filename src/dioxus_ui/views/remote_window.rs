use super::super::prelude::*;

mod dialog;
pub(in crate::dioxus_ui) mod docking;
mod menus;
mod panels;
mod resolutions;
mod toolbar;

use toolbar::RemoteToolbar;

#[component]
pub(super) fn RemoteWindow() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let session = state.session;
    let status = state.ui_state.read().clone();
    let fullscreen = use_signal(|| false);
    let custom_scale = use_signal(|| {
        PeerConfig::load(&normalize_remote_id(&(state.peer_id)()))
            .ui_flutter
            .get("custom_scale_percent")
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(100)
            .clamp(5, 1000)
    });
    let view_style =
        use_signal(|| PeerConfig::load(&normalize_remote_id(&(state.peer_id)())).view_style);
    use_drop(move || {
        if let Some(active) = session.peek().as_ref() {
            active.close();
        }
        set_current_session(None);
    });

    rsx! {
        document::Title { "{state.peer_id} - RustDesk" }
        div { class: "flex min-h-0 flex-1 flex-col overflow-hidden bg-[#090a0b]",
            if !fullscreen() {
                header { class: "flex h-8 shrink-0 items-center border-b border-[#404040] bg-rd-surface text-xs text-[#ececec] select-none",
                    div {
                        class: "flex h-full min-w-0 max-w-[320px] items-center gap-2 border-r border-b-2 border-r-[#404040] border-b-rd-accent bg-[#303137] px-3",
                        Monitor { class: "h-4 w-4 shrink-0 text-rd-accent" }
                        span { class: "min-w-0 truncate", onmousedown: move |_| dioxus::desktop::window().drag(), "{state.peer_id}" }
                        Button {
                            css_class: "ml-2 h-6 w-6 p-0",
                            title: "Close session", aria_label: "Close session",
                            variant: ButtonVariant::Ghost, size: ButtonSize::IconSm,
                            onclick: move |_| close_remote(session),
                            X { class: "h-3 w-3" }
                        }
                    }
                    div { class: "h-full min-w-6 flex-1", onmousedown: move |_| dioxus::desktop::window().drag() }
                    for (label, icon) in [("Minimize", 0), ("Maximize", 1), ("Close", 2)] {
                        Button {
                            css_class: "h-full w-8 shrink-0 rounded-none p-0 hover:bg-[#3a3b40]",
                            title: label, aria_label: label,
                            variant: ButtonVariant::Ghost, size: ButtonSize::IconSm,
                            onclick: move |_| match icon {
                                0 => dioxus::desktop::window().set_minimized(true),
                                1 => dioxus::desktop::window().toggle_maximized(),
                                _ => close_remote(session),
                            },
                            if icon == 0 { Minus { class: "h-4 w-4" } }
                            else if icon == 1 { Square { class: "h-4 w-4" } }
                            else { X { class: "h-4 w-4" } }
                        }
                    }
                }
            }
            if status.connected {
                ConnectedDesktop { fullscreen, view_style, custom_scale }
            } else {
                main { class: "relative flex min-h-0 flex-1 overflow-hidden",
                    dialog::SessionDialog {}
                }
            }
        }
    }
}

#[component]
fn ConnectedDesktop(
    fullscreen: Signal<bool>,
    view_style: Signal<String>,
    custom_scale: Signal<u16>,
) -> Element {
    let session = use_context::<super::super::state::AppState>().session;
    let dock = use_signal_sync(|| docking::ToolbarDock::load(session.peek().as_ref()));
    rsx! {
        main { id: "remote-toolbar-area", class: "relative grid min-h-0 flex-1 overflow-hidden {dock().edge.anchor_class()}",
            style: "--dock-before: {dock().fraction}fr; --dock-after: {1.0 - dock().fraction}fr",
            super::peer_browser::RemoteDesktop { standalone: true, view_style: view_style(), custom_scale: custom_scale() }
            RemoteToolbar { fullscreen, view_style, custom_scale, dock }
        }
    }
}

fn close_remote(session: Signal<Option<Session<DioxusHandler>>>) {
    if let Some(active) = session.peek().as_ref() {
        active.close();
    }
    dioxus::desktop::window().close();
}
