use super::super::super::prelude::*;
use super::{
    docking::{Edge, ToolbarDock},
    menus, panels,
};
use icons::{
    ChevronDown, ChevronUp, Grip, Keyboard, Maximize, MessageSquare, Minimize, MousePointer2,
    Phone, Pin, PinOff, Video,
};

pub(super) const ICON_BUTTON: &str = "flex h-8 w-8 shrink-0 items-center justify-center rounded-lg p-0 text-white focus-visible:outline-2 focus-visible:outline-white disabled:opacity-40 [&_svg]:block [&_svg]:size-5";

#[derive(Clone, Copy)]
pub(super) struct ToolbarState {
    pub menu: Signal<Option<&'static str>>,
    pub submenu: Signal<Option<&'static str>>,
    pub revision: Signal<u32>,
    pub quality_dialog: Signal<bool>,
}

impl ToolbarState {
    pub fn dismiss(mut self) {
        self.menu.set(None);
        self.submenu.set(None);
    }
    pub fn changed(mut self) {
        self.revision += 1;
    }
}

#[component]
pub(super) fn ToolbarButton(
    title: String,
    #[props(default)] inactive: bool,
    #[props(default)] danger: bool,
    #[props(default)] disabled: bool,
    onclick: EventHandler<dioxus::prelude::MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        button {
            class: "{ICON_BUTTON}",
            class: if danger { "bg-[#ff5252] hover:bg-red-600" } else if inactive { "bg-[#424242] hover:bg-[#616161]" } else { "bg-[#0071ff] hover:bg-[#4f95ff]" },
            r#type: "button", title: "{title}", aria_label: "{title}", disabled,
            onclick,
            {children}
        }
    }
}

#[component]
fn MenuTrigger(name: &'static str, title: String, children: Element) -> Element {
    let mut toolbar = use_context::<ToolbarState>();
    rsx! {
        button {
            class: "{ICON_BUTTON} bg-[#0071ff] hover:bg-[#4f95ff]", r#type: "button", title: "{title}", aria_label: "{title}",
            aria_haspopup: "menu", aria_expanded: (toolbar.menu)() == Some(name),
            onclick: move |_| {
                let next = if (toolbar.menu)() == Some(name) { None } else { Some(name) };
                toolbar.submenu.set(None);
                toolbar.menu.set(next);
            },
            {children}
        }
    }
}

#[component]
pub(super) fn ToolbarHandle(
    mut collapsed: Signal<bool>,
    mut fullscreen: Signal<bool>,
    dock: SyncSignal<ToolbarDock>,
) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let toolbar = use_context::<ToolbarState>();
    let status = state.ui_state.read().clone();
    rsx! {
        div { class: "flex shrink-0 items-center rounded border border-[#48494e] bg-rd-surface text-[#bfc0c2] shadow-lg [&_button]:flex [&_button]:h-5 [&_button]:w-6 [&_button]:items-center [&_button]:justify-center [&_button]:p-0 [&_button]:hover:bg-[#394357] [&_svg]:block [&_svg]:size-4",
            class: if dock().edge.horizontal() { "h-5 px-1" } else { "w-6 flex-col py-1" },
            button { class: "cursor-grab active:cursor-grabbing", r#type: "button", title: "Drag toolbar", aria_label: "Drag toolbar",
                onmousedown: move |event| if event.trigger_button() == Some(MouseButton::Primary) {
                    event.prevent_default();
                    toolbar.dismiss();
                    crate::dioxus_ui::native::window().drag_toolbar(dock);
                }, Grip {}
            }
            if collapsed() && status.displays.len() > 1 {
                button { r#type: "button", title: "Switch display", aria_label: "Switch display",
                    onclick: move |_| if let Some(active) = state.session.peek().as_ref() {
                        active.switch_display((status.current_display + 1) % status.displays.len() as i32);
                    },
                    Monitor {}
                }
            }
            button { r#type: "button", title: if fullscreen() { "Exit Fullscreen" } else { "Fullscreen" }, aria_label: if fullscreen() { "Exit Fullscreen" } else { "Fullscreen" },
                onclick: move |_| {
                    let next = !fullscreen();
                    crate::dioxus_ui::native::window().set_fullscreen(next);
                    fullscreen.set(next);
                    toolbar.dismiss();
                },
                if fullscreen() { Minimize {} } else { Maximize {} }
            }
            if fullscreen() {
                button { r#type: "button", title: "Minimize", aria_label: "Minimize",
                    onclick: move |_| crate::dioxus_ui::native::window().set_minimized(true), Minus {}
                }
            }
            button { r#type: "button", title: if collapsed() { "Show Toolbar" } else { "Hide Toolbar" }, aria_label: if collapsed() { "Show Toolbar" } else { "Hide Toolbar" },
                onclick: move |_| {
                    collapsed.set(!collapsed());
                    toolbar.dismiss();
                    if let Some(active) = state.session.peek().as_ref() { active.toggle_option("collapse_toolbar".to_owned()); }
                },
                span { class: match dock().edge { Edge::Top => "", Edge::Bottom => "rotate-180", Edge::Left => "-rotate-90", Edge::Right => "rotate-90" },
                    if collapsed() { ChevronDown {} } else { ChevronUp {} }
                }
            }
        }
    }
}

