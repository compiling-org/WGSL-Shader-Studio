// Particle simulation compute shader inspired by wgpu-compute-toy
// Simulates particle physics and writes results to storage buffers

struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    color: vec3<f32>,
    life: f32,
    size: f32,
}

struct Uniforms {
    time: f32,
    delta_time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    particle_count: u32,
    gravity: f32,
    damping: f32,
}

struct Attractor {
    position: vec2<f32>,
    strength: f32,
    radius: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<storage, read> attractors: array<Attractor>;
@group(0) @binding(3) var attractor_count: u32;

fn rand(seed: f32) -> f32 {
    return fract(sin(seed * 12.9898) * 43758.5453);
}

fn rand2(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn length_squared(v: vec2<f32>) -> f32 {
    return dot(v, v);
}

@compute @workgroup_size(64, 1, 1)
fn simulate_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= uniforms.particle_count) {
        return;
    }
    
    var particle = particles[index];
    
    // Update life
    particle.life = particle.life - uniforms.delta_time * 0.5;
    
    // Respawn dead particles
    if (particle.life <= 0.0) {
        // Random spawn position around mouse
        let spawn_radius = 50.0;
        let spawn_angle = rand(f32(index) + uniforms.time) * 6.28318;
        particle.position = uniforms.mouse + vec2<f32>(cos(spawn_angle), sin(spawn_angle)) * spawn_radius;
        
        // Random initial velocity
        let velocity_angle = rand(f32(index) * 2.0 + uniforms.time) * 6.28318;
        let velocity_magnitude = rand(f32(index) * 3.0 + uniforms.time) * 100.0 + 50.0;
        particle.velocity = vec2<f32>(cos(velocity_angle), sin(velocity_angle)) * velocity_magnitude;
        
        // Random color based on spawn time
        let hue = fract(uniforms.time * 0.1 + rand(f32(index) * 4.0));
        particle.color = hsv2rgb(vec3<f32>(hue, 0.8, 1.0));
        
        particle.life = 1.0;
        particle.size = rand(f32(index) * 5.0 + uniforms.time) * 3.0 + 2.0;
    }
    
    // Apply forces
    var acceleration = vec2<f32>(0.0, uniforms.gravity);
    
    // Mouse attraction/repulsion
    let mouse_diff = uniforms.mouse - particle.position;
    let mouse_dist_sq = length_squared(mouse_diff);
    if (mouse_dist_sq > 1.0) {
        let mouse_dist = sqrt(mouse_dist_sq);
        let mouse_force = mouse_diff / mouse_dist * 5000.0 / mouse_dist_sq;
        acceleration = acceleration + mouse_force;
    }
    
    // Attractor forces
    for (var i = 0u; i < attractor_count; i = i + 1u) {
        let attractor = attractors[i];
        let diff = attractor.position - particle.position;
        let dist_sq = length_squared(diff);
        
        if (dist_sq > attractor.radius * attractor.radius) {
            let dist = sqrt(dist_sq);
            let force = diff / dist * attractor.strength / dist_sq;
            acceleration = acceleration + force;
        }
    }
    
    // Boundary forces (keep particles on screen)
    let margin = 50.0;
    if (particle.position.x < margin) {
        acceleration.x = acceleration.x + (margin - particle.position.x) * 2.0;
    }
    if (particle.position.x > uniforms.resolution.x - margin) {
        acceleration.x = acceleration.x - (particle.position.x - (uniforms.resolution.x - margin)) * 2.0;
    }
    if (particle.position.y < margin) {
        acceleration.y = acceleration.y + (margin - particle.position.y) * 2.0;
    }
    if (particle.position.y > uniforms.resolution.y - margin) {
        acceleration.y = acceleration.y - (particle.position.y - (uniforms.resolution.y - margin)) * 2.0;
    }
    
    // Update velocity and position
    particle.velocity = particle.velocity + acceleration * uniforms.delta_time;
    particle.velocity = particle.velocity * uniforms.damping;
    particle.position = particle.position + particle.velocity * uniforms.delta_time;
    
    // Store updated particle
    particles[index] = particle;
}

// Helper function to convert HSV to RGB
fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}

// Particle rendering compute shader (separate dispatch)
@compute @workgroup_size(8, 8, 1)
fn render_particles_to_texture(@builtin(global_invocation_id) global_id: vec3<u32>,
                                @group(0) @binding(4) var output_texture: texture_storage_2d<rgba8unorm, write>) {
    let coord = global_id.xy;
    let texture_size = textureDimensions(output_texture);
    
    if (coord.x >= texture_size.x || coord.y >= texture_size.y) {
        return;
    }
    
    let uv = vec2<f32>(f32(coord.x), f32(coord.y));
    var color = vec3<f32>(0.0, 0.0, 0.05); // Dark blue background
    
    // Render particles
    for (var i = 0u; i < uniforms.particle_count; i = i + 1u) {
        let particle = particles[i];
        let particle_screen = particle.position;
        
        let dist = distance(uv, particle_screen);
        if (dist < particle.size) {
            let alpha = 1.0 - (dist / particle.size);
            let particle_color = particle.color * particle.life;
            color = mix(color, particle_color, alpha * 0.8);
        }
    }
    
    // Add some background pattern
    let pattern = sin(uv.x * 0.02 + uniforms.time * 0.5) * cos(uv.y * 0.02 + uniforms.time * 0.3);
    color = color + vec3<f32>(pattern * 0.02);
    
    textureStore(output_texture, coord, vec4<f32>(color, 1.0));
}