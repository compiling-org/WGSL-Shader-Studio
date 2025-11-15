struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var volume_tex: texture_3d<f32>;
@group(1) @binding(1) var volume_sampler: sampler;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = position.xy / res;
    // Animate z slice over time
    let z = fract(uniforms.time * 0.1);
    let sample_pos = vec3<f32>(uv, z);
    let color = textureSample(volume_tex, volume_sampler, sample_pos);
    return color;
}