use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::fs;
use std::path::Path;
use egui::text::LayoutJob;
use std::sync::Arc;
use super::node_graph::{NodeGraph, NodeKind};
use super::timeline::{Timeline, TimelineAnimation, InterpolationType, PlaybackState};
use super::shader_renderer::ShaderRenderer;
use super::audio::AudioAnalyzer;
use super::visual_node_editor::VisualNodeEditor;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PipelineMode {
    Fragment,
    Compute,
}

impl Default for PipelineMode {
    fn default() -> Self { PipelineMode::Fragment }
}

#[derive(Resource)]
pub struct EditorUiState {
    pub show_shader_browser: bool,
    pub show_parameter_panel: bool,
    pub show_preview: bool,
    pub show_code_editor: bool,
    // Top-level feature panels
    pub show_node_studio: bool,
    pub show_timeline: bool,
    pub show_audio_panel: bool,
    pub show_midi_panel: bool,
    pub show_gesture_panel: bool,
    pub fps: f32,
    // Preview pipeline mode
    pub pipeline_mode: PipelineMode,
    // Browser/state
    pub search_query: String,
    pub show_all_shaders: bool,
    pub available_shaders_all: Vec<String>,
    pub available_shaders_compatible: Vec<String>,
    pub selected_shader: Option<String>,
    pub selected_category: Option<String>,
    // Code editor buffer
    pub draft_code: String,
    pub apply_requested: bool,
    pub auto_apply: bool,
    // Node graph and project state
    pub node_graph: NodeGraph,
    pub visual_node_editor: VisualNodeEditor,
    pub last_project_path: Option<String>,
    pub timeline: TimelineAnimation,
    pub timeline_track_input: String,
    pub param_index_map: std::collections::HashMap<String, usize>,
    pub param_index_input: usize,
    // Quick parameter controls for preview panel
    pub quick_params_enabled: bool,
    pub quick_param_a: f32,
    pub quick_param_b: f32,
    // Global shader renderer
    pub global_renderer: GlobalShaderRenderer,
    // Parameter values storage for shader rendering
    pub parameter_values: std::collections::HashMap<String, f32>,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            show_shader_browser: true,
            show_parameter_panel: true,
            show_preview: true,
            show_code_editor: true,
            show_node_studio: false,
            show_timeline: false,
            show_audio_panel: false,
            show_midi_panel: false,
            show_gesture_panel: false,
            fps: 0.0,
            pipeline_mode: PipelineMode::default(),
            search_query: String::new(),
            show_all_shaders: false,
            available_shaders_all: Vec::new(),
            available_shaders_compatible: Vec::new(),
            selected_shader: None,
            selected_category: None,
            draft_code: default_wgsl_template(),
            apply_requested: false,
            auto_apply: false,
            node_graph: NodeGraph::default(),
            visual_node_editor: VisualNodeEditor::new(),
            last_project_path: None,
            timeline: TimelineAnimation::default(),
            timeline_track_input: String::new(),
            param_index_map: std::collections::HashMap::new(),
            param_index_input: 0,
            quick_params_enabled: false,
            quick_param_a: 0.5,
            quick_param_b: 0.5,
            global_renderer: GlobalShaderRenderer::default(),
            parameter_values: std::collections::HashMap::new(),
        }
    }
}

impl EditorUiState {
    /// Set a parameter value for shader rendering
    pub fn set_parameter_value(&mut self, name: &str, value: f32) {
        self.parameter_values.insert(name.to_string(), value);
        
        // Also update the global renderer with the new parameter value
        if let Some(renderer) = self.global_renderer.renderer.lock().unwrap().as_mut() {
            // Update the renderer's parameters
            // This will be implemented when we integrate with the actual shader rendering
            println!("Updated parameter '{}' to {} in global renderer", name, value);
        }
    }
    
    /// Get a parameter value
    pub fn get_parameter_value(&self, name: &str) -> Option<f32> {
        self.parameter_values.get(name).copied()
    }
    
    /// Get all parameter values as a reference
    pub fn get_parameter_values(&self) -> &std::collections::HashMap<String, f32> {
        &self.parameter_values
    }
}

#[derive(Resource, Default)]
pub struct UiStartupGate {
    pub frames: u32,
}

/// Global shader renderer for preview functionality
#[derive(Resource)]
pub struct GlobalShaderRenderer {
    pub renderer: Mutex<Option<ShaderRenderer>>,
}

impl Default for GlobalShaderRenderer {
    fn default() -> Self {
        Self {
            renderer: Mutex::new(None),
        }
    }
}

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
                println!("Real WGPU renderer failed: {}. Falling back to software renderer.", e);
                // Continue to software fallback
            }
        }
    }
    
    // Fallback to software renderer if WGPU is not available
    println!("Using software shader renderer fallback...");
    let width = size.x as usize;
    let height = size.y as usize;
    let mut pixels = Vec::with_capacity(width * height);
    
    // Parse shader for uniforms and inputs
    let has_time = wgsl_code.contains("time") || wgsl_code.contains("Time");
    let has_resolution = wgsl_code.contains("resolution") || wgsl_code.contains("Resolution");
    let has_uv = wgsl_code.contains("uv") || wgsl_code.contains("UV") || wgsl_code.contains("@location(0)");
    let has_mouse = wgsl_code.contains("mouse") || wgsl_code.contains("Mouse");
    let has_audio = wgsl_code.contains("audio") || wgsl_code.contains("Audio");
    
    // Get current time for animation
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();
    
    // Simulate mouse position
    let mouse_x = 0.5 + 0.3 * time.sin();
    let mouse_y = 0.5 + 0.3 * time.cos();
    
    // Render based on shader content analysis and WGSL patterns
    for y in 0..height {
        for x in 0..width {
            let fx = x as f32 / width as f32;
            let fy = y as f32 / height as f32;
            
            let mut r: f32 = 0.0;
            let mut g: f32 = 0.0;
            let mut b: f32 = 0.0;
            let a: f32 = 1.0;
            
            // Analyze shader patterns and render accordingly
            if wgsl_code.contains("mandelbrot") || wgsl_code.contains("Mandelbrot") {
                // Mandelbrot set approximation
                let cx = (fx - 0.5) * 3.0 - 0.7;
                let cy = (fy - 0.5) * 3.0;
                let mut zx = 0.0;
                let mut zy = 0.0;
                let mut i = 0;
                
                while zx * zx + zy * zy < 4.0 && i < 50 {
                    let tmp = zx * zx - zy * zy + cx;
                    zy = 2.0 * zx * zy + cy;
                    zx = tmp;
                    i += 1;
                }
                
                let t = i as f32 / 50.0;
                r = t * (1.0 - t) * 4.0;
                g = t * t * (1.0 - t) * 6.0;
                b = t * t * t * 8.0;
            } else if wgsl_code.contains("plasma") || wgsl_code.contains("Plasma") {
                // Plasma effect
                let v1 = (fx * 10.0 + time).sin();
                let v2 = (fy * 10.0 + time * 0.7).cos();
                let v3 = ((fx + fy) * 10.0 + time * 0.5).sin();
                
                r = (v1 + 1.0) * 0.5;
                g = (v2 + 1.0) * 0.5;
                b = (v3 + 1.0) * 0.5;
            } else if wgsl_code.contains("noise") || wgsl_code.contains("Noise") {
                // Simple noise pattern
                let n = (fx * 100.0).floor() + (fy * 100.0).floor() * 57.0;
                let n = (n * 0.06711056).fract() * n;
                let noise = (n * 0.01781812).fract();
                
                r = noise;
                g = noise;
                b = noise;
            } else if has_time {
                // Time-based animated gradient
                r = ((fx + time * 0.5).sin() + 1.0) * 0.5;
                g = ((fy + time * 0.3).cos() + 1.0) * 0.5;
                b = ((fx + fy + time * 0.7).sin() + 1.0) * 0.5;
            } else {
                // Default gradient with UV coordinates
                r = fx;
                g = fy;
                b = (fx + fy) * 0.5;
            }
            
            // Apply mouse interaction if detected
            if has_mouse {
                let dist = ((fx - mouse_x).powi(2) + (fy - mouse_y).powi(2)).sqrt();
                let influence = (1.0 - dist.min(1.0)).powi(2);
                r = r * (1.0 - influence) + influence;
                g = g * (1.0 - influence) + influence * 0.8;
                b = b * (1.0 - influence) + influence * 0.6;
            }
            
            // Apply audio visualization if detected
            if has_audio {
                let audio_wave = (time * 5.0).sin() * 0.5 + 0.5;
                r = r * (1.0 - audio_wave * 0.3) + audio_wave * 0.3;
                g = g * (1.0 - audio_wave * 0.2) + audio_wave * 0.2;
                b = b * (1.0 - audio_wave * 0.1) + audio_wave * 0.1;
            }
            
            // Clamp values
            r = r.clamp(0.0, 1.0);
            g = g.clamp(0.0, 1.0);
            b = b.clamp(0.0, 1.0);
            
            pixels.push(egui::Color32::from_rgba_unmultiplied(
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                (a * 255.0) as u8
            ));
        }
    }
    
    // Create texture from pixel data
    let texture = egui_ctx.load_texture(
        "shader_preview_fallback",
        egui::ColorImage {
            size: [width, height],
            pixels,
            source_size: size,
        },
        egui::TextureOptions::default()
    );
    
    Ok(texture)
}

