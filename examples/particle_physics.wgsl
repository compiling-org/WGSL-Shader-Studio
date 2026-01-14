// Particle Physics Simulation inspired by Party Library (https://github.com/cazala/party)
//
// This WGSL compute shader demonstrates particle physics concepts similar to those
// in the Party library, including position/velocity integration, basic forces,
// and boundary constraints.

struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    mass: f32,
    size: f32,
};

struct ParticleData {
    particles: array<Particle>,
};

struct Globals {
    deltaTime: f32,
    time: f32,
    canvasSize: vec2<f32>,
    mousePos: vec2<f32>,
    mouseActive: f32,
};

@group(0) @binding(0)
var<storage, read_write> particleBuffer: ParticleData;

@group(0) @binding(1)
var<uniform> globals: Globals;

@compute @workgroup_size(64)
fn simulateParticles(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= arrayLength(&particleBuffer.particles)) {
        return;
    }

    var particle = particleBuffer.particles[idx];

    // Apply environmental forces (gravity, friction)
    let gravity: vec2<f32> = vec2<f32>(0.0, 98.0); // Downward gravity
    let friction: f32 = 0.98; // Velocity damping
    
    // Apply gravity based on mass
    particle.velocity += gravity * particle.mass * globals.deltaTime;
    
    // Apply friction
    particle.velocity *= friction;

    // Mouse interaction (attraction/repulsion)
    let mouseInfluenceRadius: f32 = 100.0;
    let mouseVec = particle.position - globals.mousePos;
    let mouseDist = length(mouseVec);
    
    if (mouseDist < mouseInfluenceRadius && mouseDist > 0.001) {
        let influenceStrength = (1.0 - mouseDist / mouseInfluenceRadius) * 10.0;
        let direction = normalize(mouseVec);
        
        // Repulsion when close, attraction when slightly further
        if (mouseDist < mouseInfluenceRadius * 0.3) {
            particle.velocity += direction * influenceStrength * globals.mouseActive;
        } else {
            particle.velocity -= direction * influenceStrength * 0.5 * globals.mouseActive;
        }
    }

    // Update position with velocity
    particle.position += particle.velocity * globals.deltaTime;

    // Boundary constraints (bounce off edges)
    let margin: f32 = particle.size;
    if (particle.position.x < margin) {
        particle.position.x = margin;
        particle.velocity.x *= -0.8; // Bounce with energy loss
    }
    if (particle.position.x > globals.canvasSize.x - margin) {
        particle.position.x = globals.canvasSize.x - margin;
        particle.velocity.x *= -0.8;
    }
    if (particle.position.y < margin) {
        particle.position.y = margin;
        particle.velocity.y *= -0.8;
    }
    if (particle.position.y > globals.canvasSize.y - margin) {
        particle.position.y = globals.canvasSize.y - margin;
        particle.velocity.y *= -0.8;
    }

    // Store updated particle
    particleBuffer.particles[idx] = particle;
}

// Fragment shader for rendering particles
@fragment
fn fragmentMain(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.5, 0.2, 1.0); // Orange particles
}