use hbb_common::{bail, ResultType};
use scrap::{EncodeYuvFormat, Pixfmt};

pub(super) const WIDTH: usize = 1280;
pub(super) const HEIGHT: usize = 720;
pub(super) const FPS: usize = 30;
pub(super) const ITERATIONS: usize = FPS * 2;

#[derive(Clone, Copy)]
struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

pub(super) struct DesktopFrames {
    width: usize,
    height: usize,
    background: Vec<u8>,
    current: Vec<u8>,
    previous_window: Option<Rect>,
}

impl DesktopFrames {
    pub(super) fn new(width: usize, height: usize) -> Self {
        let mut background = vec![0; width * height * 4];
        fill_rect(
            &mut background,
            width,
            height,
            Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
            [48, 42, 36],
        );
        fill_rect(
            &mut background,
            width,
            height,
            Rect {
                x: 0,
                y: 0,
                width,
                height: 44.min(height),
            },
            [58, 51, 44],
        );
        fill_rect(
            &mut background,
            width,
            height,
            Rect {
                x: 0,
                y: 44.min(height),
                width: 190.min(width),
                height: height.saturating_sub(44),
            },
            [68, 61, 53],
        );

        let content_x = 210.min(width);
        let content_y = 62.min(height);
        fill_rect(
            &mut background,
            width,
            height,
            Rect {
                x: content_x,
                y: content_y,
                width: width.saturating_sub(content_x + 16),
                height: height.saturating_sub(content_y + 16),
            },
            [238, 239, 237],
        );

        for row in 0..12 {
            let y = 78 + row * 42;
            if y + 8 >= height {
                break;
            }
            fill_rect(
                &mut background,
                width,
                height,
                Rect {
                    x: content_x + 22,
                    y,
                    width: (width / 3 + (row * 37) % (width / 3).max(1))
                        .min(width.saturating_sub(content_x + 40)),
                    height: 8,
                },
                [170, 166, 159],
            );
            fill_rect(
                &mut background,
                width,
                height,
                Rect {
                    x: content_x + 22,
                    y: y + 14,
                    width: width.saturating_sub(content_x + 68),
                    height: 1,
                },
                [211, 212, 209],
            );
        }

        Self {
            width,
            height,
            current: background.clone(),
            background,
            previous_window: None,
        }
    }

    pub(super) fn next_frame(&mut self, index: usize) -> &[u8] {
        if let Some(previous) = self.previous_window {
            restore_rect(&mut self.current, &self.background, self.width, previous);
        }

        let window_width = (self.width / 3).clamp(1, 420);
        let window_height = (self.height / 3).clamp(1, 260);
        let x_span = self.width.saturating_sub(window_width);
        let y_span = self.height.saturating_sub(window_height);
        let window = Rect {
            x: if x_span == 0 {
                0
            } else {
                (index * 13) % (x_span + 1)
            },
            y: if y_span == 0 {
                0
            } else {
                (index * 7) % (y_span + 1)
            },
            width: window_width,
            height: window_height,
        };
        fill_rect(
            &mut self.current,
            self.width,
            self.height,
            window,
            [222, 224, 226],
        );
        fill_rect(
            &mut self.current,
            self.width,
            self.height,
            Rect {
                x: window.x,
                y: window.y,
                width: window.width,
                height: 24.min(window.height),
            },
            [74, 105, 138],
        );
        for row in 0..(window.height / 22).min(10) {
            fill_rect(
                &mut self.current,
                self.width,
                self.height,
                Rect {
                    x: window.x + 12,
                    y: window.y + 36 + row * 22,
                    width: (window.width.saturating_sub(24) * (50 + (row * 17) % 45)) / 100,
                    height: 6,
                },
                [130 + (index % 40) as u8, 139, 146],
            );
        }
        self.previous_window = Some(window);
        &self.current
    }
}

pub(super) fn convert_bgra_to_yuv(
    bgra: &[u8],
    format: &EncodeYuvFormat,
    output: &mut Vec<u8>,
) -> ResultType<()> {
    if bgra.len() < format.w.saturating_mul(format.h).saturating_mul(4) || format.stride.len() < 3 {
        bail!("invalid synthetic desktop frame layout");
    }

    let required = match format.pixfmt {
        Pixfmt::NV12 => format
            .u
            .saturating_add(format.stride[1].saturating_mul(format.h / 2)),
        Pixfmt::I420 => format
            .v
            .saturating_add(format.stride[2].saturating_mul(format.h / 2)),
        Pixfmt::I444 => format
            .v
            .saturating_add(format.stride[2].saturating_mul(format.h)),
        _ => bail!(
            "unsupported benchmark encoder pixel format: {:?}",
            format.pixfmt
        ),
    };
    output.resize(required, 0);

    let width = format.w as i32;
    let height = format.h as i32;
    let source_stride = (format.w * 4) as i32;
    let destination = output.as_mut_ptr();
    let result = unsafe {
        match format.pixfmt {
            Pixfmt::NV12 => scrap::ARGBToNV12(
                bgra.as_ptr(),
                source_stride,
                destination,
                format.stride[0] as i32,
                destination.add(format.u),
                format.stride[1] as i32,
                width,
                height,
            ),
            Pixfmt::I420 => scrap::ARGBToI420(
                bgra.as_ptr(),
                source_stride,
                destination,
                format.stride[0] as i32,
                destination.add(format.u),
                format.stride[1] as i32,
                destination.add(format.v),
                format.stride[2] as i32,
                width,
                height,
            ),
            Pixfmt::I444 => scrap::ARGBToI444(
                bgra.as_ptr(),
                source_stride,
                destination,
                format.stride[0] as i32,
                destination.add(format.u),
                format.stride[1] as i32,
                destination.add(format.v),
                format.stride[2] as i32,
                width,
                height,
            ),
            _ => unreachable!(),
        }
    };
    if result != 0 {
        bail!("BGRA to YUV conversion failed: {result}");
    }
    Ok(())
}

fn fill_rect(frame: &mut [u8], width: usize, height: usize, rect: Rect, color: [u8; 3]) {
    let right = rect.x.saturating_add(rect.width).min(width);
    let bottom = rect.y.saturating_add(rect.height).min(height);
    for y in rect.y.min(height)..bottom {
        for x in rect.x.min(width)..right {
            let index = (y * width + x) * 4;
            frame[index..index + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
        }
    }
}

fn restore_rect(frame: &mut [u8], background: &[u8], width: usize, rect: Rect) {
    for y in rect.y..rect.y + rect.height {
        let start = (y * width + rect.x) * 4;
        let end = start + rect.width * 4;
        frame[start..end].copy_from_slice(&background[start..end]);
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopFrames;

    #[test]
    fn desktop_frames_change_between_timestamps() {
        let mut desktop = DesktopFrames::new(320, 180);
        let first = desktop.next_frame(0).to_vec();
        let second = desktop.next_frame(1);

        assert_ne!(first, second);
        assert_eq!(first.len(), 320 * 180 * 4);
        assert_eq!(second.len(), first.len());
    }
}
