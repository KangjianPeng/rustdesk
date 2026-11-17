use crate::dioxus_ui::session::FramePacket;
use glow::HasContext;
use hbb_common::anyhow::{anyhow, bail, Result};
use scrap::ImageFormat;
use std::ffi::CString;

pub(super) struct Gpu {
    gl: glow::Context,
    program: glow::Program,
    texture: glow::Texture,
    vertices: glow::Buffer,
    array: glow::VertexArray,
    dimensions: (usize, usize, u32),
    maximum: usize,
    packed: Vec<u8>,
    crop: Option<glow::UniformLocation>,
    sampler: Option<glow::UniformLocation>,
}

impl Gpu {
    pub(super) unsafe fn new() -> Result<Self> {
        let gl = glow::Context::from_loader_function(|name| {
            let Ok(name) = CString::new(format!("epoxy_{name}")) else {
                return std::ptr::null();
            };
            let symbol = hbb_common::libc::dlsym(hbb_common::libc::RTLD_DEFAULT, name.as_ptr());
            if symbol.is_null() {
                std::ptr::null()
            } else {
                // libepoxy exports dispatch pointer variables, including for EGL contexts.
                *(symbol as *const *const std::ffi::c_void)
            }
        });
        let program = gl.create_program().map_err(|e| anyhow!(e))?;
        for (kind, source) in [
            (glow::VERTEX_SHADER, "#version 150\nin vec2 p; out vec2 uv; uniform vec4 crop; void main(){uv=crop.xy+p*crop.zw;gl_Position=vec4(p.x*2.-1.,1.-p.y*2.,0.,1.);}"),
            (glow::FRAGMENT_SHADER, "#version 150\nin vec2 uv; out vec4 color; uniform sampler2D frame; void main(){color=vec4(texture(frame,uv).rgb,1.);}"),
        ] {
            let shader = gl.create_shader(kind).map_err(|e| anyhow!(e))?;
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                let error = gl.get_shader_info_log(shader);
                gl.delete_shader(shader);
                gl.delete_program(program);
                bail!("Native video shader: {error}");
            }
            gl.attach_shader(program, shader);
            gl.delete_shader(shader);
        }
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            let error = gl.get_program_info_log(program);
            gl.delete_program(program);
            bail!("Native video program: {error}");
        }
        let texture = gl.create_texture().map_err(|e| anyhow!(e))?;
        let array = gl.create_vertex_array().map_err(|e| anyhow!(e))?;
        let vertices = gl.create_buffer().map_err(|e| anyhow!(e))?;
        gl.bind_vertex_array(Some(array));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertices));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&[0.0f32, 0., 1., 0., 0., 1., 1., 1.]),
            glow::STATIC_DRAW,
        );
        let attribute = gl
            .get_attrib_location(program, "p")
            .ok_or_else(|| anyhow!("Missing video vertex attribute"))?;
        gl.enable_vertex_attrib_array(attribute);
        gl.vertex_attrib_pointer_f32(attribute, 2, glow::FLOAT, false, 0, 0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        for parameter in [glow::TEXTURE_MIN_FILTER, glow::TEXTURE_MAG_FILTER] {
            gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::LINEAR as i32);
        }
        for parameter in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T] {
            gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::CLAMP_TO_EDGE as i32);
        }
        let maximum = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE).max(0) as usize;
        let crop = gl.get_uniform_location(program, "crop");
        let sampler = gl.get_uniform_location(program, "frame");
        hbb_common::log::info!(
            "Native video renderer: {} / {}",
            gl.get_parameter_string(glow::VENDOR),
            gl.get_parameter_string(glow::RENDERER)
        );
        Ok(Self {
            gl,
            program,
            texture,
            vertices,
            array,
            dimensions: (0, 0, 0),
            maximum,
            packed: vec![],
            crop,
            sampler,
        })
    }

    pub(super) unsafe fn draw(
        &mut self,
        packet: Option<&FramePacket>,
        size: (i32, i32),
        viewport: &super::Viewport,
    ) -> Result<()> {
        let gl = &self.gl;
        gl.disable(glow::DEPTH_TEST);
        gl.disable(glow::SCISSOR_TEST);
        gl.disable(glow::BLEND);
        gl.clear_color(9. / 255., 10. / 255., 11. / 255., 1.);
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.array));
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
        if let Some(packet) = packet {
            let Some(stride) = packet.stride() else {
                bail!("Invalid native video frame");
            };
            if packet.width > self.maximum || packet.height > self.maximum {
                bail!("Native video frame exceeds texture size {}", self.maximum);
            }
            let bpp = if matches!(packet.format, ImageFormat::Raw) {
                3
            } else {
                4
            };
            let format = match packet.format {
                ImageFormat::Raw => glow::RGB,
                ImageFormat::ARGB => glow::BGRA,
                ImageFormat::ABGR => glow::RGBA,
            };
            let internal = if bpp == 3 { glow::RGB8 } else { glow::RGBA8 };
            if self.dimensions != (packet.width, packet.height, internal) {
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    internal as i32,
                    packet.width as i32,
                    packet.height as i32,
                    0,
                    if bpp == 3 { glow::RGB } else { glow::RGBA },
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(None),
                );
            }
            let pixels = if stride % bpp == 0 {
                &packet.raw[..stride * packet.height]
            } else {
                self.packed.resize(packet.width * packet.height * bpp, 0);
                for (source, target) in packet
                    .raw
                    .chunks_exact(stride)
                    .zip(self.packed.chunks_exact_mut(packet.width * bpp))
                {
                    target.copy_from_slice(&source[..target.len()]);
                }
                &self.packed
            };
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            gl.pixel_store_i32(
                glow::UNPACK_ROW_LENGTH,
                if stride % bpp == 0 {
                    (stride / bpp) as i32
                } else {
                    0
                },
            );
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                packet.width as i32,
                packet.height as i32,
                format,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );
            gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, 0);
            self.dimensions = (packet.width, packet.height, internal);
        }
        if self.dimensions.0 > 0 {
            if let Some((rect, crop)) = viewport.project(size) {
                gl.viewport(rect[0], rect[1], rect[2], rect[3]);
                gl.uniform_4_f32(self.crop.as_ref(), crop[0], crop[1], crop[2], crop[3]);
                gl.uniform_1_i32(self.sampler.as_ref(), 0);
                gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            }
        }
        Ok(())
    }

    pub(super) unsafe fn destroy(self) {
        self.gl.delete_texture(self.texture);
        self.gl.delete_buffer(self.vertices);
        self.gl.delete_vertex_array(self.array);
        self.gl.delete_program(self.program);
    }
}
