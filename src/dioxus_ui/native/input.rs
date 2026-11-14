use super::*;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};

#[derive(Default)]
pub(super) struct RemoteInput {
    position: (f64, f64),
    cursor: (f32, f32),
    inside: bool,
    buttons: i32,
    keys: std::collections::HashMap<winit::keyboard::PhysicalKey, String>,
}

impl RemoteInput {
    pub(super) fn observe(&mut self, event: &WindowEvent, scale: f64) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.inside = true;
            self.cursor = ((position.x / scale) as f32, (position.y / scale) as f32);
        }
        if matches!(
            event,
            WindowEvent::CursorLeft { .. } | WindowEvent::Focused(false)
        ) {
            self.inside = false;
        }
    }
    pub(super) fn cursor_position(&self) -> Option<(f32, f32)> {
        self.inside.then_some(self.cursor)
    }
    pub(super) fn release(&mut self) {
        if let Some(session) = crate::dioxus_ui::current_session() {
            for (_, key) in self.keys.drain() {
                session.input_key(&key, false, false, false, false, false, false);
            }
            for button in [1, 2, 4, 8, 16] {
                if self.buttons & button != 0 {
                    session.send_mouse(
                        button << 3 | 2,
                        self.position.0 as i32,
                        self.position.1 as i32,
                        false,
                        false,
                        false,
                        false,
                    );
                }
            }
        }
        self.buttons = 0;
        self.keys.clear();
    }

    pub(super) fn handle(
        &mut self,
        view: &mut View<DioxusNativeWindowRenderer>,
        event: &WindowEvent,
    ) -> bool {
        if matches!(
            event,
            WindowEvent::Focused(false) | WindowEvent::CloseRequested
        ) {
            self.release();
            return false;
        }
        if !matches!(
            event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::KeyboardInput { .. }
        ) {
            return false;
        }
        let Some(session) = crate::dioxus_ui::current_session() else {
            return false;
        };
        let Some(status) = session.ui_handler.snapshot() else {
            return false;
        };
        if session.is_view_camera()
            || session.get_toggle_option("view-only".to_owned())
            || !*session.server_keyboard_enabled.read().unwrap()
        {
            self.release();
            return false;
        }
        let (x, y) = self.cursor;
        let doc = view.downcast_doc_mut::<DioxusDocument>();
        let Some(id) = doc.get_element_by_id("remote-desktop") else {
            return false;
        };
        if let WindowEvent::KeyboardInput { event, .. } = event {
            if doc.get_focussed_node_id() != Some(id)
                && (event.state.is_pressed() || !self.keys.contains_key(&event.physical_key))
            {
                return false;
            }
            let key = if event.state.is_pressed() {
                let Some(key) = key_name(&event.logical_key) else {
                    return true;
                };
                self.keys.insert(event.physical_key, key.clone());
                key
            } else {
                let Some(key) = self.keys.remove(&event.physical_key) else {
                    return true;
                };
                key
            };
            let modifiers = view.keyboard_modifiers.state();
            session.input_key(
                &key,
                event.state.is_pressed(),
                false,
                modifiers.alt_key(),
                modifiers.control_key(),
                modifiers.shift_key(),
                modifiers.super_key(),
            );
            return true;
        }
        let over_canvas = doc.hit(x, y).is_some_and(|hit| hit.node_id == id);
        if !over_canvas && self.buttons == 0 {
            return false;
        }
        let Some(node) = doc.get_node(id) else {
            return false;
        };
        let origin = node.absolute_position(0.0, 0.0);
        let width = node.final_layout.size.width as f64;
        let height = node.final_layout.size.height as f64;
        if width < 1.0 || height < 1.0 {
            return false;
        }
        let px = ((x - origin.x) as f64 * status.width as f64 / width)
            .round()
            .clamp(0.0, status.width.saturating_sub(1).max(0) as f64) as i32;
        let py = ((y - origin.y) as f64 * status.height as f64 / height)
            .round()
            .clamp(0.0, status.height.saturating_sub(1).max(0) as f64) as i32;
        let display = status.displays.get(status.current_display.max(0) as usize);
        let point = (
            px + display.map_or(0, |d| d.x),
            py + display.map_or(0, |d| d.y),
        );
        self.position = (point.0 as f64, point.1 as f64);
        let modifiers = view.keyboard_modifiers.state();
        let (mask, point) = match event {
            WindowEvent::CursorMoved { .. } => (self.buttons << 3, point),
            WindowEvent::MouseInput { button, state, .. } => {
                let button = match button {
                    MouseButton::Left => 1,
                    MouseButton::Right => 2,
                    MouseButton::Middle => 4,
                    MouseButton::Back => 8,
                    MouseButton::Forward => 16,
                    _ => return false,
                };
                if *state == ElementState::Pressed {
                    self.buttons |= button;
                    view.downcast_doc_mut::<DioxusDocument>().set_focus_to(id);
                } else {
                    self.buttons &= !button;
                }
                (
                    button << 3
                        | if *state == ElementState::Pressed {
                            1
                        } else {
                            2
                        },
                    point,
                )
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let point = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x.round() as i32, y.round() as i32),
                    MouseScrollDelta::PixelDelta(p) => {
                        ((p.x / 40.0).round() as i32, (p.y / 40.0).round() as i32)
                    }
                };
                (3, point)
            }
            _ => return false,
        };
        session.send_mouse(
            mask,
            point.0,
            point.1,
            modifiers.alt_key(),
            modifiers.control_key(),
            modifiers.shift_key(),
            modifiers.super_key(),
        );
        if matches!(event, WindowEvent::CursorMoved { .. }) {
            if session
                .ui_handler
                .cursors
                .lock()
                .unwrap()
                .position
                .take()
                .is_some()
            {
                view.request_redraw();
            }
        }
        !matches!(event, WindowEvent::CursorMoved { .. })
    }
}

fn key_name(key: &winit::keyboard::Key) -> Option<String> {
    use winit::keyboard::{Key, NamedKey};
    let name = match key {
        Key::Character(text) => return Some(text.to_string()),
        Key::Named(key) => match key {
            NamedKey::Backspace => "VK_BACK",
            NamedKey::Tab => "VK_TAB",
            NamedKey::Enter => "VK_ENTER",
            NamedKey::Escape => "VK_ESCAPE",
            NamedKey::Space => " ",
            NamedKey::Shift => "VK_SHIFT",
            NamedKey::Control => "VK_CONTROL",
            NamedKey::Alt => "VK_MENU",
            NamedKey::Super | NamedKey::Meta => "VK_LWIN",
            NamedKey::CapsLock => "VK_CAPITAL",
            NamedKey::PageUp => "VK_PRIOR",
            NamedKey::PageDown => "VK_NEXT",
            NamedKey::End => "VK_END",
            NamedKey::Home => "VK_HOME",
            NamedKey::ArrowLeft => "VK_LEFT",
            NamedKey::ArrowUp => "VK_UP",
            NamedKey::ArrowRight => "VK_RIGHT",
            NamedKey::ArrowDown => "VK_DOWN",
            NamedKey::Insert => "VK_INSERT",
            NamedKey::Delete => "VK_DELETE",
            _ => {
                let name = format!("{key:?}");
                if name
                    .strip_prefix('F')
                    .and_then(|n| n.parse::<u8>().ok())
                    .is_some_and(|n| (1..=24).contains(&n))
                {
                    return Some(format!("VK_{name}"));
                }
                return None;
            }
        },
        _ => return None,
    };
    Some(name.to_owned())
}
