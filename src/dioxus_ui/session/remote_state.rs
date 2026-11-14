use super::super::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Default, PartialEq)]
pub(in crate::dioxus_ui) struct RemoteUiState {
    pub permissions: HashMap<String, bool>,
    pub chat: Arc<Vec<ChatMessage>>,
    pub recording: bool,
    pub voice_call: String,
    pub connection: String,
    pub speed: String,
    pub fps: HashMap<usize, i32>,
    pub delay: Option<i32>,
    pub bitrate: Option<i32>,
    pub codec: String,
    pub chroma: String,
}

#[derive(Clone, PartialEq)]
pub(in crate::dioxus_ui) struct ChatMessage {
    pub outgoing: bool,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::super::DioxusHandler;
    use super::*;

    #[test]
    fn session_events_preserve_chat_and_partial_quality_updates() {
        fn fixture() -> Element {
            let state = use_signal_sync(|| DioxusUiState {
                message: "Connection status".to_owned(),
                ..Default::default()
            });
            let handler = DioxusHandler::new(state);
            handler.set_connection_type(true, true, "TCP");
            handler.update_quality_status(QualityStatus {
                codec_format: Some(scrap::CodecFormat::H264),
                delay: Some(12),
                ..Default::default()
            });
            handler.update_quality_status(QualityStatus {
                fps: [(0, 60)].into_iter().collect(),
                ..Default::default()
            });
            handler.new_message("Hello".to_owned());
            handler.update_record_status(true);
            handler.set_permission("clipboard", false);
            let state = state.read();
            assert_eq!(state.message, "Connection status");
            assert_eq!(state.remote.codec, "H264");
            assert_eq!(state.remote.delay, Some(12));
            assert_eq!(state.remote.fps.get(&0), Some(&60));
            assert_eq!(state.remote.connection, "Secure · Direct · TCP");
            assert_eq!(state.remote.chat[0].text, "Hello");
            assert!(!state.remote.chat[0].outgoing);
            assert!(state.remote.recording);
            assert_eq!(state.remote.permissions.get("clipboard"), Some(&false));
            rsx! { div {} }
        }
        VirtualDom::new(fixture).rebuild_in_place();
    }
}
