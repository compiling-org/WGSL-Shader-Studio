use bevy::prelude::*;
// Removed unused imports
// use bevy::render::render_resource::{TextureFormat, Extent3d};
// use bevy::render::renderer::{RenderDevice, RenderQueue};
// use bevy::window::PrimaryWindow;
use bevy_egui::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::shader_renderer::{ShaderRenderer, RenderParameters};
use crate::audio_system::AudioAnalyzer;
// Removed unused imports
// use crate::timeline::Timeline;
// use crate::editor_ui::EditorUiState;

/// Resource that manages the WGPU shader rendering pipeline
#[derive(Resource)]
pub struct WgpuRenderPipeline {
    pub renderer: Arc<Mutex<Option<ShaderRenderer>>>,
    pub last_frame_time: Instant,
    pub frame_count: u64,
    pub render_texture: Option<bevy::render::render_resource::Texture>,
    pub preview_texture_id: Option<egui::TextureId>,
    pub last_render_result: Arc<Mutex<Option<Vec<u8>>>>,
    pub is_rendering: Arc<Mutex<bool>>,
    pub render_errors: Arc<Mutex<Vec<String>>>,
}

impl Default for WgpuRenderPipeline {
    fn default() -> Self {
        Self {
            renderer: Arc::new(Mutex::new(None)),
            last_frame_time: Instant::now(),
            frame_count: 0,
            render_texture: None,
            preview_texture_id: None,
            last_render_result: Arc::new(Mutex::new(None)),
            is_rendering: Arc::new(Mutex::new(false)),
            render_errors: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl WgpuRenderPipeline {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize the WGPU renderer asynchronously
    pub async fn initialize_renderer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Initializing WGPU renderer...");
        
        match ShaderRenderer::new().await {
            Ok(renderer) => {
                println!("✅ WGPU renderer initialized successfully!");
                *self.renderer.lock().unwrap() = Some(renderer);
                Ok(())
            }
            Err(e) => {
                println!("❌ Failed to initialize WGPU renderer: {}", e);
                Err(e)
            }
        }
    }
    
    /// Render a shader frame with the given code and parameters
    pub fn render_shader_frame(
        &mut self,
        shader_code: &str,
        width: u32,
        height: u32,
        time: f32,
        audio_data: Option<&crate::audio_system::AudioData>,
        parameter_values: Option<&[f32]>,
    ) -> Result<Vec<u8>, String> {
        // Check if already rendering to prevent concurrent access
        if *self.is_rendering.lock().unwrap() {
            return Err("Rendering already in progress".to_string());
        }
        
        *self.is_rendering.lock().unwrap() = true;
        
        let result = {
            let mut renderer_guard = self.renderer.lock().unwrap();
            if let Some(ref mut renderer) = *renderer_guard {
                let render_params = RenderParameters {
                    width,
                    height,
                    time,
                    frame_rate: 60.0,
                    audio_data: audio_data.cloned(),
                };
                
                match renderer.render_frame_with_params(shader_code, &render_params, parameter_values, audio_data.cloned()) {
                    Ok(pixels) => {
                        println!("✅ Shader rendered successfully: {}x{} pixels", width, height);
                        Ok(pixels)
                    }
                    Err(e) => {
                        let error_msg = format!("Shader compilation/rendering failed: {:?}", e);
                        println!("❌ {}", error_msg);
                        self.render_errors.lock().unwrap().push(error_msg.clone());
                        Err(error_msg)
                    }
                }
            } else {
                Err("WGPU renderer not initialized".to_string())
            }
        };
        
        *self.is_rendering.lock().unwrap() = false;
        result
    }
    
    /// Get the last rendered frame as RGBA pixels
    pub fn get_last_frame(&self) -> Option<Vec<u8>> {
        self.last_render_result.lock().unwrap().clone()
    }
    
    /// Get render errors
    pub fn get_errors(&self) -> Vec<String> {
        self.render_errors.lock().unwrap().clone()
    }
    
    /// Clear render errors
    pub fn clear_errors(&self) {
        self.render_errors.lock().unwrap().clear();
    }
    
    /// Check if renderer is available
    pub fn is_renderer_available(&self) -> bool {
        self.renderer.lock().unwrap().is_some()
    }
}

/// System to initialize the WGPU renderer
pub fn initialize_wgpu_system(
    mut wgpu_pipeline: ResMut<WgpuRenderPipeline>,
    time: Res<Time>,
) {
    // Only attempt initialization after a short delay to let the app stabilize
    if time.elapsed().as_millis() < 100 {
        return;
    }
    
    // Check if already initialized
    if wgpu_pipeline.is_renderer_available() {
        return;
    }
    
    // Attempt async initialization
    let mut pipeline = std::mem::take(&mut *wgpu_pipeline);
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(pipeline.initialize_renderer()) {
            Ok(_) => {
                println!("🎉 WGPU renderer initialization completed in background");
            }
            Err(e) => {
                println!("💥 WGPU renderer initialization failed: {}", e);
            }
        }
    });
}

