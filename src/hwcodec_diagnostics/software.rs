use serde_json::{json, Value};

fn entry(codec: &str, operation: &str, name: &str) -> Value {
    json!({
        "codec": codec,
        "operation": operation,
        "name": name,
        "backend": name,
        "hardware": false,
        "available": true,
    })
}

pub(super) fn backends() -> Vec<Value> {
    let mut entries = Vec::new();
    for codec in ["VP8", "VP9", "AV1"] {
        let name = if codec == "AV1" { "libaom" } else { "libvpx" };
        entries.push(entry(codec, "encode", name));
        entries.push(entry(codec, "decode", name));
    }
    #[cfg(target_os = "linux")]
    entries.push(entry("AV1", "decode", "dav1d"));
    entries
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod performance {
    use super::*;
    use hbb_common::{
        anyhow::{anyhow, bail},
        bytes::Bytes,
        ResultType,
    };
    use scrap::{
        aom::{AomDecoder, AomEncoder, AomEncoderConfig},
        codec::{EncoderApi, EncoderCfg},
        vpxcodec::{VpxDecoder, VpxDecoderConfig, VpxEncoder, VpxEncoderConfig, VpxVideoCodecId},
        EncodeInput, GoogleImage, ImageFormat, ImageRgb,
    };
    use std::time::{Duration, Instant};

    use super::super::workload::{
        convert_bgra_to_yuv, DesktopFrames, FPS, HEIGHT, ITERATIONS, WIDTH,
    };

    type EncodedFrame = (Bytes, bool, i64);
    type Packet = Vec<EncodedFrame>;
    type PacketDecoder = Box<dyn FnMut(&Packet) -> ResultType<usize>>;

    fn encode(codec: &str) -> ResultType<(Vec<Packet>, usize, Duration)> {
        let mut encoder: Box<dyn EncoderApi> = if codec == "AV1" {
            Box::new(AomEncoder::new(
                EncoderCfg::AOM(AomEncoderConfig {
                    width: WIDTH as u32,
                    height: HEIGHT as u32,
                    quality: 0.67,
                    keyframe_interval: None,
                }),
                false,
            )?)
        } else {
            Box::new(VpxEncoder::new(
                EncoderCfg::VPX(VpxEncoderConfig {
                    width: WIDTH as u32,
                    height: HEIGHT as u32,
                    quality: 0.67,
                    keyframe_interval: None,
                    codec: if codec == "VP8" {
                        VpxVideoCodecId::VP8
                    } else {
                        VpxVideoCodecId::VP9
                    },
                }),
                false,
            )?)
        };
        let yuv_format = encoder.yuvfmt();
        let mut desktop = DesktopFrames::new(WIDTH, HEIGHT);
        let mut yuv = Vec::new();
        let mut packets = Vec::new();
        let mut frames = 0;
        let mut elapsed = Duration::ZERO;
        for index in 0..ITERATIONS {
            let bgra = desktop.next_frame(index);
            let frame_started = Instant::now();
            convert_bgra_to_yuv(bgra, &yuv_format, &mut yuv)?;
            let frame =
                encoder.encode_to_message(EncodeInput::YUV(&yuv), (index * 1000 / FPS) as i64)?;
            elapsed += frame_started.elapsed();
            let encoded = match codec {
                "VP8" => frame.vp8s(),
                "VP9" => frame.vp9s(),
                _ => frame.av1s(),
            };
            if !encoded.frames.is_empty() {
                packets.push(
                    encoded
                        .frames
                        .iter()
                        .map(|frame| (frame.data.clone(), frame.key, frame.pts))
                        .collect(),
                );
            }
            frames += encoded.frames.len();
        }
        Ok((packets, frames, elapsed))
    }

    fn decode(codec: &str, name: &str, packets: &[Packet]) -> ResultType<(usize, Duration)> {
        let mut rgb = ImageRgb::new(ImageFormat::ABGR, 1);
        let mut decode: PacketDecoder = if codec != "AV1" {
            let mut decoder = VpxDecoder::new(VpxDecoderConfig {
                codec: if codec == "VP8" {
                    VpxVideoCodecId::VP8
                } else {
                    VpxVideoCodecId::VP9
                },
            })?;
            Box::new(move |packet| {
                let mut last = None;
                for (data, _, _) in packet {
                    for picture in decoder.decode(data)? {
                        last = Some(picture);
                    }
                }
                for picture in decoder.flush()? {
                    last = Some(picture);
                }
                if let Some(picture) = last {
                    picture.to(&mut rgb);
                    Ok(1)
                } else {
                    Ok(0)
                }
            })
        } else if name == "libaom" {
            let mut decoder = AomDecoder::new()?;
            Box::new(move |packet| {
                let mut last = None;
                for (data, _, _) in packet {
                    for picture in decoder.decode(data)? {
                        last = Some(picture);
                    }
                }
                for picture in decoder.flush()? {
                    last = Some(picture);
                }
                if let Some(picture) = last {
                    picture.to(&mut rgb);
                    Ok(1)
                } else {
                    Ok(0)
                }
            })
        } else {
            #[cfg(target_os = "linux")]
            {
                use base::message_proto::{EncodedVideoFrame, EncodedVideoFrames};
                use scrap::av1_decoder::Av1Decoder;
                let mut decoder = Av1Decoder::new()?;
                if !matches!(&decoder, Av1Decoder::Dav1d(_)) {
                    bail!("dav1d initialization failed");
                }
                Box::new(move |packet| {
                    let frames = EncodedVideoFrames {
                        frames: packet
                            .iter()
                            .map(|(data, key, pts)| EncodedVideoFrame {
                                data: data.clone(),
                                key: *key,
                                pts: *pts,
                                ..Default::default()
                            })
                            .collect::<Vec<_>>()
                            .into(),
                        ..Default::default()
                    };
                    let mut chroma = None;
                    Ok(usize::from(decoder.handle_frame(
                        &frames,
                        &mut rgb,
                        &mut chroma,
                    )?))
                })
            }
            #[cfg(not(target_os = "linux"))]
            bail!("Unsupported software decoder: {name}");
        };

        let started = Instant::now();
        let mut frames = 0;
        for packet in packets {
            frames += decode(packet)?;
        }
        Ok((frames, started.elapsed()))
    }

    fn result(mut entry: Value, measured: ResultType<(usize, Duration)>) -> Value {
        match measured {
            Ok((frames, elapsed)) if frames > 0 => {
                entry["frames"] = json!(frames);
                entry["fps"] = json!(frames as f64 / elapsed.as_secs_f64().max(f64::EPSILON));
                entry["duration_ms"] = json!(elapsed.as_secs_f64() * 1000.0);
            }
            Ok(_) => entry["error"] = json!("No frames produced"),
            Err(error) => entry["error"] = json!(error.to_string()),
        }
        entry
    }

    pub(in crate::hwcodec_diagnostics) fn benchmark() -> Vec<Value> {
        let backends = backends();
        let mut results = Vec::new();
        for codec in ["VP8", "VP9", "AV1"] {
            let name = if codec == "AV1" { "libaom" } else { "libvpx" };
            let encoded = encode(codec);
            let mut encode_entry = entry(codec, "encode", name);
            match &encoded {
                Ok((_, frames, elapsed)) => {
                    encode_entry["frames"] = json!(frames);
                    encode_entry["fps"] =
                        json!(*frames as f64 / elapsed.as_secs_f64().max(f64::EPSILON));
                    encode_entry["duration_ms"] = json!(elapsed.as_secs_f64() * 1000.0);
                    encode_entry["pipeline"] = json!("BGRA to YUV conversion + encode");
                    encode_entry["error"] = Value::Null;
                }
                Err(error) => encode_entry["error"] = json!(error.to_string()),
            }
            results.push(encode_entry);

            for backend in backends
                .iter()
                .filter(|entry| entry["codec"] == codec && entry["operation"] == "decode")
            {
                let measured = match &encoded {
                    Ok((packets, _, _)) if !packets.is_empty() => {
                        decode(codec, backend["name"].as_str().unwrap_or_default(), packets)
                    }
                    _ => Err(anyhow!("Unable to prepare encoded test frames")),
                };
                let mut decode_entry = result(backend.clone(), measured);
                decode_entry["pipeline"] = json!("decode + YUV to ABGR conversion");
                decode_entry["test_stream"] = json!("dynamic desktop stream");
                results.push(decode_entry);
            }
        }
        results
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(super) use performance::benchmark;
