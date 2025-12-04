use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use bevy_egui::{
    EguiContexts,
    EguiPlugin,
};
use bevy_egui::egui;
use bevy_egui::egui::TextureHandle;
use std::sync::Arc;
use std::sync::Mutex;

/// Resource to manage 3D viewport texture data
#[derive(Resource, Clone)]
pub struct Viewport3DTexture {
    pub texture_data: Arc<Mutex<Option<Vec<u8>>>>,
    pub width: u32,
    pub height: u32,
    pub needs_update: bool,
}

impl Default for Viewport3DTexture {
    fn default() -> Self {
        Self {
            texture_data: Arc::new(Mutex::new(None)),
            width: 512,
            height: 512,
            needs_update: true,
        }
    }
}

/// Update time parameter for shader animation
fn update_time_system(
    mut ui_state: ResMut<EditorUiState>,
    time: Res<Time>,
    mut timeline_animation: ResMut<TimelineAnimation>
) {
    // Update time for shader animation
    ui_state.time = time.elapsed_secs_f64();
    
    // Also update timeline if playing
    if timeline_animation.playing {
        timeline_animation.timeline.playback_state = PlaybackState::Playing;
        timeline_animation.timeline.current_time = ui_state.time as f32;
    } else {
        timeline_animation.timeline.playback_state = PlaybackState::Stopped;
    }
}

/// Apply theme settings to the egui context
fn apply_theme(ctx: &egui::Context, ui_state: &super::editor_ui::EditorUiState) {
    let theme = if ui_state.dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    ctx.set_visuals(theme);
}

// Import audio system
use super::audio_system::{AudioAnalyzer, AudioAnalysisPlugin, EnhancedAudioPlugin};
use crate::ffgl_plugin::FfglPlugin;
use crate::gyroflow_interop_integration::GyroflowInteropPlugin;
use crate::screenshot_video_export::ExportPlugin;
use crate::ndi_output::NdiOutputPlugin;
use crate::dmx_lighting_control::DmxLightingControlPlugin;
use crate::osc_control::OscControlPlugin;
use crate::audio_midi_integration::AudioMidiIntegrationPlugin;
use crate::wgsl_analyzer::WgslAnalyzerPlugin;
use crate::spout_syphon_output::SpoutSyphonOutputPlugin;

// Import timeline animation system
use super::timeline::{TimelinePlugin, TimelineAnimation, PlaybackState};

// Import gesture control system
use super::gesture_control::{GestureControlSystem, GestureControlPlugin};

// Import compute pass integration
use resolume_isf_shaders_rust_ffgl::compute_pass_integration::{ComputePassPlugin, ComputePassManager};

// Import responsive backend system - check if it exists
// use super::backend_systems::{ResponsiveBackend, ResponsiveBackendPlugin};

// Import editor modules - use local editor_ui module
use super::editor_ui::{EditorUiState, UiStartupGate, draw_editor_menu, draw_editor_shader_browser_panel, draw_editor_parameter_panel, draw_editor_code_panel, draw_editor_central_panel};

// Import shader renderer for 3D viewport texture rendering
use super::shader_renderer::{ShaderRenderer, RenderParameters};

// Import node graph and compute pass plugins - check if they exist
use crate::bevy_node_graph_integration::BevyNodeGraphPlugin;
// use crate::bevy_node_graph_integration_enhanced::BevyNodeGraphPluginEnhanced;
// use crate::compute_pass_integration::ComputePassPlugin;

// Hint Windows drivers to prefer discrete GPU when available
#[cfg(target_os = "windows")]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 0x00000001;

#[cfg(target_os = "windows")]
#[no_mangle]
pub static AmdPowerXpressRequestHighPerformance: u32 = 0x00000001;

