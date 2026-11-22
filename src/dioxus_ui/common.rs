use super::prelude::*;

pub(super) fn update_signal<T: Send + Sync + 'static>(
    mut signal: SyncSignal<T>,
    update: impl FnOnce(&mut T),
) {
    update(&mut signal.write());
}

pub(super) fn update_ui_state(
    state: SyncSignal<DioxusUiState>,
    update: impl FnOnce(&mut DioxusUiState),
) {
    update_signal(state, update);
}

#[derive(Clone, PartialEq)]
pub(super) struct ChatMessage {
    pub(super) outgoing: bool,
    pub(super) text: String,
}

#[derive(Clone, Default)]
pub(super) struct DioxusUiState {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub(super) remote: Arc<super::session::RemoteUiState>,
    pub(super) status: String,
    pub(super) message: String,
    pub(super) connected: bool,
    pub(super) retry_available: bool,
    pub(super) password_required: bool,
    pub(super) two_factor_required: bool,
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) viewport_left: f64,
    pub(super) viewport_top: f64,
    pub(super) viewport_width: f64,
    pub(super) viewport_height: f64,
    pub(super) peer_name: String,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(super) chat: Arc<Vec<ChatMessage>>,
    pub(super) displays: Vec<DisplayInfo>,
    pub(super) current_display: i32,
    pub(super) frame_data_uri: Arc<str>,
}

#[derive(Clone, Default)]
pub(super) struct CodecUiState {
    pub(super) diagnostics: Arc<str>,
    pub(super) benchmark: Arc<str>,
    pub(super) loading: bool,
    pub(super) benchmarking: bool,
}

#[derive(Clone, Default, PartialEq)]
pub(super) struct ServiceStatusUi {
    pub(super) label: String,
    pub(super) tone: String,
}

pub(super) fn read_service_status() -> ServiceStatusUi {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let status = crate::ui_interface::get_connect_status().status_num;
        return match status {
            1 => ServiceStatusUi {
                label: "Ready".to_owned(),
                tone: "ready".to_owned(),
            },
            0 => ServiceStatusUi {
                label: "Connecting to the RustDesk network...".to_owned(),
                tone: "connecting".to_owned(),
            },
            _ => ServiceStatusUi {
                label: "Not ready. Please check your connection".to_owned(),
                tone: "warning".to_owned(),
            },
        };
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        ServiceStatusUi {
            label: "Ready".to_owned(),
            tone: "ready".to_owned(),
        }
    }
}

