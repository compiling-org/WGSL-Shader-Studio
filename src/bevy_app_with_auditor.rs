use crate::ui_auditor::{UiAuditorPlugin, UiAuditState, UiAuditCollector, PanelAudit};
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

/// Crash-safe wrappers that also run UI audit
fn safe_editor_menu(
    mut egui_ctx: EguiContexts, 
    mut ui_state: ResMut<EditorUiState>,
    mut collector: ResMut<UiAuditCollector>,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        collector.clear(); // Start fresh each frame
        if let Ok(ctx) = egui_ctx.ctx_mut() {
            // Audited menu
            let mut audit = PanelAudit::new("Menu Bar");
            egui::TopBottomPanel::top("menu_bar").show(&ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    // File menu
                    egui::menu::menu(ui, "File", |ui| {
                        if ui.button("New").clicked() {
                            ui_state.wgsl_code = String::from("// New shader\n");
                            ui.close_menu();
                            audit.mark_real();
                        }
                        if ui.button("Open...").clicked() {
                            audit.mark_real();
                        }
                        if ui.button("Save").clicked() {
                            audit.mark_real();
                        }
                    });
                    
                    // View menu  
                    egui::menu::menu(ui, "View", |ui| {
                        ui.checkbox(&mut ui_state.show_shader_browser, "Shader Browser");
                        ui.checkbox(&mut ui_state.show_parameter_panel, "Parameter Panel");
                        ui.checkbox(&mut ui_state.show_preview, "Preview");
                        ui.checkbox(&mut ui_state.show_code_editor, "Code Editor");
                        ui.checkbox(&mut ui_state.show_node_studio, "Node Studio");
                        ui.checkbox(&mut ui_state.show_timeline, "Timeline");
                        ui.checkbox(&mut ui_state.show_audio_panel, "Audio Panel");
                        audit.mark_real();
                        audit.widget_count += 7;
                    });
                    
                    // Tools menu
                    egui::menu::menu(ui, "Tools", |ui| {
                        if ui.button("Convert Shader").clicked() {
                            ui_state.show_conversion_panel = true;
                            audit.mark_real();
                        }
                    });
                });
            });
            collector.record_panel(audit);
        }
    }));
}

fn safe_editor_side_panels(
    mut egui_ctx: EguiContexts,
    mut ui_state: ResMut<EditorUiState>, 
    mut collector: ResMut<UiAuditCollector>,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Ok(ctx) = egui_ctx.ctx_mut() {
            // Shader Browser Panel
            if ui_state.show_shader_browser {
                let mut audit = PanelAudit::new("Shader Browser");
                egui::SidePanel::left("shader_browser").resizable(true).show(&ctx, |ui| {
                    ui.heading("Shader Browser");
                    
                    if !ui_state.available_shaders_all.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Search:");
                            ui.text_edit_singleline(&mut ui_state.search_query);
                        });
                        
                        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                            let names = if ui_state.show_all_shaders {
                                &ui_state.available_shaders_all
                            } else {
                                &ui_state.available_shaders_compatible
                            };
                            for name in names.iter() {
                                if !ui_state.search_query.is_empty() && !name.to_lowercase().contains(&ui_state.search_query.to_lowercase()) {
                                    continue;
                                }
                                let selected = ui.selectable_label(
                                    ui_state.selected_shader.as_ref().map(|s| s == name).unwrap_or(false), 
                                    name
                                );
                                if selected.clicked() {
                                    ui_state.selected_shader = Some(name.clone());
                                }
                            }
                        });
                        
                        audit.mark_real();
                        audit.widget_count += 3;
                    } else {
                        audit.add_placeholder("No shader list loaded");
                        ui.label("No shaders available");
                    }
                });
                collector.record_panel(audit);
            }

            // Parameter Panel  
            if ui_state.show_parameter_panel {
                let mut audit = PanelAudit::new("Parameter Panel");
                egui::SidePanel::right("parameters").resizable(true).show(&ctx, |ui| {
                    ui.heading("Parameters");
                    
                    if ui_state.selected_shader.is_some() && !ui_state.shader_parameters.is_empty() {
                        for (name, value) in &mut ui_state.shader_parameters {
                            ui.horizontal(|ui| {
                                ui.label(name);
                                ui.drag_value(value);
                            });
                        }
                        audit.mark_real();
                        audit.widget_count += ui_state.shader_parameters.len() * 2;
                    } else {
                        audit.add_placeholder("No shader parameters");
                        ui.label("Select a shader to see parameters");
                    }
                });
                collector.record_panel(audit);
            }
        }
    }));
}

fn safe_editor_code_panel(
    mut egui_ctx: EguiContexts,
    mut ui_state: ResMut<EditorUiState>,
    mut collector: ResMut<UiAuditCollector>,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if let Ok(ctx) = egui_ctx.ctx_mut() {
            let mut audit = PanelAudit::new("Code Editor");
            
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.heading("WGSL Code Editor");
                
                if !ui_state.wgsl_code.is_empty() {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut ui_state.wgsl_code)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(20)
                            .lock_focus(true)
                    );
                    
                    if response.changed() {
                        ui_state.code_changed = true;
                    }
                    
                    audit.mark_real();
                    audit.widget_count += 2;
                } else {
                    audit.add_placeholder("No code loaded");
                    ui.label("Code editor placeholder");
                }
            });
            
            collector.record_panel(audit);
        }
    }));
}

/// System that listens for F12 and triggers audit
fn ui_audit_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut audit_state: ResMut<UiAuditState>,
) {
    if keys.just_pressed(KeyCode::F12) {
        audit_state.trigger_this_frame = true;
    }
}

/// System that prints the audit report when triggered
fn ui_audit_report_system(
    audit_state: Res<UiAuditState>,
    collector: Res<UiAuditCollector>,
) {
    if audit_state.trigger_this_frame && audit_state.enabled {
        collector.print_report();
    }
}

/// Add the auditor systems to the app
pub fn add_ui_auditor_systems(app: &mut App) {
    app.add_systems(Update, ui_audit_keyboard_system)
       .add_systems(Update, ui_audit_report_system.after(ui_audit_keyboard_system));
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
        .add_plugins(UiAuditorPlugin)
        .insert_resource(UiAuditState { enabled: true, trigger_this_frame: false })
        .insert_resource(UiAuditCollector::default())
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
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_renderer)
        .add_systems(Startup, populate_shader_list)
        .add_systems(EguiPrimaryContextPass, safe_editor_menu)
        .add_systems(EguiPrimaryContextPass, safe_editor_side_panels)
        .add_systems(EguiPrimaryContextPass, safe_editor_code_panel)
        .add_systems(EguiPrimaryContextPass, preview_renderer)
        .add_systems(Update, apply_shader_selection)
        .add_systems(Update, watchdog_reset)
        .run();
}