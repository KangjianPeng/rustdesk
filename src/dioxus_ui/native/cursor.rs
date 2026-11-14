use super::*;
use std::collections::HashMap;
use winit::window::{CursorIcon, CustomCursor};

#[derive(Default)]
pub(super) struct RemoteCursor {
    cached: HashMap<u64, (Arc<CursorData>, u16, u16, CustomCursor)>,
    active: bool,
}

impl RemoteCursor {
    pub(super) fn update(
        &mut self,
        event_loop: &ActiveEventLoop,
        view: &mut View<DioxusNativeWindowRenderer>,
        position: Option<(f32, f32)>,
    ) {
        let session = crate::dioxus_ui::current_session();
        let status = session
            .as_ref()
            .and_then(|session| session.ui_handler.snapshot());
        let doc = view.downcast_doc_mut::<DioxusDocument>();
        let canvas = doc.get_element_by_id("remote-desktop");
        let over_canvas = position
            .zip(canvas)
            .is_some_and(|((x, y), id)| doc.hit(x, y).is_some_and(|hit| hit.node_id == id));
        if !over_canvas
            || session
                .as_ref()
                .is_none_or(|session| session.is_view_camera())
        {
            if self.active {
                view.window.set_cursor_visible(true);
                view.window.set_cursor(CursorIcon::Default);
                self.active = false;
            }
            return;
        }
        let (Some(session), Some(status), Some(canvas)) = (session, status, canvas) else {
            return;
        };
        self.active = true;
        let embedded = status
            .displays
            .get(status.current_display.max(0) as usize)
            .is_some_and(|display| display.cursor_embedded);
        let cursors = session.ui_handler.cursors.lock().unwrap();
        let remote_position = status
            .displays
            .get(status.current_display.max(0) as usize)
            .is_some_and(|display| cursors.overlay_rect(display, (1, 1), 1.0, false).is_some())
            && session.get_toggle_option("show-remote-cursor".to_owned());
        let shape = cursors.shape();
        drop(cursors);
        if embedded || (remote_position && shape.is_some()) {
            view.window.set_cursor_visible(false);
            return;
        }
        view.window.set_cursor_visible(true);
        if session.get_toggle_option("view-only".to_owned())
            || !*session.server_keyboard_enabled.read().unwrap()
        {
            view.window.set_cursor(CursorIcon::NotAllowed);
            return;
        }
        let Some(shape) = shape else {
            view.window.set_cursor(CursorIcon::Default);
            return;
        };
        let zoom = session.get_toggle_option("zoom-cursor".to_owned())
            || session.get_view_style() == "original";
        let scale = if zoom {
            view.doc.get_node(canvas).map_or(1.0, |node| {
                node.final_layout.size.width as f64 / status.width.max(1) as f64
            })
        } else {
            1.0
        } * view.window.scale_factor();
        let scale = scale.min(2048.0 / shape.width.max(shape.height).max(1) as f64);
        let width = (shape.width as f64 * scale).round().clamp(1.0, 2048.0) as u16;
        let height = (shape.height as f64 * scale).round().clamp(1.0, 2048.0) as u16;
        if self
            .cached
            .get(&shape.id)
            .is_none_or(|(data, w, h, _)| !Arc::ptr_eq(data, &shape) || *w != width || *h != height)
        {
            let Some(image) = image::RgbaImage::from_raw(
                shape.width as u32,
                shape.height as u32,
                shape.colors.to_vec(),
            ) else {
                return;
            };
            let pixels = image::imageops::resize(
                &image,
                width as u32,
                height as u32,
                image::imageops::FilterType::Nearest,
            )
            .into_raw();
            match CustomCursor::from_rgba(
                pixels,
                width,
                height,
                (shape.hotx as f64 * scale)
                    .round()
                    .clamp(0.0, width as f64 - 1.0) as u16,
                (shape.hoty as f64 * scale)
                    .round()
                    .clamp(0.0, height as f64 - 1.0) as u16,
            ) {
                Ok(source) => {
                    self.cached.insert(
                        shape.id,
                        (
                            shape.clone(),
                            width,
                            height,
                            event_loop.create_custom_cursor(source),
                        ),
                    );
                }
                Err(error) => {
                    hbb_common::throttled_log!(
                        Duration::from_secs(30),
                        warn,
                        "Cannot create remote cursor: {error}"
                    );
                    view.window.set_cursor(CursorIcon::Default);
                    return;
                }
            }
        }
        if let Some((_, _, _, cursor)) = self.cached.get(&shape.id) {
            view.window.set_cursor(cursor.clone());
        }
    }
}
