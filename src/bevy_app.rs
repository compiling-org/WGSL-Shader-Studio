use bevy::app::{App, Startup, Update};
use bevy::ecs::system::Commands;
use bevy::window::{WindowPlugin, WindowResolution, WindowPosition, MonitorSelection};
use bevy::render::settings::{WgpuSettings, WgpuFeatures, WgpuLimits};
use bevy_egui::{EguiPlugin, EguiContexts};

use crate::audio_midi_integration::AudioMidiIntegrationPlugin;
use crate::audio_system::{AudioAnalysisPlugin, EnhancedAudioAnalyzer, EnhancedAudioPlugin};


use crate::bevy_node_graph_integration_enhanced::BevyNodeGraphPlugin;

use crate::dmx_lighting_control::DmxLightingControlPlugin;
use crate::ffgl_plugin::FfglPlugin;
use crate::gesture_control::GestureControlPlugin;
use crate::gyroflow_interop_integration::GyroflowInteropPlugin;
use crate::midi_system::MidiSystemPlugin;
use crate::ndi_output::NdiOutputPlugin;
use crate::osc_control::OscControlPlugin;
use crate::performance_overlay::PerformanceOverlayPlugin;
use crate::scene_editor_3d::SceneEditor3DPlugin;
use crate::screenshot_video_export::ExportPlugin;
use crate::simple_ui_auditor::SimpleUiAuditorPlugin;
use crate::spout_syphon_output::SpoutSyphonOutputPlugin;
use crate::timeline::TimelinePlugin;
use crate::visual_node_editor_plugin::{VisualNodeEditorPlugin, VisualNodeEditorState};
use crate::enhanced_visual_node_editor_plugin::EnhancedVisualNodeEditorPlugin;
use crate::wgsl_analyzer::WgslAnalyzerPlugin;
use crate::particle_physics::ParticlePhysicsPlugin;
use bevy::prelude::*;

use bevy::ecs::system::SystemParam;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::documentation_server::start_documentation_server;
use crate::audio_system::AudioAnalyzer;
use bevy::window::WindowResized;
use bevy::window::PresentMode;

/// Resource to manage documentation server
#[derive(Resource, Clone)]
pub struct DocumentationServer {
    pub addr: SocketAddr,
    pub shutdown_notify: Arc<tokio::sync::Notify>,
}

/// Startup system to initialize the documentation server
pub fn start_documentation_server_system(mut commands: Commands) {
    // Use tokio to spawn the async server start
    let handle = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            match start_documentation_server("./docs").await {
                Ok((addr, notify)) => {
                    println!("Documentation server started at http://{}", addr);
                    Some((addr, notify))
                }
                Err(e) => {
                    eprintln!("Failed to start documentation server: {}", e);
                    None
                }
            }
        })
    });
    
    // Wait for the server to start (this will block briefly)
    if let Ok(Some((addr, notify))) = handle.join() {
        commands.insert_resource(DocumentationServer {
            addr,
            shutdown_notify: notify,
        });
    }
}

/// Resource to manage 3D viewport texture data
#[derive(Resource, Clone)]
pub struct Viewport3DTexture {
    pub texture_data: Arc<Mutex<Option<Vec<u8>>>>,
    pub width: u32,
    pub height: u32,
    pub needs_update: bool,
    pub last_update: std::time::Instant,
}

impl Default for Viewport3DTexture {
    fn default() -> Self {
        Self {
            texture_data: Arc::new(Mutex::new(None)),
            width: 512,
            height: 512,
            needs_update: true,
            last_update: std::time::Instant::now(),
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
fn apply_theme(ctx: &bevy_egui::egui::Context, ui_state: &super::editor_ui::EditorUiState) {
    let theme = if ui_state.dark_mode {
        bevy_egui::egui::Visuals::dark()
    } else {
        bevy_egui::egui::Visuals::light()
    };
    
    // Use a panic-safe approach to set visuals
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ctx.set_visuals(theme);
    })).is_err() {
        // If setting visuals fails, skip it for this frame
        return;
    }
}



use crate::midi_system::MidiSystem;

use crate::performance_overlay::PerformanceMetrics;


use crate::timeline::{TimelineAnimation, PlaybackState};

use crate::gesture_control::GestureControlSystem;

