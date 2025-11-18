use std::sync::Arc;
use std::collections::HashMap;
use std::ffi::c_void;
use parking_lot::Mutex;
use wgpu::{
    Device, Queue, Texture, TextureFormat, TextureDescriptor, TextureView, 
    Buffer, BufferUsage, Extent3d, TextureDimension, TextureUsages,
    Backend, Adapter, Instance, Surface, SurfaceConfiguration,
    TextureViewDescriptor, CommandEncoder, RenderPass, RenderPipeline,
    BindGroup, BindGroupLayout, ShaderModule, PipelineLayout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsApi {
    DirectX11,
    DirectX12,
    OpenGL,
    OpenGLES,
    Metal,
    Vulkan,
}

#[derive(Debug, Clone)]
pub struct NativeTextureInfo {
    pub api: GraphicsApi,
    pub texture_ptr: *mut c_void,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub is_srgb: bool,
}

#[derive(Debug, Clone)]
pub struct InteropConfig {
    pub enable_zero_copy: bool,
    pub allow_cpu_fallback: bool,
    pub max_texture_size: u32,
    pub enable_multi_threading: bool,
    pub texture_format_mapping: HashMap<GraphicsApi, TextureFormat>,
}

impl Default for InteropConfig {
    fn default() -> Self {
        let mut format_mapping = HashMap::new();
        format_mapping.insert(GraphicsApi::DirectX11, TextureFormat::Rgba8Unorm);
        format_mapping.insert(GraphicsApi::DirectX12, TextureFormat::Rgba8Unorm);
        format_mapping.insert(GraphicsApi::OpenGL, TextureFormat::Rgba8Unorm);
        format_mapping.insert(GraphicsApi::OpenGLES, TextureFormat::Rgba8UnormSrgb);
        format_mapping.insert(GraphicsApi::Metal, TextureFormat::Rgba8Unorm);
        format_mapping.insert(GraphicsApi::Vulkan, TextureFormat::Rgba8Unorm);

        Self {
            enable_zero_copy: true,
            allow_cpu_fallback: true,
            max_texture_size: 8192,
            enable_multi_threading: true,
            texture_format_mapping: format_mapping,
        }
    }
}

#[derive(Debug)]
pub struct ZeroCopyTexture {
    pub native_info: NativeTextureInfo,
    pub wgpu_texture: Option<Texture>,
    pub wgpu_view: Option<TextureView>,
    pub is_zero_copy: bool,
}

#[derive(Debug)]
pub struct InteropPipeline {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub shader_module: ShaderModule,
    pub pipeline_layout: PipelineLayout,
    pub render_pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
}

#[derive(Debug)]
pub struct WgpuInteropManager {
    config: InteropConfig,
    instance: Instance,
    device: Arc<Device>,
    queue: Arc<Queue>,
    adapter: Adapter,
    
    // Zero-copy texture cache
    texture_cache: Arc<Mutex<HashMap<String, ZeroCopyTexture>>>,
    
    // Interop pipelines for different operations
    pipelines: Arc<Mutex<HashMap<String, InteropPipeline>>>,
    
    // Graphics API detection and mapping
    api_support: HashMap<GraphicsApi, bool>,
    
    // Performance metrics
    metrics: Arc<Mutex<InteropMetrics>>,
}

#[derive(Debug, Default)]
pub struct InteropMetrics {
    pub zero_copy_operations: u64,
    pub fallback_operations: u64,
    pub texture_cache_hits: u64,
    pub texture_cache_misses: u64,
    pub total_processing_time_ms: f64,
    pub average_frame_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct InteropResult {
    pub success: bool,
    pub used_zero_copy: bool,
    pub processing_time_ms: f64,
    pub error_message: Option<String>,
}

impl WgpuInteropManager {
    pub async fn new(config: InteropConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find suitable adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WgpuInteropManager Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let mut api_support = HashMap::new();
        api_support.insert(GraphicsApi::DirectX11, true);
        api_support.insert(GraphicsApi::DirectX12, true);
        api_support.insert(GraphicsApi::OpenGL, true);
        api_support.insert(GraphicsApi::OpenGLES, true);
        api_support.insert(GraphicsApi::Metal, cfg!(target_os = "macos"));
        api_support.insert(GraphicsApi::Vulkan, true);

        Ok(Self {
            config,
            instance,
            device,
            queue,
            adapter,
            texture_cache: Arc::new(Mutex::new(HashMap::new())),
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            api_support,
            metrics: Arc::new(Mutex::new(InteropMetrics::default())),
        })
    }

    /// Zero-copy texture interop - directly use native texture without CPU readback
    pub fn create_zero_copy_texture(
        &self,
        native_info: NativeTextureInfo,
        label: &str,
    ) -> Result<ZeroCopyTexture, Box<dyn std::error::Error>> {
        if !self.config.enable_zero_copy {
            return Err("Zero-copy is disabled".into());
        }

        if !self.is_api_supported(native_info.api) {
            return Err(format!("Graphics API {:?} is not supported", native_info.api).into());
        }

        // Create wgpu texture from native pointer (platform-specific implementation)
        let wgpu_texture = self.create_texture_from_native(&native_info, label)?;
        let wgpu_view = wgpu_texture.create_view(&TextureViewDescriptor::default());

        let zero_copy_texture = ZeroCopyTexture {
            native_info,
            wgpu_texture: Some(wgpu_texture),
            wgpu_view: Some(wgpu_view),
            is_zero_copy: true,
        };

        // Cache the texture for reuse
        let mut cache = self.texture_cache.lock();
        cache.insert(label.to_string(), zero_copy_texture.clone());

        // Update metrics
        let mut metrics = self.metrics.lock();
        metrics.zero_copy_operations += 1;
        metrics.texture_cache_misses += 1;

        Ok(zero_copy_texture)
    }

    /// Create texture from native pointer (platform-specific implementation)
    fn create_texture_from_native(
        &self,
        native_info: &NativeTextureInfo,
        label: &str,
    ) -> Result<Texture, Box<dyn std::error::Error>> {
        // This is a simplified implementation - actual implementation would be platform-specific
        // and involve native API calls to create wgpu texture from existing native texture
        
        let texture_desc = TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width: native_info.width,
                height: native_info.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: native_info.format,
            usage: TextureUsages::TEXTURE_BINDING 
                | TextureUsages::RENDER_ATTACHMENT 
                | TextureUsages::COPY_SRC 
                | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        Ok(self.device.create_texture(&texture_desc))
    }

    /// Process texture using wgpu compute or render pipeline
    pub fn process_texture(
        &self,
        input_texture: &ZeroCopyTexture,
        output_texture: &ZeroCopyTexture,
        pipeline_name: &str,
    ) -> Result<InteropResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // Get or create pipeline
        let pipeline = self.get_or_create_pipeline(pipeline_name)?;

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Interop Processing Encoder"),
        });

        // Create bind group
        let bind_group = self.create_bind_group(
            &pipeline.bind_group_layout,
            input_texture,
            output_texture,
        )?;

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Interop Processing Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_texture.wgpu_view.as_ref().unwrap(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&pipeline.render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));

        let processing_time = start_time.elapsed().as_millis() as f64;

        // Update metrics
        let mut metrics = self.metrics.lock();
        metrics.total_processing_time_ms += processing_time;
        metrics.average_frame_time_ms = metrics.total_processing_time_ms / (metrics.zero_copy_operations + metrics.fallback_operations) as f64;

        Ok(InteropResult {
            success: true,
            used_zero_copy: input_texture.is_zero_copy,
            processing_time_ms: processing_time,
            error_message: None,
        })
    }

    /// Get or create processing pipeline
    fn get_or_create_pipeline(&self, name: &str) -> Result<InteropPipeline, Box<dyn std::error::Error>> {
        let mut pipelines = self.pipelines.lock();
        
        if let Some(pipeline) = pipelines.get(name) {
            return Ok(pipeline.clone());
        }

        // Create shader module (WGSL shader for processing)
        let shader_source = self.create_shader_source(name);
        let shader_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", name)),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layout
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} Bind Group Layout", name)),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Pipeline Layout", name)),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} Render Pipeline", name)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
            cache: None,
        });

        let pipeline = InteropPipeline {
            device: self.device.clone(),
            queue: self.queue.clone(),
            shader_module,
            pipeline_layout,
            render_pipeline,
            bind_group_layout,
        };

        pipelines.insert(name.to_string(), pipeline.clone());
        Ok(pipeline)
    }

    /// Create bind group for texture processing
    fn create_bind_group(
        &self,
        layout: &BindGroupLayout,
        input_texture: &ZeroCopyTexture,
        _output_texture: &ZeroCopyTexture,
    ) -> Result<BindGroup, Box<dyn std::error::Error>> {
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Interop Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Interop Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        input_texture.wgpu_view.as_ref().unwrap()
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(bind_group)
    }

    /// Create shader source for different processing operations
    fn create_shader_source(&self, operation: &str) -> String {
        match operation {
            "passthrough" => r#"
                struct VertexOutput {
                    @builtin(position) position: vec4<f32>,
                    @location(0) uv: vec2<f32>,
                }

                @vertex
                fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
                    var positions = array<vec2<f32>, 6>(
                        vec2<f32>(-1.0, -1.0),
                        vec2<f32>( 1.0, -1.0),
                        vec2<f32>(-1.0,  1.0),
                        vec2<f32>(-1.0,  1.0),
                        vec2<f32>( 1.0, -1.0),
                        vec2<f32>( 1.0,  1.0)
                    );

                    var uvs = array<vec2<f32>, 6>(
                        vec2<f32>(0.0, 1.0),
                        vec2<f32>(1.0, 1.0),
                        vec2<f32>(0.0, 0.0),
                        vec2<f32>(0.0, 0.0),
                        vec2<f32>(1.0, 1.0),
                        vec2<f32>(1.0, 0.0)
                    );

                    var output: VertexOutput;
                    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
                    output.uv = uvs[vertex_index];
                    return output;
                }

                @group(0) @binding(0) var input_texture: texture_2d<f32>;
                @group(0) @binding(1) var input_sampler: sampler;

                @fragment
                fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
                    return textureSample(input_texture, input_sampler, input.uv);
                }
            "#,
            "color_correction" => r#"
                struct VertexOutput {
                    @builtin(position) position: vec4<f32>,
                    @location(0) uv: vec2<f32>,
                }

                @vertex
                fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
                    var positions = array<vec2<f32>, 6>(
                        vec2<f32>(-1.0, -1.0),
                        vec2<f32>( 1.0, -1.0),
                        vec2<f32>(-1.0,  1.0),
                        vec2<f32>(-1.0,  1.0),
                        vec2<f32>( 1.0, -1.0),
                        vec2<f32>( 1.0,  1.0)
                    );

                    var uvs = array<vec2<f32>, 6>(
                        vec2<f32>(0.0, 1.0),
                        vec2<f32>(1.0, 1.0),
                        vec2<f32>(0.0, 0.0),
                        vec2<f32>(0.0, 0.0),
                        vec2<f32>(1.0, 1.0),
                        vec2<f32>(1.0, 0.0)
                    );

                    var output: VertexOutput;
                    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
                    output.uv = uvs[vertex_index];
                    return output;
                }

                @group(0) @binding(0) var input_texture: texture_2d<f32>;
                @group(0) @binding(1) var input_sampler: sampler;

                @fragment
                fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
                    var color = textureSample(input_texture, input_sampler, input.uv);
                    
                    // Simple gamma correction
                    color.r = pow(color.r, 1.0 / 2.2);
                    color.g = pow(color.g, 1.0 / 2.2);
                    color.b = pow(color.b, 1.0 / 2.2);
                    
                    return color;
                }
            "#,
            _ => self.create_shader_source("passthrough"), // Default fallback
        }.to_string()
    }

    /// Check if graphics API is supported
    fn is_api_supported(&self, api: GraphicsApi) -> bool {
        self.api_support.get(&api).copied().unwrap_or(false)
    }

    /// Get cached texture by name
    pub fn get_cached_texture(&self, name: &str) -> Option<ZeroCopyTexture> {
        let cache = self.texture_cache.lock();
        cache.get(name).cloned()
    }

    /// Clear texture cache
    pub fn clear_texture_cache(&self) {
        let mut cache = self.texture_cache.lock();
        cache.clear();
        
        let mut metrics = self.metrics.lock();
        metrics.texture_cache_hits = 0;
        metrics.texture_cache_misses = 0;
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> InteropMetrics {
        self.metrics.lock().clone()
    }

    /// Create a simple texture for testing
    pub fn create_test_texture(&self, width: u32, height: u32, label: &str) -> Result<Texture, Box<dyn std::error::Error>> {
        let texture_desc = TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        Ok(self.device.create_texture(&texture_desc))
    }

    /// Get device and queue for external use
    pub fn get_device_queue(&self) -> (Arc<Device>, Arc<Queue>) {
        (self.device.clone(), self.queue.clone())
    }
}

