use super::super::prelude::*;
use dioxus::desktop::{window, wry::WebViewExtUnix};
use gtk::{glib, prelude::*};
use hbb_common::anyhow::{anyhow, Result};
use std::{cell::RefCell, rc::Rc};

mod gpu;

#[derive(Clone, Copy, Default, serde::Deserialize)]
pub(in crate::dioxus_ui) struct Viewport {
    rect: [f64; 4],
    clip: [f64; 4],
    window: [f64; 2],
}

impl Viewport {
    fn project(&self, size: (i32, i32)) -> Option<([i32; 4], [f32; 4])> {
        if self
            .rect
            .iter()
            .chain(&self.clip)
            .chain(&self.window)
            .any(|v| !v.is_finite())
            || self.rect[2] <= 0.
            || self.rect[3] <= 0.
            || self.window[0] <= 0.
            || self.window[1] <= 0.
        {
            return None;
        }
        let left = self.clip[0].max(self.rect[0]).max(0.);
        let top = self.clip[1].max(self.rect[1]).max(0.);
        let right = (self.clip[0] + self.clip[2])
            .min(self.rect[0] + self.rect[2])
            .min(self.window[0]);
        let bottom = (self.clip[1] + self.clip[3])
            .min(self.rect[1] + self.rect[3])
            .min(self.window[1]);
        if right <= left || bottom <= top {
            return None;
        }
        let sx = size.0 as f64 / self.window[0];
        let sy = size.1 as f64 / self.window[1];
        let x = (left * sx).round() as i32;
        let y = (top * sy).round() as i32;
        let width = (right * sx).round() as i32 - x;
        let height = (bottom * sy).round() as i32 - y;
        Some((
            [x, size.1 - y - height, width, height],
            [
                ((left - self.rect[0]) / self.rect[2]) as f32,
                ((top - self.rect[1]) / self.rect[3]) as f32,
                ((right - left) / self.rect[2]) as f32,
                ((bottom - top) / self.rect[3]) as f32,
            ],
        ))
    }
}

#[derive(Default)]
struct SurfaceState {
    gpu: Option<gpu::Gpu>,
    pending: Option<super::super::session::FramePacket>,
    viewport: Viewport,
}

pub(in crate::dioxus_ui) struct NativeVideoSurface {
    area: gtk::GLArea,
    overlay: gtk::Overlay,
    host: gtk::Box,
    web: gtk::Widget,
    state: Rc<RefCell<SurfaceState>>,
    tick: Option<gtk::TickCallbackId>,
}

impl NativeVideoSurface {
    pub(in crate::dioxus_ui) fn mount(handler: DioxusHandler) -> Result<Self> {
        let desktop = window();
        let web: gtk::Widget = desktop.webview.webview().upcast();
        let host = web
            .parent()
            .and_then(|w| w.downcast::<gtk::Box>().ok())
            .ok_or_else(|| anyhow!("Missing WebView GTK container"))?;
        let area = gtk::GLArea::new();
        area.set_required_version(3, 2);
        area.set_use_es(false);
        area.set_auto_render(false);
        area.set_hexpand(true);
        area.set_vexpand(true);
        let overlay = gtk::Overlay::new();
        let state = Rc::new(RefCell::new(SurfaceState::default()));
        let realized = state.clone();
        area.connect_realize(move |area| {
            area.make_current();
            if let Some(error) = area.error() {
                log::error!("Cannot create native video context: {error}");
                return;
            }
            match unsafe { gpu::Gpu::new() } {
                Ok(gpu) => realized.borrow_mut().gpu = Some(gpu),
                Err(error) => log::error!("Cannot initialize native video: {error}"),
            }
        });
        let rendering = state.clone();
        let render_handler = handler.clone();
        area.connect_render(move |area, _| {
            let mut state = rendering.borrow_mut();
            let packet = state.pending.take();
            let viewport = state.viewport;
            if let Some(gpu) = &mut state.gpu {
                let scale = area.scale_factor();
                if let Err(error) = unsafe {
                    gpu.draw(
                        packet.as_ref(),
                        (
                            area.allocated_width() * scale,
                            area.allocated_height() * scale,
                        ),
                        &viewport,
                    )
                } {
                    hbb_common::throttled_log!(
                        Duration::from_secs(30),
                        warn,
                        "Native video: {error}"
                    );
                }
            }
            if let Some(packet) = packet {
                render_handler.recycle_frame(packet.raw);
            }
            glib::Propagation::Stop
        });
        let unrealized = state.clone();
        area.connect_unrealize(move |area| {
            area.make_current();
            if let Some(gpu) = unrealized.borrow_mut().gpu.take() {
                if area.error().is_none() {
                    unsafe {
                        gpu.destroy();
                    }
                }
            }
        });
        let ticking = state.clone();
        let tick = area.add_tick_callback(move |area, _| {
            if let Some(packet) = handler.take_frame() {
                if let Some(previous) = ticking.borrow_mut().pending.replace(packet) {
                    handler.recycle_frame(previous.raw);
                }
                area.queue_render();
            }
            glib::ControlFlow::Continue
        });
        desktop.webview.set_background_color((0, 0, 0, 0))?;
        host.remove(&web);
        overlay.add(&area);
        overlay.add_overlay(&web);
        host.pack_start(&overlay, true, true, 0);
        overlay.show_all();
        let surface = Self {
            area,
            overlay,
            host,
            web,
            state,
            tick: Some(tick),
        };
        if surface.state.borrow().gpu.is_none() {
            return Err(anyhow!("Native OpenGL video initialization failed"));
        }
        Ok(surface)
    }

    pub(in crate::dioxus_ui) fn set_viewport(&self, viewport: Viewport) {
        self.state.borrow_mut().viewport = viewport;
        self.area.queue_render();
    }
}

impl Drop for NativeVideoSurface {
    fn drop(&mut self) {
        if let Some(tick) = self.tick.take() {
            tick.remove();
        }
        if self.overlay.parent().is_some() {
            self.overlay.remove(&self.web);
            self.host.remove(&self.overlay);
            self.host.pack_start(&self.web, true, true, 0);
            self.web.show();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_surface_matches_fit_hidpi_and_scrolled_original_viewports() {
        let fit = Viewport {
            rect: [20., 40., 800., 450.],
            clip: [0., 32., 1000., 600.],
            window: [1000., 650.],
        };
        assert_eq!(
            fit.project((2000, 1300)),
            Some(([40, 320, 1600, 900], [0., 0., 1., 1.]))
        );
        let scrolled = Viewport {
            rect: [-100., -50., 1920., 1080.],
            clip: [0., 32., 800., 600.],
            window: [800., 650.],
        };
        let (rect, crop) = scrolled.project((800, 650)).unwrap();
        assert_eq!(rect, [0, 18, 800, 600]);
        assert!((crop[0] - 100. / 1920.).abs() < 0.0001);
        assert!((crop[1] - 82. / 1080.).abs() < 0.0001);
        assert!((crop[2] - 800. / 1920.).abs() < 0.0001);
        assert!((crop[3] - 600. / 1080.).abs() < 0.0001);
    }
}
