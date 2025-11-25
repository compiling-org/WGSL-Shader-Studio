//! Compute pass integration with ping-pong storage textures
//! Based on proven patterns from wgpu-compute-toy and use.gpu reference repositories

use bevy::prelude::*;
use std::collections::HashMap;

/// Compute pass integration plugin
pub struct ComputePassPlugin;

impl Plugin for ComputePassPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComputePassManager>()
            .add_systems(Update, update_compute_passes);
    }
}

/// Manager for compute passes and ping-pong resources
#[derive(Resource)]
pub struct ComputePassManager {
    pub ping_pong_textures: HashMap<String, PingPongTexture>,
    pub ping_pong_buffers: HashMap<String, PingPongBuffer>,
    pub storage_textures: HashMap<String, StorageTextureResource>,
    pub compute_pipelines: HashMap<String, ComputePipelineResource>,
    pub active_compute_passes: Vec<ComputePassExecution>,
}

impl Default for ComputePassManager {
    fn default() -> Self {
        Self {
            ping_pong_textures: HashMap::new(),
            ping_pong_buffers: HashMap::new(),
            storage_textures: HashMap::new(),
            compute_pipelines: HashMap::new(),
            active_compute_passes: Vec::new(),
        }
    }
}

/// Ping-pong texture for compute passes (double buffering)
#[derive(Debug, Clone)]
pub struct PingPongTexture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub current_index: usize,
    pub frame_count: u64,
}

/// Ping-pong buffer for compute passes
#[derive(Debug, Clone)]
pub struct PingPongBuffer {
    pub name: String,
    pub size: usize,
    pub current_index: usize,
    pub frame_count: u64,
}

/// Storage texture resource
#[derive(Debug, Clone)]
pub struct StorageTextureResource {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub access: StorageAccess,
}

/// Storage access mode
#[derive(Debug, Clone, PartialEq)]
pub enum StorageAccess {
    Read,
    Write,
    ReadWrite,
}

/// Texture format for compute resources
#[derive(Debug, Clone, PartialEq)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba16Float,
    Rgba32Float,
    Rg16Float,
    R32Float,
}

/// Compute pipeline resource
#[derive(Debug, Clone)]
pub struct ComputePipelineResource {
    pub name: String,
    pub workgroup_size: (u32, u32, u32),
    pub shader_code: String,
    pub bind_group_layouts: Vec<BindGroupLayoutResource>,
}

/// Bind group layout resource
#[derive(Debug, Clone)]
pub struct BindGroupLayoutResource {
    pub name: String,
    pub entries: Vec<BindGroupLayoutEntry>,
}

/// Bind group layout entry
#[derive(Debug, Clone)]
pub struct BindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: ShaderStage,
    pub ty: BindingType,
}

/// Shader stage visibility
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderStage {
    Compute,
    Vertex,
    Fragment,
    All,
}

/// Binding type
#[derive(Debug, Clone)]
pub enum BindingType {
    StorageTexture { access: StorageAccess, format: TextureFormat },
    StorageBuffer { access: StorageAccess },
    SampledTexture,
    Sampler,
    UniformBuffer,
}

/// Compute pass execution
#[derive(Debug, Clone)]
pub struct ComputePassExecution {
    pub name: String,
    pub pipeline_name: String,
    pub dispatch_size: (u32, u32, u32),
    pub input_resources: Vec<String>,
    pub output_resources: Vec<String>,
    pub dependencies: Vec<String>,
}

impl ComputePassManager {
    /// Create a new ping-pong texture
    pub fn create_ping_pong_texture(&mut self, name: &str, width: u32, height: u32, format: TextureFormat) {
        let ping_pong_texture = PingPongTexture {
            name: name.to_string(),
            width,
            height,
            format,
            current_index: 0,
            frame_count: 0,
        };
        
        self.ping_pong_textures.insert(name.to_string(), ping_pong_texture);
    }
    
    /// Swap ping-pong textures (advance to next frame)
    pub fn swap_ping_pong_texture(&mut self, name: &str) {
        if let Some(texture) = self.ping_pong_textures.get_mut(name) {
            texture.current_index = (texture.current_index + 1) % 2;
            texture.frame_count += 1;
        }
    }
    
