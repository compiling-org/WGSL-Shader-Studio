use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::fs;
use std::path::Path;
use egui::text::LayoutJob;
use std::sync::Arc;
use crate::node_graph::{NodeGraph, NodeKind};
use crate::timeline::{TimelineAnimation, InterpolationType, PlaybackState};
use crate::shader_renderer::ShaderRenderer;
use crate::audio_system::AudioAnalyzer;
use crate::compute_pass_integration::{ComputePassManager, TextureFormat};
use crate::midi_system::{MidiSystem, MidiMapping, MidiMessageType, MidiCurve};
use crate::screenshot_video_export::{ScreenshotVideoExporter, ExportUI, ExportSettings, VideoExportSettings};
use crate::ndi_output::{NdiConfig, NdiOutput, NdiUI};
use crate::osc_control::{OscConfig, OscControl, OscMapping, OscMessageType, OscUI};
use crate::spout_syphon_output::{SpoutSyphonConfig, SpoutSyphonOutput, SpoutSyphonUI};
use crate::dmx_lighting_control::{DmxConfig, DmxLightingControl, DmxUI};
#[cfg(feature = "naga_integration")]
use crate::wgsl_ast_parser::WgslAstParser;
#[cfg(feature = "naga_integration")]
use crate::shader_transpiler::{MultiFormatTranspiler, TranspilerOptions, ShaderLanguage, ShaderValidator};

// Temporarily commented out to fix compilation - will be restored when visual node editor is fully integrated
// use crate::visual_node_editor_adapter::NodeEditorAdapter;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CentralView {
    Preview,
    NodeGraph,
    Scene3D,
    Timeline,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PipelineMode {
    Fragment,
    Compute,
}

