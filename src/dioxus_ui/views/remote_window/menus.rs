use super::super::super::prelude::*;
use super::toolbar::ToolbarState;
use icons::{Check, ChevronRight};

const ITEM: &str = "flex min-h-8 w-full items-center gap-2 rounded-[3px] px-3 py-1.5 text-left text-xs hover:bg-accent focus-visible:bg-accent disabled:opacity-40 [&_svg]:block [&_svg]:size-4";

#[component]
pub(super) fn MenuItem(
    label: String,
    #[props(optional)] checked: Option<bool>,
    #[props(default)] disabled: bool,
    #[props(default)] keep_open: bool,
    #[props(default)] radio: bool,
    onclick: EventHandler<dioxus::prelude::MouseEvent>,
) -> Element {
    let toolbar = use_context::<ToolbarState>();
    rsx! {
        button {
            class: ITEM, r#type: "button", role: if radio { "menuitemradio" } else if checked.is_some() { "menuitemcheckbox" } else { "menuitem" },
            aria_checked: checked.map(|value| value.to_string()), disabled,
            onclick: move |event| {
                onclick.call(event);
                toolbar.changed();
                if !keep_open { toolbar.dismiss(); }
            },
            span { class: "flex h-4 w-4 shrink-0 items-center justify-center", if checked == Some(true) { Check {} } }
            span { class: "min-w-0 flex-1", {tr(&label)} }
        }
    }
}

