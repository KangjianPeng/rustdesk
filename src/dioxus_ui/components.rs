use super::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum UiIconName {
    BookUser,
    History,
    Radar,
    Star,
    Users,
    Settings,
    Shield,
    Network,
    Cpu,
    Monitor,
    User,
    Printer,
    Info,
}

#[component]
pub(super) fn UiIcon(icon: UiIconName) -> Element {
    rsx! {
        if icon == UiIconName::BookUser {
            BookUser { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::History {
            History { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Radar {
            Radar { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Star {
            Star { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Users {
            Users { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Settings {
            Settings { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Shield {
            Shield { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Network {
            Network { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Cpu {
            Cpu { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Monitor {
            Monitor { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::User {
            User { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if icon == UiIconName::Printer {
            Printer { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else {
            Info { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        }
    }
}

pub(super) fn copy_text_to_clipboard(text: String) {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        thread_local! { static CLIPBOARD: std::cell::RefCell<Option<arboard::Clipboard>> = const { std::cell::RefCell::new(None) }; }
        CLIPBOARD.with(|clipboard| {
            let mut clipboard = clipboard.borrow_mut();
            if clipboard.is_none() {
                match arboard::Clipboard::new() {
                    Ok(value) => *clipboard = Some(value),
                    Err(error) => {
                        log::warn!("Cannot open clipboard: {error}");
                        return;
                    }
                }
            }
            if let Some(clipboard) = clipboard.as_mut() {
                if let Err(error) = clipboard.set_text(text) {
                    log::warn!("Cannot copy text: {error}");
                }
            }
        });
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    copy_text_web(text);
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn copy_text_web(text: String) {
    let eval = dioxus::document::eval(
        r#"
const text = await dioxus.recv();
try {
    if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(text);
    } else {
        const input = document.createElement("textarea");
        input.value = text;
        input.style.position = "fixed";
        input.style.opacity = "0";
        document.body.append(input);
        input.select();
        const copied = document.execCommand("copy");
        input.remove();
        if (!copied) return false;
    }
    return true;
} catch (_) {
    return false;
}
"#,
    );
    if let Err(error) = eval.send(text) {
        log::warn!("Failed to send clipboard text to the UI: {error}");
        return;
    }
    spawn(async move {
        match eval.join::<bool>().await {
            Ok(true) => {}
            Ok(false) => log::warn!("The UI clipboard API rejected the copy request"),
            Err(error) => log::warn!("Failed to copy text through the UI clipboard: {error}"),
        }
    });
}

pub(super) fn open_external_url(url: &str) {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if let Err(error) = webbrowser::open(url) {
            log::warn!("Cannot open URL {url}: {error}");
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    match serde_json::to_string(url) {
        Ok(url) => {
            dioxus::document::eval(&format!(
                "window.open({url}, '_blank', 'noopener,noreferrer');"
            ));
        }
        Err(error) => log::warn!("Failed to prepare external URL {url}: {error}"),
    }
}

#[component]
pub(super) fn Button(
    #[props(into, optional)] css_class: Option<String>,
    #[props(into, optional)] id: Option<String>,
    #[props(default = ButtonVariant::default())] variant: ButtonVariant,
    #[props(default = ButtonSize::default())] size: ButtonSize,
    #[props(optional)] disabled: bool,
    #[props(into, optional)] href: Option<String>,
    #[props(into, optional)] button_type: Option<String>,
    #[props(optional)] onclick: Option<EventHandler<dioxus::prelude::MouseEvent>>,
    #[props(optional)] onpointerdown: Option<EventHandler<dioxus::prelude::PointerEvent>>,
    #[props(optional)] onpointerup: Option<EventHandler<dioxus::prelude::PointerEvent>>,
    #[props(optional)] onpointerleave: Option<EventHandler<dioxus::prelude::PointerEvent>>,
    #[props(optional)] onpointercancel: Option<EventHandler<dioxus::prelude::PointerEvent>>,
    #[props(into, optional)] aria_label: Option<String>,
    #[props(into, optional)] title: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        RustUiButton {
            class: css_class,
            onclick: move |event: dioxus::prelude::MouseEvent| {
                if !disabled {
                    if let Some(handler) = onclick { handler.call(event); }
                }
            },
            id,
            variant,
            size,
            disabled,
            href,
            button_type,
            onpointerdown,
            onpointerup,
            onpointerleave,
            onpointercancel,
            aria_label: aria_label.map(|text| tr(&text)),
            title: title.map(|text| tr(&text)),
            {children}
        }
    }
}

#[component]
pub(super) fn Card(
    #[props(into, optional)] css_class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        RustUiCard { class: css_class, {children} }
    }
}

#[component]
pub(super) fn Input(
    #[props(into, optional)] css_class: Option<String>,
    #[props(default = InputType::default())] r#type: InputType,
    #[props(into, optional)] placeholder: Option<String>,
    #[props(into, optional)] name: Option<String>,
    #[props(into, optional)] id: Option<String>,
    #[props(into, optional)] value: Option<String>,
    #[props(optional)] disabled: bool,
    #[props(optional)] readonly: bool,
    #[props(optional)] required: bool,
    #[props(optional)] oninput: Option<EventHandler<dioxus::prelude::FormEvent>>,
) -> Element {
    rsx! {
        RustUiInput {
            class: css_class,
            r#type,
            placeholder: placeholder.map(|text| tr(&text)),
            name,
            id,
            value,
            disabled,
            readonly,
            required,
            oninput,
        }
    }
}

#[component]
pub(super) fn Checkbox(
    #[props(into, optional)] css_class: Option<String>,
    #[props(default = false)] checked: bool,
    #[props(default = false)] disabled: bool,
    #[props(default, optional)] on_checked_change: Option<EventHandler<bool>>,
    #[props(default, optional)] on_change: Option<EventHandler<bool>>,
    #[props(default = "Checkbox".to_owned(), into)] aria_label: String,
) -> Element {
    rsx! {
        RustUiCheckbox {
            class: css_class,
            checked,
            disabled,
            on_checked_change,
            on_change,
            aria_label: tr(&aria_label),
        }
    }
}

#[component]
pub(super) fn Select(
    children: Element,
    #[props(into, optional)] css_class: Option<String>,
    #[props(into, optional)] default_value: Option<String>,
    #[props(optional)] on_change: EventHandler<Option<String>>,
) -> Element {
    rsx! {
        RustUiSelect { class: css_class, default_value, on_change, {children} }
    }
}

#[component]
pub(super) fn SelectTrigger(
    #[props(into, optional)] css_class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        RustUiSelectTrigger { class: css_class, {children} }
    }
}

#[component]
pub(super) fn SelectContent(
    #[props(into, optional)] css_class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        RustUiSelectContent { class: format!("w-full min-w-[200px] [&_[data-name=SelectGroup]]:flex [&_[data-name=SelectGroup]]:flex-col [&_[data-name=SelectOption]]:whitespace-nowrap {}", css_class.unwrap_or_default()), {children} }
    }
}