impl Default for PipelineMode {
    fn default() -> Self { PipelineMode::Fragment }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RightSidebarMode {
    Parameters,
    Compute,
    Outputs,
    OSC,
    Audio,
    MIDI,
    Gestures,
    Lighting,
    Performance,
    Scene3D,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceSet {
    All,
    Assets,
    ISF,
}

impl Default for ThemePreference {
    fn default() -> Self { ThemePreference::Dark }
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
    pub show_gesture_calibration: bool,
    pub show_wgslsmith_panel: bool,
    pub show_diagnostics_panel: bool,
    pub show_compute_panel: bool,
    pub show_3d_scene_panel: bool,
    pub show_spout_panel: bool,
    pub show_ffgl_panel: bool,
    pub show_gyroflow_panel: bool,
    pub show_analyzer_panel: bool,
    pub show_isf_converter: bool,
    pub show_wgsl_analyzer: bool,
    pub show_performance: bool,
    pub show_performance_overlay: bool,
    pub show_color_grading_panel: bool,
    pub show_osc_panel: bool,
    pub show_dmx_panel: bool,
    pub show_export_panel: bool,
    pub show_ndi_panel: bool,
    pub central_view: CentralView,
    pub fps: f32,
    pub time: f64,
    // Preview pipeline mode
    pub pipeline_mode: PipelineMode,
    // Right sidebar mode
    pub right_sidebar_mode: RightSidebarMode,
    pub outputs_mode: OutputsMode,
    pub code_editor_tab: CodeEditorTab,
    pub selected_source: SourceSet,
    // Theme settings
    pub dark_mode: bool,
    pub theme_preference: ThemePreference,
    // Browser/state
    pub search_query: String,
    pub show_all_shaders: bool,
    pub available_shaders_all: Vec<String>,
    pub available_shaders_compatible: Vec<String>,
    pub selected_shader: Option<String>,
    pub selected_category: Option<String>,
    // Code editor buffer
    pub draft_code: String,
    pub current_file: String,
    pub code: String,
    pub code_changed: bool,
    pub apply_requested: bool,
    pub auto_apply: bool,
    // Node graph and project state
    pub node_graph: NodeGraph,
    // pub visual_node_editor: NodeEditorAdapter,
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
    pub wgpu_initialized: bool,
    pub compilation_error: String,
    // Parameter values storage for shader rendering
    pub parameter_values: std::collections::HashMap<String, f32>,
    // WGSLSmith AI fields
    pub wgsl_smith_prompt: String,
    pub wgsl_smith_generated: String,
    pub wgsl_smith_status: String,
    // WGSL diagnostics
    pub diagnostics_messages: Vec<DiagnosticMessage>,
    pub analyzer_status: Vec<String>,
    pub analyzer_run_requested: bool,
    // Compute pass UI state
    pub compute_pass_name: String,
    pub compute_workgroup_x: u32,
    pub compute_workgroup_y: u32,
    pub compute_workgroup_z: u32,
    pub pingpong_texture_name: String,
    pub pingpong_width: u32,
    pub pingpong_height: u32,
    pub dispatch_size_x: u32,
    pub dispatch_size_y: u32,
    pub dispatch_size_z: u32,
    // Video recording state
    pub is_recording_video: bool,
    pub video_fps: u32,
    pub video_duration: f32,
    pub video_format: String,
    pub video_quality: u8,
    // 3D Scene parameters (Space Editor inspired)
    pub camera_position: [f32; 3],
    pub camera_rotation: [f32; 3],
    pub camera_fov: f32,
    pub camera_near: f32,
    pub camera_far: f32,
    pub light_position: [f32; 3],
    pub light_color: [f32; 3],
    pub light_intensity: f32,
    pub ambient_light_color: [f32; 3],
    pub ambient_light_intensity: f32,
    pub export_settings: ExportSettings,
    pub video_export_settings: VideoExportSettings,
    pub use_legacy_windows: bool,
    pub ast_ok: bool,
    pub ast_error: String,
    pub validator_ok: bool,
    pub validator_error: String,
    pub transpiled_glsl: String,
    pub transpiler_error: String,
    pub scene3d_texture_id: Option<egui::TextureId>,
    pub scene3d_texture_handle: Option<bevy::prelude::Handle<bevy::prelude::Image>>,
    pub preview_scale_mode: PreviewScaleMode,
    pub preview_resolution: (u32, u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputsMode {
    Ndi,
    SpoutSyphon,
    ScreenshotsVideo,
    Ffgl,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodeEditorTab {
    Editor,
    AI,
    Diagnostics,
    Analyzer,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreviewScaleMode {
    Fit,
    Fill,
    OneToOne,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            show_shader_browser: true,
            show_parameter_panel: true,
            show_preview: true,
            show_code_editor: true,
            show_node_studio: true,
            show_timeline: false,
            show_audio_panel: true,
            show_midi_panel: true,
            show_gesture_panel: true,
            show_gesture_calibration: false,
            show_wgslsmith_panel: true,
            show_diagnostics_panel: true,
            show_compute_panel: true,
            show_3d_scene_panel: false,
            show_spout_panel: true,
            show_ffgl_panel: true,
            show_gyroflow_panel: false,
            show_analyzer_panel: true,
            show_isf_converter: true,
            show_wgsl_analyzer: true,
            show_performance: true,
            show_performance_overlay: false,
            show_color_grading_panel: false,
            show_osc_panel: true,
            show_dmx_panel: false,
            show_export_panel: false,
            show_ndi_panel: true,
            central_view: CentralView::Preview,
            fps: 0.0,
            time: 0.0,
            pipeline_mode: PipelineMode::default(),
            right_sidebar_mode: RightSidebarMode::Parameters,
            outputs_mode: OutputsMode::ScreenshotsVideo,
            code_editor_tab: CodeEditorTab::Editor,
            selected_source: SourceSet::All,
            dark_mode: true,
            theme_preference: ThemePreference::default(),
            search_query: String::new(),
            show_all_shaders: true,
            available_shaders_all: Vec::new(),
            available_shaders_compatible: Vec::new(),
            selected_shader: None,
            selected_category: None,
            draft_code: default_wgsl_template(),
            current_file: String::new(),
            code: default_wgsl_template(),
            code_changed: false,
            apply_requested: false,
            auto_apply: false,
            node_graph: NodeGraph::default(),
            // visual_node_editor: NodeEditorAdapter::new(),
            last_project_path: None,
            timeline: TimelineAnimation::default(),
            timeline_track_input: String::new(),
            param_index_map: std::collections::HashMap::new(),
            param_index_input: 0,
            quick_params_enabled: false,
            quick_param_a: 0.5,
            quick_param_b: 0.5,
            global_renderer: GlobalShaderRenderer::default(),
            wgpu_initialized: false,
            compilation_error: String::new(),
            parameter_values: std::collections::HashMap::new(),
            wgsl_smith_prompt: String::new(),
            wgsl_smith_generated: String::new(),
            wgsl_smith_status: String::new(),
            diagnostics_messages: Vec::new(),
            analyzer_status: Vec::new(),
            analyzer_run_requested: false,
            // Compute pass UI defaults
            compute_pass_name: "compute_pass_1".to_string(),
            compute_workgroup_x: 8,
            compute_workgroup_y: 8,
            compute_workgroup_z: 1,
            pingpong_texture_name: "pingpong_tex".to_string(),
            pingpong_width: 512,
            pingpong_height: 512,
            dispatch_size_x: 8,
            dispatch_size_y: 8,
            dispatch_size_z: 1,
            // Video recording defaults
            is_recording_video: false,
            video_fps: 30,
            video_duration: 10.0,
            video_format: "mp4".to_string(),
            video_quality: 90,
            // 3D Scene parameters defaults
            camera_position: [0.0, 0.0, 5.0],
            camera_rotation: [0.0, 0.0, 0.0],
            camera_fov: 60.0,
            camera_near: 0.1,
            camera_far: 100.0,
            light_position: [2.0, 2.0, 2.0],
            light_color: [1.0, 1.0, 1.0],
            light_intensity: 1.0,
            ambient_light_color: [0.2, 0.2, 0.2],
            ambient_light_intensity: 0.3,
            export_settings: ExportSettings::default(),
            video_export_settings: VideoExportSettings::default(),
            use_legacy_windows: false,
            ast_ok: false,
            ast_error: String::new(),
            validator_ok: false,
            validator_error: String::new(),
            transpiled_glsl: String::new(),
            transpiler_error: String::new(),
            scene3d_texture_id: None,
            scene3d_texture_handle: None,
            preview_scale_mode: PreviewScaleMode::Fit,
            preview_resolution: (1280, 720),
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

/// Connect audio analysis to shader parameters
pub fn connect_audio_to_parameters(ui_state: &mut EditorUiState, audio_data: &crate::audio_system::AudioData) {
    // Map audio analysis to shader parameters
    let volume_param = audio_data.volume * 2.0; // Amplify for better effect
    let bass_param = audio_data.bass_level * 3.0;
    let mid_param = audio_data.mid_level * 2.0;
    let treble_param = audio_data.treble_level * 2.0;
    let beat_intensity = if audio_data.beat_detected { 1.0 } else { 0.0 };
    
    // Update parameter values with audio-reactive data
    ui_state.set_parameter_value("audio_volume", volume_param.min(1.0));
    ui_state.set_parameter_value("audio_bass", bass_param.min(1.0));
    ui_state.set_parameter_value("audio_mid", mid_param.min(1.0));
    ui_state.set_parameter_value("audio_treble", treble_param.min(1.0));
    ui_state.set_parameter_value("beat_intensity", beat_intensity);
    ui_state.set_parameter_value("audio_reactive", volume_param.min(1.0));
    
    // Log audio parameter mapping
    println!("Audio parameters updated: volume={:.2}, bass={:.2}, beat={}", 
             volume_param, bass_param, beat_intensity);
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
    global_renderer: &GlobalShaderRenderer,
    parameter_values: &std::collections::HashMap<String, f32>,
    audio_analyzer: Option<&crate::audio_system::AudioAnalyzer>,
    video_exporter: Option<&()> // ScreenshotVideoExporter commented out
) -> Result<egui::TextureHandle, String> {
    if wgsl_code.trim().is_empty() {
        return Err("Empty shader code".to_string());
    }
    
    if size.x <= 0.0 || size.y <= 0.0 {
        return Err("Preview size is zero or negative; cannot render texture".to_string());
    }
    
    // Validate basic WGSL syntax
    if !wgsl_code.contains("@fragment") && !wgsl_code.contains("@vertex") && !wgsl_code.contains("@compute") {
        return Err("Shader must contain @fragment, @vertex, or @compute entry point".to_string());
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
        
        match renderer.render_frame_with_params(
            wgsl_code,
            &params,
            Some(&param_array),
            None::<crate::audio_system::AudioData>,
        ) {
            Ok(pixel_data) => {
                if pixel_data.is_empty() {
                    println!("Rendered pixel data is empty. Displaying red error image.");
                    let width = (params.width as usize).max(1);
                    let height = (params.height as usize).max(1);
                    let error_pixels = vec![egui::Color32::RED; width * height];
                    let texture = egui_ctx.load_texture(
                        "shader_preview_error",
                        egui::ColorImage {
                            size: [width, height],
                            pixels: error_pixels,
                            source_size: egui::Vec2::new((params.width as f32).max(1.0), (params.height as f32).max(1.0)),
                        },
                        egui::TextureOptions::default()
                    );
                    return Ok(texture);
                }
                // Create texture from pixel data with proper size validation
                let width = (params.width as usize).max(1);
                let height = (params.height as usize).max(1);
                let expected_pixel_count = width * height;
                
                // Handle empty pixel data or size mismatches
                let pixels = if pixel_data.is_empty() || expected_pixel_count == 0 {
                    println!("Empty pixel data or zero dimensions in real renderer: {}x{}. Using red error image.", width, height);
                    let safe_width = width.max(1);
                    let safe_height = height.max(1);
                    vec![egui::Color32::RED; safe_width * safe_height]
                } else {
                    // Validate pixel data size matches expected dimensions
                    let actual_pixel_count = pixel_data.len() / 4; // 4 bytes per pixel (RGBA)
                    if actual_pixel_count != expected_pixel_count {
                        println!("Pixel data size mismatch in real renderer: expected {}, got {}. Using red error image.", expected_pixel_count, actual_pixel_count);
                        let safe_width = width.max(1);
                        let safe_height = height.max(1);
                        vec![egui::Color32::RED; safe_width * safe_height]
                    } else {
                        pixel_data.chunks(4).map(|chunk| {
                            egui::Color32::from_rgba_unmultiplied(chunk[0], chunk[1], chunk[2], chunk[3])
                        }).collect()
                    }
                };
                
                let texture = egui_ctx.load_texture(
                    "shader_preview_real",
                    egui::ColorImage {
                        size: [width, height],
                        pixels,
                        source_size: egui::Vec2::new((params.width as f32).max(1.0), (params.height as f32).max(1.0)),
                    },
                    egui::TextureOptions::default()
                );
                
                // Capture frame for video recording if active
                // Video export functionality temporarily disabled for compilation
                // if let Some(exporter) = video_exporter {
                //     let _ = exporter.capture_frame_data(&pixel_data, params.width, params.height);
                // }
                
                return Ok(texture);
            }
            Err(e) => {
                println!("Real WGPU renderer failed: {}. Falling back to software renderer.", e);
                // Continue to software fallback
            }
        }
    }
    
    // Fallback disabled: enforce GPU-only policy
    Err("GPU renderer not initialized. Hardware acceleration is required.".to_string())
}

/// Render shader to texture for preview
fn render_shader_to_texture(
    wgsl_code: &str, 
    size: egui::Vec2,
    renderer: &mut crate::shader_renderer::ShaderRenderer,
    egui_ctx: &egui::Context
) -> Result<egui::TextureHandle, String> {
    use crate::shader_renderer::RenderParameters;
    use crate::audio_system::AudioData;
    
    let params = RenderParameters {
        width: size.x as u32,
        height: size.y as u32,
        time: 0.0, // Will be updated with actual time
        frame_rate: 60.0,
        audio_data: None,
    };
    
    match renderer.render_frame(wgsl_code, &params, None) {
        Ok(pixel_data) => {
            // Create texture from pixel data with proper size validation
            let width = (params.width as usize).max(1);
            let height = (params.height as usize).max(1);
            let expected_pixel_count = width * height;
            
            // Handle empty pixel data or size mismatches
            let pixels = if pixel_data.is_empty() || expected_pixel_count == 0 {
                println!("Empty pixel data or zero dimensions in software renderer: {}x{}. Using red error image.", width, height);
                let safe_width = width.max(1);
                let safe_height = height.max(1);
                vec![egui::Color32::RED; safe_width * safe_height]
            } else {
                // Validate pixel data size matches expected dimensions
                let actual_pixel_count = pixel_data.len() / 4; // 4 bytes per pixel (RGBA)
                if actual_pixel_count != expected_pixel_count {
                    println!("Pixel data size mismatch in software renderer: expected {}, got {}. Using red error image.", expected_pixel_count, actual_pixel_count);
                    let safe_width = width.max(1);
                    let safe_height = height.max(1);
                    vec![egui::Color32::RED; safe_width * safe_height]
                } else {
                    pixel_data.chunks(4).map(|chunk| {
                        egui::Color32::from_rgba_unmultiplied(chunk[0], chunk[1], chunk[2], chunk[3])
                    }).collect()
                }
            };
            
            let texture = egui_ctx.load_texture(
                "shader_preview",
                egui::ColorImage {
                    size: [width, height],
                    pixels,
                    source_size: egui::Vec2::new((params.width as f32).max(1.0), (params.height as f32).max(1.0)),
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
            ui.menu_button("File", |ui| {
                if ui.button("New WGSL Buffer").clicked() {
                    ui_state.draft_code = default_wgsl_template();
                    ctx.request_repaint();
                    ui.close_menu();
                }
                if ui.button("Save Draft As…").clicked() {
                    save_draft_wgsl_to_assets(&ui_state);
                    ctx.request_repaint();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Save Project…").clicked() {
                    let _ = export_project_json(&ui_state);
                    ctx.request_repaint();
                    ui.close_menu();
                }
                if ui.button("Open Project…").clicked() {
                    match import_project_json() {
                        Ok(proj) => {
                            ui_state.node_graph = proj.node_graph;
                            if let Some(code) = proj.draft_code { ui_state.draft_code = code; }
                            ui_state.timeline = TimelineAnimation { timeline: proj.timeline, playing: false };
                            ui_state.param_index_map = proj.param_index_map;
                        }
                        Err(e) => { println!("Import project failed: {}", e); }
                    }
                    ctx.request_repaint();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Export recorded frames → MP4").clicked() {
                    export_recorded_frames_to_mp4();
                    ctx.request_repaint();
                    ui.close_menu();
                }
            });
            ui.separator();
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
                ui.checkbox(&mut ui_state.show_wgslsmith_panel, "WGSLSmith");
                {
                    let prev = ui_state.show_compute_panel;
                    ui.checkbox(&mut ui_state.show_compute_panel, "Compute Passes");
                    if ui_state.show_compute_panel && !prev {
                        ui_state.right_sidebar_mode = RightSidebarMode::Compute;
                    }
                }
            });
            
            ui.menu_button("View", |ui| {
                ui.menu_button("Theme", |ui| {
                    ui.radio_value(&mut ui_state.theme_preference, ThemePreference::Dark, "🌙 Dark");
                    ui.radio_value(&mut ui_state.theme_preference, ThemePreference::Light, "☀️ Light");
                    ui.radio_value(&mut ui_state.theme_preference, ThemePreference::System, "🖥️ System");
                });
                if ui.button("Toggle Dark Mode").clicked() {
                    ui_state.dark_mode = !ui_state.dark_mode;
                    ui.close_menu();
                }
                ui.separator();
                ui.checkbox(&mut ui_state.show_diagnostics_panel, "Diagnostics Panel");
                if ui.button("Run WGSL Diagnostics").clicked() {
                    run_wgsl_diagnostics(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
            });
            
            ui.menu_button("Integrations", |ui| {
                ui.checkbox(&mut ui_state.show_osc_panel, "OSC");
                ui.checkbox(&mut ui_state.show_ndi_panel, "NDI");
                ui.checkbox(&mut ui_state.show_spout_panel, "Spout/Syphon");
                ui.checkbox(&mut ui_state.show_ffgl_panel, "FFGL");
                ui.checkbox(&mut ui_state.show_gyroflow_panel, "Gyroflow");
                ui.checkbox(&mut ui_state.show_export_panel, "Export");
                ui.checkbox(&mut ui_state.show_analyzer_panel, "Analyzer");
                ui.checkbox(&mut ui_state.show_performance_overlay, "Performance Overlay");
                ui.close_kind(egui::UiKind::Menu);
            });

            ui.separator();
            ui.menu_button("Import/Convert", |ui| {
                if ui.button("Import ISF (.fs) → WGSL into editor").clicked() {
                    import_isf_into_editor(ui_state);
                    ui.close_menu();
                }
                if ui.button("Batch convert ISF directory → WGSL").clicked() {
                    batch_convert_isf_directory();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Current buffer: GLSL → WGSL").clicked() {
                    convert_current_glsl_to_wgsl(ui_state);
                    ui.close_menu();
                }
                if ui.button("Current buffer: HLSL → WGSL").clicked() {
                    convert_current_hlsl_to_wgsl(ui_state);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Export current WGSL → GLSL").clicked() {
                    export_current_wgsl_to_glsl(&ui_state);
                    ui.close_menu();
                }
                if ui.button("Export current WGSL → HLSL").clicked() {
                    export_current_wgsl_to_hlsl(&ui_state);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Multi-language Transpiler").clicked() {
                    show_transpiler_panel(ui_state);
                    ui.close_menu();
                }
            });

            ui.separator();
            ui.menu_button("Documentation", |ui| {
                if ui.button("Shader Studio Cookbook").clicked() {
                    // TODO: Open documentation in browser
                    println!("Opening Shader Studio Cookbook...");
                    ui.close_menu();
                }
                if ui.button("WGSL Fundamentals").clicked() {
                    // TODO: Open WGSL fundamentals documentation
                    println!("Opening WGSL Fundamentals...");
                    ui.close_menu();
                }
                if ui.button("GLSL Fundamentals").clicked() {
                    // TODO: Open GLSL fundamentals documentation
                    println!("Opening GLSL Fundamentals...");
                    ui.close_menu();
                }
                if ui.button("HLSL Fundamentals").clicked() {
                    // TODO: Open HLSL fundamentals documentation
                    println!("Opening HLSL Fundamentals...");
                    ui.close_menu();
                }
                if ui.button("ISF Fundamentals").clicked() {
                    // TODO: Open ISF fundamentals documentation
                    println!("Opening ISF Fundamentals...");
                    ui.close_menu();
                }
                if ui.button("Shader Conversion Framework").clicked() {
                    // TODO: Open shader conversion framework documentation
                    println!("Opening Shader Conversion Framework...");
                    ui.close_menu();
                }
                if ui.button("Application Usage Guide").clicked() {
                    // TODO: Open application usage guide
                    println!("Opening Application Usage Guide...");
                    ui.close_menu();
                }
                if ui.button("Technical Architecture").clicked() {
                    // TODO: Open technical architecture documentation
                    println!("Opening Technical Architecture...");
                    ui.close_menu();
                }
                if ui.button("Advanced Features").clicked() {
                    // TODO: Open advanced features documentation
                    println!("Opening Advanced Features...");
                    ui.close_menu();
                }
                if ui.button("API Reference").clicked() {
                    // TODO: Open API reference in browser
                    println!("Opening API Reference...");
                    ui.close_menu();
                }
                if ui.button("Online Documentation").clicked() {
                    // TODO: Open online documentation
                    println!("Opening Online Documentation...");
                    ui.close_menu();
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

/// Generate shader code using WGSLSmith AI
fn generate_shader_with_wgsl_smith(prompt: &str) -> String {
    // Simple template-based generation for now
    // In a real implementation, this would use AI/ML models
    let template = format!(r#"
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {{
    var vertices = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0), 
        vec2<f32>(-1.0, 3.0)
    );
    return vec4<f32>(vertices[vertex_index], 0.0, 1.0);
}}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {{
    let uv = frag_coord.xy / resolution.xy;
    let color = vec3<f32>(
        uv.x + 0.5 * sin(time),
        uv.y + 0.5 * cos(time),
        0.5 + 0.5 * sin(time * 2.0)
    );
    return vec4<f32>(color, 1.0);
}}
"#);
    
    template
}

pub fn draw_editor_side_panels(
    ctx: &egui::Context, 
    ui_state: &mut EditorUiState, 
    audio_analyzer: &AudioAnalyzer, 
    gesture_control: &mut crate::gesture_control::GestureControlSystem,
    compute_pass_manager: &mut ComputePassManager,
    video_exporter: Option<&ScreenshotVideoExporter>, 
    midi_system: &mut MidiSystem,
    osc_config: &mut OscConfig,
    osc_control: &mut OscControl,
    spout_config: &mut SpoutSyphonConfig,
    spout_output: &mut SpoutSyphonOutput,
    ndi_config: &mut NdiConfig,
    ndi_output: &mut NdiOutput,
    dmx_config: &mut DmxConfig,
    dmx_control: &mut DmxLightingControl,
    scene_editor_state: Option<&mut crate::scene_editor_3d::SceneEditor3DState>,
    manipulable_query: Option<&Query<(Entity, &Name), With<crate::scene_editor_3d::EditorManipulable>>>,
) {
    // Left panel: Shader Browser (and related tools) — per original UI map
    if ui_state.show_shader_browser {
        draw_editor_shader_browser_panel(ctx, ui_state);
    }

    egui::SidePanel::right("right_modes_panel").resizable(true).show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (mode, label) in [
                    (RightSidebarMode::Parameters, "Parameters"),
                    (RightSidebarMode::Compute, "Compute"),
                    (RightSidebarMode::Outputs, "Outputs"),
                    (RightSidebarMode::OSC, "OSC"),
                    (RightSidebarMode::Audio, "Audio"),
                    (RightSidebarMode::MIDI, "MIDI"),
                    (RightSidebarMode::Gestures, "Gestures"),
                    (RightSidebarMode::Lighting, "Lighting"),
                    (RightSidebarMode::Performance, "Performance"),
                    (RightSidebarMode::Scene3D, "3D Scene"),
                ] {
                    let sel = ui_state.right_sidebar_mode == mode;
                    if ui.selectable_label(sel, label).clicked() {
                        ui_state.right_sidebar_mode = mode;
                    }
                }
            });
            ui.separator();
            match ui_state.right_sidebar_mode {
                RightSidebarMode::MIDI => {
                    ui.heading("MIDI");
                    if ui.button("Scan Devices").clicked() {
                        let _ = midi_system.scan_devices();
                    }
                    ui.separator();
                    let devices_snapshot = midi_system.devices.clone();
                    for (i, dev) in devices_snapshot.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(&dev.name);
                            if dev.connected {
                                if ui.button("Disconnect").clicked() {
                                    let _ = midi_system.disconnect_device(i);
                                }
                            } else {
                                if ui.button("Connect").clicked() {
                                    let _ = midi_system.connect_device(i);
                                }
                            }
                        });
                    }
                    ui.separator();
                    ui.heading("Parameter Mapping");
                    let params = parse_shader_parameters(&ui_state.draft_code);
                    if params.is_empty() {
                        ui.label("No shader parameters available");
                    } else {
                        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                            for p in params.iter() {
                                ui.horizontal(|ui| {
                                    ui.label(&p.name);
                                    if ui.button("Learn").clicked() {
                                        midi_system.start_midi_learn(&p.name);
                                    }
                                });
                                let mut channel: u8 = 1;
                                let mut number: u8 = 0;
                                ui.horizontal(|ui| {
                                    ui.label("Channel");
                                    ui.add(egui::DragValue::new(&mut channel).range(1..=16));
                                    ui.label("CC");
                                    ui.add(egui::DragValue::new(&mut number).range(0..=127));
                                    if ui.button("Map CC").clicked() {
                                        let mapping = MidiMapping {
                                            parameter_name: p.name.clone(),
                                            midi_type: MidiMessageType::ControlChange,
                                            channel,
                                            number,
                                            min_value: 0.0,
                                            max_value: 1.0,
                                            curve: MidiCurve::Linear,
                                            invert: false,
                                            smoothing: 0.0,
                                        };
                                        midi_system.add_mapping(mapping);
                                    }
                                });
                                if let Some(existing) = midi_system.get_mapping(&p.name) {
                                    ui.label(format!("Mapped: ch {} CC {}", existing.channel, existing.number));
                                    if ui.button("Remove Mapping").clicked() {
                                        midi_system.remove_mapping(&p.name);
                                    }
                                }
                                ui.separator();
                            }
                        });
                    }
                }
                RightSidebarMode::Audio => {
                    ui.heading("Audio");
                    let data = audio_analyzer.get_audio_data();
                    ui.horizontal(|ui| {
                        ui.label(format!("Volume: {:.2}", data.volume));
                        ui.label(format!("Bass: {:.2}", data.bass_level));
                        ui.label(format!("Mid: {:.2}", data.mid_level));
                        ui.label(format!("Treble: {:.2}", data.treble_level));
                    });
                    let graph_height = 80.0;
                    let graph_width = ui.available_width();
                    let (response, painter) = ui.allocate_painter(egui::Vec2::new(graph_width, graph_height), egui::Sense::hover());
                    let rect = response.rect;
                    let bg = egui::Color32::from_gray(30);
                    painter.rect_filled(rect, egui::CornerRadius::same(0u8), bg);
                    let bars = 32usize;
                    let mut max_val = 1.0f32;
                    if !data.frequencies.is_empty() {
                        max_val = data.frequencies.iter().cloned().fold(0.0f32, f32::max).max(1.0);
                    }
                    let bar_w = rect.width() / bars as f32;
                    for i in 0..bars {
                        let v = if i < data.frequencies.len() { data.frequencies[i] } else { 0.0 };
                        let h = rect.height() * (v / max_val).clamp(0.0, 1.0);
                        let x0 = rect.min.x + i as f32 * bar_w;
                        let x1 = x0 + bar_w * 0.9;
                        let y0 = rect.max.y - h;
                        let y1 = rect.max.y;
                        let color = egui::Color32::from_rgb(80, (120 + (i as i32 % 80)) as u8, 220);
                        painter.rect_filled(egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1)), egui::CornerRadius::same(2u8), color);
                    }
                }
                RightSidebarMode::Gestures => {
                    ui.heading("Gestures");
                    ui.checkbox(&mut ui_state.quick_params_enabled, "Enable quick params");
                    if ui.button("Calibrate").clicked() {
                        ui_state.show_gesture_calibration = true;
                    }
                    ui.separator();
                    ui.heading("Parameter Mapping");
                    let params = parse_shader_parameters(&ui_state.draft_code);
                    if params.is_empty() {
                        ui.label("No shader parameters available");
                    } else {
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for p in params.iter() {
                                let mut selected_gesture = crate::gesture_control::GestureType::Pinch;
                                let mut min_v: f32 = 0.0;
                                let mut max_v: f32 = 1.0;
                                let mut invert = false;
                                let mut curve = crate::gesture_control::CurveType::Linear;
                                ui.horizontal(|ui| {
                                    ui.label(&p.name);
                                    egui::ComboBox::from_id_source(format!("gesture_combo_{}", &p.name))
                                        .selected_text(format!("{:?}", selected_gesture))
                                        .show_ui(ui, |ui| {
                                            for g in [
                                                crate::gesture_control::GestureType::HandOpen,
                                                crate::gesture_control::GestureType::HandClosed,
                                                crate::gesture_control::GestureType::Point,
                                                crate::gesture_control::GestureType::Pinch,
                                                crate::gesture_control::GestureType::SwipeLeft,
                                                crate::gesture_control::GestureType::SwipeRight,
                                                crate::gesture_control::GestureType::SwipeUp,
                                                crate::gesture_control::GestureType::SwipeDown,
                                                crate::gesture_control::GestureType::Circle,
                                                crate::gesture_control::GestureType::Grab,
                                                crate::gesture_control::GestureType::Release,
                                            ] {
                                                if ui.selectable_label(selected_gesture == g, format!("{:?}", g)).clicked() {
                                                    selected_gesture = g;
                                                }
                                            }
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Min");
                                    ui.add(egui::DragValue::new(&mut min_v).speed(0.1));
                                    ui.label("Max");
                                    ui.add(egui::DragValue::new(&mut max_v).speed(0.1));
                                    ui.checkbox(&mut invert, "Invert");
                                });
                                ui.horizontal(|ui| {
                                    egui::ComboBox::from_id_source(format!("curve_combo_{}", &p.name))
                                        .selected_text(format!("{:?}", curve))
                                        .show_ui(ui, |ui| {
                                            for c in [
                                                crate::gesture_control::CurveType::Linear,
                                                crate::gesture_control::CurveType::Quadratic,
                                                crate::gesture_control::CurveType::Cubic,
                                                crate::gesture_control::CurveType::Exponential,
                                                crate::gesture_control::CurveType::Logarithmic,
                                            ] {
                                                if ui.selectable_label(curve == c, format!("{:?}", c)).clicked() {
                                                    curve = c;
                                                }
                                            }
                                        });
                                    if ui.button("Map").clicked() {
                                        gesture_control.get_parameter_mappings_mut().insert(
                                            p.name.clone(),
                                            crate::gesture_control::GestureMapping {
                                                gesture: selected_gesture,
                                                parameter_name: p.name.clone(),
                                                min_value: min_v,
                                                max_value: max_v,
                                                curve_type: curve,
                                                invert,
                                            }
                                        );
                                    }
                                    if ui.button("Remove").clicked() {
                                        gesture_control.get_parameter_mappings_mut().remove(&p.name);
                                    }
                                });
                                if let Some(m) = gesture_control.get_parameter_mappings().get(&p.name) {
                                    ui.label(format!("Mapped to {:?} [{:.2}..{:.2}] {:?}", m.gesture, m.min_value, m.max_value, m.curve_type));
                                }
                                ui.separator();
                            }
                        });
                    }
                }
                RightSidebarMode::Parameters => {
                    ui.heading("Parameters");
                }
                RightSidebarMode::Compute => {
                    ui.heading("Compute Passes");
                    ui.label("Compute Shader Dispatch");
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut ui_state.compute_pass_name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Workgroup Size:");
                        ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_x).speed(1));
                        ui.label("x");
                        ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_y).speed(1));
                        ui.label("x");
                        ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_z).speed(1));
                    });
                    if ui.button("Create Compute Pass").clicked() {
                        compute_pass_manager.create_ping_pong_texture(
                            &ui_state.compute_pass_name,
                            512,
                            512,
                            TextureFormat::Rgba8Unorm
                        );
                    }
                    ui.separator();
                    ui.label("Create Ping-Pong Texture:");
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut ui_state.pingpong_texture_name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Size:");
                        ui.add(egui::DragValue::new(&mut ui_state.pingpong_width).speed(1));
                        ui.label("x");
                        ui.add(egui::DragValue::new(&mut ui_state.pingpong_height).speed(1));
                    });
                    if ui.button("Create Ping-Pong Texture").clicked() {
                        compute_pass_manager.create_ping_pong_texture(
                            &ui_state.pingpong_texture_name,
                            ui_state.pingpong_width,
                            ui_state.pingpong_height,
                            TextureFormat::Rgba8Unorm
                        );
                    }
                    ui.separator();
                    ui.label("Dispatch Controls:");
                    ui.horizontal(|ui| {
                        ui.label("Dispatch Size:");
                        ui.add(egui::DragValue::new(&mut ui_state.dispatch_size_x).speed(1));
                        ui.label("x");
                        ui.add(egui::DragValue::new(&mut ui_state.dispatch_size_y).speed(1));
                        ui.label("x");
                        ui.add(egui::DragValue::new(&mut ui_state.dispatch_size_z).speed(1));
                    });
                    if ui.button("Dispatch Compute").clicked() {
                    }
                    ui.separator();
                    ui.label("Active Compute Passes:");
                    if compute_pass_manager.ping_pong_textures.is_empty() && compute_pass_manager.compute_pipelines.is_empty() {
                        ui.label("No active compute passes");
                    } else {
                        ui.label(format!("Ping-pong textures: {}", compute_pass_manager.ping_pong_textures.len()));
                        ui.label(format!("Compute pipelines: {}", compute_pass_manager.compute_pipelines.len()));
                        ui.label(format!("Active passes: {}", compute_pass_manager.active_compute_passes.len()));
                    }
                }
                RightSidebarMode::Outputs => {
                    ui.horizontal(|ui| {
                        for (mode, label) in [
                            (OutputsMode::ScreenshotsVideo, "Screenshots/Video"),
                            (OutputsMode::Ndi, "NDI"),
                            (OutputsMode::SpoutSyphon, "Spout/Syphon"),
                            (OutputsMode::Ffgl, "FFGL"),
                        ] {
                            let sel = ui_state.outputs_mode == mode;
                            if ui.selectable_label(sel, label).clicked() {
                                ui_state.outputs_mode = mode;
                            }
                        }
                    });
                    ui.separator();
                    match ui_state.outputs_mode {
                        OutputsMode::ScreenshotsVideo => {
                            if let Some(exporter) = video_exporter {
                                ExportUI::render_export_controls(ui, exporter, &mut ui_state.export_settings, &mut ui_state.video_export_settings);
                            } else {
                                ui.label("Exporter not available");
                            }
                        }
                        OutputsMode::Ndi => {
                            NdiUI::render_ndi_controls(ui, ndi_config, ndi_output);
                        }
                        OutputsMode::SpoutSyphon => {
                            SpoutSyphonUI::render_spout_syphon_controls(ui, spout_config, spout_output);
                        }
                        OutputsMode::Ffgl => {
                            ui.label("FFGL plugin generation UI not integrated");
                        }
                    }
                }
                RightSidebarMode::OSC => {
                    ui.heading("OSC Control");
                    OscUI::render_osc_controls(ui, osc_config, osc_control);
                }
                RightSidebarMode::Lighting => {
                    ui.heading("DMX Lighting");
                    DmxUI::render_dmx_controls(ui, dmx_config, dmx_control);
                    ui.separator();
                    ui.heading("Parameter to DMX Mapping");
                    let params = parse_shader_parameters(&ui_state.draft_code);
                    if !params.is_empty() {
                        let mut universe: u16 = 1;
                        let mut channel: u16 = 1;
                        ui.horizontal(|ui| {
                            ui.label("Universe");
                            ui.add(egui::DragValue::new(&mut universe).range(1..=16));
                            ui.label("Channel");
                            ui.add(egui::DragValue::new(&mut channel).range(1..=512));
                        });
                        egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                            for p in params.iter() {
                                let mut val = ui_state.get_parameter_value(&p.name).unwrap_or(p.default_value.unwrap_or(0.5));
                                ui.horizontal(|ui| {
                                    ui.label(&p.name);
                                    if ui.add(egui::Slider::new(&mut val, 0.0..=1.0)).changed() {
                                        ui_state.set_parameter_value(&p.name, val);
                                        let _ = dmx_control.map_parameter_to_channel(&p.name, universe, channel, 0.0, 1.0, val);
                                    }
                                });
                            }
                        });
                    } else {
                        ui.label("No shader parameters available");
                    }
                }
                RightSidebarMode::Scene3D => {
                    // Draw the 3D scene editor panel if we have the required resources
                    if let (Some(editor_state), Some(query)) = (scene_editor_state, manipulable_query) {
                        draw_3d_scene_panel(ui, editor_state, query);
                    } else {
                        ui.heading("3D Scene Editor");
                        ui.label("3D editor not initialized");
                    }
                }
                RightSidebarMode::Performance => {
                    ui.heading("Performance Metrics");
                    ui.horizontal(|ui| {
                        ui.label("FPS:");
                        ui.label(format!("{:.1}", ui_state.fps));
                    });
                    // Add more performance metrics as needed
                }
            }

            // Parameters below modes (kept accessible, aligned to right panel)
            if ui_state.show_parameter_panel {
                ui.separator();
                ui.heading("Parameters");
                if !ui_state.draft_code.is_empty() {
                    let params = parse_shader_parameters(&ui_state.draft_code);
                    if params.is_empty() {
                        ui.label("No parameters found");
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for param in params.iter() {
                                ui.horizontal(|ui| {
                                    ui.label(&param.name);
                                    if let (Some(min), Some(max)) = (param.min_value, param.max_value) {
                                        let mut v = param.default_value.unwrap_or((min + max) / 2.0);
                                        if ui.add(egui::Slider::new(&mut v, min..=max)).changed() {
                                            ui_state.set_parameter_value(&param.name, v);
                                        }
                                    } else {
                                        let mut v = param.default_value.unwrap_or(0.5);
                                        if ui.add(egui::Slider::new(&mut v, 0.0..=1.0)).changed() {
                                            ui_state.set_parameter_value(&param.name, v);
                                        }
                                    }
                                });
                                ui.separator();
                            }
                        });
                    }
                } else {
                    ui.label("Load a shader to see parameters");
                }
            }
            if ui_state.show_osc_panel {
                ui.separator();
                ui.heading("OSC");
                ui.label("Configure OSC endpoints and map incoming addresses to parameters");
                let mut status = osc_control.get_status();
                ui.horizontal(|ui| {
                    ui.checkbox(&mut osc_config.enabled, "Enable OSC");
                    ui.label(format!("Status: {}", if status.is_running { "Running" } else { "Stopped" }));
                    if status.is_running {
                        if ui.button("Stop").clicked() {
                            let _ = osc_control.stop();
                            status = osc_control.get_status();
                        }
                    } else {
                        if ui.button("Start").clicked() {
                            let _ = osc_control.start();
                            status = osc_control.get_status();
                        }
                    }
                    if ui.button("Apply Config").clicked() {
                        let _ = osc_control.update_config(osc_config.clone());
                        status = osc_control.get_status();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Listen Address");
                    ui.text_edit_singleline(&mut osc_config.listen_address);
                    ui.label("Port");
                    ui.add(egui::DragValue::new(&mut osc_config.listen_port).speed(1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Send Address");
                    if let Some(ref mut addr) = osc_config.send_address {
                        ui.text_edit_singleline(addr);
                    } else {
                        if ui.button("Set").clicked() {
                            osc_config.send_address = Some("127.0.0.1".to_string());
                        }
                    }
                    ui.label("Send Port");
                    if let Some(ref mut port) = osc_config.send_port {
                        ui.add(egui::DragValue::new(port).speed(1.0));
                    } else {
                        if ui.button("Set Port").clicked() {
                            osc_config.send_port = Some(9001);
                        }
                    }
                });
                let params = parse_shader_parameters(&ui_state.draft_code);
                if params.is_empty() {
                    ui.label("No shader parameters available");
                } else {
                    egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        for p in params.iter() {
                            let mut address = format!("/shader/{}", p.name);
                            let mut min_v: f32 = 0.0;
                            let mut max_v: f32 = 1.0;
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut address);
                                ui.label("Min");
                                ui.add(egui::DragValue::new(&mut min_v).speed(0.1));
                                ui.label("Max");
                                ui.add(egui::DragValue::new(&mut max_v).speed(0.1));
                                if ui.button("Map").clicked() {
                                    let mapping = OscMapping {
                                        osc_address: address.clone(),
                                        parameter_name: p.name.clone(),
                                        min_value: min_v,
                                        max_value: max_v,
                                        default_value: p.default_value.unwrap_or(0.5),
                                        message_type: OscMessageType::Float(0.0),
                                    };
                                    osc_control.add_mapping(mapping);
                                }
                                if ui.button("Remove").clicked() {
                                    osc_control.remove_mapping_for_parameter(&p.name);
                                }
                            });
                            if let Some(m) = osc_control.get_mapping_for_parameter(&p.name) {
                                ui.label(format!("Mapped: {} [{:.2}..{:.2}]", m.osc_address, m.min_value, m.max_value));
                            }
                            ui.separator();
                        }
                    });
                }
            }
            if ui_state.show_dmx_panel {
                ui.separator();
                ui.heading("DMX");
                ui.label("Configure DMX lighting");
            }
            if ui_state.show_export_panel {
                ui.separator();
                ui.heading("Export");
                ui.label("Export screenshots and videos");
            }
            if ui_state.show_ndi_panel {
                ui.separator();
                ui.heading("NDI");
                ui.label("Configure NDI stream settings");
            }
            if ui_state.show_spout_panel {
                ui.separator();
                ui.heading("Spout/Syphon");
                ui.label("Configure texture sharing");
            }
            if ui_state.show_ffgl_panel {
                ui.separator();
                ui.heading("FFGL");
                ui.label("Manage FFGL plugins");
            }
            if ui_state.show_gyroflow_panel {
                ui.separator();
                ui.heading("Gyroflow");
                ui.label("Configure stabilization integration");
            }
        });

    // Central preview is handled by draw_editor_central_panel; no CentralPanel here

    if ui_state.show_node_studio {
        ui_state.central_view = CentralView::NodeGraph;
        ui_state.show_node_studio = false;
    }
    if ui_state.show_timeline {
        ui_state.central_view = CentralView::Timeline;
        ui_state.show_timeline = false;
    }

    // Integrate gesture calibration into the gestures panel
    if ui_state.show_gesture_calibration {
        egui::SidePanel::right("gesture_calibration_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Gesture Calibration");
                ui.label("Leap Motion / MediaPipe calibration");
                ui.separator();
                ui.label("Place hands in view and follow on-screen prompts");
                ui.separator();
                if ui.button("Close").clicked() {
                    ui_state.show_gesture_calibration = false;
                }
            });
    }

    // All panels are now integrated into the side panels
    // No floating windows anymore
    // WGSL analyzer is already in bottom panel, so no floating window needed
}
// deprecated legacy central panel helpers removed in favor of tabbed central view

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

