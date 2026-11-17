use super::super::prelude::*;

#[component]
pub(super) fn Sidebar() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let mut active_page = state.active_page;
    let help_cards = if state.outgoing_only || (state.help_card_dismissed)() {
        Vec::new()
    } else {
        current_help_cards()
    };
    let service_status = (state.service_status)();

    rsx! {
        aside { class: if state.incoming_only { "flex w-[280px] shrink-0 flex-col gap-4 overflow-auto border-r bg-card p-4" } else { "flex w-[201px] shrink-0 flex-col gap-4 overflow-auto border-r bg-card p-4" },
            div { class: "text-xs leading-relaxed text-muted-foreground",
                if !state.outgoing_only {
                    div { class: "mb-2 text-base font-semibold text-foreground", {tr("Your Desktop")} }
                    p { {tr("Your desktop can be accessed with this ID and password.")} }
                } else {
                    p { {tr("You can connect to other devices, but other devices cannot connect to this device.")} }
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
            if state.outgoing_only {
                div { class: "mt-auto px-3 py-1.5",
                    Button {
                        title: "Settings",
                        aria_label: "Settings",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconSm,
                        onclick: move |_| active_page.set("settings:general".to_owned()),
                        Settings { class: "block h-[22px] w-[22px] [&_svg]:block [&_svg]:size-full" }
                    }
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
        Card { css_class: "shrink-0 px-3",
            div { class: "flex items-center justify-between text-sm text-muted-foreground",
                span { "ID" }
                Button {
                    title: "Settings",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| active_page.set("settings:general".to_owned()),
                    Ellipsis { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
            }
            div { class: "flex min-w-0 items-center gap-1",
                span {
                    ondoubleclick: move |_| copy_text_to_clipboard((state.local_device_id)()),
                    class: "min-w-0 flex-1 truncate text-lg font-medium",
                    if (state.local_device_id)().is_empty() { {tr("Loading...")} } else { "{state.local_device_id}" }
                }
            }
        }
        Card { css_class: "shrink-0 px-3",
            span { class: "text-sm text-muted-foreground", {tr("One-time Password")} }
            div { class: "flex items-center gap-1",
                code {
                    ondoubleclick: move |_| copy_text_to_clipboard((state.temporary_password)()),
                    class: "min-w-0 flex-1 truncate text-sm",
                    if (state.temporary_password)().is_empty() { "••••••" } else { "{state.temporary_password}" }
                }
                Button {
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
                    RefreshCw { class: "block h-[22px] w-[22px] shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    title: "Change Password",
                    aria_label: "Change Password",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| active_page.set("settings:security".to_owned()),
                    icons::Pencil { class: "block h-[22px] w-[22px] [&_svg]:block [&_svg]:size-full" }
                }
            }
        }
    }
}
