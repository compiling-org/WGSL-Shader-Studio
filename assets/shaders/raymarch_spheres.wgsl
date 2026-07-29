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
fn map(p: vec3<f32>) -> f32 {
    let s1 = sd_sphere(p - vec3<f32>(sin(uniforms.time), 0.0, 3.0), 0.6);
    let s2 = sd_sphere(p - vec3<f32>(-1.0, 0.5*sin(uniforms.time*1.3), 4.0), 0.8);
    return min(s1, s2);
}
fn normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.0005;
    return normalize(vec3<f32>(
        map(p + vec3<f32>(e,0.0,0.0)) - map(p - vec3<f32>(e,0.0,0.0)),
        map(p + vec3<f32>(0.0,e,0.0)) - map(p - vec3<f32>(0.0,e,0.0)),
        map(p + vec3<f32>(0.0,0.0,e)) - map(p - vec3<f32>(0.0,0.0,e))
    ));
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
    let ro = vec3<f32>(0.0, 0.0, -3.5);
    let rd = normalize(vec3<f32>(uv, 1.7));
    var t = 0.0;
    var col = vec3<f32>(0.0);
    for (var i = 0; i < 128; i = i + 1) {
        let p = ro + rd*t;
        let d = map(p);
        if (d < 0.001) {
            let n = normal(p);
            let l = normalize(vec3<f32>(0.7, 0.6, 0.4));
            let diff = max(dot(n, l), 0.0);
            let h = normalize(l - rd);
            let spec = pow(max(dot(n, h), 0.0), 24.0);
            col = vec3<f32>(0.3, 0.6, 0.9)*diff + vec3<f32>(spec);
            break;
        }
        t = t + d;
        if (t > 25.0) { break; }
    }
    col = col + vec3<f32>(0.02, 0.02, 0.03);
    return vec4<f32>(col, 1.0);
}
