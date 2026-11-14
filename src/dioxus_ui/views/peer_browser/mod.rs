use super::super::prelude::*;

mod cards;
mod remote;

use cards::PeerCard;
pub(super) use remote::RemoteDesktop;

#[component]
pub(super) fn PeerBrowser() -> Element {
    let state = use_context::<super::super::state::AppState>();
    let peer_tab = state.peer_tab;
    let mut peer_search = state.peer_search;
    let mut peer_search_text = state.peer_search_text;
    let mut peer_view_mode = state.peer_view_mode;
    let mut peer_sort = state.peer_sort;
    let mut peer_refresh = state.peer_refresh;
    let ui_state = state.ui_state;
    let status = ui_state.read().clone();
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
    let service_status = (state.service_status)();

    if status.connected {
        return rsx! { RemoteDesktop {} };
    }

    rsx! {
        div { class: "mt-[19px] flex h-8 items-center overflow-x-auto border-b border-[#3a3b3e]",
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
                    css_class: "ml-0 h-8 w-8 shrink-0 text-[#9da0a4] [&>button]:text-inherit",
                title: "Refresh",
                aria_label: "Refresh",
                variant: ButtonVariant::Ghost,
                size: ButtonSize::IconSm,
                onclick: move |_| peer_refresh.set(peer_refresh() + 1),
                    RefreshCw { class: "block h-4 w-4 shrink-0 [&_svg]:block [&_svg]:size-full" }
            }
            Button {
                    css_class: "ml-0 h-8 w-8 shrink-0 text-[#9da0a4] [&>button]:text-inherit",
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
                    SelectTrigger { css_class: "[&>button]:flex [&>button]:h-[26px] [&>button]:w-full [&>button]:items-center [&>button]:justify-between [&>button]:rounded border [&>button]:border-[#4b4d52] [&>button]:bg-rd-surface [&>button]:px-1.5 [&>button]:py-0.5 [&>button]:text-[10px] [&>button]:text-[#aaaeb5]",
                        SelectValue { placeholder: Some("Sort by".to_owned()) }
                    }
                    SelectContent { css_class: "[&>[data-name=SelectContent]]:absolute [&>[data-name=SelectContent]]:top-[calc(100%+4px)] [&>[data-name=SelectContent]]:left-0 [&>[data-name=SelectContent]]:z-50 [&>[data-name=SelectContent]]:max-h-[220px] [&>[data-name=SelectContent]]:w-[150px] [&>[data-name=SelectContent]]:overflow-auto [&>[data-name=SelectContent]]:rounded-[5px] [&>[data-name=SelectContent]]:border [&>[data-name=SelectContent]]:border-[#4a4c51] [&>[data-name=SelectContent]]:bg-[#2b2c31] [&>[data-name=SelectContent]]:p-1 [&>[data-name=SelectContent]]:shadow-[0_6px_18px_#0008] [&>[data-name=SelectContent][data-state=closed]]:hidden [&>[data-name=SelectContent][data-state=closed]]:pointer-events-none [&>[data-name=SelectContent][data-state=open]]:block [&>[data-name=SelectContent][data-state=open]]:pointer-events-auto [&>[data-name=SelectContent][data-position=Above]]:top-auto [&>[data-name=SelectContent][data-position=Above]]:bottom-[calc(100%+4px)] max-[841px]:[&>[data-name=SelectContent]]:right-0 max-[841px]:[&>[data-name=SelectContent]]:left-auto [&>[data-name=SelectContent]>div:first-child]:hidden [&>[data-name=SelectContent]>div:last-child]:hidden [&_[data-name=SelectGroup]]:m-0 [&_[data-name=SelectGroup]]:list-none [&_[data-name=SelectGroup]]:p-0 [&_[data-name=SelectGroup]>[data-name=SelectOption]>svg]:hidden",
                        SelectGroup {
                            SelectOption { value: Some(PEER_SORT_REMOTE_ID.to_owned()), "Remote ID" }
                            SelectOption { value: Some(PEER_SORT_REMOTE_HOST.to_owned()), "Remote Host" }
                            SelectOption { value: Some(PEER_SORT_USERNAME.to_owned()), "Username" }
                            SelectOption { value: Some(PEER_SORT_STATUS.to_owned()), "Status" }
                        }
                    }
                }
            }
        }
        if peer_search() {
            Input {
                css_class: "mt-2 w-full [&>input]:rounded border [&>input]:border-[#494b50] [&>input]:bg-rd-surface [&>input]:px-2.5 [&>input]:py-[7px] [&>input]:text-xs [&>input]:text-[#eee] [&>input]:outline-none",
                placeholder: "Search",
                value: peer_search_text(),
                oninput: move |event: FormEvent| peer_search_text.set(event.value()),
            }
        }
        if displayed_peers.is_empty() {
            div { class: "flex flex-col items-center pt-[85px] text-[#7f8185] [&_p]:text-xs [&_p]:text-[#85878b]",
                div { class: "mb-2.5 grid h-11 w-11 place-items-center text-[#8c8e92]",
                    Frown { class: "block h-10 w-10 shrink-0 [&_svg]:block [&_svg]:size-full" }
                }
                p { "{empty_peers_label(&(peer_tab)())}" }
            }
        } else {
            div {
                class: match peer_view_mode().as_str() {
                    "grid" => "mt-3.5 grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3 max-[841px]:grid-cols-[minmax(0,1fr)]",
                    "tile" => "mt-3.5 grid grid-cols-1 gap-[7px] max-[841px]:grid-cols-[minmax(0,1fr)]",
                    _ => "mt-3.5 grid grid-cols-1 gap-[3px] max-[841px]:grid-cols-[minmax(0,1fr)]",
                },
                for peer in displayed_peers {
                    {
                        let selected = (state.selected_peer)() == peer.id;
                        rsx! { PeerCard { peer, selected } }
                    }
                }
            }
        }
        if !state.outgoing_only {
            ServiceStatus { status: service_status }
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
                "flex h-8 w-8 shrink-0 basis-8 items-center justify-center border-b-2 border-[#0071ff] text-[#0071ff] [&>button]:h-full [&>button]:w-full [&>button]:text-[17px] [&>button]:text-[#0071ff]"
            } else {
                "flex h-8 w-8 shrink-0 basis-8 items-center justify-center border-b-2 border-transparent text-[#70747c] [&>button]:h-full [&>button]:w-full [&>button]:text-[17px] [&>button]:text-[#70747c] hover:[&>button]:text-[#e1e1e1]"
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
