struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = position.xy / res;
    let zoom = 1.5 + 0.5 * sin(uniforms.time * 0.2);
    let center = vec2<f32>(-0.75, 0.0);
    let c = (uv * 2.0 - 1.0) / zoom + center;
    var z = vec2<f32>(0.0, 0.0);
    var i = 0i;
    var m = 0.0;
    loop {
        if i >= 120 { break; }
        let z2 = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        z = z2;
        if dot(z, z) > 4.0 { m = f32(i) / 120.0; break; }
        i += 1;
    }
    let col = vec3<f32>(0.5 + 0.5 * cos(6.28318 * m + vec3<f32>(0.0, 2.0, 4.0)));
    return vec4<f32>(col, 1.0);
}