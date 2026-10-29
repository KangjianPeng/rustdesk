use serde_json::{json, Value};

const DIAGNOSTICS_KEY: &str = "hwcodec-diagnostics";
const BENCHMARK_KEY: &str = "hwcodec-benchmark";

pub fn is_diagnostics_key(key: &str) -> bool {
    matches!(key, DIAGNOSTICS_KEY | BENCHMARK_KEY)
}

pub fn get(key: &str) -> String {
    match key {
        DIAGNOSTICS_KEY => diagnostics_json(),
        BENCHMARK_KEY => benchmark_json(),
        _ => String::new(),
    }
}

#[cfg(feature = "hwcodec")]
fn codec_name(format: hwcodec::common::DataFormat) -> &'static str {
    match format {
        hwcodec::common::DataFormat::H264 => "H264",
        hwcodec::common::DataFormat::H265 => "H265",
        hwcodec::common::DataFormat::VP8 => "VP8",
        hwcodec::common::DataFormat::VP9 => "VP9",
        hwcodec::common::DataFormat::AV1 => "AV1",
    }
}

#[cfg(feature = "hwcodec")]
fn is_hardware_encoder(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    !(name == "h264"
        || name == "hevc"
        || name.contains("libx264")
        || name.contains("libx265")
        || name.contains("x264")
        || name.contains("x265"))
}

#[cfg(feature = "hwcodec")]
fn is_hardware_decoder(info: &hwcodec::ffmpeg_ram::CodecInfo) -> bool {
    info.hwdevice != hwcodec::ffmpeg::AVHWDeviceType::AV_HWDEVICE_TYPE_NONE
        || info.name.to_ascii_lowercase().contains("mediacodec")
}

#[cfg(feature = "hwcodec")]
fn backend_name(name: &str, hwdevice: hwcodec::ffmpeg::AVHWDeviceType) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.contains("vaapi")
        || hwdevice == hwcodec::ffmpeg::AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI
    {
        "VA-API"
    } else if lower.contains("nvenc") || lower.contains("cuda") {
        "NVIDIA NVENC/NVDEC"
    } else if lower.contains("qsv")
        || hwdevice == hwcodec::ffmpeg::AVHWDeviceType::AV_HWDEVICE_TYPE_QSV
    {
        "Intel Quick Sync"
    } else if lower.contains("amf") {
        "AMD AMF"
    } else if lower.contains("videotoolbox")
        || hwdevice == hwcodec::ffmpeg::AVHWDeviceType::AV_HWDEVICE_TYPE_VIDEOTOOLBOX
    {
        "Apple VideoToolbox"
    } else if lower.contains("mediacodec") {
        "Android MediaCodec"
    } else {
        "FFmpeg software"
    }
}

#[cfg(feature = "hwcodec")]
fn backend_entry(
    info: &hwcodec::ffmpeg_ram::CodecInfo,
    operation: &'static str,
    hardware: bool,
) -> Value {
    json!({
        "codec": codec_name(info.format),
        "operation": operation,
        "name": info.name,
        "backend": backend_name(&info.name, info.hwdevice),
        "hardware": hardware,
        "available": true,
    })
}

#[cfg(all(
    feature = "hwcodec",
    not(any(target_os = "android", target_os = "ios"))
))]
pub fn detect() {
    use std::sync::Once;

    use scrap::hwcodec::HwCodecConfig;

    static DETECT_ONCE: Once = Once::new();
    if HwCodecConfig::get().ram_encode.is_empty() && HwCodecConfig::get().ram_decode.is_empty() {
        DETECT_ONCE.call_once(|| {
            let detected = scrap::hwcodec::check_available_hwcodec();
            if serde_json::from_str::<HwCodecConfig>(&detected).is_ok() {
                HwCodecConfig::set(detected);
            }
        });
    }
}

#[cfg(any(not(feature = "hwcodec"), target_os = "android", target_os = "ios"))]
pub fn detect() {}

#[cfg(feature = "hwcodec")]
fn active_entry(
    info: Option<&hwcodec::ffmpeg_ram::CodecInfo>,
    operation: &'static str,
    hardware: bool,
) -> Value {
    info.map_or(Value::Null, |info| backend_entry(info, operation, hardware))
}

#[cfg(feature = "hwcodec")]
fn active_decoder<'a>(
    hardware_enabled: bool,
    hardware: Option<&'a hwcodec::ffmpeg_ram::CodecInfo>,
    software: Option<&'a hwcodec::ffmpeg_ram::CodecInfo>,
) -> Option<&'a hwcodec::ffmpeg_ram::CodecInfo> {
    if hardware_enabled {
        hardware.or(software)
    } else {
        software
    }
}

