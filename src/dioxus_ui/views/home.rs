use super::super::prelude::*;
use super::home_page::HomePage;

#[component]
pub(crate) fn Home() -> Element {
    let startup_args = use_context::<Vec<String>>();
    let remote_window = super::super::runtime::is_remote_window(&startup_args);
    let active_page = use_signal(|| "home".to_owned());
    let settings_tab = use_signal(|| None::<String>);
    let peer_id = use_signal(|| {
        if startup_args.len() > 1 && startup_args[0].starts_with("--") {
            format_remote_id(&startup_args[1])
        } else {
            format_remote_id(&local_ui_option(LAST_REMOTE_ID_UI_OPTION))
        }
    });
    let password = use_signal(String::new);
    let two_factor_code = use_signal(String::new);
    let remember_password = use_signal(|| true);
    let connection_mode = use_signal(|| "Connect".to_owned());
    let connection_menu_open = use_signal(|| false);
    let peer_tab = use_signal(|| "recent".to_owned());
    let selected_peer = use_signal(String::new);
    let peer_search = use_signal(|| false);
    let peer_search_text = use_signal(String::new);
    let peer_view_mode = use_signal(
        || match local_ui_option(OPTION_UI_PEER_CARD_TYPE).as_str() {
            "1" => "tile".to_owned(),
            "2" => "list".to_owned(),
            _ => "grid".to_owned(),
        },
    );
    let peer_sort =
        use_signal(|| normalize_peer_sort(&local_ui_option(OPTION_UI_PEER_SORTING)).to_owned());
    let peer_refresh = use_signal(|| 0_u64);
    let peer_cards = use_signal(Vec::new);
    let session = use_signal(|| None::<Session<DioxusHandler>>);
    let ui_state = use_signal_sync(DioxusUiState::default);
    let codec_state = use_signal_sync(|| CodecUiState {
        loading: true,
        ..Default::default()
    });
    let mut service_status = use_signal(read_service_status);
    let local_device_id = use_signal(String::new);
    let temporary_password = use_signal(String::new);
    let hardware_codec = use_signal(|| option(OPTION_ENABLE_HWCODEC) != "N");
    let has_hardware_codec = crate::ui_interface::has_hwcodec();
    let incoming_only = hbb_common::config::is_incoming_only();
    let outgoing_only = hbb_common::config::is_outgoing_only();
    let help_card_dismissed = use_signal(|| local_ui_option(OPTION_HIDE_HELP_CARDS) == "Y");
    use_context_provider(|| super::super::state::AppState {
        active_page,
        peer_id,
        password,
        two_factor_code,
        remember_password,
        connection_mode,
        connection_menu_open,
        peer_tab,
        selected_peer,
        peer_search,
        peer_search_text,
        peer_view_mode,
        peer_sort,
        peer_refresh,
        peer_cards,
        session,
        ui_state,
        codec_state,
        service_status,
        local_device_id,
        temporary_password,
        hardware_codec,
        has_hardware_codec,
        incoming_only,
        outgoing_only,
        help_card_dismissed,
    });

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    use_effect(move || {
        dioxus::desktop::window().set_visible(true);
    });
    let initial_password = startup_args
        .iter()
        .position(|arg| arg == "--password")
        .and_then(|index| startup_args.get(index + 1).cloned())
        .or_else(|| {
            startup_args
                .get(2)
                .filter(|arg| !arg.starts_with("--"))
                .cloned()
        })
        .unwrap_or_default();

    let mut startup_session = session;
    use_future(move || async move {
        let diagnostics = super::super::settings::codec_json("hwcodec-diagnostics").await;
        update_signal(codec_state, |state| {
            state.diagnostics = Arc::from(diagnostics);
            state.loading = false;
        });
    });

    let mut initial_local_device_id = local_device_id;
    use_effect(move || {
        spawn(async move {
            initial_local_device_id.set(super::super::runtime::load_local_id().await);
        });
    });

    let mut initial_temporary_password = temporary_password;
    use_effect(move || {
        spawn(async move {
            initial_temporary_password.set(super::super::runtime::load_temporary_password().await);
        });
    });

    use_future(move || async move {
        loop {
            let status = read_service_status();
            if *service_status.peek() != status {
                service_status.set(status);
            }
            hbb_common::tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    use_effect(move || {
        let selected_peer_tab = peer_tab();
        peer_refresh();
        let current_peer_tab = peer_tab;
        let mut peer_cards = peer_cards;
        let load_peer_tab = selected_peer_tab.clone();
        spawn(async move {
            let loaded = match hbb_common::tokio::task::spawn_blocking(move || {
                load_peer_cards(&load_peer_tab)
            })
            .await
            {
                Ok(loaded) => loaded,
                Err(error) => {
                    log::error!("Failed to load peers for {selected_peer_tab}: {error}");
                    return;
                }
            };
            if current_peer_tab() == selected_peer_tab {
                peer_cards.set(loaded);
            }
        });
    });

    use_effect(move || {
        if remote_window && codec_state.read().loading {
            return;
        }
        if startup_session.peek().is_none()
            && startup_args.len() > 1
            && matches!(
                startup_args[0].as_str(),
                "--connect"
                    | "--view-camera"
                    | "--file-transfer"
                    | "--port-forward"
                    | "--terminal"
                    | "--rdp"
            )
        {
            let command = startup_args[0].clone();
            let peer = startup_args[1].clone();
            remember_remote_id(&peer);
            let mut args = vec![command];
            args.extend(
                startup_args
                    .iter()
                    .skip(2)
                    .filter(|arg| *arg != "--password")
                    .filter(|arg| *arg != &initial_password)
                    .cloned(),
            );
            let new_session = new_session(peer, initial_password.clone(), args, ui_state);
            set_current_session(Some(new_session.clone()));
            startup_session.set(Some(new_session.clone()));
            update_ui_state(ui_state, |state| {
                state.status = "Connecting".to_owned();
                state.message.clear();
                state.connected = false;
            });
            new_session.reconnect(false);
        }
    });

    let rustdesk_icon = rustdesk_icon_data_uri();

    return rsx! {
        div { class: if remote_window { "flex h-screen w-full flex-col overflow-hidden bg-[#090a0b]" } else { "flex h-screen min-h-[480px] w-full flex-col overflow-hidden bg-background max-[841px]:h-[100dvh] max-[841px]:min-h-[100dvh]" },
            if remote_window && cfg!(not(any(target_os = "android", target_os = "ios"))) {
                {desktop_remote_window()}
            } else {
                AppChrome { active_page, logo: rustdesk_icon, settings_tab }
                if active_page().starts_with("settings:") || settings_tab().is_some() {
                    div { class: if active_page().starts_with("settings:") { "flex min-h-0 flex-1" } else { "hidden" },
                        super::settings_page::SettingsPage {
                            page: if active_page().starts_with("settings:") { active_page() } else { settings_tab().unwrap_or_else(|| "settings:general".to_owned()) },
                        }
                    }
                }
                div { class: if active_page().starts_with("settings:") { "hidden" } else { "flex min-h-0 flex-1" },
                    HomePage {}
                }
            }
        }
    };
}

fn desktop_remote_window() -> Element {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    return rsx! { super::remote_window::RemoteWindow {} };
    #[cfg(any(target_os = "android", target_os = "ios"))]
    rsx! {}
}
