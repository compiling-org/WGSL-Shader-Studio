//! Professional WGPU shader renderer implementation for real-time WGSL shader preview
//! Features: Real WGSL compilation, live parameter updates, texture support, performance monitoring

use wgpu::{*, util::DeviceExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use egui::TextureHandle;

/// WGPU-based shader renderer for professional shader development
pub struct WgpuShaderRenderer {
    // WGPU core components
    device: Device,
    queue: Queue,
    surface: Surface,
    surface_config: SurfaceConfiguration,
    
    // Shader management
    shader_modules: HashMap<String, ShaderModule>,
    render_pipelines: HashMap<String, RenderPipeline>,
    uniform_buffers: HashMap<String, Buffer>,
    
    // Textures and render targets
    output_texture: Texture,
    output_view: TextureView,
    render_target: Texture,
    render_target_view: TextureView,
    texture_bind_groups: HashMap<String, BindGroup>,
    
    // Performance tracking
    frame_count: u64,
    last_fps_time: Instant,
    fps: f32,
    compilation_times: HashMap<String, std::time::Duration>,
    
    // Render state
    current_shader: Option<String>,
    render_size: (u32, u32),
}

/// Uniform data for shaders
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShaderUniforms {
    pub time: f32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub audio_data: [f32; 8], // 8 audio bands
    pub custom_params: [f32; 16], // 16 custom parameters
}

impl Default for ShaderUniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            resolution: [1920.0, 1080.0],
            mouse: [0.5, 0.5],
            audio_data: [0.0; 8],
            custom_params: [0.0; 16],
        }
    }
}

