//! Shared imports for Dioxus UI modules.

pub(super) use crate::{
    client::QualityStatus,
    ui_session_interface::{InvokeUiSession, Session},
};
pub(super) use base::{config::keys::*, message_proto::*};
pub(super) use dioxus::{html::input_data::MouseButton, prelude::*};
pub(super) use hbb_common::{
    base64::{engine::general_purpose::STANDARD, Engine as _},
    config::{Ab, Group, PeerConfig},
    log,
    rendezvous_proto::ConnType,
};
pub(super) use icons::{
    ArrowRight, BookUser, CircleHelp, Copy, Cpu, Ellipsis, Frown, History, House, Info, LayoutGrid,
    List, Minus, Monitor, Network, Printer, Radar, RefreshCw, Rows3, Search, Settings, Shield,
    Square, Star, StarOff, User, Users, X,
};
pub(super) use registry::ui::select::{
    Select as RustUiSelect, SelectContent as RustUiSelectContent, SelectGroup, SelectOption,
    SelectTrigger as RustUiSelectTrigger, SelectValue,
};
pub(super) use registry::ui::{
    button::{Button as RustUiButton, ButtonSize, ButtonVariant},
    card::Card as RustUiCard,
    checkbox::Checkbox as RustUiCheckbox,
    input::{Input as RustUiInput, InputType},
};
pub(super) use scrap::ImageFormat;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub(super) use std::sync::{Condvar, Weak};
pub(super) use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[cfg(any(target_os = "android", target_os = "ios"))]
pub(super) use super::session::send_touch_mouse_event;

pub(super) use super::{
    chrome::AppChrome,
    common::{
        current_help_cards, read_service_status, update_signal, update_ui_state, CodecUiState,
        DioxusUiState, HelpCard, ServiceStatus,
    },
    components::{
        copy_text_to_clipboard, open_external_url, Button, Card, Checkbox, Input, Select,
        SelectContent, SelectTrigger, UiIcon, UiIconName,
    },
    connection_manager,
    i18n::tr,
    peers::{
        format_remote_id, load_peer_cards, local_ui_option, normalize_peer_sort,
        normalize_remote_id, option, peer_display_name, peer_platform_asset, peer_subtitle,
        remember_remote_id, sort_peer_cards, PeerCardData, LAST_REMOTE_ID_UI_OPTION,
        PEER_SORT_REMOTE_HOST, PEER_SORT_REMOTE_ID, PEER_SORT_STATUS, PEER_SORT_USERNAME,
    },
    session::{
        new_session, send_keyboard_event, send_mouse_event, send_touch_event, send_wheel_event,
        set_current_session, start_session, DioxusHandler,
    },
    settings::{rustdesk_icon_data_uri, SettingsContent},
};
