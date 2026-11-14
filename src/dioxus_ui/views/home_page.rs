use super::super::prelude::*;
use super::{connection_panel::ConnectionPanel, peer_browser::PeerBrowser, sidebar::Sidebar};

#[component]
pub(super) fn HomePage() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let logo = rustdesk_icon_data_uri();
    let connected = state.ui_state.read().connected;
    rsx! {
        div { class: "flex min-h-0 min-w-0 flex-1 max-[841px]:flex-col",
            Sidebar { logo }
            if !state.incoming_only {
                if connected {
                    div { class: "min-h-0 min-w-0 flex-1 overflow-hidden bg-[#090a0b]",
                        PeerBrowser {}
                    }
                } else {
                    div { class: "min-h-0 min-w-0 flex-1 overflow-auto bg-rd-content",
                        div { class: "flex min-h-full min-w-0 flex-col px-3 pt-[22px] max-[841px]:h-auto max-[841px]:min-w-0 max-[841px]:px-2.5 max-[841px]:pt-3.5",
                            ConnectionPanel {}
                            PeerBrowser {}
                        }
                    }
                }
            }
        }
    }
}
