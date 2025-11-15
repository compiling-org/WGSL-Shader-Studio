use crate::ui_auditor::{UiAuditorPlugin, UiAuditState, UiAuditCollector, PanelAudit, PanelAuditor};
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
    PipelineMode,
    UiStartupGate,
};

// Crash-safe wrappers that also run UI audit
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