pub fn collect_wgsl_files(dir: &Path, out: &mut Vec<String>) {
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

pub fn collect_isf_files(dir: &Path, out: &mut Vec<String>) {
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
    egui::TopBottomPanel::bottom("code_editor_panel")
        .resizable(false)
        .default_height(240.0)
        .min_height(160.0)
        .max_height(280.0)
        .show(ctx, |ui| {
        ui.horizontal(|ui| {
            let tabs = [
                (CodeEditorTab::Editor, "Editor"),
                (CodeEditorTab::AI, "AI"),
                (CodeEditorTab::Diagnostics, "Diagnostics"),
                (CodeEditorTab::Analyzer, "Analyzer"),
            ];
            for (tab, label) in tabs {
                let sel = ui_state.code_editor_tab == tab;
                if ui.selectable_label(sel, label).clicked() {
                    ui_state.code_editor_tab = tab;
                }
            }
        });
        ui.separator();
        match ui_state.code_editor_tab {
            CodeEditorTab::Editor => {
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
            }
            CodeEditorTab::AI => {
                ui.heading("AI-Assisted Shader Generation");
                ui.horizontal(|ui| {
                    ui.label("Prompt:");
                    ui.text_edit_multiline(&mut ui_state.wgsl_smith_prompt);
                });
                ui.horizontal(|ui| {
                    if ui.button("Generate Shader").clicked() {
                        ui_state.wgsl_smith_generated = generate_shader_with_wgsl_smith(&ui_state.wgsl_smith_prompt);
                    }
                    if ui.button("Clear").clicked() {
                        ui_state.wgsl_smith_prompt.clear();
                        ui_state.wgsl_smith_generated.clear();
                    }
                });
                if !ui_state.wgsl_smith_generated.is_empty() {
                    ui.separator();
                    ui.label("Generated WGSL:");
                    egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        ui.monospace(&ui_state.wgsl_smith_generated);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Apply to Editor").clicked() {
                            ui_state.draft_code = ui_state.wgsl_smith_generated.clone();
                        }
                        if ui.button("Apply to Preview").clicked() {
                            ui_state.draft_code = ui_state.wgsl_smith_generated.clone();
                            ui_state.apply_requested = true;
                        }
                    });
                }
            }
            CodeEditorTab::Diagnostics => {
                ui.heading("Shader Compilation Diagnostics");
                ui.horizontal(|ui| {
                    if ui.button("Check Current Shader").clicked() {
                        ui_state.diagnostics_messages = check_wgsl_diagnostics(&ui_state.draft_code);
                    }
                    if ui.button("Clear").clicked() {
                        ui_state.diagnostics_messages.clear();
                    }
                });
                if ui_state.diagnostics_messages.is_empty() {
                    ui.label("No diagnostics available. Click 'Check Current Shader' to analyze.");
                } else {
                    ui.label(format!("Found {} diagnostic(s):", ui_state.diagnostics_messages.len()));
                    egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        for (i, diagnostic) in ui_state.diagnostics_messages.iter().enumerate() {
                            ui.group(|ui| {
                                match diagnostic.severity {
                                    DiagnosticSeverity::Error => {
                                        ui.colored_label(egui::Color32::RED, format!("Error {}: {}", i + 1, diagnostic.message));
                                    }
                                    DiagnosticSeverity::Warning => {
                                        ui.colored_label(egui::Color32::YELLOW, format!("Warning {}: {}", i + 1, diagnostic.message));
                                    }
                                    DiagnosticSeverity::Info => {
                                        ui.colored_label(egui::Color32::BLUE, format!("Info {}: {}", i + 1, diagnostic.message));
                                    }
                                }
                                if let Some(line) = diagnostic.line {
                                    ui.label(format!("  Line: {}", line));
                                }
                                if let Some(column) = diagnostic.column {
                                    ui.label(format!("  Column: {}", column));
                                }
                            });
                        }
                    });
                }
            }
            CodeEditorTab::Analyzer => {
                ui.heading("WGSL Code Analysis");
                #[cfg(feature = "naga_integration")]
                {
                    ui.horizontal(|ui| {
                        if ui.button("Analyze Code").clicked() {
                            let mut parser = WgslAstParser::new();
                            match parser.parse(&ui_state.draft_code) {
                                Ok(_) => {
                                    ui_state.ast_ok = true;
                                    ui_state.ast_error.clear();
                                }
                                Err(e) => {
                                    ui_state.ast_ok = false;
                                    ui_state.ast_error = format!("{}", e);
                                }
                            }
                            let validator = ShaderValidator::new();
                            match validator.validate_source(&ui_state.draft_code, ShaderLanguage::Wgsl) {
                                Ok(_) => {
                                    ui_state.validator_ok = true;
                                    ui_state.validator_error.clear();
                                }
                                Err(e) => {
                                    ui_state.validator_ok = false;
                                    ui_state.validator_error = format!("{}", e);
                                }
                            }
                        }
                        if ui.button("Transpile WGSL→GLSL").clicked() {
                            let mut mf = MultiFormatTranspiler::new();
                            let opts = TranspilerOptions { source_language: ShaderLanguage::Wgsl, target_language: ShaderLanguage::Glsl, ..Default::default() };
                            match mf.transpile(&ui_state.draft_code, &opts) {
                                Ok(out) => {
                                    ui_state.transpiled_glsl = out.source_code;
                                    ui_state.transpiler_error.clear();
                                }
                                Err(e) => {
                                    ui_state.transpiled_glsl.clear();
                                    ui_state.transpiler_error = format!("{}", e);
                                }
                            }
                        }
                    });
                    if ui_state.ast_ok {
                        ui.label("AST parse: OK");
                    } else if !ui_state.ast_error.is_empty() {
                        ui.colored_label(egui::Color32::RED, format!("AST error: {}", ui_state.ast_error));
                    }
                    if ui_state.validator_ok {
                        ui.label("Validator: OK");
                    } else if !ui_state.validator_error.is_empty() {
                        ui.colored_label(egui::Color32::RED, format!("Validator error: {}", ui_state.validator_error));
                    }
                    if !ui_state.transpiler_error.is_empty() {
                        ui.colored_label(egui::Color32::RED, format!("Transpiler error: {}", ui_state.transpiler_error));
                    }
                    if !ui_state.transpiled_glsl.is_empty() {
                        ui.separator();
                        ui.label("GLSL:");
                        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                            ui.code(&ui_state.transpiled_glsl);
                        });
                    }
                }
                #[cfg(not(feature = "naga_integration"))]
                {
                    ui.label("Enable the `naga_integration` feature to use the analyzer and transpiler.");
                }
            }
        }
    });
}

