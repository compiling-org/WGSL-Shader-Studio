// Tonemapping utilities
// Adapted from Phosphor / ACES standards

fn tonemap_aces(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn tonemap_reinhard(x: vec3<f32>) -> vec3<f32> {
    return x / (1.0 + x);
}

fn linear_to_srgb(x: vec3<f32>) -> vec3<f32> {
    return pow(x, vec3<f32>(1.0 / 2.2));
}

fn srgb_to_linear(x: vec3<f32>) -> vec3<f32> {
    return pow(x, vec3<f32>(2.2));
}
