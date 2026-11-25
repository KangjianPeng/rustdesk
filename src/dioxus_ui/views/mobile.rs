use super::super::prelude::*;
use super::super::session::ChatMessage;
use icons::{
    ArrowLeft, ChevronDown, ChevronUp, Ellipsis, Hand, Keyboard, MessageSquare, MousePointer2,
};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
struct MobileTab {
    id: &'static str,
    label: &'static str,
    icon: UiIconName,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MobileTouchGesture {
    Idle,
    MouseTap,
    MouseMove,
    AndroidHold,
    TwoFingerTap,
    Scroll,
}

#[component]
pub fn MobileHome() -> Element {
    let mut state = use_context::<super::super::state::AppState>();
    let tabs = mobile_tabs(state.incoming_only, state.outgoing_only);
    let tab_count = tabs.len();
    let initial_tab = tabs
        .first()
        .map(|tab| tab.id.to_owned())
        .unwrap_or_else(|| "settings".to_owned());
    let home_tab = initial_tab.clone();
    let mut selected_tab = use_signal(|| initial_tab);
    let connected = state.ui_state.read().connected;
    rsx! {
        document::Stylesheet { href: asset!("/assets/mobile.css") }
        div {
            class: "mobile-shell flex min-h-0 w-full flex-col overflow-hidden bg-background text-foreground",
            tabindex: "0",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() != Key::Escape {
                    return;
                }
                if connected {
                    event.prevent_default();
                    close_mobile_session(state);
                } else if selected_tab() != home_tab {
                    event.prevent_default();
                    selected_tab.set(home_tab.clone());
                }
            },
            if connected {
                MobileRemotePage {}
            } else {
                if selected_tab() == "chat" {
                    MobileChatPage {}
                } else {
                    header { class: "mobile-header flex shrink-0 items-center justify-center border-b border-border bg-card px-4",
                        h1 { class: "max-w-full truncate text-base font-semibold", {tr("RustDesk")} }
                    }
                    main { class: if selected_tab() == "settings" { "flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden" } else { "min-h-0 min-w-0 flex-1 overflow-y-auto overscroll-contain" },
                        match selected_tab().as_str() {
                            "connection" => rsx! { MobileConnectionPage {} },
                            "share" => rsx! { MobileSharePage {} },
                            _ => rsx! { MobileSettingsPage {} },
                        }
                    }
                }
                nav { class: "mobile-nav grid shrink-0 border-t border-border bg-card", style: "grid-template-columns: repeat({tab_count}, minmax(0, 1fr))", aria_label: tr("Main navigation"),
                    for tab in tabs.iter().copied() {
                        Button {
                            key: "mobile-tab-{tab.id}",
                            css_class: if selected_tab() == tab.id { "mobile-nav-button bg-accent text-primary" } else { "mobile-nav-button text-muted-foreground" },
                            variant: ButtonVariant::Ghost,
                            aria_label: tab.label,
                            onclick: move |_| {
                                selected_tab.set(tab.id.to_owned());
                                if tab.id == "settings" && !state.active_page.peek().starts_with("settings:") {
                                    state.active_page.set("settings:general".to_owned());
                                }
                            },
                            UiIcon { icon: tab.icon }
                            span { {tr(tab.label)} }
                        }
                    }
                }
            }
        }
    }
}

fn mobile_tabs(incoming_only: bool, outgoing_only: bool) -> Vec<MobileTab> {
    #[cfg(not(target_os = "android"))]
    let _ = outgoing_only;
    let mut tabs = Vec::new();
    if !incoming_only {
        tabs.push(MobileTab {
            id: "connection",
            label: "Connection",
            icon: UiIconName::History,
        });
    }
    #[cfg(target_os = "android")]
    if !outgoing_only {
        tabs.push(MobileTab {
            id: "chat",
            label: "Chat",
            icon: UiIconName::Users,
        });
        tabs.push(MobileTab {
            id: "share",
            label: "Share screen",
            icon: UiIconName::Monitor,
        });
    }
    tabs.push(MobileTab {
        id: "settings",
        label: "Settings",
        icon: UiIconName::Settings,
    });
    tabs
}