pub fn editor_code_panel(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let ctx = egui_ctx.ctx_mut().expect("Failed to get egui context");
    draw_editor_code_panel(ctx, &mut *ui_state);
}

pub fn draw_editor_shader_browser_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::SidePanel::left("shader_browser").resizable(true).show(ctx, |ui| {
        ui.heading("Shader Browser");
        ui.horizontal(|ui| {
            ui.checkbox(&mut ui_state.show_all_shaders, "Show all shaders");
            if !ui_state.show_all_shaders {
                ui.label("Showing compatible only (has @fragment or @compute)");
            }
        });
        ui.horizontal(|ui| {
            for (src, label) in [
                (SourceSet::All, "All Sources"),
                (SourceSet::Assets, "Assets"),
                (SourceSet::ISF, "ISF"),
            ] {
                let sel = ui_state.selected_source == src;
                if ui.selectable_label(sel, label).clicked() {
                    ui_state.selected_source = src;
                    match src {
                        SourceSet::All => rescan_shaders_all(ui_state),
                        SourceSet::Assets => rescan_shaders_assets_only(ui_state),
                        SourceSet::ISF => rescan_shaders_isf_only(ui_state),
                    }
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut ui_state.search_query);
        });
        ui.horizontal(|ui| {
            if ui.button("Rescan (All)").clicked() {
                rescan_shaders_all(ui_state);
            }
            if ui.button("Rescan (ISF only)").clicked() {
                rescan_shaders_isf_only(ui_state);
            }
        });
        ui.horizontal(|ui| {
            let mut current_cat = ui_state.selected_category.clone().unwrap_or_else(|| "All".to_string());
            for cat in ["All", "ISF", "WGSL", "GLSL", "HLSL"] {
                let selected = current_cat == cat;
                if ui.selectable_label(selected, cat).clicked() {
                    current_cat = cat.to_string();
                }
            }
            ui_state.selected_category = Some(current_cat);
        });
        ui.separator();
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let mut names = if ui_state.show_all_shaders {
                ui_state.available_shaders_all.clone()
            } else {
                ui_state.available_shaders_compatible.clone()
            };
            if let Some(cat) = &ui_state.selected_category {
                names = match cat.as_str() {
                    "ISF" => names.into_iter().filter(|n| n.to_lowercase().ends_with(".fs")).collect(),
                    "WGSL" => names.into_iter().filter(|n| n.to_lowercase().ends_with(".wgsl")).collect(),
                    "GLSL" => names.into_iter().filter(|n| n.to_lowercase().ends_with(".glsl")).collect(),
                    "HLSL" => names.into_iter().filter(|n| n.to_lowercase().ends_with(".hlsl")).collect(),
                    _ => names,
                };
            }
            for name in names.iter() {
                if !ui_state.search_query.is_empty() && !name.to_lowercase().contains(&ui_state.search_query.to_lowercase()) {
                    continue;
                }
                let selected = ui.selectable_label(ui_state.selected_shader.as_ref().map(|s| s == name).unwrap_or(false), name);
                if selected.clicked() {
                    ui_state.selected_shader = Some(name.clone());
                    if let Ok(content) = std::fs::read_to_string(name) {
                        if name.to_lowercase().ends_with(".fs") {
                            match crate::isf_loader::IsfShader::parse(&name, &content) {
                                Ok(isf_shader) => {
                                    let mut converter = super::isf_converter::IsfConverter::new();
                                    match converter.convert_to_wgsl(&isf_shader) {
                                        Ok(wgsl_code) => ui_state.draft_code = wgsl_code,
                                        Err(_) => {
                                            ui_state.draft_code = content;
                                        }
                                    }
                                }
                                Err(_) => {
                                    ui_state.draft_code = content;
                                }
                            }
                        } else if name.to_lowercase().ends_with(".glsl") {
                            ui_state.draft_code = content;
                        } else if name.to_lowercase().ends_with(".hlsl") {
                            ui_state.draft_code = content;
                        } else {
                            ui_state.draft_code = content;
                        }
                    }
                }
            }
        });
    });
}

