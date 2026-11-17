use super::super::prelude::*;

#[component]
pub(crate) fn CodecDiagnostics(state: SyncSignal<CodecUiState>) -> Element {
    let snapshot = state.read().clone();
    let diagnostics = serde_json::from_str::<serde_json::Value>(&snapshot.diagnostics)
        .unwrap_or(serde_json::Value::Null);
    let benchmark = serde_json::from_str::<serde_json::Value>(&snapshot.benchmark)
        .unwrap_or(serde_json::Value::Null);
    let enabled = diagnostics
        .get("enabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let detected = diagnostics
        .get("detected")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let active = diagnostics.get("active");
    let path = |operation: &str, codec: &str| {
        let entry = active
            .and_then(|value| value.get(operation))
            .and_then(|value| value.get(codec));
        entry
            .and_then(|value| value.get("hardware"))
            .and_then(serde_json::Value::as_bool)
            .map_or_else(
                || tr("Unavailable"),
                |hardware| {
                    let mode = tr(if hardware { "Hardware" } else { "Software" });
                    let backend = entry
                        .and_then(|value| value.get("backend"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("Unknown backend");
                    format!("{mode} · {backend}")
                },
            )
    };
    let backends = diagnostics
        .get("backends")
        .and_then(serde_json::Value::as_array);
    let software_codecs = diagnostics
        .get("software_codecs")
        .and_then(serde_json::Value::as_array)
        .map(|codecs| {
            codecs
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let benchmark_status = benchmark
        .get("ok")
        .and_then(serde_json::Value::as_bool)
        .map_or("Not tested", |ok| if ok { "Complete" } else { "Failed" });
    let acceleration_status = if enabled { "enabled" } else { "disabled" };

    rsx! {
        div { class: "mt-5 max-w-[650px] border-t border-[#3b3d41] pt-[15px] [&_p]:text-xs [&_p]:text-rd-muted",
            p { class: "mt-2 mb-0 [overflow-wrap:anywhere] text-center text-[11px] text-rd-muted",
                if snapshot.loading { {tr("Detecting codec backends...")} }
                else if detected { {tr("Codec backends detected.")} }
                else { {tr("No hardware codec backend detected.")} }
            }
            div { class: "mt-[13px] grid grid-cols-2 gap-2 text-xs text-[#d4d5d7]",
                div { class: "rounded border border-[#414348] bg-rd-panel p-2.5", "H264 {tr(\"encode\")}: {path(\"encode\", \"H264\")}" }
                div { class: "rounded border border-[#414348] bg-rd-panel p-2.5", "H264 {tr(\"decode\")}: {path(\"decode\", \"H264\")}" }
                div { class: "rounded border border-[#414348] bg-rd-panel p-2.5", "H265 {tr(\"encode\")}: {path(\"encode\", \"H265\")}" }
                div { class: "rounded border border-[#414348] bg-rd-panel p-2.5", "H265 {tr(\"decode\")}: {path(\"decode\", \"H265\")}" }
            }
            p { class: "mt-2 mb-0 [overflow-wrap:anywhere] text-center text-[11px] text-rd-muted", "{tr(\"Hardware acceleration\")}: {tr(acceleration_status)} · {tr(\"Performance test\")}: {tr(benchmark_status)}" }
            if let Some(error) = diagnostics.get("error").and_then(serde_json::Value::as_str) {
                p { class: "mt-2 [overflow-wrap:anywhere] !text-[#ee8c94]", "{error}" }
            }
            if let Some(backends) = backends {
                for backend in backends {
                    p { class: "mt-2 [overflow-wrap:anywhere]",
                        "{backend[\"codec\"].as_str().unwrap_or_default()} {backend[\"operation\"].as_str().unwrap_or_default()}: {backend[\"backend\"].as_str().unwrap_or_default()} ({backend[\"name\"].as_str().unwrap_or_default()})"
                    }
                }
            }
            p { class: "mt-2 [overflow-wrap:anywhere]", "{tr(\"Software codecs\")}: {software_codecs}" }
            if snapshot.benchmark.len() > 0 && !snapshot.benchmark.contains("\"ok\":true") {
                p { class: "!text-[#ee8c94]", {tr("Codec benchmark did not complete successfully.")} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_settings_show_backend_names_and_software_codecs() {
        fn fixture() -> Element {
            let state = use_signal_sync(|| CodecUiState {
                diagnostics: Arc::from(
                    r#"{
                    "enabled": true, "detected": true,
                    "active": {"decode": {"H264": {"hardware": true, "backend": "VA-API"}}},
                    "software_codecs": ["VP8", "VP9", "AV1"]
                }"#,
                ),
                ..Default::default()
            });
            rsx! { CodecDiagnostics { state } }
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
        assert!(text.contains("H264 decode: Hardware · VA-API"), "{text}");
        assert!(text.contains("Software codecs: VP8, VP9, AV1"), "{text}");
    }
}