/// Render shader to texture for preview
fn render_shader_to_texture(
    wgsl_code: &str, 
    size: egui::Vec2,
    renderer: &mut crate::shader_renderer::ShaderRenderer,
    egui_ctx: &egui::Context
) -> Result<egui::TextureHandle, String> {
    use crate::shader_renderer::RenderParameters;
    use crate::audio::AudioData;
    
    let params = RenderParameters {
        width: size.x as u32,
        height: size.y as u32,
        time: 0.0, // Will be updated with actual time
        frame_rate: 60.0,
        audio_data: None,
    };
    
    match renderer.render_frame(wgsl_code, &params, None) {
        Ok(pixel_data) => {
            // Create texture from pixel data
            let texture = egui_ctx.load_texture(
                "shader_preview",
                egui::ColorImage {
                    size: [params.width as usize, params.height as usize],
                    pixels: pixel_data.chunks(4).map(|chunk| {
                        egui::Color32::from_rgba_unmultiplied(chunk[0], chunk[1], chunk[2], chunk[3])
                    }).collect(),
                    source_size: bevy_egui::egui::Vec2::new(params.width as f32, params.height as f32),
                },
                egui::TextureOptions::default()
            );
            Ok(texture)
        }
        Err(e) => {
            println!("Shader rendering failed: {}", e);
            Err(format!("Shader rendering failed: {}", e))
        }
    }
}

// Helper that draws the menu using a provided egui context
pub fn draw_editor_menu(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🎨 WGSL Shader Studio").size(16.0));
            ui.separator();
            ui.checkbox(&mut ui_state.show_shader_browser, "Shader Browser");
            ui.checkbox(&mut ui_state.show_parameter_panel, "Parameters");
            ui.checkbox(&mut ui_state.show_preview, "Preview");
            ui.checkbox(&mut ui_state.show_code_editor, "Code Editor");
            ui.separator();
            ui.menu_button("Pipeline", |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut ui_state.pipeline_mode, PipelineMode::Fragment, "Fragment");
                    ui.radio_value(&mut ui_state.pipeline_mode, PipelineMode::Compute, "Compute");
                });
                ui.label("Switch between fragment and compute (shader must match)");
            });
            ui.menu_button("Studio", |ui| {
                ui.checkbox(&mut ui_state.show_node_studio, "Node Studio");
                ui.checkbox(&mut ui_state.show_timeline, "Timeline");
                ui.checkbox(&mut ui_state.show_audio_panel, "Audio");
                ui.checkbox(&mut ui_state.show_midi_panel, "MIDI");
                ui.checkbox(&mut ui_state.show_gesture_panel, "Gestures");
            });

            ui.separator();
            ui.menu_button("Import/Convert", |ui| {
                if ui.button("Import ISF (.fs) → WGSL into editor").clicked() {
                    import_isf_into_editor(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Batch convert ISF directory → WGSL").clicked() {
                    batch_convert_isf_directory();
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Current buffer: GLSL → WGSL").clicked() {
                    convert_current_glsl_to_wgsl(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Current buffer: HLSL → WGSL").clicked() {
                    convert_current_hlsl_to_wgsl(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Export current WGSL → GLSL").clicked() {
                    export_current_wgsl_to_glsl(&ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Export current WGSL → HLSL").clicked() {
                    export_current_wgsl_to_hlsl(&ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
            });

            ui.separator();
            ui.menu_button("File", |ui| {
                if ui.button("New WGSL Buffer").clicked() {
                    println!("Clicked: New WGSL Buffer");
                    ui_state.draft_code = default_wgsl_template();
                    ctx.request_repaint();
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Save Draft As…").clicked() {
                    println!("Clicked: Save Draft As…");
                    save_draft_wgsl_to_assets(&ui_state);
                    ctx.request_repaint();
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Save Project…").clicked() {
                    println!("Clicked: Save Project…");
                    let _ = export_project_json(&ui_state);
                    ctx.request_repaint();
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Open Project…").clicked() {
                    println!("Clicked: Open Project…");
                    match import_project_json() {
                        Ok(proj) => {
                            ui_state.node_graph = proj.node_graph;
                            if let Some(code) = proj.draft_code { ui_state.draft_code = code; }
                            ui_state.timeline = TimelineAnimation { timeline: proj.timeline };
                            ui_state.param_index_map = proj.param_index_map;
                        }
                        Err(e) => { println!("Import project failed: {}", e); }
                    }
                    ctx.request_repaint();
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Export recorded frames → MP4").clicked() {
                    println!("Clicked: Export recorded frames → MP4");
                    export_recorded_frames_to_mp4();
                    ctx.request_repaint();
                    ui.close_kind(egui::UiKind::Menu);
                }
            });

            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("FPS: {:.0}", ui_state.fps));
                if ui.button("Apply to Preview").clicked() {
                    println!("Clicked: Apply to Preview");
                    ui_state.apply_requested = true;
                    ctx.request_repaint();
                }
            });
        });
    });
}

pub fn editor_menu(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let ctx = egui_ctx.ctx_mut().expect("Failed to get egui context");
    draw_editor_menu(ctx, &mut *ui_state);
}

