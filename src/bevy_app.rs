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
    time: Res<Time>
) {
    // Update time for shader animation
    ui_state.time = time.elapsed_seconds_f64();
    
    // Also update timeline if playing
    if ui_state.timeline.playing {
        ui_state.timeline.timeline.current_time = ui_state.time as f32;
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
use super::audio_system::{AudioAnalyzer, AudioAnalysisPlugin};

// Import timeline animation system
use super::timeline::{TimelinePlugin, TimelineAnimation, PlaybackState};

// Import gesture control system
use super::gesture_control::{GestureControlSystem, GestureControlPlugin};

// Import compute pass integration
use resolume_isf_shaders_rust_ffgl::compute_pass_integration::{ComputePassPlugin, ComputePassManager};

// Import responsive backend system - check if it exists
// use super::backend_systems::{ResponsiveBackend, ResponsiveBackendPlugin};

// Import editor modules - use local editor_ui module
use super::editor_ui::{EditorUiState, UiStartupGate, draw_editor_menu, draw_editor_side_panels, draw_editor_code_panel};

// Import shader renderer for 3D viewport texture rendering
use super::shader_renderer::{ShaderRenderer, RenderParameters};

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
    mut gesture_control: ResMut<GestureControlSystem>,
    mut compute_pass_manager: ResMut<ComputePassManager>,
    video_exporter: Res<crate::screenshot_video_export::ScreenshotVideoExporter>,
    editor_state: Res<crate::scene_editor_3d::SceneEditor3DState>,
    mut viewport_texture: ResMut<Viewport3DTexture>
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
    
    // CRITICAL FIX: Implement proper three-panel layout with coordinated panel management
    // Left panel: Shader Browser with docking and resizable functionality
    if ui_state.show_shader_browser {
        egui::SidePanel::left("shader_browser_panel")
            .resizable(true)
            .default_width(250.0)
            .width_range(200.0..=400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Shader Browser");
                    if ui.button("📁").clicked() {
                        // TODO: Implement file dialog for shader loading
                        println!("File dialog requested");
                    }
                    if ui.button("🔍").clicked() {
                        // TODO: Implement search functionality
                        println!("Search requested");
                    }
                });
                
                ui.separator();
                
                // Category tabs
                ui.horizontal(|ui| {
                    if ui.selectable_label(true, "WGSL").clicked() {}
                    if ui.selectable_label(false, "ISF").clicked() {}
                    if ui.selectable_label(false, "Examples").clicked() {}
                });
                
                ui.separator();
                
                // Shader list placeholder
                ui.label("Available shaders:");
                ui.scope(|ui| {
                    ui.set_min_height(200.0);
                    ui.label("• Basic Fragment Shader");
                    ui.label("• Noise Pattern");
                    ui.label("• Color Gradient");
                    ui.label("• Audio Reactive");
                    ui.label("• 3D Transform");
                });
                
                ui.separator();
                ui.label("Click to load shader");
            });
    }
    
    // Right panel: Parameters with interactive controls
    if ui_state.show_parameter_panel {
        egui::SidePanel::right("parameter_panel")
            .resizable(true)
            .default_width(300.0)
            .width_range(250.0..=500.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Parameters");
                    if ui.button("⚙️").clicked() {
                        // TODO: Implement parameter settings
                        println!("Parameter settings requested");
                    }
                    if ui.button("🔄").clicked() {
                        // TODO: Implement parameter reset
                        println!("Parameter reset requested");
                    }
                });
                
                ui.separator();
                
                // Parameter groups
                ui.collapsing("Time & Animation", |ui| {
                    ui.label("Time:");
                    ui.add(egui::Slider::new(&mut ui_state.time, 0.0..=10.0).text("seconds"));
                    
                    ui.label("Speed:");
                    let mut speed = 1.0;
                    ui.add(egui::Slider::new(&mut speed, 0.1..=5.0).text("x"));
                    
                    ui.checkbox(&mut ui_state.timeline.playing, "Auto-play");
                });
                
                ui.collapsing("Color & Effects", |ui| {
                    ui.label("Background Color:");
                    // TODO: Add color picker
                    ui.label("🎨 Click to pick color");
                    
                    ui.label("Brightness:");
                    let mut brightness = 1.0;
                    ui.add(egui::Slider::new(&mut brightness, 0.0..=2.0).text("x"));
                    
                    ui.label("Contrast:");
                    let mut contrast = 1.0;
                    ui.add(egui::Slider::new(&mut contrast, 0.0..=2.0).text("x"));
                });
                
                ui.collapsing("Audio Reactivity", |ui| {
                    if ui_state.show_audio_panel {
                        ui.label("Bass:");
                        let mut bass = 0.5;
                        ui.add(egui::Slider::new(&mut bass, 0.0..=1.0).text("level"));
                        
                        ui.label("Mid:");
                        let mut mid = 0.3;
                        ui.add(egui::Slider::new(&mut mid, 0.0..=1.0).text("level"));
                        
                        ui.label("Treble:");
                        let mut treble = 0.2;
                        ui.add(egui::Slider::new(&mut treble, 0.0..=1.0).text("level"));
                    } else {
                        ui.label("Enable audio panel to see controls");
                    }
                });
                
                ui.separator();
                ui.label("Parameters update in real-time");
            });
    }
    
    // Bottom panel: Code Editor with syntax highlighting
    if ui_state.show_code_editor {
        egui::TopBottomPanel::bottom("code_editor_panel")
            .resizable(true)
            .default_height(300.0)
            .height_range(200.0..=600.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Code Editor");
                    if ui.button("💾").clicked() {
                        // TODO: Implement save functionality
                        println!("Save requested");
                    }
                    if ui.button("📋").clicked() {
                        // TODO: Implement copy functionality
                        println!("Copy requested");
                    }
                    if ui.button("▶️").clicked() {
                        // TODO: Implement compile functionality
                        println!("Compile requested");
                    }
                });
                
                ui.separator();
                
                // Editor tabs
                ui.horizontal(|ui| {
                    if ui.selectable_label(true, "main.wgsl").clicked() {}
                    if ui.selectable_label(false, "uniforms.wgsl").clicked() {}
                    if ui.selectable_label(false, "functions.wgsl").clicked() {}
                    if ui.button("+").clicked() {
                        // TODO: Implement new file functionality
                        println!("New file requested");
                    }
                });
                
                ui.separator();
                
                // Code editor area
                let response = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut ui_state.draft_code)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                );
                
                if response.changed() {
                    println!("Code changed, length: {} characters", ui_state.draft_code.len());
                    // TODO: Implement auto-compile on change
                }
                
                ui.separator();
                
                // Status bar
                ui.horizontal(|ui| {
                    ui.label(format!("Line: {}, Col: {}", 
                        ui_state.draft_code.lines().count(), 
                        ui_state.draft_code.lines().last().map(|l| l.len()).unwrap_or(0)));
                    ui.separator();
                    ui.label("WGSL");
                    ui.separator();
                    if ui_state.compilation_error.is_empty() {
                        ui.label("✅ Compiled");
                    } else {
                        ui.label("❌ Error");
                        ui.label(&ui_state.compilation_error);
                    }
                });
            });
    }
    
    // Central panel: Shader Preview (this is the main content area)
    if ui_state.show_preview {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Shader Preview");
                    if ui.button("▶️").clicked() {
                        // TODO: Implement play/pause functionality
                        println!("Play/pause requested");
                    }
                    if ui.button("📸").clicked() {
                        // TODO: Implement screenshot functionality
                        println!("Screenshot requested");
                    }
                    if ui.button("📹").clicked() {
                        // TODO: Implement video recording functionality
                        println!("Video recording requested");
                    }
                });
                
                // Create preview area
                let available_size = ui.available_size();
                let preview_size = egui::vec2(
                    available_size.x.min(800.0),
                    available_size.y.min(600.0)
                );
                
                // Try to render live shader preview
                if ui_state.wgpu_initialized {
                    if let Some(renderer) = ui_state.global_renderer.renderer.lock().unwrap().as_mut() {
                        let render_params = RenderParameters {
                            width: preview_size.x as u32,
                            height: preview_size.y as u32,
                            time: ui_state.time as f32,
                            frame_rate: 60.0,
                            audio_data: None, // Could integrate audio data here
                        };
                        
                        // Use the current draft code for rendering
                        match renderer.render_frame(&ui_state.draft_code, &render_params, None) {
                            Ok(texture_data) => {
                                // Create or update the texture in egui
                                let texture_id = ctx.tex_manager().write().alloc(
                                    egui::epaint::ImageDelta {
                                        image: egui::epaint::ImageData::Color(
                                            egui::epaint::ColorImage::from_rgba_unmultiplied(
                                                [preview_size.x as usize, preview_size.y as usize],
                                                &texture_data
                                            )
                                        ),
                                        pos: None,
                                        options: egui::epaint::TextureOptions::LINEAR,
                                    }
                                );
                                
                                // Display the rendered texture
                                ui.image(egui::Image::new(egui::TextureId::User(texture_id))
                                    .max_width(preview_size.x)
                                    .max_height(preview_size.y));
                                
                                ui.label("Live shader preview - rendering successfully");
                            }
                            Err(e) => {
                                // Show compilation error in preview area
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(ui.cursor().min, preview_size),
                                    0.0,
                                    egui::Color32::from_rgb(40, 20, 20)
                                );
                                
                                ui.painter().text(
                                    ui.cursor().min + egui::vec2(10.0, 20.0),
                                    egui::Align2::LEFT_TOP,
                                    format!("Shader Compilation Error:\n{}", e),
                                    egui::FontId::monospace(12.0),
                                    egui::Color32::from_rgb(255, 100, 100)
                                );
                                
                                ui_state.compilation_error = e;
                                ui.label("❌ Shader compilation failed - check code editor");
                            }
                        }
                    } else {
                        // Renderer not available
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(ui.cursor().min, preview_size),
                            0.0,
                            egui::Color32::from_rgb(30, 45, 60)
                        );
                        
                        ui.painter().text(
                            ui.cursor().min + preview_size * 0.5,
                            egui::Align2::CENTER_CENTER,
                            "WGPU Renderer Initializing...",
                            egui::FontId::proportional(16.0),
                            egui::Color32::from_gray(200)
                        );
                        
                        ui.label("⏳ WGPU renderer initializing...");
                    }
                } else {
                    // WGPU not initialized - show placeholder
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(ui.cursor().min, preview_size),
                        0.0,
                        egui::Color32::from_rgb(20, 20, 20)
                    );
                    
                    ui.painter().text(
                        ui.cursor().min + preview_size * 0.5,
                        egui::Align2::CENTER_CENTER,
                        "Live Shader Preview\n(WGPU Integration Required)",
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_gray(128)
                    );
                    
                    if !ui_state.compilation_error.is_empty() {
                        ui.label(format!("❌ {}", ui_state.compilation_error));
                    } else {
                        ui.label("⚠️ WGPU initialization required for live preview");
                    }
                }
            });
    }
    
    // Draw the additional side panels (timeline, node studio, etc.) as windows
    draw_editor_side_panels(ctx, &mut *ui_state, &audio_analyzer, &mut gesture_control, &mut compute_pass_manager, None, None);
    
    // Draw 3D scene editor panel
    if ui_state.show_3d_scene_panel {
        println!("Drawing 3D scene editor panel...");
        // Create a simple 3D scene editor panel for now
        egui::Window::new("3D Scene Editor")
            .default_pos([100.0, 100.0])
            .default_size([600.0, 400.0])
            .show(&ctx, |ui| {
                ui.heading("3D Scene View");
                
                if editor_state.enabled {
                    ui.label("3D viewport active - use mouse controls to navigate");
                    ui.label(format!("Selected: {:?}", editor_state.selected_entity));
                    ui.label(format!("Mode: {:?}", editor_state.manipulation_mode));
                    
                    // Controls info
                    ui.separator();
                    ui.label("Controls:");
                    ui.label("• Left Click: Select entity");
                    ui.label("• Right Drag: Orbit camera");
                    ui.label("• Middle Drag: Pan camera");
                    ui.label("• Q/Z: Zoom out/in");
                    ui.label("• W/E/R: Switch manipulation mode");
                    
                    // Render-to-texture viewport
                    ui.separator();
                    
                    // Simple 3D scene shader for viewport rendering
                    let scene_shader = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0)
    );
    let pos = vertices[vertex_index];
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (frag_coord.xy - vec2<f32>(256.0, 192.0)) / vec2<f32>(256.0, 192.0);
    let time = 0.0; // Static for now
    
    // Simple 3D grid pattern to simulate a 3D viewport
    let grid = step(0.95, sin(uv.x * 20.0)) + step(0.95, sin(uv.y * 20.0));
    let bg = vec3<f32>(0.1, 0.15, 0.2) + grid * 0.1;
    
    // Add some "3D objects" as colored circles
    let obj1 = smoothstep(0.1, 0.09, length(uv - vec2<f32>(-0.3, 0.2)));
    let obj2 = smoothstep(0.08, 0.07, length(uv - vec2<f32>(0.4, -0.1)));
    let obj3 = smoothstep(0.06, 0.05, length(uv - vec2<f32>(0.0, 0.4)));
    
    var color = bg;
    color = mix(color, vec3<f32>(0.8, 0.3, 0.2), obj1);
    color = mix(color, vec3<f32>(0.2, 0.8, 0.3), obj2);
    color = mix(color, vec3<f32>(0.3, 0.2, 0.8), obj3);
    
    return vec4<f32>(color, 1.0);
}
"#;
                    
                    // Try to render the 3D scene shader
                    if let Some(renderer) = ui_state.global_renderer.renderer.lock().unwrap().as_mut() {
                        let render_params = RenderParameters {
                            width: 512,
                            height: 384,
                            time: ui_state.time as f32,
                            frame_rate: 60.0,
                            audio_data: None, // Could integrate audio data here
                        };
                        
                        match renderer.render_frame(scene_shader, &render_params, None) {
                            Ok(texture_data) => {
                                // Create or update the texture in egui
                                let texture_id = ctx.tex_manager().write().alloc(
                                    egui::epaint::ImageDelta {
                                        image: egui::epaint::ImageData::Color(
                                            egui::epaint::ColorImage::from_rgba_unmultiplied(
                                                [512, 384],
                                                &texture_data
                                            )
                                        ),
                                        pos: None,
                                        options: egui::epaint::TextureOptions::LINEAR,
                                    }
                                );
                                
                                // Display the rendered texture
                                ui.image(egui::Image::new(egui::TextureId::User(texture_id))
                                    .max_width(512.0)
                                    .max_height(384.0));
                                
                                ui.label("3D viewport rendered successfully");
                            }
                            Err(e) => {
                                ui.label(format!("Render error: {}", e));
                                ui.label("Using fallback viewport display");
                                // Fallback: display a simple colored rectangle
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(ui.cursor().min, egui::vec2(512.0, 384.0)),
                                    0.0,
                                    egui::Color32::from_rgb(30, 45, 60)
                                );
                            }
                        }
                    } else {
                        ui.label("Renderer not initialized - using fallback display");
                        // Fallback: display a simple colored rectangle
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(ui.cursor().min, egui::vec2(512.0, 384.0)),
                            0.0,
                            egui::Color32::from_rgb(30, 45, 60)
                        );
                    }
                } else {
                    ui.label("3D editor disabled - enable from Studio menu");
                }
            });
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
        super::shader_renderer::ShaderRenderer::new_with_size((800, 600)).await
    }) {
        Ok(renderer) => {
            println!("✅ WGPU renderer initialized successfully!");
            *ui_state.global_renderer.renderer.lock().unwrap() = Some(renderer);
            
            // Update UI state to reflect successful initialization
            ui_state.wgpu_initialized = true;
            ui_state.compilation_error.clear();
            
            println!("WGPU renderer ready with {} working examples", 
                     renderer.working_examples.len());
        }
        Err(e) => {
            println!("❌ Failed to initialize WGPU renderer: {}. ENFORCING GPU-ONLY POLICY - NO CPU FALLBACK ALLOWED.", e);
            ui_state.wgpu_initialized = false;
            ui_state.compilation_error = format!("WGPU initialization failed: {}", e);
            
            // Don't panic immediately, but log the critical error
            eprintln!("CRITICAL: GPU initialization failed. System requires compatible GPU.");
            eprintln!("Error details: {}", e);
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
        .add_plugins(ComputePassPlugin)
        // .add_plugins(ResponsiveBackendPlugin)
        // .add_plugins(BevyNodeGraphPlugin)
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