#[cfg(feature = "hwcodec")]
fn diagnostics_json() -> String {
    use scrap::codec::enable_hwcodec_option;

    let config = scrap::hwcodec::HwCodecConfig::get();
    let encoders = hwcodec::ffmpeg_ram::CodecInfo::prioritized(config.ram_encode.clone());
    let decoders = hwcodec::ffmpeg_ram::CodecInfo::prioritized(config.ram_decode.clone());
    let software = hwcodec::ffmpeg_ram::CodecInfo::soft();
    let mut backends = Vec::new();
    for info in &config.ram_encode {
        backends.push(backend_entry(
            info,
            "encode",
            is_hardware_encoder(&info.name),
        ));
    }
    for info in &config.ram_decode {
        backends.push(backend_entry(info, "decode", is_hardware_decoder(info)));
    }
    for info in [software.h264.as_ref(), software.h265.as_ref()]
        .into_iter()
        .flatten()
    {
        backends.push(backend_entry(info, "decode", false));
    }

    let active_encode_h264 = if enable_hwcodec_option() {
        encoders.h264.as_ref()
    } else {
        None
    };
    let active_encode_h265 = if enable_hwcodec_option() {
        encoders.h265.as_ref()
    } else {
        None
    };
    let active_decode_h264 = active_decoder(
        enable_hwcodec_option(),
        decoders.h264.as_ref(),
        software.h264.as_ref(),
    );
    let active_decode_h265 = active_decoder(
        enable_hwcodec_option(),
        decoders.h265.as_ref(),
        software.h265.as_ref(),
    );
    let active = json!({
        "encode": {
            "H264": active_entry(active_encode_h264, "encode", active_encode_h264.is_some_and(|i| is_hardware_encoder(&i.name))),
            "H265": active_entry(active_encode_h265, "encode", active_encode_h265.is_some_and(|i| is_hardware_encoder(&i.name))),
        },
        "decode": {
            "H264": active_entry(active_decode_h264, "decode", active_decode_h264.is_some_and(is_hardware_decoder)),
            "H265": active_entry(active_decode_h265, "decode", active_decode_h265.is_some_and(is_hardware_decoder)),
        },
    });

    json!({
        "enabled": enable_hwcodec_option(),
        "detected": !config.ram_encode.is_empty() || !config.ram_decode.is_empty(),
        "backends": backends,
        "active": active,
        "software_codecs": ["VP8", "VP9", "AV1"],
    })
    .to_string()
}

#[cfg(not(feature = "hwcodec"))]
fn diagnostics_json() -> String {
    json!({
        "enabled": false,
        "detected": false,
        "backends": [],
        "active": {"encode": {}, "decode": {}},
        "software_codecs": ["VP8", "VP9", "AV1"],
    })
    .to_string()
}

