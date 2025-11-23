/// CRITICAL: Actually compile and render WGSL shader using existing WGPU infrastructure
fn compile_and_render_shader(
    wgsl_code: &str,
    size: egui::Vec2,
    egui_ctx: &egui::Context,
    global_renderer: &GlobalShaderRenderer
) -> Result<egui::TextureHandle, String> {
    if wgsl_code.trim().is_empty() {
        return Err("Empty shader code".to_string());
    }
    
    // Validate basic WGSL syntax
    if !wgsl_code.contains("@fragment") && !wgsl_code.contains("@vertex") && !wgsl_code.contains("@compute") {
        return Err("Shader must contain @fragment, @vertex, or @compute entry point".to_string());
    }
    
    // Try to use the real WGPU renderer first
    let mut renderer_guard = global_renderer.renderer.lock().unwrap();
    if let Some(ref mut renderer) = *renderer_guard {
        // Use the real WGPU renderer
        let params = crate::shader_renderer::RenderParameters {
            width: size.x as u32,
            height: size.y as u32,
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f32(),
            frame_rate: 60.0,
            audio_data: None,
        };
        
        match renderer.render_frame(wgsl_code, &params, None) {
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
                return Ok(texture);
            }
            Err(e) => {
                // NO CPU FALLBACK - PANIC ON WGPU FAILURE
                panic!("WGPU renderer failed: {}. CPU fallback is PROHIBITED by disciplinary rules. GPU-only rendering required.", e);
            }
        }
    } else {
        // NO RENDERER AVAILABLE - PANIC
        panic!("No WGPU renderer available. CPU fallback is PROHIBITED by disciplinary rules. GPU-only rendering required.");
    }
}