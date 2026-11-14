@group(0) @binding(0) var video: texture_2d<f32>;
@group(0) @binding(1) var video_sampler: sampler;
@group(1) @binding(0) var cursor_image: texture_2d<f32>;
@group(1) @binding(1) var<uniform> cursor_rect: vec4<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return VertexOutput(vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0), uv);
}

@fragment fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(video, video_sampler, input.uv).rgb;
    if cursor_rect.z > 0.0 && cursor_rect.w > 0.0 {
        let uv = (input.uv - cursor_rect.xy) / cursor_rect.zw;
        if all(uv >= vec2<f32>(0.0)) && all(uv < vec2<f32>(1.0)) {
            let cursor = textureSampleLevel(cursor_image, video_sampler, uv, 0.0);
            return vec4<f32>(mix(color, cursor.rgb, cursor.a), 1.0);
        }
    }
    return vec4<f32>(color, 1.0);
}