#[cfg(all(
    feature = "hwcodec",
    not(any(target_os = "android", target_os = "ios"))
))]
fn benchmark_json() -> String {
    use std::time::Instant;

    use hwcodec::{
        common::{DataFormat, Quality::Quality_Default, RateControl::RC_CBR},
        ffmpeg::AVPixelFormat,
        ffmpeg_ram::{
            decode::{DecodeContext, Decoder},
            encode::{EncodeContext, Encoder},
            ffmpeg_linesize_offset_length,
        },
    };
    use scrap::hwcodec::HwCodecConfig;

    const WIDTH: usize = 1280;
    const HEIGHT: usize = 720;
    const ITERATIONS: usize = 24;
    detect();
    let config = HwCodecConfig::get();
    let software = hwcodec::ffmpeg_ram::CodecInfo::soft();

    let mut results = Vec::new();
    let yuv = match ffmpeg_linesize_offset_length(
        AVPixelFormat::AV_PIX_FMT_NV12,
        WIDTH,
        HEIGHT,
        scrap::HW_STRIDE_ALIGN,
    ) {
        Ok((_, _, length)) if length > 0 => vec![0u8; length as usize],
        _ => {
            return json!({
                "ok": false,
                "error": "Unable to allocate the benchmark frame",
                "results": [],
            })
            .to_string()
        }
    };

    for info in &config.ram_encode {
        let context = EncodeContext {
            name: info.name.clone(),
            mc_name: info.mc_name.clone(),
            width: WIDTH as i32,
            height: HEIGHT as i32,
            pixfmt: AVPixelFormat::AV_PIX_FMT_NV12,
            align: scrap::HW_STRIDE_ALIGN as i32,
            fps: 30,
            gop: i32::MAX,
            rc: RC_CBR,
            quality: Quality_Default,
            kbs: 4_000,
            q: -1,
            thread_count: 4,
        };
        let mut result = backend_entry(info, "encode", is_hardware_encoder(&info.name));
        match Encoder::new(context) {
            Ok(mut encoder) => {
                let start = Instant::now();
                let mut frames = 0usize;
                let mut bytes = 0usize;
                let mut error = None;
                for index in 0..ITERATIONS {
                    match encoder.encode(&yuv, (index * 33) as i64) {
                        Ok(encoded) => {
                            frames += encoded.len();
                            bytes += encoded.iter().map(|frame| frame.data.len()).sum::<usize>();
                        }
                        Err(code) => {
                            error = Some(format!("FFmpeg error {code}"));
                            break;
                        }
                    }
                }
                let elapsed = start.elapsed().as_secs_f64();
                result["frames"] = json!(frames);
                result["bytes"] = json!(bytes);
                result["fps"] = json!(frames as f64 / elapsed.max(f64::EPSILON));
                result["duration_ms"] = json!((elapsed * 1000.0).round());
                result["error"] = error.map_or(Value::Null, Value::String);
            }
            Err(_) => result["error"] = json!("Unable to create encoder"),
        }
        results.push(result);
    }

    for info in config.ram_decode.iter().chain(
        [software.h264.as_ref(), software.h265.as_ref()]
            .into_iter()
            .flatten(),
    ) {
        let data: &[u8] = match info.format {
            DataFormat::H264 => include_bytes!("../third_party/hwcodec-vendor/src/res/720p.h264"),
            DataFormat::H265 => include_bytes!("../third_party/hwcodec-vendor/src/res/720p.h265"),
            _ => continue,
        };
        let context = DecodeContext {
            name: info.name.clone(),
            device_type: info.hwdevice,
            thread_count: 4,
        };
        let mut result = backend_entry(info, "decode", is_hardware_decoder(info));
        match Decoder::new(context) {
            Ok(mut decoder) => {
                let start = Instant::now();
                let mut frames = 0usize;
                let mut error = None;
                for _ in 0..ITERATIONS {
                    match decoder.decode(data) {
                        Ok(decoded) => frames += decoded.len(),
                        Err(code) => {
                            error = Some(format!("FFmpeg error {code}"));
                            break;
                        }
                    }
                }
                let elapsed = start.elapsed().as_secs_f64();
                result["frames"] = json!(frames);
                result["fps"] = json!(frames as f64 / elapsed.max(f64::EPSILON));
                result["duration_ms"] = json!((elapsed * 1000.0).round());
                result["error"] = error.map_or(Value::Null, Value::String);
            }
            Err(_) => result["error"] = json!("Unable to create decoder"),
        }
        results.push(result);
    }

    json!({
        "ok": true,
        "iterations": ITERATIONS,
        "width": WIDTH,
        "height": HEIGHT,
        "results": results,
    })
    .to_string()
}

#[cfg(all(test, feature = "hwcodec"))]
mod tests {
    use super::{active_decoder, backend_name, is_hardware_encoder};
    use hwcodec::ffmpeg::AVHWDeviceType;

    #[test]
    fn identifies_x265_as_software() {
        assert!(!is_hardware_encoder("libx265"));
        assert!(is_hardware_encoder("hevc_vaapi"));
    }

    #[test]
    fn reports_known_backend_names() {
        assert_eq!(
            backend_name("hevc_vaapi", AVHWDeviceType::AV_HWDEVICE_TYPE_NONE),
            "VA-API"
        );
        assert_eq!(
            backend_name("libx265", AVHWDeviceType::AV_HWDEVICE_TYPE_NONE),
            "FFmpeg software"
        );
    }

    #[test]
    fn uses_software_decoder_when_hardware_is_unavailable() {
        let software = hwcodec::ffmpeg_ram::CodecInfo::soft();
        let selected = active_decoder(true, None, software.h265.as_ref());

        assert_eq!(selected.map(|info| info.name.as_str()), Some("hevc"));
    }
}

#[cfg(any(not(feature = "hwcodec"), target_os = "android", target_os = "ios"))]
fn benchmark_json() -> String {
    json!({
        "ok": false,
        "error": "Hardware codec benchmarking is unavailable on this build",
        "results": [],
    })
    .to_string()
}
