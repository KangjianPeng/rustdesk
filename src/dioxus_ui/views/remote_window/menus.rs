use super::super::super::prelude::*;
use super::toolbar::ToolbarState;
use icons::{Check, ChevronRight};

const ITEM: &str = "flex min-h-8 w-full items-center gap-2 rounded-[3px] px-3 py-1.5 text-left text-xs hover:bg-[#3b5d91] focus-visible:bg-[#3b5d91] disabled:opacity-40 [&_svg]:block [&_svg]:size-4";

#[component]
pub(super) fn MenuItem(
    label: String,
    #[props(optional)] checked: Option<bool>,
    #[props(default)] disabled: bool,
    #[props(default)] keep_open: bool,
    onclick: EventHandler<dioxus::prelude::MouseEvent>,
) -> Element {
    let toolbar = use_context::<ToolbarState>();
    rsx! {
        button {
            class: ITEM, r#type: "button", role: if checked.is_some() { "menuitemcheckbox" } else { "menuitem" },
            aria_checked: checked.map(|value| value.to_string()), disabled,
            onclick: move |event| {
                onclick.call(event);
                toolbar.changed();
                if !keep_open { toolbar.dismiss(); }
            },
            span { class: "flex h-4 w-4 shrink-0 items-center justify-center", if checked == Some(true) { Check {} } }
            span { class: "min-w-0 flex-1", "{label}" }
        }
    }
}

#[component]
pub(super) fn Divider() -> Element {
    rsx! { div { class: "my-1.5 border-t border-[#48494e]", role: "separator" } }
}

#[component]
fn Submenu(name: &'static str, label: &'static str) -> Element {
    let mut toolbar = use_context::<ToolbarState>();
    rsx! {
        button { class: ITEM, r#type: "button", role: "menuitem", aria_haspopup: "menu", aria_expanded: (toolbar.submenu)() == Some(name),
            onclick: move |_| {
                let next = if (toolbar.submenu)() == Some(name) { None } else { Some(name) };
                toolbar.submenu.set(next);
            },
            span { class: "h-4 w-4 shrink-0" }
            span { class: "min-w-0 flex-1", "{label}" }
            ChevronRight {}
        }
    }
}

#[component]
fn Toggle(label: &'static str, option: &'static str, #[props(default)] disabled: bool) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let toolbar = use_context::<ToolbarState>();
    let _ = (toolbar.revision)();
    let checked = state
        .session
        .peek()
        .as_ref()
        .is_some_and(|active| active.get_toggle_option(option.to_owned()));
    rsx! {
        MenuItem { label, checked, disabled,
            onclick: move |_| if let Some(active) = state.session.peek().as_ref() { active.toggle_option(option.to_owned()); }
        }
    }
}

#[component]
pub(super) fn MonitorMenu() -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let status = state.ui_state.read();
    rsx! {
        for (index, display) in status.displays.iter().enumerate() {
            MenuItem { label: format!("{} · {} × {}", index + 1, display.width, display.height),
                checked: status.current_display == index as i32, disabled: !display.online,
                onclick: move |_| if let Some(active) = state.session.peek().as_ref() { active.switch_display(index as i32); }
            }
        }
    }
}

#[component]
pub(super) fn ControlMenu() -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let session = state.session;
    let status = state.ui_state.read();
    let disabled = status.remote.permissions.get("keyboard") == Some(&false)
        || session
            .peek()
            .as_ref()
            .is_none_or(|active| active.get_toggle_option("view-only".to_owned()));
    rsx! {
        MenuItem { label: "Send clipboard keystrokes", disabled,
            onclick: move |_| {
                spawn(async move {
                    let text = hbb_common::tokio::task::spawn_blocking(|| arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text())).await;
                    match text {
                        Ok(Ok(text)) => if let Some(active) = session.peek().as_ref() { active.input_string(&text); },
                        Ok(Err(error)) => log::warn!("Cannot read clipboard text: {error}"),
                        Err(error) => log::warn!("Clipboard text task failed: {error}"),
                    }
                });
            }
        }
        Divider {}
        MenuItem { label: "Insert Ctrl + Alt + Del", disabled,
            onclick: move |_| if let Some(active) = session.peek().as_ref() { active.ctrl_alt_del(); }
        }
        MenuItem { label: "Insert Lock", disabled,
            onclick: move |_| if let Some(active) = session.peek().as_ref() { active.lock_screen(); }
        }
        MenuItem { label: "Refresh",
            onclick: move |_| if let Some(active) = session.peek().as_ref() { active.refresh_video(state.ui_state.peek().current_display); }
        }
    }
}

