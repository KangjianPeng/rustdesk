use crate::ui_cm_interface::{self, Client, ConnectionManager, InvokeUiCM};
#[cfg(not(target_os = "android"))]
use base::config::keys::OPTION_ALLOW_REMOTE_CM_MODIFICATION;
#[cfg(target_os = "android")]
use hbb_common::tokio::sync::mpsc;
use hbb_common::tokio::sync::watch;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::LazyLock,
};

#[derive(Clone, Default)]
pub(super) struct ManagerState {
    pub(super) clients: BTreeMap<i32, Client>,
    pub(super) messages: VecDeque<(i32, String)>,
    pub(super) notice: String,
    pub(super) show_elevation: bool,
    pub(super) needs_deploy: bool,
}

pub(super) static STATE: LazyLock<watch::Sender<ManagerState>> = LazyLock::new(|| {
    let (tx, _) = watch::channel(ManagerState::default());
    tx
});

#[cfg(target_os = "android")]
pub(crate) fn set_needs_deploy(needs_deploy: bool) {
    STATE.send_modify(|state| state.needs_deploy = needs_deploy);
}

#[derive(Clone)]
pub(super) struct Handler;

#[cfg(target_os = "android")]
fn notify_service(name: &str, client: &Client) {
    let result = serde_json::to_string(client)
        .map_err(|error| error.to_string())
        .and_then(|json| {
            scrap::android::call_main_service_set_by_name(name, Some(&json), None)
                .map_err(|error| error.to_string())
        });
    if let Err(error) = result {
        hbb_common::throttled_log!(
            std::time::Duration::from_secs(30),
            warn,
            "Failed to notify Android service ({name}): {error}"
        );
    }
}

pub(super) fn push_message(id: i32, text: String) {
    STATE.send_modify(|state| {
        if state.messages.len() >= 100 {
            state.messages.pop_front();
        }
        state.messages.push_back((id, text));
    });
}

pub(super) async fn allow_action(id: i32) -> bool {
    #[cfg(not(target_os = "android"))]
    if !hbb_common::config::option2bool(
        OPTION_ALLOW_REMOTE_CM_MODIFICATION,
        &crate::ui_interface::get_local_option(OPTION_ALLOW_REMOTE_CM_MODIFICATION.to_owned()),
    ) {
        let click_time = hbb_common::get_time();
        ui_cm_interface::check_click_time(id);
        hbb_common::tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        if click_time.saturating_sub(ui_cm_interface::get_click_time()) <= 120 {
            STATE.send_modify(|state| {
                state.notice = "Remote input cannot modify incoming connections.".to_owned()
            });
            return false;
        }
    }
    STATE
        .borrow()
        .clients
        .get(&id)
        .is_some_and(|client| !client.disconnected)
}

#[cfg(not(target_os = "android"))]
pub(in crate::dioxus_ui) async fn close_manager(by_operator: bool) {
    let ids: Vec<_> = STATE
        .borrow()
        .clients
        .values()
        .filter(|client| !client.disconnected)
        .map(|client| client.id)
        .collect();
    if by_operator {
        for &id in &ids {
            if !allow_action(id).await {
                return;
            }
        }
    }
    for id in ids {
        #[cfg(target_os = "linux")]
        if !by_operator {
            ui_cm_interface::close_window(id);
            continue;
        }
        ui_cm_interface::close(id);
    }
    hbb_common::tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    let window = dioxus::desktop::window();
    window.close();
}

impl InvokeUiCM for Handler {
    fn add_connection(&self, client: &Client) {
        #[cfg(target_os = "android")]
        notify_service("add_connection", client);
        STATE.send_modify(|state| {
            state
                .clients
                .retain(|_, old| !(old.disconnected && old.peer_id == client.peer_id));
            state.clients.insert(client.id, client.clone());
        });
    }
    fn remove_connection(&self, id: i32, close: bool) {
        STATE.send_modify(|state| {
            if close {
                state.clients.remove(&id);
                state.messages.retain(|(client_id, _)| *client_id != id);
            } else if let Some(client) = state.clients.get_mut(&id) {
                client.disconnected = true;
            }
        });
    }
    fn new_message(&self, id: i32, text: String) {
        push_message(id, text);
    }
    fn change_theme(&self, dark: String) {
        STATE.send_modify(|state| state.notice = format!("Theme preference: {dark}"));
    }
    fn change_language(&self) {
        STATE.send_modify(|state| state.notice = "Language preference updated".to_owned());
    }
    fn show_elevation(&self, show: bool) {
        STATE.send_modify(|state| state.show_elevation = show);
    }
    fn update_voice_call_state(&self, client: &Client) {
        #[cfg(target_os = "android")]
        notify_service("update_voice_call_state", client);
        STATE.send_modify(|state| {
            state.clients.insert(client.id, client.clone());
        });
    }
    fn file_transfer_log(&self, action: &str, log: &str) {
        STATE.send_modify(|state| state.notice = format!("{action}: {log}"));
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn start_ipc() {
    #[cfg(target_os = "linux")]
    std::thread::spawn(crate::ipc::start_pa);
    std::thread::spawn(|| {
        ui_cm_interface::start_ipc(ConnectionManager {
            ui_handler: Handler,
        })
    });
}

#[cfg(target_os = "android")]
pub(crate) fn start_channel(
    rx: mpsc::UnboundedReceiver<crate::ipc::Data>,
    tx: mpsc::UnboundedSender<crate::ipc::Data>,
) {
    std::thread::spawn(move || {
        ui_cm_interface::start_listen(
            ConnectionManager {
                ui_handler: Handler,
            },
            rx,
            tx,
        )
    });
}
