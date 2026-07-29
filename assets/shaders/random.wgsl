// Random function library for WGSL

fn random(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn random_range(seed: f32, min_val: f32, max_val: f32) -> f32 {
    return min_val + random(vec2<f32>(seed, seed * 2.0)) * (max_val - min_val);
}

fn fract(x: f32) -> f32 {
    return x - floor(x);
}