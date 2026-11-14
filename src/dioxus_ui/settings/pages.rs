use super::super::prelude::*;
use super::{CodecDiagnostics, SettingChoice, SettingToggle};

#[component]
pub(crate) fn SettingsContent(
    page: String,
    codec_state: SyncSignal<CodecUiState>,
    has_hardware_codec: bool,
    hardware_codec: Signal<bool>,
) -> Element {
    let version = env!("CARGO_PKG_VERSION");
    let title = match page.as_str() {
        "settings:security" => "Security",
        "settings:network" => "Network",
        "settings:codec" => "Codec",
        "settings:display" => "Display",
        "settings:account" => "Account",
        "settings:printer" => "Printer",
        "settings:about" => "About",
        _ => "General",
    };
    let subtitle = match page.as_str() {
        "settings:security" => "Control passwords, permissions, and trusted devices.",
        "settings:network" => "Configure the RustDesk ID, relay, and transport options.",
        "settings:codec" => "Inspect detected codec backends and active encode/decode paths.",
        "settings:display" => "Choose the default remote display and image behavior.",
        "settings:account" => "Manage the account used by this RustDesk client.",
        "settings:printer" => "Configure remote printer integration.",
        "settings:about" => "RustDesk desktop client information.",
        _ => "General RustDesk desktop settings.",
    };
    let (h264_supported, h265_supported) = super::supported_hwdecodings();
    let setting = |label: &'static str,
                   description: &'static str,
                   key: &'static str,
                   default_on| {
        rsx! { SettingToggle { label: label.to_owned(), description: description.to_owned(), option_key: key, default_on } }
    };

    rsx! {
        div { class: "max-w-[650px] border-b border-[#3b3d41] pb-[19px] [&_h2]:mb-[7px] [&_h2]:text-xl [&_h2]:font-normal [&_h2]:text-rd-accent [&_p]:text-xs [&_p]:text-rd-muted", h2 { "{title}" } p { "{subtitle}" } }
        match page.as_str() {
            "settings:security" => rsx! {
                SettingChoice { label: "Password verification".to_owned(), description: "Choose whether incoming connections use a temporary or permanent password.".to_owned(), option_key: OPTION_VERIFICATION_METHOD, values: vec![("use-temporary-password".to_owned(), "Use temporary password".to_owned()), ("use-permanent-password".to_owned(), "Use permanent password".to_owned())], default_value: "use-temporary-password".to_owned() }
                {setting("Enable trusted devices", "Remember trusted devices after successful authentication.", OPTION_ENABLE_TRUSTED_DEVICES, true)}
                {setting("Enable LAN discovery", "Allow this device to be discovered by other RustDesk clients on the local network.", OPTION_ENABLE_LAN_DISCOVERY, true)}
            },
            "settings:network" => rsx! {
                {setting("Use WebSocket", "Use WebSocket transport when the network requires it.", OPTION_ALLOW_WEBSOCKET, false)}
                {setting("Allow insecure TLS fallback", "Allow a fallback when the configured TLS connection cannot be established.", OPTION_ALLOW_INSECURE_TLS_FALLBACK, false)}
                {setting("Disable UDP", "Disable UDP hole punching and UDP transport.", OPTION_DISABLE_UDP, false)}
                {setting("Adaptive bitrate", "Adjust bitrate to the current network conditions.", OPTION_ENABLE_ABR, true)}
            },
            "settings:codec" => rsx! {
                div { class: "grid w-full max-w-[650px] grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-b border-[#35373a] py-[15px] max-[560px]:items-start max-[560px]:gap-3 [&_h3]:mb-[5px] [&_h3]:text-sm [&_h3]:font-normal [&_h3]:text-[#e3e3e3] [&_p]:text-xs [&_p]:text-rd-muted max-[560px]:[&_h3]:[overflow-wrap:anywhere] max-[560px]:[&_p]:[overflow-wrap:anywhere]",
                    div { class: "min-w-0", h3 { "Enable hardware codec" } p {
                        if has_hardware_codec { "Hardware codec support is available in this build." }
                        else { "This build does not include hardware codec support." }
                    } }
                    Checkbox { css_class: "h-[18px] w-[18px] shrink-0 [&>[data-name=Checkbox]]:relative [&>[data-name=Checkbox]]:grid [&>[data-name=Checkbox]]:size-[18px] [&>[data-name=Checkbox]]:place-items-center [&>[data-name=Checkbox]]:rounded-[3px] [&>[data-name=Checkbox]]:border [&>[data-name=Checkbox]]:border-[#777b83] [&>[data-name=Checkbox]]:bg-rd-panel [&>[data-name=Checkbox]]:text-white [&>[data-name=Checkbox]]:leading-none [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:absolute [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:inset-0 [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:m-0 [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:grid [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]]:place-items-center [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]>svg]:block [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]>svg]:size-[14px] [&>[data-name=Checkbox]>[data-name=CheckboxIndicator]>svg]:shrink-0 [&>[data-state=checked]]:border-rd-accent [&>[data-state=checked]]:bg-rd-accent", checked: hardware_codec(), disabled: !has_hardware_codec, aria_label: "Enable hardware codec", on_checked_change: move |enabled| {
                        hardware_codec.set(enabled);
                        let value = if enabled { "Y" } else { "N" }.to_owned();
                        spawn(async move {
                            if let Err(error) = crate::ui_interface::set_option_async(OPTION_ENABLE_HWCODEC.to_owned(), value).await {
                                log::error!("Failed to update hardware codec setting: {error:#}");
                            }
                            let diagnostics = super::super::settings::codec_json("hwcodec-diagnostics").await;
                            update_signal(codec_state, |state| state.diagnostics = Arc::from(diagnostics));
                        });
                    } }
                }
                div { class: "mt-5 flex flex-wrap gap-[9px]",
                    Button { css_class: "min-h-7 whitespace-nowrap rounded-lg border border-[#55575b] bg-[#303135] px-[14px] text-[13px] text-[#d0d1d3] hover:bg-[#3a3b3f]", variant: ButtonVariant::Outline, disabled: codec_state.read().loading, onclick: move |_| {
                        update_signal(codec_state, |state| state.loading = true);
                        let codec_state = codec_state;
                        spawn(async move {
                            let diagnostics = super::super::settings::codec_json("hwcodec-diagnostics").await;
                            update_signal(codec_state, |state| { state.diagnostics = Arc::from(diagnostics); state.loading = false; });
                        });
                    }, if codec_state.read().loading { "Detecting..." } else { "Detect backends" } }
                    Button { css_class: "min-h-7 whitespace-nowrap rounded-lg bg-[#2c8cff] px-[14px] text-[13px] text-white hover:bg-[#5b91e6]", disabled: codec_state.read().benchmarking, onclick: move |_| {
                        update_signal(codec_state, |state| state.benchmarking = true);
                        let codec_state = codec_state;
                        spawn(async move {
                            let benchmark = super::super::settings::codec_json("hwcodec-benchmark").await;
                            update_signal(codec_state, |state| { state.benchmark = Arc::from(benchmark); state.benchmarking = false; });
                        });
                    }, if codec_state.read().benchmarking { "Testing..." } else { "Test performance" } }
                }
                CodecDiagnostics { state: codec_state }
                SettingChoice { label: "Default Codec".to_owned(), description: "Select the preferred codec for remote sessions.".to_owned(), option_key: OPTION_CODEC_PREFERENCE,
                    values: {
                        let mut values = vec![("auto".to_owned(), "Auto".to_owned()), ("vp8".to_owned(), "VP8".to_owned()), ("vp9".to_owned(), "VP9".to_owned()), ("av1".to_owned(), "AV1".to_owned())];
                        if h264_supported { values.push(("h264".to_owned(), "H264".to_owned())); }
                        if h265_supported { values.push(("h265".to_owned(), "H265".to_owned())); }
                        values
                    }, default_value: "auto".to_owned() }
            },
            "settings:display" => rsx! {
                SettingChoice { label: "Default View Style".to_owned(), description: "Choose how the remote image is fitted to the window.".to_owned(), option_key: OPTION_VIEW_STYLE, values: vec![("original".to_owned(), "Scale original".to_owned()), ("adaptive".to_owned(), "Scale adaptive".to_owned())], default_value: "adaptive".to_owned() }
                SettingChoice { label: "Default Scroll Style".to_owned(), description: "Choose how scrolling behaves when the remote image is larger than the window.".to_owned(), option_key: OPTION_SCROLL_STYLE, values: vec![("scrollauto".to_owned(), "ScrollAuto".to_owned()), ("scrollbar".to_owned(), "Scrollbar".to_owned()), ("scrolledge".to_owned(), "ScrollEdge".to_owned())], default_value: "scrollauto".to_owned() }
                SettingChoice { label: "Default Image Quality".to_owned(), description: "Balance image quality and reaction time.".to_owned(), option_key: OPTION_IMAGE_QUALITY, values: vec![("best".to_owned(), "Good image quality".to_owned()), ("balanced".to_owned(), "Balanced".to_owned()), ("low".to_owned(), "Optimize reaction time".to_owned()), ("custom".to_owned(), "Custom".to_owned())], default_value: "balanced".to_owned() }
                {setting("Show remote cursor", "Show the remote pointer in the session view.", OPTION_SHOW_REMOTE_CURSOR, true)}
                {setting("Disable remote audio", "Stop receiving audio from remote sessions.", OPTION_DISABLE_AUDIO, false)}
            },
            "settings:account" => rsx! { div { class: "max-w-[650px] py-6 text-[#d4d5d7] [&_h3]:mb-2 [&_h3]:text-base [&_h3]:font-normal [&_h3]:text-[#e9e9e9] [&_p]:text-xs [&_p]:text-rd-muted", h3 { "Account" } p { "Sign in to manage address books and account settings." } Button { css_class: "min-h-7 whitespace-nowrap rounded-lg bg-[#2c8cff] px-[14px] text-[13px] text-white hover:bg-[#5b91e6]", "Sign in" } } },
            "settings:printer" => rsx! { {setting("Enable remote printer", "Allow printers to be redirected during remote sessions.", OPTION_ENABLE_REMOTE_PRINTER, true)} },
            "settings:about" => rsx! { div { class: "max-w-[650px] py-6 text-[#d4d5d7] [&_h3]:mb-2 [&_h3]:text-base [&_h3]:font-normal [&_h3]:text-[#e9e9e9] [&_p]:text-xs [&_p]:text-rd-muted [&_a]:text-xs [&_a]:text-rd-muted", h3 { "RustDesk" } p { "Remote desktop client" } p { "Version: {version}" } a { href: "https://rustdesk.com", target: "_blank", "rustdesk.com" } } },
            _ => rsx! {
                {toolbar_docking_setting()}
                {setting("Allow automatic updates", "Allow RustDesk to install updates automatically when available.", OPTION_ALLOW_AUTO_UPDATE, false)}
                {setting("Confirm before closing multiple tabs", "Ask for confirmation when closing multiple remote tabs.", OPTION_ENABLE_CONFIRM_CLOSING_TABS, true)}
                {setting("Open connection in new tab", "Open each outgoing connection in a separate tab.", OPTION_ENABLE_OPEN_NEW_CONNECTIONS_IN_TABS, true)}
                {setting("Check for software update on startup", "Check for a newer RustDesk version when starting.", OPTION_ENABLE_CHECK_UPDATE, true)}
                {setting("Enable file copy and paste", "Allow copying files through the remote session clipboard.", OPTION_ENABLE_FILE_COPY_PASTE, true)}
            },
        }
    }
}

fn toolbar_docking_setting() -> Element {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    return rsx! { crate::dioxus_ui::views::docking::DockingSetting {} };
    #[cfg(any(target_os = "android", target_os = "ios"))]
    rsx! {}
}
