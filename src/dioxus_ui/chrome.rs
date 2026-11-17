use super::prelude::*;

#[component]
pub(crate) fn AppChrome(
    active_page: Signal<String>,
    logo: String,
    settings_tab: Option<Signal<Option<String>>>,
) -> Element {
    let local_tab = use_signal(|| None::<String>);
    let mut settings_tab = settings_tab.unwrap_or(local_tab);
    let open_settings = EventHandler::new(move |_: dioxus::prelude::MouseEvent| {
        let page = if active_page.peek().starts_with("settings:") {
            active_page.peek().clone()
        } else {
            settings_tab
                .peek()
                .clone()
                .unwrap_or_else(|| "settings:general".to_owned())
        };
        settings_tab.set(Some(page.clone()));
        active_page.set(page);
    });
    #[cfg(not(target_os = "android"))]
    {
        return rsx! {
            div { class: "box-border flex h-8 shrink-0 items-center border-0 border-b border-solid border-[#404040] bg-rd-surface select-none",
                div { class: "flex h-8 min-w-0 flex-1 items-center",
                    div {
                        class: "box-border flex h-8 w-[31px] shrink-0 items-center pl-[5px] pr-2.5 cursor-grab active:cursor-grabbing",
                        onmousedown: move |_| {
                            dioxus::desktop::window().drag();
                        },
                        img {
                            class: "block h-4 w-4 shrink-0",
                            src: "{logo}",
                            alt: "RustDesk",
                        }
                    }
                    div { class: "flex h-8 self-stretch items-stretch",
                        Button {
                            css_class: if active_page() == "home" { "flex h-8 min-w-[86px] items-center gap-1.5 border-0 bg-[#303137] px-3 text-xs text-white" } else { "flex h-8 min-w-[86px] items-center gap-1.5 border-0 bg-transparent px-3 text-xs text-[#b7b9bf] hover:bg-[#333438] hover:text-[#ececec]" },
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            aria_label: "Home tab",
                            onclick: move |_| {
                                if active_page.peek().starts_with("settings:") {
                                    settings_tab.set(Some(active_page.peek().clone()));
                                }
                                active_page.set("home".to_owned());
                            },
                            House { class: "block h-4 w-4 shrink-0 text-[#0071ff] [&_svg]:block [&_svg]:size-full" }
                            span { {tr("Home")} }
                        }
                        if active_page().starts_with("settings:") || settings_tab().is_some() {
                            div { class: if active_page().starts_with("settings:") { "flex h-8 items-center bg-[#303137] text-white" } else { "flex h-8 items-center bg-transparent text-[#b7b9bf]" },
                                Button {
                                    css_class: "h-8 min-w-[86px] bg-transparent px-3 text-xs gap-1.5 hover:bg-[#333438]",
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    aria_label: "Settings tab",
                                    onclick: move |event| open_settings.call(event),
                                    Settings { class: "block h-4 w-4 shrink-0 text-[#0071ff] [&_svg]:block [&_svg]:size-full" }
                                    span { {tr("Settings")} }
                                }
                                Button {
                                    css_class: "mr-1 h-6 w-6 bg-transparent p-0 text-[#9da0a4] hover:bg-[#44454a]",
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::IconSm,
                                    aria_label: "Close Settings tab",
                                    onclick: move |_| {
                                        settings_tab.set(None);
                                        if active_page.peek().starts_with("settings:") {
                                            active_page.set("home".to_owned());
                                        }
                                    },
                                    X { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                                }
                            }
                        }
                    }
                    div {
                        class: "h-8 min-w-6 flex-1 cursor-grab active:cursor-grabbing",
                        onmousedown: move |_| {
                            dioxus::desktop::window().drag();
                        },
                    }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#3a3b40] hover:text-white",
                    title: "Settings",
                    aria_label: "Settings",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |event| open_settings.call(event),
                    icons::Menu { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#3a3b40] hover:text-white",
                    title: "Minimize",
                    aria_label: "Minimize",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| {
                        dioxus::desktop::window().set_minimized(true);
                    },
                    Minus { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#3a3b40] hover:text-white",
                    title: "Maximize",
                    aria_label: "Maximize",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| {
                        dioxus::desktop::window().toggle_maximized();
                    },
                    Square { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#c42b1c] hover:text-white",
                    title: "Close",
                    aria_label: "Close",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| dioxus::desktop::window().close(),
                    X { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
            }
        };
    }

    #[cfg(target_os = "android")]
    {
        rsx! {
            nav { class: "flex min-h-12 items-center justify-between border-b border-[#404040] bg-rd-surface px-2.5 py-1",
                Button {
                    css_class: if active_page() == "home" { "min-h-[38px] min-w-[90px] bg-[#303137] text-white" } else { "min-h-[38px] min-w-[90px] text-[#bfc0c2]" },
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| {
                                if active_page.peek().starts_with("settings:") {
                                    settings_tab.set(Some(active_page.peek().clone()));
                                }
                                active_page.set("home".to_owned());
                            },
                    img {
                        class: "block h-[22px] w-[22px]",
                        src: "{logo}",
                        alt: "RustDesk",
                    }
                    "RustDesk"
                }
                Button {
                    css_class: if active_page().starts_with("settings:") { "min-h-[38px] min-w-[90px] bg-[#303137] text-white" } else { "min-h-[38px] min-w-[90px] text-[#bfc0c2]" },
                    variant: ButtonVariant::Ghost,
                    onclick: move |event| open_settings.call(event),
                    Settings { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                    "Settings"
                }
            }
        }
    }
}
