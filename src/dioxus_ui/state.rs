use super::{
    common::{CodecUiState, DioxusUiState, ServiceStatusUi},
    peers::PeerCardData,
    DioxusHandler,
};
use crate::ui_session_interface::Session;
use dioxus::prelude::{Signal, SyncSignal};

#[derive(Clone, Copy)]
pub(super) struct AppState {
    pub(super) active_page: Signal<String>,
    pub(super) peer_id: Signal<String>,
    pub(super) password: Signal<String>,
    pub(super) two_factor_code: Signal<String>,
    pub(super) remember_password: Signal<bool>,
    pub(super) connection_mode: Signal<String>,
    pub(super) connection_menu_open: Signal<bool>,
    pub(super) peer_tab: Signal<String>,
    pub(super) selected_peer: Signal<String>,
    pub(super) peer_search: Signal<bool>,
    pub(super) peer_search_text: Signal<String>,
    pub(super) peer_view_mode: Signal<String>,
    pub(super) peer_sort: Signal<String>,
    #[cfg(target_os = "android")]
    pub(super) mobile_chat_id: Signal<i32>,
    pub(super) peer_refresh: Signal<u64>,
    pub(super) peer_cards: Signal<Vec<PeerCardData>>,
    pub(super) session: Signal<Option<Session<DioxusHandler>>>,
    pub(super) ui_state: SyncSignal<DioxusUiState>,
    pub(super) codec_state: SyncSignal<CodecUiState>,
    pub(super) service_status: Signal<ServiceStatusUi>,
    pub(super) local_device_id: Signal<String>,
    pub(super) temporary_password: Signal<String>,
    pub(super) hardware_codec: Signal<bool>,
    pub(super) has_hardware_codec: bool,
    pub(super) incoming_only: bool,
    pub(super) outgoing_only: bool,
    pub(super) help_card_dismissed: Signal<bool>,
}