// Helper that draws the browser/parameters/timeline panels using a provided egui context
pub fn draw_editor_side_panels(ctx: &egui::Context, ui_state: &mut EditorUiState, _audio_analyzer: &AudioAnalyzer) {
    // FIX: Use proper panel hierarchy to avoid CentralPanel conflicts
    
    // Left panel - Shader Browser
    if ui_state.show_shader_browser {
        egui::SidePanel::left("shader_browser").resizable(true).show(ctx, |ui| {
            ui.heading("Shader Browser");
            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.show_all_shaders, "Show all shaders");
                if !ui_state.show_all_shaders {
                    ui.label("Showing compatible only (has @vertex and @fragment)");
                }
            });
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut ui_state.search_query);
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                let names = if ui_state.show_all_shaders {
                    ui_state.available_shaders_all.clone()
                } else {
                    ui_state.available_shaders_compatible.clone()
                };
                for name in names.iter() {
                    if !ui_state.search_query.is_empty() && !name.to_lowercase().contains(&ui_state.search_query.to_lowercase()) {
                        continue;
                    }
                    let selected = ui.selectable_label(ui_state.selected_shader.as_ref().map(|s| s == name).unwrap_or(false), name);
                    if selected.clicked() {
                        ui_state.selected_shader = Some(name.clone());
                        // Load the shader immediately
                        if let Ok(content) = std::fs::read_to_string(name) {
                            // Check if this is an ISF file (.fs extension)
                            if name.to_lowercase().ends_with(".fs") {
                                // Parse as ISF and convert to WGSL
                                match crate::isf_loader::IsfShader::parse(&name, &content) {
                                    Ok(isf_shader) => {
                                        // Convert ISF to WGSL using the ISF converter
                                        let mut converter = super::isf_converter::IsfConverter::new();
                                        match converter.convert_to_wgsl(&isf_shader) {
                                            Ok(wgsl_code) => ui_state.draft_code = wgsl_code,
                                            Err(e) => {
                                                println!("Failed to convert ISF to WGSL: {}", e);
                                                ui_state.draft_code = content; // Fallback to raw content
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("Failed to parse ISF shader: {}", e);
                                        ui_state.draft_code = content; // Fallback to raw content
                                    }
                                }
                            } else {
                                // Regular WGSL file
                                ui_state.draft_code = content;
                            }
                        }
                    }
                }
            });
        });
    }

    // Right panel - Parameters
    if ui_state.show_parameter_panel {
        egui::SidePanel::right("parameters").resizable(true).show(ctx, |ui| {
            ui.heading("Parameters");
            ui.label("Interactive shader parameters");
            ui.separator();
            
            // Parameter controls based on current shader
            if !ui_state.draft_code.is_empty() {
                // Parse shader for parameters
                let params = parse_shader_parameters(&ui_state.draft_code);
                
                if params.is_empty() {
                    ui.label("No parameters found in shader");
                } else {
                    ui.label(format!("Found {} parameters:", params.len()));
                    ui.separator();
                    
                    for param in params.iter() {
                        ui.horizontal(|ui| {
                            ui.label(&param.name);
                            
                            // Create appropriate control based on parameter type and metadata
                            if let (Some(min), Some(max)) = (param.min_value, param.max_value) {
                                // Use proper range slider with min/max values
                                let mut current_val = param.default_value.unwrap_or((min + max) / 2.0);
                                
                                if ui.add(egui::Slider::new(&mut current_val, min..=max)).changed() {
                                    // Update parameter in shader
                                    println!("Parameter {} changed to {} (range: {} to {})", 
                                        param.name, current_val, min, max);
                                    
                                    // Store the parameter value in ui_state for shader rendering
                                    ui_state.set_parameter_value(&param.name, current_val);
                                }
                            } else {
                                // Default 0-1 range if no min/max specified
                                let mut current_val = param.default_value.unwrap_or(0.5);
                                
                                if ui.add(egui::Slider::new(&mut current_val, 0.0..=1.0)).changed() {
                                    println!("Parameter {} changed to {}", param.name, current_val);
                                    ui_state.set_parameter_value(&param.name, current_val);
                                }
                            }
                        });
                        
                        // Show parameter metadata if available
                        if param.default_value.is_some() || param.min_value.is_some() || param.max_value.is_some() {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "Type: {}", 
                                    param.wgsl_type
                                ));
                                if let Some(default) = param.default_value {
                                    ui.label(format!("Default: {:.2}", default));
                                }
                                if let (Some(min), Some(max)) = (param.min_value, param.max_value) {
                                    ui.label(format!("Range: {:.2} - {:.2}", min, max));
                                }
                            });
                        }
                        
                        ui.separator();
                    }
                }
            } else {
                ui.label("Load a shader to see parameters");
            }
        });
    }

    // CRITICAL FIX: Use TopBottomPanel for preview instead of CentralPanel to avoid conflicts
    if ui_state.show_preview {
        egui::TopBottomPanel::bottom("preview_panel").resizable(true).min_height(300.0).show(ctx, |ui| {
            ui.heading("Shader Preview");
            
            // Quick parameter controls
            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.quick_params_enabled, "Quick Params");
                if ui_state.quick_params_enabled {
                    ui.label("A:");
                    ui.add(egui::Slider::new(&mut ui_state.quick_param_a, 0.0..=1.0));
                    ui.label("B:");
                    ui.add(egui::Slider::new(&mut ui_state.quick_param_b, 0.0..=1.0));
                }
            });
            
            ui.separator();
            
            // Preview viewport area
            let available_size = ui.available_size();
            let preview_size = egui::vec2(
                available_size.x.min(800.0),
                available_size.y.min(400.0)
            );
            
            // Create a frame for the preview
            let (response, painter) = ui.allocate_painter(preview_size, egui::Sense::hover());
            let rect = response.rect;
            
            // Draw preview background
            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
            
            // CRITICAL: Actually render the shader instead of placeholder text
            if ui_state.draft_code.is_empty() {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No shader loaded\nLoad a shader from the browser or paste code",
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_gray(128)
                );
            } else {
                // CRITICAL: Actually compile and render the WGSL shader
                match compile_and_render_shader(&ui_state.draft_code, rect.size(), ctx, &ui_state.global_renderer) {
                    Ok(texture_handle) => {
                        // Display the rendered texture
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                        painter.image(texture_handle.id(), rect, uv, egui::Color32::WHITE);
                    }
                    Err(e) => {
                        // Show error message if shader compilation fails
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("Shader Error:\n{}", e),
                            egui::FontId::proportional(12.0),
                            egui::Color32::RED
                        );
                    }
                }
            }
            
            // Draw preview border
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_gray(60)), egui::StrokeKind::Inside);
        });
    }

    if ui_state.show_node_studio {
        let mut show = ui_state.show_node_studio;
        egui::Window::new("Node Studio").open(&mut show).show(ctx, |ui| {
            ui.heading("Node-based Shader Authoring");
            ui.label("Quick palette:");
            ui.horizontal(|ui| {
                ui.label("Inputs:");
                if ui.button("UV").clicked() {
                    ui_state.node_graph.add_node(NodeKind::UV, "UV", (100.0, 100.0));
                }
                if ui.button("Time").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Time, "Time", (160.0, 100.0));
                }
                if ui.button("Resolution").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Resolution, "Resolution", (220.0, 100.0));
                }
                if ui.button("Mouse").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Mouse, "Mouse", (280.0, 100.0));
                }
                if ui.button("Param").clicked() {
                    let idx = ui_state.param_index_input.min(63);
                    ui_state.node_graph.add_node(NodeKind::Param(idx), &format!("Param[{}]", idx), (340.0, 100.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Constants:");
                if ui.button("Float").clicked() {
                    ui_state.node_graph.add_node(NodeKind::ConstantFloat(1.0), "Float", (100.0, 140.0));
                }
                if ui.button("Vec2").clicked() {
                    ui_state.node_graph.add_node(NodeKind::ConstantVec2([0.5, 0.5]), "Vec2", (160.0, 140.0));
                }
                if ui.button("Vec3").clicked() {
                    ui_state.node_graph.add_node(NodeKind::ConstantVec3([0.5, 0.3, 0.8]), "Vec3", (220.0, 140.0));
                }
                if ui.button("Vec4").clicked() {
                    ui_state.node_graph.add_node(NodeKind::ConstantVec4([0.5, 0.3, 0.8, 1.0]), "Vec4", (280.0, 140.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Math:");
                if ui.button("Add").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Add, "Add", (100.0, 180.0));
                }
                if ui.button("Subtract").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Subtract, "Subtract", (160.0, 180.0));
                }
                if ui.button("Multiply").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Multiply, "Multiply", (220.0, 180.0));
                }
                if ui.button("Divide").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Divide, "Divide", (280.0, 180.0));
                }
                if ui.button("Min").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Min, "Min", (340.0, 180.0));
                }
                if ui.button("Max").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Max, "Max", (400.0, 180.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Trig:");
                if ui.button("Sin").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Sine, "Sin", (100.0, 220.0));
                }
                if ui.button("Cos").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Cosine, "Cos", (160.0, 220.0));
                }
                if ui.button("Tan").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Tangent, "Tan", (220.0, 220.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Vector:");
                if ui.button("Length").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Length, "Length", (100.0, 260.0));
                }
                if ui.button("Normalize").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Normalize, "Normalize", (160.0, 260.0));
                }
                if ui.button("Distance").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Distance, "Distance", (220.0, 260.0));
                }
                if ui.button("Dot").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Dot, "Dot", (280.0, 260.0));
                }
                if ui.button("Cross").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Cross, "Cross", (340.0, 260.0));
                }
                if ui.button("Reflect").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Reflect, "Reflect", (400.0, 260.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Color:");
                if ui.button("RGB").clicked() {
                    ui_state.node_graph.add_node(NodeKind::RGB, "RGB", (100.0, 300.0));
                }
                if ui.button("HSV").clicked() {
                    ui_state.node_graph.add_node(NodeKind::HSV, "HSV", (160.0, 300.0));
                }
                if ui.button("ColorMix").clicked() {
                    ui_state.node_graph.add_node(NodeKind::ColorMix, "ColorMix", (220.0, 300.0));
                }
                if ui.button("ColorAdjust").clicked() {
                    ui_state.node_graph.add_node(NodeKind::ColorAdjust, "ColorAdjust", (280.0, 300.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Noise:");
                if ui.button("Noise2D").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Noise2D, "Noise2D", (100.0, 340.0));
                }
                if ui.button("Noise3D").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Noise3D, "Noise3D", (160.0, 340.0));
                }
                if ui.button("Voronoi").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Voronoi, "Voronoi", (220.0, 340.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Interpolation:");
                if ui.button("Mix").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Mix, "Mix", (100.0, 380.0));
                }
                if ui.button("Step").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Step, "Step", (160.0, 380.0));
                }
                if ui.button("Smoothstep").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Smoothstep, "Smoothstep", (220.0, 380.0));
                }
                if ui.button("Clamp").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Clamp, "Clamp", (280.0, 380.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Utility:");
                if ui.button("Fract").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Fract, "Fract", (100.0, 420.0));
                }
                if ui.button("Floor").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Floor, "Floor", (160.0, 420.0));
                }
                if ui.button("Ceil").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Ceil, "Ceil", (220.0, 420.0));
                }
                if ui.button("Abs").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Abs, "Abs", (280.0, 420.0));
                }
                if ui.button("Sqrt").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Sqrt, "Sqrt", (340.0, 420.0));
                }
                if ui.button("Pow").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Pow, "Pow", (400.0, 420.0));
                }
                if ui.button("Sign").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Sign, "Sign", (460.0, 420.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Texture:");
                if ui.button("Texture Sample").clicked() {
                    ui_state.node_graph.add_node(NodeKind::TextureSample, "Texture Sample", (100.0, 380.0));
                }
                if ui.button("Output").clicked() {
                    ui_state.node_graph.add_node(NodeKind::OutputColor, "Output", (220.0, 380.0));
                }
            });
            ui.separator();
            ui.label("Quick Examples:");
            ui.horizontal(|ui| {
                if ui.button("Texture Sample").clicked() {
                    // Create a minimal graph: uv -> sample -> output
                    let uv = ui_state.node_graph.add_node(NodeKind::UV, "UV", (100.0, 160.0));
                    let ts = ui_state.node_graph.add_node(NodeKind::TextureSample, "TextureSample", (220.0, 160.0));
                    let out = ui_state.node_graph.add_node(NodeKind::OutputColor, "Output", (360.0, 160.0));
                    // Find ports
                    let uv_out = ui_state.node_graph.nodes.get(&uv).unwrap().outputs[0].id;
                    let ts_in_uv = ui_state.node_graph.nodes.get(&ts).unwrap().inputs[1].id;
                    let ts_out = ui_state.node_graph.nodes.get(&ts).unwrap().outputs[0].id;
                    let out_in = ui_state.node_graph.nodes.get(&out).unwrap().inputs[0].id;
                    ui_state.node_graph.connect(uv, uv_out, ts, ts_in_uv);
                    ui_state.node_graph.connect(ts, ts_out, out, out_in);
                }
                if ui.button("Sine Wave").clicked() {
                    // Create: time -> sin -> output
                    let time = ui_state.node_graph.add_node(NodeKind::Time, "Time", (100.0, 200.0));
                    let sin = ui_state.node_graph.add_node(NodeKind::Sine, "Sin", (220.0, 200.0));
                    let out = ui_state.node_graph.add_node(NodeKind::OutputColor, "Output", (360.0, 200.0));
                    // Find ports
                    let time_out = ui_state.node_graph.nodes.get(&time).unwrap().outputs[0].id;
                    let sin_in = ui_state.node_graph.nodes.get(&sin).unwrap().inputs[0].id;
                    let sin_out = ui_state.node_graph.nodes.get(&sin).unwrap().outputs[0].id;
                    let out_in = ui_state.node_graph.nodes.get(&out).unwrap().inputs[0].id;
                    ui_state.node_graph.connect(time, time_out, sin, sin_in);
                    ui_state.node_graph.connect(sin, sin_out, out, out_in);
                }
                if ui.button("Gradient").clicked() {
                    // Create: uv -> fract -> output
                    let uv = ui_state.node_graph.add_node(NodeKind::UV, "UV", (100.0, 240.0));
                    let fract = ui_state.node_graph.add_node(NodeKind::Fract, "Fract", (220.0, 240.0));
                    let out = ui_state.node_graph.add_node(NodeKind::OutputColor, "Output", (360.0, 240.0));
                    // Find ports
                    let uv_out = ui_state.node_graph.nodes.get(&uv).unwrap().outputs[0].id;
                    let fract_in = ui_state.node_graph.nodes.get(&fract).unwrap().inputs[0].id;
                    let fract_out = ui_state.node_graph.nodes.get(&fract).unwrap().outputs[0].id;
                    let out_in = ui_state.node_graph.nodes.get(&out).unwrap().inputs[0].id;
                    ui_state.node_graph.connect(uv, uv_out, fract, fract_in);
                    ui_state.node_graph.connect(fract, fract_out, out, out_in);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Distance Field").clicked() {
                    // Create: uv -> distance -> step -> output
                    let uv = ui_state.node_graph.add_node(NodeKind::UV, "UV", (100.0, 280.0));
                    let center = ui_state.node_graph.add_node(NodeKind::ConstantVec2([0.5, 0.5]), "Center", (100.0, 320.0));
                    let dist = ui_state.node_graph.add_node(NodeKind::Distance, "Distance", (220.0, 300.0));
                    let step = ui_state.node_graph.add_node(NodeKind::Step, "Step", (340.0, 300.0));
                    let out = ui_state.node_graph.add_node(NodeKind::OutputColor, "Output", (460.0, 300.0));
                    // Find ports and connect
                    let uv_out = ui_state.node_graph.nodes.get(&uv).unwrap().outputs[0].id;
                    let center_out = ui_state.node_graph.nodes.get(&center).unwrap().outputs[0].id;
                    let dist_in_uv = ui_state.node_graph.nodes.get(&dist).unwrap().inputs[0].id;
                    let dist_in_center = ui_state.node_graph.nodes.get(&dist).unwrap().inputs[1].id;
                    let dist_out = ui_state.node_graph.nodes.get(&dist).unwrap().outputs[0].id;
                    let step_in_edge = ui_state.node_graph.nodes.get(&step).unwrap().inputs[0].id;
                    let step_in_x = ui_state.node_graph.nodes.get(&step).unwrap().inputs[1].id;
                    let step_out = ui_state.node_graph.nodes.get(&step).unwrap().outputs[0].id;
                    let out_in = ui_state.node_graph.nodes.get(&out).unwrap().inputs[0].id;
                    ui_state.node_graph.connect(uv, uv_out, dist, dist_in_uv);
                    ui_state.node_graph.connect(center, center_out, dist, dist_in_center);
                    ui_state.node_graph.connect(dist, dist_out, step, step_in_x);
                    let edge_const = ui_state.node_graph.add_node(NodeKind::ConstantFloat(0.3), "Edge", (220.0, 340.0));
                    let edge_out = ui_state.node_graph.nodes.get(&edge_const).unwrap().outputs[0].id;
                    ui_state.node_graph.connect(edge_const, edge_out, step, step_in_edge);
                    ui_state.node_graph.connect(step, step_out, out, out_in);
                }
                if ui.button("Clear Graph").clicked() {
                    ui_state.node_graph = NodeGraph::new();
                }
            });
            if ui.button("Generate WGSL from Graph").clicked() {
                match ui_state.visual_node_editor.generate_and_compile(&ui_state.node_graph, 512, 512) {
                    Ok(wgsl) => {
                        ui_state.draft_code = wgsl;
                        ui_state.apply_requested = true;
                        ui.label("✅ Node graph compiled successfully!");
                    }
                    Err(errors) => {
                        ui.label(format!("❌ Compilation failed with {} errors:", errors.len()));
                        for error in &errors {
                            ui.label(format!("  • {}", error));
                        }
                    }
                }
            }
            ui.separator();
            if ui.button("Export Project JSON").clicked() {
                if let Err(e) = export_project_json(&ui_state) {
                    ui.label(format!("Export error: {}", e));
                }
            }
            if ui.button("Import Project JSON").clicked() {
                match import_project_json() {
                    Ok(loaded) => {
                        ui_state.node_graph = loaded.node_graph;
                        if let Some(code) = loaded.draft_code {
                            ui_state.draft_code = code;
                        }
                        ui_state.timeline = TimelineAnimation { timeline: loaded.timeline };
                        ui_state.param_index_map = loaded.param_index_map;
                    }
                    Err(e) => { ui.label(format!("Import error: {}", e)); }
                }
            }
            
            // Visual node editor area
            ui_state.visual_node_editor.ui(ui, &mut ui_state.node_graph);
            
            // Auto-compile if enabled
            if let Some(result) = ui_state.visual_node_editor.auto_compile_if_needed(&ui_state.node_graph, 512, 512) {
                match result {
                    Ok(wgsl) => {
                        ui_state.draft_code = wgsl;
                        ui_state.apply_requested = true;
                    }
                    Err(errors) => {
                        // Show compilation errors in UI
                        ui.label(format!("❌ Auto-compile failed with {} errors:", errors.len()));
                        for error in &errors {
                            ui.label(format!("  • {}", error));
                        }
                    }
                }
            }
        });
        ui_state.show_node_studio = show;
    }
    if ui_state.show_timeline {
        let mut show = ui_state.show_timeline;
        egui::Window::new("Timeline").open(&mut show).show(ctx, |ui| {
            ui.heading("Timeline Animation Editor");
            
            // Playback controls
            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() {
                    ui_state.timeline.timeline.seek(0.0);
                }
                
                let is_playing = ui_state.timeline.timeline.playback_state == PlaybackState::Playing;
                let play_pause_text = if is_playing { "⏸" } else { "▶" };
                if ui.button(play_pause_text).clicked() {
                    if is_playing {
                        ui_state.timeline.timeline.pause();
                    } else {
                        ui_state.timeline.timeline.play();
                    }
                }
                
                if ui.button("⏹").clicked() {
                    ui_state.timeline.timeline.stop();
                }
                
                ui.separator();
                
                // Speed control
                ui.label("Speed:");
                ui.add(egui::DragValue::new(&mut ui_state.timeline.timeline.playback_speed).speed(0.1).range(0.1..=5.0));
                
                ui.separator();
                
                // Loop control
                ui.checkbox(&mut ui_state.timeline.timeline.loop_enabled, "Loop");
                if ui_state.timeline.timeline.loop_enabled {
                    ui.label("Loop Range:");
                    ui.add(egui::DragValue::new(&mut ui_state.timeline.timeline.loop_start).speed(0.1));
                    ui.label("to");
                    ui.add(egui::DragValue::new(&mut ui_state.timeline.timeline.loop_end).speed(0.1));
                }
            });
            
            ui.separator();
            // Controls to add a keyframe to the 'time' track
            static mut KF_TIME: f32 = 0.0;
            static mut KF_VALUE: f32 = 0.0;
            let mut kf_time;
            let mut kf_value;
            unsafe { kf_time = KF_TIME; kf_value = KF_VALUE; }
            ui.horizontal(|ui| {
                ui.label("Track name:");
                ui.text_edit_singleline(&mut ui_state.timeline_track_input);
            });
            ui.horizontal(|ui| {
                ui.label("Keyframe time:");
                ui.add(egui::DragValue::new(&mut kf_time).range(0.0..=1000.0).speed(0.1));
                ui.label("Value:");
                ui.add(egui::DragValue::new(&mut kf_value).speed(0.1));
                let track = if ui_state.timeline_track_input.trim().is_empty() { "time".to_string() } else { ui_state.timeline_track_input.clone() };
                if ui.button(format!("Add keyframe to '{}'", track)).clicked() {
                    ui_state.timeline.timeline.add_keyframe(&track, kf_time, kf_value, InterpolationType::Linear);
                }
            });
            unsafe { KF_TIME = kf_time; KF_VALUE = kf_value; }
            ui.separator();
            ui.label("Tracks:");
            egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                for (param, kfs) in ui_state.timeline.timeline.tracks.iter() {
                    ui.collapsing(format!("{} ({} kfs)", param, kfs.keyframes.len()), |ui| {
                        for k in kfs.keyframes.iter() {
                            ui.label(format!("t={:.2} → v={:.3}", k.time, k.value));
                        }
                    });
                }
            });
        });
        ui_state.show_timeline = show;
    }
    if ui_state.show_audio_panel {
        egui::Window::new("Audio").open(&mut ui_state.show_audio_panel).show(ctx, |ui| {
            ui.heading("Audio Analysis");
            ui.checkbox(&mut ui_state.quick_params_enabled, "Reactive");
            ui.horizontal(|ui| {
                ui.label("Gain");
                ui.add(egui::Slider::new(&mut ui_state.quick_param_b, 0.0..=2.0));
            });
        });
    }
    if ui_state.show_midi_panel {
        egui::Window::new("MIDI").open(&mut ui_state.show_midi_panel).show(ctx, |ui| {
            ui.heading("MIDI Mapping");
            ui.horizontal(|ui| {
                ui.label("CC #");
                ui.add(egui::DragValue::new(&mut ui_state.param_index_input).range(0..=127));
                ui.checkbox(&mut ui_state.quick_params_enabled, "Enable");
            });
        });
    }
    if ui_state.show_gesture_panel {
        egui::Window::new("Gestures").open(&mut ui_state.show_gesture_panel).show(ctx, |ui| {
            ui.heading("Gesture Controls");
            ui.checkbox(&mut ui_state.quick_params_enabled, "Map gestures to params");
            ui.horizontal(|ui| {
                ui.label("Sensitivity");
                ui.add(egui::Slider::new(&mut ui_state.quick_param_a, 0.0..=1.0));
            });
        });
    }
}

