use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use bevy_egui::{
    EguiContexts,
    EguiPlugin,
};

// Import audio system
use crate::audio::{AudioAnalyzer, AudioAnalysisPlugin};

// Import timeline animation system
use crate::timeline::{TimelinePlugin, TimelineAnimation};

// Import editor modules - use local editor_ui module
// use super::editor_ui::{EditorUiState, UiStartupGate, draw_editor_menu, draw_editor_side_panels, draw_editor_code_panel}; // Temporarily disabled

// Hint Windows drivers to prefer discrete GPU when available
#[cfg(target_os = "windows")]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 0x00000001;

#[cfg(target_os = "windows")]
#[no_mangle]
pub static AmdPowerXpressRequestHighPerformance: u32 = 0x00000001;

/// Main editor UI system with full functionality
/* // Temporarily disabled - editor_ui module issues
fn editor_ui_system(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>, mut startup_gate: ResMut<UiStartupGate>, audio_analyzer: Res<AudioAnalyzer>) {
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
        ui_state.show_node_studio = false; // Keep disabled for now
        ui_state.show_timeline = false; // Keep disabled for now
        ui_state.show_audio_panel = false; // Keep disabled for now
        ui_state.show_midi_panel = false; // Keep disabled for now
        ui_state.show_gesture_panel = false; // Keep disabled for now
        
        // Initialize with some default content
        ui_state.draft_code = String::from("// WGSL Shader Studio\n// Welcome to the shader editor\n\n@fragment\nfn main() -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n}");
        
        // CRITICAL: Actually populate the shader browser with real files
        println!("Initializing shader browser with real WGSL files...");
        
        // Call the populate_shader_list function to load real shaders
        // This will scan directories and load actual WGSL and ISF files
        
        println!("UI state initialized with {} shaders and {} lines of code", 
                 ui_state.available_shaders_compatible.len(),
                 ui_state.draft_code.lines().count());
    }
    
    // Draw menu bar
    println!("Drawing menu bar...");
    draw_editor_menu(ctx, &mut *ui_state);
    
    // Draw side panels (shader browser, parameters, timeline)
    println!("Drawing side panels...");
    draw_editor_side_panels(ctx, &mut *ui_state, &audio_analyzer);
    
    // Draw code editor panel
    println!("Drawing code editor panel...");
    draw_editor_code_panel(ctx, &mut *ui_state);
    
    // Draw the main preview panel - this should be the CentralPanel
    // Only draw if preview is enabled, otherwise let other panels fill the space
    if ui_state.show_preview {
        println!("Drawing preview panel...");
        // The preview panel is drawn within draw_editor_side_panels when show_preview is true
        // This avoids the CentralPanel conflict
    }
}
*/

fn setup_camera(mut commands: Commands) {
    // Use Camera2d for proper UI rendering with egui
    commands.spawn(Camera2d);
}

/* // Temporarily disabled - editor_ui module issues
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
) {*/
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
            println!("❌ Failed to initialize WGPU renderer: {}. Using software fallback.", e);
            // Keep None, software fallback will be used
        }
    }
}
*/

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
        // .insert_resource(EditorUiState::default()) // Temporarily disabled
        // .insert_resource(UiStartupGate::default()) // Temporarily disabled
        .add_systems(Startup, setup_camera)
        // .add_systems(Startup, initialize_wgpu_renderer) // Temporarily disabled
        // .add_systems(Update, async_initialize_wgpu_renderer) // Temporarily disabled
        // .add_systems(bevy_egui::EguiPrimaryContextPass, editor_ui_system) // Temporarily disabled
        .run();
}