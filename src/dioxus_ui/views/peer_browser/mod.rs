use super::super::prelude::*;

mod cards;
mod remote;

use cards::PeerCard;
pub(super) use remote::RemoteDesktop;

#[component]
pub(super) fn PeerBrowser(peer_menu: Signal<String>) -> Element {
    let state = use_context::<super::super::state::AppState>();
    let peer_tab = state.peer_tab;
    let mut peer_search = state.peer_search;
    let mut peer_search_text = state.peer_search_text;
    let mut peer_view_mode = state.peer_view_mode;
    let mut peer_sort = state.peer_sort;
    let mut peer_refresh = state.peer_refresh;
    let ui_state = state.ui_state;
    if ui_state.read().connected {
        return rsx! { RemoteDesktop {} };
    }
    let filter = (peer_search_text)().trim().to_ascii_lowercase();
    let mut displayed_peers: Vec<PeerCardData> = (state.peer_cards)()
        .into_iter()
        .filter(|peer| {
            filter.is_empty()
                || peer.id.to_ascii_lowercase().contains(&filter)
                || peer.alias.to_ascii_lowercase().contains(&filter)
                || peer.username.to_ascii_lowercase().contains(&filter)
                || peer.hostname.to_ascii_lowercase().contains(&filter)
        })
        .collect();
    sort_peer_cards(&mut displayed_peers, &(peer_tab)(), &(peer_sort)());

    rsx! {
        div { class: "flex min-h-0 flex-1 flex-col",
            div { class: "flex min-h-9 shrink-0 flex-wrap items-center gap-1",
                PeerTabButton { tab: "recent", title: "Recent sessions", icon: UiIconName::History }
                PeerTabButton { tab: "favorites", title: "Favorites", icon: UiIconName::Star }
                PeerTabButton { tab: "discovered", title: "Discovered", icon: UiIconName::Radar }
                PeerTabButton { tab: "address-book", title: "Address book", icon: UiIconName::BookUser }
                PeerTabButton { tab: "groups", title: "Groups", icon: UiIconName::Users }
                Button {
                        css_class: "ml-auto h-8 w-8 shrink-0",
                    title: "Search",
                    aria_label: "Search",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| peer_search.set(!peer_search()),
                        Search { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    title: "Refresh",
                    aria_label: "Refresh",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| peer_refresh.set(peer_refresh() + 1),
                        RefreshCw { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                Button {
                    title: "Change view",
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::IconSm,
                    onclick: move |_| {
                        let mode = match peer_view_mode().as_str() {
                            "grid" => "tile",
                            "tile" => "list",
                            _ => "grid",
                        };
                        peer_view_mode.set(mode.to_owned());
                        crate::ui_interface::set_local_ui_option(
                            OPTION_UI_PEER_CARD_TYPE.to_owned(),
                            match mode { "grid" => "0", "tile" => "1", _ => "2" }.to_owned(),
                        );
                    },
                    ViewModeIcon {
                        mode: peer_view_mode(),
                    }
                }
                if peer_tab() != "recent" {
                    Select {
                        css_class: "relative ml-[5px] w-[104px] shrink-0 basis-[104px] max-[841px]:w-[88px] max-[841px]:basis-[88px]",
                        default_value: Some(peer_sort()),
                        on_change: move |value: Option<String>| {
                            if let Some(value) = value {
                                let value = normalize_peer_sort(&value).to_owned();
                                peer_sort.set(value.clone());
                                crate::ui_interface::set_local_ui_option(OPTION_UI_PEER_SORTING.to_owned(), value);
                            }
                        },
                        SelectTrigger {
                            SelectValue { placeholder: Some("Sort by".to_owned()) }
                        }
                        SelectContent {
                            SelectGroup {
                                SelectOption { value: Some(PEER_SORT_REMOTE_ID.to_owned()), {tr("Remote ID")} }
                                SelectOption { value: Some(PEER_SORT_REMOTE_HOST.to_owned()), {tr("Remote Host")} }
                                SelectOption { value: Some(PEER_SORT_USERNAME.to_owned()), {tr("Username")} }
                                SelectOption { value: Some(PEER_SORT_STATUS.to_owned()), {tr("Status")} }
                            }
                        }
                    }
                }
            }
            if peer_search() {
                Input {
                    css_class: "mt-2 w-full",
                    placeholder: "Search",
                    value: peer_search_text(),
                    oninput: move |event: FormEvent| peer_search_text.set(event.value()),
                }
            }
            div { class: "my-3 min-h-0 flex-1 overflow-auto",
                if displayed_peers.is_empty() {
                    div { class: "flex flex-col items-center pt-[85px] text-muted-foreground [&_p]:text-xs [&_p]:text-muted-foreground",
                        div { class: "mb-2.5 grid h-11 w-11 place-items-center text-muted-foreground",
                            Frown { class: "block h-10 w-10 shrink-0 [&_svg]:block [&_svg]:size-full" }
                        }
                        p { {tr(empty_peers_label(&(peer_tab)()))} }
                    }
                } else {
                    div {
                        role: "list",
                        class: match peer_view_mode().as_str() {
                            "grid" => "grid grid-cols-[repeat(auto-fill,minmax(min(220px,100%),220px))] gap-3",
                            "tile" => "grid grid-cols-[repeat(auto-fill,minmax(min(220px,100%),220px))] gap-3",
                            _ => "grid grid-cols-1 gap-2",
                        },
                        for peer in displayed_peers {
                            {
                                let selected = (state.selected_peer)() == peer.id;
                                rsx! { PeerCard { peer, selected, peer_menu } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PeerTabButton(tab: &'static str, title: &'static str, icon: UiIconName) -> Element {
    let state = use_context::<super::super::state::AppState>();
    let mut peer_tab = state.peer_tab;
    rsx! {
        Button {
            css_class: if peer_tab() == tab {
                "bg-accent text-accent-foreground"
            } else {
                "text-muted-foreground"
            },
            title,
            aria_label: title,
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            onclick: move |_| peer_tab.set(tab.to_owned()),
            UiIcon { icon }
        }
    }
}

#[component]
fn ViewModeIcon(mode: String) -> Element {
    rsx! {
        if mode == "grid" {
            LayoutGrid { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else if mode == "tile" {
            Rows3 { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        } else {
            List { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
        }
    }
}

fn empty_peers_label(tab: &str) -> &'static str {
    match tab {
        "favorites" => "No favorite peers",
        "discovered" => "No discovered peers",
        "address-book" => "No address book peers",
        "groups" => "No groups",
        _ => "No recent connections",
    }
}
