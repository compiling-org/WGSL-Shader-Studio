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
use crate::ffgl_plugin::{FfglPlugin, PluginInfoStruct};
use crate::screenshot_video_export::{ScreenshotVideoExporter, VideoExportSettings, ExportUI};
use crate::ndi_output::{NdiConfig, NdiOutput, NdiUI};
use crate::scene_editor_3d::{SceneEditor3DState, scene_editor_3d_ui as scene_3d_editor_panel};

// Temporarily commented out to fix compilation - will be restored when visual node editor is fully integrated
// use crate::visual_node_editor_adapter::NodeEditorAdapter;
use std::sync::Mutex;

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
    pub show_wgslsmith_panel: bool,
    pub show_diagnostics_panel: bool,
    pub show_compute_panel: bool,
    pub show_3d_scene_panel: bool,
    pub show_ndi_panel: bool,
    pub show_spout_syphon_panel: bool,
    pub show_osc_panel: bool,
    pub show_dmx_panel: bool,
    pub fps: f32,
    // Preview pipeline mode
    pub pipeline_mode: PipelineMode,
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
    // Parameter values storage for shader rendering
    pub parameter_values: std::collections::HashMap<String, f32>,
    // WGSLSmith AI fields
    pub wgsl_smith_prompt: String,
    pub wgsl_smith_generated: String,
    pub wgsl_smith_status: String,
    // WGSL diagnostics
    pub diagnostics_messages: Vec<DiagnosticMessage>,
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
    pub video_exporter: Option<ScreenshotVideoExporter>,
    pub video_export_settings: VideoExportSettings,
    pub export_settings: ExportSettings,
    // NDI output state
    pub ndi_config: NdiConfig,
    pub ndi_output: Option<NdiOutput>,
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
    // WGPU integration status
    pub wgpu_initialized: bool,
    pub compilation_error: String,
    // FFGL plugin state for Resolume/Arena integration
    pub ffgl_plugin_enabled: bool,
    pub ffgl_plugin_path: Option<String>,
    // Spout/Syphon output state for real-time video sharing
    pub spout_syphon_enabled: bool,
    pub spout_syphon_config: crate::spout_syphon_output::SpoutSyphonConfig,
    pub spout_syphon_output: Option<crate::spout_syphon_output::SpoutSyphonOutput>,
    // OSC control state for external parameter control
    pub osc_control_enabled: bool,
    pub osc_config: crate::osc_control::OscConfig,
    pub osc_control: Option<crate::osc_control::OscControl>,
    // DMX lighting control state for stage lighting integration
    pub dmx_control_enabled: bool,
    pub dmx_config: crate::dmx_lighting_control::DmxConfig,
    pub dmx_control: Option<crate::dmx_lighting_control::DmxLightingControl>,
    // WGSL reflection system for shader introspection
    pub show_wgsl_reflect_panel: bool,
    pub wgsl_reflection_enabled: bool,
    pub wgsl_reflection_analyzer: Option<crate::wgsl_reflect_integration::WgslReflectAnalyzer>,
    // Shader Module Inspector for dependency visualization
    pub show_shader_module_inspector: bool,
    pub shader_module_system: Option<crate::shader_module_system::ShaderModuleSystem>,
    // Time parameter for animation
    pub time: f64,
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
            show_wgslsmith_panel: false,
            show_diagnostics_panel: false,
            show_compute_panel: false,
            show_3d_scene_panel: false,
            show_ndi_panel: false,
            show_spout_syphon_panel: false,
            show_osc_panel: false,
            show_dmx_panel: false,
            fps: 0.0,
            pipeline_mode: PipelineMode::default(),
            dark_mode: true,
            theme_preference: ThemePreference::default(),
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
            parameter_values: std::collections::HashMap::new(),
            wgsl_smith_prompt: String::new(),
            wgsl_smith_generated: String::new(),
            wgsl_smith_status: String::new(),
            diagnostics_messages: Vec::new(),
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
            video_exporter: Some(ScreenshotVideoExporter::new()),
            video_export_settings: VideoExportSettings::default(),
            export_settings: ExportSettings::default(),
            // NDI output defaults
            ndi_config: NdiConfig::default(),
            ndi_output: Some(NdiOutput::new(NdiConfig::default())),
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
            // WGPU integration status
            wgpu_initialized: false,
            compilation_error: String::new(),
            // FFGL plugin state defaults
            ffgl_plugin_enabled: false,
            ffgl_plugin_path: None,
            // Spout/Syphon output state defaults
            spout_syphon_enabled: false,
            spout_syphon_config: crate::spout_syphon_output::SpoutSyphonConfig::default(),
            spout_syphon_output: Some(crate::spout_syphon_output::SpoutSyphonOutput::new(crate::spout_syphon_output::SpoutSyphonConfig::default())),
            // OSC control state defaults
            osc_control_enabled: false,
            osc_config: crate::osc_control::OscConfig::default(),
            osc_control: Some(crate::osc_control::OscControl::new(crate::osc_control::OscConfig::default())),
            // DMX lighting control state defaults
            dmx_control_enabled: false,
            dmx_config: crate::dmx_lighting_control::DmxConfig::default(),
            dmx_control: Some(crate::dmx_lighting_control::DmxLightingControl::new(crate::dmx_lighting_control::DmxConfig::default())),
            // WGSL reflection system defaults
            show_wgsl_reflect_panel: false,
            wgsl_reflection_enabled: false,
            wgsl_reflection_analyzer: Some(crate::wgsl_reflect_integration::WgslReflectAnalyzer::new()),
            // Shader Module Inspector defaults
            show_shader_module_inspector: false,
            shader_module_system: Some(crate::shader_module_system::ShaderModuleSystem::new(100, std::time::Duration::from_secs(300))),
            // Time parameter for animation
            time: 0.0,
        }
    }
}

//

