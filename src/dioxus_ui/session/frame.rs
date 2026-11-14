use super::super::prelude::*;

pub(super) struct FramePacket {
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) format: ImageFormat,
    pub(super) align: usize,
    pub(super) raw: Vec<u8>,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Default)]
pub(super) struct FramePipeline {
    pub(super) pending: Mutex<Option<FramePacket>>,
    pub(super) recycled: Mutex<Vec<Vec<u8>>>,
    pub(super) wake: Condvar,
}

#[cfg(any(target_os = "android", target_os = "ios", test))]
pub(crate) fn encode_frame(
    width: usize,
    height: usize,
    format: ImageFormat,
    align: usize,
    raw: &[u8],
) -> Option<String> {
    if width == 0 || height == 0 {
        return None;
    }
    let bytes_per_pixel = match format {
        ImageFormat::Raw => 3,
        ImageFormat::ARGB | ImageFormat::ABGR => 4,
    };
    let align = align.max(1);
    let stride = (width.checked_mul(bytes_per_pixel)? + align - 1) & !(align - 1);
    let required = stride.checked_mul(height)?;
    if raw.len() < required {
        return None;
    }

    let width_u32 = u32::try_from(width).ok()?;
    let height_u32 = u32::try_from(height).ok()?;
    let mut png = Vec::new();
    let mut encoder = repng::Options::fastest(width_u32, height_u32)
        .build(&mut png)
        .ok()?;
    let mut row = vec![0_u8; width * 4];
    for source_row in raw.chunks_exact(stride).take(height) {
        for x in 0..width {
            let source = x * bytes_per_pixel;
            let target = x * 4;
            match format {
                ImageFormat::Raw => {
                    row[target..target + 3].copy_from_slice(&source_row[source..source + 3]);
                    row[target + 3] = 255;
                }
                ImageFormat::ARGB => {
                    row[target..target + 4].copy_from_slice(&[
                        source_row[source + 2],
                        source_row[source + 1],
                        source_row[source],
                        source_row[source + 3],
                    ]);
                }
                ImageFormat::ABGR => {
                    row[target..target + 4].copy_from_slice(&source_row[source..source + 4]);
                }
            }
        }
        encoder.write(&row).ok()?;
    }
    encoder.finish().ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub(super) fn frame_worker(pipeline: Weak<FramePipeline>, state: SyncSignal<DioxusUiState>) {
    loop {
        let Some(pipeline) = pipeline.upgrade() else {
            return;
        };
        let packet = {
            let mut pending = match pipeline.pending.lock() {
                Ok(pending) => pending,
                Err(error) => {
                    log::error!("Dioxus frame queue was poisoned: {error}");
                    return;
                }
            };
            while pending.is_none() {
                pending = match pipeline.wake.wait_timeout(pending, Duration::from_secs(1)) {
                    Ok((pending, timeout)) => {
                        if timeout.timed_out() && Arc::strong_count(&pipeline) == 1 {
                            return;
                        }
                        pending
                    }
                    Err(error) => {
                        log::error!("Dioxus frame queue wait failed: {error}");
                        return;
                    }
                };
            }
            pending.take()
        };

        let Some(packet) = packet else {
            continue;
        };
        let FramePacket {
            width,
            height,
            format,
            align,
            raw,
        } = packet;
        if let Some(frame_data_uri) = encode_frame(width, height, format, align, &raw) {
            let mut state = state;
            let mut state = state.write();
            state.width = width as i32;
            state.height = height as i32;
            state.frame_data_uri = Arc::from(frame_data_uri);
        }
        let recycled_result = pipeline.recycled.lock();
        if let Ok(mut recycled) = recycled_result {
            recycled.push(raw);
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod desktop {
    use super::super::video::VideoTexture;
    use super::*;
    use dioxus::native::{CustomPaintCtx, CustomPaintSource, DeviceHandle, TextureHandle};

    #[derive(Default)]
    pub(in crate::dioxus_ui) struct FramePipeline {
        pending: Mutex<Option<FramePacket>>,
        pub(in crate::dioxus_ui) recycled: Mutex<Vec<Vec<u8>>>,
    }
    impl FramePipeline {
        pub(in crate::dioxus_ui::session) fn submit(&self, packet: FramePacket) {
            if let Some(previous) = self.pending.lock().unwrap().replace(packet) {
                self.recycle(previous.raw);
            }
            crate::dioxus_ui::native::request_redraw();
        }
        fn recycle(&self, raw: Vec<u8>) {
            let mut recycled = self.recycled.lock().unwrap();
            if recycled.len() < 2 {
                recycled.push(raw);
            }
        }
    }

    #[derive(Default)]
    pub(in crate::dioxus_ui) struct NativeVideo {
        frames: Arc<FramePipeline>,
        texture: Option<VideoTexture>,
    }
    impl NativeVideo {
        pub(in crate::dioxus_ui) fn new(frames: Arc<FramePipeline>) -> Self {
            Self {
                frames,
                texture: None,
            }
        }
    }
    impl CustomPaintSource for NativeVideo {
        fn resume(&mut self, device: &DeviceHandle) {
            let info = device.adapter.get_info();
            log::info!("Native video renderer: {} ({:?})", info.name, info.backend);
            self.texture = Some(VideoTexture::new(device.clone()));
        }
        fn suspend(&mut self) {
            self.texture = None;
        }
        fn render(
            &mut self,
            mut ctx: CustomPaintCtx<'_>,
            width: u32,
            height: u32,
            scale: f64,
        ) -> Option<TextureHandle> {
            let texture = self.texture.as_mut()?;
            let packet = self.frames.pending.lock().unwrap().take();
            if let Some(packet) = packet {
                if let Some((width, height, stride, _format)) = texture_layout(&packet) {
                    texture.upload(width, height, stride, &packet.raw);
                } else {
                    hbb_common::throttled_log!(
                        Duration::from_secs(30),
                        warn,
                        "Invalid native video frame"
                    );
                }
                self.frames.recycle(packet.raw);
            }
            texture.render(&mut ctx, width, height, scale)
        }
    }

    fn texture_layout(packet: &FramePacket) -> Option<(u32, u32, u32, wgpu::TextureFormat)> {
        let format = match packet.format {
            ImageFormat::ARGB => return None,
            ImageFormat::ABGR => wgpu::TextureFormat::Rgba8Unorm,
            ImageFormat::Raw => return None,
        };
        let align = packet.align.max(1);
        let stride = packet.width.checked_mul(4)?.checked_add(align - 1)? / align * align;
        if packet.width == 0
            || packet.height == 0
            || packet.raw.len() < stride.checked_mul(packet.height)?
        {
            return None;
        }
        Some((
            u32::try_from(packet.width).ok()?,
            u32::try_from(packet.height).ok()?,
            u32::try_from(stride).ok()?,
            format,
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn native_texture_preserves_rgba_stride_and_rejects_truncated_pixels() {
            let mut packet = FramePacket {
                width: 1,
                height: 2,
                format: ImageFormat::ABGR,
                align: 8,
                raw: vec![0, 0, 255, 255, 9, 9, 9, 9, 0, 255, 0, 255, 9, 9, 9, 9],
            };
            assert_eq!(
                texture_layout(&packet),
                Some((1, 2, 8, wgpu::TextureFormat::Rgba8Unorm))
            );
            packet.raw.truncate(8);
            assert!(texture_layout(&packet).is_none());
        }
        #[test]
        fn native_frame_queue_presents_latest_frame() {
            let frames = FramePipeline::default();
            for pixel in [1, 2, 3] {
                frames.submit(FramePacket {
                    width: 1,
                    height: 1,
                    format: ImageFormat::ARGB,
                    align: 1,
                    raw: vec![pixel; 4],
                });
            }
            assert_eq!(
                frames.pending.lock().unwrap().take().unwrap().raw,
                vec![3; 4]
            );
        }
    }
}
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(super) use desktop::{FramePipeline, NativeVideo};
