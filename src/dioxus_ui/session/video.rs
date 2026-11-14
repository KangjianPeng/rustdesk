use dioxus::native::{CustomPaintCtx, DeviceHandle, TextureHandle};

pub(super) struct VideoTexture {
    device: DeviceHandle,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    source: Option<(wgpu::Texture, wgpu::BindGroup)>,
    output: Option<(wgpu::Texture, TextureHandle)>,
    dirty: bool,
    cursor: super::cursor_texture::CursorTexture,
}

impl VideoTexture {
    pub(super) fn new(device: DeviceHandle) -> Self {
        let shader = device
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RustDesk video scale"),
                source: wgpu::ShaderSource::Wgsl(include_str!("video.wgsl").into()),
            });
        let pipeline = device
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("RustDesk video scale"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });
        let sampler = device.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("RustDesk video scale"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let cursor =
            super::cursor_texture::CursorTexture::new(&device, pipeline.get_bind_group_layout(1));
        Self {
            device,
            pipeline,
            sampler,
            source: None,
            output: None,
            dirty: false,
            cursor,
        }
    }

    pub(super) fn upload(&mut self, width: u32, height: u32, stride: u32, pixels: &[u8]) {
        if self
            .source
            .as_ref()
            .is_none_or(|(texture, _)| texture.width() != width || texture.height() != height)
        {
            let texture = self.texture(
                width,
                height,
                wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            );
            let view = texture.create_view(&Default::default());
            let bind_group = self
                .device
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("RustDesk video scale"),
                    layout: &self.pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                });
            self.source = Some((texture, bind_group));
        }
        if let Some((texture, _)) = &self.source {
            self.device.queue.write_texture(
                texture.as_image_copy(),
                pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            self.dirty = true;
        }
    }

    pub(super) fn render(
        &mut self,
        ctx: &mut CustomPaintCtx<'_>,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Option<TextureHandle> {
        if width == 0 || height == 0 || self.source.is_none() {
            return None;
        }
        let maximum = self.device.device.limits().max_texture_dimension_2d;
        if width > maximum || height > maximum {
            hbb_common::throttled_log!(
                std::time::Duration::from_secs(30),
                warn,
                "Native video viewport {width}x{height} exceeds texture limit {maximum}"
            );
            return None;
        }
        self.dirty |= self.cursor.update(&self.device, width, height, scale);
        if self
            .output
            .as_ref()
            .is_none_or(|(texture, _)| texture.width() != width || texture.height() != height)
        {
            if let Some((_, handle)) = self.output.take() {
                ctx.unregister_texture(handle);
            }
            let texture = self.texture(
                width,
                height,
                wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            let handle = ctx.register_texture(texture.clone());
            self.output = Some((texture, handle));
            self.dirty = true;
        }
        let (output, handle) = self.output.as_ref()?;
        if self.dirty {
            let (_, bind_group) = self.source.as_ref()?;
            let view = output.create_view(&Default::default());
            let mut encoder =
                self.device
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("RustDesk video scale"),
                    });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("RustDesk video scale"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_bind_group(1, &self.cursor.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
            self.device.queue.submit([encoder.finish()]);
            self.dirty = false;
            hbb_common::log::trace!("Native video submitted: {width}x{height}");
        }
        Some(handle.clone())
    }

    fn texture(&self, width: u32, height: u32, usage: wgpu::TextureUsages) -> wgpu::Texture {
        self.device.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RustDesk remote video"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage,
            view_formats: &[],
        })
    }
}