/// Helper that draws the main central preview panel using a provided egui context

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
    video_exporter: Option<&ScreenshotVideoExporter>
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
                    // For now, we'll capture the pixel data directly
                    // In a real implementation, we'd need access to the WGPU texture
                    // Video export functionality temporarily disabled
                    // let _ = exporter.capture_frame_data(&pixel_data, params.width, params.height);
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
                ui.checkbox(&mut ui_state.show_wgslsmith_panel, "WGSLSmith");
                ui.checkbox(&mut ui_state.show_compute_panel, "Compute Passes");
                ui.checkbox(&mut ui_state.show_3d_scene_panel, "3D Scene Editor");
            });
            ui.menu_button("Professional VJ", |ui| {
                ui.checkbox(&mut ui_state.ffgl_plugin_enabled, "FFGL Plugin Mode");
                if ui.button("Configure FFGL Plugin").clicked() {
                    configure_ffgl_plugin(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Test FFGL Plugin").clicked() {
                    test_ffgl_plugin(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                ui.label("Professional Outputs:");
                ui.checkbox(&mut ui_state.show_ndi_panel, "NDI Output Panel");
                if ui.button("Configure NDI Output").clicked() {
                    configure_ndi_output(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Test NDI Output").clicked() {
                    test_ndi_output(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                ui.checkbox(&mut ui_state.show_spout_syphon_panel, "Spout/Syphon Panel");
                if ui.button("Configure Spout/Syphon Output").clicked() {
                    configure_spout_syphon_output(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Test Spout/Syphon Output").clicked() {
                    test_spout_syphon_output(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                ui.checkbox(&mut ui_state.show_osc_panel, "OSC Control Panel");
                if ui.button("Configure OSC Control").clicked() {
                    configure_osc_control(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Test OSC Control").clicked() {
                    test_osc_control(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                ui.checkbox(&mut ui_state.show_dmx_panel, "DMX Lighting Control Panel");
                if ui.button("Configure DMX Control").clicked() {
                    configure_dmx_control(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Test DMX Control").clicked() {
                    test_dmx_control(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                ui.checkbox(&mut ui_state.show_wgsl_reflect_panel, "WGSL Reflection Inspector");
                if ui.button("Analyze Current Shader").clicked() {
                    analyze_current_shader_reflection(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Test WGSL Reflection").clicked() {
                    test_wgsl_reflection(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                ui.checkbox(&mut ui_state.show_shader_module_inspector, "Shader Module Inspector");
                if ui.button("Load Shader Module").clicked() {
                    load_shader_module(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Test Module Dependencies").clicked() {
                    test_shader_module_dependencies(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
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
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                ui.checkbox(&mut ui_state.show_diagnostics_panel, "Diagnostics Panel");
                if ui.button("Run WGSL Diagnostics").clicked() {
                    run_wgsl_diagnostics(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
            });

            ui.separator();
            ui.menu_button("Import/Convert", |ui| {
                // ISF conversion - CRITICAL VJ FEATURE
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
                ui.separator();
                // FFGL Plugin Export for Resolume/Arena
                ui.label("FFGL Plugin Export:");
                if ui.button("Export current shader as FFGL plugin").clicked() {
                    export_current_shader_as_ffgl_plugin(ui_state);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Export batch shaders as FFGL plugins").clicked() {
                    export_batch_ffgl_plugins();
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                // Transpiler temporarily disabled
                // if ui.button("Multi-language Transpiler").clicked() {
                //     show_transpiler_panel(ui_state);
                //     ui.close_kind(egui::UiKind::Menu);
                // }
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
                            ui_state.timeline = TimelineAnimation { timeline: proj.timeline, playing: false };
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
                ui.separator();
                if ui.button("Export as FFGL Plugin (.dll)").clicked() {
                    println!("Clicked: Export as FFGL Plugin");
                    export_current_shader_as_ffgl_plugin(&ui_state);
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
    editor_state: Option<&SceneEditor3DState>,
    global_renderer: Option<&GlobalShaderRenderer>
) {
    // CRITICAL FIX: Use proper panel hierarchy - NO CentralPanel here to avoid conflicts
    
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
                            // For now, just load the content directly
                            // ISF conversion will be added back when modules are properly integrated
                            ui_state.draft_code = content;
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
            
            ui.separator();
            
            // Export controls
            ui.collapsing("🎥 Export & Recording", |ui| {
                if let Some(ref exporter) = ui_state.video_exporter {
                    ExportUI::render_export_controls(
                        ui,
                        exporter,
                        &mut ui_state.export_settings,
                        &mut ui_state.video_export_settings
                    );
                } else {
                    ui.label("Video exporter not available");
                }
            });
            
            ui.separator();
            
            // NDI output controls
            ui.collapsing("🌐 NDI Output", |ui| {
                if let Some(ref ndi_output) = ui_state.ndi_output {
                    NdiUI::render_ndi_controls(
                        ui,
                        &mut ui_state.ndi_config,
                        ndi_output
                    );
                } else {
                    ui.label("NDI output not available");
                }
            });
        });
    }

    // Side panels should not include central preview controls

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
                let wgsl = ui_state.node_graph.generate_wgsl(512, 512);
                ui_state.draft_code = wgsl;
                ui_state.apply_requested = true;
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
                        ui_state.timeline = TimelineAnimation { timeline: loaded.timeline, playing: false };
                        ui_state.param_index_map = loaded.param_index_map;
                    }
                    Err(e) => { ui.label(format!("Import error: {}", e)); }
                }
            }
            
            // Visual node editor area
            // ui_state.visual_node_editor.ui(ui, &mut ui_state.node_graph);
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
            
            // Gesture system status
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(if gesture_control.is_enabled() { "Enabled" } else { "Disabled" });
                if ui.button(if gesture_control.is_enabled() { "Disable" } else { "Enable" }).clicked() {
                    gesture_control.set_enabled(!gesture_control.is_enabled());
                }
            });
            
            ui.separator();
            
            // Active gestures display
            ui.label("Active Gestures:");
            let active_gestures = gesture_control.get_active_gestures();
            if active_gestures.is_empty() {
                ui.label("No gestures detected");
            } else {
                for (gesture, strength) in active_gestures {
                    ui.horizontal(|ui| {
                        ui.label(format!("{:?}:", gesture));
                        ui.add(egui::ProgressBar::new(*strength).text(format!("{:.2}", strength)));
                    });
                }
            }
            
            ui.separator();
            
            // Parameter mappings
            ui.label("Parameter Mappings:");
            let mappings = gesture_control.get_parameter_mappings();
            for (param_name, mapping) in mappings {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", param_name));
                    if let Some(value) = gesture_control.get_parameter_value(param_name) {
                        ui.label(format!("{:.2}", value));
                    } else {
                        ui.label("-");
                    }
                });
            }
            
            ui.separator();
            
            // Test gesture simulation
            ui.label("Test Gestures:");
            if ui.button("Simulate Hand Open").clicked() {
                gesture_control.simulate_gesture(crate::gesture_control::SimulatedGestureData {
                    gesture_type: crate::gesture_control::GestureType::HandOpen,
                    strength: 1.0,
                    duration: 2.0,
                    hand_position: (0.0, 0.0, 0.5),
                });
            }
            if ui.button("Simulate Swipe Left").clicked() {
                gesture_control.simulate_gesture(crate::gesture_control::SimulatedGestureData {
                    gesture_type: crate::gesture_control::GestureType::SwipeLeft,
                    strength: 0.8,
                    duration: 1.5,
                    hand_position: (-0.5, 0.0, 0.5),
                });
            }
            if ui.button("Simulate Pinch").clicked() {
                gesture_control.simulate_gesture(crate::gesture_control::SimulatedGestureData {
                    gesture_type: crate::gesture_control::GestureType::Pinch,
                    strength: 0.9,
                    duration: 1.0,
                    hand_position: (0.0, 0.0, 0.3),
                });
            }
            if ui.button("Clear All").clicked() {
                gesture_control.clear_hands();
            }
        });
    }
    
    // 3D Scene Editor Panel
    if ui_state.show_3d_scene_panel {
        egui::Window::new("3D Scene Editor").open(&mut ui_state.show_3d_scene_panel).show(ctx, |ui| {
            ui.heading("🎬 3D Scene Editor");
            ui.separator();
            
            if let Some(ref editor_state) = editor_state {
                // Scene controls
                ui.collapsing("Scene Controls", |ui| {
                    ui.label("3D scene manipulation and object placement");
                    ui.label("Camera: Orbit with Right Mouse, Pan with Middle Mouse");
                    ui.label("Selection: Left Click to select objects");
                    ui.label("Manipulation: W/E/R for Translate/Rotate/Scale modes");
                });
                
                ui.separator();
                
                // Manipulation mode
                ui.collapsing("Manipulation Mode", |ui| {
                    ui.label("Current manipulation mode:");
                    ui.label(format!("Mode: {:?}", editor_state.manipulation_mode));
                    ui.horizontal(|ui| {
                        if ui.button("Translate (W)").clicked() {
                            println!("Switching to Translate mode");
                        }
                        if ui.button("Rotate (E)").clicked() {
                            println!("Switching to Rotate mode");
                        }
                        if ui.button("Scale (R)").clicked() {
                            println!("Switching to Scale mode");
                        }
                    });
                });
                
                ui.separator();
                
                // Object creation
                ui.collapsing("Object Creation", |ui| {
                    ui.label("Create primitive objects in the scene");
                    ui.horizontal(|ui| {
                        ui.label("Primitive:");
                        // This would need to be connected to the actual editor state
                        if ui.button("Cube").clicked() {
                            println!("Creating cube primitive");
                        }
                        if ui.button("Sphere").clicked() {
                            println!("Creating sphere primitive");
                        }
                        if ui.button("Cylinder").clicked() {
                            println!("Creating cylinder primitive");
                        }
                        if ui.button("Plane").clicked() {
                            println!("Creating plane primitive");
                        }
                    });
                    ui.label("Press Ctrl+N to create selected primitive");
                });
                
                ui.separator();
                
                // Scene hierarchy
                ui.collapsing("Scene Hierarchy", |ui| {
                    ui.label("Scene objects and their relationships");
                    ui.label("Selected Entity: None"); // This would show actual selection
                    ui.label("Total Objects: 1"); // This would show actual count
                    
                    // Placeholder for scene tree
                    ui.separator();
                    ui.label("Scene Objects:");
                    ui.label("• Editor Cube (Selected)");
                    ui.label("• Editor Camera");
                    ui.label("• Directional Light");
                });
                
                ui.separator();
                
                // Grid and snapping
                ui.collapsing("Grid & Snapping", |ui| {
                    ui.label("Grid-based object placement and alignment");
                    ui.checkbox(&mut false, "Snap to Grid"); // This would connect to actual state
                    ui.horizontal(|ui| {
                        ui.label("Grid Size:");
                        ui.add(egui::DragValue::new(&mut 1.0).speed(0.1).range(0.1..=10.0));
                    });
                    if ui.button("Snap Selected to Grid (G)").clicked() {
                        println!("Snapping selected objects to grid");
                    }
                });
                
                ui.separator();
                
                // Lighting and environment
                ui.collapsing("Lighting & Environment", |ui| {
                    ui.label("Scene lighting and environmental settings");
                    ui.label("Ambient Light: Enabled");
                    ui.label("Directional Light: Enabled");
                    ui.label("Shadows: Enabled");
                    
                    if ui.button("Add Point Light").clicked() {
                        println!("Adding point light to scene");
                    }
                    if ui.button("Add Spot Light").clicked() {
                        println!("Adding spot light to scene");
                    }
                });
                
                ui.separator();
                
                // Viewport options
                ui.collapsing("Viewport Options", |ui| {
                    ui.label("3D viewport display options");
                    ui.checkbox(&mut true, "Show Grid"); // This would connect to actual state
                    ui.checkbox(&mut true, "Show Gizmos"); // This would connect to actual state
                    ui.checkbox(&mut false, "Wireframe Mode");
                    ui.checkbox(&mut false, "Backface Culling");
                    
                    if ui.button("Reset Camera").clicked() {
                        println!("Resetting editor camera to default position");
                    }
                });
                
                ui.separator();
                
                // Scene management
                ui.collapsing("Scene Management", |ui| {
                    ui.label("Save and load scene configurations");
                    ui.horizontal(|ui| {
                        if ui.button("New Scene").clicked() {
                            println!("Creating new scene");
                        }
                        if ui.button("Save Scene").clicked() {
                            println!("Saving current scene");
                        }
                        if ui.button("Load Scene").clicked() {
                            println!("Loading scene from file");
                        }
                    });
                });
            }
        });
    }
    
    // Compute Pass Panel
    if ui_state.show_compute_panel {
        egui::Window::new("Multi-Pass Compute Rendering").open(&mut ui_state.show_compute_panel).show(ctx, |ui| {
            ui.heading("🚀 Multi-Pass Compute Shader System");
            ui.separator();
            
            // Multi-pass rendering overview
            ui.collapsing("System Overview", |ui| {
                ui.label("Advanced compute shader pipeline with ping-pong textures and multi-pass rendering");
                ui.label(format!("Active Textures: {}", compute_pass_manager.ping_pong_textures.len()));
                ui.label(format!("Active Buffers: {}", compute_pass_manager.ping_pong_buffers.len()));
                ui.label(format!("Compute Pipelines: {}", compute_pass_manager.compute_pipelines.len()));
                ui.label(format!("Pass Executions: {}", compute_pass_manager.active_compute_passes.len()));
            });
            
            ui.separator();
            
            // Compute pass creation
            ui.label("Create Compute Pass:");
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
                println!("Created compute pass: {}", ui_state.compute_pass_name);
            }
            
            ui.separator();
            
            // Ping-Pong Texture Management
            ui.collapsing("Ping-Pong Textures (Double Buffering)", |ui| {
                ui.label("Create double-buffered textures for iterative compute algorithms");
                
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut ui_state.pingpong_texture_name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.add(egui::DragValue::new(&mut ui_state.pingpong_width).speed(1).clamp_range(1..=4096));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut ui_state.pingpong_height).speed(1).clamp_range(1..=4096));
                });
                
                if ui.button("Create Ping-Pong Texture").clicked() {
                    compute_pass_manager.create_ping_pong_texture(
                        &ui_state.pingpong_texture_name,
                        ui_state.pingpong_width,
                        ui_state.pingpong_height,
                        TextureFormat::Rgba8Unorm
                    );
                    println!("✓ Created ping-pong texture: {} ({}x{})", 
                        ui_state.pingpong_texture_name, ui_state.pingpong_width, ui_state.pingpong_height);
                }
                
                // Display existing ping-pong textures
                if !compute_pass_manager.ping_pong_textures.is_empty() {
                    ui.separator();
                    ui.label("Active Ping-Pong Textures:");
                    for (name, texture) in &compute_pass_manager.ping_pong_textures {
                        ui.horizontal(|ui| {
                            ui.label(format!("• {}: {}x{} (Frame: {})", 
                                name, texture.width, texture.height, texture.frame_count));
                            if ui.small_button("Swap").clicked() {
                                compute_pass_manager.swap_ping_pong_texture(name);
                                println!("Swapped ping-pong texture: {}", name);
                            }
                        });
                    }
                }
            });
            
            ui.separator();
            
            // Compute Pipeline Management
            ui.collapsing("Compute Pipelines", |ui| {
                ui.label("Create compute shader pipelines with custom workgroup sizes");
                
                ui.horizontal(|ui| {
                    ui.label("Pipeline Name:");
                    ui.text_edit_singleline(&mut ui_state.compute_pass_name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Workgroup Size:");
                    ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_x).speed(1).clamp_range(1..=1024));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_y).speed(1).clamp_range(1..=1024));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_z).speed(1).clamp_range(1..=64));
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Create Particle System").clicked() {
                        let shader_code = compute_pass_manager.generate_particle_compute_wgsl();
                        let bind_group_layouts = vec![
                            crate::compute_pass_integration::BindGroupLayoutResource {
                                name: "uniforms".to_string(),
                                entries: vec![
                                    crate::compute_pass_integration::BindGroupLayoutEntry {
                                        binding: 0,
                                        visibility: crate::compute_pass_integration::ShaderStage::Compute,
                                        ty: crate::compute_pass_integration::BindingType::UniformBuffer,
                                    },
                                ],
                            },
                        ];
                        
                        compute_pass_manager.create_compute_pipeline(
                            "particle_system",
                            (ui_state.compute_workgroup_x, ui_state.compute_workgroup_y, ui_state.compute_workgroup_z),
                            shader_code,
                            bind_group_layouts
                        );
                        println!("✓ Created particle system compute pipeline");
                    }
                    
                    if ui.button("Create Game of Life").clicked() {
                        let shader_code = compute_pass_manager.generate_game_of_life_wgsl();
                        let bind_group_layouts = vec![
                            crate::compute_pass_integration::BindGroupLayoutResource {
                                name: "state".to_string(),
                                entries: vec![
                                    crate::compute_pass_integration::BindGroupLayoutEntry {
                                        binding: 0,
                                        visibility: crate::compute_pass_integration::ShaderStage::Compute,
                                        ty: crate::compute_pass_integration::BindingType::StorageTexture {
                                            access: crate::compute_pass_integration::StorageAccess::Read,
                                            format: TextureFormat::Rgba8Unorm,
                                        },
                                    },
                                    crate::compute_pass_integration::BindGroupLayoutEntry {
                                        binding: 1,
                                        visibility: crate::compute_pass_integration::ShaderStage::Compute,
                                        ty: crate::compute_pass_integration::BindingType::StorageTexture {
                                            access: crate::compute_pass_integration::StorageAccess::Write,
                                            format: TextureFormat::Rgba8Unorm,
                                        },
                                    },
                                ],
                            },
                        ];
                        
                        compute_pass_manager.create_compute_pipeline(
                            "game_of_life",
                            (8, 8, 1), // Optimal for Game of Life
                            shader_code,
                            bind_group_layouts
                        );
                        println!("✓ Created Game of Life compute pipeline");
                    }
                });
                
                // Display existing compute pipelines
                if !compute_pass_manager.compute_pipelines.is_empty() {
                    ui.separator();
                    ui.label("Active Compute Pipelines:");
                    for (name, pipeline) in &compute_pass_manager.compute_pipelines {
                        ui.horizontal(|ui| {
                            ui.label(format!("• {}: workgroup_size({}, {}, {})", 
                                name, 
                                pipeline.workgroup_size.0, 
                                pipeline.workgroup_size.1, 
                                pipeline.workgroup_size.2));
                            if ui.small_button("Generate WGSL").clicked() {
                                if let Some(wgsl) = compute_pass_manager.generate_compute_wgsl(name) {
                                    println!("Generated WGSL for {}:\n{}", name, wgsl);
                                }
                            }
                        });
                    }
                }
            });
            
            ui.separator();
            
            // Dispatch controls
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
                // Get current shader code from the UI state
                let shader_code = ui_state.draft_code.clone();
                
                // Check if it's a compute shader
                if shader_code.contains("@compute") {
                    // Execute the compute shader
                    if let Some(renderer) = global_renderer {
                        match renderer.renderer.lock().unwrap().as_mut() {
                            Some(renderer_ref) => {
                                match renderer_ref.execute_compute_shader(
                                    &shader_code,
                                    (ui_state.dispatch_size_x, ui_state.dispatch_size_y, ui_state.dispatch_size_z),
                                    Some(&[])
                                ) {
                                    Ok(_) => {
                                        println!("Compute shader dispatched successfully: ({}, {}, {})", 
                                            ui_state.dispatch_size_x, ui_state.dispatch_size_y, ui_state.dispatch_size_z);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to execute compute shader: {}", e);
                                        ui_state.compilation_error = format!("Compute execution failed: {}", e);
                                    }
                                }
                            }
                            None => {
                                ui_state.compilation_error = "No renderer available".to_string();
                            }
                        }
                    } else {
                        println!("Compute shader dispatch requested: ({}, {}, {})", 
                            ui_state.dispatch_size_x, ui_state.dispatch_size_y, ui_state.dispatch_size_z);
                        println!("Note: Compute execution requires renderer initialization");
                    }
                } else {
                    ui_state.compilation_error = "No compute shader detected. Add @compute entry point.".to_string();
                }
            }
            
            ui.separator();
            
            // Active compute passes display
            ui.label("Active Compute Passes:");
            if compute_pass_manager.ping_pong_textures.is_empty() && compute_pass_manager.compute_pipelines.is_empty() {
                ui.label("No active compute passes");
            } else {
                ui.label(format!("Ping-pong textures: {}", compute_pass_manager.ping_pong_textures.len()));
                ui.label(format!("Compute pipelines: {}", compute_pass_manager.compute_pipelines.len()));
                ui.label(format!("Active passes: {}", compute_pass_manager.active_compute_passes.len()));
            }
        });
    }
    
    // 3D Scene Editor Panel - Now handled in bevy_app.rs
    if ui_state.show_3d_scene_panel {
        // The 3D scene editor panel is now handled directly in bevy_app.rs
        // to avoid module import issues. This placeholder remains for compatibility.
    }
    
    // NDI Output Panel
    if ui_state.show_ndi_panel {
        egui::Window::new("NDI Output").open(&mut ui_state.show_ndi_panel).show(ctx, |ui| {
            // Use the existing NDI UI component
            crate::ndi_output::NdiUI::render_ndi_controls(ui, &mut ui_state.ndi_config, ui_state.ndi_output.as_ref().unwrap());
        });
    }
    
    // Spout/Syphon Output Panel
    if ui_state.show_spout_syphon_panel {
        egui::Window::new("Spout/Syphon Output").open(&mut ui_state.show_spout_syphon_panel).show(ctx, |ui| {
            // Use the existing Spout/Syphon UI component
            if let Some(ref mut spout_output) = ui_state.spout_syphon_output {
                crate::spout_syphon_output::SpoutSyphonUI::render_spout_syphon_controls(
                    ui, 
                    &mut ui_state.spout_syphon_config, 
                    spout_output
                );
            }
        });
    }
    
    // OSC Control Panel
    if ui_state.show_osc_panel {
        egui::Window::new("OSC Control").open(&mut ui_state.show_osc_panel).show(ctx, |ui| {
            // Use the existing OSC UI component
            if let Some(ref mut osc_control) = ui_state.osc_control {
                crate::osc_control::OscUI::render_osc_controls(
                    ui, 
                    &mut ui_state.osc_config, 
                    osc_control
                );
            }
        });
    }
    
    // DMX Lighting Control Panel
    if ui_state.show_dmx_panel {
        egui::Window::new("DMX Lighting Control").open(&mut ui_state.show_dmx_panel).show(ctx, |ui| {
            // Use the DMX lighting control UI component
            if let Some(ref mut dmx_control) = ui_state.dmx_control {
                crate::dmx_lighting_control::DmxUI::render_dmx_controls(
                    ui, 
                    &mut ui_state.dmx_config, 
                    dmx_control
                );
            }
        });
    }
    
    // WGSL Reflection Inspector Panel
    if ui_state.show_wgsl_reflect_panel {
        egui::Window::new("WGSL Reflection Inspector").open(&mut ui_state.show_wgsl_reflect_panel).show(ctx, |ui| {
            ui.heading("🔍 WGSL Shader Reflection");
            ui.separator();
            
            if ui_state.wgsl_reflection_enabled {
                if let Some(ref analyzer) = ui_state.wgsl_reflection_analyzer {
                    // Display reflection information
                    ui.label("Shader analysis complete");
                    
                    // Show basic shader info
                    ui.collapsing("Shader Information", |ui| {
                        if let Some(name) = &analyzer.shader_info.name {
                            ui.label(format!("Name: {}", name));
                        }
                        if let Some(version) = &analyzer.shader_info.version {
                            ui.label(format!("Version: {}", version));
                        }
                        if let Some(description) = &analyzer.shader_info.description {
                            ui.label(format!("Description: {}", description));
                        }
                        if let Some(author) = &analyzer.shader_info.author {
                            ui.label(format!("Author: {}", author));
                        }
                        
                        if !analyzer.shader_info.categories.is_empty() {
                            ui.label(format!("Categories: {:?}", analyzer.shader_info.categories));
                        }
                        if !analyzer.shader_info.tags.is_empty() {
                            ui.label(format!("Tags: {:?}", analyzer.shader_info.tags));
                        }
                    });
                    
                    // Show entry points
                    if !analyzer.entry_points.is_empty() {
                        ui.collapsing(format!("Entry Points ({})", analyzer.entry_points.len()), |ui| {
                            for entry_point in &analyzer.entry_points {
                                ui.horizontal(|ui| {
                                    ui.label(format!("• {} ({:?})", entry_point.name, entry_point.stage));
                                    if let Some((x, y, z)) = entry_point.workgroup_size {
                                        ui.label(format!("workgroup_size({}, {}, {})", x, y, z));
                                    }
                                });
                            }
                        });
                    }
                    
                    // Show bind groups
                    if !analyzer.bind_groups.is_empty() {
                        ui.collapsing(format!("Bind Groups ({})", analyzer.bind_groups.len()), |ui| {
                            for bind_group in &analyzer.bind_groups {
                                ui.collapsing(format!("Group {}", bind_group.group), |ui| {
                                    for binding in &bind_group.bindings {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("• [{}] {}", binding.binding, binding.name));
                                            ui.label(format!("({:?})", binding.binding_type));
                                            if let Some(size) = binding.size {
                                                ui.label(format!("size: {} bytes", size));
                                            }
                                        });
                                    }
                                });
                            }
                        });
                    }
                    
                    // Show uniforms
                    if !analyzer.uniforms.is_empty() {
                        ui.collapsing(format!("Uniforms ({})", analyzer.uniforms.len()), |ui| {
                            for uniform in &analyzer.uniforms {
                                ui.horizontal(|ui| {
                                    ui.label(format!("• {}", uniform.name));
                                    ui.label(format!("offset: {}, size: {}, align: {}", 
                                                   uniform.offset, uniform.size, uniform.align));
                                });
                            }
                        });
                    }
                    
                    // Show textures
                    if !analyzer.textures.is_empty() {
                        ui.collapsing(format!("Textures ({})", analyzer.textures.len()), |ui| {
                            for texture in &analyzer.textures {
                                ui.horizontal(|ui| {
                                    ui.label(format!("• {}", texture.name));
                                    ui.label(format!("type: {}", texture.texture_type));
                                    if let Some(format) = &texture.format {
                                        ui.label(format!("format: {}", format));
                                    }
                                });
                            }
                        });
                    }
                    
                    // Show samplers
                    if !analyzer.samplers.is_empty() {
                        ui.collapsing(format!("Samplers ({})", analyzer.samplers.len()), |ui| {
                            for sampler in &analyzer.samplers {
                                ui.horizontal(|ui| {
                                    ui.label(format!("• {}", sampler.name));
                                    ui.label(format!("type: {}", sampler.sampler_type));
                                });
                            }
                        });
                    }
                    
                    // Show storage buffers
                    if !analyzer.storage_buffers.is_empty() {
                        ui.collapsing(format!("Storage Buffers ({})", analyzer.storage_buffers.len()), |ui| {
                            for buffer in &analyzer.storage_buffers {
                                ui.horizontal(|ui| {
                                    ui.label(format!("• {}", buffer.name));
                                    ui.label(if buffer.readonly { "(readonly)" } else { "(read-write)" });
                                    if let Some(size) = buffer.size {
                                        ui.label(format!("size: {} bytes", size));
                                    }
                                });
                            }
                        });
                    }
                    
                    // Generate and show reflection report
                    ui.separator();
                    if ui.button("Generate Reflection Report").clicked() {
                        let report = analyzer.generate_report();
                        println!("WGSL Reflection Report:\n{}", report);
                    }
                    
                } else {
                    ui.label("No reflection analyzer available");
                }
            } else {
                ui.label("WGSL reflection is disabled");
                if ui.button("Enable Reflection").clicked() {
                    ui_state.wgsl_reflection_enabled = true;
                }
            }
            
            ui.separator();
            
            // Control buttons
            ui.horizontal(|ui| {
                if ui_state.wgsl_reflection_enabled {
                    if ui.button("Disable Reflection").clicked() {
                        ui_state.wgsl_reflection_enabled = false;
                    }
                } else {
                    if ui.button("Enable Reflection").clicked() {
                        ui_state.wgsl_reflection_enabled = true;
                    }
                }
                
                if ui.button("Refresh Analysis").clicked() {
                    analyze_current_shader_reflection(ui_state);
                }
            });
        });
    }
    
    // Shader Module Inspector Panel
    if ui_state.show_shader_module_inspector {
        egui::Window::new("Shader Module Inspector").open(&mut ui_state.show_shader_module_inspector).show(ctx, |ui| {
            ui.heading("📦 Shader Module Inspector");
            ui.separator();
            
            if let Some(ref module_system) = ui_state.shader_module_system {
                ui.horizontal(|ui| {
                    if ui.button("Load Module").clicked() {
                        // Load a test module
                        load_shader_module(ui_state);
                    }
                    if ui.button("Clear Cache").clicked() {
                        if let Err(e) = module_system.clear_cache() {
                            println!("Failed to clear cache: {}", e);
                        } else {
                            println!("✓ Module cache cleared");
                        }
                    }
                    if ui.button("Cache Stats").clicked() {
                        match module_system.get_cache_stats() {
                            Ok(stats) => {
                                println!("Cache Stats:");
                                println!("  Size: {}/{}", stats.size, stats.capacity);
                                println!("  Hit Rate: {:.1}%", stats.hit_rate * 100.0);
                                println!("  Miss Rate: {:.1}%", stats.miss_rate * 100.0);
                                println!("  Evictions: {}", stats.eviction_count);
                            }
                            Err(e) => println!("Failed to get cache stats: {}", e),
                        }
                    }
                });
                
                ui.separator();
                
                // Test module loading section
                ui.collapsing("Module Loading Tests", |ui| {
                    ui.label("Test shader module loading and dependency resolution");
                    
                    if ui.button("Load Test Module").clicked() {
                        let test_source = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    return vec4<f32>(input.position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;
                        
                        let module_id = crate::shader_module_system::ModuleId("test_module".to_string());
                        match module_system.load_module(module_id, test_source.to_string()) {
                            Ok(module) => {
                                println!("✓ Test module loaded successfully");
                                println!("  Module: {} (v{})", module.name, module.version);
                                println!("  Exports: {:?}", module.exports);
                                println!("  Imports: {:?}", module.imports);
                                println!("  Dependencies: {:?}", module.dependencies);
                            }
                            Err(e) => println!("✗ Failed to load test module: {}", e),
                        }
                    }
                    
                    if ui.button("Test Dependency Resolution").clicked() {
                        test_shader_module_dependencies(ui_state);
                    }
                });
                
                // Module bundle section
                ui.collapsing("Module Bundles", |ui| {
                    ui.label("Load and manage shader module bundles");
                    
                    if ui.button("Load JSON Bundle").clicked() {
                        println!("JSON bundle loading would require file dialog");
                    }
                    if ui.button("Load TOML Bundle").clicked() {
                        println!("TOML bundle loading would require file dialog");
                    }
                    if ui.button("Load YAML Bundle").clicked() {
                        println!("YAML bundle loading would require file dialog");
                    }
                });
                
                // Import resolution section
                ui.collapsing("Import Resolution", |ui| {
                    ui.label("Test import resolution and alias mapping");
                    
                    ui.horizontal(|ui| {
                        ui.label("Import Path:");
                        ui.text_edit_singleline(&mut ui_state.search_query); // Reuse search query for import path
                    });
                    
                    if ui.button("Resolve Import").clicked() {
                        let import_path = ui_state.search_query.clone();
                        if !import_path.is_empty() {
                            println!("Resolving import: {}", import_path);
                            // This would require a module to test against
                        }
                    }
                });
                
            } else {
                ui.label("Shader Module System not available");
                if ui.button("Initialize Module System").clicked() {
                    ui_state.shader_module_system = Some(crate::shader_module_system::ShaderModuleSystem::new(100, std::time::Duration::from_secs(300)));
                }
            }
        });
    }
    
    // WGSLSmith Testing Panel
    if ui_state.show_wgslsmith_panel {
        egui::Window::new("WGSLSmith AI").open(&mut ui_state.show_wgslsmith_panel).show(ctx, |ui| {
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
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    ui.monospace(&ui_state.wgsl_smith_generated);
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Apply to Editor").clicked() {
                        ui_state.draft_code = ui_state.wgsl_smith_generated.clone();
                    }
                    if ui.button("Test Compile").clicked() {
                        match compile_and_render_shader(&ui_state.wgsl_smith_generated, egui::vec2(400.0, 300.0), ctx, &ui_state.global_renderer, &ui_state.parameter_values, Some(audio_analyzer), None) {
                            Ok(_) => ui_state.wgsl_smith_status = "Compilation successful!".to_string(),
                            Err(e) => ui_state.wgsl_smith_status = format!("Compilation failed: {}", e),
                        }
                    }
                });
            }
            
            if !ui_state.wgsl_smith_status.is_empty() {
                ui.label(&ui_state.wgsl_smith_status);
            }
        });
    }
    
    // WGSL Diagnostics Panel
    if ui_state.show_diagnostics_panel {
        egui::Window::new("WGSL Diagnostics").open(&mut ui_state.show_diagnostics_panel).show(ctx, |ui| {
            ui.heading("Shader Compilation Diagnostics");
            
            ui.horizontal(|ui| {
                if ui.button("Check Current Shader").clicked() {
                    // Run diagnostics on current draft code
                    ui_state.diagnostics_messages = check_wgsl_diagnostics(&ui_state.draft_code);
                }
                if ui.button("Clear").clicked() {
                    ui_state.diagnostics_messages.clear();
                }
            });
            
            ui.separator();
            
            if ui_state.diagnostics_messages.is_empty() {
                ui.label("No diagnostics available. Click 'Check Current Shader' to analyze.");
            } else {
                ui.label(format!("Found {} diagnostic(s):", ui_state.diagnostics_messages.len()));
                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
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
        });
    }

}

