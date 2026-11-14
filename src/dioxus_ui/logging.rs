use hbb_common::{env_logger, log};

// Match the shared debug defaults while keeping native GPU loader diagnostics opt-in.
const DEFAULT_FILTER: &str = "info,reqwest=warn,hyper_util=warn,rustls=warn,webrtc-sctp=warn,webrtc=warn,webrtc_ice::agent::agent_internal=error,webrtc::peer_connection=error,smithay_client_toolkit=warn,wgpu_hal=warn";

pub(crate) fn init_debug_log(_name: &str) {
    let env = env_logger::Env::default().default_filter_or(DEFAULT_FILTER);
    if let Err(error) = env_logger::Builder::from_env(env).try_init() {
        log::warn!("Cannot initialize native debug logging: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_logging_hides_gpu_info_and_keeps_diagnostics() {
        let mut builder = env_logger::Builder::new();
        let logger = builder.parse_filters(DEFAULT_FILTER).build();
        for (target, level, visible) in [
            ("wgpu_hal::vulkan::instance", log::Level::Info, false),
            ("wgpu_hal::gles::egl", log::Level::Info, false),
            ("wgpu_hal::vulkan::instance", log::Level::Warn, true),
            ("wgpu_hal::vulkan::instance", log::Level::Error, true),
            ("librustdesk::client", log::Level::Info, true),
            ("scrap::common::av1_decoder", log::Level::Info, true),
        ] {
            assert_eq!(
                logger.matches(&log::Record::builder().target(target).level(level).build()),
                visible,
                "{target} {level}"
            );
        }
        let logger = env_logger::Builder::new()
            .parse_filters(DEFAULT_FILTER)
            .parse_filters("wgpu_hal=info")
            .build();
        assert!(logger.matches(
            &log::Record::builder()
                .target("wgpu_hal::vulkan::instance")
                .level(log::Level::Info)
                .build()
        ));
    }
}
