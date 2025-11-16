// Default WGSL shader for WGSL Shader Studio
// This is a simple gradient shader to test the system

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    return out;
}

struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<uniform> params: array<f32, 16>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = uniforms.time;
    
    // Simple gradient with time animation
    let r = sin(uv.x * 3.14159 + time) * 0.5 + 0.5;
    let g = sin(uv.y * 3.14159 + time * 0.7) * 0.5 + 0.5;
    let b = sin((uv.x + uv.y) * 3.14159 + time * 0.3) * 0.5 + 0.5;
    
    return vec4<f32>(r, g, b, 1.0);
}