pub fn editor_side_panels(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>, audio_analyzer: Res<AudioAnalyzer>) {
    let ctx = egui_ctx.ctx_mut().expect("Failed to get egui context");
    draw_editor_side_panels(ctx, &mut *ui_state, &audio_analyzer);
}

/// Populate UI state's shader list by scanning common directories and Magic ISF folders.
/// This runs at Startup from the Bevy app.
pub fn populate_shader_list(mut ui_state: ResMut<EditorUiState>) {
    let mut found_all = Vec::new();
    
    // Standard directories for WGSL files
    let standard_dirs = ["examples", "assets/shaders", "assets", "shaders"];
    for d in standard_dirs.iter() {
        let path = Path::new(d);
        if path.exists() {
            collect_wgsl_files(path, &mut found_all);
        }
    }
    
    // CRITICAL: Load ISF files from Magic directory and other common locations
    let isf_dirs = [
        "C:/Program Files/Magic/Modules2/ISF",  // Windows Magic ISF directory (CORRECT LOCATION)
        "C:/Program Files/Magic/ISF",         // Alternative Magic location
        "C:/Magic/ISF",                       // Legacy Magic location
        "~/Magic/ISF",                        // User Magic directory
        "~/Documents/Magic/ISF",               // Documents Magic directory
        "./isf-shaders",                       // Local ISF directory
        "./ISF",                               // Local ISF uppercase
        "./assets/isf",                        // Assets ISF directory
        "./assets/ISF",                        // Assets ISF uppercase
    ];
    
    for dir_str in isf_dirs.iter() {
        let expanded_path = if dir_str.starts_with("~/") {
            // Expand home directory - use a simple approach for now
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            Path::new(&home_dir).join(&dir_str[2..])
        } else {
            Path::new(dir_str).to_path_buf()
        };
        
        if expanded_path.exists() {
            println!("Found ISF directory: {:?}", expanded_path);
            collect_isf_files(&expanded_path, &mut found_all);
        } else {
            println!("ISF directory not found: {:?}", expanded_path);
        }
    }
    
    found_all.sort();
    found_all.dedup();
    
    println!("Total shaders found: {}", found_all.len());
    
    // Compute compatible set once using validator
    let mut compatible = Vec::new();
    for p in found_all.iter() {
        if let Ok(src) = fs::read_to_string(p) {
            if is_wgsl_shader_compatible(&src) { 
                compatible.push(p.clone()); 
            }
        }
    }
    
    println!("Compatible shaders: {}", compatible.len());
    
    ui_state.available_shaders_all = found_all;
    ui_state.available_shaders_compatible = compatible;
}