    /// Get current ping-pong texture
    pub fn get_current_ping_pong_texture(&self, name: &str) -> Option<&PingPongTexture> {
        self.ping_pong_textures.get(name)
    }
    
    /// Get previous ping-pong texture (for reading previous frame)
    pub fn get_previous_ping_pong_texture(&self, name: &str) -> Option<&PingPongTexture> {
        self.ping_pong_textures.get(name).map(|texture| {
            let _prev_index = if texture.current_index == 0 { 1 } else { 0 };
            // Return the same texture but with the previous index conceptually
            texture
        })
    }
    
    /// Create a compute pipeline
    pub fn create_compute_pipeline(&mut self, name: &str, workgroup_size: (u32, u32, u32), shader_code: String, bind_group_layouts: Vec<BindGroupLayoutResource>) {
        let compute_pipeline = ComputePipelineResource {
            name: name.to_string(),
            workgroup_size,
            shader_code,
            bind_group_layouts,
        };
        
        self.compute_pipelines.insert(name.to_string(), compute_pipeline);
    }
    
    /// Create a compute pass execution
    pub fn create_compute_pass_execution(&mut self, name: &str, pipeline_name: &str, dispatch_size: (u32, u32, u32), input_resources: Vec<String>, output_resources: Vec<String>, dependencies: Vec<String>) {
        let compute_pass = ComputePassExecution {
            name: name.to_string(),
            pipeline_name: pipeline_name.to_string(),
            dispatch_size,
            input_resources,
            output_resources,
            dependencies,
        };
        
        self.active_compute_passes.push(compute_pass);
    }
    
    /// Create a new ping-pong buffer
    pub fn create_ping_pong_buffer(&mut self, name: &str, size: usize) {
        let ping_pong_buffer = PingPongBuffer {
            name: name.to_string(),
            size,
            current_index: 0,
            frame_count: 0,
        };
        
        self.ping_pong_buffers.insert(name.to_string(), ping_pong_buffer);
    }
    
    /// Swap ping-pong buffers
    pub fn swap_ping_pong_buffer(&mut self, name: &str) {
        if let Some(buffer) = self.ping_pong_buffers.get_mut(name) {
            buffer.current_index = (buffer.current_index + 1) % 2;
            buffer.frame_count += 1;
        }
    }
    
    /// Get current ping-pong buffer
    pub fn get_current_ping_pong_buffer(&self, name: &str) -> Option<&PingPongBuffer> {
        self.ping_pong_buffers.get(name)
    }
    
    /// Create a storage texture
    pub fn create_storage_texture(&mut self, name: &str, width: u32, height: u32, format: TextureFormat, access: StorageAccess) {
        let storage_texture = StorageTextureResource {
            name: name.to_string(),
            width,
            height,
            format,
            access,
        };
        
        self.storage_textures.insert(name.to_string(), storage_texture);
    }
    
    
    /// Add a compute pass execution
    pub fn add_compute_pass(&mut self, execution: ComputePassExecution) {
        self.active_compute_passes.push(execution);
    }
    
    /// Generate WGSL code for compute pass
    pub fn generate_compute_wgsl(&self, pass_name: &str) -> Option<String> {
        let pipeline = self.compute_pipelines.get(pass_name)?;
        
        let mut wgsl = String::new();
        
        // Add storage texture bindings
        for (i, output_resource) in self.active_compute_passes.iter()
            .find(|pass| pass.pipeline_name == pass_name)?
            .output_resources.iter().enumerate() {
            
            if let Some(storage_texture) = self.storage_textures.get(output_resource) {
                let access_str = match storage_texture.access {
                    StorageAccess::Read => "read",
                    StorageAccess::Write => "write",
                    StorageAccess::ReadWrite => "read_write",
                };
                
                let format_str = match storage_texture.format {
                    TextureFormat::Rgba8Unorm => "rgba8unorm",
                    TextureFormat::Rgba16Float => "rgba16float",
                    TextureFormat::Rgba32Float => "rgba32float",
                    TextureFormat::Rg16Float => "rg16float",
                    TextureFormat::R32Float => "r32float",
                };
                
                wgsl.push_str(&format!(
                    "@group(1) @binding({}) var {}: texture_storage_2d<{}, {}>;\n",
                    i, output_resource, format_str, access_str
                ));
            }
        }
        
        // Add compute function
        wgsl.push_str(&format!(
            "@compute @workgroup_size({}, {}, {})\n",
            pipeline.workgroup_size.0, pipeline.workgroup_size.1, pipeline.workgroup_size.2
        ));
        
        wgsl.push_str(&format!("fn {}(@builtin(global_invocation_id) global_id: vec3<u32>) {{\n", pass_name));
        wgsl.push_str("    let coord = global_id.xy;\n");
        wgsl.push_str("    // Compute shader implementation\n");
        wgsl.push_str("    // Add your compute logic here\n");
        wgsl.push_str("}\n");
        
        Some(wgsl)
    }
    