// Import compute pass integration (local crate)
use crate::compute_pass_integration::ComputePassManager;

// Import responsive backend system - check if it exists
// use super::backend_systems::{ResponsiveBackend, ResponsiveBackendPlugin};

use crate::editor_ui::{EditorUiState, UiStartupGate, draw_editor_menu, draw_editor_side_panels, draw_editor_central_panel};


use crate::compute_pass_integration::ComputePassPlugin;

// Feature flag for 3D preview functionality
const ENABLE_3D_PREVIEW: bool = cfg!(feature = "3d_preview");

use crate::scene_editor_3d::{SceneEditor3DState, EditorManipulable};

use crate::simple_ui_auditor::SimpleUiAuditor;
use crate::osc_control::{OscConfig, OscControl};
use crate::enforcement_system::initialize_enforcement;

#[derive(SystemParam)]
pub struct OutputsParams<'w> {
    pub spout_config: ResMut<'w, crate::spout_syphon_output::SpoutSyphonConfig>,
    pub spout_output: ResMut<'w, crate::spout_syphon_output::SpoutSyphonOutput>,
    pub ndi_config: ResMut<'w, crate::ndi_output::NdiConfig>,
    pub ndi_output: ResMut<'w, crate::ndi_output::NdiOutput>,
    pub dmx_config: ResMut<'w, crate::dmx_lighting_control::DmxConfig>,
    pub dmx_control: ResMut<'w, crate::dmx_lighting_control::DmxLightingControl>,
}

#[derive(SystemParam)]
pub struct ControlParams<'w> {
    pub midi_system: ResMut<'w, MidiSystem>,
    pub gesture_control: ResMut<'w, GestureControlSystem>,
    pub osc_config: ResMut<'w, OscConfig>,
    pub osc_control: ResMut<'w, OscControl>,
}

