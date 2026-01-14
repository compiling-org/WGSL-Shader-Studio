//! Particle Physics Module inspired by Party Library
//! This module provides basic particle physics functionality that can be used with WGSL shaders
//! based on concepts from the Party library (https://github.com/cazala/party)

use bevy::prelude::*;
use rand::Rng;

/// Component for entities that represent individual particles
#[derive(Component, Clone)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub mass: f32,
    pub size: f32,
    pub color: Color,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            lifetime: 0.0,
            max_lifetime: 5.0,
            mass: 1.0,
            size: 1.0,
            color: Color::WHITE,
        }
    }
}

/// Component for entities that represent a particle system
#[derive(Component)]
pub struct ParticleSystem {
    pub spawn_rate: f32,
    pub last_spawn: f32,
    pub max_particles: usize,
    pub active_particles: usize,
    pub emitter_position: Vec3,
    pub emitter_velocity: Vec3,
    pub spawn_shape: SpawnShape,
    pub spawn_params: SpawnParams,
}

#[derive(Clone)]
pub enum SpawnShape {
    Point,
    Sphere { radius: f32 },
    Box { size: Vec3 },
    Cone { angle: f32, height: f32 },
}

#[derive(Clone)]
pub struct SpawnParams {
    pub initial_velocity: Vec3,
    pub velocity_spread: f32,
    pub size_range: (f32, f32),
    pub color_range: (Color, Color),
}

impl Default for SpawnParams {
    fn default() -> Self {
        Self {
            initial_velocity: Vec3::new(0.0, 5.0, 0.0), // Upward
            velocity_spread: 0.5,
            size_range: (0.5, 1.5),
            color_range: (Color::WHITE, Color::WHITE),
        }
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self {
            spawn_rate: 10.0, // Particles per second
            last_spawn: 0.0,
            max_particles: 1000,
            active_particles: 0,
            emitter_position: Vec3::ZERO,
            emitter_velocity: Vec3::ZERO,
            spawn_shape: SpawnShape::Point,
            spawn_params: SpawnParams::default(),
        }
    }
}

/// Resource to manage global particle system settings
#[derive(Resource, Default)]
pub struct ParticlePhysicsSettings {
    pub gravity: Vec3,
    pub global_damping: f32,
    pub time_scale: f32,
}

/// Plugin to add particle physics functionality
pub struct ParticlePhysicsPlugin;

impl Plugin for ParticlePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ParticlePhysicsSettings>()
            .add_systems(Update, (update_particles, spawn_particles));
    }
}

