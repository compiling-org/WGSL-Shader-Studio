struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = position.xy / res;
    let m = uniforms.mouse / res;
    let d = distance(uv, m);
    let ring = 1.0 - smoothstep(0.08, 0.1, d);
    let col = vec3<f32>(ring, 0.3 * ring, 0.8 * (1.0 - ring));
    return vec4<f32>(col, 1.0);
}