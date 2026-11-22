use super::super::prelude::*;
use super::{
    backend::{allow_action, close_manager, push_message, Handler, STATE},
    client_card::VoiceCallActions,
};
use crate::ui_cm_interface::{self, Client, InvokeUiCM};
use base::config::keys::OPTION_APPROVE_MODE;
use dioxus::{
    desktop::{window, LogicalSize},
    html::FormValue,
};
use icons::{Clipboard, Folder, Keyboard, MessageSquare, Mouse, RotateCcw, Video, Volume2};
use registry::ui::{
    avatar::{Avatar, AvatarFallback, AvatarSize},
    tabs::{Tabs, TabsContent, TabsList, TabsTrigger, TabsVariant},
};

pub(in crate::dioxus_ui) fn window_size(chat_open: bool) -> LogicalSize<f64> {
    LogicalSize::new(if chat_open { 700.0 } else { 300.0 }, 490.0)
}

#[component]
pub(crate) fn Window() -> Element {
    let mut state = use_signal(|| STATE.borrow().clone());
    let chat_open = use_signal(|| false);
    use_future(move || async move {
        let mut rx = STATE.subscribe();
        loop {
            state.set(rx.borrow_and_update().clone());
            if rx.changed().await.is_err() {
                break;
            }
        }
    });
    use_effect(move || {
        if state.read().clients.is_empty() && chat_open() {
            resize_chat(chat_open, false);
        }
    });
    let snapshot = state.read();
    if snapshot.clients.is_empty() {
        return rsx! {
            div { class: "flex h-screen w-full flex-col bg-background text-foreground",
                header { class: "flex h-9 shrink-0 items-center border-b border-border bg-card",
                    div { class: "flex h-full flex-1 items-center px-3 text-sm", onmousedown: move |_| window().drag(), {tr("Incoming connection")} }
                    WindowButtons {}
                }
                p { class: "grid flex-1 place-items-center text-sm text-muted-foreground", role: "status", {tr("Waiting")} }
            }
        };
    }
    let ids = snapshot
        .clients
        .keys()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let selected = snapshot.clients.keys().next_back().map(i32::to_string);
    rsx! {
        // rust-ui reads default_value on mount. Key the set so a removed tab cannot stay selected.
        for ids in std::iter::once(ids) {
        Tabs { key: "{ids}", default_value: selected.clone(), class: "h-screen min-h-0 gap-0 overflow-hidden bg-background text-foreground",
            header { class: "flex h-9 shrink-0 items-center border-b border-border bg-card",
                TabsList { variant: TabsVariant::Line, class: "min-w-0 justify-start overflow-x-auto px-2 [scrollbar-width:none]",
                    for client in snapshot.clients.values() {
                        TabsTrigger { key: "{client.id}", value: client.id.to_string(), class: "max-w-28 flex-none px-2 group-data-[orientation=Horizontal]/tabs:after:bottom-0",
                            span { class: "truncate", title: "{client.name} ({client.peer_id})", {peer_name(client)} }
                        }
                    }
                }
                div { class: "h-full min-w-4 flex-1", onmousedown: move |_| window().drag() }
                WindowButtons {}
            }
            for client in snapshot.clients.values() {
                TabsContent { key: "{client.id}", value: client.id.to_string(), class: "mt-0! min-h-0 overflow-hidden",
                    div { class: "flex h-full min-h-0",
                        Connection { client: client.clone(), chat_open, show_elevation: snapshot.show_elevation }
                        if chat_open() {
                            Chat {
                                client: client.clone(), open: chat_open,
                                messages: snapshot.messages.iter().filter(|(id, _)| *id == client.id).map(|(_, text)| text.clone()).collect::<Vec<_>>(),
                            }
                        }
                    }
                }
            }
        }
        }
    }
}

#[component]
fn WindowButtons() -> Element {
    rsx! {
        Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, aria_label: tr("Minimize"), title: tr("Minimize"), onclick: move |_| window().set_minimized(true),
            Minus { class: "size-4" }
        }
        Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, aria_label: tr("Close"), title: tr("Close"), onclick: move |_| { spawn(close_manager(true)); },
            X { class: "size-4" }
        }
    }
}

fn peer_name(client: &Client) -> &str {
    if client.name.trim().is_empty() {
        &client.peer_id
    } else {
        &client.name
    }
}

