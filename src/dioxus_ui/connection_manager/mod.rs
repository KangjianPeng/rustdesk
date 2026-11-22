//! Incoming connection manager UI and its IPC bridge.

#[cfg(target_os = "android")]
use crate::ui_cm_interface;

mod backend;
mod client_card;
#[cfg(target_os = "android")]
mod panel;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod window;

#[cfg(target_os = "android")]
pub(crate) use backend::set_needs_deploy;
#[cfg(target_os = "android")]
pub(crate) use backend::start_channel;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) use backend::start_ipc;
#[cfg(target_os = "android")]
pub(crate) use panel::Panel;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(super) use window::{window_size, Window};

#[cfg(target_os = "android")]
pub(super) fn subscribe_mobile_chat(
) -> hbb_common::tokio::sync::watch::Receiver<backend::ManagerState> {
    backend::STATE.subscribe()
}

#[cfg(target_os = "android")]
pub(super) fn send_mobile_chat(id: i32, text: String) -> bool {
    if !backend::STATE
        .borrow()
        .clients
        .get(&id)
        .is_some_and(|client| client.authorized && !client.disconnected)
    {
        return false;
    }
    ui_cm_interface::send_chat(id, text.clone());
    backend::push_message(id, format!("You: {text}"));
    true
}
