use super::super::prelude::*;

#[component]
pub(super) fn ConnectionPanel(mut suggestions_open: Signal<bool>) -> Element {
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
        Card { css_class: "relative z-20 w-[360px] max-w-full shrink-0 px-6",
            h2 { class: "flex items-center gap-2 font-semibold",
                {tr("Control Remote Desktop")}
                span {
                    class: "inline-flex h-4 w-4 shrink-0 items-center justify-center text-[#999]",
                    title: tr("Enter the remote ID"),
                    CircleHelp { class: "block h-4 w-4 [&_svg]:block [&_svg]:size-full" }
                }
            }
            div { class: "flex min-w-0 flex-col gap-3",
                div { class: "relative z-30 w-full min-w-0 border-transparent",
                    div { class: "border-transparent",
                        onmousedown: move |event| {
                            event.stop_propagation();
                            suggestions_open.set(true);
                        },
                        onkeydown: move |event: KeyboardEvent| {
                            let key = event.data.key().to_string();
                            if key == "Escape" || key == "Tab" { suggestions_open.set(false); }
                            if key == "Enter" {
                                event.prevent_default();
                                start_session(session, peer_id(), password(), connection_args(&connection_mode()), ui_state);
                            }
                        },
                        Input {
                            css_class: "w-full",
                            value: format_remote_id(&peer_id()),
                            placeholder: "Enter remote ID",
                            oninput: move |event: FormEvent| {
                                peer_id.set(format_remote_id(&event.value()));
                                suggestions_open.set(true);
                            },
                        }
                        if suggestions_open() && !matching_peers.is_empty() {
                            div { class: "absolute inset-x-0 top-full z-20 mt-1 max-h-56 overflow-auto rounded-md border bg-popover p-1 text-popover-foreground shadow-md", role: "listbox",
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
                                                css_class: "h-auto min-h-11 w-full min-w-0 justify-start text-left",
                                                variant: ButtonVariant::Ghost,
                                                size: ButtonSize::Sm,
                                                onclick: move |_| {
                                                    peer_id.set(format_remote_id(&selected_id));
                                                    suggestions_open.set(false);
                                                },
                                                span { class: if online { "h-2 w-2 shrink-0 self-center rounded-full bg-[#42c49c]" } else { "h-2 w-2 shrink-0 self-center rounded-full bg-[#777a7f]" } }
                                                span { class: "block min-w-0 flex-1",
                                                    span { class: "block truncate text-xs", title: "{display_name}", "{display_name}" }
                                                    if !subtitle.is_empty() {
                                                        span { class: "block truncate text-[11px] text-[#9da0a6]", title: "{subtitle}", "{subtitle}" }
                                                    }
                                                    if display_name != suggested_id {
                                                        span { class: "block truncate font-mono text-[11px] text-[#8a8d93]", title: "{suggested_id}", "{suggested_id}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if status.password_required {
                    Input {
                        css_class: "min-w-0 flex-1",
                        r#type: InputType::Password,
                        value: password(),
                        placeholder: "Password",
                        oninput: move |event: FormEvent| password.set(event.value()),
                    }
                    Button {
                        css_class: "shrink-0",
                        onclick: move |_| {
                            if let Some(active) = session.read().as_ref() {
                                active.login(String::new(), String::new(), password(), (state.remember_password)());
                                update_ui_state(ui_state, |state| {
                                    state.password_required = false;
                                    state.status = "Authenticating".to_owned();
                                });
                            }
                        },
                        {tr("Continue")}
                    }
                } else if status.two_factor_required {
                    Input {
                        css_class: "min-w-0 flex-1",
                        value: two_factor_code(),
                        placeholder: "Two-factor code",
                        oninput: move |event: FormEvent| two_factor_code.set(event.value()),
                    }
                    Button {
                        css_class: "shrink-0",
                        onclick: move |_| {
                            if let Some(active) = session.read().as_ref() {
                                active.send2fa(two_factor_code(), false);
                                update_ui_state(ui_state, |state| {
                                    state.two_factor_required = false;
                                    state.status = "Verifying".to_owned();
                                });
                            }
                        },
                        {tr("Verify")}
                    }
                } else if session.read().is_none() {
                    div { class: "relative flex w-full shrink-0 items-center justify-end border-transparent",
                        Button {
                            css_class: "shrink-0",
                            disabled: peer_id().trim().is_empty(),
                            aria_label: "Connect",
                            onclick: move |_| start_session(session, peer_id(), password(), connection_args(&connection_mode()), ui_state),
                            {tr("Connect")}
                        }
                        Button {
                            css_class: "ml-2",
                            title: "Connection type",
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            onclick: move |_| connection_menu_open.set(!connection_menu_open()),
                            icons::EllipsisVertical { class: "block h-[14px] w-[14px] [&_svg]:block [&_svg]:size-full" }
                        }
                        if connection_menu_open() {
                            div { class: "absolute top-full right-0 z-20 mt-1 w-40 rounded-md border bg-popover p-1 text-popover-foreground shadow-md",
                                for mode in ["Connect", "Transfer file", "View camera", "Terminal (beta)", "TCP tunneling"] {
                                    Button {
                                        css_class: if connection_mode() == mode {
                                            "w-full justify-start bg-accent"
                                        } else {
                                            "w-full justify-start"
                                        },
                                        variant: ButtonVariant::Ghost,
                                        size: ButtonSize::Sm,
                                        onclick: move |_| {
                                            connection_mode.set(mode.to_owned());
                                            connection_menu_open.set(false);
                                        },
                                        {tr(mode)}
                                    }
                                }
                            }
                        }
                    }
                } else if status.retry_available {
                    Button {
                        css_class: "shrink-0",
                        onclick: move |_| {
                            if let Some(active) = session.read().as_ref() {
                                active.reconnect(false);
                                update_ui_state(ui_state, |state| {
                                    state.status = "Connecting".to_owned();
                                    state.retry_available = false;
                                });
                            }
                        },
                        {tr("Retry")}
                    }
                } else {
                    Button {
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
                        {tr("Disconnect")}
                    }
                }
            }
            if !status.status.is_empty() {
                p { class: "mt-[13px] text-center text-xs text-[#79a9ee]", {tr(&status.status)} }
            }
            if !status.message.is_empty() {
                p { class: "mt-2 [overflow-wrap:anywhere] text-center text-[11px] text-rd-muted", {tr(&status.message)} }
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
