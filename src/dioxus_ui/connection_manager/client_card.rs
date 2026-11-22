use super::super::prelude::*;
use super::backend::{allow_action, push_message, Handler, STATE};
use crate::ui_cm_interface::{self, Client, InvokeUiCM};

#[component]
pub(super) fn ClientCard(
    client: ReadSignal<Client>,
    messages: Vec<String>,
    show_elevation: bool,
) -> Element {
    let mut draft = use_signal(String::new);
    let client = client.read().clone();
    let id = client.id;
    let status = if client.disconnected {
        "Disconnected"
    } else if client.authorized {
        "Connected"
    } else {
        "Waiting for your approval"
    };
    rsx! {
        Card { css_class: "my-3 block [overflow-wrap:anywhere] rounded-lg border border-border p-4",
            h3 { class: "text-base", "{client.name} ({client.peer_id})" }
            p { role: "status", "{status}" }
            if client.is_file_transfer { p { "File transfer" } }
            if client.is_terminal { p { "Terminal" } }
            div { class: "flex flex-wrap gap-2 [&>.inline-flex]:min-h-9 [&>.inline-flex]:rounded-[5px] [&>.inline-flex]:border [&>.inline-flex>button]:min-h-9 [&>.inline-flex>button]:px-[14px] [&>.inline-flex>button]:py-2",
                if !client.disconnected
                    && !client.authorized
                    && crate::ui_interface::get_option(base::config::keys::OPTION_APPROVE_MODE) != "password"
                {
                    Button { onclick: move |_| { spawn(async move {
                        if !allow_action(id).await { return; }
                        ui_cm_interface::authorize(id);
                        STATE.send_modify(|state| if let Some(client) = state.clients.get_mut(&id) { client.authorized = true; });
                    }); }, "Accept" }
                }
                if !client.disconnected {
                    Button { variant: ButtonVariant::Destructive, onclick: move |_| {
                        spawn(async move { if allow_action(id).await { ui_cm_interface::close(id); } });
                    }, if client.authorized { "Disconnect" } else { "Reject" } }
                } else {
                    Button { onclick: move |_| { ui_cm_interface::remove(id); Handler.remove_connection(id, true); }, "Dismiss" }
                }
                if show_elevation && ui_cm_interface::can_elevate() && client.authorized && !client.disconnected {
                    Button { onclick: move |_| { spawn(async move { if allow_action(id).await { ui_cm_interface::elevate_portable(id); } }); }, "Elevate" }
                }
                VoiceCallActions { client: client.clone() }
            }
            if !messages.is_empty() { div { class: "mt-3 max-h-[200px] overflow-y-auto [&_p]:whitespace-pre-wrap", role: "log", for text in messages { p { "{text}" } } } }
            if client.authorized && !client.disconnected {
                form { class: "mt-3 flex gap-2 [&>.inline-flex:first-child]:min-w-0 [&>.inline-flex:first-child]:flex-1 [&_input]:rounded-[5px] [&_input]:border [&_input]:p-2 [&>.inline-flex:last-child]:min-h-9 [&>.inline-flex:last-child]:rounded-[5px] [&>.inline-flex:last-child]:border [&>.inline-flex:last-child>button]:min-h-9 [&>.inline-flex:last-child>button]:px-[14px] [&>.inline-flex:last-child>button]:py-2", onsubmit: move |event| {
                    event.prevent_default();
                    let text = draft().trim().to_owned();
                    if !text.is_empty() { ui_cm_interface::send_chat(id, text.clone()); push_message(id, format!("You: {text}")); draft.set(String::new()); }
                },
                    Input { value: draft(), placeholder: "Message", oninput: move |event: FormEvent| draft.set(event.value()) }
                    Button { button_type: "submit", disabled: draft().trim().is_empty(), "Send" }
                }
            }
        }
    }
}

#[component]
pub(super) fn VoiceCallActions(client: ReadSignal<Client>) -> Element {
    let client = client.read();
    let id = client.id;
    rsx! {
        if client.incoming_voice_call && !client.disconnected {
            Button { onclick: move |_| { spawn(async move { if allow_action(id).await { ui_cm_interface::handle_incoming_voice_call(id, true); } }); }, "Accept voice call" }
            Button { variant: ButtonVariant::Outline, onclick: move |_| { spawn(async move { if allow_action(id).await { ui_cm_interface::handle_incoming_voice_call(id, false); } }); }, "Reject voice call" }
        }
        if client.in_voice_call && !client.disconnected {
            Button { onclick: move |_| { spawn(async move { if allow_action(id).await { ui_cm_interface::close_voice_call(id); } }); }, "End voice call" }
        }
    }
}
