pub fn start(args: Vec<String>) {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if args.first().map(String::as_str) == Some("--cm") {
        connection_manager::start_ipc();
    }
    #[cfg(target_os = "android")]
    dioxus::LaunchBuilder::mobile()
        .with_context(args)
        .launch(Root);
    #[cfg(not(target_os = "android"))]
    {
        let title = if is_remote_window(&args) {
            format!("{} - RustDesk", args[1])
        } else {
            "RustDesk".to_owned()
        };
        let runtime = if hbb_common::tokio::runtime::Handle::try_current().is_err() {
            match hbb_common::tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => Some(runtime),
                Err(error) => {
                    log::error!("Failed to start the UI runtime: {error}");
                    return;
                }
            }
        } else {
            None
        };
        let _guard = runtime.as_ref().map(|runtime| runtime.enter());
        super::native::launch(Root, args, title);
    }
}

pub(super) fn is_remote_window(args: &[String]) -> bool {
    args.len() > 1 && matches!(args[0].as_str(), "--connect" | "--view-camera")
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(super) fn open_remote_window(
    peer_id: String,
    password: String,
    mut args: Vec<String>,
    state: SyncSignal<DioxusUiState>,
) {
    let peer_id = normalize_remote_id(peer_id.trim());
    if peer_id.is_empty() {
        return;
    }
    remember_remote_id(&peer_id);
    if !args
        .first()
        .is_some_and(|arg| matches!(arg.as_str(), "--connect" | "--view-camera"))
    {
        args.insert(0, "--connect".to_owned());
    }
    args.insert(1, peer_id);
    if !password.is_empty() {
        args.extend(["--password".to_owned(), password]);
    }
    match crate::run_me(args) {
        Ok(mut child) => {
            update_ui_state(state, |state| {
                state.status.clear();
                state.message.clear();
            });
            if let Err(error) = std::thread::Builder::new()
                .name("rustdesk-session-window".to_owned())
                .spawn(move || {
                    if let Err(error) = child.wait() {
                        log::warn!("Failed to wait for the remote window: {error}");
                    }
                })
            {
                log::error!("Failed to start the remote window monitor: {error}");
            }
        }
        Err(error) => {
            log::error!("Failed to open the remote window: {error}");
            update_ui_state(state, |state| {
                state.status = "Connection error".to_owned();
                state.message = error.to_string();
            });
        }
    }
}

#[component]
fn Root() -> Element {
    let args = use_context::<Vec<String>>();
    if args.first().map(String::as_str) == Some("--cm") {
        return rsx! {
            connection_manager::Window {}
        };
    }
    rsx! {
        super::app::App {}
        if cfg!(target_os = "android") {
            div { class: "fixed top-12 right-3 left-3 z-50 [&_.cm-panel]:max-h-[calc(100vh-64px)] [&_.cm-panel]:rounded-lg [&_.cm-panel]:border [&_.cm-panel]:border-[#48494e] [&_.cm-panel]:shadow-[0_8px_24px_#0008]", connection_manager::Panel {} }
        }
    }
}

pub(crate) async fn load_local_id() -> String {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        hbb_common::config::Config::get_id()
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        match hbb_common::tokio::task::spawn_blocking(crate::ipc::get_id).await {
            Ok(id) => id,
            Err(error) => {
                log::error!("Failed to load local ID: {error}");
                String::new()
            }
        }
    }
}

pub(crate) async fn load_temporary_password() -> String {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        crate::ui_interface::temporary_password()
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        match hbb_common::tokio::task::spawn_blocking(crate::ui_interface::temporary_password).await
        {
            Ok(password) => password,
            Err(error) => {
                log::error!("Failed to load temporary password: {error}");
                String::new()
            }
        }
    }
}

use super::prelude::*;
