struct Material {
    time: f32,
}

@group(1) @binding(0)
var<uniform> material: Material;

// Fragment shader for a simple animated plasma effect.
// Expects UV coordinates at @location(0).
@fragment
fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let t = material.time;
    let p = uv * 2.0 - vec2<f32>(1.0, 1.0);

    let r = 0.5 + 0.5 * sin(3.0 * p.x + t);
    let g = 0.5 + 0.5 * sin(3.0 * p.y + t * 1.3);
    let b = 0.5 + 0.5 * sin(3.0 * (p.x + p.y) + t * 0.7);

    return vec4<f32>(r, g, b, 1.0);
}