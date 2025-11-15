struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn map(p: vec3<f32>) -> f32 {
    // Two spheres orbiting
    let t = uniforms.time;
    let s1 = sd_sphere(p - vec3<f32>(0.6 * sin(t), 0.0, 0.6 * cos(t)), 0.4);
    let s2 = sd_sphere(p - vec3<f32>(-0.6 * cos(t * 0.8), 0.0, -0.6 * sin(t * 0.8)), 0.3);
    return min(s1, s2);
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    var i = 0i;
    loop {
        if i >= 64 { break; }
        let p = ro + rd * t;
        let d = map(p);
        if d < 0.001 { break; }
        t += d;
        i += 1;
        if t > 10.0 { break; }
    }
    return t;
}

fn get_normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.001;
    let d = map(p);
    let nx = map(p + vec3<f32>(e, 0.0, 0.0)) - d;
    let ny = map(p + vec3<f32>(0.0, e, 0.0)) - d;
    let nz = map(p + vec3<f32>(0.0, 0.0, e)) - d;
    return normalize(vec3<f32>(nx, ny, nz));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = (position.xy / res) * 2.0 - 1.0;
    let aspect = res.x / res.y;
    let ro = vec3<f32>(0.0, 0.0, 2.5);
    let rd = normalize(vec3<f32>(uv.x * aspect, uv.y, -1.6));

    let t = raymarch(ro, rd);
    let p = ro + rd * t;

    var col = vec3<f32>(0.0, 0.0, 0.0);
    if t < 10.0 {
        let n = get_normal(p);
        let l = normalize(vec3<f32>(0.8, 0.6, 0.3));
        let diff = max(dot(n, l), 0.0);
        let spec = pow(max(dot(reflect(-l, n), -rd), 0.0), 32.0);
        col = vec3<f32>(0.2, 0.3, 0.9) * diff + vec3<f32>(0.9, 0.9, 0.9) * spec;
    }

    return vec4<f32>(col, 1.0);
}