use super::super::super::prelude::*;

#[component]
pub(super) fn PeerCard(peer: PeerCardData, selected: bool) -> Element {
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
    let display_name = peer_display_name(&peer);
    let subtitle = peer_subtitle(&peer);
    rsx! {
        div {
            class: match (view_mode.as_str(), selected) {
                ("tile", true) => "relative flex min-h-[66px] cursor-pointer overflow-hidden rounded-[5px] border border-[#5d91e8] bg-[#303238] shadow-[inset_0_0_0_1px_#5d91e8] hover:border-rd-accent",
                ("tile", false) => "relative flex min-h-[66px] cursor-pointer overflow-hidden rounded-[5px] border border-[#3d3f44] bg-[#292a2f] hover:border-rd-accent hover:bg-[#303238]",
                ("list", true) => "relative flex min-h-12 cursor-pointer overflow-hidden rounded-[3px] border border-[#5d91e8] bg-[#303238] shadow-[inset_0_0_0_1px_#5d91e8] hover:border-rd-accent",
                ("list", false) => "relative flex min-h-12 cursor-pointer overflow-hidden rounded-[3px] border border-[#3d3f44] bg-[#292a2f] hover:border-rd-accent hover:bg-[#303238]",
                (_, true) => "relative flex min-h-[92px] cursor-pointer overflow-hidden rounded-[7px] border border-[#5d91e8] bg-[#303238] shadow-[inset_0_0_0_1px_#5d91e8] hover:border-rd-accent",
                _ => "relative flex min-h-[92px] cursor-pointer overflow-hidden rounded-[7px] border border-[#3d3f44] bg-[#292a2f] hover:border-rd-accent hover:bg-[#303238]",
            },
            onclick: move |_| selected_peer.set(selected_id.clone()),
            ondoubleclick: move |_| start_session(session, id.clone(), String::new(), Vec::new(), ui_state),
            div { class: match view_mode.as_str() {
                "tile" => "grid w-11 shrink-0 basis-11 place-items-center bg-[#34363b] text-[27px] text-[#7fa9ed]",
                "list" => "grid w-[38px] shrink-0 basis-[38px] place-items-center bg-[#34363b] text-[27px] text-[#7fa9ed]",
                _ => "grid w-[58px] shrink-0 basis-[58px] place-items-center bg-[#34363b] text-[27px] text-[#7fa9ed]",
            },
                img { src: "{peer_platform_asset(&peer.platform)}", alt: "{peer.platform}", class: "h-[31px] w-[31px] object-contain" }
            }
            div { class: match view_mode.as_str() {
                "tile" => "min-w-0 flex-1 pt-[9px] pr-9 pb-2 pl-2.5",
                "list" => "flex min-w-0 flex-1 items-center gap-3 py-2 pr-9 pl-2.5",
                _ => "min-w-0 flex-1 pt-3.5 pr-9 pb-2.5 pl-3",
            },
                div { class: "flex items-center gap-[7px] overflow-hidden text-[13px] font-medium text-[#ececec] text-ellipsis whitespace-nowrap",
                    span { class: if peer.online { "h-2 w-2 shrink-0 rounded-full bg-[#42c49c]" } else { "h-2 w-2 shrink-0 rounded-full bg-[#777a7f]" } }
                    "{display_name}"
                }
                if !subtitle.is_empty() { div { class: if view_mode == "list" { "mt-0 overflow-hidden text-[11px] text-[#9da0a6] text-ellipsis whitespace-nowrap" } else { "mt-[7px] overflow-hidden text-[11px] text-[#9da0a6] text-ellipsis whitespace-nowrap" }, "{subtitle}" } }
                div { class: "mt-[3px] overflow-hidden font-mono text-[11px] text-[#70747c] text-ellipsis whitespace-nowrap", "{peer.id}" }
            }
            Button {
                css_class: "absolute top-1.5 right-2 h-[22px] w-[22px] bg-transparent p-0 text-[13px] text-[#e7bd43] [&>button]:h-full [&>button]:w-full",
                title: if peer.favorite { "Remove from Favorites" } else { "Add to Favorites" },
                variant: ButtonVariant::Ghost,
                size: ButtonSize::IconSm,
                onclick: move |event: dioxus::prelude::MouseEvent| {
                    event.stop_propagation();
                    let mut favorites = crate::ui_interface::get_fav();
                    if peer.favorite { favorites.retain(|favorite| favorite != &favorite_id); }
                    else if !favorites.iter().any(|favorite| favorite == &favorite_id) { favorites.push(favorite_id.clone()); }
                    crate::ui_interface::store_fav(favorites);
                    peer_refresh.set(peer_refresh() + 1);
                },
                if peer.favorite {
                    Star { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                } else {
                    StarOff { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
            }
            Button {
                css_class: "absolute right-[7px] bottom-[7px] h-6 w-6 rounded-full bg-transparent p-0 text-[17px] text-[#8eb6f4] hover:bg-[#3d5e92] hover:text-white [&>button]:h-full [&>button]:w-full",
                title: "Connect",
                variant: ButtonVariant::Ghost,
                size: ButtonSize::IconSm,
                onclick: move |_| start_session(session, connect_peer_id.clone(), String::new(), Vec::new(), ui_state),
                ArrowRight { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
            }
        }
    }
}