impl WgpuShaderRenderer {
    /// Create new WGPU shader renderer
    pub async fn new(window: &winit::window::Window) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize WGPU with high-performance settings
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.ok_or("No suitable GPU adapter found")?;

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: Some("WGSL Shader Studio GPU"),
            required_features: Features::TEXTURE_COMPRESSION_BC | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
            required_limits: Limits {
                max_texture_dimension_2d: 4096,
                max_bind_groups: 16,
                max_uniform_buffer_binding_size: 1 << 20,
                ..Limits::default()
            },
            ..Default::default()
        }).await?;

        // Configure surface
        let size = window.inner_size();
        let safe_width = size.width.max(1);
        let safe_height = size.height.max(1);
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
            format: TextureFormat::Rgba8UnormSrgb,
            width: safe_width,
            height: safe_height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![TextureFormat::Rgba8UnormSrgb],
        };
        
        surface.configure(&device, &surface_config);

        // Create output textures
        let output_texture = device.create_texture(&TextureDescriptor {
            label: Some("Shader Output"),
            size: wgpu::Extent3d {
                width: safe_width,
                height: safe_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[TextureFormat::Rgba8Unorm],
        });

        let render_target = device.create_texture(&TextureDescriptor {
            label: Some("Render Target"),
            size: wgpu::Extent3d {
                width: safe_width,
                height: safe_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::Rgba8Unorm],
        });

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            shader_modules: HashMap::new(),
            render_pipelines: HashMap::new(),
            uniform_buffers: HashMap::new(),
            output_texture,
            output_view: output_texture.create_view(&TextureViewDescriptor::default()),
            render_target,
            render_target_view: render_target.create_view(&TextureViewDescriptor::default()),
            texture_bind_groups: HashMap::new(),
            frame_count: 0,
            last_fps_time: Instant::now(),
            fps: 0.0,
            compilation_times: HashMap::new(),
            current_shader: None,
            render_size: (safe_width, safe_height),
        })
    }

    /// Compile and register WGSL shader
    pub fn compile_shader(&mut self, name: &str, wgsl_source: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // Create shader module
        let shader_module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&format!("Shader: {}", name)),
            source: ShaderSource::Wgsl(wgsl_source.into()),
        });

        // Parse uniforms from WGSL
        let uniforms = self.parse_uniforms(wgsl_source);
        
        // Create uniform buffer
        let uniform_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some(&format!("Uniform Buffer: {}", name)),
            size: std::mem::size_of::<ShaderUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout for uniforms
        let bind_group_layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(&format!("Uniform Layout: {}", name)),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("Bind Group: {}", name)),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
        });

        // Create render pipeline
        let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("Pipeline Layout: {}", name)),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline: {}", name)),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: Some(BlendState::ALPHA_PREMULTIPLIED),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        // Store all components
        self.shader_modules.insert(name.to_string(), shader_module);
        self.render_pipelines.insert(name.to_string(), pipeline);
        self.uniform_buffers.insert(name.to_string(), uniform_buffer);

        let compilation_time = start_time.elapsed();
        self.compilation_times.insert(name.to_string(), compilation_time);
        
        println!("Shader '{}' compiled in {:.2}ms", name, compilation_time.as_millis());
        Ok(())
    }

    /// Parse uniform variables from WGSL source
    fn parse_uniforms(&self, wgsl_source: &str) -> Vec<String> {
        let mut uniforms = Vec::new();
        
        for line in wgsl_source.lines() {
            if line.trim_start().starts_with("var<uniform>") {
                if let Some(name_start) = line.find('<').and_then(|i| line[i..].find('>')) {
                    let uniform_line = &line[name_start..];
                    if let Some(name_end) = uniform_line.find(':') {
                        let name = uniform_line[1..name_end].trim().to_string();
                        uniforms.push(name);
                    }
                }
            }
        }
        
        uniforms
    }

    /// Render frame with current shader
    pub fn render_frame(&mut self, uniforms: &ShaderUniforms) -> Result<TextureView, Box<dyn std::error::Error>> {
        let Some(shader_name) = &self.current_shader else {
            return Err("No shader selected".into());
        };

        let pipeline = self.render_pipelines.get(shader_name)
            .ok_or_else(|| format!("Shader '{}' not compiled", shader_name))?;

        let uniform_buffer = self.uniform_buffers.get(shader_name)
            .ok_or_else(|| format!("Uniform buffer for '{}' not found", shader_name))?;

        // Update uniform buffer
        self.queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Shader Render"),
        });

        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.render_target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set pipeline and draw
        render_pass.set_pipeline(pipeline);
        render_pass.draw(0..6, 0..1);

        drop(render_pass);

        // Copy to output texture
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.render_target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::Extent3d {
                width: self.render_size.0,
                height: self.render_size.1,
                depth_or_array_layers: 1,
            },
        );

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));

        // Update FPS
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_time);
        
        if elapsed.as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.frame_count = 0;
            self.last_fps_time = now;
        }

        Ok(self.output_view.clone())
    }

    /// Set current shader
    pub fn set_current_shader(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.shader_modules.contains_key(name) {
            return Err(format!("Shader '{}' not found. Compile it first.", name).into());
        }
        
        self.current_shader = Some(name.to_string());
        Ok(())
    }

    /// Resize renderer
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.render_size = (width, height);
        
        self.surface.configure(&self.device, &self.surface_config);

        // Recreate textures
        self.output_texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Resized Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[TextureFormat::Rgba8Unorm],
        });

        self.render_target = self.device.create_texture(&TextureDescriptor {
            label: Some("Resized Render Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::Rgba8Unorm],
        });

        self.output_view = self.output_texture.create_view(&TextureViewDescriptor::default());
        self.render_target_view = self.render_target.create_view(&TextureViewDescriptor::default());

        Ok(())
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> RendererStats {
        RendererStats {
            fps: self.fps,
            shader_count: self.shader_modules.len(),
            frame_count: self.frame_count,
            compilation_times: self.compilation_times.clone(),
        }
    }

    /// Load texture from file
    pub fn load_texture(&mut self, name: &str, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let image = image::open(path)?.to_rgba8();
        let (width, height) = image.dimensions();

        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(&format!("Loaded Texture: {}", name)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[TextureFormat::Rgba8Unorm],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Create texture bind group
        let bind_group_layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(&format!("Texture Layout: {}", name)),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("Texture Bind Group: {}", name)),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture.create_view(&TextureViewDescriptor::default())),
            }],
        });

        self.texture_bind_groups.insert(name.to_string(), bind_group);
        Ok(())
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct RendererStats {
    pub fps: f32,
    pub shader_count: usize,
    pub frame_count: u64,
    pub compilation_times: HashMap<String, std::time::Duration>,
}