#[component]
fn Connection(
    client: ReadSignal<Client>,
    chat_open: Signal<bool>,
    show_elevation: bool,
) -> Element {
    let current = client.read();
    let id = current.id;
    let name = peer_name(&current);
    let initial = name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_owned());
    let status = if current.disconnected {
        "Disconnected"
    } else if current.authorized {
        "Connected"
    } else {
        "Request access to your device"
    };
    let kind = if current.is_terminal {
        Some(tr("Terminal beta"))
    } else if current.is_file_transfer {
        Some(tr("Transfer file"))
    } else if current.is_view_camera {
        Some(tr("View camera"))
    } else if !current.port_forward.is_empty() {
        Some(format!("{}: {}", tr("TCP tunneling"), current.port_forward))
    } else {
        None
    };
    let desktop =
        !current.is_file_transfer && !current.is_terminal && current.port_forward.is_empty();
    let action_class = "min-w-0 flex-1 [&>button]:w-full";
    rsx! {
        section { class: "flex h-full w-[300px] max-w-full shrink-0 flex-col",
            div { class: "min-h-0 flex-1 overflow-y-auto p-3",
                section { class: "relative mb-3 flex min-w-0 items-start gap-3 rounded-lg bg-[#0964ce] p-3 text-white",
                    Avatar { size: AvatarSize::Lg, class: "rounded-lg after:hidden data-[size=lg]:size-[70px]",
                        AvatarFallback { class: "rounded-none bg-white/20 text-4xl font-bold text-white", "{initial}" }
                    }
                    div { class: "min-w-0 flex-1",
                        div { class: "flex min-w-0 items-start gap-1",
                            h2 { class: "min-w-0 flex-1 truncate text-xl font-semibold", title: "{name}", "{name}" }
                            if current.authorized && !current.disconnected && desktop {
                                Button { css_class: "-mt-1 -mr-1 shrink-0 [&>button]:text-white [&>button:hover]:bg-white/20 [&>button:hover]:text-white", size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, title: tr("Chat"), aria_label: tr("Chat"), onclick: move |_| { spawn(async move {
                                    if allow_action(id).await { resize_chat(chat_open, !chat_open()); }
                                }); }, MessageSquare { class: "size-4" } }
                            }
                        }
                        p { class: "truncate text-sm text-white/90", "({current.peer_id})" }
                        if let Some(kind) = kind { p { class: "mt-1 text-xs text-white/90 [overflow-wrap:anywhere]", "{kind}" } }
                        p { class: "mt-2 text-sm [overflow-wrap:anywhere]", role: "status", {tr(status)} }
                    }
                }
                if !current.disconnected && desktop {
                    Permissions { client }
                }
            }
            footer { class: "grid shrink-0 gap-2 border-t border-border px-3 py-3",
                if current.authorized && !current.disconnected {
                    VoiceCallActions { client }
                    if show_elevation && desktop && !current.is_view_camera && ui_cm_interface::can_elevate() {
                        Button { css_class: action_class, variant: ButtonVariant::Outline, onclick: move |_| { spawn(async move { if allow_action(id).await { ui_cm_interface::elevate_portable(id); } }); }, {tr("Elevate")} }
                    }
                }
                div { class: "flex gap-2",
                    if current.disconnected {
                        Button { css_class: action_class, onclick: move |_| {
                            ui_cm_interface::remove(id);
                            Handler.remove_connection(id, true);
                            if STATE.borrow().clients.is_empty() { window().close(); }
                        }, {tr("Close")} }
                    } else {
                        if !current.authorized && crate::ui_interface::get_option(OPTION_APPROVE_MODE) != "password" {
                            Button { css_class: action_class, onclick: move |_| { spawn(async move {
                                if !allow_action(id).await { return; }
                                ui_cm_interface::authorize(id);
                                STATE.send_modify(|state| if let Some(client) = state.clients.get_mut(&id) { client.authorized = true; });
                                window().set_minimized(true);
                            }); }, {tr("Accept")} }
                        }
                        Button { css_class: action_class, variant: if current.authorized { ButtonVariant::Destructive } else { ButtonVariant::Outline }, onclick: move |_| { spawn(async move {
                            if allow_action(id).await { ui_cm_interface::close(id); }
                        }); }, {tr(if current.authorized { "Disconnect" } else { "Cancel" })} }
                    }
                }
            }
        }
    }
}

#[component]
fn Permissions(client: ReadSignal<Client>) -> Element {
    let client = client.read().clone();
    let can_modify = ui_cm_interface::can_modify_permission("audio");
    rsx! {
        section { class: "rounded-lg border border-border bg-card px-3 py-3", aria_label: tr("Permissions"),
            div { class: "mb-3 flex items-center justify-between gap-3",
                h2 { class: "text-sm font-semibold", {tr("Permissions")} }
            }
            ul { class: "grid grid-cols-2 gap-2",
                if client.is_view_camera {
                    Permission { client_id: client.id, name: "audio", label: "Enable audio", enabled: client.audio, can_modify, Volume2 { class: "size-4" } }
                    Permission { client_id: client.id, name: "recording", label: "Enable recording session", enabled: client.recording, can_modify, Video { class: "size-4" } }
                } else {
                    Permission { client_id: client.id, name: "keyboard", label: "Enable keyboard/mouse", enabled: client.keyboard, can_modify: true, Keyboard { class: "size-4" } }
                    Permission { client_id: client.id, name: "clipboard", label: "Enable clipboard", enabled: client.clipboard, can_modify, Clipboard { class: "size-4" } }
                    Permission { client_id: client.id, name: "audio", label: "Enable audio", enabled: client.audio, can_modify, Volume2 { class: "size-4" } }
                    Permission { client_id: client.id, name: "file", label: "Enable file transfer", enabled: client.file, can_modify, Folder { class: "size-4" } }
                    Permission { client_id: client.id, name: "restart", label: "Enable remote restart", enabled: client.restart, can_modify, RotateCcw { class: "size-4" } }
                    Permission { client_id: client.id, name: "recording", label: "Enable recording session", enabled: client.recording, can_modify, Video { class: "size-4" } }
                    if cfg!(target_os = "windows") {
                        Permission { client_id: client.id, name: "block_input", label: "Enable blocking user input", enabled: client.block_input, can_modify, Mouse { class: "size-4" } }
                    }
                    if !crate::privacy_mode::get_supported_privacy_mode_impl().is_empty() {
                        Permission { client_id: client.id, name: "privacy_mode", label: "Enable privacy mode", enabled: client.privacy_mode, can_modify, Shield { class: "size-4" } }
                    }
                }
            }
        }
    }
}