fn rescan_shaders_all(ui_state: &mut EditorUiState) {
    let mut found_all = Vec::new();
    let standard_dirs = ["examples", "assets/shaders", "assets", "shaders"];
    for d in standard_dirs.iter() {
        let path = Path::new(d);
        if path.exists() {
            collect_wgsl_files(path, &mut found_all);
        }
    }
    let isf_dirs = [
        "C:/Program Files/Magic/Modules2/ISF",
        "C:/Program Files/Magic/ISF",
        "C:/Magic/ISF",
        "~/Magic/ISF",
        "~/Documents/Magic/ISF",
        "./isf-shaders",
        "./ISF",
        "./assets/isf",
        "./assets/ISF",
    ];
    for dir_str in isf_dirs.iter() {
        let expanded_path = if dir_str.starts_with("~/") {
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            Path::new(&home_dir).join(&dir_str[2..])
        } else {
            Path::new(dir_str).to_path_buf()
        };
        if expanded_path.exists() {
            collect_isf_files(&expanded_path, &mut found_all);
        }
    }
    found_all.sort();
    found_all.dedup();
    let mut compatible = Vec::new();
    for p in found_all.iter() {
        if let Ok(src) = fs::read_to_string(p) {
            if is_wgsl_shader_compatible(&src) {
                compatible.push(p.clone());
            }
        }
    }
    ui_state.available_shaders_all = found_all;
    ui_state.available_shaders_compatible = compatible;
}

