// Particle Render Shader
// Renders particles as instanced quads

struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    color: vec4<f32>,
    age: f32,
    lifetime: f32,
    size: f32,
    _pad: f32,
}

@group(0) @binding(1) var<storage, read> particles: array<Particle>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @builtin(instance_index) i_idx: u32,
) -> VertexOutput {
    let p = particles[i_idx];
    
    if (p.age >= p.lifetime) {
        return VertexOutput(vec4<f32>(0.0), vec4<f32>(0.0), vec2<f32>(0.0));
    }

    let quad = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0)
    );
    
    let pos = quad[v_idx];
    let world_pos = p.position + pos * p.size;
    
    var out: VertexOutput;
    out.position = vec4<f32>(world_pos, 0.0, 1.0);
    out.color = p.color;
    out.uv = pos * 0.5 + 0.5;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv - 0.5);
    if (dist > 0.5) { discard; }
    
    // Soft radial glow
    let alpha = smoothstep(0.5, 0.0, dist) * in.color.a;
    return vec4<f32>(in.color.rgb, alpha);
}
