use super::super::prelude::*;

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum SettingsScope {
    #[default]
    Server,
    Local,
    UserDefault,
}

impl SettingsScope {
    fn read(self, key: &str) -> String {
        match self {
            Self::Server => option(key),
            Self::Local => crate::ui_interface::get_local_option(key.to_owned()),
            Self::UserDefault => hbb_common::config::UserDefaultConfig::load().get(key),
        }
    }

    async fn write(self, key: &str, value: String) -> Result<(), String> {
        match self {
            Self::Server => crate::ui_interface::set_option_async(key.to_owned(), value)
                .await
                .map_err(|error| format!("Unable to save this setting: {error:#}")),
            Self::Local => {
                crate::ui_interface::set_local_option(key.to_owned(), value);
                Ok(())
            }
            Self::UserDefault => {
                hbb_common::config::UserDefaultConfig::load().set(key.to_owned(), value);
                Ok(())
            }
        }
    }
}

#[component]
pub(crate) fn SettingsCard(title: String, children: Element) -> Element {
    rsx! {
        Card { css_class: "mt-4 w-full max-w-[540px] px-4",
            h2 { class: "font-semibold", {tr(&title)} }
            {children}
        }
    }
}

#[component]
pub(crate) fn SettingToggle(
    label: String,
    #[props(default)] description: String,
    option_key: &'static str,
    default_on: bool,
    #[props(default)] scope: SettingsScope,
    #[props(default)] reverse: bool,
    #[props(default)] disabled: bool,
    value_override: Option<bool>,
    on_value_change: Option<EventHandler<bool>>,
) -> Element {
    let mut enabled = use_signal(|| match scope.read(option_key).as_str() {
        "Y" => true,
        "N" => false,
        _ => default_on,
    });
    let mut error = use_signal(String::new);
    let checked = value_override.unwrap_or(enabled()) != reverse;
    let change = EventHandler::new(move |checked: bool| {
        let value = checked != reverse;
        enabled.set(value);
        spawn(async move {
            match scope
                .write(option_key, if value { "Y" } else { "N" }.to_owned())
                .await
            {
                Ok(()) => {
                    error.set(String::new());
                    if let Some(handler) = on_value_change {
                        handler.call(value);
                    }
                }
                Err(message) => {
                    log::error!("Failed to update setting {option_key}: {message}");
                    error.set(message);
                }
            }
        });
    });
    rsx! {
        div { class: "grid min-h-9 grid-cols-[16px_minmax(0,1fr)] items-center gap-x-2.5",
            Checkbox {
                checked,
                disabled,
                aria_label: label.clone(),
                on_checked_change: move |value| change.call(value),
            }
            Button {
                css_class: "h-auto min-h-7 w-full justify-start p-0 text-left whitespace-normal",
                variant: ButtonVariant::Ghost,
                disabled,
                onclick: move |_| change.call(!checked),
                {tr(&label)}
            }
            if !description.is_empty() { p { class: "col-start-2 mt-1 text-xs text-rd-muted", {tr(&description)} } }
            if !error().is_empty() { p { role: "alert", class: "col-span-2 text-xs text-red-400", "{error}" } }
        }
    }
}

#[component]
pub(crate) fn SettingChoice(
    label: String,
    #[props(default)] description: String,
    option_key: &'static str,
    values: Vec<(String, String)>,
    default_value: String,
    #[props(default)] scope: SettingsScope,
    #[props(default)] radio: bool,
    #[props(default)] disabled: bool,
    on_value_change: Option<EventHandler<String>>,
) -> Element {
    let mut selected = use_signal(|| {
        let configured = scope.read(option_key);
        if configured.is_empty() {
            default_value.clone()
        } else {
            configured
        }
    });
    let fallback_label = values
        .iter()
        .find(|(value, _)| value == &default_value)
        .map(|(_, label)| label.clone())
        .or_else(|| values.first().map(|(_, label)| label.clone()))
        .unwrap_or_else(|| default_value.clone());
    let selected_label = values
        .iter()
        .find(|(value, _)| value == &selected())
        .map(|(_, label)| label.clone())
        .unwrap_or_else(|| fallback_label.clone());
    let mut error = use_signal(String::new);
    let change = EventHandler::new(move |value: String| {
        selected.set(value.clone());
        if let Some(handler) = on_value_change {
            handler.call(value.clone());
        }
        spawn(async move {
            match scope.write(option_key, value).await {
                Ok(()) => error.set(String::new()),
                Err(message) => {
                    log::error!("Failed to update setting {option_key}: {message}");
                    error.set(message);
                }
            }
        });
    });
    if radio {
        return rsx! {
            div { role: "radiogroup", aria_label: tr(&label),
                for (value, title) in &values {
                    div { role: "radio", aria_checked: selected() == *value,
                        Button {
                            css_class: "flex min-h-9 w-full bg-transparent px-2.5 py-1 text-[15px] text-[#e3e3e3] justify-start gap-2.5 text-left",
                            variant: ButtonVariant::Ghost,
                            disabled,
                            aria_label: title.clone(),
                            onclick: { let value = value.clone(); move |_| change.call(value.clone()) },
                            span { class: if selected() == *value { "box-border grid size-[18px] shrink-0 place-items-center rounded-full border-2 border-solid border-rd-accent" } else { "box-border grid size-[18px] shrink-0 place-items-center rounded-full border-2 border-solid border-[#999b9f]" },
                                if selected() == *value { span { class: "block size-2.5 rounded-full bg-rd-accent" } }
                            }
                            span { {tr(title)} }
                        }
                    }
                }
                if !error().is_empty() { p { role: "alert", class: "px-[15px] text-xs text-red-400", "{error}" } }
            }
        };
    }
    let choice_values = values.clone();
    rsx! {
        div { class: "relative px-[15px] py-2",
            if !label.is_empty() { h3 { class: "mb-2 text-[15px] font-normal", {tr(&label)} } }
            if !description.is_empty() { p { class: "mb-2 text-xs text-rd-muted", {tr(&description)} } }
            Select {
                css_class: "w-[240px] max-w-full",
                default_value: Some(selected_label.clone()),
                on_change: move |value: Option<String>| {
                    if let Some(label) = value {
                        if let Some((value, _)) = choice_values.iter().find(|(_, title)| title == &label) {
                            change.call(value.clone());
                        }
                    }
                },
                SelectTrigger {
                    span { class: "min-w-0 flex-1 truncate text-left", {tr(&selected_label)} }
                }
                    SelectContent {
                    SelectGroup {
                        for (_, title) in values.iter() {
                            SelectOption { value: Some(title.clone()), {tr(title)} }
                        }
                    }
                }
            }
            if !error().is_empty() { p { role: "alert", class: "mt-2 text-xs text-red-400", "{error}" } }
        }
    }
}