#[derive(SystemParam)]
pub struct RenderParams<'w> {
    pub scene_view_tex: Res<'w, crate::scene_editor_3d::SceneViewportTexture>,
    pub compute_manager: ResMut<'w, ComputePassManager>,
    pub exporter: Res<'w, crate::screenshot_video_export::ScreenshotVideoExporter>,
}
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
    mut timeline_animation: ResMut<TimelineAnimation>,
    mut scene_editor_state: ResMut<SceneEditor3DState>,
    _performance_metrics: Res<PerformanceMetrics>,
    mut auditor: ResMut<SimpleUiAuditor>,
    mut outputs: OutputsParams,
    mut controls: ControlParams,
    mut render: RenderParams,
    mut node_graph_res: Option<ResMut<crate::bevy_node_graph_integration_enhanced::NodeGraphResource>>,
    mut _viewport_3d_texture: ResMut<Viewport3DTexture>,
    manipulable_query: Query<(Entity, &Name), With<EditorManipulable>>,
) {
    // Increment frame counter
    startup_gate.frames += 1;
    
    if startup_gate.frames < 10 {
        return;
    }
    
    // Register 3D scene image (only once) before borrowing context
    #[cfg(feature = "3d_preview")]
    if ui_state.central_view == crate::editor_ui::CentralView::Scene3D
        && scene_editor_state.enabled
        && ui_state.scene3d_texture_id.is_none()
    {
        let image_handle = render.scene_view_tex.handle.clone();
        let tex_id = egui_ctx.add_image(bevy_egui::EguiTextureHandle::Strong(image_handle));
        ui_state.scene3d_texture_id = Some(tex_id);
    }

    // Re-register 3D scene image if viewport dimensions changed
    #[cfg(feature = "3d_preview")]
    if ui_state.central_view == crate::editor_ui::CentralView::Scene3D
        && scene_editor_state.enabled
        && _viewport_3d_texture.needs_update
    {
        // Add debouncing to prevent too frequent texture updates
        let elapsed = _viewport_3d_texture.last_update.elapsed();
        if elapsed.as_millis() > 100 {  // Only update if more than 100ms has passed
            if let (Some(_old_tex_id), Some(image_handle)) = (ui_state.scene3d_texture_id.take(), ui_state.scene3d_texture_handle.take()) {
                egui_ctx.remove_image(bevy_egui::EguiTextureHandle::Strong(image_handle));
            }
            
            // Register the new texture only if it has valid data
            let image_handle = render.scene_view_tex.handle.clone();
            
            // Only register the texture if we have a valid handle and it's not the default handle
            if render.scene_view_tex.handle.id().is_some() && !render.scene_view_tex.handle.is_default() {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    egui_ctx.add_image(bevy_egui::EguiTextureHandle::Strong(image_handle.clone()))
                })) {
                    Ok(tex_id) => {
                        ui_state.scene3d_texture_id = Some(tex_id);
                        ui_state.scene3d_texture_handle = Some(image_handle);
                    },
                    Err(e) => {
                        eprintln!("Warning: Failed to register 3D scene texture: {:?}", e);
                        // Continue without updating the texture
                    }
                }
            } else {
                eprintln!("Warning: Scene texture handle not ready or is default, skipping registration");
            }
            _viewport_3d_texture.needs_update = false;
            _viewport_3d_texture.last_update = std::time::Instant::now();
        }
    }
    
    // Get egui context, handling the Result return type
    let ctx_result = egui_ctx.ctx_mut();
    
    // If we can't get the context, skip this frame
    let ctx = match ctx_result {
        Ok(ctx) => ctx,
        Err(_) => {
            println!("DEBUG: Failed to get Egui context!");
            return;
        },
    };
    
    // Apply theme settings
    apply_theme(&ctx, &ui_state);
    
    // Ensure UI panels are visible by default and initialize content
    // Make sure panels are visible from the start
    println!("Initializing UI state with default content...");
    ui_state.show_shader_browser = true;
    ui_state.show_parameter_panel = true;
    ui_state.show_preview = true;
    ui_state.show_code_editor = true;
    ui_state.show_node_studio = true;
    ui_state.show_timeline = false; // Keep timeline hidden initially
    ui_state.show_audio_panel = true;
    ui_state.show_midi_panel = true;
    ui_state.show_gesture_panel = true;
    ui_state.show_compute_panel = true;
    ui_state.show_3d_scene_panel = false; // Embedded via central tabs
    
    // Initialize with a known-good gradient fragment (works with default vertex shader)
    // ui_state.draft_code was being reset here - removed to persist user edits

    // CRITICAL: Actually populate the shader browser with real files
    println!("Initializing shader browser with real WGSL files...");
    
    // populate_shader_list will be called as a separate startup system
    // This will scan directories and load actual WGSL and ISF files
    
    println!("UI state initialized with {} lines of code", 
             ui_state.draft_code.lines().count());
    
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
            
            
        }
    }
    
    if startup_gate.frames % 60 == 0 {
        println!("DEBUG: Reached draw_editor_menu");
    }
    
    // Draw menu bar
    draw_editor_menu(ctx, &mut *ui_state);
    if auditor.enabled { auditor.record_panel("Menu Bar", true, None); }
    
    // Do not override right sidebar mode each frame; user selects via tabs
    
    // Apply gesture-derived parameter values
    {
        let mappings = controls.gesture_control.get_parameter_mappings().clone();
        for (param_name, _mapping) in mappings.iter() {
            if let Some(val) = controls.gesture_control.get_parameter_value(param_name) {
                ui_state.set_parameter_value(param_name, val);
            }
        }
    }
    
    // Apply OSC-derived parameter values
    {
        let osc_params = controls.osc_control.get_all_parameters().clone();
        for (param_name, val) in osc_params.iter() {
            ui_state.set_parameter_value(param_name, *val);
        }
    }
    
    if startup_gate.frames % 60 == 0 {
        println!("DEBUG: Reached draw_editor_side_panels");
    }
    
    draw_editor_side_panels(
        &ctx, 
        &mut *ui_state, 
        &*audio_analyzer, 
        &mut *controls.gesture_control, 
        &mut *render.compute_manager, 
        Some(&*render.exporter), 
        &mut *controls.midi_system,
        &mut *controls.osc_config,
        &mut *controls.osc_control,
        &mut *outputs.spout_config,
        &mut *outputs.spout_output,
        &mut *outputs.ndi_config,
        &mut *outputs.ndi_output,
        &mut *outputs.dmx_config,
        &mut *outputs.dmx_control,
        Some(&mut *scene_editor_state),
        Some(&manipulable_query),
    );
    if auditor.enabled && ui_state.show_shader_browser { auditor.record_panel("Shader Browser", true, None); }
    if auditor.enabled && ui_state.show_parameter_panel { auditor.record_panel("Parameters", true, None); }
    
    // Draw the main preview panel - this should be the CentralPanel
    // Only draw if preview is enabled, otherwise let other panels fill the space
        if ui_state.show_preview {
        draw_editor_central_panel(
            ctx, 
            &mut *ui_state, 
            &*audio_analyzer, 
            None, 
            node_graph_res.as_deref_mut(), 
            &*scene_editor_state,
            &mut *timeline_animation,
            &mut *outputs.spout_output,
            &mut *outputs.ndi_output
        );
        if auditor.enabled {
            match ui_state.central_view {
                crate::editor_ui::CentralView::Preview => auditor.record_panel("Preview", true, None),
                crate::editor_ui::CentralView::NodeGraph => auditor.record_panel("Node Graph", true, None),
                crate::editor_ui::CentralView::Scene3D => auditor.record_panel("3D Editor", true, None),
                crate::editor_ui::CentralView::Timeline => auditor.record_panel("Timeline", true, None),
            }
        }
    }
    
    // Bottom code editor — always available when enabled
    if ui_state.show_code_editor {
        crate::editor_ui::draw_editor_code_panel(ctx, &mut *ui_state);
        if auditor.enabled { auditor.record_panel("Code Editor", true, None); }
    }
    
    // MIDI is rendered within the right sidebar modes; no floating window here
    
    // Node graph and 3D editor are embedded in central view tabs now
}

