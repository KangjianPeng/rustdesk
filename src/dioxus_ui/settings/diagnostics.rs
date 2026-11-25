use super::super::prelude::*;
use dioxus::core::Runtime;
use registry::ui::{
    badge::{Badge, BadgeVariant},
    table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow, TableWrapper},
};
use serde_json::{json, Value};

fn codec_rows(diagnostics: &Value, benchmark: &Value, operation: &str) -> Vec<Value> {
    let backends = diagnostics["backends"].as_array();
    let results = benchmark["results"].as_array();
    let mut rows = Vec::new();
    for codec in ["H264", "H265", "VP8", "VP9", "AV1"] {
        let mut entries: Vec<_> = backends
            .into_iter()
            .flatten()
            .filter(|entry| entry["codec"] == codec && entry["operation"] == operation)
            .cloned()
            .collect();
        if entries.is_empty() {
            entries.push(json!({ "codec": codec, "operation": operation, "available": false }));
        }
        for mut entry in entries {
            if let Some(result) = results.into_iter().flatten().find(|result| {
                ["codec", "operation", "name", "backend", "hardware"]
                    .iter()
                    .all(|key| result[*key] == entry[*key])
            }) {
                entry["performance"] = result.clone();
            }
            rows.push(entry);
        }
    }
    rows
}

