// Test shader with parameters for WGSL Shader Studio

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
    
    // Use parameters for customization
    let speed = params[0];        // Animation speed
    let intensity = params[1];    // Color intensity
    let offset = params[2];       // Phase offset
    let scale = params[3];        // Scale factor
    
    // Create animated pattern using parameters
    let wave = sin(uv.x * 10.0 * scale + time * speed + offset) * intensity;
    let pattern = sin(uv.y * 8.0 * scale + time * speed * 0.7) * intensity;
    
    let r = wave * 0.5 + 0.5;
    let g = pattern * 0.5 + 0.5;
    let b = (wave + pattern) * 0.25 + 0.5;
    
    return vec4<f32>(r, g, b, 1.0);
}