#[component]
pub(super) fn DisplayMenu(
    mut fullscreen: Signal<bool>,
    mut view_style: Signal<String>,
    custom_scale: Signal<u16>,
) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let session = state.session;
    let status = state.ui_state.read();
    let camera = session
        .peek()
        .as_ref()
        .is_some_and(|active| active.is_view_camera());
    let view_only = session
        .peek()
        .as_ref()
        .is_some_and(|active| active.get_toggle_option("view-only".to_owned()));
    let keyboard = status.remote.permissions.get("keyboard") != Some(&false);
    rsx! {
        MenuItem { label: if fullscreen() { "Exit Fullscreen" } else { "Fullscreen" },
            onclick: move |_| {
                let next = !fullscreen();
                crate::dioxus_ui::native::window().set_fullscreen(next);
                fullscreen.set(next);
            }
        }
        Divider {}
        for (value, label) in [("original", "Scale original"), ("adaptive", "Scale adaptive"), ("custom", "Scale custom")] {
            MenuItem { label, checked: view_style() == value, keep_open: value == "custom",
                onclick: move |_| {
                    view_style.set(value.to_owned());
                    if let Some(active) = session.peek().as_ref() { active.save_view_style(value.to_owned()); }
                }
            }
        }
        if view_style() == "custom" { ScaleControls { value: custom_scale } }
        Divider {}
        Submenu { name: "quality", label: "Image Quality" }
        Submenu { name: "codec", label: "Codec" }
        Divider {}
        Toggle { label: "Show quality monitor", option: "show-quality-monitor" }
        if !camera {
            Toggle { label: "Show remote cursor", option: "show-remote-cursor" }
            Toggle { label: "Zoom cursor", option: "zoom-cursor" }
            if status.remote.permissions.get("audio") != Some(&false) { Toggle { label: "Mute", option: "disable-audio" } }
            if keyboard && status.remote.permissions.get("clipboard") != Some(&false) { Toggle { label: "Disable clipboard", option: "disable-clipboard", disabled: view_only } }
            if keyboard { Toggle { label: "Lock after session end", option: "lock-after-session-end", disabled: view_only } }
        }
    }
}

