use super::super::prelude::*;

#[component]
pub(crate) fn SettingToggle(
    label: String,
    description: String,
    option_key: &'static str,
    default_on: bool,
) -> Element {
    let initial = match option(option_key).as_str() {
        "Y" => true,
        "N" => false,
        _ => default_on,
    };
    let mut enabled = use_signal(|| initial);
    rsx! {
        div { class: "grid w-full max-w-[650px] grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-b border-[#35373a] py-[15px] max-[560px]:items-start max-[560px]:gap-3 [&_h3]:mb-[5px] [&_h3]:text-sm [&_h3]:font-normal [&_h3]:text-[#e3e3e3] [&_p]:text-xs [&_p]:text-rd-muted max-[560px]:[&_h3]:[overflow-wrap:anywhere] max-[560px]:[&_p]:[overflow-wrap:anywhere]",
            div { class: "min-w-0", h3 { "{label}" } p { "{description}" } }
            Checkbox {
                css_class: "h-[18px] w-[18px] shrink-0 [&>[data-name=Checkbox]]:relative [&>[data-name=Checkbox]]:grid [&>[data-name=Checkbox]]:size-[18px] [&>[data-name=Checkbox]]:place-items-center [&>[data-name=Checkbox]]:rounded-[3px] [&>[data-name=Checkbox]]:border [&>[data-name=Checkbox]]:border-[#777b83] [&>[data-name=Checkbox]]:bg-rd-panel [&>[data-name=Checkbox]]:text-white [&>[data-name=Checkbox]]:leading-none [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:absolute [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:inset-0 [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:m-0 [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:grid [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:place-items-center [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]>svg]:block [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]>svg]:size-[14px] [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]>svg]:shrink-0 [&>[data-state=checked]]:border-rd-accent [&>[data-state=checked]]:bg-rd-accent",
                checked: enabled(),
                aria_label: label.clone(),
                on_checked_change: move |value| {
                    enabled.set(value);
                    let option_key = option_key.to_owned();
                    let option_value = if value { "Y".to_owned() } else { "N".to_owned() };
                    spawn(async move {
                        if let Err(error) = crate::ui_interface::set_option_async(option_key, option_value).await {
                            log::error!("Failed to update setting: {error:#}");
                        }
                    });
                },
            }
        }
    }
}

