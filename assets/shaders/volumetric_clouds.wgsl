struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

fn hash3(p: vec3<f32>) -> f32 {
    let h = dot(p, vec3<f32>(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash3(i);
    let b = hash3(i + vec3<f32>(1.0, 0.0, 0.0));
    let c = hash3(i + vec3<f32>(0.0, 1.0, 0.0));
    let d = hash3(i + vec3<f32>(1.0, 1.0, 0.0));
    let e = hash3(i + vec3<f32>(0.0, 0.0, 1.0));
    let g = hash3(i + vec3<f32>(1.0, 0.0, 1.0));
    let h = hash3(i + vec3<f32>(0.0, 1.0, 1.0));
    let k = hash3(i + vec3<f32>(1.0, 1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    let mix_xy = mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
    let mix_zw = mix(e, g, u.x) + (h - e) * u.y * (1.0 - u.x) + (k - g) * u.x * u.y;
    return mix(mix_xy, mix_zw, u.z);
}

fn fbm3(p: vec3<f32>) -> f32 {
    var f = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    for (var i = 0i; i < 6; i = i + 1) {
        f += amp * noise3(p * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return f;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = (position.xy / res) * 2.0 - 1.0;
    let aspect = res.x / res.y;
    let ro = vec3<f32>(0.0, 0.0, 3.0);
    let rd = normalize(vec3<f32>(uv.x * aspect, uv.y, -1.6));
    var t = 0.0;
    var col = vec3<f32>(0.6, 0.7, 0.8);
    for (var i = 0i; i < 64; i = i + 1) {
        let p = ro + rd * t;
        let d = fbm3(p * 0.6 + vec3<f32>(uniforms.time * 0.05, 0.0, 0.0)) - 0.5;
        let c = clamp(d * 2.0 + 0.3, 0.0, 1.0);
        col = col * (1.0 - c) + vec3<f32>(1.0) * c;
        t += 0.05;
        if t > 6.0 { break; }
    }
    return vec4<f32>(pow(col, vec3<f32>(0.4545)), 1.0);
}