#[component]
pub(super) fn Divider() -> Element {
    rsx! { div { class: "my-1.5 border-t border-border", role: "separator" } }
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
            span { class: "min-w-0 flex-1", {tr(label)} }
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
                checked: status.current_display == index as i32, radio: true, disabled: !display.online,
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
    mut custom_scale: Signal<u16>,
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
                dioxus::desktop::window().set_fullscreen(next);
                fullscreen.set(next);
            }
        }
        Divider {}
        for (value, label) in [("original", "Scale original"), ("adaptive", "Scale adaptive"), ("custom", "Scale custom")] {
            MenuItem { label, checked: view_style() == value, radio: true, keep_open: value == "custom",
                onclick: move |_| {
                    view_style.set(value.to_owned());
                    if let Some(active) = session.peek().as_ref() { active.save_view_style(value.to_owned()); }
                }
            }
        }
        if view_style() == "custom" {
            ScaleControls {
                value: custom_scale(),
                maximum: 1000,
                onchange: move |percent| {
                    custom_scale.set(percent);
                    if let Some(active) = session.peek().as_ref() {
                        let mut config = active.load_config();
                        config.ui_flutter.insert("custom_scale_percent".to_owned(), percent.to_string());
                        active.save_config(config);
                    }
                }
            }
        }
        Divider {}
        Submenu { name: "quality", label: "Image Quality" }
        Submenu { name: "codec", label: "Codec" }
        if !camera && keyboard && status.current_display >= 0 && status.remote.resolutions.len() > 1 && status.displays.get(status.current_display as usize).is_some() {
            Submenu { name: "resolution", label: "Resolution" }
        }
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
fn ScaleControls(value: u16, maximum: u16, onchange: EventHandler<u16>) -> Element {
    let out_class = if value <= 5 {
        "text-muted-foreground"
    } else {
        "text-foreground hover:bg-accent"
    };
    let in_class = if value >= maximum {
        "text-muted-foreground"
    } else {
        "text-foreground hover:bg-accent"
    };
    let reset_class = if maximum < 100 || value == 100 {
        "text-muted-foreground"
    } else {
        "text-foreground hover:bg-accent"
    };
    rsx! {
        div { class: "mx-3 my-2 grid grid-cols-[28px_minmax(0,1fr)_auto_28px_auto] items-center gap-1.5 text-xs", role: "group", aria_label: tr("Scale (%)"),
            button { class: "box-border flex size-7 items-center justify-center rounded border border-input bg-background p-0 {out_class}", r#type: "button", title: tr("Zoom out"), aria_label: tr("Zoom out"), disabled: value <= 5,
                onclick: move |_| onchange.call(value.saturating_sub(5).clamp(5, maximum)),
                Minus { class: "size-3.5" }
            }
            input { class: "box-border h-7 min-w-0 w-full rounded border border-input bg-background px-1 text-center text-foreground", r#type: "number", aria_label: tr("Scale (%)"), min: "5", max: "{maximum}", step: "5", value: "{value}",
                onchange: move |event| if let Ok(percent) = event.value().parse::<u16>() {
                    onchange.call(percent.clamp(5, maximum));
                }
            }
            span { "%" }
            button { class: "box-border flex size-7 items-center justify-center rounded border border-input bg-background p-0 {in_class}", r#type: "button", title: tr("Zoom in"), aria_label: tr("Zoom in"), disabled: value >= maximum,
                onclick: move |_| onchange.call(value.saturating_add(5).clamp(5, maximum)),
                icons::Plus { class: "size-3.5" }
            }
            button { class: "box-border h-7 rounded border border-input bg-background px-2 {reset_class}", r#type: "button", title: tr("Reset zoom"), aria_label: tr("Reset zoom"), disabled: maximum < 100 || value == 100,
                onclick: move |_| onchange.call(100.min(maximum)), "100%"
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
    match name {
        "quality" => {
            let quality = session
                .peek()
                .as_ref()
                .map(|active| active.get_image_quality())
                .unwrap_or_default();
            rsx! {
                for (value, label) in [("best", "Good image quality"), ("balanced", "Balanced"), ("low", "Optimize reaction time"), ("custom", "Custom")] {
                    MenuItem { label, radio: true, checked: quality == value || (quality.is_empty() && value == "balanced"), onclick: move |_| if let Some(active) = session.peek().as_ref() {
                            if value == "custom" { let mut open = toolbar.quality_dialog; open.set(true); }
                            else { active.save_image_quality(value.to_owned()); }
                        }
                    }
                }
            }
        }
        "codec" => {
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
                for (value, label, enabled) in [("auto", "Auto", true), ("vp8", "VP8", codecs.0), ("vp9", "VP9", true), ("av1", "AV1", codecs.1), ("h264", "H264", codecs.2), ("h265", "H265", codecs.3)] {
                    if enabled {
                        MenuItem { label, radio: true, checked: codec == value || (codec.is_empty() && value == "auto"),
                            onclick: move |_| if let Some(active) = session.peek().as_ref() {
                                active.set_option(OPTION_CODEC_PREFERENCE.to_owned(), value.to_owned());
                                active.update_supported_decodings();
                            }
                        }
                    }
                }
            }
        }
        "resolution" => {
            let status = state.ui_state.read();
            rsx! {
                if let Some(display) = usize::try_from(status.current_display).ok().and_then(|index| status.displays.get(index)) {
                    super::resolutions::ResolutionMenu {
                        display: display.clone(), resolutions: status.remote.resolutions.clone(),
                        local_resolution: crate::dioxus_ui::desktop::monitor_size(),
                        disabled: status.remote.permissions.get("keyboard") == Some(&false) || session.peek().as_ref().is_none_or(|active| active.get_toggle_option("view-only".to_owned())),
                        onchange: move |(width, height)| if let Some(active) = session.peek().as_ref() {
                            let status = state.ui_state.peek();
                            if status.current_display >= 0 && status.remote.permissions.get("keyboard") != Some(&false) && !active.get_toggle_option("view-only".to_owned()) {
                                active.change_resolution(status.current_display, width, height);
                            }
                        }
                    }
                }
            }
        }
        _ => rsx! {},
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
            .to_string()
    });
    let mut fps = use_signal(|| {
        state
            .session
            .peek()
            .as_ref()
            .and_then(|active| {
                active
                    .get_option(OPTION_CUSTOM_FPS.to_owned())
                    .parse::<i32>()
                    .ok()
            })
            .unwrap_or(30)
            .to_string()
    });
    let mut error = use_signal(String::new);
    let mut save = move || {
        let values = quality()
            .trim()
            .parse::<i32>()
            .ok()
            .zip(fps().trim().parse::<i32>().ok());
        let Some((quality, fps)) =
            values.filter(|(quality, fps)| (1..=100).contains(quality) && (5..=120).contains(fps))
        else {
            error.set("Enter a bitrate between 1 and 100 and an FPS between 5 and 120.".to_owned());
            return;
        };
        let active = state.session.peek();
        let Some(active) = active.as_ref() else {
            error.set("The remote session is no longer available.".to_owned());
            return;
        };
        active.save_custom_image_quality(quality);
        active.set_custom_fps(fps);
        document::eval("document.getElementById('remote-quality-dialog')?.close();");
    };
    rsx! {
        dialog {
            id: "remote-quality-dialog",
            class: "fixed top-1/2 left-1/2 m-0 max-h-[calc(100vh-32px)] w-[420px] max-w-[calc(100vw-32px)] [transform:translate(-50%,-50%)] overflow-auto rounded-xl border bg-card text-sm text-card-foreground shadow-xl backdrop:bg-black/50",
            aria_labelledby: "remote-quality-title",
            onmounted: move |_| async move {
                let eval = document::eval("const dialog = document.getElementById('remote-quality-dialog'); return await new Promise(resolve => { dialog.addEventListener('close', () => resolve(true), { once: true }); dialog.showModal(); });");
                if let Err(error) = eval.join::<bool>().await {
                    log::error!("Image quality dialog failed: {error}");
                }
                open.set(false);
            },
            onclick: move |_| { document::eval("document.getElementById('remote-quality-dialog')?.close();"); },
            form { class: "flex flex-col gap-5 p-6",
                onclick: move |event| event.stop_propagation(),
                onsubmit: move |event| { event.prevent_default(); save(); },
                h2 { id: "remote-quality-title", class: "text-lg leading-6 font-semibold", {tr("Custom Image Quality")} }
                div { class: "grid grid-cols-[minmax(0,1fr)_104px] items-center gap-x-4 gap-y-3",
                    label { r#for: "custom-quality-bitrate", {tr("Bitrate (%)")} }
                    Input { id: "custom-quality-bitrate", r#type: InputType::Number, required: true, value: quality(),
                        oninput: move |event: FormEvent| quality.set(event.value()),
                    }
                    label { r#for: "custom-quality-fps", "FPS" }
                    Input { id: "custom-quality-fps", r#type: InputType::Number, required: true, value: fps(),
                        oninput: move |event: FormEvent| fps.set(event.value()),
                    }
                }
                if !error().is_empty() { p { class: "text-xs text-destructive", role: "alert", {tr(&error())} } }
                div { class: "flex justify-end gap-2",
                    Button { button_type: "button", variant: ButtonVariant::Outline,
                        onclick: move |_| { document::eval("document.getElementById('remote-quality-dialog')?.close();"); }, {tr("Cancel")}
                    }
                    Button { button_type: "submit",
                        {tr("Save")}
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
