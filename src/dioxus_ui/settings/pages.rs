use super::super::prelude::*;
use super::{
    fields::{DefaultImageQuality, SettingsCard, SettingsScope},
    CodecDiagnostics, SettingChoice, SettingToggle,
};

#[component]
pub(crate) fn SettingsContent(
    page: String,
    codec_state: SyncSignal<CodecUiState>,
    has_hardware_codec: bool,
    hardware_codec: Signal<bool>,
) -> Element {
    let version = env!("CARGO_PKG_VERSION");
    let (h264_supported, h265_supported) = super::supported_hwdecodings();
    rsx! {
        match page.as_str() {
            "settings:security" => rsx! {
                PermissionsSettings {}
                PasswordSettings {}
                SettingsCard { title: "Security",
                    SettingToggle { label: "Enable trusted devices", option_key: OPTION_ENABLE_TRUSTED_DEVICES, default_on: true }
                    SettingToggle { label: "Deny LAN discovery", option_key: OPTION_ENABLE_LAN_DISCOVERY, default_on: true, reverse: true }
                    SettingToggle { label: "Keep awake during incoming sessions", option_key: OPTION_KEEP_AWAKE_DURING_INCOMING_SESSIONS, default_on: true }
                }
            },
            "settings:network" => rsx! {
                SettingsCard { title: "Network",
                    SettingToggle { label: "Use WebSocket", option_key: OPTION_ALLOW_WEBSOCKET, default_on: false }
                    SettingToggle { label: "Allow insecure TLS fallback", option_key: OPTION_ALLOW_INSECURE_TLS_FALLBACK, default_on: false }
                    SettingToggle { label: "Disable UDP", option_key: OPTION_DISABLE_UDP, default_on: false }
                }
            },
            "settings:codec" => rsx! { CodecDiagnostics { state: codec_state } },
            "settings:display" => rsx! {
                SettingsCard { title: "Default View Style",
                    SettingChoice { label: "Default View Style", radio: true, scope: SettingsScope::UserDefault, option_key: OPTION_VIEW_STYLE, values: choices(&[("original", "Scale original"), ("adaptive", "Scale adaptive")]), default_value: "original" }
                }
                SettingsCard { title: "Default Scroll Style",
                    SettingChoice { label: "Default Scroll Style", radio: true, scope: SettingsScope::UserDefault, option_key: OPTION_SCROLL_STYLE, values: choices(&[("scrollauto", "ScrollAuto"), ("scrollbar", "Scrollbar"), ("scrolledge", "ScrollEdge")]), default_value: "scrollauto" }
                }
                SettingsCard { title: "Default Image Quality",
                    DefaultImageQuality {}
                }
                SettingsCard { title: "Default Codec",
                SettingChoice { label: "Default Codec".to_owned(), scope: SettingsScope::UserDefault, radio: true, option_key: OPTION_CODEC_PREFERENCE,
                    values: {
                        let mut values = vec![("auto".to_owned(), "Auto".to_owned()), ("vp8".to_owned(), "VP8".to_owned()), ("vp9".to_owned(), "VP9".to_owned()), ("av1".to_owned(), "AV1".to_owned())];
                        if h264_supported { values.push(("h264".to_owned(), "H264".to_owned())); }
                        if h265_supported { values.push(("h265".to_owned(), "H265".to_owned())); }
                        values
                    }, default_value: "auto".to_owned() }
                }
                SettingsCard { title: "Other Default Options",
                    SettingToggle { label: "View only", option_key: OPTION_VIEW_ONLY, scope: SettingsScope::UserDefault, default_on: false }
                    SettingToggle { label: "Show remote cursor", option_key: OPTION_SHOW_REMOTE_CURSOR, scope: SettingsScope::UserDefault, default_on: false }
                    SettingToggle { label: "Show quality monitor", option_key: OPTION_SHOW_QUALITY_MONITOR, scope: SettingsScope::UserDefault, default_on: false }
                    SettingToggle { label: "Disable audio", option_key: OPTION_DISABLE_AUDIO, scope: SettingsScope::UserDefault, default_on: false }
                    SettingToggle { label: "Enable file copy and paste", option_key: OPTION_ENABLE_FILE_COPY_PASTE, scope: SettingsScope::UserDefault, default_on: true }
                    SettingToggle { label: "Disable clipboard", option_key: OPTION_DISABLE_CLIPBOARD, scope: SettingsScope::UserDefault, default_on: false }
                }
            },
            "settings:account" => rsx! {
                SettingsCard { title: "Account",
                    p { class: "px-[15px] pb-2 text-[15px] text-muted-foreground", {tr("Account sign-in is not yet available in the native interface.")} }
                }
            },
            "settings:printer" => rsx! {
                if cfg!(target_os = "windows") {
                    SettingsCard { title: "Printer",
                        SettingToggle { label: "Enable remote printer", option_key: OPTION_ENABLE_REMOTE_PRINTER, default_on: true }
                    }
                }
            },
            "settings:about" => rsx! {
                SettingsCard { title: "RustDesk",
                    div { class: "flex flex-col gap-3 px-[15px] pb-2 text-[15px]",
                        p { {tr("Version")} ": " {version} }
                        Button { css_class: "w-fit bg-transparent p-0 text-primary", variant: ButtonVariant::Ghost, onclick: move |_| open_external_url("https://rustdesk.com"), {tr("Website")} }
                        Button { css_class: "w-fit bg-transparent p-0 text-primary", variant: ButtonVariant::Ghost, onclick: move |_| open_external_url("https://rustdesk.com/privacy"), {tr("Privacy Statement")} }
                    }
                }
                div { class: "box-border mt-[15px] w-full max-w-[540px] rounded bg-secondary p-5 text-[15px] leading-6 text-secondary-foreground",
                    p { "RustDesk" }
                    p { {tr("Licensed under the GNU Affero General Public License v3.0")} }
                }
            },
            _ => rsx! {
                SettingsCard { title: "Language", super::super::i18n::LanguageSetting {} }
                SettingsCard { title: "Hardware Codec",
                    SettingToggle { label: "Enable hardware codec", option_key: OPTION_ENABLE_HWCODEC, default_on: true, disabled: !has_hardware_codec,
                        on_value_change: move |enabled| {
                            hardware_codec.set(enabled);
                            spawn(async move {
                                let diagnostics = super::codec_json("hwcodec-diagnostics").await;
                                update_signal(codec_state, |state| state.diagnostics = Arc::from(diagnostics));
                            });
                        }
                    }
                }
                SettingsCard { title: "Other",
                    SettingToggle { label: "Confirm before closing multiple tabs", option_key: OPTION_ENABLE_CONFIRM_CLOSING_TABS, scope: SettingsScope::Local, default_on: true }
                    SettingToggle { label: "Allow toolbar docking to all edges", option_key: OPTION_ALLOW_MULTI_EDGE_TOOLBAR_DOCK, scope: SettingsScope::Local, default_on: false }
                    SettingToggle { label: "Adaptive bitrate", option_key: OPTION_ENABLE_ABR, default_on: true }
                    SettingToggle { label: "Open connection in new tab", option_key: OPTION_ENABLE_OPEN_NEW_CONNECTIONS_IN_TABS, scope: SettingsScope::Local, default_on: true }
                    SettingToggle { label: "Reuse one connection for port forwarding", option_key: OPTION_ENABLE_PORT_FORWARD_MUX, scope: SettingsScope::Local, default_on: true }
                    SettingToggle { label: "Check for software update on startup", option_key: OPTION_ENABLE_CHECK_UPDATE, scope: SettingsScope::Local, default_on: true }
                    SettingToggle { label: "Allow automatic updates", option_key: OPTION_ALLOW_AUTO_UPDATE, default_on: false }
                    SettingToggle { label: "Enable TCP hole punching", option_key: OPTION_ENABLE_TCP_PUNCH, scope: SettingsScope::Local, default_on: true }
                    SettingToggle { label: "Enable UDP hole punching", option_key: OPTION_ENABLE_UDP_PUNCH, scope: SettingsScope::Local, default_on: true }
                    SettingToggle { label: "Enable IPv6 hole punching", option_key: OPTION_ENABLE_IPV6_PUNCH, scope: SettingsScope::Local, default_on: true }
                    SettingToggle { label: "Keep awake during outgoing sessions", option_key: OPTION_KEEP_AWAKE_DURING_OUTGOING_SESSIONS, scope: SettingsScope::Local, default_on: true }
                }
            },
        }
    }
}

