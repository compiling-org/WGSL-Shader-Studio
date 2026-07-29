// Particle Simulation Compute Shader
// Optimized for WGPU

struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    color: vec4<f32>,
    age: f32,
    lifetime: f32,
    size: f32,
    _pad: f32,
}

struct AudioFeatures {
    sub_bass: f32, bass: f32, low_mid: f32, mid: f32, upper_mid: f32, presence: f32, brilliance: f32,
    rms: f32, kick: f32, 
    centroid: f32, flux: f32, flatness: f32, rolloff: f32, bandwidth: f32, zcr: f32,
    onset: f32, beat: f32, beat_phase: f32, bpm: f32, beat_strength: f32,
}

struct Uniforms {
    delta_time: f32,
    time: f32,
    max_particles: u32,
    emit_count: u32,
    emitter_pos: vec2<f32>,
    emitter_radius: f32,
    emitter_shape: u32,
    _pad: f32,
    audio: AudioFeatures,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> particles_in: array<Particle>;
@group(0) @binding(2) var<storage, read_write> particles_out: array<Particle>;

fn hash(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453123);
}

fn random_vec2(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(hash(dot(p, vec2<f32>(127.1, 311.7))), hash(dot(p, vec2<f32>(269.5, 183.3))));
}

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= uniforms.max_particles) { return; }

    var p = particles_in[idx];
    
    if (p.age >= p.lifetime) {
        // Recycle particle
        let spawn_idx = uniforms.max_particles - 1u - idx; // Spread out spawns
        if (id.x < uniforms.emit_count) {
            let seed = uniforms.time + f32(idx);
            let rand = random_vec2(vec2<f32>(seed, seed * 1.618));
            
            p.position = uniforms.emitter_pos + (rand - 0.5) * uniforms.emitter_radius * uniforms.audio.kick;
            p.velocity = (rand - 0.5) * 2.0 * (1.0 + uniforms.audio.rms);
            p.age = 0.0;
            p.lifetime = 1.0 + hash(seed * 2.0) * 4.0;
            p.size = 0.01 + hash(seed * 3.0) * 0.05 * uniforms.audio.presence;
            
            // Color based on audio bands
            p.color = vec4<f32>(
                uniforms.audio.sub_bass,
                uniforms.audio.mid,
                uniforms.audio.brilliance,
                1.0
            );
        } else {
            p.age = p.lifetime; // Keep dead
        }
    } else {
        // Update existing particle
        p.age += uniforms.delta_time;
        
        // Physics
        let gravity = vec2<f32>(0.0, -9.8) * uniforms.audio.sub_bass;
        p.velocity += gravity * uniforms.delta_time;
        p.position += p.velocity * uniforms.delta_time;
        
        // Audio reactivity: expand size on beat
        let size_mult = 1.0 + uniforms.audio.beat * 0.5;
        p.size *= (1.0 - uniforms.delta_time); // Shrink over time
        
        // Fade out
        p.color.a = 1.0 - (p.age / p.lifetime);
    }

    particles_out[idx] = p;
}