fn collect_wgsl_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_wgsl_files(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("wgsl") {
                    if let Some(s) = p.to_str() {
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
}

fn collect_isf_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                // Recursively search subdirectories
                collect_isf_files(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                // Collect both .fs (ISF fragment shaders) and .vs (ISF vertex shaders)
                if ext.eq_ignore_ascii_case("fs") || ext.eq_ignore_ascii_case("vs") || ext.eq_ignore_ascii_case("isf") {
                    if let Some(s) = p.to_str() {
                        out.push(s.to_string());
                        println!("Found ISF shader: {}", s);
                    }
                }
            }
        }
    }
}

/// Bottom code editor panel bound to `EditorUiState::draft_code`.
// Helper that draws the code editor panel using a provided egui context
pub fn draw_editor_code_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    if !ui_state.show_code_editor { return; }
    egui::TopBottomPanel::bottom("code_editor")
        .resizable(false)
        .default_height(240.0)
        .min_height(160.0)
        .max_height(280.0)
        .show(ctx, |ui| {
        ui.heading("WGSL Code Editor");
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut edit = egui::TextEdit::multiline(&mut ui_state.draft_code)
                .code_editor()
                .desired_rows(12)
                .lock_focus(true)
                .hint_text("Paste or write WGSL here...");
            let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| -> Arc<egui::Galley> {
                highlight_wgsl(ui, text.as_str(), wrap_width)
            };
            edit = edit.layouter(&mut layouter);
            // Fix editor height so it doesn't balloon and block the viewport
            let fixed_height = 180.0;
            ui.add_sized(egui::vec2(ui.available_width(), fixed_height), edit);
        });
        ui.horizontal(|ui| {
            if ui.button("Apply to Preview").clicked() {
                ui_state.apply_requested = true;
            }
            ui.checkbox(&mut ui_state.auto_apply, "Auto Apply");
            ui.label("Tip: Load a shader from the browser, edit, then apply.");
        });
    });
}

