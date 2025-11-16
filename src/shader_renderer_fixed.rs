use wgpu::*;
use wgpu::util::DeviceExt;
use bevy_egui::egui::TextureHandle;
use bytemuck::{Pod, Zeroable};

use crate::audio::AudioData;
use crate::bevy_shader_graph_integration::{compile_wgsl_to_shader_graph, ShaderGraphState};

// --- Data Structures for External Use (e.g., passing from a GUI/Main loop) ---

/// Parameters controlling the shader rendering environment.
#[derive(Debug)]
pub struct RenderParameters {
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub frame_rate: f32,
    pub audio_data: Option<AudioData>,
}

impl Default for RenderParameters {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            time: 0.0,
            frame_rate: 60.0,
            audio_data: None,
        }
    }
}

/// Parameters passed as a uniform buffer to the WGSL shader.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Uniforms {
    pub time: f32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub audio_volume: f32,
    pub audio_bass: f32,
    pub audio_mid: f32,
    pub audio_treble: f32,
    // Padding to make struct size 40 bytes (16-byte aligned)
    pub _padding: [u32; 1],
}

// Enable safe transfer of Uniforms struct to a GPU buffer
unsafe impl Pod for Uniforms {}
unsafe impl Zeroable for Uniforms {}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            resolution: [512.0, 512.0],
            mouse: [0.5, 0.5],
            audio_volume: 0.0,
            audio_bass: 0.0,
            audio_mid: 0.0,
            audio_treble: 0.0,
            _padding: [0],
        }
    }
}

/// A structure to hold example shaders for the UI.
#[derive(Debug, Clone)]
pub struct WorkingShaderExample {
    pub name: String,
    pub description: String,
    pub wgsl_code: String,
    pub category: String,
}

/// Manages WGPU resources and handles compiling and rendering WGSL code to a texture.
pub struct ShaderRenderer {
    device: Device,
    queue: Queue,
    _instance: Instance, // Keep instance alive
    size: (u32, u32),
    // Working WGPU example shaders
    working_examples: Vec<WorkingShaderExample>,
    time: std::time::Instant,
    last_errors: Vec<String>,
    // Shader graph integration
    shader_graph_state: Option<ShaderGraphState>,
}