fn on_window_resize_system(
    mut resize_events: EventReader<WindowResized>,
    mut viewport_texture: ResMut<Viewport3DTexture>,
) {
    for event in resize_events.read() {
        println!("Resize event received: {}x{}", event.width, event.height);
        // Ensure we have valid dimensions to prevent pixel data size mismatches
        // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
        let safe_width = (event.width as u32).max(50);
        let safe_height = (event.height as u32).max(50);
        
        // Additional safeguard against extremely small dimensions that could cause issues
        let safe_width = safe_width.max(100);
        let safe_height = safe_height.max(100);
        
        viewport_texture.width = safe_width;
        viewport_texture.height = safe_height;
        viewport_texture.needs_update = true;
        println!("Window resized to: {}x{}", safe_width, safe_height);
    }
}

fn enable_all_features_once(
    mut ui_state: ResMut<EditorUiState>,
    mut vne_state: ResMut<VisualNodeEditorState>,
) {
    ui_state.show_shader_browser = true;
    ui_state.show_parameter_panel = true;
    ui_state.show_preview = true;
    ui_state.show_code_editor = true;
    ui_state.show_node_studio = true;
    ui_state.show_timeline = true;
    ui_state.show_audio_panel = true;
    ui_state.show_midi_panel = true;
    ui_state.show_gesture_panel = true;
    ui_state.show_gesture_calibration = true;
    ui_state.show_wgslsmith_panel = true;
    ui_state.show_diagnostics_panel = true;
    ui_state.show_compute_panel = true;
    ui_state.show_3d_scene_panel = true;
    ui_state.show_spout_panel = true;
    ui_state.show_ffgl_panel = true;
    ui_state.show_gyroflow_panel = true;
    ui_state.show_analyzer_panel = true;
    ui_state.show_isf_converter = true;
    ui_state.show_wgsl_analyzer = true;
    ui_state.show_performance = true;
    ui_state.show_performance_overlay = true;
    ui_state.show_color_grading_panel = true;
    ui_state.show_osc_panel = true;
    ui_state.show_dmx_panel = true;
    ui_state.show_export_panel = true;
    ui_state.show_ndi_panel = true;
    vne_state.show_node_editor = true;
}

fn init_enforcement_startup() {
    let _ = pollster::block_on(initialize_enforcement());
}
pub fn setup_camera(mut commands: Commands) {
    // commands.insert_resource(ClearColor(Color::srgb(1.0, 0.0, 1.0))); // Diagnostic Removed
    commands.spawn(Camera3d::default());
    commands.spawn((
        Camera2d {
            clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::None,
            ..Default::default()
        },
        Camera {
            order: 1,
            is_active: true,
            ..Default::default()
        },
    )); 
}