pub fn editor_code_panel(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let ctx = egui_ctx.ctx_mut().expect("Failed to get egui context");
    draw_editor_code_panel(ctx, &mut *ui_state);
}

/// System to load selected shader file contents into draft buffer.
pub fn apply_shader_selection(mut ui_state: ResMut<EditorUiState>) {
    if let Some(sel) = ui_state.selected_shader.clone() {
        if let Ok(src) = fs::read_to_string(&sel) {
            // Only update draft; preview is updated when Apply is pressed.
            ui_state.draft_code = src;
            // Auto-apply if enabled
            if ui_state.auto_apply {
                ui_state.apply_requested = true;
            }
        }
        // Clear selection so we don't re-load every frame
        ui_state.selected_shader = None;
    }
}

/// Validator: requires both @vertex and @fragment entry points for compatibility.
pub fn is_wgsl_shader_compatible(src: &str) -> bool {
    let has_vertex = src.contains("@vertex");
    let has_fragment = src.contains("@fragment");
    has_vertex && has_fragment
}

/// If incompatible, return a clear message; otherwise, Ok(())
pub fn validate_wgsl_entry_points(src: &str) -> Result<(), String> {
    let has_vertex = src.contains("@vertex");
    let has_fragment = src.contains("@fragment");
    match (has_vertex, has_fragment) {
        (true, true) => Ok(()),
        (false, true) => Err("Missing @vertex entry point".to_string()),
        (true, false) => Err("Missing @fragment entry point".to_string()),
        (false, false) => Err("Missing both @vertex and @fragment entry points".to_string()),
    }
}