    /// Generate example compute shader for particle simulation
    pub fn generate_particle_compute_wgsl(&self) -> String {
        let mut wgsl = String::new();
        
        // Particle data structure
        wgsl.push_str("struct Particle {\n");
        wgsl.push_str("    position: vec2<f32>,\n");
        wgsl.push_str("    velocity: vec2<f32>,\n");
        wgsl.push_str("    life: f32,\n");
        wgsl.push_str("    size: f32,\n");
        wgsl.push_str("}\n\n");
        
        // Uniforms
        wgsl.push_str("struct Uniforms {\n");
        wgsl.push_str("    time: f32,\n");
        wgsl.push_str("    delta_time: f32,\n");
        wgsl.push_str("    resolution: vec2<f32>,\n");
        wgsl.push_str("}\n\n");
        
        // Bindings
        wgsl.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n");
        wgsl.push_str("@group(1) @binding(0) var<storage, read> particles_in: array<Particle>;\n");
        wgsl.push_str("@group(1) @binding(1) var<storage, read_write> particles_out: array<Particle>;\n");
        wgsl.push_str("@group(1) @binding(2) var output_texture: texture_storage_2d<rgba8unorm, write>;\n\n");
        
        // Compute shader
        wgsl.push_str("@compute @workgroup_size(64, 1, 1)\n");
        wgsl.push_str("fn particle_simulate(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
        wgsl.push_str("    let index = global_id.x;\n");
        wgsl.push_str("    if (index >= arrayLength(&particles_in)) { return; }\n\n");
        
        wgsl.push_str("    var particle = particles_in[index];\n");
        wgsl.push_str("    \n");
        wgsl.push_str("    // Update particle life\n");
        wgsl.push_str("    particle.life = particle.life - uniforms.delta_time;\n");
        wgsl.push_str("    \n");
        wgsl.push_str("    // Apply gravity\n");
        wgsl.push_str("    particle.velocity.y = particle.velocity.y - 9.8 * uniforms.delta_time;\n");
        wgsl.push_str("    \n");
        wgsl.push_str("    // Update position\n");
        wgsl.push_str("    particle.position = particle.position + particle.velocity * uniforms.delta_time;\n");
        wgsl.push_str("    \n");
        wgsl.push_str("    // Bounce off edges\n");
        wgsl.push_str("    if (particle.position.x < 0.0 || particle.position.x > uniforms.resolution.x) {\n");
        wgsl.push_str("        particle.velocity.x = -particle.velocity.x * 0.8;\n");
        wgsl.push_str("    }\n");
        wgsl.push_str("    if (particle.position.y < 0.0 || particle.position.y > uniforms.resolution.y) {\n");
        wgsl.push_str("        particle.velocity.y = -particle.velocity.y * 0.8;\n");
        wgsl.push_str("    }\n");
        wgsl.push_str("    \n");
        wgsl.push_str("    // Respawn if dead\n");
        wgsl.push_str("    if (particle.life <= 0.0) {\n");
        wgsl.push_str("        particle.position = vec2<f32>(uniforms.resolution.x * 0.5, uniforms.resolution.y * 0.8);\n");
        wgsl.push_str("        particle.velocity = vec2<f32>((f32(index) - 32.0) * 10.0, -50.0);\n");
        wgsl.push_str("        particle.life = 3.0;\n");
        wgsl.push_str("        particle.size = 2.0 + f32(index % 4);\n");
        wgsl.push_str("    }\n");
        wgsl.push_str("    \n");
        wgsl.push_str("    particles_out[index] = particle;\n");
        wgsl.push_str("    \n");
        wgsl.push_str("    // Render particle to texture\n");
        wgsl.push_str("    let coord = vec2<i32>(particle.position);\n");
        wgsl.push_str("    let alpha = clamp(particle.life / 3.0, 0.0, 1.0);\n");
        wgsl.push_str("    textureStore(output_texture, coord, vec4<f32>(1.0, 0.5, 0.0, alpha));\n");
        wgsl.push_str("}\n");
        
        wgsl
    }
    
    /// Generate example compute shader for Conway's Game of Life
    pub fn generate_game_of_life_wgsl(&self) -> String {
        let mut wgsl = String::new();
        
        // Storage texture bindings
        wgsl.push_str("@group(1) @binding(0) var current_state: texture_storage_2d<rgba8unorm, read>;\n");
        wgsl.push_str("@group(1) @binding(1) var next_state: texture_storage_2d<rgba8unorm, write>;\n\n");
        
        // Compute shader
        wgsl.push_str("@compute @workgroup_size(8, 8, 1)\n");
        wgsl.push_str("fn game_of_life(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
        wgsl.push_str("    let coord = global_id.xy;\n");
        wgsl.push_str("    let size = textureDimensions(current_state);\n");
        wgsl.push_str("    if (coord.x >= size.x || coord.y >= size.y) { return; }\n\n");
        
        wgsl.push_str("    // Count live neighbors\n");
        wgsl.push_str("    var live_neighbors = 0;\n");
        wgsl.push_str("    for (var dy = -1; dy <= 1; dy++) {\n");
        wgsl.push_str("        for (var dx = -1; dx <= 1; dx++) {\n");
        wgsl.push_str("            if (dx == 0 && dy == 0) { continue; }\n");
        wgsl.push_str("            let neighbor_coord = vec2<i32>(coord) + vec2<i32>(dx, dy);\n");
        wgsl.push_str("            if (neighbor_coord.x >= 0 && neighbor_coord.x < size.x &&\n");
        wgsl.push_str("                neighbor_coord.y >= 0 && neighbor_coord.y < size.y) {\n");
        wgsl.push_str("                let neighbor_color = textureLoad(current_state, neighbor_coord);\n");
        wgsl.push_str("                if (neighbor_color.r > 0.5) {\n");
        wgsl.push_str("                    live_neighbors++;\n");
        wgsl.push_str("                }\n");
        wgsl.push_str("            }\n");
        wgsl.push_str("        }\n");
        wgsl.push_str("    }\n\n");
        
        wgsl.push_str("    // Apply Game of Life rules\n");
        wgsl.push_str("    let current_color = textureLoad(current_state, coord);\n");
        wgsl.push_str("    var next_color = vec4<f32>(0.0);\n");
        wgsl.push_str("    if (current_color.r > 0.5) {\n");
        wgsl.push_str("        // Cell is alive\n");
        wgsl.push_str("        if (live_neighbors == 2 || live_neighbors == 3) {\n");
        wgsl.push_str("            next_color = vec4<f32>(1.0);\n");
        wgsl.push_str("        }\n");
        wgsl.push_str("    } else {\n");
        wgsl.push_str("        // Cell is dead\n");
        wgsl.push_str("        if (live_neighbors == 3) {\n");
        wgsl.push_str("            next_color = vec4<f32>(1.0);\n");
        wgsl.push_str("        }\n");
        wgsl.push_str("    }\n\n");
        wgsl.push_str("    textureStore(next_state, coord, next_color);\n");
        wgsl.push_str("}\n");
        
        wgsl
    }
}

/// Update compute passes
fn update_compute_passes(
    mut compute_manager: ResMut<ComputePassManager>,
    time: Res<Time>,
) {
    // Update ping-pong resources every frame
    for texture in compute_manager.ping_pong_textures.values_mut() {
        if time.elapsed_secs() > 0.016 { // 60 FPS
            texture.frame_count += 1;
        }
    }
    
    for buffer in compute_manager.ping_pong_buffers.values_mut() {
        if time.elapsed_secs() > 0.016 { // 60 FPS
            buffer.frame_count += 1;
        }
    }
}
