use super::super::super::prelude::*;
use super::{
    docking::{Edge, ToolbarDock},
    menus, panels,
};
use icons::{
    ChevronDown, ChevronLeft, ChevronRight, ChevronUp, Grip, Keyboard, Maximize, MessageSquare,
    Minimize, MousePointer2, Phone, Pin, PinOff, Video,
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
            r#type: "button", title: tr(&title), aria_label: tr(&title), disabled,
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
            class: "{ICON_BUTTON} bg-[#0071ff] hover:bg-[#4f95ff]", r#type: "button", title: tr(&title), aria_label: tr(&title),
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
            button { id: "remote-toolbar-drag", class: "cursor-grab touch-none active:cursor-grabbing", r#type: "button", title: tr("Drag toolbar"), aria_label: tr("Drag toolbar"),
                onmounted: move |_| super::docking::install_drag(dock),
                onpointerdown: move |_| toolbar.dismiss(), Grip {}
            }
            if collapsed() && status.displays.len() > 1 {
                button { r#type: "button", title: tr("Switch display"), aria_label: tr("Switch display"),
                    onclick: move |_| if let Some(active) = state.session.peek().as_ref() {
                        active.switch_display((status.current_display + 1) % status.displays.len() as i32);
                    },
                    Monitor {}
                }
            }
            button { r#type: "button", title: tr(if fullscreen() { "Exit Fullscreen" } else { "Fullscreen" }), aria_label: tr(if fullscreen() { "Exit Fullscreen" } else { "Fullscreen" }),
                onclick: move |_| {
                    let next = !fullscreen();
                    dioxus::desktop::window().set_fullscreen(next);
                    fullscreen.set(next);
                    toolbar.dismiss();
                },
                if fullscreen() { Minimize {} } else { Maximize {} }
            }
            if fullscreen() {
                button { r#type: "button", title: tr("Minimize"), aria_label: tr("Minimize"),
                    onclick: move |_| dioxus::desktop::window().set_minimized(true), Minus {}
                }
            }
            button { r#type: "button", title: tr(if collapsed() { "Show Toolbar" } else { "Hide Toolbar" }), aria_label: tr(if collapsed() { "Show Toolbar" } else { "Hide Toolbar" }),
                onclick: move |_| {
                    collapsed.set(!collapsed());
                    toolbar.dismiss();
                    if let Some(active) = state.session.peek().as_ref() { active.toggle_option("collapse_toolbar".to_owned()); }
                },
                match (dock().edge, collapsed()) {
                    (Edge::Top, true) | (Edge::Bottom, false) => rsx! { ChevronDown {} },
                    (Edge::Top, false) | (Edge::Bottom, true) => rsx! { ChevronUp {} },
                    (Edge::Left, true) | (Edge::Right, false) => rsx! { ChevronRight {} },
                    (Edge::Left, false) | (Edge::Right, true) => rsx! { ChevronLeft {} },
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
    dock: SyncSignal<ToolbarDock>,
) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let session = state.session;
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
    let menu_anchor = edge.menu_class();
    let orientation = match edge {
        Edge::Top => "col-start-2 row-start-1 flex-col",
        Edge::Bottom => "col-start-2 row-start-2 flex-col-reverse",
        Edge::Left => "col-start-1 row-start-2 flex-row",
        Edge::Right => "col-start-2 row-start-2 flex-row-reverse",
    };
    let menu_direction = if edge == Edge::Right || (edge.horizontal() && dock().fraction > 0.5) {
        "flex-row-reverse"
    } else {
        "flex-row"
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
        div {
            id: "remote-toolbar",
            class: "relative z-40 flex h-fit w-fit min-h-0 min-w-0 max-h-full max-w-full items-center select-none {orientation}",
            onmouseenter: move |_| hovering.set(true),
            onmouseleave: move |_| hovering.set(false),
            onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); toolbar.dismiss(); },
            if !collapsed() {
                div { class: "flex min-h-0 min-w-0 gap-1 rounded border border-[#48494e] bg-rd-surface p-1.5 shadow-lg",
                    class: if dock().edge.horizontal() { "max-w-full flex-row items-center overflow-x-auto overflow-y-hidden" } else { "max-h-[calc(100vh-52px)] flex-col items-center overflow-x-hidden overflow-y-auto" },
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
                    div { class: "absolute flex w-max items-start gap-1 overflow-y-auto text-sm text-[#ececec] {menu_anchor} {menu_direction}",
                        class: if edge.horizontal() { "max-h-[calc(100vh-100px)] max-w-[calc(100vw-16px)]" } else { "max-h-full max-w-[calc(100vw-84px)]" },
                        class: if edge.horizontal() && dock().fraction > 0.5 { "right-0" } else if edge.horizontal() { "left-0" } else { "" },
                    div { class: "w-64 shrink-0 max-w-full rounded-[7px] border border-[#48494e] bg-rd-surface py-2 shadow-lg",
                        class: if submenu().is_some() { "max-[1100px]:hidden" } else { "" }, role: "menu", aria_label: name,
                        match name {
                            "monitor" => rsx! { menus::MonitorMenu {} },
                            "control" => rsx! { menus::ControlMenu {} },
                            "display" => rsx! { menus::DisplayMenu { fullscreen, view_style, custom_scale } },
                            "keyboard" => rsx! { menus::KeyboardMenu {} },
                            _ => rsx! {},
                        }
                    }
                if let Some(name) = submenu() {
                    div { class: "w-64 shrink-0 max-w-full rounded-[7px] border border-[#48494e] bg-rd-surface py-2 shadow-lg", role: "menu", aria_label: name,
                        button { class: "flex min-h-8 w-full items-center px-3 text-left text-xs hover:bg-[#3b5d91] min-[1100px]:hidden", r#type: "button", onclick: move |_| { let mut submenu = submenu; submenu.set(None); }, "‹ Back" }
                        menus::DisplaySubmenu { name }
                    }
                }
                    }
                }
            }
            ToolbarHandle { collapsed, fullscreen, dock }
        }
        if quality_dialog() { menus::CustomQuality { open: quality_dialog } }
        panels::ChatPanel { open: chat_open }
        panels::QualityMonitor { revision: revision() }
        if !status.remote.voice_call.is_empty() {
            div { class: "pointer-events-none absolute bottom-3 left-3 z-10 flex max-w-[calc(100%-24px)] items-center gap-2 rounded border bg-card px-3 py-2 text-xs text-card-foreground shadow-lg",
                Phone { class: "size-4 shrink-0" }
                span { {tr(&status.remote.voice_call)} }
            }
        }
    }
}
