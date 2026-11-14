#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use librustdesk::*;

#[cfg(all(
    feature = "dioxus-ui",
    not(any(target_os = "android", target_os = "ios"))
))]
fn main() -> std::process::ExitCode {
    #[cfg(windows)]
    if let Some(exit_code) = platform::handle_service_control() {
        return exit_code;
    }
    #[cfg(windows)]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(2);
    }
    let exit_code = match crate::core_main::core_main() {
        Some(crate::core_main::CoreMainAction::StartUi(args)) => {
            dioxus_ui::start(args);
            std::process::ExitCode::SUCCESS
        }
        Some(crate::core_main::CoreMainAction::ExitFailure(err)) => {
            eprintln!("Command failed: {err}");
            hbb_common::log::error!("Command failed: {err}");
            std::process::ExitCode::FAILURE
        }
        None => std::process::ExitCode::SUCCESS,
    };
    common::global_clean();
    exit_code
}

#[cfg(all(feature = "dioxus-ui", target_os = "android"))]
fn main() {
    if !common::global_init() {
        eprintln!("Global initialization failed.");
        return;
    }
    common::test_rendezvous_server();
    common::test_nat_type();
    dioxus_ui::start(Vec::new());
    common::global_clean();
}
