use super::super::super::prelude::*;
use crate::dioxus_ui::session::ChatMessage;

#[component]
pub(super) fn ChatPanel(mut open: Signal<bool>) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    let status = state.ui_state.read().clone();
    let mut text = use_signal(String::new);
    let mut received = use_signal(|| status.remote.chat.len());
    use_effect(move || {
        let count = state.ui_state.read().remote.chat.len();
        if count > *received.peek() {
            received.set(count);
            open.set(true);
        }
    });
    if !open() {
        return rsx! {};
    }
    rsx! {
        section { class: "absolute top-20 right-4 z-20 flex max-h-[calc(100%-96px)] w-80 max-w-[calc(100%-32px)] flex-col overflow-hidden rounded border border-[#48494e] bg-rd-surface text-sm text-white shadow-lg", aria_label: "Chat",
            header { class: "flex h-10 shrink-0 items-center justify-between border-b border-[#48494e] px-3",
                span { "Chat" }
                button { class: "flex size-6 items-center justify-center hover:bg-[#414348]", r#type: "button", title: "Close chat", aria_label: "Close chat", onclick: move |_| open.set(false), X { class: "size-4" } }
            }
            div { class: "flex min-h-20 flex-1 flex-col gap-2 overflow-auto p-3", role: "log", aria_live: "polite",
                if status.remote.chat.is_empty() { p { class: "text-xs text-rd-muted", "No messages" } }
                for message in status.remote.chat.iter() {
                    p { class: if message.outgoing { "ml-6 self-end rounded bg-[#2864a8] px-3 py-2 text-xs whitespace-pre-wrap [overflow-wrap:anywhere]" } else { "mr-6 self-start rounded bg-[#383a40] px-3 py-2 text-xs whitespace-pre-wrap [overflow-wrap:anywhere]" }, "{message.text}" }
                }
            }
            form { class: "flex shrink-0 gap-2 border-t border-[#48494e] p-2",
                onsubmit: move |event| {
                    event.prevent_default();
                    let value = text();
                    if value.trim().is_empty() { return; }
                    if let Some(active) = state.session.peek().as_ref() {
                        active.send_chat(value.clone());
                        update_ui_state(state.ui_state, |status| Arc::make_mut(&mut Arc::make_mut(&mut status.remote).chat).push(ChatMessage { outgoing: true, text: value }));
                        text.set(String::new());
                    }
                },
                input { class: "h-8 min-w-0 flex-1 rounded border border-[#55575b] bg-rd-content px-2 text-xs", r#type: "text", placeholder: "Write a message", aria_label: "Chat message", value: "{text}", oninput: move |event| text.set(event.value()) }
                button { class: "h-8 rounded bg-[#0071ff] px-3 text-xs text-white disabled:opacity-40", r#type: "submit", disabled: text().trim().is_empty(), "Send" }
            }
        }
    }
}

#[component]
pub(super) fn QualityMonitor(revision: u32) -> Element {
    let state = use_context::<super::super::super::state::AppState>();
    if !state
        .session
        .peek()
        .as_ref()
        .is_some_and(|active| active.get_toggle_option("show-quality-monitor".to_owned()))
    {
        return rsx! {};
    }
    let status = state.ui_state.read();
    rsx! { QualityDetails { remote: status.remote.clone(), display: status.current_display, width: status.width, height: status.height } }
}

#[component]
fn QualityDetails(
    remote: Arc<super::super::super::session::RemoteUiState>,
    display: i32,
    width: i32,
    height: i32,
) -> Element {
    let fps = remote
        .fps
        .get(&(display.max(0) as usize))
        .map(i32::to_string)
        .unwrap_or_else(|| "—".to_owned());
    let delay = remote
        .delay
        .map_or_else(|| "—".to_owned(), |value| format!("{value} ms"));
    let bitrate = remote
        .bitrate
        .map_or_else(|| "—".to_owned(), |value| format!("{value} kb/s"));
    rsx! {
        aside { class: "pointer-events-none absolute bottom-3 left-3 z-10 rounded bg-black/70 px-3 py-2 text-xs text-white", aria_label: "Quality monitor",
            p { "{remote.connection}" }
            p { "{width} × {height} · {fps} FPS · {remote.codec} {remote.chroma}" }
            p { "Speed: {remote.speed} · Delay: {delay}" }
            p { "Target bitrate: {bitrate}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_monitor_shows_current_display_and_connection_statistics() {
        fn fixture() -> Element {
            let remote = Arc::new(super::super::super::super::session::RemoteUiState {
                connection: "Secure · Direct · TCP".to_owned(),
                fps: [(0, 15), (1, 60)].into_iter().collect(),
                speed: "2 MB/s".to_owned(),
                delay: Some(12),
                bitrate: Some(8000),
                codec: "H264".to_owned(),
                ..Default::default()
            });
            rsx! { QualityDetails { remote, display: 1, width: 1920, height: 1080 } }
        }
        let mut dom = VirtualDom::new(fixture);
        let mutations = dom.rebuild_to_vec();
        let text = mutations
            .edits
            .iter()
            .filter_map(|edit| match edit {
                dioxus::core::Mutation::CreateTextNode { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Secure · Direct · TCP"), "{text}");
        assert!(text.contains("1920 × 1080 · 60 FPS · H264"), "{text}");
        assert!(text.contains("Speed: 2 MB/s · Delay: 12 ms"), "{text}");
        assert!(text.contains("Target bitrate: 8000 kb/s"), "{text}");
    }
}