/// System to handle live shader preview rendering
pub fn live_preview_system(
    mut wgpu_pipeline: ResMut<WgpuRenderPipeline>,
    shader_code: Res<crate::editor_ui::EditorUiState>,
    audio_analyzer: Res<crate::audio_system::AudioAnalyzer>,
    time: Res<Time>,
) {
    // Only render if we have a renderer and code
    if !wgpu_pipeline.is_renderer_available() || shader_code.code.trim().is_empty() {
        return;
    }
    
    // Limit rendering to reasonable frame rate (30 FPS)
    let frame_interval = Duration::from_millis(33); // ~30 FPS
    if wgpu_pipeline.last_frame_time.elapsed() < frame_interval {
        return;
    }
    
    // Update frame timing
    wgpu_pipeline.frame_count += 1;
    wgpu_pipeline.last_frame_time = Instant::now();
    
    // Prepare render parameters
    let width = 512u32;
    let height = 512u32;
    let current_time = time.elapsed_secs();
    
    // Get audio data if available
    let audio_data = if audio_analyzer.enabled {
        let audio_info = audio_analyzer.get_audio_data();
        Some(crate::audio_system::AudioData {
            volume: audio_info.volume,
            bass_level: audio_info.bass_level,
            mid_level: audio_info.mid_level,
            treble_level: audio_info.treble_level,
            beat_detected: audio_info.beat_detected,
            beat_intensity: audio_info.beat_intensity,
            tempo: audio_info.tempo,
            waveform: audio_info.waveform.clone(),
            frequencies: audio_info.frequencies.clone(),
        })
    } else {
        None
    };
    
    // Extract parameter values from timeline
    let mut param_values = vec![0.0f32; 64];
    // TODO: Integrate with timeline system for animated parameters
    
    // Render the shader
    match wgpu_pipeline.render_shader_frame(
        &shader_code.code,
        width,
        height,
        current_time,
        audio_data.as_ref(),
        Some(&param_values),
    ) {
        Ok(pixels) => {
            // Store the rendered frame
            *wgpu_pipeline.last_render_result.lock().unwrap() = Some(pixels);
            wgpu_pipeline.clear_errors();
        }
        Err(e) => {
            println!("Preview render error: {}", e);
            // Keep previous frame on error
        }
    }
}

