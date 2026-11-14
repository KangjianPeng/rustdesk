use super::{aom, codec::codec_thread_num, GoogleImage, ImageRgb};
use base::message_proto::{Chroma, EncodedVideoFrames};
use dav1d::{Decoder, Error, Picture, PixelLayout, PlanarImageComponent as Plane, Settings};
use hbb_common::{anyhow::anyhow, log, ResultType};

pub enum Av1Decoder {
    Dav1d(Decoder),
    Aom(aom::AomDecoder),
}

impl Av1Decoder {
    pub fn new() -> ResultType<Self> {
        let mut settings = Settings::new();
        settings.set_n_threads(codec_thread_num(64) as u32);
        settings.set_max_frame_delay(1);
        match Decoder::with_settings(&settings) {
            Ok(decoder) => {
                log::info!(
                    "AV1 decoder: dav1d, low latency, {} threads",
                    settings.get_n_threads()
                );
                Ok(Self::Dav1d(decoder))
            }
            Err(error) => {
                log::warn!("Cannot initialize dav1d, using libaom: {error}");
                Ok(Self::Aom(aom::AomDecoder::new()?))
            }
        }
    }

    pub fn handle_frame(
        &mut self,
        frames: &EncodedVideoFrames,
        rgb: &mut ImageRgb,
        chroma: &mut Option<Chroma>,
    ) -> ResultType<bool> {
        let mut last = None;
        match self {
            Self::Dav1d(decoder) => {
                for frame in &frames.frames {
                    let mut pending =
                        match decoder.send_data(frame.data.clone(), None, Some(frame.pts), None) {
                            Ok(()) => false,
                            Err(Error::Again) => true,
                            Err(error) => return Err(error.into()),
                        };
                    loop {
                        loop {
                            match decoder.get_picture() {
                                Ok(picture) => last = Some(picture),
                                Err(Error::Again) => break,
                                Err(error) => return Err(error.into()),
                            }
                        }
                        if !pending {
                            break;
                        }
                        pending = match decoder.send_pending_data() {
                            Ok(()) => false,
                            Err(Error::Again) => true,
                            Err(error) => return Err(error.into()),
                        };
                    }
                }
                if let Some(picture) = last {
                    if picture.bit_depth() != 8
                        || !matches!(
                            picture.pixel_layout(),
                            PixelLayout::I420 | PixelLayout::I444
                        )
                    {
                        return Err(anyhow!(
                            "Unsupported AV1 picture: {:?}, {} bit",
                            picture.pixel_layout(),
                            picture.bit_depth()
                        ));
                    }
                    *chroma = Some(picture.chroma());
                    picture.to(rgb);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Self::Aom(decoder) => {
                let mut last = aom::Image::new();
                for frame in &frames.frames {
                    for picture in decoder.decode(&frame.data)? {
                        last = picture;
                    }
                }
                for picture in decoder.flush()? {
                    last = picture;
                }
                if last.is_null() {
                    Ok(false)
                } else {
                    *chroma = Some(last.chroma());
                    last.to(rgb);
                    Ok(true)
                }
            }
        }
    }
}

impl GoogleImage for Picture {
    fn width(&self) -> usize {
        self.width() as usize
    }
    fn height(&self) -> usize {
        self.height() as usize
    }
    fn stride(&self) -> Vec<i32> {
        [Plane::Y, Plane::U, Plane::V]
            .map(|plane| self.stride(plane) as i32)
            .to_vec()
    }
    fn planes(&self) -> Vec<*mut u8> {
        [Plane::Y, Plane::U, Plane::V]
            .map(|plane| self.plane_data_ptr(plane).cast())
            .to_vec()
    }
    fn chroma(&self) -> Chroma {
        if self.pixel_layout() == PixelLayout::I444 {
            Chroma::I444
        } else {
            Chroma::I420
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aom::{AomEncoder, AomEncoderConfig},
        codec::{EncoderApi, EncoderCfg},
        EncodeInput, ImageFormat,
    };

    #[test]
    fn dav1d_decodes_i420_and_i444_without_frame_delay() {
        let benchmark = std::env::var_os("RUSTDESK_AV1_BENCH").is_some();
        let (width, height, count) = if benchmark {
            (1920, 1080, 60)
        } else {
            (64, 64, 3)
        };
        for i444 in [false, true] {
            let mut encoder = AomEncoder::new(
                EncoderCfg::AOM(AomEncoderConfig {
                    width,
                    height,
                    quality: 1.0,
                    keyframe_interval: None,
                }),
                i444,
            )
            .unwrap();
            let format = encoder.yuvfmt();
            let mut yuv = vec![
                128;
                format.v
                    + format.stride[2]
                        * if i444 {
                            height as usize
                        } else {
                            height as usize / 2
                        }
            ];
            let mut native = Av1Decoder::new().unwrap();
            assert!(matches!(native, Av1Decoder::Dav1d(_)));
            let mut reference = Av1Decoder::Aom(aom::AomDecoder::new().unwrap());
            let mut rgb = ImageRgb::new(ImageFormat::ABGR, 1);
            let mut expected = ImageRgb::new(ImageFormat::ABGR, 1);
            let mut chroma = None;
            let mut native_time = std::time::Duration::ZERO;
            let mut reference_time = std::time::Duration::ZERO;
            for index in 0..count {
                for y in 0..height as usize {
                    for x in 0..width as usize {
                        yuv[y * format.stride[0] + x] =
                            32 + ((x / 4 + y / 3 + index * 7) % 192) as u8;
                    }
                }
                let frame = encoder
                    .encode_to_message(EncodeInput::YUV(&yuv), index as i64 * 33)
                    .unwrap();
                let frames = frame.av1s();
                let started = std::time::Instant::now();
                assert!(native.handle_frame(frames, &mut rgb, &mut chroma).unwrap());
                native_time += started.elapsed();
                let started = std::time::Instant::now();
                assert!(reference
                    .handle_frame(frames, &mut expected, &mut None)
                    .unwrap());
                reference_time += started.elapsed();
                assert_eq!((rgb.w, rgb.h), (width as usize, height as usize));
                assert_eq!(chroma, Some(if i444 { Chroma::I444 } else { Chroma::I420 }));
                assert_eq!(rgb.raw, expected.raw);
            }
            println!("AV1 {}x{} i444={i444}: {count} frames, dav1d+RGB {:.3} ms/frame, libaom+RGB {:.3} ms/frame", width, height, native_time.as_secs_f64() * 1000.0 / count as f64, reference_time.as_secs_f64() * 1000.0 / count as f64);
        }
    }
}
