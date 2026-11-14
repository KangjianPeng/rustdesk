use super::prelude::*;
use blitz_dom::net::Resource;
use blitz_shell::{BlitzApplication, BlitzShellEvent, BlitzShellNetCallback, View, WindowConfig};
use blitz_traits::{
    net::{BoxedHandler, NetCallback, NetProvider, Request},
    shell::{ClipboardError, FileDialogFilter, ShellProvider},
};
use dioxus::{
    document::{Document, NoOpDocument},
    native::{DioxusDocument, DioxusNativeWindowRenderer, DocumentConfig},
};
use std::{rc::Rc, sync::OnceLock};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Fullscreen, Window, WindowId},
};

#[path = "native/cursor.rs"]
mod cursor;
#[path = "native/input.rs"]
mod input;
#[path = "native/select.rs"]
pub(super) mod select;
#[path = "native/toolbar.rs"]
mod toolbar;

static WINDOW: OnceLock<Weak<Window>> = OnceLock::new();

#[derive(Clone)]
pub(super) struct WindowHandle {
    window: Arc<Window>,
    proxy: EventLoopProxy<BlitzShellEvent>,
}

pub(super) fn window() -> WindowHandle {
    consume_context()
}

pub(super) fn request_redraw() {
    if let Some(window) = WINDOW.get().and_then(Weak::upgrade) {
        window.request_redraw();
    }
}

impl WindowHandle {
    pub(super) fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }
    pub(super) fn drag(&self) {
        if let Err(error) = self.window.drag_window() {
            log::trace!("Cannot drag native window: {error}");
        }
    }
    pub(super) fn drag_toolbar(&self, dock: SyncSignal<super::views::docking::ToolbarDock>) {
        if self
            .proxy
            .send_event(BlitzShellEvent::embedder_event(Command::DragToolbar(dock)))
            .is_err()
        {
            log::trace!("Native toolbar already closed");
        }
    }
    pub(super) fn set_minimized(&self, minimized: bool) {
        self.window.set_minimized(minimized);
    }
    pub(super) fn toggle_maximized(&self) {
        self.window.set_maximized(!self.window.is_maximized());
    }
    pub(super) fn set_fullscreen(&self, fullscreen: bool) {
        self.window
            .set_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
    }
    pub(super) fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible);
    }
    pub(super) fn close(&self) {
        if self
            .proxy
            .send_event(BlitzShellEvent::embedder_event(Command::Close))
            .is_err()
        {
            log::trace!("Native window already closed");
        }
    }
}

enum Command {
    Close,
    DragToolbar(SyncSignal<super::views::docking::ToolbarDock>),
    Head {
        name: String,
        attributes: Vec<(String, String)>,
        contents: Option<String>,
    },
}

impl Document for WindowHandle {
    fn set_title(&self, title: String) {
        self.window.set_title(&title);
    }
    fn eval(&self, js: String) -> dioxus::document::Eval {
        NoOpDocument.eval(js)
    }
    fn create_head_element(
        &self,
        name: &str,
        attributes: &[(&str, String)],
        contents: Option<String>,
    ) {
        let command = Command::Head {
            name: name.to_owned(),
            attributes: attributes
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect(),
            contents,
        };
        if self
            .proxy
            .send_event(BlitzShellEvent::embedder_event(command))
            .is_err()
        {
            log::trace!("Native document already closed");
        }
    }
    fn create_head_component(&self) -> bool {
        true
    }
}

struct Assets {
    callback: Arc<dyn NetCallback<Resource>>,
    data: Arc<dyn NetProvider<Resource>>,
}
impl NetProvider<Resource> for Assets {
    fn fetch(&self, doc_id: usize, request: Request, handler: BoxedHandler<Resource>) {
        if request.url.scheme() == "dioxus" {
            match dioxus_asset_resolver::native::serve_asset(request.url.path()) {
                Ok(response) => {
                    handler.bytes(doc_id, response.into_body().into(), self.callback.clone())
                }
                Err(error) => log::warn!("Cannot load native asset {}: {error:?}", request.url),
            }
        } else {
            self.data.fetch(doc_id, request, handler);
        }
    }
}

struct NativeShell(Arc<dyn ShellProvider>);
impl ShellProvider for NativeShell {
    fn request_redraw(&self) {
        self.0.request_redraw();
    }
    fn set_cursor(&self, icon: blitz_shell::CursorIcon) {
        self.0.set_cursor(icon);
    }
    // Blitz also forwards SVG title nodes here. Dioxus sets the application title through Document.
    fn set_window_title(&self, _title: String) {}
    fn get_clipboard_text(&self) -> Result<String, ClipboardError> {
        self.0.get_clipboard_text()
    }
    fn set_clipboard_text(&self, text: String) -> Result<(), ClipboardError> {
        self.0.set_clipboard_text(text)
    }
    fn open_file_dialog(
        &self,
        multiple: bool,
        filter: Option<FileDialogFilter>,
    ) -> Vec<std::path::PathBuf> {
        self.0.open_file_dialog(multiple, filter)
    }
}

