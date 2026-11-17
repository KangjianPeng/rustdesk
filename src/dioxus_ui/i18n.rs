use super::prelude::*;
use dioxus_i18n::{
    prelude::{try_use_init_i18n, DioxusI18nError, I18n, I18nConfig},
    unic_langid::{langid, LanguageIdentifier},
};

const ENGLISH: &str = include_str!("i18n/en.ftl");
const SIMPLIFIED_CHINESE: &str = include_str!("i18n/zh-CN.ftl");
const TRADITIONAL_CHINESE: &str = include_str!("i18n/zh-TW.ftl");

fn locale(preference: &str, system: &str) -> LanguageIdentifier {
    let language = if preference.is_empty() || preference == "default" {
        system
    } else {
        preference
    };
    let language = language
        .split(['.', '@'])
        .next()
        .unwrap_or_default()
        .replace('_', "-")
        .to_ascii_lowercase();
    if language.split('-').next() == Some("zh") {
        if !language.contains("hans")
            && ["hant", "tw", "hk", "mo"]
                .iter()
                .any(|region| language.split('-').any(|part| part == *region))
        {
            langid!("zh-TW")
        } else {
            langid!("zh-CN")
        }
    } else {
        langid!("en")
    }
}

fn system_locale() -> String {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        sys_locale::get_locale().unwrap_or_default()
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        String::new()
    }
}

pub(super) fn config(language: LanguageIdentifier) -> I18nConfig {
    I18nConfig::new(language)
        .with_fallback(langid!("en"))
        .with_locale((langid!("en"), ENGLISH))
        .with_locale((langid!("zh-CN"), SIMPLIFIED_CHINESE))
        .with_locale((langid!("zh-TW"), TRADITIONAL_CHINESE))
}

pub(super) fn use_language() {
    if let Err(error) = try_use_init_i18n(|| {
        config(locale(
            &crate::ui_interface::get_local_option(OPTION_LANGUAGE.to_owned()),
            &system_locale(),
        ))
    }) {
        log::error!("Cannot initialize UI translations: {error}");
    }
}

// Keep existing English option labels as stable values; only their presentation is translated.
fn message_id(key: &str) -> String {
    let mut id = String::with_capacity(key.len());
    for part in key
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
    {
        if !id.is_empty() {
            id.push('-');
        }
        id.extend(part.bytes().map(|byte| byte.to_ascii_lowercase() as char));
    }
    id
}

pub(super) fn tr(key: &str) -> String {
    let Some(i18n) = try_consume_context::<I18n>() else {
        return key.to_owned();
    };
    match i18n.try_translate(&message_id(key)) {
        Ok(text) => text,
        Err(DioxusI18nError::MessageIdNotFound(_)) => key.to_owned(),
        Err(error) => {
            log::trace!("Cannot translate UI text {key:?}: {error}");
            key.to_owned()
        }
    }
}

#[component]
pub(super) fn LanguageSetting() -> Element {
    let Some(mut i18n) = try_consume_context::<I18n>() else {
        return rsx! {};
    };
    let mut selected = use_signal(|| {
        let preference = crate::ui_interface::get_local_option(OPTION_LANGUAGE.to_owned());
        if preference.is_empty() || preference == "default" {
            String::new()
        } else {
            locale(&preference, "").to_string().to_ascii_lowercase()
        }
    });
    let mut error = use_signal(String::new);
    let name = match selected().as_str() {
        ""|"default"=> tr("System default"),
        "zh-cn" => "简体中文".to_owned(),
        "zh-tw" => "繁體中文".to_owned(),
        _ => "English".to_owned(),
    };
    rsx! {
        div { id: "ui-language", class: "px-[15px] py-2",
            Select {
                css_class: "w-[200px] max-w-full",
                default_value: selected(),
                on_change: move |value: Option<String>| if let Some(value) = value {
                    match i18n.try_set_language(locale(&value, &system_locale())) {
                        Ok(()) => {
                            crate::ui_interface::set_local_option(OPTION_LANGUAGE.to_owned(), value.clone());
                            selected.set(value);
                            error.set(String::new());
                        }
                        Err(reason) => {
                            log::error!("Cannot change UI language: {reason}");
                            error.set(tr("Unable to change language"));
                        }
                    }
                },
                SelectTrigger { span { class: "block min-w-0 flex-1 text-left", "{name}" } }
                SelectContent { SelectGroup {
                    SelectOption { value: "", {tr("System default")} }
                    SelectOption { value: "en", "English" }
                    SelectOption { value: "zh-cn", "简体中文" }
                    SelectOption { value: "zh-tw", "繁體中文" }
                } }
            }
            if !error().is_empty() { p { role: "alert", class: "mt-2 text-xs text-red-400", "{error}" } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_i18n::fluent::FluentResource;

    #[test]
    fn locales_accept_existing_preferences_and_resources_are_valid() {
        for (preference, system, expected) in [
            ("", "zh_CN.UTF-8", "zh-CN"),
            ("", "zh-Hans-CN", "zh-CN"),
            ("", "zh_HK", "zh-TW"),
            ("", "zh_TW.UTF-8", "zh-TW"),
            ("", "zh-Hant", "zh-TW"),
            ("zh-cn", "en_US", "zh-CN"),
            ("zh-tw", "en_US", "zh-TW"),
            ("en", "zh_CN", "en"),
            ("default", "zh_TW", "zh-TW"),
            ("", "de_DE", "en"),
            ("unknown", "zh_CN", "en"),
        ] {
            assert_eq!(locale(preference, system).to_string(), expected);
        }
        for source in [ENGLISH, SIMPLIFIED_CHINESE, TRADITIONAL_CHINESE] {
            FluentResource::try_new(source.to_owned())
                .unwrap_or_else(|(_, errors)| panic!("{errors:?}"));
        }
    }
}
