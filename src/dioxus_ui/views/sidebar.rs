use super::super::prelude::*;

#[component]
pub(super) fn Sidebar(logo: String) -> Element {
    let state = use_context::<super::super::state::AppState>();
    let mut active_page = state.active_page;
    let help_cards = if state.outgoing_only || (state.help_card_dismissed)() {
        Vec::new()
    } else {
        current_help_cards()
    };
    let service_status = (state.service_status)();

    rsx! {
        aside { class: if state.incoming_only { "flex w-[280px] shrink-0 flex-col overflow-auto bg-rd-panel max-[841px]:grid max-[841px]:w-auto max-[841px]:max-h-none max-[841px]:grid-cols-2 max-[841px]:[grid-template-areas:'logo_footer'_'tip_id'_'tip_password'] max-[841px]:content-start max-[841px]:gap-x-2 max-[841px]:gap-y-1.5 max-[841px]:px-2.5 max-[841px]:py-1 max-[841px]:[grid-template-areas:'logo_footer'_'tip_id'_'tip_password']" } else { "flex w-[200px] shrink-0 flex-col overflow-auto bg-rd-panel max-[841px]:grid max-[841px]:w-auto max-[841px]:max-h-none max-[841px]:grid-cols-2 max-[841px]:[grid-template-areas:'logo_footer'_'tip_id'_'tip_password'] max-[841px]:content-start max-[841px]:gap-x-2 max-[841px]:gap-y-1.5 max-[841px]:px-2.5 max-[841px]:py-1" },
            div { class: "flex min-h-[50px] items-center gap-2 px-5 pt-3 pb-1 text-[15px] text-[#e7e8ea] max-[841px]:[grid-area:logo] max-[841px]:min-h-[42px] max-[841px]:px-1 max-[841px]:py-1.5",
                img { class: "h-[25px] w-[25px]", src: "{logo}" }
                span { "RustDesk" }
            }
            div { class: "px-4 pt-4 pb-[5px] text-xs leading-[1.35] text-[#aaaeb5] [&_p]:m-0 max-[841px]:[grid-area:tip] max-[841px]:px-1 max-[841px]:py-2",
                if !state.outgoing_only {
                    div { class: "mb-2.5 text-[19px] text-[#ededed]", "Your Desktop" }
                    p { "Your desktop can be accessed with this ID and password." }
                } else {
                    p { "You can connect to other devices, but other devices cannot connect to this device." }
                }
            }
            if !state.outgoing_only {
                LocalDeviceCards {}
                for data in help_cards {
                    HelpCard { data }
                }
            }
            if state.incoming_only {
                ServiceStatus { status: service_status }
            }
            div { class: "mt-auto flex items-center gap-[7px] px-4 pt-3 pb-[15px] pl-5 text-[10px] text-[#74777d] [&_a]:overflow-hidden [&_a]:text-inherit [&_a]:no-underline [&_a]:text-ellipsis [&_a]:whitespace-nowrap max-[841px]:[grid-area:footer] max-[841px]:mt-0 max-[841px]:justify-end max-[841px]:px-1 max-[841px]:py-2",
                Button {
                    css_class: "h-6 w-6 bg-transparent p-0 text-[#9da0a4] [&>button]:h-full [&>button]:w-full",
                    title: "Settings",
                    aria_label: "Settings",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| active_page.set("settings:general".to_owned()),
                    Settings { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                a {
                    href: "https://rustdesk.com",
                    target: "_blank",
                    "Powered by RustDesk"
                }
            }
        }
    }
}

#[component]
fn LocalDeviceCards() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let mut active_page = state.active_page;
    rsx! {
        Card { css_class: "my-2 ml-5 mr-[11px] flex w-[calc(100%-31px)] shrink-0 [&>div[data-name=Card]]:flex-row max-[841px]:[grid-area:id] max-[841px]:m-0 max-[841px]:w-full",
            div { class: "w-0.5 shrink-0 bg-[#4d85dd]" }
            div { class: "min-w-0 flex-1 pl-[7px]",
                div { class: "flex h-[25px] items-center justify-between text-[13px] text-[#9b9da1]",
                    span { "ID" }
                    Button {
                        css_class: "h-[25px] w-[25px] bg-transparent p-0 text-[19px] text-[#a8aaad] [&>button]:h-full [&>button]:w-full",
                        title: "Settings",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconSm,
                        onclick: move |_| active_page.set("settings:general".to_owned()),
                        Ellipsis { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                    }
                }
                div { class: "flex min-w-0 items-center gap-1",
                    span { class: "min-w-0 flex-1 overflow-hidden py-2 text-[20px] font-mono text-[#ededed] text-ellipsis whitespace-nowrap",
                        if (state.local_device_id)().is_empty() { "Loading..." } else { "{state.local_device_id}" }
                    }
                    if !(state.local_device_id)().is_empty() {
                        Button {
                            css_class: "h-[25px] w-[25px] shrink-0 bg-transparent p-0 text-[#bfc0c2] [&>button]:h-full [&>button]:w-full",
                            title: "Copy ID",
                            aria_label: "Copy ID",
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            onclick: move |_| copy_text_to_clipboard((state.local_device_id)()),
                            Copy { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                        }
                    }
                }
            }
        }
        Card { css_class: "my-3 mb-2.5 ml-5 mr-[11px] flex w-[calc(100%-31px)] shrink-0 [&>div[data-name=Card]]:flex-row max-[841px]:[grid-area:password] max-[841px]:m-0 max-[841px]:w-full",
            div { class: "w-0.5 shrink-0 bg-[#4d85dd]" }
            div { class: "min-w-0 flex-1 pl-[7px]",
                span { class: "block pt-1 text-xs text-[#9b9da1]", "One-time Password" }
                div { class: "flex items-center justify-between gap-1 py-1.5",
                    code { class: "overflow-hidden text-sm tracking-wider text-[#e3e3e3] text-ellipsis whitespace-nowrap",
                        if (state.temporary_password)().is_empty() { "••••••" } else { "{state.temporary_password}" }
                    }
                    if !(state.temporary_password)().is_empty() {
                        Button {
                            css_class: "h-[25px] w-[25px] bg-transparent p-0 text-[#bfc0c2] [&>button]:h-full [&>button]:w-full",
                            title: "Copy password",
                            aria_label: "Copy password",
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            onclick: move |_| copy_text_to_clipboard((state.temporary_password)()),
                            Copy { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                        }
                    }
                    Button {
                        css_class: "h-[25px] w-[25px] bg-transparent p-0 text-[#bfc0c2] [&>button]:h-full [&>button]:w-full",
                        title: "Refresh password",
                        aria_label: "Refresh password",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconSm,
                        onclick: move |_| {
                            let mut temporary_password = state.temporary_password;
                            spawn(async move {
                                let _ = hbb_common::tokio::task::spawn_blocking(
                                    crate::ui_interface::update_temporary_password,
                                ).await;
                                temporary_password.set(super::super::runtime::load_temporary_password().await);
                            });
                        },
                        RefreshCw { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                    }
                }
            }
        }
    }
}
