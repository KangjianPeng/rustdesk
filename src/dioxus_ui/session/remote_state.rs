use super::super::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Default, PartialEq)]
pub(in crate::dioxus_ui) struct RemoteUiState {
    pub resolutions: Vec<Resolution>,
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
    fn resolution_choices_follow_the_peer_and_current_monitor() {
        fn fixture() -> Element {
            let state = use_signal_sync(DioxusUiState::default);
            let handler = DioxusHandler::new(state);
            let resolution = |width, height| Resolution {
                width,
                height,
                ..Default::default()
            };
            handler.set_peer_info(&PeerInfo {
                displays: vec![
                    DisplayInfo::default(),
                    DisplayInfo {
                        scale: 1.5,
                        ..Default::default()
                    },
                ],
                resolutions: Some(SupportedResolutions {
                    resolutions: vec![resolution(1920, 1080), resolution(1280, 720)],
                    ..Default::default()
                })
                .into(),
                ..Default::default()
            });
            assert_eq!(handler.snapshot().unwrap().remote.resolutions.len(), 2);
            handler.switch_display(&SwitchDisplay {
                display: 1,
                width: 2560,
                height: 1440,
                original_resolution: Some(resolution(1920, 1080)).into(),
                resolutions: Some(SupportedResolutions {
                    resolutions: vec![resolution(2560, 1440)],
                    ..Default::default()
                })
                .into(),
                ..Default::default()
            });
            let snapshot = handler.snapshot().unwrap();
            assert_eq!(snapshot.remote.resolutions, vec![resolution(2560, 1440)]);
            assert_eq!(snapshot.displays[1].original_resolution.width, 1920);
            assert_eq!(snapshot.displays[1].scale, 1.5);
            handler.switch_display(&SwitchDisplay {
                display: 0,
                ..Default::default()
            });
            assert!(handler.snapshot().unwrap().remote.resolutions.is_empty());
            rsx! { div {} }
        }
        VirtualDom::new(fixture).rebuild_in_place();
    }

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