pub(super) fn launch(app: fn() -> Element, args: Vec<String>, title: String) {
    if let Err(error) = dioxus::logger::init(dioxus::logger::tracing::Level::WARN) {
        log::trace!("Native tracing already configured: {error}");
    }
    let event_loop = match EventLoop::<BlitzShellEvent>::with_user_event().build() {
        Ok(event_loop) => event_loop,
        Err(error) => {
            log::error!("Cannot create native event loop: {error}");
            return;
        }
    };
    let proxy = event_loop.create_proxy();
    let callback = BlitzShellNetCallback::shared(proxy.clone());
    let assets = Arc::new(Assets {
        data: blitz_shell::DataUriNetProvider::shared(callback.clone()),
        callback,
    });
    let mut vdom = VirtualDom::new(app);
    vdom.provide_root_context(args);
    let doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            base_url: Some("dioxus://index/".to_owned()),
            net_provider: Some(assets),
            ..Default::default()
        },
    );
    let attributes = Window::default_attributes()
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .with_decorations(false)
        .with_visible(true);
    let config =
        WindowConfig::with_attributes(Box::new(doc), DioxusNativeWindowRenderer::new(), attributes);
    let mut application = Application {
        inner: BlitzApplication::new(proxy),
        pending: Some(config),
        input: input::RemoteInput::default(),
        cursor: cursor::RemoteCursor::default(),
        toolbar_drag: None,
    };
    if let Err(error) = event_loop.run_app(&mut application) {
        log::error!("Native event loop failed: {error}");
    }
}

struct Application {
    inner: BlitzApplication<DioxusNativeWindowRenderer>,
    pending: Option<WindowConfig<DioxusNativeWindowRenderer>>,
    input: input::RemoteInput,
    cursor: cursor::RemoteCursor,
    toolbar_drag: Option<toolbar::ToolbarDrag>,
}
impl ApplicationHandler<BlitzShellEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(config) = self.pending.take() {
            let mut view = View::init(config, event_loop, &self.inner.proxy);
            let handle = WindowHandle {
                window: view.window.clone(),
                proxy: self.inner.proxy.clone(),
            };
            let _ = WINDOW.set(Arc::downgrade(&view.window));
            let renderer = view.renderer.clone();
            let shell = Arc::new(NativeShell(view.doc.shell_provider.clone()));
            view.doc.set_shell_provider(shell);
            let doc = view.downcast_doc_mut::<DioxusDocument>();
            doc.vdom.in_scope(ScopeId::ROOT, || {
                provide_context(handle.clone());
                provide_context(Rc::new(handle) as Rc<dyn Document>);
                provide_context(renderer);
            });
            doc.initial_build();
            view.request_redraw();
            self.inner.windows.insert(view.window_id(), view);
        }
        self.inner.resumed(event_loop);
    }
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.suspended(event_loop);
    }
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.inner.new_events(event_loop, cause);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::CloseRequested) {
            if let Some(view) = self.inner.windows.get_mut(&id) {
                self.input.release();
                if view
                    .downcast_doc_mut::<DioxusDocument>()
                    .vdom
                    .in_scope(ScopeId::ROOT, || consume_context::<Vec<String>>())
                    .first()
                    .is_some_and(|arg| arg == "--cm")
                {
                    view.window.set_visible(false);
                    view.downcast_doc_mut::<DioxusDocument>()
                        .vdom
                        .in_scope(ScopeId::ROOT, || {
                            spawn(super::connection_manager::close_manager(false));
                        });
                    return;
                }
            }
        }
        if let Some(view) = self.inner.windows.get_mut(&id) {
            self.input.observe(&event, view.window.scale_factor());
            if let Some(drag) = self.toolbar_drag.as_mut() {
                if !matches!(event, WindowEvent::RedrawRequested) {
                    if drag.handle(view, &event) {
                        if matches!(
                            event,
                            WindowEvent::CursorMoved { .. }
                                | WindowEvent::MouseInput { .. }
                                | WindowEvent::MouseWheel { .. }
                        ) {
                            return;
                        }
                    } else {
                        self.toolbar_drag = None;
                        if matches!(event, WindowEvent::MouseInput { .. }) {
                            return;
                        }
                    }
                }
            }
            if matches!(event, WindowEvent::RedrawRequested)
                && view.doc.get_element_by_id("remote-desktop").is_some()
            {
                // Blitz animates all canvases continuously. Video frames already request redraws.
                let visible = view.is_visible;
                view.is_visible = false;
                view.redraw();
                view.is_visible = visible;
                if self.toolbar_drag.is_none() {
                    self.cursor
                        .update(event_loop, view, self.input.cursor_position());
                }
                return;
            }
            if self.input.handle(view, &event) {
                return;
            }
        }
        self.inner.window_event(event_loop, id, event);
        if let Some(view) = self.inner.windows.get_mut(&id) {
            self.cursor
                .update(event_loop, view, self.input.cursor_position());
        }
    }
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: BlitzShellEvent) {
        if let BlitzShellEvent::Embedder(value) = &event {
            if let Some(command) = value.downcast_ref::<Command>() {
                match command {
                    Command::DragToolbar(dock) => {
                        if let Some(view) = self.inner.windows.values_mut().next() {
                            self.toolbar_drag = toolbar::ToolbarDrag::start(view, *dock);
                        }
                    }
                    Command::Close => {
                        self.input.release();
                        self.inner.windows.clear();
                        event_loop.exit();
                    }
                    Command::Head {
                        name,
                        attributes,
                        contents,
                    } => {
                        for view in self.inner.windows.values_mut() {
                            view.downcast_doc_mut::<DioxusDocument>()
                                .create_head_element(name, attributes, contents);
                            view.poll();
                        }
                    }
                }
                return;
            }
        }
        self.inner.user_event(event_loop, event);
        for view in self.inner.windows.values_mut() {
            if self.toolbar_drag.is_none() {
                self.cursor
                    .update(event_loop, view, self.input.cursor_position());
            }
        }
    }
}
