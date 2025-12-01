/// CRITICAL: Actually compile and render WGSL shader using existing WGPU infrastructure
fn compile_and_render_shader(
    wgsl_code: &str,
    size: egui::Vec2,
    egui_ctx: &egui::Context,
    global_renderer: &GlobalShaderRenderer,
    parameter_values: &std::collections::HashMap<String, f32>,
    audio_analyzer: Option<&crate::audio_system::AudioAnalyzer>,
    video_exporter: Option<&crate::screenshot_video_export::ScreenshotVideoExporter>
) -> Result<egui::TextureHandle, String> {
    if wgsl_code.trim().is_empty() {
        return Err("Empty shader code".to_string());
    }
    
    // Validate basic WGSL syntax
    if !wgsl_code.contains("@fragment") && !wgsl_code.contains("@vertex") && !wgsl_code.contains("@compute") {
        return Err("Shader must contain @fragment, @vertex, or @compute entry point".to_string());
    }
    
    // Perform WGSL reflection analysis
    let reflection_result = crate::wgsl_reflect_integration::analyze_shader_reflection(wgsl_code);
    match reflection_result {
        Ok(reflection_info) => {
            println!("✅ WGSL Reflection Analysis Complete:");
            println!("  - Entry Points: {}", reflection_info.entry_points.len());
            println!("  - Bind Groups: {}", reflection_info.bind_groups.len());
            println!("  - Uniforms: {}", reflection_info.uniforms.len());
            println!("  - Textures: {}", reflection_info.textures.len());
            println!("  - Samplers: {}", reflection_info.samplers.len());
            println!("  - Storage Buffers: {}", reflection_info.storage_buffers.len());
        }
        Err(e) => {
            println!("⚠️  WGSL Reflection Analysis Failed: {}", e);
            // Continue with compilation even if reflection fails
        }
    }
    
    // Perform WGSL bindgen analysis for uniform layout
    let mut bindgen_analyzer = crate::wgsl_bindgen_integration::WgslBindgenAnalyzer::new();
    match bindgen_analyzer.analyze_shader(wgsl_code, "main_shader") {
        Ok(uniform_layouts) => {
            println!("✅ WGSL Bindgen Analysis Complete:");
            println!("  - Uniform Layouts: {}", uniform_layouts.len());
            for layout in &uniform_layouts {
                println!("    - {}: binding={}, group={}, size={} bytes", 
                    layout.name, layout.binding, layout.group, layout.size);
            }
        }
        Err(e) => {
            println!("⚠️  WGSL Bindgen Analysis Failed: {}", e);
            // Continue with compilation even if bindgen analysis fails
        }
    }
    
    // Try to use the real WGPU renderer first
    let mut renderer_guard = global_renderer.renderer.lock().unwrap();
    if let Some(ref mut renderer) = *renderer_guard {
        // Use the real WGPU renderer with parameter values
        let audio_data = audio_analyzer.map(|analyzer| {
            let data = analyzer.get_audio_data();
            crate::audio_system::AudioData {
                volume: data.volume,
                bass_level: data.bass_level,
                mid_level: data.mid_level,
                treble_level: data.treble_level,
                beat_detected: data.beat_detected,
                beat_intensity: data.beat_intensity,
                tempo: data.tempo,
                frequencies: data.frequencies.clone(),
                waveform: data.waveform.clone(),
            }
        });
        
        let params = crate::shader_renderer::RenderParameters {
            width: size.x as u32,
            height: size.y as u32,
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f32(),
            frame_rate: 60.0,
            audio_data,
        };
        
        // Convert parameter values to array for shader
        let mut param_array = vec![0.0f32; 64];
        for (name, &value) in parameter_values.iter() {
            // Simple hash-based mapping for parameter names to array indices
            let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
            let index = (hash as usize) % 64;
            param_array[index] = value;
        }
        
        match renderer.render_frame_with_params(wgsl_code, &params, Some(&param_array)) {
            Ok(pixel_data) => {
                // Create texture from pixel data
                let texture = egui_ctx.load_texture(
                    "shader_preview_real",
                    egui::ColorImage {
                        size: [params.width as usize, params.height as usize],
                        pixels: pixel_data.chunks(4).map(|chunk| {
                            egui::Color32::from_rgba_unmultiplied(chunk[0], chunk[1], chunk[2], chunk[3])
                        }).collect(),
                        source_size: size,
                    },
                    egui::TextureOptions::default()
                );
                
                // Capture frame for video recording if active
                if let Some(exporter) = video_exporter {
                    // Capture the pixel data for video recording
                    let _ = exporter.capture_frame_from_pixels(&pixel_data, params.width, params.height);
                }
                
                return Ok(texture);
            }
            Err(e) => {
                return Err(format!("GPU rendering initialization failed: {}. Please ensure WGPU-compatible hardware is available.", e));
            }
        }
    }
    
    // GPU-only enforcement - return error instead of panic
    return Err("WGPU renderer unavailable - GPU-only rendering enforced. Hardware GPU required for operation.".to_string());
}