/// System to update particle physics
fn update_particles(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle)>,
    settings: Res<ParticlePhysicsSettings>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs() * settings.time_scale;
    
    for (entity, mut particle) in particles.iter_mut() {
        // Apply gravity
        particle.velocity += settings.gravity * delta_time;
        
        // Store velocity for position update
        let velocity = particle.velocity;
        
        // Apply damping
        particle.velocity *= settings.global_damping;
        
        // Update position
        particle.position += velocity * delta_time;
        
        // Update lifetime
        particle.lifetime += delta_time;
        
        // Despawn particles that exceed their lifetime
        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// System to spawn new particles
fn spawn_particles(
    mut commands: Commands,
    mut particle_systems: Query<(Entity, &mut ParticleSystem)>,
    settings: Res<ParticlePhysicsSettings>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    
    for (entity, mut system) in particle_systems.iter_mut() {
        system.last_spawn += delta_time;
        let spawn_count = (system.last_spawn * system.spawn_rate).floor() as usize;
        
        if spawn_count > 0 {
            system.last_spawn = 0.0;
            
            for _ in 0..spawn_count.min(system.max_particles - system.active_particles) {
                let mut rng = rand::thread_rng();
                
                // Calculate spawn position based on shape
                let spawn_pos = match &system.spawn_shape {
                    SpawnShape::Point => system.emitter_position,
                    SpawnShape::Sphere { radius } => {
                        let r = rng.gen::<f32>() * radius;
                        let theta = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                        let phi = rng.gen::<f32>() * std::f32::consts::PI;
                        let x = r * phi.sin() * theta.cos();
                        let y = r * phi.sin() * theta.sin();
                        let z = r * phi.cos();
                        system.emitter_position + Vec3::new(x, y, z)
                    },
                    SpawnShape::Box { size } => {
                        let x = (rng.gen::<f32>() - 0.5) * size.x;
                        let y = (rng.gen::<f32>() - 0.5) * size.y;
                        let z = (rng.gen::<f32>() - 0.5) * size.z;
                        system.emitter_position + Vec3::new(x, y, z)
                    },
                    SpawnShape::Cone { angle, height } => {
                        let r = rng.gen::<f32>() * (*angle) * (*height);
                        let theta = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                        let x = r * theta.cos();
                        let y = rng.gen::<f32>() * (*height);
                        let z = r * theta.sin();
                        system.emitter_position + Vec3::new(x, y, z)
                    },
                };
                
                // Calculate initial velocity with spread
                let velocity = system.spawn_params.initial_velocity 
                    + Vec3::new(
                        (rng.gen::<f32>() - 0.5) * 2.0 * system.spawn_params.velocity_spread,
                        (rng.gen::<f32>() - 0.5) * 2.0 * system.spawn_params.velocity_spread,
                        (rng.gen::<f32>() - 0.5) * 2.0 * system.spawn_params.velocity_spread,
                    );
                
                // Calculate size
                let size_range = system.spawn_params.size_range;
                let size = size_range.0 + rng.gen::<f32>() * (size_range.1 - size_range.0);
                
                // Calculate color
                let color = system.spawn_params.color_range.0; // Simplified - just use first color
                
                // Create particle
                commands.spawn((
                    Particle {
                        position: spawn_pos,
                        velocity,
                        lifetime: 0.0,
                        max_lifetime: 5.0, // TODO: make configurable
                        mass: 1.0, // TODO: make configurable
                        size,
                        color,
                    },
                    Transform::from_translation(spawn_pos),
                    Visibility::Visible,
                ));
                
                system.active_particles += 1;
            }
        }
    }
}

/// Function to create a basic particle system entity
pub fn create_particle_system(
    commands: &mut Commands,
    position: Vec3,
    spawn_rate: f32,
) -> Entity {
    commands.spawn((
        ParticleSystem {
            spawn_rate,
            emitter_position: position,
            ..Default::default()
        },
        Transform::from_translation(position),
        Visibility::Visible,
    )).id()
}

/// Enum representing different particle physics forces inspired by Party library
#[derive(Debug, Clone)]
pub enum ParticleForce {
    /// Environmental forces (gravity, wind, etc.)
    Environment { 
        gravity: Vec3, 
        wind: Vec3, 
        friction: f32 
    },
    /// Boundary interactions (bounce, wrap, kill)
    Boundary { 
        mode: BoundaryMode, 
        restitution: f32,
        bounds: Bounds,
    },
    /// Collision detection between particles
    Collisions { 
        restitution: f32 
    },
    /// Flocking behaviors (cohesion, alignment, separation)
    Behavior { 
        cohesion: f32, 
        alignment: f32, 
        separation: f32,
        view_radius: f32,
    },
    /// Fluid dynamics (SPH - Smoothed Particle Hydrodynamics)
    Fluids { 
        pressure_multiplier: f32, 
        viscosity: f32,
        influence_radius: f32,
    },
    /// Mouse/user interaction
    Interaction { 
        strength: f32,
        radius: f32,
    },
}

#[derive(Debug, Clone)]
pub enum BoundaryMode {
    Bounce,
    Wrap,
    Kill,
    Repel { strength: f32 },
}

#[derive(Debug, Clone)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for ParticleForce {
    fn default() -> Self {
        Self::Environment { 
            gravity: Vec3::new(0.0, -9.8, 0.0), 
            wind: Vec3::ZERO, 
            friction: 0.1 
        }
    }
}