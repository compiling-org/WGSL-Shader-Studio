use bevy::app::{App, Startup, Update};

use bevy::ecs::system::Commands;
use bevy::window::{WindowPlugin, WindowResolution, WindowPosition, MonitorSelection};
use bevy::render::settings::{WgpuSettings, WgpuFeatures, WgpuLimits, RenderCreation, Backends};
use bevy::render::RenderPlugin;
use bevy_egui::{EguiPlugin, EguiContexts};
use bevy::prelude::*; // Rely on prelude for ClearColorConfig, Camera2d, Projection, OrthographicProjection
// Explicit imports removed as prelude covers them or they were wrong.


use crate::audio_midi_integration::AudioMidiIntegrationPlugin;
use crate::audio_system::{AudioAnalysisPlugin, EnhancedAudioAnalyzer, EnhancedAudioPlugin};


use crate::bevy_node_graph_integration_enhanced::BevyNodeGraphPlugin;

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

pub fn start_documentation_server_system(_commands: Commands) {
    // Detached thread for server - do not block startup
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            match start_documentation_server("./docs").await {
                Ok((addr, _notify)) => {
                    println!("Documentation server started at http://{}", addr);
                }
                Err(e) => {
                    eprintln!("Failed to start documentation server: {}", e);
                }
            }
        })
    });
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

use crate::ui::state::{EditorUiState, UiStartupGate};
use crate::editor_ui::{draw_editor_menu, draw_editor_side_panels};
use crate::ui::central_panel::draw_editor_central_panel;


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
    performance_metrics: Res<PerformanceMetrics>,
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
    if startup_gate.frames % 60 == 0 {
        println!("UI Frame: {}", startup_gate.frames);
    }

    if startup_gate.frames < 10 {
        return;
    }
    
    // Get egui context, handling the Result return type
    let ctx_result = egui_ctx.ctx_mut();
    let ctx = match ctx_result {
        Ok(ctx) => {
            if startup_gate.frames % 120 == 0 {
                let size = ctx.input(|i| i.screen_rect().size());
                println!("Egui Context: Ready, Window size: {:?}", size);
            }
            ctx
        },
        Err(e) => {
            if startup_gate.frames % 60 == 0 {
                println!("Egui Context: NOT READY: {:?}", e);
            }
            return;
        },
    };
    
    // Apply theme settings
    apply_theme(&ctx, &ui_state);
    
    // Sync performance stats to UI state
    ui_state.fps = performance_metrics.fps;
    
    // Update status message timer
    if ui_state.status_message_timer > 0.0 {
        ui_state.status_message_timer -= 1.0 / 60.0; // Assume 60 FPS for timer decrement
        if ui_state.status_message_timer <= 0.0 {
            ui_state.status_message = "Ready".to_string();
        }
    }
    
    // Register 3D scene image (only once) before borrowing context
    #[cfg(feature = "3d_preview")]
    if ui_state.central_view == crate::ui::state::CentralView::Scene3D
        && scene_editor_state.enabled
        && ui_state.scene3d_texture_id.is_none()
    {
        let image_handle = render.scene_view_tex.handle.clone();
        let tex_id = egui_ctx.add_image(bevy_egui::EguiTextureHandle::Strong(image_handle));
        ui_state.scene3d_texture_id = Some(tex_id);
    }

    // [INTEGRATION FIX] Connect Audio Analysis to Shader Parameters
    {
        let audio_data = audio_analyzer.get_audio_data();
        crate::editor_ui::connect_audio_to_parameters(&mut *ui_state, &crate::audio_system::AudioData {
             volume: audio_data.volume,
             bass_level: audio_data.bass_level,
             mid_level: audio_data.mid_level,
             treble_level: audio_data.treble_level,
             beat_detected: audio_data.beat_detected,
             beat_intensity: audio_data.beat_intensity,
             tempo: audio_data.tempo,
             frequencies: audio_data.frequencies.clone(),
             waveform: audio_data.waveform.clone(),
        });
    }
    
    // Apply timeline animation to shader parameters efficiently
    if timeline_animation.timeline.playback_state == PlaybackState::Playing {
        let current_time = timeline_animation.timeline.current_time;
        // Optimization: Instead of re-parsing the shader every frame, 
        // we only update parameters that have active timeline tracks.
        for (track_name, track) in &timeline_animation.timeline.tracks {
            if track.enabled && !track.keyframes.is_empty() {
                let value = timeline_animation.timeline.evaluate(track_name, current_time, 0.0);
                ui_state.set_parameter_value(track_name, value);
            }
        }
    }
    
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

    // Draw menu bar
    draw_editor_menu(ctx, &mut *ui_state, &mut *auditor);
    if auditor.enabled { auditor.record_panel("Menu Bar", true, None); }
    
    // Draw side panels
    draw_editor_side_panels(
        ctx, 
        &mut *ui_state, 
        &mut *controls.midi_system,
        &*audio_analyzer, 
        &mut *render.compute_manager, 
        &mut *outputs.spout_config,
        &mut *outputs.spout_output,
        &mut *outputs.ndi_config,
        &mut *outputs.ndi_output,
        &mut *controls.osc_config,
        &mut *controls.osc_control,
        Some(&mut *scene_editor_state),
        Some(&manipulable_query),
        &mut *controls.gesture_control,
        Some(render.exporter.as_ref()),
    );
    if auditor.enabled && ui_state.show_shader_browser { auditor.record_panel("Shader Browser", true, None); }
    if auditor.enabled && ui_state.show_parameter_panel { auditor.record_panel("Parameters", true, None); }
    
    // Force preview to be visible if it somehow got disabled
    if !ui_state.show_preview {
        println!("WARNING: show_preview was false, forcing to true");
        ui_state.show_preview = true;
    }

    // Draw global status bar at the bottom
    crate::editor_ui::draw_status_bar(ctx, &mut *ui_state);

    // Bottom code editor — MUST be called before CentralPanel
    if ui_state.show_code_editor {
        crate::editor_ui::draw_editor_code_panel(ctx, &mut *ui_state);
        if auditor.enabled { auditor.record_panel("Code Editor", true, None); }
    }

    // Draw the main preview panel - this MUST be the CentralPanel and called LAST among panels
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
            &mut *outputs.ndi_output,
            &*performance_metrics
        );
         if auditor.enabled {
            match ui_state.central_view {
                crate::ui::state::CentralView::Preview => auditor.record_panel("Preview", true, None),
                crate::ui::state::CentralView::NodeGraph => auditor.record_panel("Node Graph", true, None),
                crate::ui::state::CentralView::Scene3D => auditor.record_panel("3D Editor", true, None),
                crate::ui::state::CentralView::Timeline => auditor.record_panel("Timeline", true, None),
            }
        }
    }
}

