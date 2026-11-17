use super::super::prelude::*;

pub(in crate::dioxus_ui) struct FramePacket {
    pub(in crate::dioxus_ui) width: usize,
    pub(in crate::dioxus_ui) height: usize,
    pub(in crate::dioxus_ui) format: ImageFormat,
    pub(in crate::dioxus_ui) align: usize,
    pub(in crate::dioxus_ui) raw: Vec<u8>,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Default)]
pub(super) struct FramePipeline {
    pub(super) pending: Mutex<Option<FramePacket>>,
    pub(super) recycled: Mutex<Vec<Vec<u8>>>,
    pub(super) wake: Condvar,
}

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
#[derive(Default)]
pub(super) struct FramePipeline {
    pub(super) pending: Mutex<Option<FramePacket>>,
    pub(super) recycled: Mutex<Vec<Vec<u8>>>,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl FramePipeline {
    pub(super) fn submit(&self, packet: FramePacket) {
        if let Some(previous) = self.pending.lock().unwrap().replace(packet) {
            let mut recycled = self.recycled.lock().unwrap();
            if recycled.len() < 2 {
                recycled.push(previous.raw);
            }
        }
    }

    pub(super) fn recycle(&self, raw: Vec<u8>) {
        let mut recycled = self.recycled.lock().unwrap();
        if recycled.len() < 2 {
            recycled.push(raw);
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl FramePacket {
    pub(in crate::dioxus_ui) fn stride(&self) -> Option<usize> {
        let bytes_per_pixel = if matches!(self.format, ImageFormat::Raw) {
            3
        } else {
            4
        };
        let align = self.align.max(1);
        let stride = self
            .width
            .checked_mul(bytes_per_pixel)?
            .checked_add(align - 1)?
            / align
            * align;
        (self.width > 0 && self.height > 0 && self.raw.len() >= stride.checked_mul(self.height)?)
            .then_some(stride)
    }
}

#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod tests {
    use super::*;

    #[test]
    fn webview_receives_latest_frame_with_its_original_stride() {
        let frames = FramePipeline::default();
        for pixel in [1, 2, 3] {
            frames.submit(FramePacket {
                width: 1,
                height: 2,
                format: ImageFormat::ABGR,
                align: 8,
                raw: vec![pixel; 16],
            });
        }
        let mut packet = frames.pending.lock().unwrap().take().unwrap();
        assert_eq!(packet.raw, vec![3; 16]);
        assert_eq!(packet.stride(), Some(8));
        packet.raw.truncate(8);
        assert!(packet.stride().is_none());
    }
}