fn choices(values: &[(&str, &str)]) -> Vec<(String, String)> {
    values
        .iter()
        .map(|(value, title)| ((*value).to_owned(), (*title).to_owned()))
        .collect()
}

#[component]
fn PermissionsSettings() -> Element {
    let mut mode = use_signal(|| option(OPTION_ACCESS_MODE));
    let forced = match mode().as_str() {
        "full" => Some(true),
        "view" => Some(false),
        _ => None,
    };
    rsx! {
        SettingsCard { title: "Permissions",
            SettingChoice { label: "", option_key: OPTION_ACCESS_MODE, values: choices(&[("", "Custom"), ("full", "Full Access"), ("view", "Screen Share")]), default_value: "", on_value_change: move |value| mode.set(value) }
            for (key, title) in [
                (OPTION_ENABLE_KEYBOARD, "Enable keyboard/mouse"),
                (OPTION_ENABLE_CLIPBOARD, "Enable clipboard"),
                (OPTION_ENABLE_FILE_TRANSFER, "Enable file transfer"),
                (OPTION_ENABLE_AUDIO, "Enable audio"),
                (OPTION_ENABLE_CAMERA, "Enable camera"),
                (OPTION_ENABLE_TERMINAL, "Enable terminal"),
                (OPTION_ENABLE_TUNNEL, "Enable TCP tunneling"),
                (OPTION_ENABLE_REMOTE_RESTART, "Enable remote restart"),
                (OPTION_ENABLE_RECORD_SESSION, "Enable recording session"),
                (OPTION_ALLOW_REMOTE_CONFIG_MODIFICATION, "Enable remote configuration modification"),
            ] {
                SettingToggle { key: "{key}", label: title, option_key: key, default_on: key != OPTION_ALLOW_REMOTE_CONFIG_MODIFICATION, disabled: forced.is_some(), value_override: forced }
            }
        }
    }
}

#[component]
fn PasswordSettings() -> Element {
    let mut approval = use_signal(|| option(OPTION_APPROVE_MODE));
    let mut method = use_signal(|| option(OPTION_VERIFICATION_METHOD));
    rsx! {
        SettingsCard { title: "Password",
            SettingChoice { label: "", option_key: OPTION_APPROVE_MODE, values: choices(&[("password", "Accept sessions via password"), ("click", "Accept sessions via click"), ("", "Accept sessions via both")]), default_value: "", on_value_change: move |value| approval.set(value) }
            if approval() != "click" {
                SettingChoice { label: "Password verification", radio: true, option_key: OPTION_VERIFICATION_METHOD, values: choices(&[("use-temporary-password", "Use one-time password"), ("use-permanent-password", "Use permanent password"), ("use-both-passwords", "Use both passwords")]), default_value: "use-temporary-password", on_value_change: move |value| method.set(value) }
                if method() != "use-permanent-password" {
                    SettingChoice { label: "One-time password length", option_key: OPTION_TEMPORARY_PASSWORD_LENGTH, values: choices(&[("6", "6"), ("8", "8"), ("10", "10")]), default_value: "6" }
                }
            }
        }
    }
}
