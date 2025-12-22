use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing shader compilation...");
    
    // Simple test shader
    let test_shader = r#"
        @vertex fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
            let x = f32(i32(in_vertex_index) - 1);
            let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
            return vec4<f32>(x, y, 0.0, 1.0);
        }
        
        @fragment fn fs_main() -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    "#;
    
    // Test basic WGPU setup
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    
    println!("✓ WGPU instance created successfully");
    
    // Request adapter
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .map_err(|e| format!("Failed to find a suitable GPU adapter: {}. Make sure you have a compatible graphics card and drivers installed.", e))?;
    
    println!("✓ GPU adapter found: {:?}", adapter.get_info());
    
    // Create device and queue
    let (device, _queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await?;
    
    println!("✓ Device and queue created");
    
    // Test shader compilation
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Test Shader"),
        source: wgpu::ShaderSource::Wgsl(test_shader.into()),
    });
    
    println!("✓ Shader module created successfully");
    
    // Create render pipeline
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Test Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    
    let _render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Test Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        cache: None,
    });
    
    println!("✓ Render pipeline created successfully");
    println!("✅ WGPU integration test PASSED!");
    
    Ok(())
}