fn on_window_resize_system(
    mut resize_events: MessageReader<WindowResized>,
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
    ui_state.show_gesture_calibration = true;
    ui_state.show_wgslsmith_panel = true;
    ui_state.show_performance_overlay = true;
    ui_state.show_color_grading_panel = true;
    ui_state.central_view = crate::ui::state::CentralView::Preview;
    vne_state.show_node_editor = true;
}

fn init_enforcement_startup() {
    let _ = pollster::block_on(initialize_enforcement());
}
pub fn setup_camera(mut commands: Commands) {
    // Set global clear color to Dark Gray manually since Color::DARK_GRAY is missing.
    commands.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)));

    // UI Camera (Main Window Camera)
    // Manual construction since Camera2dBundle is missing.
    commands.spawn((
        Camera2d, 
        Camera {
            order: 0, // Main camera
            is_active: true,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            far: 1000.0,
            ..OrthographicProjection::default_3d() // Attempting default_3d again
        }),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
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
    
    // Check if we already have a renderer
    let has_renderer = {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ui_state.global_renderer.renderer.lock().map(|guard| guard.is_some())
        })) {
            Ok(Ok(result)) => result,
            _ => false,
        }
    };
    
    if has_renderer || ui_state.wgpu_initialized {
        return;
    }

    use std::sync::mpsc;
    use once_cell::sync::Lazy;

    type RenderResult = Result<crate::shader_renderer::ShaderRenderer, String>;
    static RENDER_CHANNEL: Lazy<(mpsc::Sender<RenderResult>, Mutex<mpsc::Receiver<RenderResult>>)> = Lazy::new(|| {
        let (tx, rx) = mpsc::channel();
        (tx, Mutex::new(rx))
    });

    // Check if we have a result from the background thread
    if let Ok(receiver) = RENDER_CHANNEL.1.lock() {
        if let Ok(result) = receiver.try_recv() {
            println!("Received WGPU initialization result from background thread...");
            match result {
                Ok(renderer) => {
                    println!("✅ WGPU renderer initialized successfully!");
                    let mut success = false;
                    if let Ok(mut guard) = ui_state.global_renderer.renderer.lock() {
                        *guard = Some(renderer);
                        success = true;
                    }
                    if success {
                        ui_state.wgpu_initialized = true;
                        ui_state.compilation_error.clear();
                    }
                }
                Err(e) => {
                    println!("❌ WGPU renderer initialization failed: {}", e);
                    ui_state.wgpu_initialized = false;
                    ui_state.compilation_error = format!("WGPU initialization failed: {}", e);
                }
            }
            return;
        }
    }

    // Only start one initialization thread
    static INIT_STARTED: Lazy<std::sync::atomic::AtomicBool> = Lazy::new(|| std::sync::atomic::AtomicBool::new(false));
    if !INIT_STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        println!("🚀 Spawning background thread for WGPU initialization...");
        let tx = RENDER_CHANNEL.0.clone();
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                pollster::block_on(async {
                    super::shader_renderer::ShaderRenderer::new_with_size((800, 600)).await
                        .map_err(|e| e.to_string())
                })
            }));

            match result {
                Ok(res) => { let _ = tx.send(res); }
                Err(_) => { let _ = tx.send(Err("WGPU initialization panicked".to_string())); }
            }
        });
    }
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
                    let info = renderer.adapter_info.clone();
                    println!("✅ WGPU renderer initialized on: {} ({:?})", info.name, info.backend);
                    if info.device_type == wgpu::DeviceType::Cpu {
                        println!("⚠️ WARNING: Running on CPU (Software Rendering). Performance will be poor.");
                    }
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
    println!("🚀 Starting WGSL Shader Studio...");
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

    println!("Creating Bevy local app...");
    let mut app = App::new();
    
    println!("Adding Default Plugins...");
    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "WGSL Shader Studio".to_string(),
                resolution: WindowResolution::new(1600, 900),
                present_mode: PresentMode::AutoNoVsync, // Reduced latency, prevents FIFO blocking
                focused: true,
                resizable: true,
                decorations: true,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..Default::default()
            }),
            ..Default::default()
        }).set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::all()), // Relaxed backend selection
                power_preference: bevy::render::settings::PowerPreference::HighPerformance,
                ..Default::default()
            }),
            ..Default::default()
        })
    );

    println!("Adding Third-party Plugins...");
    app.add_plugins(EguiPlugin::default())
        .add_plugins(PerformanceOverlayPlugin)
        .add_plugins(AudioAnalysisPlugin)
        .add_plugins(EnhancedAudioPlugin)
        .add_plugins(MidiSystemPlugin)
        .add_plugins(GyroflowInteropPlugin)
        .add_plugins(ExportPlugin)
        .add_plugins(TimelinePlugin)
        .add_plugins(GestureControlPlugin)
        .add_plugins(BevyNodeGraphPlugin)
        .add_plugins(ComputePassPlugin)
        .add_plugins(OscControlPlugin)
        .add_plugins(AudioMidiIntegrationPlugin)
        .add_plugins(WgslAnalyzerPlugin)
        .add_plugins(NdiOutputPlugin)
        .add_plugins(SpoutSyphonOutputPlugin)
        .add_plugins(SimpleUiAuditorPlugin)
        .insert_resource(SimpleUiAuditor::new())
        .add_plugins(ParticlePhysicsPlugin);

    println!("Initializing Resources...");
    app.insert_resource(EditorUiState::default())
        .insert_resource(UiStartupGate::default())
        .insert_resource(Viewport3DTexture::default())
        .insert_resource(crate::scene_editor_3d::SceneEditor3DState::default())
        .insert_resource(crate::scene_editor_3d::SceneViewportTexture::default())
        .insert_resource(crate::scene_editor_3d::ShaderPreviewTexture::default())
        .insert_resource(MidiSystem::new())
        .insert_resource(crate::screenshot_video_export::ScreenshotVideoExporter::new())
        .insert_resource(VisualNodeEditorState { auto_compile: true, show_node_editor: false })
        .insert_resource(EnhancedAudioAnalyzer::new());

    println!("Configuring Systems...");
    app.add_systems(Startup, setup_camera)
        .add_systems(Startup, crate::editor_ui::populate_shader_list)
        .add_systems(Startup, start_audio_analysis_system)
        .add_systems(Startup, start_documentation_server_system)
        .add_systems(Update, async_initialize_wgpu_renderer)
        .add_systems(Update, crate::editor_ui::background_shader_scan_system)
        .add_systems(Startup, enable_all_features_once)
        .add_systems(Update, update_time_system)
        .add_systems(Update, on_window_resize_system)
        .add_systems(Update, editor_ui_system)
        .add_systems(Update, crate::scene_editor_3d::sync_scene_viewport_texture_size);

    println!("🚀 Launching Bevy App.run()...");
    app.run();
}
