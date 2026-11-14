use super::prelude::*;

#[component]
pub(crate) fn AppChrome(active_page: Signal<String>, logo: String) -> Element {
    #[cfg(not(target_os = "android"))]
    {
        return rsx! {
            div { class: "flex h-8 items-center border-b border-[#404040] bg-rd-surface px-[5px] select-none",
                div { class: "flex h-8 min-w-0 flex-1 items-center",
                    div {
                        class: "flex h-8 w-28 shrink-0 items-center gap-1.5 overflow-hidden pl-[5px] text-sm text-[#4f95ff] whitespace-nowrap cursor-grab active:cursor-grabbing",
                        onmousedown: move |_| {
                            crate::dioxus_ui::native::window().drag();
                        },
                        img {
                            class: "block h-[18px] w-[18px] shrink-0",
                            src: "{logo}",
                            alt: "RustDesk",
                        }
                        span { "RustDesk" }
                    }
                    div { class: "flex h-8 self-stretch items-stretch",
                        Button {
                            css_class: if active_page() == "home" { "flex h-8 min-w-[86px] items-center gap-1.5 border-r border-[#404040] border-b-2 border-b-[#4f95ff] bg-[#303137] px-3 text-xs text-white [&>button]:h-full [&>button]:w-full" } else { "flex h-8 min-w-[86px] items-center gap-1.5 border-r border-[#404040] bg-transparent px-3 text-xs text-[#b7b9bf] hover:bg-[#333438] hover:text-[#ececec] [&>button]:h-full [&>button]:w-full" },
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            onclick: move |_| active_page.set("home".to_owned()),
                            House { class: "block h-4 w-4 shrink-0 text-[#0071ff] [&_svg]:block [&_svg]:size-full" }
                            span { "Home" }
                        }
                        if active_page().starts_with("settings:") {
                            Button {
                                css_class: "flex h-8 min-w-[86px] items-center gap-1.5 border-r border-[#404040] border-b-2 border-b-[#4f95ff] bg-[#303137] px-3 text-xs text-white [&>button]:h-full [&>button]:w-full",
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Sm,
                                onclick: move |_| active_page.set("settings:general".to_owned()),
                                Settings { class: "block h-4 w-4 shrink-0 text-[#0071ff] [&_svg]:block [&_svg]:size-full" }
                                span { "Settings" }
                                span {
                                    class: "ml-auto text-sm text-[#9da0a4]",
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        active_page.set("home".to_owned());
                                    },
                                    X { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                                }
                            }
                        }
                    }
                    div {
                        class: "h-8 min-w-6 flex-1 cursor-grab active:cursor-grabbing",
                        onmousedown: move |_| {
                            crate::dioxus_ui::native::window().drag();
                        },
                    }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#3a3b40] hover:text-white [&>button]:h-full [&>button]:w-full",
                    title: "Settings",
                    aria_label: "Settings",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| active_page.set("settings:general".to_owned()),
                    Settings { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#3a3b40] hover:text-white [&>button]:h-full [&>button]:w-full",
                    title: "Minimize",
                    aria_label: "Minimize",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| {
                        crate::dioxus_ui::native::window().set_minimized(true);
                    },
                    Minus { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#3a3b40] hover:text-white [&>button]:h-full [&>button]:w-full",
                    title: "Maximize",
                    aria_label: "Maximize",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| {
                        crate::dioxus_ui::native::window().toggle_maximized();
                    },
                    Square { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    css_class: "h-8 w-8 bg-transparent p-0 text-[#bfc0c2] hover:bg-[#c42b1c] hover:text-white [&>button]:h-full [&>button]:w-full",
                    title: "Close",
                    aria_label: "Close",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| crate::dioxus_ui::native::window().close(),
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
                    css_class: if active_page() == "home" { "min-h-[38px] min-w-[90px] bg-[#303137] text-white [&>button]:h-full [&>button]:w-full" } else { "min-h-[38px] min-w-[90px] text-[#bfc0c2] [&>button]:h-full [&>button]:w-full" },
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| active_page.set("home".to_owned()),
                    img {
                        class: "block h-[22px] w-[22px]",
                        src: "{logo}",
                        alt: "RustDesk",
                    }
                    "RustDesk"
                }
                Button {
                    css_class: if active_page().starts_with("settings:") { "min-h-[38px] min-w-[90px] bg-[#303137] text-white [&>button]:h-full [&>button]:w-full" } else { "min-h-[38px] min-w-[90px] text-[#bfc0c2] [&>button]:h-full [&>button]:w-full" },
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| active_page.set("settings:general".to_owned()),
                    Settings { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                    "Settings"
                }
            }
        }
    }
}
