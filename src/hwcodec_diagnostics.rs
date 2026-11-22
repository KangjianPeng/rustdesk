use serde_json::{json, Value};

mod software;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod workload;

const DIAGNOSTICS_KEY: &str = "hwcodec-diagnostics";
const BENCHMARK_KEY: &str = "hwcodec-benchmark";

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
    } else if lower.contains("nvenc")
        || lower.contains("cuda")
        || hwdevice == hwcodec::ffmpeg::AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA
    {
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

    if !HwCodecConfig::already_set() {
        if let Err(error) = crate::ipc::get_hwcodec_config_from_server() {
            hbb_common::log::trace!("Cannot load codec backends from the service: {error:#}");
        }
    }
    static DETECT_ONCE: Once = Once::new();
    if HwCodecConfig::get().ram_encode.is_empty() && HwCodecConfig::get().ram_decode.is_empty() {
        DETECT_ONCE.call_once(|| {
            let detected = scrap::hwcodec::check_available_hwcodec();
            match serde_json::from_str::<HwCodecConfig>(&detected) {
                Ok(_) => HwCodecConfig::set(detected),
                Err(error) => hbb_common::log::error!("Invalid codec detection result: {error}"),
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
    for info in config
        .ram_decode
        .iter()
        .filter(|info| is_hardware_decoder(info))
    {
        backends.push(backend_entry(info, "decode", is_hardware_decoder(info)));
    }
    for info in [software.h264.as_ref(), software.h265.as_ref()]
        .into_iter()
        .flatten()
    {
        backends.push(backend_entry(info, "decode", false));
    }
    backends.extend(software::backends());

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
        "detected": backends.iter().any(|backend| backend["hardware"] == true),
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
        "backends": software::backends(),
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
    use hwcodec::common::DataFormat;
    use scrap::hwcodec::HwCodecConfig;
    use scrap::{
        codec::{EncoderApi, EncoderCfg, BR_BALANCED},
        hwcodec::{HwRamDecoder, HwRamEncoder, HwRamEncoderConfig},
        ImageFormat, ImageRgb,
    };
    use std::time::{Duration, Instant};

    use self::workload::{convert_bgra_to_yuv, DesktopFrames, HEIGHT, ITERATIONS, WIDTH};

    detect();
    let config = HwCodecConfig::get();
    let software = hwcodec::ffmpeg_ram::CodecInfo::soft();

    let mut results = Vec::new();
    let mut encoded_streams = Vec::<(DataFormat, Vec<Vec<u8>>)>::new();

    for info in &config.ram_encode {
        if !matches!(info.format, DataFormat::H264 | DataFormat::H265) {
            continue;
        }
        let mut result = backend_entry(info, "encode", is_hardware_encoder(&info.name));
        let config = HwRamEncoderConfig {
            name: info.name.clone(),
            mc_name: info.mc_name.clone(),
            width: WIDTH,
            height: HEIGHT,
            quality: BR_BALANCED,
            keyframe_interval: None,
        };
        match HwRamEncoder::new(EncoderCfg::HWRAM(config), false) {
            Ok(mut encoder) => {
                let yuv_format = encoder.yuvfmt();
                let mut yuv = Vec::new();
                let mut desktop = DesktopFrames::new(WIDTH, HEIGHT);
                let mut elapsed = Duration::ZERO;
                let mut frames = 0usize;
                let mut bytes = 0usize;
                let mut error = None;
                let mut stream = Vec::new();
                let save_stream = !encoded_streams
                    .iter()
                    .any(|(format, _)| *format == info.format);
                for index in 0..ITERATIONS {
                    let bgra = desktop.next_frame(index);
                    let frame_started = Instant::now();
                    if let Err(err) = convert_bgra_to_yuv(bgra, &yuv_format, &mut yuv) {
                        error = Some(err.to_string());
                        break;
                    }
                    let encoded = match encoder.encode(&yuv, (index * 1000 / workload::FPS) as i64)
                    {
                        Ok(encoded) => encoded,
                        Err(code) => {
                            error = Some(format!("FFmpeg error {code}"));
                            break;
                        }
                    };
                    elapsed += frame_started.elapsed();
                    for frame in encoded.iter() {
                        frames += 1;
                        bytes += frame.data.len();
                        if save_stream {
                            stream.push(frame.data.clone());
                        }
                    }
                }
                let elapsed = elapsed.as_secs_f64();
                result["frames"] = json!(frames);
                result["bytes"] = json!(bytes);
                result["fps"] = json!(frames as f64 / elapsed.max(f64::EPSILON));
                result["duration_ms"] = json!((elapsed * 1000.0).round());
                result["pipeline"] = json!("BGRA to YUV conversion + encode");
                if error.is_none() && !stream.is_empty() {
                    encoded_streams.push((info.format, stream));
                }
                result["error"] = error.map_or(Value::Null, Value::String);
            }
            Err(error) => result["error"] = json!(error.to_string()),
        }
        results.push(result);
    }

    for info in config
        .ram_decode
        .iter()
        .filter(|info| is_hardware_decoder(info))
        .chain(
            [software.h264.as_ref(), software.h265.as_ref()]
                .into_iter()
                .flatten(),
        )
    {
        let stream = encoded_streams
            .iter()
            .find(|(format, _)| *format == info.format);
        let (packets, samples, source) = match stream {
            Some((_, packets)) => (
                packets.iter().map(Vec::as_slice).collect::<Vec<_>>(),
                packets.len(),
                "dynamic desktop stream",
            ),
            None => {
                let data: &[u8] = match info.format {
                    DataFormat::H264 => {
                        include_bytes!("../third_party/hwcodec-vendor/src/res/1080p.h264")
                    }
                    DataFormat::H265 => {
                        include_bytes!("../third_party/hwcodec-vendor/src/res/1080p.h265")
                    }
                    _ => continue,
                };
                (vec![data], ITERATIONS, "embedded sample repeated")
            }
        };
        let mut result = backend_entry(info, "decode", is_hardware_decoder(info));
        match HwRamDecoder::new_with_info(info.clone()) {
            Ok(mut decoder) => {
                let mut rgb = ImageRgb::new(ImageFormat::ABGR, 1);
                let mut i420 = Vec::new();
                let start = Instant::now();
                let mut frames = 0usize;
                let mut error = None;
                for data in packets.into_iter().cycle().take(samples) {
                    match decoder.decode_without_fallback(data) {
                        Ok(decoded) => {
                            for image in decoded {
                                if let Err(err) = image.to_fmt(&mut rgb, &mut i420) {
                                    error = Some(err.to_string());
                                    break;
                                }
                                frames += 1;
                            }
                        }
                        Err(err) => {
                            error = Some(err.to_string());
                            break;
                        }
                    }
                    if error.is_some() {
                        break;
                    }
                }
                let elapsed = start.elapsed().as_secs_f64();
                result["frames"] = json!(frames);
                result["fps"] = json!(frames as f64 / elapsed.max(f64::EPSILON));
                result["duration_ms"] = json!((elapsed * 1000.0).round());
                result["error"] = error.map_or(Value::Null, Value::String);
                result["pipeline"] = json!("decode + YUV to ABGR conversion");
                result["test_stream"] = json!(source);
            }
            Err(error) => result["error"] = json!(error.to_string()),
        }
        results.push(result);
    }
    results.extend(software::benchmark());

    json!({
        "ok": true,
        "iterations": ITERATIONS,
        "width": WIDTH,
        "height": HEIGHT,
        "target_fps": workload::FPS,
        "source": "changing desktop-like BGRA frames",
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
        assert_eq!(
            backend_name("h264", AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA),
            "NVIDIA NVENC/NVDEC"
        );
    }

    #[test]
    fn uses_software_decoder_when_hardware_is_unavailable() {
        let software = hwcodec::ffmpeg_ram::CodecInfo::soft();
        let selected = active_decoder(true, None, software.h265.as_ref());

        assert_eq!(selected.map(|info| info.name.as_str()), Some("hevc"));
    }
}

#[cfg(all(
    not(feature = "hwcodec"),
    not(any(target_os = "android", target_os = "ios"))
))]
fn benchmark_json() -> String {
    json!({
        "ok": true,
        "iterations": workload::ITERATIONS,
        "width": workload::WIDTH,
        "height": workload::HEIGHT,
        "target_fps": workload::FPS,
        "source": "changing desktop-like BGRA frames",
        "results": software::benchmark(),
    })
    .to_string()
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn benchmark_json() -> String {
    json!({
        "ok": false,
        "error": "Hardware codec benchmarking is unavailable on this build",
        "results": [],
    })
    .to_string()
}
