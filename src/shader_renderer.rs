use wgpu::*;
use winit::window::Window;
use winit::dpi::PhysicalSize;
use egui::TextureHandle;
use bytemuck::{Pod, Zeroable};

use crate::isf_loader::*;
use crate::audio::AudioData;

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
}

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
        }
    }
}

pub struct ShaderRenderer {
    device: Device,
    queue: Queue,
    _instance: Instance, // Keep instance alive
    size: (u32, u32),
    // Working WGPU example shaders
    working_examples: Vec<WorkingShaderExample>,
}

#[derive(Debug, Clone)]
pub struct WorkingShaderExample {
    pub name: String,
    pub description: String,
    pub wgsl_code: String,
    pub category: String,
}

impl ShaderRenderer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let instance = Instance::new(&wgpu::InstanceDescriptor::default());
        let size = (512, 512);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to create adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;

        let mut working_examples = Vec::new();
        ShaderRenderer::add_working_examples(&mut working_examples);

        Ok(Self {
            device,
            queue,
            _instance: instance,
            size,
            working_examples,
        })
    }

    fn add_working_examples(examples: &mut Vec<WorkingShaderExample>) {
        examples.push(WorkingShaderExample {
            name: "Animated Gradient".to_string(),
            description: "Beautiful animated color gradient using time".to_string(),
            category: "Basic".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
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
            name: "Mandelbrot Fractal".to_string(),
            description: "Classic Mandelbrot fractal with coloring".to_string(),
            category: "Fractal".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn mandelbrot(c: vec2<f32>) -> f32 {
    var z = vec2<f32>(0.0, 0.0);
    let max_iter = 100.0;
    
    var iterations: f32 = 0.0;
    loop {
        if (dot(z, z) > 4.0 || iterations >= max_iter) {
            break;
        }
        z = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        iterations = iterations + 1.0;
    }
    return iterations / max_iter;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy - 0.5 * uniforms.resolution) / min(uniforms.resolution.x, uniforms.resolution.y);
    let zoom = 2.0;
    let pan = vec2<f32>(-0.5, 0.0);
    let c = uv * zoom + pan;
    
    let m = mandelbrot(c);
    let color = vec3<f32>(m, m * 0.5, m * 0.8);
    
    return vec4<f32>(color, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Audio Reactive Wave".to_string(),
            description: "Wave pattern that responds to audio".to_string(),
            category: "Audio".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
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
    
    let wave = sin(uv.x * 10.0 + time * 2.0 + uniforms.audio_volume * 5.0) * 0.5 + 0.5;
    let audio_boost = uniforms.audio_volume * 0.3;
    
    let r = wave + audio_boost;
    let g = 0.5 + 0.5 * sin(time + uv.y * 6.28318 + uniforms.audio_mid);
    let b = 0.5 + 0.5 * cos(time + uniforms.audio_bass);
    
    return vec4<f32>(r, g, b, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Plasma Effect".to_string(),
            description: "Classic plasma effect with smooth colors".to_string(),
            category: "Effects".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy / uniforms.resolution - 0.5) * 2.0;
    let time = uniforms.time;
    
    let r = sin(uv.x * 3.0 + time) + sin(uv.y * 2.0 + time * 0.5);
    let g = sin(uv.x * 2.0 + time * 0.7) + sin(uv.y * 3.0 + time * 1.2);
    let b = sin(uv.x * 4.0 + time * 0.3) + sin(uv.y * 1.0 + time * 0.9);
    
    let col = vec3<f32>(0.5 + 0.5 * r, 0.5 + 0.5 * g, 0.5 + 0.5 * b);
    
    return vec4<f32>(col, 1.0);
}"#),
        });
    }

    pub fn get_working_examples(&self) -> &[WorkingShaderExample] {
        &self.working_examples
    }

    pub async fn render_frame(
        &mut self,
        wgsl_code: &str,
        params: &RenderParameters,
        audio_data: Option<AudioData>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just validate the shader compiles
        // In a full implementation, this would render to an actual surface
        
        let _uniforms = Uniforms {
            time: params.time,
            resolution: [params.width as f32, params.height as f32],
            mouse: [0.5, 0.5],
            audio_volume: audio_data.as_ref().map(|a| a.volume).unwrap_or(0.0),
            audio_bass: audio_data.as_ref().map(|a| a.bass_level).unwrap_or(0.0),
            audio_mid: audio_data.as_ref().map(|a| a.mid_level).unwrap_or(0.0),
            audio_treble: audio_data.as_ref().map(|a| a.treble_level).unwrap_or(0.0),
        };

        // Create a temporary pipeline to validate shader compilation
        let _shader_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
        });

        println!("Shader compilation validated successfully");
        Ok(())
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.size = (width, height);
        Ok(())
    }
}

// Simple full-screen triangle vertex shader
const VERTEX_SHADER: &str = r#"
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