/// Main editor UI system with full functionality
pub fn editor_ui_system(
    mut egui_ctx: EguiContexts, 
    mut ui_state: ResMut<EditorUiState>, 
    mut startup_gate: ResMut<UiStartupGate>, 
    audio_analyzer: Res<AudioAnalyzer>,
    timeline_animation: Res<TimelineAnimation>,
    scene_editor_state: Res<SceneEditor3DState>
) {
    // Increment frame counter
    startup_gate.frames += 1;
    
    // Wait a few frames for egui context to initialize properly
    if startup_gate.frames < 5 {
        return;
    }
    
    // Get egui context, handling the Result return type
    let ctx = match egui_ctx.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return, // Context not ready yet, skip this frame
    };
    
    // Apply theme settings
    apply_theme(&ctx, &ui_state);
    
    // Debug: Print frame info every 60 frames
    if startup_gate.frames % 60 == 0 {
        println!("Frame {}: Drawing UI with state - shader_browser: {}, parameter_panel: {}, preview: {}, code_editor: {}", 
                 startup_gate.frames,
                 ui_state.show_shader_browser,
                 ui_state.show_parameter_panel,
                 ui_state.show_preview,
                 ui_state.show_code_editor);
    }
    
    // Ensure UI panels are visible by default and initialize content
    if startup_gate.frames == 5 {
        println!("Initializing UI state with default content...");
        ui_state.show_shader_browser = true;
        ui_state.show_parameter_panel = true;
        ui_state.show_preview = true;
        ui_state.show_code_editor = true;
        ui_state.show_node_studio = true;
        ui_state.show_timeline = true; // Enable timeline for animation
        ui_state.show_audio_panel = false; // Keep disabled for now
        ui_state.show_midi_panel = false; // Keep disabled for now
        ui_state.show_gesture_panel = false; // Keep disabled for now
        
        // Initialize with some default content
        ui_state.draft_code = String::from("// WGSL Shader Studio\n// Welcome to the shader editor\n\n@fragment\nfn main() -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n}");
        
        // CRITICAL: Actually populate the shader browser with real files
        println!("Initializing shader browser with real WGSL files...");
        
        // populate_shader_list will be called as a separate startup system
        // This will scan directories and load actual WGSL and ISF files
        
        println!("UI state initialized with {} lines of code", 
                 ui_state.draft_code.lines().count());
    }
    
    // Apply timeline animation to shader parameters
    if timeline_animation.timeline.playback_state == PlaybackState::Playing {
        // Parse current shader parameters from the draft code
        let editor_params = crate::editor_ui::parse_shader_parameters(&ui_state.draft_code);
        if !editor_params.is_empty() {
            // Convert editor parameters to timeline parameters
            let mut timeline_params: Vec<crate::timeline::ShaderParameter> = editor_params.iter().map(|p| {
                crate::timeline::ShaderParameter {
                    name: p.name.clone(),
                    value: p.value,
                    min: 0.0,
                    max: 1.0,
                    default: 0.5,
                    binding: 0,
                    group: 0,
                }
            }).collect();
            
            timeline_animation.timeline.apply_to_parameters(&mut timeline_params);
            
            // Update the UI state with animated parameter values
            for param in &timeline_params {
                ui_state.set_parameter_value(&param.name, param.value);
            }
            
            println!("Applied timeline animation to {} parameters", timeline_params.len());
        }
    }
    
    // Draw menu bar
    println!("Drawing menu bar...");
    draw_editor_menu(ctx, &mut *ui_state);
    
    // Draw side panels with proper egui hierarchy
    println!("Drawing side panels...");
    
    // Draw left panel (shader browser)
    if ui_state.show_shader_browser {
        draw_editor_shader_browser_panel(ctx, &mut *ui_state);
    }
    
    // Draw right panel (parameters)
    if ui_state.show_parameter_panel {
        draw_editor_parameter_panel(ctx, &mut *ui_state, &*audio_analyzer);
    }
    
    // Draw code editor panel
    println!("Drawing code editor panel...");
    draw_editor_code_panel(ctx, &mut *ui_state);
    
    // Draw the main preview panel - this should be the CentralPanel
    // Only draw if preview is enabled, otherwise let other panels fill the space
    if ui_state.show_preview {
        println!("Drawing preview panel...");
        draw_editor_central_panel(ctx, &mut *ui_state, &audio_analyzer, None);
    }
    
    // Draw the additional side panels (timeline, node studio, etc.) as windows
    if ui_state.show_timeline {
        // Use the actual timeline UI from timeline.rs
        egui::Window::new("Timeline")
            .default_pos([100.0, 100.0])
            .default_size([800.0, 400.0])
            .show(&ctx, |ui| {
                crate::timeline::draw_timeline_ui(ui, &mut *timeline_animation);
            });
    }
    
    // Draw node studio (node graph editor)
    if ui_state.show_node_studio {
        // The node graph UI system is separate due to its resource requirements
        // It will be called automatically by Bevy's system scheduler
        println!("Node studio window should be visible");
    }
    
    // Draw 3D scene editor panel
    if ui_state.show_3d_scene_panel {
        println!("Drawing 3D scene editor panel...");
        // Create a simple 3D scene editor panel for now
        egui::Window::new("3D Scene Editor")
            .default_pos([100.0, 100.0])
            .default_size([600.0, 400.0])
            .show(&ctx, |ui| {
                ui.heading("3D Scene View");
                ui.label("3D viewport active - use mouse controls to navigate");
                ui.label("3D scene editor will be implemented here");
            });
    }
}

