struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    audio_volume: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_treble: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn sd_sphere(p: vec3<f32>, r: f32) -> f32 { return length(p) - r; }
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}
fn op_union(a: f32, b: f32) -> f32 { return min(a, b); }
fn op_smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5*(b - a)/k, 0.0, 1.0);
    return mix(b, a, h) - k*h*(1.0 - h);
}
fn map(p: vec3<f32>) -> f32 {
    let s = sd_sphere(p - vec3<f32>(0.0, 0.0, 3.0), 1.0 + 0.2*sin(uniforms.time));
    let b = sd_box(p - vec3<f32>(1.2*sin(uniforms.time*0.7), 0.5, 3.0), vec3<f32>(0.6, 0.6, 0.6));
    return op_smooth_union(s, b, 0.4);
}
fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.0005;
    let n = vec3<f32>(
        map(p + vec3<f32>(e, 0.0, 0.0)) - map(p - vec3<f32>(e, 0.0, 0.0)),
        map(p + vec3<f32>(0.0, e, 0.0)) - map(p - vec3<f32>(0.0, e, 0.0)),
        map(p + vec3<f32>(0.0, 0.0, e)) - map(p - vec3<f32>(0.0, 0.0, e))
    );
    return normalize(n);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var pos = vec2<f32>(0.0, 0.0);
    switch vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>(3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0, 3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = (position.xy - 0.5*res) / res.y;
    let ro = vec3<f32>(0.0, 0.0, -3.0);
    let rd = normalize(vec3<f32>(uv, 1.5));
    var t = 0.0;
    var hit = false;
    for (var i = 0; i < 128; i = i + 1) {
        let p = ro + rd * t;
        let d = map(p);
        if (d < 0.001) { hit = true; break; }
        t = t + d;
        if (t > 20.0) { break; }
    }
    var col = vec3<f32>(0.0);
    if (hit) {
        let p = ro + rd * t;
        let n = calc_normal(p);
        let l = normalize(vec3<f32>(-0.4, 0.6, 0.8));
        let diff = max(dot(n, l), 0.0);
        let h = normalize(l - rd);
        let spec = pow(max(dot(n, h), 0.0), 32.0);
        col = vec3<f32>(0.2, 0.5, 0.9) * diff + vec3<f32>(spec);
    } else {
        col = vec3<f32>(0.02, 0.03, 0.05) + 0.1*vec3<f32>(uv.x + 0.5, uv.y + 0.5, 1.0);
    }
    return vec4<f32>(col, 1.0);
}