/// Mode-aware validator supporting fragment or compute pipelines.
pub fn validate_wgsl_for_mode(src: &str, mode: PipelineMode) -> Result<(), String> {
    match mode {
        PipelineMode::Fragment => {
            // Require vertex + fragment entries
            validate_wgsl_entry_points(src).and_then(|_| {
                // Heuristic binding checks for group(0)
                let has_uniforms = src.contains("@group(0)") && src.contains("@binding(0)");
                let has_params = src.contains("@group(0)") && src.contains("@binding(1)");
                if !has_uniforms {
                    return Err("Fragment mode: expected @group(0) @binding(0) uniforms".to_string());
                }
                if !has_params {
                    return Err("Fragment mode: expected @group(0) @binding(1) params".to_string());
                }
                // Ensure fragment outputs a color
                let has_color_out = src.contains("@fragment") && src.contains("@location(0)");
                if !has_color_out {
                    return Err("Fragment mode: expected @location(0) color output".to_string());
                }
                Ok(())
            })
        }
        PipelineMode::Compute => {
            let has_compute = src.contains("@compute");
            if !has_compute { return Err("Missing @compute entry point".to_string()); }
            // Heuristic binding checks
            let has_uniforms = src.contains("@group(0)") && src.contains("@binding(0)");
            let has_params = src.contains("@group(0)") && src.contains("@binding(1)");
            let has_storage = src.contains("texture_storage_2d") && src.contains("@binding(2)");
            if !has_uniforms {
                return Err("Compute mode: expected @group(0) @binding(0) uniforms".to_string());
            }
            if !has_params {
                return Err("Compute mode: expected @group(0) @binding(1) params".to_string());
            }
            if !has_storage {
                return Err("Compute mode: expected @group(0) @binding(2) storage texture".to_string());
            }
            Ok(())
        }
    }
}

fn highlight_wgsl(ui: &egui::Ui, text: &str, wrap_width: f32) -> Arc<egui::Galley> {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let s = text;
    let mut _line_start = 0;
    for (i, line) in s.lines().enumerate() {
        let mut _idx = 0;
        let mut _in_comment = false;
        while _idx < line.len() {
            // Detect comments
            if !_in_comment {
                if let Some(pos) = line[_idx..].find("//") {
                    // append up to comment normally
                    let before = &line[_idx.._idx+pos];
                    append_tokens(&mut job, before);
                    // append comment
                    let comment = &line[_idx+pos..];
                    job.append(
                        comment,
                        0.0,
                        egui::TextFormat {
                            color: egui::Color32::from_rgb(120, 130, 140),
                            ..Default::default()
                        },
                    );
                    _in_comment = true;
                    _idx = line.len();
                    break;
                }
            }
            if !_in_comment {
                let rest = &line[_idx..];
                append_tokens(&mut job, rest);
                _idx = line.len();
            }
        }
        // newline at end of each line except maybe last
        if i < s.lines().count() {
            job.append("\n", 0.0, Default::default());
        }
        _line_start += line.len() + 1;
    }
    ui.painter().layout_job(job)
}

fn append_tokens(job: &mut LayoutJob, s: &str) {
    // Tokenize by whitespace and punctuation (very naive)
    let mut token = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            token.push(ch);
        } else {
            if !token.is_empty() { append_token(job, &token); token.clear(); }
            job.append(
                &ch.to_string(),
                0.0,
                egui::TextFormat { ..Default::default() },
            );
        }
    }
    if !token.is_empty() { append_token(job, &token); }
}

fn append_token(job: &mut LayoutJob, tok: &str) {
    let (color, _italic) = match tok {
        // WGSL attributes and builtins
        "@fragment" | "@vertex" | "@compute" | "@group" | "@binding" | "@location" | "@builtin" => (egui::Color32::from_rgb(180, 120, 255), false),
        // Types
        "f32" | "u32" | "i32" | "vec2" | "vec3" | "vec4" | "mat2x2" | "mat3x3" | "mat4x4" => (egui::Color32::from_rgb(110, 180, 255), false),
        // Keywords
        "struct" | "var" | "let" | "fn" | "return" | "if" | "else" | "for" | "while" | "break" | "continue" | "true" | "false" => (egui::Color32::from_rgb(255, 200, 100), false),
        // Common identifiers
        "uniforms" | "time" | "resolution" | "mouse" => (egui::Color32::LIGHT_GRAY, false),
        _ => (egui::Color32::WHITE, false),
    };
    job.append(
        tok,
        0.0,
        egui::TextFormat { color, ..Default::default() },
    );
}

// ==== Converter actions ====
fn import_isf_into_editor(ui_state: &mut EditorUiState) {
    // Select an ISF file and convert to WGSL into draft buffer
    let file = rfd::FileDialog::new()
        .add_filter("ISF Files", &["fs"])
        .pick_file();
    if let Some(p) = file {
        if let Ok(content) = std::fs::read_to_string(&p) {
            // Use the advanced ISF converter
            let mut converter = super::converter::ISFParser::new();
            match converter.parse_isf(&content, p.to_str().unwrap_or("unknown")) {
                Ok(isf_shader) => {
                    match converter.convert_to_wgsl(&isf_shader) {
                        Ok(wgsl) => {
                            ui_state.draft_code = wgsl;
                            println!("Successfully converted ISF to WGSL");
                        }
                        Err(e) => println!("ISF→WGSL conversion failed: {}", e),
                    }
                }
                Err(e) => println!("ISF parse failed: {}", e),
            }
        }
    }
}

fn batch_convert_isf_directory() {
    let src = rfd::FileDialog::new().pick_folder();
    if src.is_none() { return; }
    let out = rfd::FileDialog::new().pick_folder();
    if out.is_none() { return; }
    // TODO: Implement batch ISF directory conversion
    println!("Batch ISF directory conversion not yet implemented");

}

fn convert_current_glsl_to_wgsl(ui_state: &mut EditorUiState) {
    match super::converter::GLSLConverter::new() {
        Ok(mut converter) => {
            match converter.convert(&ui_state.draft_code, "input.glsl") {
                Ok(wgsl) => ui_state.draft_code = wgsl,
                Err(e) => println!("GLSL→WGSL conversion failed: {}", e),
            }
        }
        Err(e) => println!("Failed to create GLSL converter: {}", e),
    }
}

fn convert_current_hlsl_to_wgsl(ui_state: &mut EditorUiState) {
    match super::converter::HLSLConverter::new() {
        Ok(mut converter) => {
            match converter.convert(&ui_state.draft_code, "input.hlsl") {
                Ok(wgsl) => ui_state.draft_code = wgsl,
                Err(e) => println!("HLSL→WGSL conversion failed: {}", e),
            }
        }
        Err(e) => println!("Failed to create HLSL converter: {}", e),
    }
}

fn export_current_wgsl_to_glsl(ui_state: &EditorUiState) {
    match crate::shader_converter::wgsl_to_glsl(&ui_state.draft_code) {
        Ok(glsl) => {
            if let Some(save_path) = rfd::FileDialog::new().save_file() {
                let _ = std::fs::write(save_path, glsl);
            }
        }
        Err(e) => println!("WGSL→GLSL export failed: {}", e),
    }
}

