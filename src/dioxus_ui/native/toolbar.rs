use super::*;
use crate::dioxus_ui::views::docking::{self, ToolbarDock};
use winit::event::{ElementState, MouseButton};

pub(super) struct ToolbarDrag {
    dock: SyncSignal<ToolbarDock>,
    initial: ToolbarDock,
    size: (f64, f64),
    grab: f64,
    multi_edge: bool,
    bounds: (f64, f64),
}

impl ToolbarDrag {
    pub fn start(
        view: &mut View<DioxusNativeWindowRenderer>,
        mut dock: SyncSignal<ToolbarDock>,
    ) -> Option<Self> {
        let position = view.mouse_pos;
        let doc = view.downcast_doc_mut::<DioxusDocument>();
        let node = doc.get_node(doc.get_element_by_id("remote-toolbar")?)?;
        let origin = node.absolute_position(0.0, 0.0);
        let initial = *dock.peek();
        let grab = if initial.edge.horizontal() {
            position.0 - origin.x
        } else {
            position.1 - origin.y
        } as f64;
        let drag = Self {
            dock,
            initial,
            grab,
            size: (
                node.final_layout.size.width as f64,
                node.final_layout.size.height as f64,
            ),
            multi_edge: docking::local_multi_edge(),
            bounds: docking::fraction_bounds(docking::Edge::Top),
        };
        dock.write().dragging = true;
        view.window.set_cursor(winit::window::CursorIcon::Grabbing);
        Some(drag)
    }

    pub fn handle(
        &mut self,
        view: &mut View<DioxusNativeWindowRenderer>,
        event: &WindowEvent,
    ) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let x = position.x / view.window.scale_factor();
                let y = position.y / view.window.scale_factor();
                view.mouse_pos = (x as f32, y as f32);
                let doc = view.downcast_doc_mut::<DioxusDocument>();
                if let Some(node) = doc
                    .get_element_by_id("remote-toolbar-area")
                    .and_then(|id| doc.get_node(id))
                {
                    let origin = node.absolute_position(0.0, 0.0);
                    self.dock.set(docking::drag_position(
                        (x - origin.x as f64, y - origin.y as f64),
                        (
                            node.final_layout.size.width as f64,
                            node.final_layout.size.height as f64,
                        ),
                        self.size,
                        self.grab,
                        self.multi_edge,
                        self.bounds,
                    ));
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                ..
            } => {
                self.dock.write().dragging = false;
                if let Some(session) = crate::dioxus_ui::current_session() {
                    let value = *self.dock.peek();
                    session.set_option(
                        OPTION_REMOTE_MENUBAR_EDGE.to_owned(),
                        value.edge.name().to_owned(),
                    );
                    session.set_option(
                        OPTION_REMOTE_MENUBAR_FRACTION.to_owned(),
                        value.fraction.to_string(),
                    );
                    if !self.multi_edge {
                        session.set_option(
                            "remote-menubar-drag-x".to_owned(),
                            value.fraction.to_string(),
                        );
                    }
                }
                view.window.set_cursor(winit::window::CursorIcon::Default);
                view.buttons = Default::default();
                return false;
            }
            WindowEvent::Focused(false) | WindowEvent::CloseRequested => {
                self.dock.set(self.initial);
                view.window.set_cursor(winit::window::CursorIcon::Default);
                view.buttons = Default::default();
                return false;
            }
            _ => {}
        }
        true
    }
}