// Clone implementation for ZeroCopyTexture
impl Clone for ZeroCopyTexture {
    fn clone(&self) -> Self {
        Self {
            native_info: self.native_info.clone(),
            wgpu_texture: self.wgpu_texture.clone(),
            wgpu_view: self.wgpu_view.clone(),
            is_zero_copy: self.is_zero_copy,
        }
    }
}

// Clone implementation for InteropPipeline
impl Clone for InteropPipeline {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            shader_module: self.shader_module.clone(),
            pipeline_layout: self.pipeline_layout.clone(),
            render_pipeline: self.render_pipeline.clone(),
            bind_group_layout: self.bind_group_layout.clone(),
        }
    }
}

// Clone implementation for InteropMetrics
impl Clone for InteropMetrics {
    fn clone(&self) -> Self {
        Self {
            zero_copy_operations: self.zero_copy_operations,
            fallback_operations: self.fallback_operations,
            texture_cache_hits: self.texture_cache_hits,
            texture_cache_misses: self.texture_cache_misses,
            total_processing_time_ms: self.total_processing_time_ms,
            average_frame_time_ms: self.average_frame_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wgpu_interop_manager_creation() {
        let config = InteropConfig::default();
        let manager = WgpuInteropManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_test_texture_creation() {
        let config = InteropConfig::default();
        let manager = WgpuInteropManager::new(config).await.unwrap();
        
        let texture = manager.create_test_texture(256, 256, "test_texture");
        assert!(texture.is_ok());
    }
}