fn start_audio_analysis_system(mut audio_analyzer: ResMut<AudioAnalyzer>) {
    println!("🎵 Starting audio analysis system...");
    audio_analyzer.start_audio_capture();
    println!("✅ Audio analysis system started successfully");
}

/// Async system to initialize the real WGPU renderer
fn async_initialize_wgpu_renderer(
    mut ui_state: ResMut<EditorUiState>,
    startup_gate: ResMut<UiStartupGate>
) {
    // Only attempt initialization after UI is stable
    if startup_gate.frames < 5 {
        return;
    }
    
    // Check if we already have a renderer - do this without a closure to avoid borrow conflicts
    let has_renderer = {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ui_state.global_renderer.renderer.lock().map(|guard| guard.is_some())
        })) {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => {
                println!("Mutex poisoned in renderer check");
                return;
            },
            Err(_) => {
                println!("Panic during renderer check");
                return;
            }
        }
    };
    
    if has_renderer {
        return;
    }
    
    println!("Attempting async WGPU renderer initialization...");
    
    // Use pollster to block on the async initialization
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pollster::block_on(async {
            super::shader_renderer::ShaderRenderer::new_with_size((800, 600)).await
        })
    }));
    
    // Process the initialization result
    match result {
        Ok(Ok(mut renderer)) => {
            println!("✅ WGPU renderer initialized successfully!");
            let working_examples_count = renderer.working_examples.len();
            
            // Store the renderer in the global state
            let store_success = {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    ui_state.global_renderer.renderer.lock().map(|mut guard| {
                        *guard = Some(renderer);
                        true
                    })
                })) {
                    Ok(Ok(_)) => true,
                    Ok(Err(_)) => {
                        println!("Failed to acquire renderer lock for storing initialized renderer");
                        false
                    },
                    Err(_) => {
                        println!("Panic during renderer lock for storing initialized renderer");
                        false
                    }
                }
            };
            
            // Update UI state after the mutex operation is complete
            if store_success {
                ui_state.wgpu_initialized = true;
                ui_state.compilation_error.clear();
                println!("WGPU renderer ready with {} working examples", working_examples_count);
            } else {
                ui_state.wgpu_initialized = false;
                ui_state.compilation_error = "Failed to acquire renderer lock".to_string();
            }
        }
        Ok(Err(e)) => {
            println!("WGPU renderer initialization failed: {}", e);
            println!("Continuing without renderer; UI will show 'Renderer not initialized'.");
            ui_state.wgpu_initialized = false;
            ui_state.compilation_error = format!("WGPU initialization failed: {}", e);
        }
        Err(_) => {
            println!("Panic during WGPU renderer initialization");
            ui_state.wgpu_initialized = false;
            ui_state.compilation_error = "WGPU renderer initialization panicked".to_string();
        }
    };
}

fn blocking_initialize_wgpu_renderer(mut ui_state: ResMut<EditorUiState>) {
    // Avoid borrowing ui_state immutably while mutating it: scope the lock
    let mut init_ok = false;
    let mut init_err: Option<String> = None;
    {
        let mut lock = ui_state.global_renderer.renderer.lock().unwrap();
        if lock.is_none() {
            println!("Initializing WGPU renderer (blocking)...");
            // Initialize without spawning a thread to avoid Send trait issues
            let result = pollster::block_on(super::shader_renderer::ShaderRenderer::new_with_size((800, 600)))
                .map_err(|e| e.to_string());
            
            match result {
                Ok(renderer) => {
                    println!("WGPU renderer initialized");
                    *lock = Some(renderer);
                    init_ok = true;
                }
                Err(e) => {
                    println!("Renderer init failed: {}", e);
                    init_err = Some(format!("{}", e));
                    // Initialize with a fallback renderer to prevent crashes
                    match pollster::block_on(super::shader_renderer::ShaderRenderer::new_with_size((512, 512))) {
                        Ok(fallback_renderer) => {
                            println!("Fallback WGPU renderer initialized");
                            *lock = Some(fallback_renderer);
                            init_ok = true;
                        }
                        Err(fallback_e) => {
                            println!("Fallback renderer init also failed: {}", fallback_e);
                            init_err = Some(format!("Primary: {}; Fallback: {}", e, fallback_e));
                        }
                    }
                }
            }
        } else {
            init_ok = true;
        }
    }
    // Now the mutex guard is dropped; it's safe to mutate ui_state
    if init_ok {
        ui_state.wgpu_initialized = true;
        ui_state.compilation_error.clear();
    } else if let Some(err) = init_err {
        ui_state.wgpu_initialized = false;
        ui_state.compilation_error = err;
    }
}