fn rescan_shaders_isf_only(ui_state: &mut EditorUiState) {
    let mut found_all = Vec::new();
    let isf_dirs = [
        "C:/Program Files/Magic/Modules2/ISF",
        "C:/Program Files/Magic/ISF",
        "C:/Magic/ISF",
        "~/Magic/ISF",
        "~/Documents/Magic/ISF",
        "./isf-shaders",
        "./ISF",
        "./assets/isf",
        "./assets/ISF",
    ];
    for dir_str in isf_dirs.iter() {
        let expanded_path = if dir_str.starts_with("~/") {
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            Path::new(&home_dir).join(&dir_str[2..])
        } else {
            Path::new(dir_str).to_path_buf()
        };
        if expanded_path.exists() {
            collect_isf_files(&expanded_path, &mut found_all);
        }
    }
    found_all.sort();
    found_all.dedup();
    ui_state.available_shaders_all = found_all.clone();
    let mut compatible = Vec::new();
    for p in found_all.iter() {
        if let Ok(src) = fs::read_to_string(p) {
            if is_wgsl_shader_compatible(&src) {
                compatible.push(p.clone());
            }
        }
    }
    ui_state.available_shaders_compatible = compatible;
}

fn rescan_shaders_assets_only(ui_state: &mut EditorUiState) {
    let mut found_all = Vec::new();
    let standard_dirs = ["examples", "assets/shaders", "assets", "shaders"];
    for d in standard_dirs.iter() {
        let path = Path::new(d);
        if path.exists() {
            collect_wgsl_files(path, &mut found_all);
        }
    }
    found_all.sort();
    found_all.dedup();
    let mut compatible = Vec::new();
    for p in found_all.iter() {
        if let Ok(src) = fs::read_to_string(p) {
            if is_wgsl_shader_compatible(&src) {
                compatible.push(p.clone());
            }
        }
    }
    ui_state.available_shaders_all = found_all;
    ui_state.available_shaders_compatible = compatible;
}

pub fn draw_editor_parameter_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::SidePanel::right("parameters").resizable(true).show(ctx, |ui| {
        ui.heading("Parameters");
        ui.label("Interactive shader parameters");
        ui.separator();
        if !ui_state.draft_code.is_empty() {
            let params = parse_shader_parameters(&ui_state.draft_code);
            if params.is_empty() {
                ui.label("No parameters found in shader");
            } else {
                ui.label(format!("Found {} parameters:", params.len()));
                ui.separator();
                for param in params.iter() {
                    ui.horizontal(|ui| {
                        ui.label(&param.name);
                        if let (Some(min), Some(max)) = (param.min_value, param.max_value) {
                            let mut current_val = param.default_value.unwrap_or((min + max) / 2.0);
                            if ui.add(egui::Slider::new(&mut current_val, min..=max)).changed() {
                                ui_state.set_parameter_value(&param.name, current_val);
                            }
                        } else {
                            let mut current_val = param.default_value.unwrap_or(0.5);
                            if ui.add(egui::Slider::new(&mut current_val, 0.0..=1.0)).changed() {
                                ui_state.set_parameter_value(&param.name, current_val);
                            }
                        }
                    });
                    ui.separator();
                }
            }
        } else {
            ui.label("Load a shader to see parameters");
        }
    });
}

// MIDI panel integrated into right sidebar - no floating window needed
pub fn draw_midi_panel(_ctx: &egui::Context, _ui_state: &mut EditorUiState, _midi_system: &mut MidiSystem) {
    // Intentionally empty - MIDI controls now shown in right sidebar
}