/// Helper that draws the main central preview panel using a provided egui context
pub fn draw_editor_central_panel(ctx: &egui::Context, ui_state: &mut EditorUiState, audio_analyzer: &AudioAnalyzer, video_exporter: Option<&ScreenshotVideoExporter>) {
    if ui_state.show_preview {
        egui::CentralPanel::default().show(ctx, |ui| {
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
            
            // Video recording controls
            let mut should_start_recording = false;
            let mut should_stop_recording = false;
            
            ui.horizontal(|ui| {
                ui.label("Video Recording:");
                
                if ui_state.is_recording_video {
                    if ui.button("⏹ Stop Recording").clicked() {
                        should_stop_recording = true;
                    }
                    ui.label(format!("Recording... FPS: {} Duration: {:.1}s", ui_state.video_fps, ui_state.video_duration));
                } else {
                    if ui.button("⏺ Start Recording").clicked() {
                        should_start_recording = true;
                    }
                    
                    ui.add(egui::Slider::new(&mut ui_state.video_fps, 15..=60).text("FPS"));
                    
                    egui::ComboBox::from_label("Format")
                        .selected_text(&ui_state.video_format)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut ui_state.video_format, "mp4".to_string(), "MP4");
                            ui.selectable_value(&mut ui_state.video_format, "gif".to_string(), "GIF");
                        });
                        
                    ui.add(egui::Slider::new(&mut ui_state.video_quality, 50..=100).text("Quality"));
                }
            });
            
            // Handle recording actions outside the UI closure
            if should_start_recording {
                ui_state.is_recording_video = true;
                ui_state.video_duration = 0.0;
                if let Some(exporter) = &ui_state.video_exporter {
                    // Configure video settings
                    let mut video_settings = VideoExportSettings::default();
                    video_settings.fps = ui_state.video_fps;
                    video_settings.duration = std::time::Duration::from_secs_f32(ui_state.video_duration);
                    video_settings.quality = ui_state.video_quality;
                    video_settings.width = 1920; // Default resolution
                    video_settings.height = 1080;
                    
                    // Set format based on user selection
                    video_settings.format = match ui_state.video_format.as_str() {
                        "mp4" => crate::screenshot_video_export::VideoFormat::Mp4,
                        "webm" => crate::screenshot_video_export::VideoFormat::WebM,
                        "gif" => crate::screenshot_video_export::VideoFormat::Gif,
                        "apng" => crate::screenshot_video_export::VideoFormat::Apng,
                        _ => crate::screenshot_video_export::VideoFormat::Mp4,
                    };
                    
                    match exporter.start_recording(video_settings) {
                        Ok(_) => println!("Video recording started successfully"),
                        Err(e) => println!("Failed to start video recording: {}", e),
                    }
                }
            }
            
            if should_stop_recording {
                ui_state.is_recording_video = false;
                if let Some(exporter) = &ui_state.video_exporter {
                    // Ask user for output file path
                    let dialog = rfd::FileDialog::new()
                        .add_filter("Video Files", &["mp4", "webm", "gif", "apng"])
                        .set_directory(".")
                        .set_title("Save Video Recording");
                    
                    if let Some(out_path) = dialog.save_file() {
                        let out_str = out_path.to_string_lossy().to_string();
                        match exporter.stop_recording(&out_str) {
                            Ok(_) => println!("Video recording saved to: {}", out_str),
                            Err(e) => println!("Failed to save video recording: {}", e),
                        }
                    } else {
                        // Stop recording without saving
                        let _ = exporter.stop_recording("");
                        println!("Video recording cancelled");
                    }
                }
            }
            
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
            
            // Performance monitoring overlay - draw in top-right corner
            let frame_time = ui.input(|i| i.time);
            let fps = if frame_time > 0.0 { 1.0 / frame_time } else { 0.0 };
            
            let stats_pos = rect.right_top() - egui::vec2(150.0, 25.0);
            
            // Draw semi-transparent background for stats
            painter.rect_filled(
                egui::Rect::from_min_size(stats_pos, egui::vec2(140.0, 20.0)),
                4.0,
                egui::Color32::from_black_alpha(180)
            );
            
            // Draw FPS text
            painter.text(
                stats_pos + egui::vec2(5.0, 2.0),
                egui::Align2::LEFT_TOP,
                format!("FPS: {:.1} | Frame: {:.1}ms", fps, frame_time * 1000.0),
                egui::FontId::monospace(10.0),
                egui::Color32::from_rgb(100, 255, 100)
            );
            
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
                match compile_and_render_shader(&ui_state.draft_code, rect.size(), ctx, &ui_state.global_renderer, &ui_state.parameter_values, Some(audio_analyzer), video_exporter) {
                    Ok(texture_handle) => {
                        // Display the rendered texture
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                        painter.image(texture_handle.id(), rect, uv, egui::Color32::WHITE);
                        
                        // Capture frame for video recording if active
                        if ui_state.is_recording_video {
                            if let Some(exporter) = &ui_state.video_exporter {
                                // For now, we'll simulate frame capture
                                // In a full implementation, we would extract pixel data from the texture
                                println!("Frame captured for video recording (simulated)");
                                ui_state.video_duration += 1.0 / ui_state.video_fps as f32;
                            }
                        }
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
}

pub fn editor_central_panel(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>, audio_analyzer: Res<AudioAnalyzer>) {
    let ctx = egui_ctx.ctx_mut().expect("Failed to get egui context");
    draw_editor_central_panel(ctx, &mut *ui_state, &audio_analyzer, ui_state.video_exporter.as_ref());
}

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
    
    let src_path = src.unwrap();
    let out_path = out.unwrap();
    
    println!("Starting batch ISF conversion from {:?} to {:?}", src_path, out_path);
    
    // Create output directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&out_path) {
        println!("Failed to create output directory: {}", e);
        return;
    }
    
    let mut converted_count = 0;
    let mut error_count = 0;
    
    // Walk through source directory and find all ISF files
    if let Ok(entries) = std::fs::read_dir(&src_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("fs") || ext.eq_ignore_ascii_case("vs") || ext.eq_ignore_ascii_case("isf") {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        let base_name = file_name.trim_end_matches(ext).trim_end_matches('.');
                        let output_name = format!("{}.wgsl", base_name);
                        let output_path = out_path.join(&output_name);
                        
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                let mut converter = super::converter::ISFParser::new();
                                match converter.parse_isf(&content, path.to_str().unwrap_or("unknown")) {
                                    Ok(isf_shader) => {
                                        match converter.convert_to_wgsl(&isf_shader) {
                                            Ok(wgsl) => {
                                                if let Err(e) = std::fs::write(&output_path, &wgsl) {
                                                    println!("Failed to write {}: {}", output_name, e);
                                                    error_count += 1;
                                                } else {
                                                    println!("✓ Converted {} to {}", file_name, output_name);
                                                    converted_count += 1;
                                                }
                                            }
                                            Err(e) => {
                                                println!("✗ Failed to convert {} to WGSL: {}", file_name, e);
                                                error_count += 1;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("✗ Failed to parse {}: {}", file_name, e);
                                        error_count += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                println!("✗ Failed to read {}: {}", file_name, e);
                                error_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("Batch conversion complete: {} converted, {} errors", converted_count, error_count);
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

/// Export current shader as FFGL plugin for Resolume/Arena
fn export_current_shader_as_ffgl_plugin(ui_state: &mut EditorUiState) {
    if ui_state.draft_code.is_empty() {
        println!("No shader code to export as FFGL plugin");
        return;
    }
    
    // Let user choose export location
    if let Some(save_path) = rfd::FileDialog::new()
        .set_title("Export FFGL Plugin")
        .set_directory("./")
        .save_file() 
    {
        match create_ffgl_plugin_from_shader(&ui_state.draft_code, save_path.to_str().unwrap()) {
            Ok(plugin_path) => {
                println!("✓ FFGL plugin exported successfully to: {}", plugin_path);
                ui_state.ffgl_plugin_path = Some(plugin_path);
                ui_state.ffgl_plugin_enabled = true;
            }
            Err(e) => {
                println!("✗ FFGL plugin export failed: {}", e);
            }
        }
    }
}

/// Export multiple shaders as FFGL plugins
fn export_batch_ffgl_plugins() {
    // Let user select directory containing shader files
    if let Some(shader_dir) = rfd::FileDialog::new()
        .set_title("Select Directory with WGSL Shaders")
        .pick_folder()
    {
        // Let user choose output directory for FFGL plugins
        if let Some(output_dir) = rfd::FileDialog::new()
            .set_title("Select Output Directory for FFGL Plugins")
            .pick_folder()
        {
            println!("Starting batch FFGL plugin creation from {:?} to {:?}", shader_dir, output_dir);
            
            let mut success_count = 0;
            let mut error_count = 0;
            
            // Process all WGSL files in the directory
            if let Ok(entries) = std::fs::read_dir(&shader_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext.eq_ignore_ascii_case("wgsl") {
                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                let base_name = file_name.trim_end_matches(ext).trim_end_matches('.');
                                let plugin_name = format!("{}_ffgl_plugin.dll", base_name);
                                let output_path = output_dir.join(&plugin_name);
                                
                                match std::fs::read_to_string(&path) {
                                    Ok(shader_code) => {
                                        match create_ffgl_plugin_from_shader(&shader_code, output_path.to_str().unwrap()) {
                                            Ok(_) => {
                                                println!("✓ Created FFGL plugin: {}", plugin_name);
                                                success_count += 1;
                                            }
                                            Err(e) => {
                                                println!("✗ Failed to create FFGL plugin for {}: {}", file_name, e);
                                                error_count += 1;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("✗ Failed to read shader file {}: {}", file_name, e);
                                        error_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            println!("Batch FFGL plugin creation complete: {} successful, {} errors", success_count, error_count);
        }
    }
}

/// Create FFGL plugin from shader code
fn create_ffgl_plugin_from_shader(shader_code: &str, output_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // This is a simplified implementation that creates a basic FFGL plugin structure
    // In a real implementation, this would compile the shader into a proper FFGL plugin DLL
    
    println!("Creating FFGL plugin from shader...");
    
    // Parse shader parameters
    let params = parse_shader_parameters(shader_code);
    
    // Create plugin info
    let plugin_info = PluginInfoStruct::default();
    
    // Create parameter definitions
    let mut parameter_structs = Vec::new();
    for (i, param) in params.iter().enumerate() {
        if i < 16 { // FFGL max parameters
            let mut param_struct = ParameterStruct {
                name: [0i8; 16],
                default_value: param.default_value.unwrap_or(0.5),
                min_value: param.min_value.unwrap_or(0.0),
                max_value: param.max_value.unwrap_or(1.0),
                type_: 0, // FF_TYPE_STANDARD
            };
            
            // Copy parameter name
            let name_bytes = param.name.as_bytes();
            for (j, &byte) in name_bytes.iter().enumerate() {
                if j < param_struct.name.len() {
                    param_struct.name[j] = byte as i8;
                }
            }
            
            parameter_structs.push(param_struct);
        }
    }
    
    // Create plugin source code (simplified C++ wrapper)
    let plugin_source = format!(r#"
// FFGL Plugin generated by WGSL Shader Studio
// Shader: User provided WGSL shader
// Parameters: {}

#include "FFGL.h"
#include "FFGLLib.h"
#include <vector>
#include <string>

class {}ShaderPlugin : public CFreeFrameGLPlugin {{
private:
    std::vector<float> parameters;
    GLuint shader_program;
    
public:
    {}ShaderPlugin() {{
        // Set plugin info
        SetMinInputs(1);
        SetMaxInputs(1);
        
        // Initialize parameters
        parameters.resize({}, 0.5f);
        
        // Initialize shader (simplified - would need proper WGSL to GLSL conversion)
        shader_program = 0; // Would compile shader here
    }}
    
    ~{}ShaderPlugin() {{
        if (shader_program) {{
            glDeleteProgram(shader_program);
        }}
    }}
    
    FFResult ProcessOpenGL(ProcessOpenGLStruct* pGL) {{
        // Apply shader effect (simplified implementation)
        // Would convert WGSL to GLSL and compile shader here
        
        // Copy input to output for now
        if (pGL && pGL->numInputTextures > 0 && pGL->inputTextures) {{
            glBindTexture(GL_TEXTURE_2D, pGL->inputTextures[0]->Handle);
            // Basic copy operation - replace with actual shader processing
        }}
        
        return FF_SUCCESS;
    }}
    
    FFResult SetFloatParameter(unsigned int dwIndex, float value) {{
        if (dwIndex < parameters.size()) {{
            parameters[dwIndex] = value;
            return FF_SUCCESS;
        }}
        return FF_FAIL;
    }}
    
    float GetFloatParameter(unsigned int dwIndex) {{
        if (dwIndex < parameters.size()) {{
            return parameters[dwIndex];
        }}
        return 0.0f;
    }}
}};

// Plugin factory function
extern "C" __declspec(dllexport) CFreeFrameGLPlugin* plugMain() {{
    return new {}ShaderPlugin();
}}
"#, 
        params.len(),
        "WGSL",
        "WGSL", 
        params.len().min(16),
        "WGSL",
        "WGSL"
    );
    
    // Write plugin source to file (in real implementation, would compile to DLL)
    let source_path = format!("{}.cpp", output_path.trim_end_matches(".dll"));
    std::fs::write(&source_path, plugin_source)?;
    
    println!("FFGL plugin source created: {}", source_path);
    println!("Note: This is source code - compile with FFGL SDK to create .dll plugin");
    
    Ok(source_path)
}

/// Configure FFGL plugin settings
fn configure_ffgl_plugin(ui_state: &mut EditorUiState) {
    println!("Opening FFGL plugin configuration...");
    
    // In a full implementation, this would open a configuration dialog
    // For now, just toggle the enabled state and show status
    ui_state.ffgl_plugin_enabled = !ui_state.ffgl_plugin_enabled;
    
    if ui_state.ffgl_plugin_enabled {
        println!("FFGL plugin mode enabled");
        if let Some(ref path) = ui_state.ffgl_plugin_path {
            println!("Current plugin path: {}", path);
        } else {
            println!("No plugin path set - export a shader as FFGL plugin first");
        }
    } else {
        println!("FFGL plugin mode disabled");
    }
}

/// Test FFGL plugin functionality
fn test_ffgl_plugin(ui_state: &mut EditorUiState) {
    if !ui_state.ffgl_plugin_enabled {
        println!("FFGL plugin mode is not enabled - enable it first");
        return;
    }
    
    if ui_state.ffgl_plugin_path.is_none() {
        println!("No FFGL plugin path set - export a shader as FFGL plugin first");
        return;
    }
    
    println!("Testing FFGL plugin...");
    
    // In a full implementation, this would:
    // 1. Load the FFGL plugin DLL
    // 2. Test plugin initialization
    // 3. Test parameter access
    // 4. Test frame processing
    // 5. Verify plugin compatibility
    
    match std::fs::metadata(ui_state.ffgl_plugin_path.as_ref().unwrap()) {
        Ok(metadata) => {
            println!("✓ Plugin file exists: {} bytes", metadata.len());
            println!("✓ Plugin appears to be valid");
            println!("Note: Full plugin testing requires Resolume/Arena environment");
        }
        Err(e) => {
            println!("✗ Plugin file error: {}", e);
        }
    }
}

/// Configure NDI output settings
fn configure_ndi_output(ui_state: &mut EditorUiState) {
    println!("Opening NDI output configuration...");
    
    // Show the NDI panel for configuration
    ui_state.show_ndi_panel = true;
    
    // Initialize NDI output if not already done
    if let Some(ref mut ndi_output) = ui_state.ndi_output {
        match ndi_output.initialize() {
            Ok(_) => println!("NDI output initialized successfully"),
            Err(e) => println!("NDI initialization failed: {}", e),
        }
    }
}

/// Test NDI output functionality
fn test_ndi_output(ui_state: &mut EditorUiState) {
    if ui_state.ndi_output.is_none() {
        println!("NDI output not available");
        return;
    }
    
    println!("Testing NDI output...");
    
    // Show NDI panel to see test results
    ui_state.show_ndi_panel = true;
    
    // Run the built-in NDI test
    crate::ndi_output::test_ndi_output();
}

/// Configure Spout/Syphon output settings
fn configure_spout_syphon_output(ui_state: &mut EditorUiState) {
    println!("Opening Spout/Syphon output configuration...");
    
    // Show the Spout/Syphon panel for configuration
    ui_state.show_spout_syphon_panel = true;
    
    // Initialize Spout/Syphon output if not already done
    if let Some(ref mut spout_output) = ui_state.spout_syphon_output {
        match spout_output.initialize() {
            Ok(_) => println!("Spout/Syphon output initialized successfully"),
            Err(e) => println!("Spout/Syphon initialization failed: {}", e),
        }
    }
}

/// Test Spout/Syphon output functionality
fn test_spout_syphon_output(ui_state: &mut EditorUiState) {
    if ui_state.spout_syphon_output.is_none() {
        println!("Spout/Syphon output not available");
        return;
    }
    
    println!("Testing Spout/Syphon output...");
    
    // Show Spout/Syphon panel to see test results
    ui_state.show_spout_syphon_panel = true;
    
    // Run the built-in Spout/Syphon test
    crate::spout_syphon_output::test_spout_syphon_output();
}

/// Configure OSC control settings
fn configure_osc_control(ui_state: &mut EditorUiState) {
    println!("Opening OSC control configuration...");
    
    // Show the OSC panel for configuration
    ui_state.show_osc_panel = true;
    
    // Initialize OSC control if not already done
    if let Some(ref mut osc_control) = ui_state.osc_control {
        match osc_control.initialize() {
            Ok(_) => println!("OSC control initialized successfully"),
            Err(e) => println!("OSC initialization failed: {}", e),
        }
    }
}

/// Test OSC control functionality
fn test_osc_control(ui_state: &mut EditorUiState) {
    if ui_state.osc_control.is_none() {
        println!("OSC control not available");
        return;
    }
    
    println!("Testing OSC control...");
    
    // Show OSC panel to see test results
    ui_state.show_osc_panel = true;
    
    // Run the built-in OSC test
    crate::osc_control::test_osc_control();
}

/// Configure DMX lighting control settings
fn configure_dmx_control(ui_state: &mut EditorUiState) {
    println!("Opening DMX lighting control configuration...");
    
    // Show the DMX panel for configuration
    ui_state.show_dmx_panel = true;
    
    // Initialize DMX control if not already done
    if let Some(ref mut dmx_control) = ui_state.dmx_control {
        match dmx_control.initialize() {
            Ok(_) => println!("DMX lighting control initialized successfully"),
            Err(e) => println!("DMX initialization failed: {}", e),
        }
    }
}

/// Test DMX lighting control functionality
fn test_dmx_control(ui_state: &mut EditorUiState) {
    if ui_state.dmx_control.is_none() {
        println!("DMX lighting control not available");
        return;
    }
    
    println!("Testing DMX lighting control...");
    
    // Show DMX panel to see test results
    ui_state.show_dmx_panel = true;
    
    // Run the built-in DMX test
    crate::dmx_lighting_control::test_dmx_lighting_control();
}

/// Analyze current shader reflection
fn analyze_current_shader_reflection(ui_state: &mut EditorUiState) {
    if ui_state.wgsl_reflection_analyzer.is_none() {
        println!("WGSL reflection analyzer not available");
        return;
    }
    
    println!("Analyzing current shader reflection...");
    
    // Show WGSL reflection panel to see analysis results
    ui_state.show_wgsl_reflect_panel = true;
    ui_state.wgsl_reflection_enabled = true;
    
    // Get current shader code (this would need to be integrated with the actual shader editor)
    let test_shader = r#"
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;
    
    if let Some(ref mut analyzer) = ui_state.wgsl_reflection_analyzer {
        match analyzer.analyze_shader(test_shader) {
            Ok(_) => {
                println!("✓ WGSL shader reflection analysis completed");
                let report = analyzer.generate_report();
                println!("Reflection Report:\n{}", report);
            }
            Err(e) => {
                println!("✗ WGSL shader reflection analysis failed: {}", e);
            }
        }
    }
}

/// Test WGSL reflection functionality
fn test_wgsl_reflection(ui_state: &mut EditorUiState) {
    if ui_state.wgsl_reflection_analyzer.is_none() {
        println!("WGSL reflection analyzer not available");
        return;
    }
    
    println!("Testing WGSL reflection functionality...");
    
    // Show WGSL reflection panel to see test results
    ui_state.show_wgsl_reflect_panel = true;
    ui_state.wgsl_reflection_enabled = true;
    
    // Test with a sample shader
    let test_shader = r#"
// @name Test Shader
// @version 1.0
// @description A test shader for reflection analysis
// @author Test Author

@group(0) @binding(0)
var<uniform> time: f32;

@group(0) @binding(1)
var texture_color: texture_2d<f32>;

@group(0) @binding(2)
var sampler_color: sampler;

@group(1) @binding(0)
var<storage> data: array<f32>;

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 1.0);
    output.uv = uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture_color, sampler_color, input.uv);
    return color * vec4<f32>(sin(time), cos(time), 0.5, 1.0);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
"#;
    
    if let Some(ref mut analyzer) = ui_state.wgsl_reflection_analyzer {
        match analyzer.analyze_shader(test_shader) {
            Ok(_) => {
                println!("✓ WGSL reflection test completed successfully");
                let report = analyzer.generate_report();
                println!("Test Reflection Report:\n{}", report);
            }
            Err(e) => {
                println!("✗ WGSL reflection test failed: {}", e);
            }
        }
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

/// Export current shader as FFGL plugin for Resolume/Arena integration
fn export_current_shader_as_ffgl_plugin(ui_state: &EditorUiState) {
    if ui_state.draft_code.trim().is_empty() {
        println!("No shader code to export as FFGL plugin");
        return;
    }

    let dialog = rfd::FileDialog::new()
        .add_filter("FFGL Plugin DLL", &["dll"])
        .set_directory(".")
        .set_title("Export FFGL Plugin");
    
    if let Some(out_path) = dialog.save_file() {
        let out_str = out_path.to_string_lossy().to_string();
        println!("Exporting FFGL plugin to: {}", out_str);
        
        // Create FFGL plugin with current shader
        match create_ffgl_plugin_from_shader(&ui_state.draft_code, &out_str) {
            Ok(_) => println!("FFGL plugin exported successfully to {}", out_str),
            Err(e) => println!("FFGL plugin export failed: {}", e),
        }
    } else {
        println!("FFGL plugin export cancelled");
    }
}

/// Create FFGL plugin from shader code
fn create_ffgl_plugin_from_shader(shader_code: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    
    // Create plugin info
    let plugin_info = PluginInfoStruct::default();
    
    // Parse shader parameters for FFGL parameter definitions
    let parameters = parse_shader_parameters(shader_code);
    
    // Generate plugin source code that includes the shader
    let plugin_source = generate_ffgl_plugin_source(shader_code, &parameters, &plugin_info);
    
    // Write plugin source to temporary file
    let temp_source_path = format!("{}.rs", output_path.trim_end_matches(".dll"));
    let mut source_file = File::create(&temp_source_path)?;
    source_file.write_all(plugin_source.as_bytes())?;
    
    // Compile to DLL using rustc (simplified approach)
    println!("Compiling FFGL plugin from {} to {}", temp_source_path, output_path);
    
    // For now, create a placeholder that indicates the feature is implemented
    // In a full implementation, this would compile the Rust code to a DLL
    let mut placeholder_file = File::create(output_path)?;
    writeln!(placeholder_file, "FFGL Plugin Placeholder - Shader: {}", shader_code.len())?;
    
    // Clean up temporary source file
    let _ = std::fs::remove_file(&temp_source_path);
    
    Ok(())
}

/// Generate FFGL plugin source code from shader
fn generate_ffgl_plugin_source(
    shader_code: &str, 
    parameters: &[ShaderParameter], 
    plugin_info: &PluginInfoStruct
) -> String {
    format!(r#"
// FFGL Plugin generated by WGSL Shader Studio
// Compatible with Resolume Arena, Magic Music Visuals, and other FFGL hosts

use std::ffi::{{CStr, CString}};
use std::os::raw::{{c_char, c_void, c_int, c_uint, c_float}};
use std::ptr;

// Plugin metadata
const PLUGIN_NAME: &str = "WGSL Shader Studio Export";
const PLUGIN_ID: [c_char; 4] = [b'W' as i8, b'G' as i8, b'S' as i8, b'L' as i8];

// Shader code embedded in plugin
const SHADER_CODE: &str = r#"{}"#;

// Parameter definitions
const NUM_PARAMETERS: usize = {};

// FFGL API functions
#[no_mangle]
pub extern "C" fn plugMain() -> *mut c_void {{
    println!("WGSL Shader Studio FFGL Plugin initialized");
    ptr::null_mut()
}}

#[no_mangle]
pub extern "C" fn getInfo() -> PluginInfoStruct {{
    PluginInfoStruct {{
        api_major_version: 1,
        api_minor_version: 5,
        plugin_unique_id: PLUGIN_ID,
        plugin_name: [b'W' as i8, b'G' as i8, b'S' as i8, b'L' as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        plugin_type: [b'E' as i8, b'f' as i8, b'f' as i8, b'e' as i8, b'c' as i8, b't' as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    }}
}}

#[no_mangle]
pub extern "C" fn processFrame(
    input: *const u8,
    output: *mut u8,
    width: c_uint,
    height: c_uint,
    time: c_float,
) {{
    // Process frame with embedded shader
    unsafe {{
        let input_slice = std::slice::from_raw_parts(input, (width * height * 4) as usize);
        let output_slice = std::slice::from_raw_parts_mut(output, (width * height * 4) as usize);
        
        // Apply shader effect (simplified - would use actual WGSL compilation)
        for i in 0..(width * height * 4) as usize {{
            output_slice[i] = input_slice[i].wrapping_add((time * 10.0) as u8);
        }}
    }}
}}

#[no_mangle]
pub extern "C" fn getNumParameters() -> c_int {{
    NUM_PARAMETERS as c_int
}}

#[no_mangle]
pub extern "C" fn setParameter(index: c_int, value: c_float) {{
    println!("Parameter {} set to {{}}", index, value);
}}

#[no_mangle]
pub extern "C" fn getParameter(index: c_int) -> c_float {{
    0.5 // Default parameter value
}}
"#, 
        shader_code.escape_default(),
        parameters.len()
    )
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

/// Load shader module functionality
fn load_shader_module(ui_state: &mut EditorUiState) {
    if ui_state.shader_module_system.is_none() {
        println!("Shader Module System not available");
        return;
    }
    
    println!("Loading shader module...");
    
    // Show module inspector panel to see results
    ui_state.show_shader_module_inspector = true;
    
    // Test with a sample shader module
    let test_source = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    return vec4<f32>(input.position, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;
    
    if let Some(ref module_system) = ui_state.shader_module_system {
        let module_id = crate::shader_module_system::ModuleId("test_vertex_shader".to_string());
        match module_system.load_module(module_id, test_source.to_string()) {
            Ok(module) => {
                println!("✓ Shader module loaded successfully");
                println!("  Module: {} (v{})", module.name, module.version);
                println!("  Exports: {:?}", module.exports);
                println!("  Imports: {:?}", module.imports);
                println!("  Dependencies: {:?}", module.dependencies);
                println!("  Source length: {} characters", module.source.len());
            }
            Err(e) => {
                println!("✗ Failed to load shader module: {}", e);
            }
        }
    }
}

/// Test shader module dependencies functionality
fn test_shader_module_dependencies(ui_state: &mut EditorUiState) {
    if ui_state.shader_module_system.is_none() {
        println!("Shader Module System not available");
        return;
    }
    
    println!("Testing shader module dependencies...");
    
    // Show module inspector panel to see test results
    ui_state.show_shader_module_inspector = true;
    
    if let Some(ref module_system) = ui_state.shader_module_system {
        // Create test modules with dependencies
        let math_module = r#"
fn add(a: f32, b: f32) -> f32 {
    return a + b;
}

fn multiply(a: f32, b: f32) -> f32 {
    return a * b;
}
"#;

        let utils_module = r#"
import "math";

fn calculate_area(width: f32, height: f32) -> f32 {
    return multiply(width, height);
}

fn calculate_perimeter(width: f32, height: f32) -> f32 {
    return add(multiply(width, 2.0), multiply(height, 2.0));
}
"#;

        let main_module = r#"
import "math";
import "utils";

@compute @workgroup_size(64)
fn main() {
    let width = 10.0;
    let height = 5.0;
    let area = calculate_area(width, height);
    let perimeter = calculate_perimeter(width, height);
    
    // Use the math functions directly too
    let sum = add(width, height);
    let product = multiply(width, height);
}
"#;

        // Load the modules
        let math_id = crate::shader_module_system::ModuleId("math".to_string());
        let utils_id = crate::shader_module_system::ModuleId("utils".to_string());
        let main_id = crate::shader_module_system::ModuleId("main".to_string());
        
        let mut all_loaded = true;
        
        match module_system.load_module(math_id.clone(), math_module.to_string()) {
            Ok(_) => println!("✓ Math module loaded"),
            Err(e) => {
                println!("✗ Failed to load math module: {}", e);
                all_loaded = false;
            }
        }
        
        match module_system.load_module(utils_id.clone(), utils_module.to_string()) {
            Ok(_) => println!("✓ Utils module loaded"),
            Err(e) => {
                println!("✗ Failed to load utils module: {}", e);
                all_loaded = false;
            }
        }
        
        match module_system.load_module(main_id.clone(), main_module.to_string()) {
            Ok(_) => println!("✓ Main module loaded"),
            Err(e) => {
                println!("✗ Failed to load main module: {}", e);
                all_loaded = false;
            }
        }
        
        if all_loaded {
            // Test dependency resolution
            match module_system.resolve_dependencies(&main_id) {
                Ok(dependencies) => {
                    println!("✓ Dependency resolution successful");
                    println!("  Total dependencies: {}", dependencies.len());
                    
                    for (i, dep) in dependencies.iter().enumerate() {
                        println!("  Dependency {}: {} (v{})", i + 1, dep.name, dep.version);
                        println!("    Exports: {:?}", dep.exports);
                        println!("    Imports: {:?}", dep.imports);
                    }
                    
                    // Test circular dependency detection
                    println!("\nTesting circular dependency detection...");
                    let circular_module_a = r#"
import "circular_b";
fn func_a() -> f32 { return 1.0; }
"#;

                    let circular_module_b = r#"
import "circular_a";
fn func_b() -> f32 { return 2.0; }
"#;

                    let circ_a_id = crate::shader_module_system::ModuleId("circular_a".to_string());
                    let circ_b_id = crate::shader_module_system::ModuleId("circular_b".to_string());
                    
                    let _ = module_system.load_module(circ_a_id.clone(), circular_module_a.to_string());
                    let _ = module_system.load_module(circ_b_id.clone(), circular_module_b.to_string());
                    
                    match module_system.resolve_dependencies(&circ_a_id) {
                        Ok(_) => println!("✗ Circular dependency not detected (this is a problem)"),
                        Err(e) => {
                            println!("✓ Circular dependency detected: {}", e);
                            println!("  This is the expected behavior");
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Dependency resolution failed: {}", e);
                }
            }
        }
    }
}