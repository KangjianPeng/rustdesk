use super::views::Home;
use dioxus::prelude::*;

/// Application-level view boundary.
///
/// Startup, session wiring, and the page-specific markup live in separate
/// modules so new pages do not grow the runtime entry point.
#[component]
pub(crate) fn App() -> Element {
    rsx! { Home {} }
}