#[component]
pub(crate) fn DefaultImageQuality() -> Element {
    let scope = SettingsScope::UserDefault;
    let mut mode = use_signal(|| scope.read(OPTION_IMAGE_QUALITY));
    let mut quality = use_signal(|| scope.read(OPTION_CUSTOM_IMAGE_QUALITY));
    let mut fps = use_signal(|| scope.read(OPTION_CUSTOM_FPS));
    let mut status = use_signal(String::new);
    rsx! {
        SettingChoice { label: "Default Image Quality", radio: true, scope,
            option_key: OPTION_IMAGE_QUALITY,
            values: vec![("best".into(), "Good image quality".into()), ("balanced".into(), "Balanced".into()), ("low".into(), "Optimize reaction time".into()), ("custom".into(), "Custom".into())],
            default_value: "balanced",
            on_value_change: move |value| mode.set(value),
        }
        if mode() == "custom" {
            div { class: "ml-[38px] mr-[15px] mt-2 grid grid-cols-[minmax(0,1fr)_90px] items-center gap-3 text-[15px]",
                label { r#for: "default-quality", {tr("Bitrate (%)")} }
                input { id: "default-quality", aria_label: tr("Default bitrate (%)"), class: "box-border h-9 min-w-0 rounded border border-solid border-[#55575b] bg-rd-content px-2 text-sm", r#type: "number", min: "10", max: "100", value: quality(), oninput: move |event| { quality.set(event.value()); status.set(String::new()); } }
                label { r#for: "default-fps", "FPS" }
                input { id: "default-fps", aria_label: tr("Default FPS"), class: "box-border h-9 min-w-0 rounded border border-solid border-[#55575b] bg-rd-content px-2 text-sm", r#type: "number", min: "5", max: "120", value: fps(), oninput: move |event| { fps.set(event.value()); status.set(String::new()); } }
                div { class: "col-span-2 flex justify-end gap-2",
                    Button { variant: ButtonVariant::Outline, onclick: move |_| {
                        quality.set(scope.read(OPTION_CUSTOM_IMAGE_QUALITY));
                        fps.set(scope.read(OPTION_CUSTOM_FPS));
                        status.set(String::new());
                    }, {tr("Cancel")} }
                    Button { aria_label: "Save default image quality", onclick: move |_| {
                        let values = quality().trim().parse::<u32>().ok().zip(fps().trim().parse::<u32>().ok());
                        if let Some((quality, fps)) = values.filter(|(quality, fps)| (10..=100).contains(quality) && (5..=120).contains(fps)) {
                            let mut config = hbb_common::config::UserDefaultConfig::load();
                            config.set(OPTION_CUSTOM_IMAGE_QUALITY.to_owned(), quality.to_string());
                            config.set(OPTION_CUSTOM_FPS.to_owned(), fps.to_string());
                            status.set("Saved".to_owned());
                        } else {
                            status.set("Enter a bitrate from 10 to 100 and an FPS from 5 to 120.".to_owned());
                        }
                    }, {tr("Save")} }
                }
                if !status().is_empty() { p { role: "status", class: "col-span-2 text-xs text-rd-muted", {tr(&status())} } }
            }
        }
    }
}
