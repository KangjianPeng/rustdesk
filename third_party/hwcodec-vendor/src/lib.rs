#[cfg(target_os = "android")]
pub mod android;
pub mod common;
pub mod ffmpeg;
pub mod ffmpeg_ram;
pub mod mux;
#[cfg(all(windows, feature = "vram"))]
pub mod vram;

#[no_mangle]
pub extern "C" fn hwcodec_log(level: i32, message: *const std::os::raw::c_char) {
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(message);
        if let Ok(str_slice) = c_str.to_str() {
            if ffmpeg::is_probing() {
                log::trace!("{}", str_slice.trim_end());
                return;
            }
            let string = String::from(str_slice);
            match level {
                0 => log::error!("{}", string),
                1 => log::warn!("{}", string),
                2 => log::info!("{}", string),
                3 => log::debug!("{}", string),
                4 => log::trace!("{}", string),
                _ => {}
            }
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use crate::{
        common::{Quality::Quality_Default, RateControl::RC_CBR},
        ffmpeg::{AVHWDeviceType::AV_HWDEVICE_TYPE_NONE, AVPixelFormat::AV_PIX_FMT_NV12},
        ffmpeg_ram::{
            decode::{DecodeContext, Decoder},
            encode::{EncodeContext, Encoder},
        },
    };
    use std::sync::Mutex;

    struct TestLogger(Mutex<Vec<(log::Level, String)>>);
    static LOGGER: TestLogger = TestLogger(Mutex::new(Vec::new()));

    impl log::Log for TestLogger {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            self.0
                .lock()
                .unwrap()
                .push((record.level(), record.args().to_string()));
        }

        fn flush(&self) {}
    }

    #[test]
    fn probing_skips_incompatible_encoders_and_preserves_runtime_errors() {
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Trace);
        let encoders = Encoder::available_encoders(
            EncodeContext {
                name: String::new(),
                mc_name: None,
                width: 1920,
                height: 1080,
                pixfmt: AV_PIX_FMT_NV12,
                align: 0,
                fps: 30,
                gop: i32::MAX,
                rc: RC_CBR,
                quality: Quality_Default,
                kbs: 1000,
                q: -1,
                thread_count: 1,
            },
            None,
        );
        let decoders = Decoder::available_decoders();
        assert!(encoders.iter().all(|codec| codec.name != "libx265"));
        assert!(decoders
            .iter()
            .any(|codec| codec.hwdevice == AV_HWDEVICE_TYPE_NONE));
        {
            let logs = LOGGER.0.lock().unwrap();
            assert!(
                logs.iter().all(|(level, _)| *level > log::Level::Warn),
                "{logs:?}"
            );
            assert!(
                logs.iter().all(|(_, text)| !text.contains("libx265")),
                "{logs:?}"
            );
            assert!(
                logs.iter()
                    .all(|(_, text)| !text.contains("hw_frames_ctx is NULL")),
                "{logs:?}"
            );
        }

        let invalid = || {
            Decoder::new(DecodeContext {
                name: "missing_test_decoder".to_owned(),
                device_type: AV_HWDEVICE_TYPE_NONE,
                thread_count: 1,
            })
            .is_err()
        };
        {
            let _probe = crate::ffmpeg::ProbeLogGuard::enter();
            assert!(std::thread::spawn(invalid).join().unwrap());
            crate::ffmpeg::hwcodec_av_log_callback(
                crate::ffmpeg::AV_LOG_ERROR as _,
                c"test_ffmpeg_probe_failure".as_ptr(),
            );
        }
        assert!(invalid());
        crate::ffmpeg::hwcodec_av_log_callback(
            crate::ffmpeg::AV_LOG_ERROR as _,
            c"test_ffmpeg_runtime_failure".as_ptr(),
        );
        assert_eq!(
            LOGGER
                .0
                .lock()
                .unwrap()
                .iter()
                .filter(|(level, text)| {
                    *level == log::Level::Error && text.contains("missing_test_decoder")
                })
                .count(),
            2
        );
        let logs = LOGGER.0.lock().unwrap();
        assert!(logs.contains(&(log::Level::Trace, "test_ffmpeg_probe_failure".to_owned())));
        assert!(logs.contains(&(log::Level::Error, "test_ffmpeg_runtime_failure".to_owned())));
    }
}