#[component]
pub(super) fn ServiceStatus(status: ServiceStatusUi) -> Element {
    rsx! {
        div { class: "box-border flex h-[43px] shrink-0 items-center gap-[14px] border-0 border-t border-solid border-border px-[14px] text-sm text-muted-foreground", role: "status",
            span { class: if status.tone == "ready" { "h-2 w-2 shrink-0 rounded-full bg-success" } else if status.tone == "connecting" { "h-2 w-2 shrink-0 rounded-full bg-warning" } else { "h-2 w-2 shrink-0 rounded-full bg-destructive" } }
            span { class: "min-w-0 overflow-hidden text-ellipsis whitespace-nowrap", {tr(&status.label)} }
        }
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct HelpCardData {
    pub(super) title: String,
    pub(super) message: String,
    pub(super) action: Option<String>,
    pub(super) action_url: Option<String>,
    pub(super) help: Option<String>,
    pub(super) help_url: Option<String>,
    pub(super) close_option: Option<String>,
    pub(super) dismissible: bool,
}

#[component]
pub(super) fn HelpCard(data: HelpCardData) -> Element {
    let mut dismissed = use_signal(|| false);
    if dismissed() {
        return rsx! {};
    }

    let HelpCardData {
        title,
        message,
        action,
        action_url,
        help,
        help_url,
        close_option,
        dismissible,
    } = data;
    rsx! {
        div { class: "box-border relative mt-5 w-full shrink-0 bg-muted p-5 text-sm text-foreground [&_h3]:m-0 [&_h3]:border-0 [&_h3]:p-0 [&_h3]:text-center [&_h3]:text-sm [&_h3]:font-bold [&_p]:my-2 [&_p]:leading-normal",
            if !title.is_empty() {
                h3 { "{title}" }
            }
            p { "{message}" }
            if let Some(action) = action {
                Button {
                    css_class: "mr-1.5",
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        if let Some(url) = action_url.as_deref() {
                            open_external_url(url);
                        }
                    },
                    "{action}"
                }
            }
            if let (Some(help), Some(url)) = (help, help_url) {
                Button {
                    css_class: "text-primary",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    onclick: move |_| open_external_url(&url),
                    "{help}"
                }
            }
            if dismissible {
                Button {
                    css_class: "absolute top-0.5 right-0.5",
                    title: "Close",
                    aria_label: "Close",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| {
                        dismissed.set(true);
                        if let Some(option) = close_option.as_deref() {
                            crate::ui_interface::set_local_option(option.to_owned(), "N".to_owned());
                        }
                    },
                    X { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
            }
        }
    }
}

fn translate_ui_text(text: String) -> String {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        crate::lang::translate(text)
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        crate::lang::translate_locale(text, &hbb_common::config::LocalConfig::get_option("lang"))
    }
}

pub(super) fn current_help_cards() -> Vec<HelpCardData> {
    let update_url = match crate::common::SOFTWARE_UPDATE_URL.lock() {
        Ok(update_url) => update_url.clone(),
        Err(error) => {
            log::warn!("Failed to read the software update URL: {error}");
            String::new()
        }
    };
    if !update_url.is_empty() {
        let version = crate::ui_interface::get_new_version();
        return vec![HelpCardData {
            title: translate_ui_text("Status".to_owned()),
            message: format!(
                "{} ({version}).",
                translate_ui_text(format!("new-version-of-{{{}}}-tip", crate::get_app_name()))
            ),
            action: Some("Download".to_owned()),
            action_url: Some("https://rustdesk.com/download".to_owned()),
            help: None,
            help_url: None,
            close_option: None,
            dismissible: true,
        }];
    }

    let mut cards = Vec::new();
    #[cfg(target_os = "linux")]
    {
        if crate::platform::linux::is_selinux_enforcing()
            && crate::ui_interface::get_local_option("show-selinux-help-tip".to_owned()) != "N"
        {
            cards.push(HelpCardData {
                title: translate_ui_text("Warning".to_owned()),
                message: translate_ui_text("selinux_tip".to_owned()),
                action: None,
                action_url: None,
                help: Some(translate_ui_text("Help".to_owned())),
                help_url: Some(
                    "https://rustdesk.com/docs/en/client/linux/#permissions-issue".to_owned(),
                ),
                close_option: Some("show-selinux-help-tip".to_owned()),
                dismissible: true,
            });
        }
        if crate::platform::linux::current_is_wayland() {
            cards.push(HelpCardData {
                title: translate_ui_text("Warning".to_owned()),
                message: translate_ui_text("wayland_experiment_tip".to_owned()),
                action: None,
                action_url: None,
                help: Some(translate_ui_text("Help".to_owned())),
                help_url: Some(
                    "https://rustdesk.com/docs/en/client/linux/#x11-required".to_owned(),
                ),
                close_option: None,
                dismissible: false,
            });
        } else if crate::platform::linux::is_login_wayland() {
            cards.push(HelpCardData {
                title: translate_ui_text("Warning".to_owned()),
                message: translate_ui_text(
                    "Login screen using Wayland is not supported".to_owned(),
                ),
                action: None,
                action_url: None,
                help: Some(translate_ui_text("Help".to_owned())),
                help_url: Some(
                    "https://rustdesk.com/docs/en/client/linux/#login-screen".to_owned(),
                ),
                close_option: None,
                dismissible: false,
            });
        }
    }

    cards
}