#[component]
fn ScaleControls(mut value: Signal<u16>) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let status = state.ui_state.read();
    let maximum = super::maximum_scale(
        status.width,
        status.height,
        crate::dioxus_ui::native::window().scale_factor(),
    );
    rsx! {
        label { class: "mx-3 my-2 flex items-center gap-2 text-xs", "Scale (%)",
            input { class: "h-7 w-20 rounded border border-[#55575b] bg-rd-content px-2", r#type: "number", min: "5", max: "{maximum}", step: "5", value: "{value}",
                onchange: move |event| if let Ok(percent) = event.value().parse::<u16>() {
                    let percent = percent.clamp(5, maximum);
                    value.set(percent);
                    if let Some(active) = state.session.peek().as_ref() {
                        let mut config = active.load_config();
                        config.ui_flutter.insert("custom_scale_percent".to_owned(), percent.to_string());
                        active.save_config(config);
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn DisplaySubmenu(name: &'static str) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let session = state.session;
    let toolbar = use_context::<ToolbarState>();
    let _ = (toolbar.revision)();
    let quality = session
        .peek()
        .as_ref()
        .map(|active| active.get_image_quality())
        .unwrap_or_default();
    let codec = session
        .peek()
        .as_ref()
        .map(|active| active.get_option(OPTION_CODEC_PREFERENCE.to_owned()))
        .unwrap_or_default();
    let codecs = session
        .peek()
        .as_ref()
        .map(|active| active.alternative_codecs())
        .unwrap_or_default();
    rsx! {
        if name == "quality" {
            for (value, label) in [("best", "Good image quality"), ("balanced", "Balanced"), ("low", "Optimize reaction time"), ("custom", "Custom")] {
                MenuItem { label, checked: quality == value || (quality.is_empty() && value == "balanced"), onclick: move |_| if let Some(active) = session.peek().as_ref() {
                        active.save_image_quality(value.to_owned());
                        if value == "custom" { let mut open = toolbar.quality_dialog; open.set(true); }
                    }
                }
            }
        } else if name == "codec" {
            for (value, label, enabled) in [("auto", "Auto", true), ("vp8", "VP8", codecs.0), ("vp9", "VP9", true), ("av1", "AV1", codecs.1), ("h264", "H264", codecs.2), ("h265", "H265", codecs.3)] {
                if enabled {
                    MenuItem { label, checked: codec == value || (codec.is_empty() && value == "auto"),
                        onclick: move |_| if let Some(active) = session.peek().as_ref() {
                            active.set_option(OPTION_CODEC_PREFERENCE.to_owned(), value.to_owned());
                            active.update_supported_decodings();
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn CustomQuality(mut open: Signal<bool>) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let mut quality = use_signal(|| {
        state
            .session
            .peek()
            .as_ref()
            .and_then(|active| active.get_custom_image_quality().first().copied())
            .unwrap_or(50)
    });
    let mut fps = use_signal(|| {
        state
            .session
            .peek()
            .as_ref()
            .and_then(|active| *active.lc.read().unwrap().custom_fps.lock().unwrap())
            .unwrap_or(30) as i32
    });
    rsx! {
        div { class: "absolute inset-0 z-50 flex items-center justify-center bg-black/50 p-4", role: "dialog", aria_modal: "true", aria_label: "Custom Image Quality",
            div { class: "w-full max-w-[420px] rounded-lg border border-[#48494e] bg-rd-surface p-6 text-white shadow-xl",
                h2 { class: "mb-4 text-lg font-medium", "Custom Image Quality" }
                for (label, value, max) in [("Bitrate", quality(), 100), ("FPS", fps(), 120)] {
                    label { class: "mx-3 my-2 flex items-center justify-between gap-2 text-xs", "{label}",
                        input { class: "h-7 w-20 rounded border border-[#55575b] bg-rd-content px-2", r#type: "number", min: "1", max: "{max}", value: "{value}",
                            onchange: move |event| if let Ok(value) = event.value().parse::<i32>() {
                                let value = value.clamp(1, max);
                                if let Some(active) = state.session.peek().as_ref() {
                                    if label == "FPS" { fps.set(value); active.set_custom_fps(value); }
                                    else { quality.set(value); active.save_custom_image_quality(value); }
                                }
                            }
                        }
                    }
                }
                div { class: "mt-4 flex justify-end",
                    button { class: "h-9 rounded bg-[#0071ff] px-4 text-sm text-white", r#type: "button",
                        onclick: move |_| {
                            if let Some(active) = state.session.peek().as_ref() {
                                active.save_custom_image_quality(quality());
                                active.set_custom_fps(fps());
                            }
                            open.set(false);
                        }, "Close"
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn KeyboardMenu() -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let toolbar = use_context::<ToolbarState>();
    let _ = (toolbar.revision)();
    let session = state.session;
    let keyboard = state.ui_state.read().remote.permissions.get("keyboard") != Some(&false);
    let view_only = session
        .peek()
        .as_ref()
        .is_none_or(|active| active.get_toggle_option("view-only".to_owned()));
    let reverse = session
        .peek()
        .as_ref()
        .is_some_and(|active| active.get_reverse_mouse_wheel() == "Y");
    rsx! {
        Toggle { label: "View only", option: "view-only" }
        Divider {}
        MenuItem { label: "Reverse mouse wheel", checked: reverse, disabled: !keyboard || view_only,
            onclick: move |_| if let Some(active) = session.peek().as_ref() { active.save_reverse_mouse_wheel(if reverse { "N" } else { "Y" }.to_owned()); }
        }
        Toggle { label: "Swap left-right mouse", option: "swap-left-right-mouse", disabled: !keyboard || view_only }
    }
}
