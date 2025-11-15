// Compute-to-texture pipeline example inspired by wgpu-compute-toy
// This shader writes to a texture using compute shader dispatch

struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    frame: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

// Noise function for procedural patterns
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    for (var i = 0; i < 5; i = i + 1) {
        value = value + amplitude * noise(p * frequency);
        frequency = frequency * 2.0;
        amplitude = amplitude * 0.5;
    }
    return value;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture_size = textureDimensions(output_texture);
    let coord = global_id.xy;
    
    // Bounds check
    if (coord.x >= texture_size.x || coord.y >= texture_size.y) {
        return;
    }
    
    // Convert to UV coordinates
    let uv = vec2<f32>(f32(coord.x), f32(coord.y)) / vec2<f32>(f32(texture_size.x), f32(texture_size.y));
    
    // Create animated pattern
    let time = uniforms.time;
    let mouse = uniforms.mouse / uniforms.resolution;
    
    // Multi-layer noise pattern
    var color = vec3<f32>(0.0);
    
    // Base layer
    let base_noise = fbm(uv * 4.0 + time * 0.1);
    color = color + vec3<f32>(base_noise) * 0.5;
    
    // Flow layer
    let flow_uv = uv + vec2<f32>(sin(time * 0.5 + uv.y * 3.0), cos(time * 0.3 + uv.x * 3.0)) * 0.1;
    let flow_noise = fbm(flow_uv * 8.0 + time * 0.2);
    color = color + vec3<f32>(flow_noise) * vec3<f32>(1.0, 0.7, 0.3) * 0.3;
    
    // Mouse interaction layer
    let mouse_dist = distance(uv, mouse);
    let mouse_influence = exp(-mouse_dist * 10.0) * sin(time * 3.0);
    color = color + vec3<f32>(mouse_influence) * vec3<f32>(0.8, 0.2, 1.0) * 0.2;
    
    // Frame-based variation
    let frame_pattern = sin(f32(uniforms.frame) * 0.01 + length(uv - 0.5) * 20.0);
    color = color + vec3<f32>(frame_pattern * 0.1);
    
    // Final color with contrast and saturation
    color = pow(color, vec3<f32>(0.8));
    color = mix(vec3<f32>(0.2), color, 1.2);
    
    // Write to texture
    textureStore(output_texture, coord, vec4<f32>(color, 1.0));
}