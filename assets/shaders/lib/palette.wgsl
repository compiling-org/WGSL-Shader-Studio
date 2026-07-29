// Cosine-based palettes
// Adapted from: https://github.com/kevinraymond/phosphor/blob/main/assets/shaders/lib/palette.wgsl
// See: https://iquilezles.org/articles/palettes/

fn palette(t: f32, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    return a + b * cos(6.28318 * (c * t + d));
}

// Common palettes
fn palette_rainbow(t: f32) -> vec3<f32> {
    return palette(t, vec3<f32>(0.5), vec3<f32>(0.5), vec3<f32>(1.0), vec3<f32>(0.0, 0.33, 0.67));
}

fn palette_fire(t: f32) -> vec3<f32> {
    return palette(t, vec3<f32>(0.5), vec3<f32>(0.5), vec3<f32>(1.0), vec3<f32>(0.0, 0.1, 0.2));
}

fn palette_ocean(t: f32) -> vec3<f32> {
    return palette(t, vec3<f32>(0.5), vec3<f32>(0.5), vec3<f32>(1.0), vec3<f32>(0.3, 0.2, 0.2));
}

fn palette_neon(t: f32) -> vec3<f32> {
    return palette(t, vec3<f32>(0.5), vec3<f32>(0.5), vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(0.5, 0.2, 0.25));
}
