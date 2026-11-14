use super::prelude::*;

pub(super) const LAST_REMOTE_ID_UI_OPTION: &str = "last-remote-id";
pub(super) const PEER_SORT_REMOTE_ID: &str = "Remote ID";
pub(super) const PEER_SORT_REMOTE_HOST: &str = "Remote Host";
pub(super) const PEER_SORT_USERNAME: &str = "Username";
pub(super) const PEER_SORT_STATUS: &str = "Status";

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct PeerCardData {
    pub(super) id: String,
    pub(super) alias: String,
    pub(super) username: String,
    pub(super) hostname: String,
    pub(super) platform: String,
    pub(super) favorite: bool,
    pub(super) online: bool,
}
pub(super) fn option(key: &str) -> String {
    crate::ui_interface::get_option(key)
}

pub(super) fn local_ui_option(key: &str) -> String {
    crate::ui_interface::get_local_ui_option(key.to_owned())
}

pub(super) fn remember_remote_id(peer_id: &str) {
    let peer_id = normalize_remote_id(peer_id.trim());
    if !peer_id.is_empty() {
        crate::ui_interface::set_local_ui_option(LAST_REMOTE_ID_UI_OPTION.to_owned(), peer_id);
    }
}

fn peer_card(id: String, info: &hbb_common::config::PeerInfoSerde, favorite: bool) -> PeerCardData {
    PeerCardData {
        id,
        alias: String::new(),
        username: info.username.clone(),
        hostname: info.hostname.clone(),
        platform: info.platform.clone(),
        favorite,
        online: false,
    }
}

pub(super) fn load_peer_cards(tab: &str) -> Vec<PeerCardData> {
    let favorites = crate::ui_interface::get_fav();
    match tab {
        "recent" => PeerConfig::peers(None)
            .into_iter()
            .map(|(id, _, config)| {
                let favorite = favorites.contains(&id);
                let mut peer = peer_card(id, &config.info, favorite);
                peer.alias = config.options.get("alias").cloned().unwrap_or_default();
                peer
            })
            .collect(),
        "favorites" => PeerConfig::peers(Some(favorites.clone()))
            .into_iter()
            .map(|(id, _, config)| {
                let mut peer = peer_card(id, &config.info, true);
                peer.alias = config.options.get("alias").cloned().unwrap_or_default();
                peer
            })
            .collect(),
        "discovered" => crate::ui_interface::get_lan_peers()
            .into_iter()
            .filter_map(|mut peer| {
                let id = peer.remove("id")?;
                Some(PeerCardData {
                    id,
                    alias: String::new(),
                    username: peer.remove("username").unwrap_or_default(),
                    hostname: peer.remove("hostname").unwrap_or_default(),
                    platform: peer.remove("platform").unwrap_or_default(),
                    favorite: false,
                    online: true,
                })
            })
            .collect(),
        "address-book" => load_cached_peer_cards("address-book"),
        "groups" => load_cached_peer_cards("groups"),
        _ => Vec::new(),
    }
}

fn load_cached_peer_cards(tab: &str) -> Vec<PeerCardData> {
    let favorites = crate::ui_interface::get_fav();
    if tab == "address-book" {
        return Ab::load()
            .ab_entries
            .into_iter()
            .flat_map(|entry| entry.peers)
            .map(|peer| PeerCardData {
                favorite: favorites.iter().any(|fav| fav == &peer.id),
                id: peer.id,
                alias: peer.alias,
                username: peer.username,
                hostname: peer.hostname,
                platform: peer.platform,
                online: false,
            })
            .collect();
    }
    Group::load()
        .peers
        .into_iter()
        .map(|peer| PeerCardData {
            favorite: favorites.iter().any(|fav| fav == &peer.id),
            id: peer.id,
            alias: String::new(),
            username: peer.username,
            hostname: peer.hostname,
            platform: peer.platform,
            online: false,
        })
        .collect()
}

pub(super) fn peer_display_name(peer: &PeerCardData) -> String {
    if !peer.alias.is_empty() {
        peer.alias.clone()
    } else if peer.id.is_empty() {
        "Unknown peer".to_owned()
    } else {
        peer.id.clone()
    }
}

pub(super) fn peer_subtitle(peer: &PeerCardData) -> String {
    match (peer.username.is_empty(), peer.hostname.is_empty()) {
        (false, false) => format!("{}@{}", peer.username, peer.hostname),
        (false, true) => peer.username.clone(),
        (true, false) => peer.hostname.clone(),
        (true, true) => String::new(),
    }
}

pub(super) fn normalize_peer_sort(value: &str) -> &'static str {
    match value {
        PEER_SORT_REMOTE_HOST => PEER_SORT_REMOTE_HOST,
        PEER_SORT_USERNAME => PEER_SORT_USERNAME,
        PEER_SORT_STATUS | "online" => PEER_SORT_STATUS,
        PEER_SORT_REMOTE_ID | "id" | "name" => PEER_SORT_REMOTE_ID,
        _ => PEER_SORT_REMOTE_ID,
    }
}

pub(super) fn sort_peer_cards(peers: &mut [PeerCardData], tab: &str, sort: &str) {
    if tab == "recent" {
        return;
    }
    match normalize_peer_sort(sort) {
        PEER_SORT_REMOTE_HOST => peers.sort_by_key(|peer| peer.hostname.to_lowercase()),
        PEER_SORT_USERNAME => peers.sort_by_key(|peer| peer.username.to_lowercase()),
        PEER_SORT_STATUS => peers.sort_by(|left, right| {
            right
                .online
                .cmp(&left.online)
                .then_with(|| peer_display_name(left).cmp(&peer_display_name(right)))
        }),
        _ => peers.sort_by_key(peer_display_name),
    }
}

pub(super) fn format_remote_id(id: &str) -> String {
    let compact = id.replace(' ', "");
    let suffix_start = compact.len().saturating_sub(2);
    let bytes = compact.as_bytes();
    let has_relay_suffix = bytes.ends_with(b"/r") || bytes.ends_with(b"\\r");
    let (digits, suffix) = if has_relay_suffix {
        compact.split_at(suffix_start)
    } else {
        (compact.as_str(), "")
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return id.to_owned();
    }
    let first_group = match digits.len() % 3 {
        0 => 3,
        remainder => remainder,
    };
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3 + suffix.len());
    formatted.push_str(&digits[..first_group]);
    for start in (first_group..digits.len()).step_by(3) {
        formatted.push(' ');
        formatted.push_str(&digits[start..(start + 3).min(digits.len())]);
    }
    formatted.push_str(suffix);
    formatted
}

pub(super) fn normalize_remote_id(id: &str) -> String {
    id.replace(' ', "")
}

pub(super) fn peer_platform_asset(platform: &str) -> String {
    let platform = platform.to_ascii_lowercase();
    let asset: &[u8] = if platform.contains("windows") {
        include_bytes!("../../res/dioxus/win.svg")
    } else if platform.contains("mac") || platform.contains("darwin") {
        include_bytes!("../../res/dioxus/mac.svg")
    } else if platform.contains("android") {
        include_bytes!("../../res/dioxus/android.svg")
    } else if platform.contains("linux") {
        include_bytes!("../../res/dioxus/linux.svg")
    } else {
        include_bytes!("../../res/dioxus/icon.svg")
    };
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(asset))
}
