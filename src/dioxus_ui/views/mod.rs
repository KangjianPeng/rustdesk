mod connection_panel;
mod home;
mod home_page;
mod peer_browser;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod remote_window;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(in crate::dioxus_ui) use remote_window::docking;
mod settings_page;
mod sidebar;

pub(super) use home::Home;