#[component]
pub(crate) fn SettingChoice(
    label: String,
    description: String,
    option_key: &'static str,
    values: Vec<(String, String)>,
    default_value: String,
) -> Element {
    let configured = option(option_key);
    let mut selected = use_signal(|| {
        if configured.is_empty() {
            default_value.clone()
        } else {
            configured.clone()
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
    let choice_values = values.clone();
    rsx! {
        div { class: "grid w-full max-w-[650px] grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-b border-[#35373a] py-[15px] max-[560px]:items-start max-[560px]:gap-3 [&_h3]:mb-[5px] [&_h3]:text-sm [&_h3]:font-normal [&_h3]:text-[#e3e3e3] [&_p]:text-xs [&_p]:text-rd-muted max-[560px]:[&_h3]:[overflow-wrap:anywhere] max-[560px]:[&_p]:[overflow-wrap:anywhere]",
            div { class: "min-w-0", h3 { "{label}" } p { "{description}" } }
            Select {
                css_class: "relative w-[200px] shrink-0 [&_[data-name=SelectGroup]]:m-0 [&_[data-name=SelectGroup]]:list-none [&_[data-name=SelectGroup]]:p-0 [&_[data-name=SelectOption]]:flex [&_[data-name=SelectOption]]:min-h-8 [&_[data-name=SelectOption]]:w-full [&_[data-name=SelectOption]]:items-center [&_[data-name=SelectOption]]:gap-2 [&_[data-name=SelectOption]]:rounded [&_[data-name=SelectOption]]:px-2 [&_[data-name=SelectOption]]:py-1.5 [&_[data-name=SelectOption]]:text-sm [&_[data-name=SelectOption]]:leading-5 [&_[data-name=SelectOption]]:text-[#f4f4f5] [&_[data-name=SelectOption]]:whitespace-nowrap [&_[data-name=SelectOption]]:cursor-pointer [&_[data-name=SelectOption]]:transition-colors [&_[data-name=SelectOption]:hover]:bg-[#5e5e5e] [&_[data-name=SelectOption]:hover]:text-white [&_[data-name=SelectOption][aria-selected=true]]:bg-[#5e5e5e] [&_[data-name=SelectOption][aria-selected=true]]:text-white [&_[data-name=SelectContent][data-state=closed]]:hidden [&_[data-name=SelectContent][data-state=closed]]:pointer-events-none [&_[data-name=SelectContent][data-state=open]]:block [&_[data-name=SelectContent][data-state=open]]:pointer-events-auto max-[560px]:col-span-2 max-[560px]:w-full",
                default_value: Some(selected_label),
                on_change: move |value: Option<String>| {
                    if let Some(label) = value {
                        if let Some((value, _)) = choice_values.iter().find(|(_, title)| title == &label) {
                            selected.set(value.clone());
                            let option_key = option_key.to_owned();
                            let value = value.clone();
                            spawn(async move {
                                if let Err(error) = crate::ui_interface::set_option_async(option_key, value).await {
                                    log::error!("Failed to update setting: {error:#}");
                                }
                            });
                        }
                    }
                },
                SelectTrigger { css_class: "[&>button]:flex [&>button]:min-h-9 [&>button]:w-full [&>button]:min-w-0 [&>button]:items-center [&>button]:justify-between [&>button]:gap-2 [&>button]:rounded-md [&>button]:border [&>button]:border-[#464646] [&>button]:bg-[#262626] [&>button]:px-3 [&>button]:py-2 [&>button]:text-left [&>button]:text-sm [&>button]:font-medium [&>button]:leading-5 [&>button]:text-[#fafafa] [&>button]:whitespace-nowrap [&>button]:transition-colors [&>button:hover]:border-[#595959] [&>button:hover]:bg-[#5e5e5e] [&>button:focus-visible]:outline-2 [&>button:focus-visible]:outline-[#707075] [&>button:focus-visible]:outline-offset-1 [&_[data-name=SelectValue]]:min-w-0 [&_[data-name=SelectValue]]:flex-1 [&_[data-name=SelectValue]]:overflow-hidden [&_[data-name=SelectValue]]:text-ellipsis [&_[data-name=SelectValue]]:whitespace-nowrap [&_svg]:h-4 [&_svg]:w-4 [&_svg]:shrink-0 [&_svg]:text-[#a1a1aa]",
                    SelectValue { placeholder: Some(fallback_label) }
                }
                    SelectContent { css_class: "[&>[data-name=SelectContent]]:absolute [&>[data-name=SelectContent]]:left-0 [&>[data-name=SelectContent]]:top-[calc(100%+4px)] [&>[data-name=SelectContent]]:z-50 [&>[data-name=SelectContent]]:max-h-[260px] [&>[data-name=SelectContent]]:max-w-[calc(100vw-32px)] [&>[data-name=SelectContent]]:min-w-full [&>[data-name=SelectContent]]:w-max [&>[data-name=SelectContent]]:overflow-auto [&>[data-name=SelectContent]]:rounded-md [&>[data-name=SelectContent]]:border [&>[data-name=SelectContent]]:border-[#3f3f42] [&>[data-name=SelectContent]]:bg-[#2f2f2f] [&>[data-name=SelectContent]]:p-1 [&>[data-name=SelectContent]]:text-[#fafafa] [&>[data-name=SelectContent]]:shadow-[0_8px_24px_#0009] [&>[data-name=SelectContent][data-state=closed]]:hidden [&>[data-name=SelectContent][data-state=closed]]:pointer-events-none [&>[data-name=SelectContent][data-state=open]]:block [&>[data-name=SelectContent][data-state=open]]:pointer-events-auto [&>[data-name=SelectContent][data-position=Above]]:top-auto [&>[data-name=SelectContent][data-position=Above]]:bottom-[calc(100%+4px)] max-[560px]:[&>[data-name=SelectContent]]:right-0 max-[560px]:[&>[data-name=SelectContent]]:left-auto [&>[data-name=SelectContent]>div:first-child]:hidden [&>[data-name=SelectContent]>div:last-child]:hidden [&_[data-name=SelectOption]>svg]:hidden",
                    SelectGroup {
                        for (_, title) in values.iter() {
                            SelectOption { value: Some(title.clone()), "{title}" }
                        }
                    }
                }
            }
        }
    }
}
