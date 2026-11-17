//! Dioxus desktop and mobile frontend.

mod app;
mod chrome;
mod common;
mod components;
mod i18n;
pub(crate) mod connection_manager;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod desktop;
mod peers;
mod prelude;
mod runtime;
mod session;
mod settings;
mod state;
#[cfg(test)]
mod tests;
mod views;
pub use runtime::start;
pub(crate) use session::{current_session, DioxusHandler};