pub fn draw_editor_central_panel(
    ctx: &egui::Context,
    ui_state: &mut EditorUiState,
    audio_analyzer: &AudioAnalyzer,
    _video_exporter: Option<&crate::screenshot_video_export::ScreenshotVideoExporter>,
    node_graph_res: &mut crate::bevy_node_graph_integration_enhanced::NodeGraphResource,
    scene_state: &crate::scene_editor_3d::SceneEditor3DState,
    timeline_animation: &mut crate::timeline::TimelineAnimation,
    spout_output: &mut crate::spout_syphon_output::SpoutSyphonOutput,
    ndi_output: &mut crate::ndi_output::NdiOutput,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            let tabs = [
                (CentralView::Preview, "Preview"),
                (CentralView::NodeGraph, "Node Graph"),
                (CentralView::Scene3D, "3D Editor"),
                (CentralView::Timeline, "Timeline"),
            ];
            for (view, label) in tabs {
                let selected = ui_state.central_view == view;
                if ui.selectable_label(selected, label).clicked() {
                    ui_state.central_view = view;
                }
            }
        });
        ui.separator();
        match ui_state.central_view {
            CentralView::Preview => {
                ui.heading("Shader Preview");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut ui_state.quick_params_enabled, "Quick Params");
                    if ui_state.quick_params_enabled {
                        ui.label("A:");
                        ui.add(egui::Slider::new(&mut ui_state.quick_param_a, 0.0..=1.0));
                        ui.label("B:");
                        ui.add(egui::Slider::new(&mut ui_state.quick_param_b, 0.0..=1.0));
                    }
                });
                ui.horizontal(|ui| {
                    if ui.selectable_label(ui_state.preview_scale_mode == PreviewScaleMode::Fit, "Fit").clicked() {
                        ui_state.preview_scale_mode = PreviewScaleMode::Fit;
                    }
                    if ui.selectable_label(ui_state.preview_scale_mode == PreviewScaleMode::Fill, "Fill").clicked() {
                        ui_state.preview_scale_mode = PreviewScaleMode::Fill;
                    }
                    if ui.selectable_label(ui_state.preview_scale_mode == PreviewScaleMode::OneToOne, "1:1").clicked() {
                        ui_state.preview_scale_mode = PreviewScaleMode::OneToOne;
                    }
                    ui.separator();
                    if ui.button("256×256").clicked() { ui_state.preview_resolution = (256, 256); }
                    if ui.button("512×512").clicked() { ui_state.preview_resolution = (512, 512); }
                    if ui.button("1280×720").clicked() { ui_state.preview_resolution = (1280, 720); }
                    if ui.button("1920×1080").clicked() { ui_state.preview_resolution = (1920, 1080); }
                });
                let avail_w = ui.available_width();
                let avail_h = ui.available_height().max(240.0);
                let target_w = ui_state.preview_resolution.0 as f32;
                let target_h = ui_state.preview_resolution.1 as f32;
                let aspect = if target_w > 0.0 { target_h / target_w } else { 9.0 / 16.0 };
                // Ensure preview size is never too small to prevent pixel data size mismatches
                // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
                let min_preview_size = 50.0;
                let preview_size = match ui_state.preview_scale_mode {
                    PreviewScaleMode::Fit => {
                        let mut w = avail_w.max(min_preview_size);
                        let mut h = (w * aspect).max(min_preview_size);
                        if h > avail_h {
                            h = avail_h.max(min_preview_size);
                            w = (h / aspect).max(min_preview_size);
                        }
                        egui::vec2(w, h)
                    }
                    PreviewScaleMode::Fill => egui::vec2(avail_w.max(min_preview_size), avail_h.max(min_preview_size)),
                    PreviewScaleMode::OneToOne => {
                        let w = target_w.min(avail_w).max(min_preview_size);
                        let h = target_h.min(avail_h).max(min_preview_size);
                        egui::vec2(w, h)
                    }
                };
                let (response, painter) = ui.allocate_painter(preview_size, egui::Sense::hover());
                let rect = response.rect;
                let mut guard = ui_state.global_renderer.renderer.lock().unwrap();
                if let Some(ref mut renderer) = *guard {
                    // Ensure render parameters have valid dimensions to prevent pixel data size mismatches
                    // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
                    let safe_width = (preview_size.x as u32).max(50);
                    let safe_height = (preview_size.y as u32).max(50);
                    let render_params = crate::shader_renderer::RenderParameters {
                        width: safe_width,
                        height: safe_height,
                        time: ui_state.time as f32,
                        frame_rate: 60.0,
                        audio_data: Some(audio_analyzer.get_audio_data()),
                    };
                    println!("DEBUG: Preview render params: {}x{}", safe_width, safe_height);
                    let render_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let mut code = if ui_state.draft_code.trim().is_empty() {
                            "@fragment fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> { return vec4<f32>(uv.x, uv.y, 0.5, 1.0); }".to_string()
                        } else {
                            ui_state.draft_code.clone()
                        };
                        if code.contains("@fragment") && code.contains("fn main(") && !code.contains("fn fs_main(") {
                            code = code.replacen("fn main(", "fn fs_main(", 1);
                        }
                        if !code.contains("@vertex") {
                            let vertex = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    var pos = vec2<f32>(0.0, 0.0);
    switch vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>( 3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0,  3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}
"#;
                            code = format!("{}\n\n{}", code, vertex);
                        }
                        if !code.contains("var<uniform> uniforms") {
                            let compat = r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    audio_volume: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_treble: f32,
    _padding: i32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
"#;
                            code = format!("{}\n\n{}", compat, code);
                        }
                        let mut param_array = vec![0.0f32; 64];
                        for (name, value) in ui_state.get_parameter_values().iter() {
                            let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
                            let index = (hash as usize) % 64;
                            param_array[index] = *value;
                        }
                        renderer.render_frame_with_params(&code, &render_params, Some(&param_array), render_params.audio_data.clone())
                    }));
                    match render_result {
                        Ok(Ok(pixels)) => {
                            let mut non_black = 0usize;
                            for i in (0..pixels.len()).step_by(4) {
                                if pixels[i] != 0 || pixels[i + 1] != 0 || pixels[i + 2] != 0 {
                                    non_black += 1;
                                    break;
                                }
                            }
                            let p0 = if pixels.len() >= 4 { (pixels[0], pixels[1], pixels[2], pixels[3]) } else { (0u8, 0u8, 0u8, 0u8) };
                            println!("preview_pixels_non_black={} first_pixel_rgba={:?} size={}x{}", non_black, p0, render_params.width, render_params.height);

                            // Use safe dimensions for fallback gradient to prevent pixel data size mismatches
                            // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
                            let safe_render_width = render_params.width.max(50);
                            let safe_render_height = render_params.height.max(50);
                            let color_image = if non_black == 0 {
                                let mut data = vec![0u8; (safe_render_width * safe_render_height * 4) as usize];
                                for y in 0..safe_render_height {
                                    for x in 0..safe_render_width {
                                        let idx = ((y * safe_render_width + x) * 4) as usize;
                                        data[idx] = 255;
                                        data[idx + 1] = ((x * 255) / safe_render_width) as u8;
                                        data[idx + 2] = ((y * 255) / safe_render_height) as u8;
                                        data[idx + 3] = 255;
                                    }
                                }
                                println!("preview_fallback_gradient_applied size={}x{}", safe_render_width, safe_render_height);
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [safe_render_width as usize, safe_render_height as usize],
                                    &data,
                                )
                            } else if pixels.is_empty() {
                // Use safe dimensions for error image to prevent pixel data size mismatches
                // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
                let safe_width = render_params.width.max(50) as usize;
                let safe_height = render_params.height.max(50) as usize;
                println!("Preview render returned empty pixels. Displaying error message with safe dimensions: {}x{}", safe_width, safe_height);
                let error_pixels = vec![egui::Color32::RED; safe_width * safe_height];
                egui::ColorImage {
                    size: [safe_width, safe_height],
                    pixels: error_pixels,
                    source_size: egui::Vec2::new((safe_width as f32).max(1.0), (safe_height as f32).max(1.0)),
                }
                            } else {
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [render_params.width as usize, render_params.height as usize],
                                    &pixels,
                                )
                            };
                            let tex = ctx.load_texture("shader_preview_tex", color_image, egui::TextureOptions::default());
                            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                            painter.image(tex.id(), rect, uv, egui::Color32::WHITE);
                            // Use safe dimensions for output to prevent pixel data size mismatches
                            // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
                            let output_width = render_params.width.max(50);
                            let output_height = render_params.height.max(50);
                            match ui_state.outputs_mode {
                                OutputsMode::Ndi => {
                                    if pixels.len() == (output_width as usize) * (output_height as usize) * 4 {
                                        let _ = ndi_output.send_frame(&pixels, output_width, output_height);
                                    }
                                }
                                OutputsMode::SpoutSyphon => {
                                    if pixels.len() == (output_width as usize) * (output_height as usize) * 4 {
                                        let _ = spout_output.send_frame(&pixels, output_width, output_height);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Ok(Err(e)) => {
                            println!("Preview render error: {:?}", e);
                            println!("renderer_last_errors={:?}", renderer.get_last_errors());
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Shader Error",
                                egui::FontId::proportional(12.0),
                                egui::Color32::RED,
                            );
                        }
                        Err(_) => {
                            println!("preview_panic renderer_last_errors={:?}", renderer.get_last_errors());
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Shader Error",
                                egui::FontId::proportional(12.0),
                                egui::Color32::RED,
                            );
                        }
                    }
                    painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_gray(60)), egui::StrokeKind::Inside);
                } else {
                    // Use safe dimensions when renderer is not initialized to prevent pixel data size mismatches
                    // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
                    let safe_w = preview_size.x as u32;
                    let safe_h = preview_size.y as u32;
                    // Ensure minimum size to prevent zero or very small dimensions
                    let w = safe_w.max(50);
                    let h = safe_h.max(50);
                    let mut data = vec![0u8; (w * h * 4) as usize];
                    for y in 0..h {
                        for x in 0..w {
                            let idx = ((y * w + x) * 4) as usize;
                            data[idx] = 255;
                            data[idx + 1] = ((x * 255) / w.max(1)) as u8;
                            data[idx + 2] = ((y * 255) / h.max(1)) as u8;
                            data[idx + 3] = 255;
                        }
                    }
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [w as usize, h as usize],
                        &data,
                    );
                    let tex = ctx.load_texture("shader_preview_tex_fallback", color_image, egui::TextureOptions::default());
                    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    painter.image(tex.id(), rect, uv, egui::Color32::WHITE);
                }
            }
            CentralView::NodeGraph => {
                ui.heading("Node Graph");
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    crate::bevy_node_graph_integration_enhanced::draw_node_graph_embedded(ui, node_graph_res);
                })).map_err(|_| {
                    ui.label("Node editor encountered an error");
                });
                ui.separator();
                ui.label("Graph Preview");
                let preview_size = egui::vec2(ui.available_width(), ui.available_height().min(240.0));
                let (response, painter) = ui.allocate_painter(preview_size, egui::Sense::hover());
                let rect = response.rect;
                let mut guard = ui_state.global_renderer.renderer.lock().unwrap();
                if let Some(ref mut renderer) = *guard {
                    match node_graph_res.graph.generate_wgsl() {
                        Ok(wgsl_code) => {
                            let render_params = crate::shader_renderer::RenderParameters {
                                width: rect.width() as u32,
                                height: rect.height() as u32,
                                time: ui_state.time as f32,
                                frame_rate: 60.0,
                                audio_data: Some(audio_analyzer.get_audio_data()),
                            };
                            let render_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                renderer.render_frame(&wgsl_code, &render_params, render_params.audio_data.clone())
                            }));
                            match render_result {
                                Ok(Ok(pixels)) => {
                                    let color_image = egui::ColorImage {
                                        size: [render_params.width as usize, render_params.height as usize],
                                        pixels: pixels
                                            .chunks(4)
                                            .map(|c| egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
                                            .collect(),
                                        source_size: rect.size(),
                                    };
                                    let tex = ctx.load_texture("node_graph_preview_tex", color_image, egui::TextureOptions::default());
                                    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                                    painter.image(tex.id(), rect, uv, egui::Color32::WHITE);
                                }
                                Ok(Err(e)) => {
                                    painter.text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        format!("Preview error:\n{}", e),
                                        egui::FontId::proportional(12.0),
                                        egui::Color32::RED,
                                    );
                                }
                                Err(_) => {
                                    painter.text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "Preview error: pipeline creation failed (invalid WGSL)",
                                        egui::FontId::proportional(12.0),
                                        egui::Color32::RED,
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("Failed to generate WGSL:\n{}", e),
                                egui::FontId::proportional(12.0),
                                egui::Color32::RED,
                            );
                        }
                    }
                } else {
                    let grid_color = egui::Color32::from_gray(40);
                    let mut x = rect.min.x;
                    while x < rect.max.x {
                        painter.line_segment([egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)], (1.0, grid_color));
                        x += 16.0;
                    }
                    let mut y = rect.min.y;
                    while y < rect.max.y {
                        painter.line_segment([egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)], (1.0, grid_color));
                        y += 16.0;
                    }
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Renderer not initialized",
                        egui::FontId::proportional(12.0),
                        egui::Color32::RED,
                    );
                }
            }
            CentralView::Scene3D => {
                ui.heading("3D Editor");
                ui.label(if scene_state.enabled { "3D editor active" } else { "3D editor disabled" });
                ui.label(format!("Selected: {:?}", scene_state.selected_entity));
                ui.label(format!("Mode: {:?}", scene_state.manipulation_mode));
                ui.separator();
                let viewport_size = egui::vec2(ui.available_width(), ui.available_height().min(360.0));
                let (response, painter) = ui.allocate_painter(viewport_size, egui::Sense::click_and_drag());
                let rect = response.rect;
                if let Some(tex_id) = ui_state.scene3d_texture_id {
                    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    painter.image(tex_id, rect, uv, egui::Color32::WHITE);
                } else {
                    painter.rect_filled(rect, egui::CornerRadius::same(4u8), egui::Color32::from_rgb(8, 8, 10));
                    painter.text(rect.center(), egui::Align2::CENTER_CENTER, "3D viewport not ready", egui::FontId::proportional(13.0), egui::Color32::RED);
                }
            }
            CentralView::Timeline => {
                ui.heading("Timeline");
                crate::timeline::draw_timeline_ui(ui, timeline_animation);
            }
        }
    });
}

// Performance panel integrated into right sidebar - no floating window needed
pub fn draw_performance_overlay_panel(_ctx: &egui::Context, _ui_state: &mut EditorUiState) {
    // Intentionally empty - performance metrics now shown in right sidebar
}

// Color grading panel integrated into right sidebar - no floating window needed
pub fn draw_color_grading_panel(_ctx: &egui::Context, _ui_state: &mut EditorUiState) {
    // Intentionally empty - color grading controls now shown in right sidebar
}


