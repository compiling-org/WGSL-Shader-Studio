struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

fn opRep(p: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    return p - c * floor((p + 0.5 * c) / c);
}

fn box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn menger(p: vec3<f32>) -> f32 {
    var d = box(p, vec3<f32>(1.0));
    var s = 1.0;
    for (var i = 0i; i < 5; i = i + 1) {
        var a = opRep(p * s, vec3<f32>(2.0));
        s *= 3.0;
        var b = opRep(p * s, vec3<f32>(2.0));
        var c = opRep(p * s, vec3<f32>(2.0));
        let cross = max(box(vec3<f32>(a.x, b.y, c.z), vec3<f32>(1.0)),-box(vec3<f32>(a.x, b.y, c.z), vec3<f32>(1.0)));
        d = max(d, -cross);
    }
    return d;
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0i; i < 128; i = i + 1) {
        let p = ro + rd * t;
        let d = menger(p);
        if d < 0.001 { break; }
        t += d * 0.7;
        if t > 20.0 { break; }
    }
    return t;
}

fn normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.002;
    let d = menger(p);
    let n = vec3<f32>(
        menger(p + vec3<f32>(e, 0.0, 0.0)) - d,
        menger(p + vec3<f32>(0.0, e, 0.0)) - d,
        menger(p + vec3<f32>(0.0, 0.0, e)) - d,
    );
    return normalize(n);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = (position.xy / res) * 2.0 - 1.0;
    let aspect = res.x / res.y;
    let t = uniforms.time * 0.2;
    let ro = vec3<f32>(2.5 * sin(t), 1.6, 2.5 * cos(t));
    let target = vec3<f32>(0.0, 0.0, 0.0);
    let ww = normalize(target - ro);
    let uu = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), ww));
    let vv = cross(ww, uu);
    let rd = normalize(uv.x * uu * aspect + uv.y * vv - 1.8 * ww);

    let tHit = raymarch(ro, rd);
    var col = vec3<f32>(0.0);
    if tHit < 20.0 {
        let p = ro + rd * tHit;
        let n = normal(p);
        let l = normalize(vec3<f32>(0.4, 0.8, 0.2));
        let diff = max(dot(n, l), 0.0);
        let ao = clamp(menger(p + n * 0.02) * 50.0, 0.0, 1.0);
        col = vec3<f32>(0.9, 0.85, 0.8) * diff * ao;
    }
    col = pow(col, vec3<f32>(0.4545));
    return vec4<f32>(col, 1.0);
}