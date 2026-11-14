use super::super::prelude::*;
use dioxus::native::DeviceHandle;

pub(super) struct CursorTexture {
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    shape: Option<Arc<CursorData>>,
    rect: [f32; 4],
}

impl CursorTexture {
    pub fn new(device: &DeviceHandle, layout: wgpu::BindGroupLayout) -> Self {
        let uniform = device.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RustDesk cursor rectangle"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let view = Self::texture(device, 1, 1).create_view(&Default::default());
        let bind_group = Self::bind_group(device, &layout, &view, &uniform);
        Self {
            layout,
            uniform,
            bind_group,
            shape: None,
            rect: [0.0; 4],
        }
    }

    pub fn update(&mut self, device: &DeviceHandle, width: u32, height: u32, scale: f64) -> bool {
        let mut shape = None;
        let mut rect = [0.0; 4];
        if let Some(session) = crate::dioxus_ui::current_session() {
            if session.get_toggle_option("show-remote-cursor".to_owned()) {
                if let Some(status) = session.ui_handler.snapshot() {
                    let cursors = session.ui_handler.cursors.lock().unwrap();
                    if let Some(display) =
                        status.displays.get(status.current_display.max(0) as usize)
                    {
                        let zoom = session.get_toggle_option("zoom-cursor".to_owned())
                            || session.get_view_style() == "original";
                        if let Some(value) =
                            cursors.overlay_rect(display, (width, height), scale, zoom)
                        {
                            rect = value;
                            shape = cursors.shape();
                        }
                    }
                }
            }
        }
        let mut dirty = false;
        if let Some(shape) = shape.filter(|shape| {
            self.shape
                .as_ref()
                .is_none_or(|old| !Arc::ptr_eq(old, shape))
        }) {
            let texture = Self::texture(device, shape.width as u32, shape.height as u32);
            device.queue.write_texture(
                texture.as_image_copy(),
                &shape.colors,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(shape.width as u32 * 4),
                    rows_per_image: Some(shape.height as u32),
                },
                texture.size(),
            );
            self.bind_group = Self::bind_group(
                device,
                &self.layout,
                &texture.create_view(&Default::default()),
                &self.uniform,
            );
            self.shape = Some(shape);
            dirty = true;
        }
        if self.rect != rect {
            let bytes: Vec<u8> = rect.into_iter().flat_map(f32::to_ne_bytes).collect();
            device.queue.write_buffer(&self.uniform, 0, &bytes);
            self.rect = rect;
            dirty = true;
        }
        dirty
    }

    fn texture(device: &DeviceHandle, width: u32, height: u32) -> wgpu::Texture {
        device.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RustDesk remote cursor"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn bind_group(
        device: &DeviceHandle,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        uniform: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RustDesk remote cursor"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform.as_entire_binding(),
                },
            ],
        })
    }
}