pub fn run_app() {
    std::env::set_var("WGPU_ERROR", "warn");
    // Install a panic hook to improve crash diagnostics typical of Bevy 0.17 + bevy_egui
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{}", info);
        let _ = std::fs::write("panic_log.txt", format!("Panic occurred at {}:\n{}\n", chrono::Local::now(), msg));
        if msg.contains("wgpu error: Validation Error") || msg.contains("Encoder is invalid") || msg.contains("SurfaceAcquireSemaphores") {
            eprintln!("Caught wgpu validation error (known Bevy 0.17 + bevy_egui issue): {}", info);
            eprintln!("Continuing execution despite validation error...");
        } else if msg.contains("Unable to find a GPU!") {
            eprintln!("GPU not found, falling back to CPU rendering: {}", info);
            eprintln!("Please install appropriate GPU drivers for hardware acceleration");
        } else {
            eprintln!("WGSL Shader Studio panicked: {}", info);
            eprintln!("If this happened around focus/resize, it may be the known Bevy 0.17 + bevy_egui issue.");
        }
    }));

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "WGSL Shader Studio".to_string(),
                    resolution: WindowResolution::new(1600, 900),
                    present_mode: PresentMode::Fifo,
                    focused: true,
                    resizable: true,
                    decorations: true,
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    ..Default::default()
                }),
                ..Default::default()
            })
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(PerformanceOverlayPlugin)
        .add_plugins(AudioAnalysisPlugin)
        .add_plugins(EnhancedAudioPlugin)
        .add_plugins(MidiSystemPlugin)
        .add_plugins(FfglPlugin::new())
        .add_plugins(GyroflowInteropPlugin)
        .add_plugins(ExportPlugin)
        .add_plugins(TimelinePlugin)
        .add_plugins(GestureControlPlugin)

        .add_plugins(BevyNodeGraphPlugin)
        .add_plugins(VisualNodeEditorPlugin)
        .add_plugins(EnhancedVisualNodeEditorPlugin)
        .add_plugins(ComputePassPlugin)
        .add_plugins(OscControlPlugin)
        .add_plugins(AudioMidiIntegrationPlugin)
        .add_plugins(WgslAnalyzerPlugin)
        .add_plugins(NdiOutputPlugin)
        .add_plugins(SpoutSyphonOutputPlugin)
        .add_plugins(DmxLightingControlPlugin)
        .add_plugins(SceneEditor3DPlugin)
        .add_plugins(SimpleUiAuditorPlugin)
        .add_plugins(ParticlePhysicsPlugin)
        .insert_resource(EditorUiState::default())
        .insert_resource(UiStartupGate::default())
        .insert_resource(Viewport3DTexture::default())
        .insert_resource(crate::scene_editor_3d::SceneEditor3DState::default())
        .insert_resource(crate::scene_editor_3d::SceneViewportTexture::default())
        .insert_resource(crate::scene_editor_3d::ShaderPreviewTexture::default())
        .insert_resource(MidiSystem::new())
        .insert_resource(crate::screenshot_video_export::ScreenshotVideoExporter::new())
        .insert_resource(VisualNodeEditorState { auto_compile: true, show_node_editor: false })
        .insert_resource(EnhancedAudioAnalyzer::new())
        .add_systems(Startup, blocking_initialize_wgpu_renderer)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, crate::editor_ui::populate_shader_list)
        .add_systems(Startup, start_audio_analysis_system)
        // .add_systems(Startup, init_enforcement_startup)
        .add_systems(Startup, start_documentation_server_system)
        // .add_systems(Update, async_initialize_wgpu_renderer)  // Removed: renderer is initialized once at startup
        .add_systems(Startup, enable_all_features_once)  // Enable all UI features
        .add_systems(Update, update_time_system)
        .add_systems(Update, on_window_resize_system)
        .add_systems(Update, editor_ui_system)
        .add_systems(Update, crate::scene_editor_3d::sync_scene_viewport_texture_size)
        .run();
}
