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
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4",
            form {
                class: "flex max-h-full w-full max-w-[420px] flex-col gap-4 overflow-auto rounded-xl border bg-card p-6 text-sm text-card-foreground shadow-lg",
                role: "dialog", aria_modal: "true", aria_label: tr("Remote connection"),
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
                h2 { class: "text-lg font-medium", if status.password_required { {tr("Password Required")} } else if status.two_factor_required { {tr("Two-factor code")} } else { {tr(&status.status)} } }
                if !status.message.is_empty() { p { class: "text-muted-foreground [overflow-wrap:anywhere]", {tr(&status.message)} } }
                if status.password_required {
                    Input {
                        css_class: "w-full",
                        r#type: InputType::Password,
                        placeholder: "Password", value: password(),
                        oninput: move |event: FormEvent| password.set(event.value()),
                    }
                    label { class: "flex items-center gap-2 text-xs text-muted-foreground",
                        Checkbox { checked: remember(), aria_label: "Remember password", on_checked_change: move |value| remember.set(value) }
                        {tr("Remember password")}
                    }
                } else if status.two_factor_required {
                    Input {
                        css_class: "w-full",
                        placeholder: "Two-factor code", value: code(),
                        oninput: move |event: FormEvent| code.set(event.value()),
                    }
                } else if !status.retry_available && status.status != "Connection error" {
                    span { class: "mx-auto h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" }
                }
                div { class: "flex justify-end gap-2",
                    Button { button_type: "button",
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| super::close_remote(session),
                        {tr("Cancel")}
                    }
                    if authenticating {
                        Button { button_type: "submit",
                            if status.two_factor_required { {tr("Verify")} } else { {tr("OK")} }
                        }
                    } else if status.retry_available {
                        Button { button_type: "button",
                            onclick: move |_| {
                                update_ui_state(ui_state, |state| {
                                    *state = DioxusUiState { status: "Connecting".to_owned(), ..Default::default() };
                                });
                                if let Some(active) = session.peek().as_ref() { active.reconnect(false); }
                            },
                            {tr("Retry")}
                        }
                    } else if status.status == "Connection error" {
                        Button { button_type: "button",
                            onclick: move |_| super::close_remote(session),
                            {tr("OK")}
                        }
                    }
                }
            }
        }
    }
}
