use super::prelude::*;
use dioxus::desktop::{
    launch::launch_virtual_dom_blocking,
    wry::http::{header, HeaderValue, Request, Response, StatusCode},
    Config, LogicalSize, WindowBuilder,
};
use std::borrow::Cow;

#[cfg(target_os = "linux")]
mod native_video;
#[cfg(target_os = "linux")]
pub(super) use native_video::{NativeVideoSurface, Viewport};

pub(super) fn launch(app: fn() -> Element, args: Vec<String>, title: String) {
    #[cfg(target_os = "linux")]
    let remote = super::runtime::is_remote_window(&args);
    let dom = VirtualDom::new(app);
    dom.provide_root_context(args);
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title(title)
                .with_inner_size(LogicalSize::new(1280.0, 720.0))
                .with_decorations(false),
        )
        .with_custom_protocol("rustdesk", |_, request| serve_session(request));
    #[cfg(target_os = "linux")]
    // Dioxus's fallback mutates GDK_BACKEND after GTK starts; child windows then
    // switch to X11, breaking the native video/WebView composition on Wayland.
    let config = config.with_disable_dma_buf_on_wayland(false);
    #[cfg(target_os = "linux")]
    let config = if remote {
        config.with_on_window(|window, _| remove_remote_system_titlebar(&window))
    } else {
        config
    };
    launch_virtual_dom_blocking(dom, config);
}

#[cfg(target_os = "linux")]
pub(super) fn remove_remote_system_titlebar(window: &dioxus::desktop::tao::window::Window) {
    use dioxus::desktop::tao::platform::unix::WindowExtUnix;
    use gtk::prelude::*;

    let window = window.gtk_window();
    if window.display().type_().name() == "GdkWaylandDisplay" {
        // Tao 0.34 needs an empty CSD widget to suppress Wayland server decorations.
        window.set_titlebar(Some(&gtk::EventBox::new()));
        window.set_decorated(false);
    }
}

pub(super) fn monitor_size() -> Option<(i32, i32)> {
    let size = dioxus::desktop::window().current_monitor()?.size();
    Some((
        i32::try_from(size.width).ok()?,
        i32::try_from(size.height).ok()?,
    ))
}

pub(super) fn session_url() -> &'static str {
    if cfg!(target_os = "windows") {
        "http://rustdesk.localhost"
    } else {
        "rustdesk://localhost"
    }
}

fn serve_session(request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let mut response = Response::new(Cow::Borrowed(&[][..]));
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static("x-frame-width, x-frame-height, x-frame-stride, x-frame-format"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    let Some(session) = super::current_session() else {
        *response.status_mut() = StatusCode::NO_CONTENT;
        return response;
    };
    match request.uri().path() {
        "/frame" => {
            if let Some(mut packet) = session.ui_handler.take_frame() {
                if let Some(stride) = packet.stride() {
                    let headers = response.headers_mut();
                    headers.insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/octet-stream"),
                    );
                    headers.insert("x-frame-width", HeaderValue::from(packet.width));
                    headers.insert("x-frame-height", HeaderValue::from(packet.height));
                    headers.insert("x-frame-stride", HeaderValue::from(stride));
                    headers.insert(
                        "x-frame-format",
                        HeaderValue::from_static(match packet.format {
                            ImageFormat::Raw => "rgb",
                            ImageFormat::ARGB => "bgra",
                            ImageFormat::ABGR => "rgba",
                        }),
                    );
                    packet.raw.truncate(stride * packet.height);
                    *response.body_mut() = Cow::Owned(packet.raw);
                } else {
                    *response.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
                    hbb_common::throttled_log!(
                        Duration::from_secs(30),
                        warn,
                        "Invalid WebView video frame"
                    );
                }
            } else {
                *response.status_mut() = StatusCode::NO_CONTENT;
            }
        }
        "/cursor" => match session.ui_handler.cursor_snapshot(&session) {
            Ok(body) => {
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                *response.body_mut() = Cow::Owned(body);
            }
            Err(error) => {
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                hbb_common::throttled_log!(
                    Duration::from_secs(30),
                    warn,
                    "Cannot read remote cursor: {error}"
                );
            }
        },
        _ => *response.status_mut() = StatusCode::NOT_FOUND,
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_protocol_returns_binary_pixels_once_without_png_encoding() {
        fn fixture() -> Element {
            let state = use_signal_sync(DioxusUiState::default);
            let session = new_session(
                "webview-protocol-test".to_owned(),
                String::new(),
                vec![],
                state,
            );
            let mut image = scrap::ImageRgb::new(ImageFormat::ABGR, 8);
            image.w = 1;
            image.h = 2;
            image.raw = vec![255, 0, 0, 255, 9, 9, 9, 9, 0, 255, 0, 255, 9, 9, 9, 9];
            session.ui_handler.on_rgba(0, &mut image);
            set_current_session(Some(session));
            let request = || {
                Request::builder()
                    .uri("rustdesk://localhost/frame")
                    .body(Vec::new())
                    .unwrap()
            };
            let response = serve_session(request());
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()["x-frame-width"], "1");
            assert_eq!(response.headers()["x-frame-height"], "2");
            assert_eq!(response.headers()["x-frame-stride"], "8");
            assert_eq!(response.headers()["x-frame-format"], "rgba");
            assert_eq!(&response.body()[..4], &[255, 0, 0, 255]);
            assert_eq!(&response.body()[8..12], &[0, 255, 0, 255]);
            assert_eq!(serve_session(request()).status(), StatusCode::NO_CONTENT);
            set_current_session(None);
            rsx! { div {} }
        }
        VirtualDom::new(fixture).rebuild_in_place();
    }
}
