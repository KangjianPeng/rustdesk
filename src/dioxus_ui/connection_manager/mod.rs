//! Incoming connection manager UI and its IPC bridge.

mod backend;
mod client_card;
mod panel;
mod window;

#[cfg(target_os = "android")]
pub(crate) use backend::set_needs_deploy;
#[cfg(target_os = "android")]
pub(crate) use backend::start_channel;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) use backend::start_ipc;
pub(crate) use panel::Panel;
pub(crate) use window::Window;
