use super::super::super::prelude::*;

#[component]
pub(super) fn PeerCard(peer: PeerCardData, selected: bool, mut peer_menu: Signal<String>) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let mut selected_peer = state.selected_peer;
    let mut peer_refresh = state.peer_refresh;
    let view_mode = (state.peer_view_mode)();
    let session = state.session;
    let ui_state = state.ui_state;
    let id = peer.id.clone();
    let selected_id = id.clone();
    let favorite_id = peer.id.clone();
    let connect_peer_id = peer.id.clone();
    let menu_id = peer.id.clone();
    let copy_id = peer.id.clone();
    let display_name = peer_display_name(&peer);
    let subtitle = peer_subtitle(&peer);
    let grid = view_mode == "grid";
    rsx! {
        div {
            role: "listitem",
            class: if peer_menu() == peer.id { "relative z-30 min-w-0" } else { "relative min-w-0" },
            onclick: move |_| selected_peer.set(selected_id.clone()),
            ondoubleclick: move |_| start_session(session, id.clone(), String::new(), Vec::new(), ui_state),
            div {
                class: match (view_mode.as_str(), selected) {
                    ("grid", true) => "box-border flex h-[140px] w-[220px] flex-col overflow-hidden rounded-[14px] border-2 border-solid border-primary bg-card",
                    ("grid", false) => "box-border flex h-[140px] w-[220px] flex-col overflow-hidden rounded-[14px] border-2 border-solid border-transparent bg-card hover:border-ring",
                    ("tile", true) => "box-border flex h-[42px] w-[220px] overflow-hidden rounded-[5px] border-2 border-solid border-primary bg-card",
                    ("tile", false) => "box-border flex h-[42px] w-[220px] overflow-hidden rounded-[5px] border-2 border-solid border-transparent bg-card hover:border-ring",
                    (_, true) => "box-border flex h-[45px] w-full overflow-hidden rounded-[5px] border-2 border-solid border-primary bg-card",
                    _ => "box-border flex h-[45px] w-full overflow-hidden rounded-[5px] border-2 border-solid border-transparent bg-card hover:border-ring",
                },
                div { class: if grid { "relative flex min-h-0 flex-1 flex-col items-center justify-center gap-1 bg-muted px-1" } else { "relative flex w-[42px] shrink-0 items-center justify-center bg-muted" },
                    img {
                        src: "{peer_platform_asset(&peer.platform)}", alt: "{peer.platform}",
                        class: if grid { "relative z-10 h-[60px] w-[60px] shrink-0 object-contain" } else { "relative z-10 h-[30px] w-[30px] object-contain" },
                    }
                    if grid && !subtitle.is_empty() {
                        span { class: "relative z-10 block w-full min-w-0 truncate text-center text-xs text-muted-foreground", title: "{subtitle}", "{subtitle}" }
                    }
                }
                div { class: if grid { "box-border flex h-[36px] shrink-0 items-center px-2.5" } else { "box-border flex min-w-0 flex-1 items-center pl-2" },
                    div { class: "min-w-0 flex-1",
                        div { class: "flex min-w-0 items-center gap-2 text-sm text-foreground",
                            span { class: if peer.online { "h-2 w-2 shrink-0 rounded-full bg-success" } else { "h-2 w-2 shrink-0 rounded-full bg-muted-foreground" } }
                            span { class: "min-w-0 flex-1 truncate", title: "{display_name}", "{display_name}" }
                        }
                        if !grid && !subtitle.is_empty() {
                            div { class: "mt-0.5 min-w-0 truncate text-[11px] leading-none text-muted-foreground", title: "{subtitle}", "{subtitle}" }
                        }
                    }
                    div {
                        onmousedown: move |event| event.stop_propagation(),
                        Button {
                            css_class: "h-7 w-6 shrink-0 bg-transparent p-0 text-muted-foreground",
                            title: "More",
                            aria_label: format!("More actions for {}", peer.id),
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::IconSm,
                            onclick: move |event: dioxus::prelude::MouseEvent| {
                                event.stop_propagation();
                                peer_menu.set(if peer_menu() == menu_id { String::new() } else { menu_id.clone() });
                            },
                            icons::EllipsisVertical { class: "block h-4 w-4 [&_svg]:block [&_svg]:size-full" }
                        }
                    }
                }
            }
            if peer_menu() == peer.id {
                div {
                    class: if grid { "box-border absolute right-0 bottom-8 z-40 w-[190px] rounded-[5px] border border-solid border-border bg-popover p-1 shadow-[0_6px_18px_#0008]" } else { "box-border absolute top-0 right-0 z-40 w-[190px] rounded-[5px] border border-solid border-border bg-popover p-1 shadow-[0_6px_18px_#0008]" },
                    role: "menu",
                    onmousedown: move |event| event.stop_propagation(),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape { peer_menu.set(String::new()); }
                    },
                    Button {
                        css_class: "box-border flex h-8 w-full items-center gap-2 rounded px-2 text-xs text-foreground hover:bg-accent justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |event: dioxus::prelude::MouseEvent| {
                            event.stop_propagation();
                            peer_menu.set(String::new());
                            start_session(session, connect_peer_id.clone(), String::new(), Vec::new(), ui_state);
                        },
                        ArrowRight { class: "block h-4 w-4 [&_svg]:block [&_svg]:size-full" }
                        {tr("Connect")}
                    }
                    Button {
                        css_class: "box-border flex h-8 w-full items-center gap-2 rounded px-2 text-xs text-foreground hover:bg-accent justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |event: dioxus::prelude::MouseEvent| {
                            event.stop_propagation();
                            copy_text_to_clipboard(copy_id.clone());
                            peer_menu.set(String::new());
                        },
                        Copy { class: "block h-4 w-4 [&_svg]:block [&_svg]:size-full" }
                        {tr("Copy ID")}
                    }
                    Button {
                        css_class: "box-border flex h-8 w-full items-center gap-2 rounded px-2 text-xs text-foreground hover:bg-accent justify-start gap-2",
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        onclick: move |event: dioxus::prelude::MouseEvent| {
                            event.stop_propagation();
                            let mut favorites = crate::ui_interface::get_fav();
                            if peer.favorite { favorites.retain(|favorite| favorite != &favorite_id); }
                            else if !favorites.iter().any(|favorite| favorite == &favorite_id) { favorites.push(favorite_id.clone()); }
                            crate::ui_interface::store_fav(favorites);
                            peer_menu.set(String::new());
                            peer_refresh.set(peer_refresh() + 1);
                        },
                        if peer.favorite {
                            StarOff { class: "block h-4 w-4 [&_svg]:block [&_svg]:size-full" }
                        } else {
                            Star { class: "block h-4 w-4 [&_svg]:block [&_svg]:size-full" }
                        }
                        if peer.favorite { {tr("Remove from Favorites")} } else { {tr("Add to Favorites")} }
                    }
                }
            }
        }
    }
}
