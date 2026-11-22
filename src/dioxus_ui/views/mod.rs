mod connection_panel;
mod home;
mod home_page;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile;
mod peer_browser;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod remote_window;
mod settings_page;
mod sidebar;

pub(super) use home::Home;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub(super) use mobile::MobileHome;