#[component]
fn Permission(
    client_id: i32,
    name: &'static str,
    label: &'static str,
    enabled: bool,
    can_modify: bool,
    children: Element,
) -> Element {
    let label = tr(label);
    let status = tr(if enabled { "On" } else { "Off" });
    rsx! {
        li { class: "flex min-w-0 items-center gap-2 rounded-md border border-border bg-muted/30 px-2 py-2",
            div { class: if enabled { "grid size-8 shrink-0 place-items-center rounded-md bg-[#0964ce] text-white" } else { "grid size-8 shrink-0 place-items-center rounded-md bg-muted text-muted-foreground" },
                {children}
            }
            div { class: "min-w-0 flex-1",
                p { class: "truncate text-xs font-medium", title: "{label}", "{label}" }
                p { class: "text-[11px] text-muted-foreground", "{status}" }
            }
            Checkbox {
                checked: enabled,
                disabled: !can_modify,
                aria_label: "{label}: {status}",
                on_checked_change: move |value| update_permission(client_id, name, value),
            }
        }
    }
}

fn update_permission(id: i32, name: &'static str, enabled: bool) {
    spawn(async move {
        if !allow_action(id).await {
            return;
        }
        ui_cm_interface::switch_permission(id, name.to_owned(), enabled);
        STATE.send_modify(|state| {
            if let Some(client) = state.clients.get_mut(&id) {
                match name {
                    "keyboard" => client.keyboard = enabled,
                    "clipboard" => client.clipboard = enabled,
                    "audio" => client.audio = enabled,
                    "file" => client.file = enabled,
                    "restart" => client.restart = enabled,
                    "recording" => client.recording = enabled,
                    "block_input" => client.block_input = enabled,
                    "privacy_mode" => client.privacy_mode = enabled,
                    _ => {}
                }
            }
        });
    });
}

fn resize_chat(mut open: Signal<bool>, value: bool) {
    let window = window();
    window.set_inner_size(window_size(value));
    super::super::desktop::position_connection_manager(&window, value);
    open.set(value);
}

#[component]
fn Chat(client: ReadSignal<Client>, open: Signal<bool>, messages: Vec<String>) -> Element {
    let mut draft = use_signal(String::new);
    let mut reset = use_signal(|| false);
    let current = client.read();
    let id = current.id;
    rsx! {
        aside { class: "flex min-w-0 flex-1 flex-col border-l border-border",
            header { class: "flex h-10 shrink-0 items-center justify-between border-b border-border px-3",
                h2 { class: "font-semibold", {tr("Chat")} }
                Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, title: tr("Close chat"), aria_label: tr("Close chat"), onclick: move |_| resize_chat(open, false), X { class: "size-4" } }
            }
            div { class: "min-h-0 flex-1 space-y-2 overflow-y-auto p-3", role: "log",
                if messages.is_empty() { p { class: "text-muted-foreground", {tr("No messages")} } }
                for text in messages { p { class: "whitespace-pre-wrap [overflow-wrap:anywhere]", "{text}" } }
            }
            if current.authorized && !current.disconnected {
                form { class: "flex shrink-0 gap-2 border-t border-border p-3 [&>.inline-flex:first-child]:min-w-0 [&>.inline-flex:first-child]:flex-1", onsubmit: move |event| {
                    event.prevent_default();
                    let Some(FormValue::Text(draft_text)) = event.get_first("message") else { return; };
                    let text = draft_text.trim().to_owned();
                    if !text.is_empty() {
                        let you = tr("You");
                        spawn(async move {
                            if allow_action(id).await {
                                ui_cm_interface::send_chat(id, text.clone());
                                push_message(id, format!("{you}: {text}"));
                                if draft() == draft_text {
                                    draft.set(String::new());
                                    reset.set(!reset());
                                }
                            }
                        });
                    }
                },
                    Input { key: "{reset()}", name: "message", placeholder: tr("Write a message"), oninput: move |event: FormEvent| draft.set(event.value()) }
                    Button { button_type: "submit", disabled: draft().trim().is_empty(), {tr("Send")} }
                }
            }
        }
    }
}
