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
            div { class: "box-border flex h-8 shrink-0 items-center border-0 border-b border-solid border-border bg-card select-none",
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
                            css_class: if active_page() == "home" { "flex h-8 min-w-[86px] items-center gap-1.5 border-0 bg-accent px-3 text-xs text-foreground" } else { "flex h-8 min-w-[86px] items-center gap-1.5 border-0 bg-transparent px-3 text-xs text-muted-foreground hover:bg-accent hover:text-foreground" },
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            aria_label: "Home tab",
                            onclick: move |_| {
                                if active_page.peek().starts_with("settings:") {
                                    settings_tab.set(Some(active_page.peek().clone()));
                                }
                                active_page.set("home".to_owned());
                            },
                            House { class: "block h-4 w-4 shrink-0 text-primary [&_svg]:block [&_svg]:size-full" }
                            span { {tr("Home")} }
                        }
                        if active_page().starts_with("settings:") || settings_tab().is_some() {
                            div { class: if active_page().starts_with("settings:") { "flex h-8 items-center bg-accent text-foreground" } else { "flex h-8 items-center bg-transparent text-muted-foreground" },
                                Button {
                                    css_class: "h-8 min-w-[86px] bg-transparent px-3 text-xs gap-1.5 hover:bg-accent",
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    aria_label: "Settings tab",
                                    onclick: move |event| open_settings.call(event),
                                    Settings { class: "block h-4 w-4 shrink-0 text-primary [&_svg]:block [&_svg]:size-full" }
                                    span { {tr("Settings")} }
                                }
                                Button {
                                    css_class: "mr-1 h-6 w-6 bg-transparent p-0 text-muted-foreground hover:bg-accent",
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
                    css_class: "h-8 w-8 bg-transparent p-0 text-muted-foreground hover:bg-accent hover:text-foreground",
                    title: "Settings",
                    aria_label: "Settings",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |event| open_settings.call(event),
                    icons::Menu { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-muted-foreground hover:bg-accent hover:text-foreground",
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
                    css_class: "h-8 w-8 bg-transparent p-0 text-muted-foreground hover:bg-accent hover:text-foreground",
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
                    css_class: "h-8 w-8 bg-transparent p-0 text-muted-foreground hover:bg-destructive hover:text-white",
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
            nav { class: "flex min-h-12 items-center justify-between border-b border-border bg-card px-2.5 py-1",
                Button {
                    css_class: if active_page() == "home" { "min-h-[38px] min-w-[90px] bg-accent text-foreground" } else { "min-h-[38px] min-w-[90px] text-muted-foreground" },
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
                    css_class: if active_page().starts_with("settings:") { "min-h-[38px] min-w-[90px] bg-accent text-foreground" } else { "min-h-[38px] min-w-[90px] text-muted-foreground" },
                    variant: ButtonVariant::Ghost,
                    onclick: move |event| open_settings.call(event),
                    Settings { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                    "Settings"
                }
            }
        }
    }
}
