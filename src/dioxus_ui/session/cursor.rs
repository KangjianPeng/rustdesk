use super::super::prelude::*;
use std::collections::HashMap;

#[derive(serde::Serialize)]
struct CursorShape {
    id: String,
    uri: String,
    width: i32,
    height: i32,
    hotx: i32,
    hoty: i32,
}

#[derive(Default)]
pub(in crate::dioxus_ui) struct RemoteCursorState {
    shapes: HashMap<u64, CursorShape>,
    selected: Option<u64>,
    pub position: Option<(i32, i32)>,
}

impl RemoteCursorState {
    pub fn store(&mut self, data: CursorData) {
        let Some(uri) = super::frame::encode_frame(
            data.width.max(0) as usize,
            data.height.max(0) as usize,
            ImageFormat::ABGR,
            1,
            &data.colors,
        ) else {
            hbb_common::throttled_log!(
                Duration::from_secs(30),
                warn,
                "Invalid remote cursor image"
            );
            return;
        };
        self.selected = Some(data.id);
        self.shapes.insert(
            data.id,
            CursorShape {
                id: data.id.to_string(),
                uri,
                width: data.width,
                height: data.height,
                hotx: data.hotx,
                hoty: data.hoty,
            },
        );
    }

    pub fn select(&mut self, id: &str) {
        self.selected = id.parse().ok();
    }
}

impl super::DioxusHandler {
    pub(in crate::dioxus_ui) fn cursor_snapshot(
        &self,
        session: &Session<Self>,
    ) -> Result<Vec<u8>, serde_json::Error> {
        let status = self.read_state();
        let display = status
            .as_ref()
            .and_then(|state| state.displays.get(state.current_display.max(0) as usize));
        let cursors = self.cursors.lock().unwrap();
        let shape = cursors.selected.and_then(|id| cursors.shapes.get(&id));
        serde_json::to_vec(&serde_json::json!({
            "shape": shape,
            "position": cursors.position,
            "origin": [display.map_or(0, |d| d.x), display.map_or(0, |d| d.y)],
            "embedded": display.is_some_and(|d| d.cursor_embedded),
            "show": session.get_toggle_option("show-remote-cursor".to_owned()),
            "zoom": session.get_toggle_option("zoom-cursor".to_owned()) || session.get_view_style() == "original",
            "blocked": session.is_view_camera() || session.get_toggle_option("view-only".to_owned()) || !*session.server_keyboard_enabled.read().unwrap(),
        }))
    }
}
