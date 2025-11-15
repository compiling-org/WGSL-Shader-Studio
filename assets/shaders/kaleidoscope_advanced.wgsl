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

fn palette(t: f32) -> vec3<f32> {
    return vec3<f32>(0.5 + 0.5*sin(6.28318*(t + 0.00)),
                     0.5 + 0.5*sin(6.28318*(t + 0.33)),
                     0.5 + 0.5*sin(6.28318*(t + 0.67)));
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
    var uv = (position.xy - 0.5*res) / res.y;
    let t = uniforms.time*0.4;
    let r = length(uv);
    let a = atan2(uv.y, uv.x);
    let segs = 8.0;
    let k = 6.28318/segs;
    let m = abs(fract((a + t)*segs/k) * 2.0 - 1.0);
    let p = vec2<f32>(cos(m*k), sin(m*k)) * r;
    let v = sin(10.0*p.x) + cos(10.0*p.y);
    let col = palette(0.5 + 0.5*v);
    return vec4<f32>(col, 1.0);
}
