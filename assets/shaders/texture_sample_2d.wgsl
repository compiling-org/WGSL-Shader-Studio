struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var input_texture: texture_2d<f32>;
@group(1) @binding(1) var input_sampler: sampler;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = position.xy / res;
    let color = textureSample(input_texture, input_sampler, uv);
    return color;
}