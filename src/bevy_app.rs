use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use bevy_egui::{
    EguiContexts,
    EguiPlugin,
};
use bevy_egui::egui;

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
use super::audio_system::{AudioAnalyzer, AudioAnalysisPlugin};

// Import timeline animation system
use super::timeline::{TimelinePlugin, TimelineAnimation, PlaybackState};

// Import gesture control system
use super::gesture_control::{GestureControlSystem, GestureControlPlugin};

// Import responsive backend system - check if it exists
// use super::backend_systems::{ResponsiveBackend, ResponsiveBackendPlugin};

// Import editor modules - use local editor_ui module
use super::editor_ui::{EditorUiState, UiStartupGate, draw_editor_menu, draw_editor_side_panels, draw_editor_code_panel};

// Import node graph and compute pass plugins - check if they exist
// use crate::bevy_node_graph_integration::BevyNodeGraphPlugin;
// use crate::compute_pass_integration::ComputePassPlugin;

// Hint Windows drivers to prefer discrete GPU when available
#[cfg(target_os = "windows")]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 0x00000001;

#[cfg(target_os = "windows")]
#[no_mangle]
pub static AmdPowerXpressRequestHighPerformance: u32 = 0x00000001;

/// Main editor UI system with full functionality
fn editor_ui_system(
    mut egui_ctx: EguiContexts, 
    mut ui_state: ResMut<EditorUiState>, 
    mut startup_gate: ResMut<UiStartupGate>, 
    audio_analyzer: Res<AudioAnalyzer>,
    timeline_animation: Res<TimelineAnimation>,
    mut gesture_control: ResMut<GestureControlSystem>
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
    
    // Update gesture control system and apply gesture parameters
    gesture_control.update();
    
    // Apply gesture control parameters to shader
    if ui_state.show_gesture_panel {
        // Get gesture-controlled parameter values
        for param_name in &["time", "speed", "intensity"] {
            if let Some(gesture_value) = gesture_control.get_parameter_value(param_name) {
                ui_state.set_parameter_value(param_name, gesture_value);
                println!("Applied gesture control to parameter '{}': {}", param_name, gesture_value);
            }
        }
    }
    
    // Draw menu bar
    println!("Drawing menu bar...");
    draw_editor_menu(ctx, &mut *ui_state);
    
    // Draw side panels (shader browser, parameters, timeline)
    println!("Drawing side panels...");
    draw_editor_side_panels(ctx, &mut *ui_state, &audio_analyzer, &gesture_control);
    
    // Draw code editor panel
    println!("Drawing code editor panel...");
    draw_editor_code_panel(ctx, &mut *ui_state);
    
    // Draw the main preview panel using CentralPanel - this creates proper three-panel layout
    if ui_state.show_preview {
        println!("Drawing preview panel...");
        draw_editor_central_panel(ctx, &mut *ui_state);
    }
}

fn setup_camera(mut commands: Commands) {
    // Use Camera2d for proper UI rendering with egui
    commands.spawn(Camera2d);
}

fn initialize_wgpu_renderer(ui_state: ResMut<EditorUiState>) {
    println!("Initializing WGPU renderer...");
    
    // Initialize the global renderer with None for now
    // The actual async initialization can be handled in a separate system
    *ui_state.global_renderer.renderer.lock().unwrap() = None;
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
        super::shader_renderer::ShaderRenderer::new().await
    }) {
        Ok(renderer) => {
            println!("✅ WGPU renderer initialized successfully!");
            *ui_state.global_renderer.renderer.lock().unwrap() = Some(renderer);
        }
        Err(e) => {
            println!("❌ Failed to initialize WGPU renderer: {}. ENFORCING GPU-ONLY POLICY - NO CPU FALLBACK ALLOWED.", e);
            panic!("GPU initialization failed - NO CPU FALLBACK ALLOWED: {}", e);
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
        .add_plugins(TimelinePlugin)
        .add_plugins(GestureControlPlugin)
        // .add_plugins(ResponsiveBackendPlugin)
        // .add_plugins(BevyNodeGraphPlugin)
        // .add_plugins(ComputePassPlugin)
        .insert_resource(EditorUiState::default())
        .insert_resource(UiStartupGate::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, initialize_wgpu_renderer)
        .add_systems(Update, async_initialize_wgpu_renderer)
        .add_systems(bevy_egui::EguiPrimaryContextPass, editor_ui_system)
        .run();
}
fn draw_editor_central_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Shader Preview");
        
        let available_size = ui.available_size();
        let preview_size = egui::vec2(available_size.x.min(800.0), available_size.y.min(400.0));
        let (response, painter) = ui.allocate_painter(preview_size, egui::Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Preview initialized",
            egui::FontId::proportional(14.0),
            egui::Color32::from_gray(180),
        );
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_gray(60)), egui::StrokeKind::Inside);
    });
}