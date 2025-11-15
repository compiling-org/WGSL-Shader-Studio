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

fn noise(p: vec3<f32>) -> f32 {
    let q = sin(vec3<f32>(dot(p, vec3<f32>(127.1, 311.7, 74.7)),
                          dot(p, vec3<f32>(269.5, 183.3, 246.1)),
                          dot(p, vec3<f32>(113.5, 271.9, 124.6))));
    return fract(q.x * q.y * q.z * 43758.5453);
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
    let ro = vec3<f32>(0.0, 0.0, -2.0);
    let rd = normalize(vec3<f32>(uv, 1.3));
    var t = 0.0;
    var acc = 0.0;
    for (var i = 0; i < 96; i = i + 1) {
        let p = ro + rd * t;
        let d = noise(p*2.0 + vec3<f32>(0.0, uniforms.time*0.2, 0.0));
        acc = acc + d*0.02;
        t = t + 0.03;
        if (t > 6.0) { break; }
    }
    let col = vec3<f32>(acc) * vec3<f32>(0.8, 0.9, 1.0);
    return vec4<f32>(col, 1.0);
}
