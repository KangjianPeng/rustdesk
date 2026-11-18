use super::super::prelude::*;

#[component]
pub(super) fn SettingsPage(page: String) -> Element {
    let state = use_context::<super::super::state::AppState>();
    rsx! {
        div { class: "flex min-h-0 min-w-0 flex-1 overflow-hidden",
            aside { class: "w-[200px] shrink-0 overflow-y-auto bg-card pt-2.5",
                div { class: "flex h-[62px] items-center px-5 text-xl font-normal text-primary", {tr("Settings")} }
                SettingTab { page: "general", label: "General", icon: UiIconName::Settings, selected: page == "settings:general" }
                SettingTab { page: "security", label: "Security", icon: UiIconName::Shield, selected: page == "settings:security" }
                SettingTab { page: "network", label: "Network", icon: UiIconName::Network, selected: page == "settings:network" }
                SettingTab { page: "codec", label: "Codec", icon: UiIconName::Cpu, selected: page == "settings:codec" }
                SettingTab { page: "display", label: "Display", icon: UiIconName::Monitor, selected: page == "settings:display" }
                if !hbb_common::config::is_disable_account() {
                    SettingTab { page: "account", label: "Account", icon: UiIconName::User, selected: page == "settings:account" }
                }
                if cfg!(target_os = "windows") {
                    SettingTab { page: "printer", label: "Printer", icon: UiIconName::Printer, selected: page == "settings:printer" }
                }
                SettingTab { page: "about", label: "About", icon: UiIconName::Info, selected: page == "settings:about" }
            }
            section { class: "box-border min-h-0 min-w-0 flex-1 overflow-auto bg-background px-[15px] pb-[15px]",
                SettingsContent {
                    key: "{page}",
                    page,
                    codec_state: state.codec_state,
                    has_hardware_codec: state.has_hardware_codec,
                    hardware_codec: state.hardware_codec,
                }
            }
        }
    }
}

#[component]
fn SettingTab(
    page: &'static str,
    label: &'static str,
    icon: UiIconName,
    selected: bool,
) -> Element {
    let mut active_page = use_context::<super::super::state::AppState>().active_page;
    rsx! {
        div { class: "relative h-[42px]",
            if selected {
                span { class: "absolute left-0 top-[6px] h-[30px] w-1 bg-primary" }
            }
            Button {
                css_class: if selected { "flex h-[42px] w-full items-center bg-accent px-5 text-[15px] text-accent-foreground justify-start gap-3 [&_svg]:size-5" } else { "flex h-[42px] w-full items-center bg-transparent px-5 text-[15px] text-muted-foreground hover:bg-accent justify-start gap-3 [&_svg]:size-5" },
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Sm,
                aria_label: format!("{} {}", tr("Settings"), tr(label)),
                onclick: move |_| active_page.set(format!("settings:{page}")),
                UiIcon { icon }
                span { {tr(label)} }
            }
        }
    }
}
