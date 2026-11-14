use super::super::prelude::*;

#[component]
pub(super) fn SettingsPage() -> Element {
    let state = use_context::<super::super::state::AppState>();
    rsx! {
        div { class: "flex min-h-0 flex-1 flex-col overflow-hidden min-[841px]:flex-row",
            aside { class: "grid shrink-0 grid-cols-4 bg-rd-panel pt-1 min-[841px]:block min-[841px]:w-[200px] min-[841px]:shrink-0 min-[841px]:pt-[21px]",
                div { class: "h-[62px] px-5 py-[9px] text-xl font-normal text-rd-accent max-[841px]:hidden", "Settings" }
                SettingTab { page: "general", label: "General", icon: UiIconName::Settings }
                SettingTab { page: "security", label: "Security", icon: UiIconName::Shield }
                SettingTab { page: "network", label: "Network", icon: UiIconName::Network }
                SettingTab { page: "codec", label: "Codec", icon: UiIconName::Cpu }
                SettingTab { page: "display", label: "Display", icon: UiIconName::Monitor }
                SettingTab { page: "account", label: "Account", icon: UiIconName::User }
                SettingTab { page: "printer", label: "Printer", icon: UiIconName::Printer }
                SettingTab { page: "about", label: "About", icon: UiIconName::Info }
            }
            section { class: "min-h-0 min-w-0 flex-1 overflow-auto bg-rd-window px-10 py-7 max-[841px]:min-h-0 max-[841px]:px-4 max-[841px]:py-5",
                SettingsContent {
                    page: (state.active_page)(),
                    codec_state: state.codec_state,
                    has_hardware_codec: state.has_hardware_codec,
                    hardware_codec: state.hardware_codec,
                }
            }
        }
    }
}

#[component]
fn SettingTab(page: &'static str, label: &'static str, icon: UiIconName) -> Element {
    let state = use_context::<super::super::state::AppState>();
    let mut active_page = state.active_page;
    let key = format!("settings:{page}");
    let selected = active_page() == key;
    rsx! {
        Button {
            css_class: if selected { "flex h-10 w-full items-center justify-center border-b-2 border-r-0 border-b-rd-accent bg-[#34363a] px-[5px] text-[11px] text-[#e9e9e9] min-[841px]:h-[42px] min-[841px]:border-b-0 min-[841px]:border-l-4 min-[841px]:border-l-rd-accent min-[841px]:px-[18px] min-[841px]:text-[13px] [&>button]:h-full [&>button]:w-full [&>button]:justify-center [&>button]:gap-1 min-[841px]:[&>button]:justify-start min-[841px]:[&>button]:gap-3 [&>button>svg]:hidden min-[841px]:[&>button>svg]:block hover:bg-[#323337] hover:text-[#eee] [&>button>svg]:text-[#a6a8ac]" } else { "flex h-10 w-full items-center justify-center border-b-2 border-transparent bg-transparent px-[5px] text-[11px] text-[#a1a3a7] min-[841px]:h-[42px] min-[841px]:border-b-0 min-[841px]:border-l-4 min-[841px]:border-l-transparent min-[841px]:px-[18px] min-[841px]:text-[13px] [&>button]:h-full [&>button]:w-full [&>button]:justify-center [&>button]:gap-1 min-[841px]:[&>button]:justify-start min-[841px]:[&>button]:gap-3 [&>button>svg]:hidden min-[841px]:[&>button>svg]:block hover:bg-[#323337] hover:text-[#eee] [&>button>svg]:text-[#a6a8ac]" },
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Sm,
            onclick: move |_| active_page.set(key.clone()),
            UiIcon { icon }
            span { "{label}" }
        }
    }
}
