//! Rendering pipeline using wgpu for ISF shaders

use crate::{IsfShader, ShaderValue, RenderParameters};
use wgpu::{*, util::DeviceExt};
use std::collections::HashMap;

/// WGPU-based shader renderer
pub struct ShaderRenderer {
    device: Device,
    queue: Queue,
    surface: Option<Surface>,
    config: Option<SurfaceConfiguration>,
    pipelines: HashMap<String, RenderPipeline>,
    bind_groups: HashMap<String, BindGroup>,
    textures: HashMap<String, Texture>,
}

impl ShaderRenderer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create WGPU instance
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        // Create adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find an appropriate adapter")?;

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;

        Ok(Self {
            device,
            queue,
            surface: None,
            config: None,
            pipelines: HashMap::new(),
            bind_groups: HashMap::new(),
            textures: HashMap::new(),
        })
    }

    /// Compile and prepare a shader for rendering
    pub fn prepare_shader(&mut self, shader: &IsfShader) -> Result<(), Box<dyn std::error::Error>> {
        // Convert ISF to WGSL
        let wgsl_source = crate::shader_converter::isf_to_wgsl(shader)?;

        // Create shader module
        let shader_module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&shader.name),
            source: ShaderSource::Wgsl(wgsl_source.into()),
        });

        // Create bind group layout
        let bind_group_layout = self.create_bind_group_layout(shader)?;

        // Create pipeline layout
        let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{}_layout", shader.name)),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&shader.name),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
                    blend: Some(BlendState::REPLACE),
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
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create bind group
        let bind_group = self.create_bind_group(shader, &bind_group_layout)?;

        self.pipelines.insert(shader.name.clone(), pipeline);
        self.bind_groups.insert(shader.name.clone(), bind_group);

        Ok(())
    }

    /// Render a frame with the specified shader
    pub fn render_frame(
        &mut self,
        shader_name: &str,
        params: &RenderParameters,
        input_texture: Option<&Texture>,
        output_texture: &Texture,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pipeline = self.pipelines.get(shader_name)
            .ok_or("Shader not prepared")?;
        let bind_group = self.bind_groups.get(shader_name)
            .ok_or("Bind group not created")?;

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Update uniforms
        self.update_uniforms(shader_name, params)?;

        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &output_texture.create_view(&TextureViewDescriptor::default()),
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

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1); // Draw fullscreen quad

        drop(render_pass);

        // Submit command buffer
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    /// Create bind group layout for shader inputs
    fn create_bind_group_layout(&self, shader: &IsfShader) -> Result<BindGroupLayout, Box<dyn std::error::Error>> {
        let mut entries = Vec::new();

        // Time uniform
        entries.push(BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        // Resolution uniform
        entries.push(BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        // Shader parameters
        for (i, input) in shader.inputs.iter().enumerate() {
            let binding_type = match input.input_type {
                crate::InputType::Float | crate::InputType::Bool => BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                crate::InputType::Color | crate::InputType::Point2D => BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                crate::InputType::Image => BindingType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
            };

            entries.push(BindGroupLayoutEntry {
                binding: 2 + i as u32,
                visibility: ShaderStages::FRAGMENT,
                ty: binding_type,
                count: None,
            });

            // Add sampler for image inputs
            if matches!(input.input_type, crate::InputType::Image) {
                entries.push(BindGroupLayoutEntry {
                    binding: 2 + i as u32 + 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                });
            }
        }

        let layout = self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(&format!("{}_bind_group_layout", shader.name)),
            entries: &entries,
        });

        Ok(layout)
    }

    /// Create bind group with actual resources
    fn create_bind_group(&self, shader: &IsfShader, layout: &BindGroupLayout) -> Result<BindGroup, Box<dyn std::error::Error>> {
        let mut entries = Vec::new();

        // Create uniform buffers
        let time_buffer = self.device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Time Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let resolution_buffer = self.device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Resolution Buffer"),
            contents: bytemuck::cast_slice(&[1920.0f32, 1080.0f32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        entries.push(BindGroupEntry {
            binding: 0,
            resource: time_buffer.as_entire_binding(),
        });

        entries.push(BindGroupEntry {
            binding: 1,
            resource: resolution_buffer.as_entire_binding(),
        });

        // Create parameter buffers
        for (i, input) in shader.inputs.iter().enumerate() {
            let buffer = match &input.value {
                ShaderValue::Float(val) => self.device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some(&format!("{}_buffer", input.name)),
                    contents: bytemuck::cast_slice(&[*val]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                }),
                ShaderValue::Bool(val) => self.device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some(&format!("{}_buffer", input.name)),
                    contents: bytemuck::cast_slice(&[*val as u32]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                }),
                ShaderValue::Color(val) => self.device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some(&format!("{}_buffer", input.name)),
                    contents: bytemuck::cast_slice(val),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                }),
                ShaderValue::Point2D(val) => self.device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some(&format!("{}_buffer", input.name)),
                    contents: bytemuck::cast_slice(val),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                }),
            };

            entries.push(BindGroupEntry {
                binding: 2 + i as u32,
                resource: buffer.as_entire_binding(),
            });
        }

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("{}_bind_group", shader.name)),
            layout,
            entries: &entries,
        });

        Ok(bind_group)
    }

    /// Update uniform values for rendering
    fn update_uniforms(&self, shader_name: &str, params: &RenderParameters) -> Result<(), Box<dyn std::error::Error>> {
        // This would update the uniform buffers with current parameter values
        // Implementation depends on how we store and access the buffers
        Ok(())
    }

    /// Create a texture for rendering
    pub fn create_texture(&self, width: u32, height: u32, format: TextureFormat, usage: TextureUsages) -> Texture {
        self.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    }
}