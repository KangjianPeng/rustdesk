use super::super::prelude::*;
use super::{connection_panel::ConnectionPanel, peer_browser::PeerBrowser, sidebar::Sidebar};

#[component]
pub(super) fn HomePage() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let connected = state.ui_state.read().connected;
    let mut suggestions_open = use_signal(|| false);
    let mut peer_menu = use_signal(String::new);
    rsx! {
        div { class: "flex min-h-0 min-w-0 flex-1",
            onmousedown: move |_| {
                suggestions_open.set(false);
                peer_menu.set(String::new());
            },
            Sidebar {}
            if !state.incoming_only {
                if connected {
                    div { class: "min-h-0 min-w-0 flex-1 overflow-hidden bg-[#090a0b]",
                        PeerBrowser { peer_menu }
                    }
                } else {
                    div { class: "flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-background",
                        div { class: "flex min-h-0 w-full min-w-0 flex-1 flex-col px-4 pt-4",
                            ConnectionPanel { suggestions_open }
                            div { class: "my-4 shrink-0 border-t" }
                            PeerBrowser { peer_menu }
                        }
                        if !state.outgoing_only {
                            div { class: "shrink-0",
                                ServiceStatus { status: (state.service_status)() }
                            }
                        }
                    }
                }
            }
        }
    }
}