#[component]
fn MobileConnectionPage() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let mut peer_id = state.peer_id;
    let mut password = state.password;
    let mut two_factor_code = state.two_factor_code;
    let status = state.ui_state.read().clone();
    let mut peers = (state.peer_cards)()
        .into_iter()
        .filter(|peer| (state.peer_tab)() != "favorites" || peer.favorite)
        .collect::<Vec<_>>();
    sort_peer_cards(&mut peers, &(state.peer_tab)(), &(state.peer_sort)());
    let mut peer_tab = state.peer_tab;

    rsx! {
        div { class: "mobile-page flex flex-col gap-4",
            div { class: "flex items-center gap-2",
                Input {
                    css_class: "h-12 min-w-0 flex-1 text-center text-base tabular-nums",
                    name: "remote-id",
                    value: format_remote_id(&peer_id()),
                    placeholder: "Enter remote ID",
                    oninput: move |event: FormEvent| peer_id.set(format_remote_id(&event.value())),
                }
                Button {
                    css_class: "h-12 w-12 shrink-0 p-0",
                    title: "Connect",
                    aria_label: "Connect",
                    disabled: peer_id().trim().is_empty() || state.session.read().is_some(),
                    onclick: move |_| start_session(state.session, peer_id(), password(), Vec::new(), state.ui_state),
                    ArrowRight { class: "block h-5 w-5 [&_svg]:block [&_svg]:size-full" }
                }
            }
            if !status.status.is_empty() || !status.message.is_empty() {
                Card { css_class: "mobile-status w-full gap-2 px-4 py-3",
                    if !status.status.is_empty() { p { class: "text-sm font-medium", {tr(&status.status)} } }
                    if !status.message.is_empty() { p { class: "mt-1 text-xs text-muted-foreground [overflow-wrap:anywhere]", {tr(&status.message)} } }
                    if status.password_required {
                        div { class: "mobile-form flex gap-2",
                            Input { css_class: "min-w-0 flex-1", r#type: InputType::Password, value: password(), placeholder: "Password", oninput: move |event: FormEvent| password.set(event.value()) }
                            Button { onclick: move |_| if let Some(active) = state.session.read().as_ref() {
                                active.login(String::new(), String::new(), password(), (state.remember_password)());
                                update_ui_state(state.ui_state, |ui| { ui.password_required = false; ui.status = "Authenticating".to_owned(); });
                            }, {tr("Continue")} }
                        }
                    } else if status.two_factor_required {
                        div { class: "mobile-form flex gap-2",
                            Input { css_class: "min-w-0 flex-1", value: two_factor_code(), placeholder: "Two-factor code", oninput: move |event: FormEvent| two_factor_code.set(event.value()) }
                            Button { onclick: move |_| if let Some(active) = state.session.read().as_ref() {
                                active.send2fa(two_factor_code(), false);
                                update_ui_state(state.ui_state, |ui| { ui.two_factor_required = false; ui.status = "Verifying".to_owned(); });
                            }, {tr("Verify")} }
                        }
                    } else if status.retry_available {
                        Button { css_class: "mt-3", onclick: move |_| if let Some(active) = state.session.read().as_ref() {
                            active.reconnect(false);
                            update_ui_state(state.ui_state, |ui| { ui.status = "Connecting".to_owned(); ui.retry_available = false; });
                        }, {tr("Retry")} }
                    } else if state.session.read().is_some() {
                        Button { css_class: "mt-3", variant: ButtonVariant::Destructive, onclick: move |_| close_mobile_session(state), {tr("Cancel connection")} }
                    }
                }
            }
            div { class: "flex min-w-0 flex-col gap-3",
                div { class: "mobile-peer-tabs grid grid-cols-2 gap-1 rounded-lg bg-muted p-1",
                    for (id, label) in [("recent", "Recent sessions"), ("favorites", "Favorites")] {
                        Button {
                            css_class: if peer_tab() == id { "bg-card text-foreground shadow-sm" } else { "text-muted-foreground" },
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| peer_tab.set(id.to_owned()),
                            {tr(label)}
                        }
                    }
                }
                if peers.is_empty() {
                    div { class: "flex flex-col items-center gap-2 py-8 text-center text-sm text-muted-foreground",
                        if peer_tab() == "favorites" { Star { class: "size-6" } } else { History { class: "size-6" } }
                        p { {tr(if peer_tab() == "favorites" { "No favorites" } else { "No recent sessions" })} }
                    }
                } else {
                    div { class: "divide-y divide-border overflow-hidden rounded-lg border border-border bg-card",
                        for peer in peers {
                            {
                                let id = peer.id.clone();
                                let name = peer_display_name(&peer);
                                let subtitle = peer_subtitle(&peer);
                                let online = peer.online;
                                rsx! {
                                    button {
                                        key: "mobile-peer-{id}",
                                        r#type: "button",
                                        class: "flex min-h-16 w-full items-center gap-3 px-4 py-2 text-left active:bg-accent",
                                        onclick: move |_| {
                                            peer_id.set(format_remote_id(&id));
                                            start_session(state.session, id.clone(), String::new(), Vec::new(), state.ui_state);
                                        },
                                        span { class: if online { "h-2.5 w-2.5 shrink-0 rounded-full bg-success" } else { "h-2.5 w-2.5 shrink-0 rounded-full bg-muted-foreground" } }
                                        span { class: "min-w-0 flex-1",
                                            span { class: "block truncate text-sm font-medium", "{name}" }
                                            if !subtitle.is_empty() { span { class: "block truncate text-xs text-muted-foreground", "{subtitle}" } }
                                        }
                                        ArrowRight { class: "h-4 w-4 shrink-0 text-muted-foreground" }
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

#[cfg(target_os = "android")]
#[component]
fn MobileChatPage() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let mut updates = use_hook(super::super::connection_manager::subscribe_mobile_chat);
    let mut snapshot = use_signal(|| updates.borrow_and_update().clone());
    use_future(move || {
        let mut updates = updates.clone();
        async move {
            loop {
                snapshot.set(updates.borrow_and_update().clone());
                if updates.changed().await.is_err() {
                    break;
                }
            }
        }
    });
    let mut selected_id = state.mobile_chat_id;
    let data = snapshot.read();
    let clients = data.clients.values().filter(|client| !client.disconnected);
    let selected = selected_id();
    let active = clients
        .clone()
        .find(|client| client.id == selected)
        .or_else(|| clients.clone().next());
    let active_id = active.map(|client| client.id);
    let title = active.map_or_else(|| tr("Chat"), |client| mobile_chat_label(client).to_owned());
    let messages: Vec<_> = data
        .messages
        .iter()
        .filter(|(id, _)| Some(*id) == active_id)
        .collect();

    rsx! {
        header { class: "mobile-header flex shrink-0 items-center justify-center gap-2 border-b border-border bg-card px-4",
            h1 { class: "min-w-0 truncate text-base font-medium", "{title}" }
            if active.is_some() {
                span { class: "h-2.5 w-2.5 shrink-0 rounded-full bg-success", title: tr("Connected") }
            }
        }
        main { class: "mobile-page flex min-h-0 flex-1 flex-col overflow-hidden overscroll-contain",
            if let Some(active) = active {
                div { class: "mb-3 flex shrink-0 gap-2 overflow-x-auto pb-1",
                    for client in clients {
                        {
                            let id = client.id;
                            let label = mobile_chat_label(client);
                            rsx! {
                                Button {
                                    key: "mobile-chat-client-{id}",
                                    css_class: if active_id == Some(id) { "max-w-[14rem] shrink-0 bg-accent" } else { "max-w-[14rem] shrink-0" },
                                    variant: ButtonVariant::Outline,
                                    onclick: move |_| selected_id.set(id),
                                    span { class: "truncate", "{label}" }
                                }
                            }
                        }
                    }
                }
                for active in std::iter::once(active) {
                    div { key: "{active.id}", class: "flex min-h-0 flex-1 flex-col",
                        div { class: "min-h-0 flex-1 space-y-2 overflow-y-auto rounded-lg border border-border bg-card p-3", role: "log", aria_live: "polite",
                            if messages.is_empty() { p { class: "text-sm text-muted-foreground", {tr("No messages")} } }
                            for (index, (_, text)) in messages.iter().enumerate() {
                                p { key: "mobile-message-{index}", class: "whitespace-pre-wrap text-sm [overflow-wrap:anywhere]", "{text}" }
                            }
                        }
                        MobileChatComposer { id: active.id, enabled: active.authorized }
                    }
                }
            } else {
                div { class: "flex flex-1 flex-col items-center justify-center gap-3 text-center text-muted-foreground",
                    Users { class: "h-10 w-10" }
                    p { {tr("No active connections")} }
                }
            }
        }
    }
}

#[cfg(target_os = "android")]
fn mobile_chat_label(client: &crate::ui_cm_interface::Client) -> &str {
    if client.name.is_empty() {
        &client.peer_id
    } else {
        &client.name
    }
}

#[cfg(target_os = "android")]
#[component]
fn MobileChatComposer(id: i32, enabled: bool) -> Element {
    let mut draft = use_signal(String::new);
    rsx! {
        form { class: "mobile-form mt-3 flex shrink-0 gap-2", onsubmit: move |event| {
            event.prevent_default();
            let text = draft().trim().to_owned();
            if enabled && !text.is_empty() && super::super::connection_manager::send_mobile_chat(id, text) {
                draft.set(String::new());
            }
        },
            Input { css_class: "min-w-0 flex-1", value: draft(), placeholder: tr("Write a message"), disabled: !enabled, oninput: move |event: FormEvent| draft.set(event.value()) }
            Button { button_type: "submit", disabled: !enabled || draft().trim().is_empty(), {tr("Send")} }
        }
    }
}

#[cfg(not(target_os = "android"))]
#[component]
fn MobileChatPage() -> Element {
    rsx! {}
}

#[cfg(target_os = "android")]
#[component]
fn MobileSharePage() -> Element {
    let mut state = use_context::<super::super::state::AppState>();
    rsx! {
        div { class: "mobile-page flex flex-col gap-4",
            Card { css_class: "gap-3 p-4",
                h2 { class: "text-base font-semibold", {tr("Your Device")} }
                div { class: "flex items-center gap-2",
                    div { class: "min-w-0 flex-1",
                        p { class: "text-xs text-muted-foreground", {tr("ID")} }
                        p { class: "select-all text-lg font-medium tabular-nums [overflow-wrap:anywhere]", "{(state.local_device_id)()}" }
                    }
                    Button { css_class: "mobile-icon-button", variant: ButtonVariant::Ghost, title: "Copy ID", aria_label: "Copy ID", onclick: move |_| copy_text_to_clipboard((state.local_device_id)()), Copy { class: "size-4" } }
                }
                div { class: "flex items-center gap-2 border-t border-border pt-3",
                    div { class: "min-w-0 flex-1",
                        p { class: "text-xs text-muted-foreground", {tr("One-time Password")} }
                        p { class: "select-all text-lg font-medium tabular-nums [overflow-wrap:anywhere]", "{(state.temporary_password)()}" }
                    }
                    Button { css_class: "mobile-icon-button", variant: ButtonVariant::Ghost, title: "Refresh", aria_label: "Refresh", onclick: move |_| {
                        crate::ui_interface::update_temporary_password();
                        state.temporary_password.set(crate::ui_interface::temporary_password());
                    }, RefreshCw { class: "size-4" } }
                    Button { css_class: "mobile-icon-button", variant: ButtonVariant::Ghost, title: "Copy password", aria_label: "Copy password", onclick: move |_| copy_text_to_clipboard((state.temporary_password)()), Copy { class: "size-4" } }
                }
            }
            Card { css_class: "px-4 py-3", ServiceStatus { status: (state.service_status)() } }
            div { class: "mobile-incoming", super::super::connection_manager::Panel {} }
        }
    }
}

#[cfg(not(target_os = "android"))]
#[component]
fn MobileSharePage() -> Element {
    rsx! {}
}

#[component]
fn MobileSettingsPage() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let mut active_page = state.active_page;
    let mut effect_active_page = state.active_page;
    use_effect(move || {
        if !effect_active_page.peek().starts_with("settings:") {
            effect_active_page.set("settings:general".to_owned());
        }
    });
    let mut tabs = vec![
        ("settings:general", "General"),
        ("settings:security", "Security"),
        ("settings:network", "Network"),
        ("settings:codec", "Codec"),
        ("settings:display", "Display"),
        ("settings:about", "About"),
    ];
    if !hbb_common::config::is_disable_account() {
        tabs.push(("settings:account", "Account"));
    }
    rsx! {
        div { class: "flex min-h-0 flex-1 flex-col",
            nav { class: "mobile-settings-tabs border-b border-border bg-card", aria_label: tr("Settings sections"),
                div { class: "mobile-page flex gap-1",
                    for (page, label) in tabs.iter().copied() {
                        Button {
                            key: "mobile-setting-tab-{page}",
                            css_class: if active_page() == page { "shrink-0 bg-accent text-primary" } else { "shrink-0 text-muted-foreground" },
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| active_page.set(page.to_owned()),
                            {tr(label)}
                        }
                    }
                }
            }
            section { class: "min-h-0 flex-1 overflow-y-auto overscroll-contain",
                div { class: "mobile-page mobile-settings-content",
                    SettingsContent {
                        key: "{active_page()}",
                        page: active_page(),
                        codec_state: state.codec_state,
                        has_hardware_codec: state.has_hardware_codec,
                        hardware_codec: state.hardware_codec,
                    }
                }
            }
        }
    }
}

#[component]
fn MobileRemotePage() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let status = state.ui_state.read().clone();
    let session = state.session;
    let mut toolbar_visible = use_signal(|| true);
    let mut keyboard_open = use_signal(|| false);
    let mut keyboard_text = use_signal(String::new);
    let mut chat_open = use_signal(|| false);
    let mut chat_text = use_signal(String::new);
    let mut gesture_help = use_signal(|| false);
    let mut actions_open = use_signal(|| false);
    let session_info = session.read();
    let peer_is_android = session_info.as_ref().is_some_and(|active| {
        active
            .get_platform(true)
            .to_ascii_lowercase()
            .contains("android")
    });
    let local_touch_mode = crate::ui_interface::get_local_option(OPTION_TOUCH_MODE.to_owned());
    let initial_touch_mode = if peer_is_android {
        true
    } else if local_touch_mode.is_empty() {
        session_info
            .as_ref()
            .is_some_and(|active| !active.get_option(OPTION_TOUCH_MODE.to_owned()).is_empty())
    } else {
        local_touch_mode == "Y"
    };
    drop(session_info);
    let mut touch_mode = use_signal(|| initial_touch_mode);
    let can_control = session
        .read()
        .as_ref()
        .is_some_and(mobile_session_can_control);
    let display_count = status.displays.len();
    let current_display = status.current_display;

    rsx! {
        div { class: "relative flex min-h-0 flex-1 flex-col overflow-hidden bg-[#090a0b]",
            main { class: "relative flex min-h-0 flex-1 items-center justify-center overflow-hidden",
                class: if !toolbar_visible() { "mobile-toolbar-collapsed" } else { "" },
                MobileRemoteSurface { touch_mode: touch_mode(), peer_is_android }
                if gesture_help() {
                    aside { class: "mobile-overlay z-20 overflow-y-auto overscroll-contain rounded-xl border border-border bg-card p-4 text-sm text-card-foreground shadow-lg",
                        h2 { class: "mb-2 font-semibold", {tr("Touch gestures")} }
                        ul { class: "space-y-1 text-muted-foreground",
                            li { {tr("Tap to click")} }
                            li { {tr("Drag to move or select")} }
                            li { {tr("Long press for right-click")} }
                            li { {tr("Use two fingers to scroll")} }
                            li { {tr("Use the keyboard button to type on the remote device")} }
                        }
                    }
                }
                if actions_open() {
                    div { class: "mobile-overlay mobile-actions z-30 grid grid-cols-2 gap-2 overflow-y-auto overscroll-contain rounded-xl border border-border bg-card p-3 text-card-foreground shadow-lg",
                        Button { variant: ButtonVariant::Outline, disabled: !can_control, onclick: move |_| {
                            if let Some(active) = session.read().as_ref() { active.ctrl_alt_del(); }
                            actions_open.set(false);
                        }, {tr("Insert Ctrl + Alt + Del")} }
                        Button { variant: ButtonVariant::Outline, disabled: !can_control, onclick: move |_| {
                            if let Some(active) = session.read().as_ref() { active.lock_screen(); }
                            actions_open.set(false);
                        }, {tr("Insert Lock")} }
                        Button { variant: ButtonVariant::Outline, disabled: display_count == 0, onclick: move |_| {
                            if let Some(active) = session.read().as_ref() { active.refresh_video(current_display); }
                            actions_open.set(false);
                        }, {tr("Refresh")} }
                        Button { variant: ButtonVariant::Ghost, onclick: move |_| actions_open.set(false), {tr("Close")} }
                    }
                }
                if chat_open() {
                    aside { class: "mobile-overlay mobile-chat-overlay z-40 flex min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-card text-card-foreground shadow-xl",
                        div { class: "flex shrink-0 items-center justify-between gap-2 border-b border-border px-3",
                            h2 { class: "font-semibold", {tr("Text chat")} }
                            Button { css_class: "mobile-icon-button", variant: ButtonVariant::Ghost, title: "Close", aria_label: "Close", onclick: move |_| chat_open.set(false), X { class: "size-4" } }
                        }
                        div { class: "min-h-0 flex-1 space-y-2 overflow-y-auto p-3", role: "log", aria_live: "polite",
                            if status.chat.is_empty() {
                                p { class: "text-sm text-muted-foreground", {tr("No messages")} }
                            }
                            for (index, message) in status.chat.iter().enumerate() {
                                p {
                                    key: "mobile-remote-chat-{index}",
                                    class: if message.outgoing { "ml-8 rounded-lg bg-primary/10 px-3 py-2 text-right text-sm whitespace-pre-wrap [overflow-wrap:anywhere]" } else { "mr-8 rounded-lg bg-muted px-3 py-2 text-sm whitespace-pre-wrap [overflow-wrap:anywhere]" },
                                    "{message.text}"
                                }
                            }
                        }
                        form { class: "mobile-form flex shrink-0 gap-2 border-t border-border p-2", onsubmit: move |event| {
                            event.prevent_default();
                            let text = chat_text().trim().to_owned();
                            if text.is_empty() { return; }
                            let current = session.peek();
                            let Some(active) = current.as_ref() else { return; };
                            active.send_chat(text.clone());
                            update_ui_state(state.ui_state, |ui| {
                                let chat = Arc::make_mut(&mut ui.chat);
                                if chat.len() >= 100 {
                                    chat.remove(0);
                                }
                                chat.push(ChatMessage { outgoing: true, text });
                            });
                            chat_text.set(String::new());
                        },
                            Input { css_class: "min-w-0 flex-1", name: "mobile-remote-chat-text", value: chat_text(), placeholder: tr("Write a message"), oninput: move |event: FormEvent| chat_text.set(event.value()) }
                            Button { button_type: "submit", disabled: chat_text().trim().is_empty(), {tr("Send")} }
                        }
                    }
                }
                if !toolbar_visible() {
                    Button {
                        css_class: "mobile-show-toolbar mobile-icon-button rounded-full shadow-lg",
                        title: "Show Toolbar",
                        aria_label: "Show Toolbar",
                        onclick: move |_| toolbar_visible.set(true),
                        ChevronUp { class: "size-5" }
                    }
                }
            }
            if keyboard_open() && can_control {
                form { class: "mobile-form flex shrink-0 gap-2 border-t border-border bg-card p-2", onsubmit: move |event| {
                    event.prevent_default();
                    let text = keyboard_text();
                    if !text.is_empty() {
                        if let Some(active) = session.read().as_ref() { active.input_string(&text); }
                        keyboard_text.set(String::new());
                    }
                },
                    Input { css_class: "min-w-0 flex-1", name: "mobile-keyboard-text", value: keyboard_text(), placeholder: "Type text for the remote device", oninput: move |event: FormEvent| keyboard_text.set(event.value()) }
                    Button { button_type: "submit", disabled: keyboard_text().is_empty(), {tr("Send")} }
                    Button { css_class: "mobile-icon-button", variant: ButtonVariant::Ghost, title: "Close", aria_label: "Close", onclick: move |_| keyboard_open.set(false), X { class: "size-4" } }
                }
            }
            if toolbar_visible() {
                nav { class: "mobile-toolbar flex shrink-0 items-center gap-1 border-t border-border bg-card", aria_label: tr("Remote controls"),
                    Button { css_class: "mobile-icon-button text-destructive", variant: ButtonVariant::Ghost, title: "Disconnect", aria_label: "Disconnect", onclick: move |_| close_mobile_session(state), X { class: "size-5" } }
                    div { class: "mobile-toolbar-scroll flex min-w-0 flex-1 items-center gap-1 overflow-x-auto",
                        Button { css_class: if chat_open() { "mobile-icon-button bg-accent" } else { "mobile-icon-button" }, variant: ButtonVariant::Ghost, title: "Text chat", aria_label: "Text chat", onclick: move |_| { actions_open.set(false); gesture_help.set(false); chat_open.set(!chat_open()); }, MessageSquare { class: "size-5" } }
                        if can_control {
                            Button { css_class: if keyboard_open() { "mobile-icon-button bg-accent" } else { "mobile-icon-button" }, variant: ButtonVariant::Ghost, title: "Keyboard", aria_label: "Keyboard", onclick: move |_| keyboard_open.set(!keyboard_open()), Keyboard { class: "size-5" } }
                        }
                        if peer_is_android && can_control {
                            Button {
                                css_class: "mobile-icon-button",
                                variant: ButtonVariant::Ghost,
                                title: "Back",
                                aria_label: "Back",
                                onclick: move |_| if let Some(active) = session.read().as_ref() { send_mobile_back(active); },
                                ArrowLeft { class: "size-5" }
                            }
                            Button {
                                css_class: "mobile-icon-button",
                                variant: ButtonVariant::Ghost,
                                title: "Home",
                                aria_label: "Home",
                                onclick: move |_| if let Some(active) = session.read().as_ref() { send_mobile_home(active); },
                                House { class: "size-5" }
                            }
                            Button {
                                css_class: "mobile-icon-button",
                                variant: ButtonVariant::Ghost,
                                title: "Apps",
                                aria_label: "Apps",
                                onclick: move |_| if let Some(active) = session.read().as_ref() { send_mobile_apps(active.clone()); },
                                Ellipsis { class: "size-5" }
                            }
                        }
                        if !peer_is_android && can_control {
                            Button {
                                css_class: if touch_mode() { "mobile-icon-button bg-accent" } else { "mobile-icon-button" },
                                variant: ButtonVariant::Ghost,
                                title: if touch_mode() { "Touch mode" } else { "Mouse mode" },
                                onclick: move |_| {
                                    let next = !touch_mode();
                                    touch_mode.set(next);
                                    crate::ui_interface::set_local_option(OPTION_TOUCH_MODE.to_owned(), if next { "Y" } else { "N" }.to_owned());
                                },
                                if touch_mode() { Hand { class: "size-5" } } else { MousePointer2 { class: "size-5" } }
                            }
                        }
                        Button { css_class: if gesture_help() { "mobile-icon-button bg-accent" } else { "mobile-icon-button" }, variant: ButtonVariant::Ghost, title: "Touch gestures", aria_label: "Touch gestures", onclick: move |_| { actions_open.set(false); gesture_help.set(!gesture_help()); }, CircleHelp { class: "size-5" } }
                        if display_count > 0 {
                            Button {
                                css_class: "mobile-display-button",
                                variant: ButtonVariant::Ghost,
                                title: "Switch display",
                                disabled: display_count < 2,
                                onclick: move |_| if let Some(active) = session.read().as_ref() {
                                    active.switch_display((current_display + 1).rem_euclid(display_count as i32));
                                },
                                Monitor { class: "size-5" }
                                span { class: "text-xs", "{current_display + 1}/{display_count}" }
                            }
                        }
                    }
                    Button { css_class: if actions_open() { "mobile-icon-button bg-accent" } else { "mobile-icon-button" }, variant: ButtonVariant::Ghost, title: "More actions", aria_label: "More actions", onclick: move |_| { gesture_help.set(false); actions_open.set(!actions_open()); }, Ellipsis { class: "size-5" } }
                    Button { css_class: "mobile-icon-button", variant: ButtonVariant::Ghost, title: "Hide Toolbar", aria_label: "Hide Toolbar", onclick: move |_| toolbar_visible.set(false), ChevronDown { class: "size-5" } }
                }
            }
        }
    }
}

fn mobile_session_can_control(session: &Session<DioxusHandler>) -> bool {
    !session.is_view_camera()
        && !session.get_toggle_option("view-only".to_owned())
        && session
            .server_keyboard_enabled
            .read()
            .is_ok_and(|enabled| *enabled)
}

fn send_mobile_back(session: &Session<DioxusHandler>) {
    let button = if session.get_peer_version() >= hbb_common::get_version_number("1.3.8") {
        crate::common::input::MOUSE_BUTTON_BACK
    } else {
        crate::common::input::MOUSE_BUTTON_RIGHT
    };
    if button == crate::common::input::MOUSE_BUTTON_RIGHT {
        session.send_mouse(
            button << 3 | crate::common::input::MOUSE_TYPE_DOWN,
            0,
            0,
            false,
            false,
            false,
            false,
        );
    }
    session.send_mouse(
        button << 3 | crate::common::input::MOUSE_TYPE_UP,
        0,
        0,
        false,
        false,
        false,
        false,
    );
}

fn send_mobile_home(session: &Session<DioxusHandler>) {
    let button = crate::common::input::MOUSE_BUTTON_WHEEL << 3;
    session.send_mouse(
        button | crate::common::input::MOUSE_TYPE_DOWN,
        0,
        0,
        false,
        false,
        false,
        false,
    );
    session.send_mouse(
        button | crate::common::input::MOUSE_TYPE_UP,
        0,
        0,
        false,
        false,
        false,
        false,
    );
}

fn send_mobile_apps(session: Session<DioxusHandler>) {
    let button = crate::common::input::MOUSE_BUTTON_WHEEL << 3;
    session.send_mouse(
        button | crate::common::input::MOUSE_TYPE_DOWN,
        0,
        0,
        false,
        false,
        false,
        false,
    );
    spawn(async move {
        hbb_common::tokio::time::sleep(Duration::from_millis(500)).await;
        session.send_mouse(
            button | crate::common::input::MOUSE_TYPE_UP,
            0,
            0,
            false,
            false,
            false,
            false,
        );
    });
}

fn touch_client_point(data: &TouchData) -> Option<(f64, f64)> {
    let touch = data
        .touches()
        .into_iter()
        .next()
        .or_else(|| data.touches_changed().into_iter().next())?;
    let point = touch.client_coordinates();
    Some((point.x, point.y))
}

fn touch_centroid(data: &TouchData) -> Option<(f64, f64)> {
    let touches = data.touches();
    if touches.len() < 2 {
        return None;
    }
    let (x, y) = touches.iter().fold((0.0, 0.0), |(x, y), touch| {
        let point = touch.client_coordinates();
        (x + point.x, y + point.y)
    });
    let count = touches.len() as f64;
    Some((x / count, y / count))
}

#[component]
fn MobileRemoteSurface(touch_mode: bool, peer_is_android: bool) -> Element {
    let state = use_context::<super::super::state::AppState>();
    let session = state.session;
    let ui_state = state.ui_state;
    let status = ui_state.read().clone();
    let mouse_down_state = status.clone();
    let mouse_up_state = status.clone();
    let mouse_move_state = status.clone();
    let context_mouse_down_state = status.clone();
    let context_mouse_up_state = status.clone();
    let touch_start_state = status.clone();
    let touch_move_state = status.clone();
    let touch_end_state = status.clone();
    let touch_cancel_state = status.clone();
    let gesture = use_hook(|| Rc::new(Cell::new(MobileTouchGesture::Idle)));
    let scroll_center = use_hook(|| Rc::new(Cell::new(None::<(f64, f64)>)));
    let touch_start = use_hook(|| Rc::new(Cell::new(None::<(f64, f64)>)));
    let gesture_started = use_hook(|| Rc::new(Cell::new(None::<Instant>)));
    let touch_start_gesture = gesture.clone();
    let touch_move_gesture = gesture.clone();
    let touch_end_gesture = gesture.clone();
    let touch_cancel_gesture = gesture.clone();
    let touch_start_scroll = scroll_center.clone();
    let touch_move_scroll = scroll_center.clone();
    let touch_end_scroll = scroll_center.clone();
    let touch_cancel_scroll = scroll_center.clone();
    let touch_start_origin = touch_start.clone();
    let touch_move_origin = touch_start.clone();
    let touch_end_origin = touch_start.clone();
    let touch_cancel_origin = touch_start.clone();
    let touch_start_time = gesture_started.clone();
    let touch_end_time = gesture_started.clone();
    let touch_cancel_time = gesture_started.clone();

    if status.frame_data_uri.is_empty() {
        return rsx! {
            div { class: "flex h-full flex-col items-center justify-center gap-2 text-sm text-muted-foreground",
                span { class: "h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" }
                span { {tr("Connected, waiting for image...")} }
            }
        };
    }

    rsx! {
        img {
            class: "m-auto block h-auto max-h-full w-auto max-w-full bg-[#090a0b] object-contain [touch-action:none] [user-select:none]",
            src: "{status.frame_data_uri}",
            width: "{status.width}",
            height: "{status.height}",
            alt: "Remote desktop",
            tabindex: "0",
            onmounted: move |event| async move {
                if let Ok(rect) = event.get_client_rect().await {
                    update_ui_state(ui_state, |state| {
                        state.viewport_left = rect.origin.x;
                        state.viewport_top = rect.origin.y;
                        state.viewport_width = rect.width();
                        state.viewport_height = rect.height();
                    });
                }
            },
            onresize: move |event| {
                if let Ok(size) = event.get_border_box_size() {
                    update_ui_state(ui_state, |state| {
                        state.viewport_width = size.width;
                        state.viewport_height = size.height;
                    });
                }
            },
            onmousedown: move |event| if let Some(active) = session.read().as_ref() { if mobile_session_can_control(active) { send_mouse_event(&event.data, 1, active, &mouse_down_state); } },
            onmouseup: move |event| if let Some(active) = session.read().as_ref() { send_mouse_event(&event.data, 2, active, &mouse_up_state); },
            onmousemove: move |event| if let Some(active) = session.read().as_ref() { if mobile_session_can_control(active) { send_mouse_event(&event.data, 0, active, &mouse_move_state); } },
            onwheel: move |event| {
                event.prevent_default();
                if let Some(active) = session.read().as_ref() { if mobile_session_can_control(active) { send_wheel_event(&event.data, active); } }
            },
            onkeydown: move |event| {
                event.prevent_default();
                if let Some(active) = session.read().as_ref() { if mobile_session_can_control(active) { send_keyboard_event(&event.data, true, active); } }
            },
            onkeyup: move |event| {
                event.prevent_default();
                if let Some(active) = session.read().as_ref() { send_keyboard_event(&event.data, false, active); }
            },
            oncontextmenu: move |event| {
                event.prevent_default();
                if let Some(active) = session.read().as_ref() {
                    if mobile_session_can_control(active) {
                        send_mouse_event(&event.data, 1, active, &context_mouse_down_state);
                        send_mouse_event(&event.data, 2, active, &context_mouse_up_state);
                    }
                }
            },
            ontouchstart: move |event| {
                event.prevent_default();
                let data = &event.data;
                if data.touches().len() > 1 {
                    let previous = touch_start_gesture.get();
                    if matches!(previous, MobileTouchGesture::MouseMove | MobileTouchGesture::AndroidHold) {
                        if let Some(active) = session.read().as_ref() {
                            send_touch_mouse_event(data, 1, 2, active, &touch_start_state);
                        }
                    }
                    if data.touches().len() == 2
                        && matches!(previous, MobileTouchGesture::Idle | MobileTouchGesture::MouseTap)
                    {
                        touch_start_gesture.set(MobileTouchGesture::TwoFingerTap);
                        touch_start_time.set(Some(Instant::now()));
                    } else {
                        touch_start_gesture.set(MobileTouchGesture::Scroll);
                    }
                    touch_start_scroll.set(touch_centroid(data));
                    touch_start_origin.set(touch_centroid(data));
                    return;
                }
                if touch_start_gesture.get() != MobileTouchGesture::Idle { return; }
                if let Some(active) = session.read().as_ref() {
                    if !mobile_session_can_control(active) { return; }
                    if peer_is_android {
                        touch_start_gesture.set(MobileTouchGesture::AndroidHold);
                        send_touch_mouse_event(data, 1, 1, active, &touch_start_state);
                    } else {
                        touch_start_gesture.set(MobileTouchGesture::MouseTap);
                        touch_start_origin.set(touch_client_point(data));
                        touch_start_time.set(Some(Instant::now()));
                    }
                }
            },
            ontouchmove: move |event| {
                event.prevent_default();
                let data = &event.data;
                if data.touches().len() > 1 {
                    let current = touch_centroid(data);
                    if touch_move_gesture.get() == MobileTouchGesture::TwoFingerTap {
                        let moved = touch_move_origin.get().zip(current).is_some_and(|(start, end)| {
                            (end.0 - start.0).abs().max((end.1 - start.1).abs()) > 12.0
                        });
                        if moved {
                            touch_move_gesture.set(MobileTouchGesture::Scroll);
                            touch_move_scroll.set(current);
                        }
                        return;
                    }
                    if touch_move_gesture.get() != MobileTouchGesture::Scroll {
                        if matches!(touch_move_gesture.get(), MobileTouchGesture::MouseMove | MobileTouchGesture::AndroidHold) {
                            if let Some(active) = session.read().as_ref() {
                                    send_touch_mouse_event(data, 1, 2, active, &touch_move_state);
                            }
                        }
                        touch_move_gesture.set(if data.touches().len() == 2 {
                            MobileTouchGesture::TwoFingerTap
                        } else {
                            MobileTouchGesture::Scroll
                        });
                        touch_move_origin.set(current);
                        touch_move_scroll.set(current);
                        return;
                    }
                    let Some(current) = touch_centroid(data) else { return; };
                    if let Some(previous) = touch_move_scroll.replace(Some(current)) {
                        let delta_x = (current.0 - previous.0).round() as i32;
                        let delta_y = (current.1 - previous.1).round() as i32;
                        if delta_x != 0 || delta_y != 0 {
                            if let Some(active) = session.read().as_ref() {
                                if mobile_session_can_control(active) { active.send_mouse(3, delta_x, delta_y, false, false, false, false); }
                            }
                        }
                    }
                    return;
                }
                match touch_move_gesture.get() {
                    MobileTouchGesture::AndroidHold => {
                        if let Some(active) = session.read().as_ref() {
                            if mobile_session_can_control(active) {
                                send_touch_mouse_event(data, 1, 0, active, &touch_move_state);
                            }
                        }
                    }
                    MobileTouchGesture::MouseTap | MobileTouchGesture::MouseMove => {
                        let current = touch_client_point(data);
                        let moved = touch_move_origin.get().zip(current).is_some_and(|(start, end)| {
                            (end.0 - start.0).abs().max((end.1 - start.1).abs()) > 12.0
                        });
                        let start_drag = moved && touch_move_gesture.get() == MobileTouchGesture::MouseTap;
                        if start_drag { touch_move_gesture.set(MobileTouchGesture::MouseMove); }
                        if touch_move_gesture.get() == MobileTouchGesture::MouseMove {
                            if let Some(active) = session.read().as_ref() {
                                if mobile_session_can_control(active) {
                                    if start_drag {
                                        send_touch_mouse_event(data, 1, 1, active, &touch_move_state);
                                    }
                                    send_touch_mouse_event(data, 1, 0, active, &touch_move_state);
                                }
                            }
                        }
                    }
                    MobileTouchGesture::TwoFingerTap => {
                        let current = touch_centroid(data);
                        let moved = touch_move_origin.get().zip(current).is_some_and(|(start, end)| {
                            (end.0 - start.0).abs().max((end.1 - start.1).abs()) > 12.0
                        });
                        if moved {
                            touch_move_gesture.set(MobileTouchGesture::Scroll);
                            touch_move_scroll.set(current);
                        }
                    }
                    MobileTouchGesture::Idle | MobileTouchGesture::Scroll => {}
                }
            },
            ontouchend: move |event| {
                event.prevent_default();
                let data = &event.data;
                if data.touches().is_empty() {
                    match touch_end_gesture.get() {
                        MobileTouchGesture::MouseTap => if let Some(active) = session.read().as_ref() {
                            if mobile_session_can_control(active) {
                                // A tap may start outside the image's rendered origin. Send a
                                // coordinate-only move before the click so the remote cursor is
                                // positioned at the touch location in both modes.
                                send_touch_mouse_event(data, 0, 0, active, &touch_end_state);
                                let held = touch_end_time.get().is_some_and(|started| started.elapsed() >= Duration::from_millis(500));
                                let button = if held { 2 } else { 1 };
                                send_touch_mouse_event(data, button, 1, active, &touch_end_state);
                                send_touch_mouse_event(data, button, 2, active, &touch_end_state);
                            }
                        },
                        MobileTouchGesture::MouseMove => if let Some(active) = session.read().as_ref() {
                            send_touch_mouse_event(data, 1, 2, active, &touch_end_state);
                        },
                        MobileTouchGesture::AndroidHold => if let Some(active) = session.read().as_ref() {
                            send_touch_mouse_event(data, 1, 2, active, &touch_end_state);
                        },
                        MobileTouchGesture::TwoFingerTap if !touch_mode && !peer_is_android => if let Some(active) = session.read().as_ref() {
                            if mobile_session_can_control(active) {
                                send_touch_mouse_event(data, 0, 0, active, &touch_end_state);
                                send_touch_mouse_event(data, 2, 1, active, &touch_end_state);
                                send_touch_mouse_event(data, 2, 2, active, &touch_end_state);
                            }
                        },
                        _ => {}
                    }
                    touch_end_gesture.set(MobileTouchGesture::Idle);
                    touch_end_scroll.set(None);
                    touch_end_origin.set(None);
                    touch_end_time.set(None);
                } else if touch_end_gesture.get() == MobileTouchGesture::Scroll {
                    // A remaining finger cannot continue a two-finger scroll. Reset the
                    // centroid so a later touch starts a fresh gesture.
                    touch_end_scroll.set(None);
                    touch_end_origin.set(None);
                }
            },
            ontouchcancel: move |event| {
                event.prevent_default();
                if let Some(active) = session.read().as_ref() {
                    match touch_cancel_gesture.get() {
                        MobileTouchGesture::MouseMove => {
                            send_touch_mouse_event(&event.data, 1, 2, active, &touch_cancel_state);
                        }
                        MobileTouchGesture::AndroidHold => {
                            send_touch_mouse_event(&event.data, 1, 2, active, &touch_cancel_state);
                        }
                        _ => {}
                    }
                }
                touch_cancel_gesture.set(MobileTouchGesture::Idle);
                touch_cancel_scroll.set(None);
                touch_cancel_origin.set(None);
                touch_cancel_time.set(None);
            },
        }
    }
}

fn close_mobile_session(mut state: super::super::state::AppState) {
    if let Some(active) = state.session.peek().as_ref() {
        active.close();
    }
    set_current_session(None);
    state.session.set(None);
    update_ui_state(state.ui_state, |ui| {
        ui.connected = false;
        ui.status = "Disconnected".to_owned();
        ui.message.clear();
        ui.chat = Arc::new(Vec::new());
        ui.frame_data_uri = Arc::from("");
    });
}