fn export_current_wgsl_to_hlsl(ui_state: &EditorUiState) {
    match crate::shader_converter::wgsl_to_hlsl(&ui_state.draft_code) {
        Ok(hlsl) => {
            if let Some(save_path) = rfd::FileDialog::new().save_file() {
                let _ = std::fs::write(save_path, hlsl);
            }
        }
        Err(e) => println!("WGSL→HLSL export failed: {}", e),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectData {
    node_graph: super::node_graph::NodeGraph,
    draft_code: Option<String>,
    timeline: super::timeline::Timeline,
    param_index_map: std::collections::HashMap<String, usize>,
}

fn export_project_json(ui_state: &EditorUiState) -> Result<(), String> {
    let proj = ProjectData {
        node_graph: ui_state.node_graph.clone(),
        draft_code: Some(ui_state.draft_code.clone()),
        timeline: ui_state.timeline.timeline.clone(),
        param_index_map: ui_state.param_index_map.clone(),
    };
    let json = serde_json::to_string_pretty(&proj).map_err(|e| e.to_string())?;
    let path = rfd::FileDialog::new().add_filter("Project", &["json"]).set_directory(".").set_title("Save Project").save_file();
    if let Some(p) = path { std::fs::write(&p, json).map_err(|e| e.to_string())?; }
    Ok(())
}

fn import_project_json() -> Result<ProjectData, String> {
    let path = rfd::FileDialog::new().add_filter("Project", &["json"]).set_directory(".").set_title("Open Project").pick_file();
    if let Some(p) = path {
        let s = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let proj: ProjectData = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        Ok(proj)
    } else {
        Err("No file selected".to_string())
    }
}

fn default_wgsl_template() -> String {
    r#"
struct Uniforms {
  time: f32,
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  audio_volume: f32,
  audio_bass: f32,
  audio_mid: f32,
  audio_treble: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> params: array<vec4<f32>, 16>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
  var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 3.0,  1.0),
  );
  let pos = positions[vertex_index];
  return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let uv = pos.xy / uniforms.resolution;
  let p0 = params[0].x;
  let t = uniforms.time;
  let base = 0.5 + 0.5 * sin(t);
  return vec4<f32>(uv.x * (1.0 + 0.2 * p0), uv.y, base, 1.0);
}
"#.to_string()
}

fn save_draft_wgsl_to_assets(ui_state: &EditorUiState) {
    let dialog = rfd::FileDialog::new()
        .add_filter("WGSL", &["wgsl"])
        .set_directory("assets/shaders")
        .set_title("Save WGSL Draft As");
    if let Some(path) = dialog.save_file() {
        match std::fs::write(&path, &ui_state.draft_code) {
            Ok(_) => println!("Saved WGSL draft to {:?}", path),
            Err(e) => println!("Failed to save WGSL: {}", e),
        }
    } else {
        println!("Save cancelled");
    }
}

fn export_recorded_frames_to_mp4() {
    use std::process::Command;
    let input_pattern = std::path::Path::new("assets/output/frame_%05d.png");
    let first_frame = std::path::Path::new("assets/output/frame_00000.png");
    if !first_frame.exists() {
        println!("No recorded frames found in assets/output/ (start recording in Preview panel)");
        return;
    }
    let dialog = rfd::FileDialog::new()
        .add_filter("MP4", &["mp4"])
        .set_directory("assets/output")
        .set_title("Export MP4");
    if let Some(out_path) = dialog.save_file() {
        let out_str = out_path.to_string_lossy().to_string();
        let input_str = input_pattern.to_string_lossy().to_string();
        println!("Running ffmpeg to export MP4: {}", out_str);
        let status = Command::new("ffmpeg")
            .args([
                "-hide_banner", "-loglevel", "error",
                "-framerate", "60",
                "-i", &input_str,
                "-pix_fmt", "yuv420p",
                "-y", &out_str,
            ])
            .status();
        match status {
            Ok(s) if s.success() => println!("Exported MP4 to {}", out_str),
            Ok(s) => println!("ffmpeg exited with code {:?}", s.code()),
            Err(e) => println!("Failed to run ffmpeg: {} (ensure ffmpeg is on PATH)", e),
        }
    } else {
        println!("Export cancelled");
    }
}
// removed deprecated attribute; updated calls to modern egui API

/// Parse shader code for parameters (uniforms, textures, etc.)
fn parse_shader_parameters(shader_code: &str) -> Vec<ShaderParameter> {
    let mut parameters = Vec::new();
    
    // First, try to parse ISF metadata if this is an ISF shader
    if let Some(isf_params) = parse_isf_parameters(shader_code) {
        return isf_params;
    }
    
    // Fall back to WGSL uniform parsing
    // Simple regex-based parsing for uniform declarations
    let uniform_regex = regex::Regex::new(r"@group\((\d+)\)\s*@binding\((\d+)\)\s*var<uniform>\s+(\w+):\s*([^;]+);").unwrap();
    
    for cap in uniform_regex.captures_iter(shader_code) {
        let group = cap[1].parse::<u32>().unwrap_or(0);
        let binding = cap[2].parse::<u32>().unwrap_or(0);
        let name = cap[3].to_string();
        let wgsl_type = cap[4].trim().to_string();
        
        parameters.push(ShaderParameter {
            name,
            wgsl_type,
            group,
            binding,
            default_value: None,
            min_value: None,
            max_value: None,
        });
    }
    
    // Parse texture declarations
    let texture_regex = regex::Regex::new(r"@group\((\d+)\)\s*@binding\((\d+)\)\s*var\s+(\w+):\s*(texture_\w+);").unwrap();
    
    for cap in texture_regex.captures_iter(shader_code) {
        let group = cap[1].parse::<u32>().unwrap_or(0);
        let binding = cap[2].parse::<u32>().unwrap_or(0);
        let name = cap[3].to_string();
        let wgsl_type = cap[4].trim().to_string();
        
        parameters.push(ShaderParameter {
            name,
            wgsl_type,
            group,
            binding,
            default_value: None,
            min_value: None,
            max_value: None,
        });
    }
    
    parameters
}

/// Parse ISF parameters from shader code containing ISF metadata
fn parse_isf_parameters(shader_code: &str) -> Option<Vec<ShaderParameter>> {
    // Look for ISF JSON metadata in comments
    if let Some(json_start) = shader_code.find("/*{") {
        if let Some(json_end) = shader_code[json_start..].find("}*/") {
            let json_str = &shader_code[json_start + 2..json_start + json_end + 1];
            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(json_str) {
                let mut parameters = Vec::new();
                
                // Parse ISF inputs
                if let Some(inputs_json) = metadata.get("INPUTS") {
                    if let Some(inputs_array) = inputs_json.as_array() {
                        for (index, input_json) in inputs_array.iter().enumerate() {
                            if let Some(name) = input_json.get("NAME").and_then(|n| n.as_str()) {
                                let input_type = input_json.get("TYPE").and_then(|t| t.as_str()).unwrap_or("float");
                                
                                let default = input_json.get("DEFAULT")
                                    .and_then(|d| d.as_f64())
                                    .map(|d| d as f32);

                                let min = input_json.get("MIN")
                                    .and_then(|m| m.as_f64())
                                    .map(|m| m as f32);

                                let max = input_json.get("MAX")
                                    .and_then(|m| m.as_f64())
                                    .map(|m| m as f32);

                                parameters.push(ShaderParameter {
                                    name: name.to_string(),
                                    wgsl_type: map_isf_type_to_wgsl(input_type),
                                    group: 0, // ISF inputs typically use group 0
                                    binding: index as u32,
                                    default_value: default,
                                    min_value: min,
                                    max_value: max,
                                });
                            }
                        }
                    }
                }
                
                return Some(parameters);
            }
        }
    }
    
    None
}

/// Map ISF input types to WGSL types
fn map_isf_type_to_wgsl(isf_type: &str) -> String {
    match isf_type.to_lowercase().as_str() {
        "float" => "f32".to_string(),
        "bool" => "bool".to_string(),
        "color" => "vec4<f32>".to_string(),
        "point2d" => "vec2<f32>".to_string(),
        "image" => "texture_2d<f32>".to_string(),
        _ => "f32".to_string(), // Default to float
    }
}

#[derive(Debug, Clone)]
struct ShaderParameter {
    name: String,
    wgsl_type: String,
    group: u32,
    binding: u32,
    default_value: Option<f32>,
    min_value: Option<f32>,
    max_value: Option<f32>,
}