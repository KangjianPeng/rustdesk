use super::super::super::prelude::*;

#[component]
pub(super) fn SessionDialog() -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let session = state.session;
    let ui_state = state.ui_state;
    let status = ui_state.read().clone();
    let mut password = state.password;
    let mut code = state.two_factor_code;
    let mut remember = state.remember_password;
    let authenticating = status.password_required || status.two_factor_required;
    rsx! {
        div { class: "absolute inset-0 z-30 flex items-center justify-center bg-black/40 p-4",
            form {
                class: "flex w-full max-w-[420px] flex-col gap-4 rounded-lg border border-[#48494e] bg-rd-surface p-6 text-sm text-[#eee] shadow-xl",
                role: "dialog", aria_modal: "true", aria_label: "Remote connection",
                onsubmit: move |event| {
                    event.prevent_default();
                    if let Some(active) = session.peek().as_ref() {
                        if status.password_required {
                            active.login(String::new(), String::new(), password(), remember());
                        } else if status.two_factor_required {
                            active.send2fa(code(), false);
                        } else {
                            return;
                        }
                        update_ui_state(ui_state, |state| {
                            state.password_required = false;
                            state.two_factor_required = false;
                            state.status = "Authenticating".to_owned();
                        });
                    }
                },
                h2 { class: "text-lg font-medium", if status.password_required { "Password Required" } else if status.two_factor_required { "Two-factor code" } else { "{status.status}" } }
                if !status.message.is_empty() { p { class: "text-rd-muted [overflow-wrap:anywhere]", "{status.message}" } }
                if status.password_required {
                    Input {
                        css_class: "w-full [&>input]:h-10 [&>input]:w-full [&>input]:rounded [&>input]:border [&>input]:border-[#55575b] [&>input]:bg-rd-content [&>input]:px-3",
                        r#type: InputType::Password,
                        placeholder: "Password", value: password(),
                        oninput: move |event: FormEvent| password.set(event.value()),
                    }
                    label { class: "flex items-center gap-2 text-xs text-rd-muted",
                        Checkbox { checked: remember(), aria_label: "Remember password", on_checked_change: move |value| remember.set(value) }
                        "Remember password"
                    }
                } else if status.two_factor_required {
                    Input {
                        css_class: "w-full [&>input]:h-10 [&>input]:w-full [&>input]:rounded [&>input]:border [&>input]:border-[#55575b] [&>input]:bg-rd-content [&>input]:px-3",
                        placeholder: "Two-factor code", value: code(),
                        oninput: move |event: FormEvent| code.set(event.value()),
                    }
                } else if !status.retry_available && status.status != "Connection error" {
                    span { class: "mx-auto h-6 w-6 animate-spin rounded-full border-2 border-rd-accent border-t-transparent" }
                }
                div { class: "flex justify-end gap-2",
                    Button { css_class: "h-9 px-4", button_type: "button",
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| super::close_remote(session),
                        "Cancel"
                    }
                    if authenticating {
                        Button { css_class: "h-9 rounded bg-[#0071ff] px-4 text-white", button_type: "submit",
                            if status.two_factor_required { "Verify" } else { "OK" }
                        }
                    } else if status.retry_available {
                        Button { css_class: "h-9 rounded bg-[#0071ff] px-4 text-white", button_type: "button",
                            onclick: move |_| {
                                update_ui_state(ui_state, |state| {
                                    *state = DioxusUiState { status: "Connecting".to_owned(), ..Default::default() };
                                });
                                if let Some(active) = session.peek().as_ref() { active.reconnect(false); }
                            },
                            "Retry"
                        }
                    } else if status.status == "Connection error" {
                        Button { css_class: "h-9 rounded bg-[#0071ff] px-4 text-white", button_type: "button",
                            onclick: move |_| super::close_remote(session),
                            "OK"
                        }
                    }
                }
            }
        }
    }
}