#[component]
pub(crate) fn CodecDiagnostics(state: SyncSignal<CodecUiState>) -> Element {
    let snapshot = state.read().clone();
    let diagnostics = serde_json::from_str::<Value>(&snapshot.diagnostics).unwrap_or(Value::Null);
    let benchmark = serde_json::from_str::<Value>(&snapshot.benchmark).unwrap_or(Value::Null);
    let acceleration = if diagnostics["enabled"] == true {
        "enabled"
    } else {
        "disabled"
    };
    let busy = snapshot.loading || snapshot.benchmarking;
    rsx! {
        Card { css_class: "mt-4 w-full min-w-0 max-w-[960px] px-4",
            div { class: "codec-diagnostics-heading flex items-center justify-between gap-3",
                h2 { class: "min-w-0 font-semibold", {tr("Codec diagnostics")} }
                div { class: "flex shrink-0 items-center gap-2",
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::IconSm,
                        title: "Detect backends", aria_label: "Detect backends", disabled: busy,
                        onclick: move |_| {
                            if state.peek().loading || state.peek().benchmarking { return; }
                            update_signal(state, |state| { state.loading = true; state.benchmark = Arc::from(""); });
                            Runtime::current().spawn(state.origin_scope(), async move {
                                let diagnostics = super::codec_json("hwcodec-diagnostics").await;
                                update_signal(state, |state| { state.diagnostics = Arc::from(diagnostics); state.loading = false; });
                            });
                        }, RefreshCw { class: "size-4" }
                    }
                    Button { size: ButtonSize::Sm, disabled: busy, onclick: move |_| {
                        if state.peek().loading || state.peek().benchmarking { return; }
                        update_signal(state, |state| state.benchmarking = true);
                        Runtime::current().spawn(state.origin_scope(), async move {
                            let benchmark = super::codec_json("hwcodec-benchmark").await;
                            let diagnostics = super::codec_json("hwcodec-diagnostics").await;
                            update_signal(state, |state| {
                                state.diagnostics = Arc::from(diagnostics);
                                state.benchmark = Arc::from(benchmark);
                                state.benchmarking = false;
                            });
                        });
                    }, if snapshot.benchmarking { {tr("Testing...")} } else { {tr("Test performance")} } }
                }
            }
            p { class: "text-xs text-muted-foreground",
                "{tr(\"Platform\")}: {std::env::consts::OS} · {tr(\"Hardware acceleration\")}: {tr(acceleration)}"
            }
            if snapshot.loading { p { role: "status", class: "text-sm text-muted-foreground", {tr("Detecting codec backends...")} } }
            for (operation, title) in [("encode", "Encoders"), ("decode", "Decoders")] {
                section { class: "min-w-0 space-y-3", aria_label: tr(title),
                    h3 { class: "text-sm font-medium", {tr(title)} }
                    TableWrapper { class: "mobile-codec-table max-h-none",
                        Table {
                            class: "min-w-[600px]",
                            TableHeader { TableRow {
                                for label in ["Codec format", "Support", "Backend", "Performance"] {
                                    TableHead { class: "px-3", {tr(label)} }
                                }
                            } }
                            TableBody {
                                for entry in codec_rows(&diagnostics, &benchmark, operation) {
                                    CodecBackend { entry, testing: snapshot.benchmarking, loading: snapshot.loading }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(error) = diagnostics["error"].as_str() {
                p { role: "alert", class: "text-xs text-destructive [overflow-wrap:anywhere]", "{error}" }
            }
            if let Some(error) = benchmark["error"].as_str() {
                p { role: "alert", class: "text-xs text-destructive [overflow-wrap:anywhere]", "{error}" }
            }
            if let (Some(width), Some(height), Some(iterations)) = (benchmark["width"].as_u64(), benchmark["height"].as_u64(), benchmark["iterations"].as_u64()) {
                p { class: "text-xs text-muted-foreground", "{width} × {height} · {iterations} {tr(\"test frames\")}" }
            }
        }
    }
}

#[component]
fn CodecBackend(entry: Value, testing: bool, loading: bool) -> Element {
    let available = entry["available"] == true;
    let name = entry["name"].as_str().unwrap_or("—");
    let backend = entry["backend"].as_str().unwrap_or("—");
    let (support, badge_variant, badge_class) = if loading && !available {
        ("Detecting...", BadgeVariant::Muted, "")
    } else if !available {
        (
            "Unsupported",
            BadgeVariant::Outline,
            "border-destructive/30 bg-destructive/10 text-destructive",
        )
    } else if entry["hardware"] == true {
        ("Hardware", BadgeVariant::Success, "")
    } else {
        ("Software", BadgeVariant::Info, "")
    };
    let result = &entry["performance"];
    let error = result["error"].as_str();
    let metrics = result["fps"]
        .as_f64()
        .filter(|fps| fps.is_finite() && *fps > 0.0)
        .zip(result["frames"].as_u64().filter(|frames| *frames > 0))
        .zip(result["duration_ms"].as_f64());
    let peak_throughput = tr("Peak throughput");
    let performance = if !available {
        "—".to_owned()
    } else if testing {
        tr("Testing...")
    } else if error.is_some() {
        tr("Failed")
    } else if let Some(((fps, frames), duration)) = metrics {
        format!(
            "{peak_throughput}: {fps:.1} FPS · {:.2} {}",
            duration / frames as f64,
            tr("ms/frame")
        )
    } else if !result.is_null() {
        tr("No frames produced")
    } else {
        tr("Not tested")
    };
    rsx! {
        TableRow {
            TableCell { class: "p-3 font-medium",
                span { class: "mobile-codec-label", {tr("Codec format")} }
                span { "{entry[\"codec\"].as_str().unwrap_or_default()}" }
            }
            TableCell { class: "p-3",
                span { class: "mobile-codec-label", {tr("Support")} }
                Badge { variant: badge_variant, class: "whitespace-nowrap {badge_class}", {tr(support)} }
            }
            TableCell { class: "p-3",
                span { class: "mobile-codec-label", {tr("Backend")} }
                div { class: "flex min-w-0 flex-col gap-1",
                    span { "{backend}" }
                    if available && name != backend {
                        span { class: "text-xs text-muted-foreground", "{name}" }
                    }
                }
            }
            TableCell { class: "w-[220px] p-3 tabular-nums",
                span { class: "mobile-codec-label", {tr("Performance")} }
                div { class: "flex min-w-0 flex-col gap-1",
                    span { class: "whitespace-nowrap", "{performance}" }
                    if !testing {
                        if let Some(error) = error {
                            span { class: "max-w-[220px] text-xs text-destructive [overflow-wrap:anywhere]", {tr(error)} }
                        } else if result["test_stream"] == "embedded sample repeated" {
                            span { class: "text-xs text-muted-foreground", {tr("Repeated test clip")} }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_encoders_decoders_and_matches_performance_to_each_backend() {
        let diagnostics = json!({"backends": [
            {"codec":"H264", "operation":"encode", "name":"h264_vaapi", "backend":"VA-API", "hardware":true, "available":true},
            {"codec":"H264", "operation":"decode", "name":"h264", "backend":"VA-API", "hardware":true, "available":true},
            {"codec":"H264", "operation":"decode", "name":"h264", "backend":"NVIDIA NVENC/NVDEC", "hardware":true, "available":true},
            {"codec":"VP8", "operation":"encode", "name":"libvpx", "backend":"libvpx", "hardware":false, "available":true}
        ]});
        let benchmark = json!({"ok":true, "results": [
            {"codec":"H264", "operation":"decode", "name":"h264", "backend":"VA-API", "hardware":true, "frames":24, "fps":120.0, "duration_ms":200.0},
            {"codec":"VP8", "operation":"encode", "name":"libvpx", "backend":"libvpx", "hardware":false, "error":"Test encoder error"}
        ]});
        let encode = codec_rows(&diagnostics, &benchmark, "encode");
        let decode = codec_rows(&diagnostics, &benchmark, "decode");
        assert!(encode[0]["performance"].is_null());
        assert_eq!(decode[0]["performance"]["fps"], 120.0);
        assert!(decode[1]["performance"].is_null());
        assert_eq!(encode[1]["codec"], "H265");
        assert_eq!(encode[1]["available"], false);
        assert_eq!(encode[2]["performance"]["error"], "Test encoder error");
    }

    #[test]
    fn renders_support_and_performance_in_the_lists() {
        fn fixture() -> Element {
            let state = use_signal_sync(|| CodecUiState {
                diagnostics: Arc::from(
                    r#"{"enabled":true,"backends":[{"codec":"H264","operation":"decode","name":"h264","backend":"VA-API","hardware":true,"available":true},{"codec":"VP8","operation":"decode","name":"libvpx","backend":"libvpx","hardware":false,"available":true}]}"#,
                ),
                benchmark: Arc::from(
                    r#"{"ok": true,"results":[{"codec":"H264","operation":"decode","name":"h264","backend":"VA-API","hardware":true,"frames":24,"fps":120.0,"duration_ms":200.0}]}"#,
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
        for expected in [
            "Encoders",
            "Decoders",
            "VA-API",
            "Hardware",
            "Software",
            "Unsupported",
            "Peak throughput: 120.0 FPS · 8.33 ms/frame",
        ] {
            assert!(text.contains(expected), "Missing {expected}: {text}");
        }
        assert!(text.find("Support").unwrap() < text.find("Backend").unwrap());
    }
}