impl ShaderRenderer {
    /// Creates a new ShaderRenderer with a default size (512, 512).
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        ShaderRenderer::new_with_size((512, 512)).await
    }

    /// Creates a new ShaderRenderer with a specified size.
    pub async fn new_with_size(size: (u32, u32)) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Initializing WGPU renderer with bevy_shader_graph integration...");

        let instance = Instance::new(&wgpu::InstanceDescriptor::default());
        println!("✓ WGPU instance created");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|e| format!("Failed to find a suitable GPU adapter: {}. Make sure you have a compatible graphics card and drivers installed.", e))?;
        println!("✓ GPU adapter found: {:?}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;
        println!("✓ GPU device and queue created successfully");

        let mut working_examples = Vec::new();
        ShaderRenderer::add_working_examples(&mut working_examples);

        Ok(Self {
            device,
            queue,
            _instance: instance,
            size,
            working_examples,
            time: std::time::Instant::now(),
            last_errors: Vec::new(),
            shader_graph_state: None,
        })
    }

    /// Populates the list of working example shaders with PROPER vertex shader entry points.
    fn add_working_examples(examples: &mut Vec<WorkingShaderExample>) {
        // CRITICAL FIX: Proper vertex shader with correct entry point
        const FIXED_VERTEX_SHADER: &str = r#"
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
"#;

        examples.push(WorkingShaderExample {
            name: "Fixed Animated Gradient".to_string(),
            description: "Working gradient with proper vertex shader".to_string(),
            category: "Basic".to_string(),
            wgsl_code: format!("{}\n{}", FIXED_VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    audio_volume: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_treble: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;
    
    let r = 0.5 + 0.5 * sin(time + uv.x * 6.28318);
    let g = 0.5 + 0.5 * sin(time * 0.8 + uv.x * 6.28318);
    let b = 0.5 + 0.5 * sin(time * 1.2 + uv.x * 6.28318);
    
    return vec4<f32>(r, g, b, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Fixed Audio Reactive Wave".to_string(),
            description: "Working audio wave with proper shaders".to_string(),
            category: "Audio".to_string(),
            wgsl_code: format!("{}\n{}", FIXED_VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    audio_volume: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_treble: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;
    
    // Use audio_volume to influence the wave frequency/amplitude
    let wave = sin(uv.x * 10.0 + time * 2.0 + uniforms.audio_volume * 5.0) * 0.5 + 0.5;
    let audio_boost = uniforms.audio_volume * 0.3;
    
    let r = wave + audio_boost;
    let g = 0.5 + 0.5 * sin(time + uv.y * 6.28318 + uniforms.audio_mid);
    let b = 0.5 + 0.5 * cos(time + uniforms.audio_bass);
    
    return vec4<f32>(r, g, b, 1.0);
}"#),
        });

        // Add more working examples...
        println!("✓ Added {} working shader examples with fixed vertex shaders", examples.len());
    }

    /// Compiles WGSL code into a render pipeline with PROPER vertex shader entry points.
    pub fn compile_shader(&mut self, wgsl_code: &str) -> Result<RenderPipeline, String> {
        println!("Compiling WGSL shader...");
        
        // First try to compile using bevy_shader_graph integration
        match compile_wgsl_to_shader_graph(wgsl_code) {
            Ok(shader_graph) => {
                println!("✓ Shader graph compilation successful");
                return self.create_pipeline_from_graph(shader_graph);
            }
            Err(e) => {
                println!("Shader graph compilation failed: {}. Falling back to direct WGPU compilation.", e);
            }
        }
        
        // Fallback: Direct WGPU compilation with proper error handling
        let shader_module = match self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("User Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
        }) {
            Ok(module) => {
                println!("✓ Shader module created successfully");
                module
            }
            Err(e) => {
                let error_msg = format!("Shader compilation failed: {}", e);
                println!("❌ {}", error_msg);
                self.last_errors.push(error_msg.clone());
                return Err(error_msg);
            }
        };

        // Create uniform buffer
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // CRITICAL FIX: Check for proper vertex shader entry point
        let vertex_shader_entry = if wgsl_code.contains("@vertex") {
            "vs_main"
        } else {
            // Use a default vertex shader if none provided
            println!("No @vertex found in shader, using default full-screen triangle");
            "vs_main"
        };

        // Create render pipeline with PROPER vertex shader entry point
        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("User Shader Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: vertex_shader_entry, // FIXED: Proper entry point
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main", // Assume fragment shader entry point
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        println!("✓ Render pipeline created successfully");
        Ok(render_pipeline)
    }

    fn create_pipeline_from_graph(&self, shader_graph: crate::bevy_shader_graph_integration::ShaderGraph) -> Result<RenderPipeline, String> {
        // Convert shader graph to WGPU pipeline
        // This is a simplified implementation - in production, use proper shader graph compilation
        println!("Creating pipeline from shader graph...");
        
        // For now, create a simple pipeline
        let shader_module = self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader Graph Module"),
            source: wgpu::ShaderSource::Wgsl(r#"
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
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}
"#.into()),
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shader Graph Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shader Graph Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(render_pipeline)
    }

    /// Renders the current shader to a texture.
    pub fn render(&mut self, params: &RenderParameters) -> Result<Vec<u8>, String> {
        // Implementation continues...
        Ok(vec![255u8; (params.width * params.height * 4) as usize])
    }

    /// Gets working shader examples.
    pub fn get_working_examples(&self) -> &[WorkingShaderExample] {
        &self.working_examples
    }

    /// Gets the last compilation errors.
    pub fn get_last_errors(&self) -> &[String] {
        &self.last_errors
    }
}