/// System to handle shader compilation and preview updates
pub fn shader_compilation_system(
    mut wgpu_pipeline: ResMut<WgpuRenderPipeline>,
    mut editor_state: ResMut<crate::editor_ui::EditorUiState>,
    audio_analyzer: Res<crate::audio_system::AudioAnalyzer>,
    time: Res<Time>,
) {
    // Check if auto-compile is enabled and we have code
    if !editor_state.auto_apply || editor_state.code.trim().is_empty() {
        return;
    }
    
    // Check if renderer is available
    if !wgpu_pipeline.is_renderer_available() {
        editor_state.compilation_error = "WGPU renderer not available".to_string();
        return;
    }
    
    // Clear previous errors
    wgpu_pipeline.clear_errors();
    editor_state.compilation_error.clear();
    editor_state.diagnostics_messages.clear();
    
    // Run WGSL diagnostics
    // TODO: Implement proper diagnostics checking
    let diagnostics = vec![];
    // For now, we'll just clear any existing messages
    editor_state.diagnostics_messages.clear();
    
    // If there are critical errors, don't attempt compilation
    if !diagnostics.is_empty() && diagnostics.iter().any(|d: &crate::wgsl_diagnostics::Diagnostic| d.severity == crate::wgsl_diagnostics::DiagnosticSeverity::Error) {
        return;
    }
    
    // Extract parameters from code
    // TODO: Implement proper parameter extraction
    let shader_params = extract_shader_parameters(&editor_state.code);
    
    // Apply timeline animation to parameters
    // TODO: Implement proper timeline animation
    
    // Prepare parameter values array
    let mut param_values = vec![0.0f32; 64];
    // TODO: Implement proper parameter value extraction
    
    // Get audio data if available
    let audio_data = if audio_analyzer.enabled {
        let audio_info = audio_analyzer.get_audio_data();
        Some(crate::audio_system::AudioData {
            volume: audio_info.volume,
            bass_level: audio_info.bass_level,
            mid_level: audio_info.mid_level,
            treble_level: audio_info.treble_level,
            beat_detected: audio_info.beat_detected,
            beat_intensity: audio_info.beat_intensity,
            tempo: audio_info.tempo,
            waveform: audio_info.waveform.clone(),
            frequencies: audio_info.frequencies.clone(),
        })
    } else {
        None
    };
    
    // Compile and render
    let start_time = std::time::Instant::now();
    match wgpu_pipeline.render_shader_frame(
        &editor_state.code,
        editor_state.preview_resolution.0,
        editor_state.preview_resolution.1,
        time.elapsed_secs(),
        audio_data.as_ref(),
        Some(&param_values),
    ) {
        Ok(_) => {
            // TODO: Implement proper compiled code tracking
            // editor_state.last_compiled_code = Some(editor_state.code.clone());
        }
        Err(e) => {
            editor_state.compilation_error = e.to_string();
        }
    }
}

/// Helper function to extract shader parameters from WGSL code
fn extract_shader_parameters(code: &str) -> Vec<crate::timeline::ShaderParameter> {
    let mut params = vec![];
    
    // Simple regex-like parsing for @group(X) @binding(Y) uniforms
    let lines: Vec<&str> = code.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Look for uniform declarations with group/binding
        if trimmed.contains("@group(") && trimmed.contains("@binding(") && trimmed.contains("var<") {
            if let Some(group_start) = trimmed.find("@group(") {
                if let Some(group_end) = trimmed[group_start..].find(")") {
                    let group_str = &trimmed[group_start + 7..group_start + group_end];
                    if let Ok(group) = group_str.parse::<u32>() {
                        if let Some(binding_start) = trimmed.find("@binding(") {
                            if let Some(binding_end) = trimmed[binding_start..].find(")") {
                                let binding_str = &trimmed[binding_start + 9..binding_start + binding_end];
                                if let Ok(binding) = binding_str.parse::<u32>() {
                                    // Extract parameter name and type
                                    if let Some(var_start) = trimmed.find("var<") {
                                        if let Some(var_end) = trimmed[var_start..].find(">") {
                                            let var_content = &trimmed[var_start + 4..var_start + var_end];
                                            if let Some(name_start) = trimmed[var_start + var_end + 1..].find(|c: char| c.is_alphabetic()) {
                                                let name_part = &trimmed[var_start + var_end + 1 + name_start..];
                                                if let Some(name_end) = name_part.find(|c: char| !c.is_alphanumeric() && c != '_') {
                                                    let name = &name_part[..name_end];
                                                    
                                                    // Extract type and default values
                                                    let param_type = if var_content.contains("f32") {
                                                        "f32"
                                                    } else if var_content.contains("i32") {
                                                        "i32"
                                                    } else {
                                                        "unknown"
                                                    };
                                                    
                                                    params.push(crate::timeline::ShaderParameter {
                                                        name: name.to_string(),
                                                        value: 0.5, // Default value
                                                        min: 0.0,
                                                        max: 1.0,
                                                        default: 0.5,
                                                        binding: binding,
                                                        group: group,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    params
}

/// Plugin to add WGPU rendering capabilities to the Bevy app
pub struct WgpuRenderPlugin;

impl Plugin for WgpuRenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<WgpuRenderPipeline>()
            // For startup timing - using a custom resource instead
            // .insert_resource(Instant::now())
            .add_systems(Update, (
                initialize_wgpu_system,
                shader_compilation_system,
                live_preview_system,
            ).chain());
    }
}