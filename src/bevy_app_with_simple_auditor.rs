use crate::simple_ui_auditor::{SimpleUiAuditorPlugin, SimpleUiAuditor};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::prelude::Camera2d;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::window::{PresentMode, WindowResolution};
use bevy_egui::{
    egui,
    EguiContexts,
    EguiPlugin,
    EguiTextureHandle,
    EguiPrimaryContextPass,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use resolume_isf_shaders_rust_ffgl::shader_renderer::{RenderParameters, ShaderRenderer};
use resolume_isf_shaders_rust_ffgl::audio::AudioMidiSystem;
use resolume_isf_shaders_rust_ffgl::gesture_control::GestureControlSystem;
use resolume_isf_shaders_rust_ffgl::timeline::Timeline;
use resolume_isf_shaders_rust_ffgl::editor_ui::{
    EditorUiState,
    editor_menu,
    editor_side_panels,
    populate_shader_list,
    editor_code_panel,
    apply_shader_selection,
    validate_wgsl_for_mode,
    PipelineMode,
    UiStartupGate,
};

// Hint Windows drivers to prefer discrete GPU when available
#[cfg(target_os = "windows")]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 0x00000001;

#[cfg(target_os = "windows")]
#[no_mangle]
pub static AmdPowerXpressRequestHighPerformance: u32 = 0x00000001;

#[derive(Resource)]
struct PreviewState {
    renderer: ShaderRenderer,
    wgsl_code: String,
    width: u32,
    height: u32,
    time: f32,
    frame_rate: f32,
    image_handle: Option<Handle<Image>>, // Bevy GPU texture
    texture_id: Option<egui::TextureId>, // egui user texture id
    last_error: Option<String>,
    recording_enabled: bool,
    frame_count: u64,
}

impl PreviewState {
    fn new(renderer: ShaderRenderer) -> Self {
        Self {
            renderer,
            wgsl_code: String::from(""),
            width: 512,
            height: 512,
            time: 0.0,
            frame_rate: 60.0,
            image_handle: None,
            texture_id: None,
            last_error: None,
            recording_enabled: false,
            frame_count: 0,
        }
    }
}

/// Resource that tracks if the GPU watchdog has fired this frame
#[derive(Resource, Default)]
struct GpuWatchdog {
    fired: bool,
}

// Crash-safe wrappers to prevent early egui context panics while still drawing when ready
fn safe_editor_menu(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        editor_menu(egui_ctx, ui_state);
    }));
}

fn safe_editor_side_panels(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        editor_side_panels(egui_ctx, ui_state);
    }));
}

fn safe_editor_code_panel(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        editor_code_panel(egui_ctx, ui_state);
    }));
}

/// System that audits UI panels and tracks real vs placeholder widgets
fn ui_audit_system(
    mut egui_ctx: EguiContexts,
    mut ui_state: ResMut<EditorUiState>,
    mut auditor: ResMut<SimpleUiAuditor>,
) {
    if let Ok(ctx) = egui_ctx.ctx_mut() {
        // Clear previous audit
        auditor.clear();
        
        // Audit Menu Bar
        if ctx.memory(|mem| mem.area_rect(egui::Id::new("menu_bar")).is_some()) {
            auditor.record_panel("Menu Bar", true, None);
        }
        
        // Audit Shader Browser
        if ui_state.show_shader_browser && !ui_state.available_shaders_all.is_empty() {
            auditor.record_panel("Shader Browser", true, None);
        } else if ui_state.show_shader_browser {
            auditor.record_panel("Shader Browser", false, Some("No shaders loaded".to_string()));
        }
        
        // Audit Parameter Panel
        if ui_state.show_parameter_panel && ui_state.selected_shader.is_some() && !ui_state.shader_parameters.is_empty() {
            auditor.record_panel("Parameter Panel", true, None);
        } else if ui_state.show_parameter_panel {
            auditor.record_panel("Parameter Panel", false, Some("No parameters or shader selected".to_string()));
        }
        
        // Audit Code Editor
        if !ui_state.wgsl_code.is_empty() {
            auditor.record_panel("Code Editor", true, None);
        } else {
            auditor.record_panel("Code Editor", false, Some("No code loaded".to_string()));
        }
        
        // Audit Preview Panel
        if ui_state.show_preview {
            auditor.record_panel("Preview Panel", false, Some("Preview not implemented".to_string()));
        }
        
        // Audit Node Editor
        if ui_state.show_node_studio {
            auditor.record_panel("Node Editor", false, Some("Node editor not implemented".to_string()));
        }
        
        // Audit Timeline
        if ui_state.show_timeline {
            auditor.record_panel("Timeline", false, Some("Timeline not implemented".to_string()));
        }
        
        // Audit Audio Panel
        if ui_state.show_audio_panel {
            auditor.record_panel("Audio Panel", false, Some("Audio panel not implemented".to_string()));
        }
    }
}

/// System that listens for F12 and prints audit report
fn ui_audit_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    auditor: Res<SimpleUiAuditor>,
) {
    if keys.just_pressed(KeyCode::F12) {
        auditor.print_report();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_renderer(mut commands: Commands) {
    // Initialize renderer here if needed
}

fn watchdog_reset(mut watchdog: ResMut<GpuWatchdog>) {
    watchdog.fired = false;
}

fn preview_renderer(
    mut egui_ctx: EguiContexts,
    mut ui_state: ResMut<EditorUiState>,
    mut preview_state: ResMut<PreviewState>,
    time: Res<Time>,
    watchdog: Res<GpuWatchdog>,
) {
    // Preview rendering logic would go here
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
                    resolution: WindowResolution::new(1280, 800),
                    present_mode: PresentMode::AutoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(SimpleUiAuditorPlugin)
        .init_resource::<SimpleUiAuditor>()
        .insert_resource(EditorUiState {
            show_shader_browser: true,
            show_parameter_panel: true,
            show_preview: true,
            show_code_editor: true,
            ..Default::default()
        })
        .insert_non_send_resource(AudioMidiSystem::new())
        .insert_resource(GestureControlSystem::new())
        .insert_resource(Timeline::new())
        .insert_resource(UiStartupGate::default())
        .init_resource::<GpuWatchdog>()
        .init_resource::<PreviewState>()
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_renderer)
        .add_systems(Startup, populate_shader_list)
        .add_systems(EguiPrimaryContextPass, safe_editor_menu)
        .add_systems(EguiPrimaryContextPass, safe_editor_side_panels)
        .add_systems(EguiPrimaryContextPass, safe_editor_code_panel)
        .add_systems(EguiPrimaryContextPass, preview_renderer)
        .add_systems(Update, apply_shader_selection)
        .add_systems(Update, ui_audit_system)
        .add_systems(Update, ui_audit_keyboard_system)
        .add_systems(Update, watchdog_reset)
        .run();
}