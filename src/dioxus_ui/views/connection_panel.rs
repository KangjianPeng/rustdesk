use super::super::prelude::*;

#[component]
pub(super) fn ConnectionPanel() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let status = state.ui_state.read().clone();
    let mut peer_id = state.peer_id;
    let mut password = state.password;
    let mut two_factor_code = state.two_factor_code;
    let mut session = state.session;
    let mut connection_mode = state.connection_mode;
    let mut connection_menu_open = state.connection_menu_open;
    let ui_state = state.ui_state;
    let peer_query = peer_id().trim().to_ascii_lowercase();
    let peer_id_filter = normalize_remote_id(&peer_query).to_ascii_lowercase();
    let matching_peers: Vec<PeerCardData> = (state.peer_cards)()
        .into_iter()
        .filter(|peer| {
            !peer_id_filter.is_empty()
                && [
                    peer.id.as_str(),
                    peer.alias.as_str(),
                    peer.username.as_str(),
                    peer.hostname.as_str(),
                ]
                .iter()
                .any(|value| {
                    let value = value.to_ascii_lowercase();
                    value.contains(&peer_query)
                        || normalize_remote_id(&value).contains(&peer_id_filter)
                })
        })
        .take(6)
        .collect();

    rsx! {
        Card { css_class: "flex w-[min(360px,100%)] max-w-full flex-col rounded-[13px] border border-[#24252b] bg-rd-surface px-5 pt-6 pb-[22px] max-[841px]:w-[min(100%,420px)] max-[841px]:px-3.5 max-[841px]:py-[18px]",
            h2 { class: "mb-[15px] flex items-center gap-1 text-[19px] font-normal text-[#ececec]",
                "Control Remote Desktop"
                span {
                    class: "inline-grid h-4 w-4 place-items-center rounded-full border border-[#777] text-[11px] text-[#999]",
                    title: "Enter the remote ID",
                    CircleHelp { class: "block h-3 w-3 [&_svg]:block [&_svg]:size-full" }
                }
            }
            div { class: "flex flex-row items-center gap-2 max-[841px]:flex-wrap",
                div { class: "relative min-w-0 flex-1",
                    div {
                        onkeydown: move |event: KeyboardEvent| {
                            if event.data.key().to_string() == "Enter" {
                                event.prevent_default();
                                start_session(session, peer_id(), password(), connection_args(&connection_mode()), ui_state);
                            }
                        },
                        Input {
                            css_class: "w-full min-h-[43px] [&>input]:min-h-[43px] [&>input]:rounded-lg [&>input]:border [&>input]:border-[#55575b] [&>input]:bg-rd-surface [&>input]:px-[15px] [&>input]:py-2.5 [&>input]:text-[22px] [&>input]:text-[#ededed] [&>input]:outline-none [&>input]:placeholder:text-[#818388] [&>input:focus]:border-[#6a9be9]",
                            value: format_remote_id(&peer_id()),
                            placeholder: "Enter remote ID",
                            oninput: move |event: FormEvent| peer_id.set(format_remote_id(&event.value())),
                        }
                    }
                    if !matching_peers.is_empty() {
                        div { class: "absolute inset-x-0 top-[calc(100%+4px)] z-20 max-h-[220px] overflow-auto rounded-[5px] border border-[#4a4c51] bg-[#2b2c31] p-1 shadow-[0_6px_18px_#0008]", role: "listbox",
                            for peer in matching_peers {
                                {
                                    let suggested_id = peer.id.clone();
                                    let selected_id = suggested_id.clone();
                                    let display_name = peer_display_name(&peer);
                                    let subtitle = peer_subtitle(&peer);
                                    let online = peer.online;
                                    rsx! {
                                        Button {
                                            key: "suggestion-{peer.id}",
                                            css_class: "flex min-h-[34px] w-full min-w-0 justify-start gap-2 px-2 py-1.5 text-left text-[#d0d1d3] [&>button]:flex [&>button]:w-full [&>button]:min-w-0 [&>button]:justify-start [&>button]:gap-2",
                                            variant: ButtonVariant::Ghost,
                                            size: ButtonSize::Sm,
                                            onclick: move |_| peer_id.set(format_remote_id(&selected_id)),
                                            span { class: if online { "h-2 w-2 shrink-0 self-center rounded-full bg-[#42c49c]" } else { "h-2 w-2 shrink-0 self-center rounded-full bg-[#777a7f]" } }
                                            span { class: "min-w-0 flex-1 overflow-hidden text-ellipsis whitespace-nowrap", "{display_name}" }
                                            if !subtitle.is_empty() {
                                                span { class: "max-w-[38%] overflow-hidden text-ellipsis whitespace-nowrap text-[#9da0a6]", "{subtitle}" }
                                            }
                                            span { class: "font-mono text-[#8a8d93] whitespace-nowrap", "{suggested_id}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if status.password_required {
                    Input {
                        css_class: "min-h-[43px] min-w-0 flex-1 [&>input]:min-h-[43px] [&>input]:rounded-lg [&>input]:border [&>input]:border-[#55575b] [&>input]:bg-rd-surface [&>input]:px-[15px] [&>input]:py-2.5 [&>input]:text-[22px] [&>input]:text-[#ededed] [&>input]:outline-none [&>input]:placeholder:text-[#818388] [&>input:focus]:border-[#6a9be9]",
                        r#type: InputType::Password,
                        value: password(),
                        placeholder: "Password",
                        oninput: move |event: FormEvent| password.set(event.value()),
                    }
                    Button {
                        css_class: "min-h-[43px] whitespace-nowrap rounded-lg bg-[#2c8cff] px-[14px] text-[13px] text-white hover:bg-[#5b91e6]",
                        onclick: move |_| {
                            if let Some(active) = session.read().as_ref() {
                                active.login(String::new(), String::new(), password(), (state.remember_password)());
                                update_ui_state(ui_state, |state| {
                                    state.password_required = false;
                                    state.status = "Authenticating".to_owned();
                                });
                            }
                        },
                        "Continue"
                    }
                } else if status.two_factor_required {
                    Input {
                        css_class: "min-h-[43px] min-w-0 flex-1 [&>input]:min-h-[43px] [&>input]:rounded-lg [&>input]:border [&>input]:border-[#55575b] [&>input]:bg-rd-surface [&>input]:px-[15px] [&>input]:py-2.5 [&>input]:text-[22px] [&>input]:text-[#ededed] [&>input]:outline-none [&>input]:placeholder:text-[#818388] [&>input:focus]:border-[#6a9be9]",
                        value: two_factor_code(),
                        placeholder: "Two-factor code",
                        oninput: move |event: FormEvent| two_factor_code.set(event.value()),
                    }
                    Button {
                        css_class: "min-h-[43px] whitespace-nowrap rounded-lg bg-[#2c8cff] px-[14px] text-[13px] text-white hover:bg-[#5b91e6]",
                        onclick: move |_| {
                            if let Some(active) = session.read().as_ref() {
                                active.send2fa(two_factor_code(), false);
                                update_ui_state(ui_state, |state| {
                                    state.two_factor_required = false;
                                    state.status = "Verifying".to_owned();
                                });
                            }
                        },
                        "Verify"
                    }
                } else if session.read().is_none() {
                    div { class: "relative flex items-center gap-1 max-[841px]:ml-auto",
                        Button {
                            css_class: "min-h-[43px] whitespace-nowrap rounded-lg bg-[#2c8cff] px-[14px] text-[13px] text-white hover:bg-[#5b91e6]",
                            disabled: peer_id().trim().is_empty(),
                            onclick: move |_| start_session(session, peer_id(), password(), connection_args(&connection_mode()), ui_state),
                            "Connect"
                        }
                        Button {
                            css_class: "h-[25px] w-[25px] bg-transparent p-0 text-[19px] text-[#a8aaad] [&>button]:h-full [&>button]:w-full",
                            title: "Connection type",
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            onclick: move |_| connection_menu_open.set(!connection_menu_open()),
                            "⋮"
                        }
                        if connection_menu_open() {
                            div { class: "absolute top-[calc(100%+4px)] right-0 z-20 w-[150px] rounded-[5px] border border-[#4a4c51] bg-[#2b2c31] py-1 shadow-[0_6px_18px_#0008]",
                                for mode in ["Connect", "Transfer file", "View camera", "Terminal (beta)", "TCP tunneling"] {
                                    Button {
                                        css_class: if connection_mode() == mode {
                                            "flex w-full bg-[#3b5d91] text-white [&>button]:w-full [&>button]:justify-start [&>button]:px-3 [&>button]:py-2 [&>button]:text-left [&>button]:text-xs [&>button]:text-inherit"
                                        } else {
                                            "flex w-full text-[#d0d1d3] [&>button]:w-full [&>button]:justify-start [&>button]:px-3 [&>button]:py-2 [&>button]:text-left [&>button]:text-xs [&>button]:text-inherit hover:bg-[#3b5d91] hover:text-white hover:[&>button]:text-white"
                                        },
                                        variant: ButtonVariant::Ghost,
                                        size: ButtonSize::Sm,
                                        onclick: move |_| {
                                            connection_mode.set(mode.to_owned());
                                            connection_menu_open.set(false);
                                        },
                                        "{mode}"
                                    }
                                }
                            }
                        }
                    }
                } else if status.retry_available {
                    Button {
                        css_class: "min-h-[43px] whitespace-nowrap rounded-lg bg-[#2c8cff] px-[14px] text-[13px] text-white hover:bg-[#5b91e6]",
                        onclick: move |_| {
                            if let Some(active) = session.read().as_ref() {
                                active.reconnect(false);
                                update_ui_state(ui_state, |state| {
                                    state.status = "Connecting".to_owned();
                                    state.retry_available = false;
                                });
                            }
                        },
                        "Retry"
                    }
                } else {
                    Button {
                        css_class: "min-h-[43px] whitespace-nowrap rounded-lg bg-[#8e4650] px-[14px] text-[13px] text-white",
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| {
                            if let Some(active) = session.read().as_ref() {
                                active.close();
                            }
                            set_current_session(None);
                            session.set(None);
                            update_ui_state(ui_state, |state| {
                                state.connected = false;
                                state.status = "Disconnected".to_owned();
                                state.retry_available = false;
                            });
                        },
                        "Disconnect"
                    }
                }
            }
            if !status.status.is_empty() {
                p { class: "mt-[13px] text-center text-xs text-[#79a9ee]", "{status.status}" }
            }
            if !status.message.is_empty() {
                p { class: "mt-2 [overflow-wrap:anywhere] text-center text-[11px] text-rd-muted", "{status.message}" }
            }
        }
    }
}

fn connection_args(mode: &str) -> Vec<String> {
    match mode {
        "Transfer file" => vec!["--file-transfer".to_owned()],
        "View camera" => vec!["--view-camera".to_owned()],
        "TCP tunneling" => vec!["--port-forward".to_owned()],
        "Terminal (beta)" => vec!["--terminal".to_owned()],
        _ => Vec::new(),
    }
}