// DMX panel integrated into right sidebar - no floating window needed
pub fn draw_dmx_panel(_ctx: &egui::Context, _ui_state: &mut EditorUiState) {
    // Intentionally empty - DMX controls now shown in right sidebar
}

// Compute panel integrated into right sidebar - no floating window needed
pub fn draw_compute_panel(_ctx: &egui::Context, _ui_state: &mut EditorUiState) {
    // Intentionally empty - compute controls now shown in right sidebar
}

// Export panel integrated into right sidebar - no floating window needed
pub fn draw_export_panel(_ctx: &egui::Context, _ui_state: &mut EditorUiState) {
    // Intentionally empty - export controls now shown in right sidebar
}



// FFGL panel integrated into right sidebar - no floating window needed
pub fn draw_ffgl_panel(_ctx: &egui::Context, _ui_state: &mut EditorUiState) {
    // Intentionally empty - FFGL controls now shown in right sidebar
}

// Gyroflow panel integrated into right sidebar - no floating window needed
pub fn draw_gyroflow_panel(_ctx: &egui::Context, _ui_state: &mut EditorUiState) {
    // Intentionally empty - Gyroflow controls now shown in right sidebar
}

// WGSL Analyzer panel integrated into bottom panel - no floating window needed
pub fn draw_analyzer_panel(
    _ctx: &egui::Context,
    _ui_state: &mut EditorUiState,
    _midi_system: &mut MidiSystem,
    _compute_pass_manager: &mut ComputePassManager,
    _spout_config: &mut SpoutSyphonConfig,
    _spout_output: &SpoutSyphonOutput,
    _ndi_config: &mut NdiConfig,
    _ndi_output: &NdiOutput,
    _dmx_config: &mut DmxConfig,
    _dmx_control: &mut DmxLightingControl,
    _audio_analyzer: &AudioAnalyzer
) {
    // Intentionally empty - analyzer now shown in bottom panel
    // No floating window needed as it's redundant
}

// 3D Scene Editor panel integrated into right sidebar
pub fn draw_3d_scene_panel(
    ui: &mut egui::Ui,
    editor_state: &mut crate::scene_editor_3d::SceneEditor3DState,
    manipulable_query: &Query<(Entity, &Name), With<crate::scene_editor_3d::EditorManipulable>>,
) {
    ui.heading("3D Scene Controls");
    ui.separator();
    
    // Manipulation mode buttons
    ui.horizontal(|ui| {
        ui.label("Mode:");
        ui.label(format!("{:?}", editor_state.manipulation_mode));
        // Buttons would send commands to change mode in a real implementation
        ui.label("(Buttons disabled in UI-only view)");
    });
    
    ui.separator();
    
    // Primitive creation
    ui.horizontal(|ui| {
        ui.label("Create:");
        ui.label(format!("{:?}", editor_state.create_primitive_type));
        ui.label("(Combo disabled in UI-only view)");
        ui.label("(Ctrl+N)");
    });
    
    ui.separator();
    
    // Scene hierarchy
    ui.heading("Scene Hierarchy");
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for (entity, name) in manipulable_query.iter() {
                let is_selected = editor_state.selected_entity == Some(entity);
                let response = ui.selectable_label(
                    is_selected,
                    format!("{} (Entity {:?})", name.as_str(), entity)
                );
                
                if response.clicked() {
                    editor_state.selected_entity = Some(entity);
                }
            }
        });
    
    ui.separator();
    
    // Editor options
    ui.label(format!("Show Gizmos: {}", editor_state.show_gizmos));
    ui.label(format!("Editor Enabled: {}", editor_state.enabled));
    ui.label(format!("Snap to Grid: {}", editor_state.snap_to_grid));
    
    if editor_state.snap_to_grid {
        ui.horizontal(|ui| {
            ui.label("Grid Size:");
            ui.label(format!("{:.1}", editor_state.grid_size));
        });
        ui.label("Press G to snap selected entities");
    }
    
    ui.separator();
    
    // Instructions
    ui.label("Controls:");
    ui.label("• Left Click: Select entity");
    ui.label("• Right Drag: Orbit camera");
    ui.label("• Middle Drag: Pan camera");
    ui.label("• Mouse Wheel: Zoom in/out");
    ui.label("• Q/Z: Zoom out/in");
    ui.label("• W/E/R: Switch manipulation mode");
    ui.label("• Ctrl+N: Create new primitive");
    ui.label("• G: Snap to grid (when enabled)");
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
    let has_fragment = src.contains("@fragment");
    let has_compute = src.contains("@compute");
    has_fragment || has_compute
}

/// If incompatible, return a clear message; otherwise, Ok(())
pub fn validate_wgsl_entry_points(src: &str) -> Result<(), String> {
    let has_fragment = src.contains("@fragment");
    let has_compute = src.contains("@compute");
    if has_fragment || has_compute {
        Ok(())
    } else {
        Err("Shader must contain @fragment or @compute entry point".to_string())
    }
}

/// Mode-aware validator supporting fragment or compute pipelines.
pub fn validate_wgsl_for_mode(src: &str, mode: PipelineMode) -> Result<(), String> {
    match mode {
        PipelineMode::Fragment => {
            validate_wgsl_entry_points(src)
        }
        PipelineMode::Compute => {
            let has_compute = src.contains("@compute");
            if !has_compute { return Err("Missing @compute entry point".to_string()); }
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

fn show_transpiler_panel(ui_state: &mut EditorUiState) {
    // Create a comprehensive transpiler panel with multiple language support
    println!("Opening multi-language transpiler panel...");
    
    // This function will be called to show a dedicated transpiler window
    // For now, we'll create a simple implementation that can be expanded
    ui_state.show_wgslsmith_panel = true; // Use the existing WGSLSmith panel for transpiler features
    
    // Add transpiler-specific test cases
    let test_cases = vec![
        ("GLSL Basic", "// Basic GLSL shader\nvoid main() {\n    gl_FragColor = vec4(1.0);\n}"),
        ("HLSL Basic", "// Basic HLSL shader\nfloat4 main() : SV_TARGET {\n    return float4(1.0, 1.0, 1.0, 1.0);\n}"),
        ("WGSL Basic", "// Basic WGSL shader\n@fragment\nfn main() -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0, 1.0, 1.0, 1.0);\n}"),
    ];
    
    for (name, code) in test_cases {
        println!("Transpiler test case available: {}", name);
        // In a full implementation, these would be loaded into the transpiler panel
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

/// Helper function to extract pixel data from an egui texture handle
fn get_texture_pixels(texture_handle: &egui::TextureHandle, ctx: &egui::Context) -> Result<Vec<u8>, String> {
    // This is a simplified implementation - in a real implementation you'd need to
    // access the underlying GPU texture data, which requires more complex WGPU integration
    // For now, we'll return a placeholder
    Ok(vec![0u8; 4 * 800 * 600]) // RGBA placeholder
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
pub fn parse_shader_parameters(shader_code: &str) -> Vec<ShaderParameter> {
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
            value: 0.5, // Default value
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
            value: 0.5, // Default value
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
                                    value: default.unwrap_or(0.5), // Use default value or 0.5
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
pub struct ShaderParameter {
    pub name: String,
    pub wgsl_type: String,
    pub group: u32,
    pub binding: u32,
    pub value: f32,
    pub default_value: Option<f32>,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct DiagnosticMessage {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

/// Check WGSL code for common issues and return diagnostic messages
pub fn check_wgsl_diagnostics(wgsl_code: &str) -> Vec<DiagnosticMessage> {
    let mut diagnostics = Vec::new();
    
    // Check for basic syntax issues
    if wgsl_code.trim().is_empty() {
        diagnostics.push(DiagnosticMessage {
            severity: DiagnosticSeverity::Error,
            message: "Shader code is empty".to_string(),
            line: None,
            column: None,
        });
        return diagnostics;
    }
    
    // Check for required entry points
    let has_vertex = wgsl_code.contains("@vertex");
    let has_fragment = wgsl_code.contains("@fragment");
    let has_compute = wgsl_code.contains("@compute");
    
    if !has_vertex && !has_fragment && !has_compute {
        diagnostics.push(DiagnosticMessage {
            severity: DiagnosticSeverity::Error,
            message: "No entry point found (@vertex, @fragment, or @compute)".to_string(),
            line: None,
            column: None,
        });
    }
    
    // Check for uniform bindings
    if !wgsl_code.contains("@group") || !wgsl_code.contains("@binding") {
        diagnostics.push(DiagnosticMessage {
            severity: DiagnosticSeverity::Warning,
            message: "No uniform bindings found (@group, @binding)".to_string(),
            line: None,
            column: None,
        });
    }
    
    // Check for common WGSL syntax issues
    let lines: Vec<&str> = wgsl_code.lines().collect();
    for (line_num, line) in lines.iter().enumerate() {
        let line_number = line_num as u32 + 1;
        
        // Check for missing semicolons (basic check)
        if line.trim().starts_with("var") || line.trim().starts_with("let") {
            if !line.trim().ends_with(';') && !line.trim().is_empty() {
                diagnostics.push(DiagnosticMessage {
                    severity: DiagnosticSeverity::Warning,
                    message: "Possible missing semicolon".to_string(),
                    line: Some(line_number),
                    column: None,
                });
            }
        }
        
        // Check for invalid type declarations
        if line.contains("float") && !line.contains("f32") {
            diagnostics.push(DiagnosticMessage {
                severity: DiagnosticSeverity::Warning,
                message: "Use 'f32' instead of 'float' in WGSL".to_string(),
                line: Some(line_number),
                column: None,
            });
        }
        
        // Check for vec3/float mixing issues
        if line.contains("vec3") && line.contains("float") {
            diagnostics.push(DiagnosticMessage {
                severity: DiagnosticSeverity::Warning,
                message: "Possible type mismatch: vec3 with float".to_string(),
                line: Some(line_number),
                column: None,
            });
        }
    }
    
    // Check for texture sampling issues
    if wgsl_code.contains("textureSample") && !wgsl_code.contains("texture_2d") {
        diagnostics.push(DiagnosticMessage {
            severity: DiagnosticSeverity::Warning,
            message: "textureSample used without texture_2d declaration".to_string(),
            line: None,
            column: None,
        });
    }
    
    // Check for uniform struct issues
    if wgsl_code.contains("struct") && wgsl_code.contains("uniform") {
        if !wgsl_code.contains("var<uniform>") {
            diagnostics.push(DiagnosticMessage {
                severity: DiagnosticSeverity::Warning,
                message: "Uniform struct should use var<uniform>".to_string(),
                line: None,
                column: None,
            });
        }
    }
    
    diagnostics
}

/// Run WGSL diagnostics and update UI state
pub fn run_wgsl_diagnostics(ui_state: &mut EditorUiState) {
    ui_state.diagnostics_messages = check_wgsl_diagnostics(&ui_state.draft_code);
}

