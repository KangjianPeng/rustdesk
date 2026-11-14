use super::super::prelude::*;

#[derive(Clone, Copy)]
struct SelectContext {
    open: Signal<bool>,
    value: Signal<Option<String>>,
    on_change: EventHandler<Option<String>>,
}

#[component]
pub(in crate::dioxus_ui) fn Select(
    children: Element,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] default_value: Option<String>,
    #[props(optional)] on_change: EventHandler<Option<String>>,
) -> Element {
    let open = use_signal(|| false);
    let value = use_signal(|| default_value);
    provide_context(SelectContext {
        open,
        value,
        on_change,
    });
    rsx! { div { class: "relative {class.as_deref().unwrap_or(\"\")}", "data-name": "Select", {children} } }
}

#[component]
pub(in crate::dioxus_ui) fn SelectTrigger(children: Element) -> Element {
    let mut context = use_context::<SelectContext>();
    rsx! {
        button {
            r#type: "button", "data-name": "SelectTrigger", aria_haspopup: "listbox", aria_expanded: "{context.open}",
            class: "inline-flex h-9 w-full items-center justify-between rounded-md border px-2 text-sm",
            onclick: move |_| { let open = *context.open.peek(); context.open.set(!open); },
            onkeydown: move |event| if event.key() == Key::Escape { context.open.set(false); },
            {children}
            span { class: "ml-2 inline-flex h-4 w-4 shrink-0 items-center justify-center",
                icons::ChevronDown { class: "block h-4 w-4" }
            }
        }
    }
}

#[component]
pub(in crate::dioxus_ui) fn SelectValue(
    #[props(into, optional)] placeholder: Option<String>,
) -> Element {
    let context = use_context::<SelectContext>();
    let value = context
        .value
        .read()
        .clone()
        .or(placeholder)
        .unwrap_or_default();
    rsx! { span { "data-name": "SelectValue", class: "min-w-0 truncate", "{value}" } }
}

#[component]
pub(in crate::dioxus_ui) fn SelectContent(children: Element) -> Element {
    let mut context = use_context::<SelectContext>();
    if !*context.open.read() {
        return rsx! {};
    }
    rsx! {
        div { class: "fixed inset-0 z-40", onclick: move |_| context.open.set(false) }
        div {
            "data-name": "SelectContent", "data-state": "open",
            class: "absolute top-full left-0 z-50 mt-1 max-h-72 min-w-full overflow-auto rounded-md border border-[#484b53] bg-[#24252b] p-1 shadow-lg",
            onkeydown: move |event| if event.key() == Key::Escape { context.open.set(false); },
            {children}
        }
    }
}

#[component]
pub(in crate::dioxus_ui) fn SelectGroup(
    children: Element,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] aria_label: Option<String>,
) -> Element {
    rsx! { ul { "data-name": "SelectGroup", role: "listbox", class, aria_label, {children} } }
}

#[component]
pub(in crate::dioxus_ui) fn SelectOption(
    children: Element,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] value: Option<String>,
) -> Element {
    let mut context = use_context::<SelectContext>();
    let selected = *context.value.read() == value;
    let keyboard_value = value.clone();
    rsx! {
        li {
            "data-name": "SelectOption", role: "option", aria_selected: "{selected}", tabindex: "0",
            class: "flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-[#343944] {class.as_deref().unwrap_or(\"\")}",
            onclick: move |_| { context.value.set(value.clone()); context.open.set(false); context.on_change.call(value.clone()); },
            onkeydown: move |event| if event.key() == Key::Enter || event.key() == Key::Character(" ".to_owned()) {
                context.value.set(keyboard_value.clone()); context.open.set(false); context.on_change.call(keyboard_value.clone());
            },
            {children}
            if selected { span { class: "ml-auto inline-flex h-4 w-4 shrink-0 items-center justify-center",
                icons::Check { class: "block h-4 w-4" }
            } }
        }
    }
}
