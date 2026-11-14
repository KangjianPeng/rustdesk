use super::prelude::*;

mod diagnostics;
mod fields;
mod pages;

pub(crate) use diagnostics::CodecDiagnostics;
pub(crate) use fields::{SettingChoice, SettingToggle};
pub(crate) use pages::SettingsContent;

pub(super) async fn codec_json(key: &'static str) -> String {
    match hbb_common::tokio::task::spawn_blocking(move || {
        if key == "hwcodec-diagnostics" {
            crate::hwcodec_diagnostics::detect();
        }
        crate::hwcodec_diagnostics::get(key)
    })
    .await
    {
        Ok(diagnostics) => diagnostics,
        Err(error) => {
            log::error!("Failed to load codec diagnostics: {error}");
            serde_json::json!({ "ok": false, "error": error.to_string() }).to_string()
        }
    }
}

pub(super) fn supported_hwdecodings() -> (bool, bool) {
    crate::ui_interface::supported_hwdecodings()
}

pub(super) fn rustdesk_icon_data_uri() -> String {
    format!(
        "data:image/svg+xml;base64,{}",
        STANDARD.encode(include_bytes!("../../../res/dioxus/icon.svg"))
    )
}