#[component]
pub(super) fn RemoteToolbar(
    fullscreen: Signal<bool>,
    view_style: Signal<String>,
    custom_scale: Signal<u16>,
) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let session = state.session;
    let dock = use_signal_sync(|| ToolbarDock::load(session.peek().as_ref()));
    let menu = use_signal(|| None);
    let submenu = use_signal(|| None);
    let revision = use_signal(|| 0);
    let quality_dialog = use_signal(|| false);
    let toolbar = use_context_provider(|| ToolbarState {
        menu,
        submenu,
        revision,
        quality_dialog,
    });
    let mut collapsed = use_signal(|| {
        session
            .peek()
            .as_ref()
            .is_some_and(|active| active.get_toggle_option("collapse_toolbar".to_owned()))
    });
    let mut pinned = use_signal(|| {
        serde_json::from_str::<serde_json::Value>(&local_ui_option("remoteMenubarState"))
            .ok()
            .and_then(|value| value["pin"].as_bool())
            .unwrap_or(false)
    });
    let mut hovering = use_signal(|| false);
    let mut chat_open = use_signal(|| false);
    let status = state.ui_state.read().clone();
    let permissions = &status.remote.permissions;
    let recording_allowed = permissions.get("recording") != Some(&false);
    let audio_allowed = permissions.get("audio") != Some(&false);
    let edge = dock().edge;
    let anchor = edge.anchor_class();
    let menu_anchor = edge.menu_class();
    let submenu_offset = if edge == Edge::Right || (edge.horizontal() && dock().fraction > 0.5) {
        "min-[1100px]:-translate-x-[260px]"
    } else {
        "min-[1100px]:translate-x-[260px]"
    };
    let _auto_hide = use_resource(move || async move {
        if !pinned() && !hovering() && !collapsed() && menu().is_none() && !dock().dragging {
            hbb_common::tokio::time::sleep(Duration::from_secs(5)).await;
            collapsed.set(true);
        }
    });
    rsx! {
        if menu().is_some() {
            div { class: "absolute inset-0 z-30", onclick: move |_| toolbar.dismiss() }
        }
        div { id: "remote-toolbar-area", class: "pointer-events-none absolute inset-0 z-40",
        div {
            id: "remote-toolbar",
            class: "pointer-events-auto absolute flex w-fit max-h-full max-w-full items-center select-none {anchor}",
            style: "--dock-position: {dock().fraction * 100.0}%; --dock-shift: {-dock().fraction * 100.0}%",
            onmouseenter: move |_| hovering.set(true),
            onmouseleave: move |_| hovering.set(false),
            onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); toolbar.dismiss(); },
            if !collapsed() {
                div { class: "flex max-h-full max-w-full flex-wrap items-center justify-center gap-1 overflow-auto rounded border border-[#48494e] bg-rd-surface px-1.5 py-1.5 shadow-lg",
                    class: if dock().edge.horizontal() { "flex-row" } else { "flex-col" },
                    ToolbarButton { title: if pinned() { "Unpin Toolbar" } else { "Pin Toolbar" }, inactive: !pinned(),
                        onclick: move |_| {
                            pinned.set(!pinned());
                            crate::ui_interface::set_local_ui_option("remoteMenubarState".to_owned(), serde_json::json!({"pin": pinned()}).to_string());
                        },
                        if pinned() { Pin {} } else { PinOff {} }
                    }
                    if status.displays.len() > 1 {
                        MenuTrigger { name: "monitor", title: "Select Monitor", Monitor {} }
                    }
                    if !session.peek().as_ref().is_some_and(|active| active.is_view_camera()) {
                        MenuTrigger { name: "control", title: "Control Actions", MousePointer2 {} }
                    }
                    MenuTrigger { name: "display", title: "Display Settings", Monitor {} }
                    if !session.peek().as_ref().is_some_and(|active| active.is_view_camera()) {
                        MenuTrigger { name: "keyboard", title: "Keyboard Settings", Keyboard {} }
                    }
                    ToolbarButton { title: "Chat", onclick: move |_| { toolbar.dismiss(); chat_open.set(!chat_open()); }, MessageSquare {} }
                    ToolbarButton { title: if status.remote.voice_call.is_empty() { "Voice call" } else { "End voice call" }, disabled: !audio_allowed,
                        danger: !status.remote.voice_call.is_empty(),
                        onclick: move |_| if let Some(active) = session.peek().as_ref() {
                            if state.ui_state.peek().remote.voice_call.is_empty() { active.request_voice_call(); } else { active.close_voice_call(); }
                        }, Phone {}
                    }
                    if local_ui_option("hide-recording-button") != "Y" && (recording_allowed || status.remote.recording) {
                        ToolbarButton { title: if status.remote.recording { "Stop session recording" } else { "Start session recording" }, danger: status.remote.recording,
                            onclick: move |_| if let Some(active) = session.peek().as_ref() { active.record_screen(!active.is_recording()); }, Video {}
                        }
                    }
                    ToolbarButton { title: "Disconnect", danger: true, onclick: move |_| super::close_remote(session), X {} }
                }
                if let Some(name) = menu() {
                    div { class: "absolute max-h-[calc(100vh-100px)] w-64 max-w-[calc(100vw-16px)] overflow-auto rounded-[7px] border border-[#48494e] bg-rd-surface py-2 text-sm text-[#ececec] shadow-lg {menu_anchor}",
                        class: if submenu().is_some() { "max-[1100px]:hidden" } else { "" }, role: "menu", aria_label: name,
                        match name {
                            "monitor" => rsx! { menus::MonitorMenu {} },
                            "control" => rsx! { menus::ControlMenu {} },
                            "display" => rsx! { menus::DisplayMenu { fullscreen, view_style, custom_scale } },
                            "keyboard" => rsx! { menus::KeyboardMenu {} },
                            _ => rsx! {},
                        }
                    }
                }
                if let Some(name) = submenu() {
                    div { class: "absolute z-50 max-h-[calc(100vh-100px)] w-64 max-w-[calc(100vw-16px)] overflow-auto rounded-[7px] border border-[#48494e] bg-rd-surface py-2 text-[#ececec] shadow-lg {menu_anchor} {submenu_offset}", role: "menu", aria_label: name,
                        button { class: "flex min-h-8 w-full items-center px-3 text-left text-xs hover:bg-[#3b5d91] min-[1100px]:hidden", r#type: "button", onclick: move |_| { let mut submenu = submenu; submenu.set(None); }, "‹ Back" }
                        menus::DisplaySubmenu { name }
                    }
                }
            }
            ToolbarHandle { collapsed, fullscreen, dock }
        }
        }
        if quality_dialog() { menus::CustomQuality { open: quality_dialog } }
        panels::ChatPanel { open: chat_open }
        panels::QualityMonitor { revision: revision() }
        if !status.remote.voice_call.is_empty() {
            div { class: "absolute bottom-3 left-3 z-20 rounded border border-[#48494e] bg-rd-surface px-3 py-2 text-xs text-white", "{status.remote.voice_call}" }
        }
    }
}