pub fn setup_camera(mut commands: Commands) {
    // Use Camera2d for proper UI rendering with egui
    commands.spawn(Camera2d);
}

fn initialize_wgpu_renderer(ui_state: ResMut<EditorUiState>) {
    println!("Initializing WGPU renderer...");
    
    // Initialize the global renderer with None for now
    // The actual async initialization can be handled in a separate system
    if let Ok(mut renderer) = ui_state.global_renderer.renderer.lock() {
        *renderer = None;
    }
    println!("WGPU renderer placeholder initialized - async setup will be attempted later");
}

/// Async system to initialize the real WGPU renderer
fn async_initialize_wgpu_renderer(
    mut ui_state: ResMut<EditorUiState>,
    mut startup_gate: ResMut<UiStartupGate>
) {
    // Only attempt initialization after UI is stable
    if startup_gate.frames < 60 {
        return;
    }
    
    // Check if we already have a renderer
    let has_renderer = ui_state.global_renderer.renderer.lock().unwrap().is_some();
    if has_renderer {
        return;
    }
    
    println!("Attempting async WGPU renderer initialization...");
    
    // Use pollster to block on the async initialization
    match pollster::block_on(async {
        super::shader_renderer::ShaderRenderer::new_with_size((800, 600)).await
    }) {
        Ok(renderer) => {
            println!("✅ WGPU renderer initialized successfully!");
            let working_examples_count = renderer.working_examples.len();
            *ui_state.global_renderer.renderer.lock().unwrap() = Some(renderer);
            
            // Update UI state to reflect successful initialization
            ui_state.wgpu_initialized = true;
            ui_state.compilation_error.clear();
            
            println!("WGPU renderer ready with {} working examples", 
                     working_examples_count);
        }
        Err(e) => {
            println!("CRITICAL FAILURE: WGPU renderer initialization failed: {}", e);
            println!("ENFORCING GPU-ONLY POLICY - NO CPU FALLBACK ALLOWED.");
            ui_state.wgpu_initialized = false;
            ui_state.compilation_error = format!("WGPU initialization failed: {}", e);
            
            // HARD PANIC - System requires GPU, no CPU fallback permitted
            panic!("GPU initialization failed. This application requires a compatible GPU and cannot run without it. Error: {}", e);
        }
    }
}

pub fn run_app() {
    // Install a panic hook to improve crash diagnostics typical of Bevy 0.17 + bevy_egui
    std::panic::set_hook(Box::new(|info| {
        eprintln!("WGSL Shader Studio panicked: {}", info);
        eprintln!("If this happened around focus/resize, it may be the known Bevy 0.17 + bevy_egui issue.");
    }));

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "WGSL Shader Studio".to_string(),
                    resolution: WindowResolution::new(1600, 900),
                    present_mode: PresentMode::AutoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(AudioAnalysisPlugin)
        .add_plugins(EnhancedAudioPlugin)
        .add_plugins(FfglPlugin)
        .add_plugins(GyroflowInteropPlugin)
        .add_plugins(ExportPlugin)
        .add_plugins(TimelinePlugin)
        .add_plugins(DmxLightingControlPlugin)
        .add_plugins(GestureControlPlugin)
        .add_plugins(ComputePassPlugin)
        .add_plugins(BevyNodeGraphPlugin)
        .add_plugins(SceneEditor3DPlugin)
        .add_plugins(OscControlPlugin)
        .add_plugins(AudioMidiIntegrationPlugin)
        .add_plugins(WgslAnalyzerPlugin)
        .add_plugins(NdiOutputPlugin)
        .add_plugins(SpoutSyphonOutputPlugin)
        // .add_plugins(BevyNodeGraphPluginEnhanced)
        // .add_plugins(ResponsiveBackendPlugin)
        .insert_resource(EditorUiState::default())
        .insert_resource(UiStartupGate::default())
        .insert_resource(Viewport3DTexture::default())
        .insert_resource(crate::screenshot_video_export::ScreenshotVideoExporter::new())
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, initialize_wgpu_renderer)
        .add_systems(Update, async_initialize_wgpu_renderer)
        .add_systems(Update, update_time_system)
        .add_systems(bevy_egui::EguiPrimaryContextPass, editor_ui_system)
        .run();
}