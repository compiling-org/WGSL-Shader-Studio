use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::editor_ui::EditorUiState;

pub fn draw_code_editor(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::Window::new("Code Editor")
        .open(&mut ui_state.show_code_editor)
        .default_width(800.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            ui.heading("WGSL Shader Code Editor");
            
            // Tab selection for different code editor views
            ui.horizontal(|ui| {
                ui.selectable_value(&mut ui_state.code_editor_tab, crate::editor_ui::CodeEditorTab::Editor, "Editor");
                ui.selectable_value(&mut ui_state.code_editor_tab, crate::editor_ui::CodeEditorTab::AI, "WGSLSmith AI");
                ui.selectable_value(&mut ui_state.code_editor_tab, crate::editor_ui::CodeEditorTab::Diagnostics, "Diagnostics");
                ui.selectable_value(&mut ui_state.code_editor_tab, crate::editor_ui::CodeEditorTab::Analyzer, "Analyzer");
            });
            
            ui.separator();
            
            match ui_state.code_editor_tab {
                crate::editor_ui::CodeEditorTab::Editor => {
                    draw_wgsl_editor(ui, ui_state);
                },
                crate::editor_ui::CodeEditorTab::AI => {
                    draw_wgsl_smith_ai(ui, ui_state);
                },
                crate::editor_ui::CodeEditorTab::Diagnostics => {
                    draw_diagnostics_tab(ui, ui_state);
                },
                crate::editor_ui::CodeEditorTab::Analyzer => {
                    draw_analyzer_tab(ui, ui_state);
                },
            }
        });
}

fn draw_wgsl_editor(ui: &mut egui::Ui, ui_state: &mut EditorUiState) {
    ui.horizontal(|ui| {
        if ui.button("Load Shader").clicked() {
            load_shader_file(ui_state);
        }
        
        if ui.button("Save Shader").clicked() {
            save_shader_file(ui_state);
        }
        
        if ui.button("Save Draft").clicked() {
            save_draft_wgsl_to_assets(ui_state);
        }
        
        ui.separator();
        
        if ui.button("Apply to Preview").clicked() {
            ui_state.apply_requested = true;
        }
        
        ui.checkbox(&mut ui_state.auto_apply, "Auto Apply");
    });
    
    ui.separator();
    
    // Text editor area
    let response = egui::ScrollArea::vertical()
        .id_source("shader_code_editor")
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut ui_state.draft_code)
                    .font(egui::TextStyle::Monospace) // Use monospace font
                    .code_editor()
                    .desired_rows(20)
                    .desired_width(f32::INFINITY)
            )
        });
    
    // Check if text was changed
    if response.inner.changed() {
        ui_state.code_changed = true;
        // In a real implementation, you might want to trigger auto-compilation here
    }
    
    // Show status
    ui.horizontal(|ui| {
        ui.label(format!("Characters: {}", ui_state.draft_code.len()));
        ui.separator();
        let lines: usize = ui_state.draft_code.lines().count();
        ui.label(format!("Lines: {}", lines));
        
        if ui_state.code_changed {
            ui.colored_label(egui::Color32::YELLOW, "• CHANGED");
        }
        
        if ui_state.apply_requested {
            ui.colored_label(egui::Color32::GREEN, "✓ APPLIED");
        }
    });
}

fn draw_wgsl_smith_ai(ui: &mut egui::Ui, ui_state: &mut EditorUiState) {
    ui.heading("WGSLSmith AI Assistant");
    
    ui.label("Describe the shader effect you want to create:");
    
    ui.add(egui::TextEdit::multiline(&mut ui_state.wgsl_smith_prompt)
        .desired_rows(3)
        .hint_text("e.g., 'Create a plasma effect with animated colors'"));
    
    ui.horizontal(|ui| {
        if ui.button("Generate Shader").clicked() {
            generate_shader_with_ai(ui_state);
        }
        
        if ui.button("Insert to Editor").clicked() {
            ui_state.draft_code = ui_state.wgsl_smith_generated.clone();
            ui_state.code_changed = true;
        }
    });
    
    ui.separator();
    ui.label("Generated Shader Code:");
    
    egui::ScrollArea::vertical()
        .id_source("wgsl_smith_output")
        .show(ui, |ui| {
            ui.add(egui::TextEdit::multiline(&mut ui_state.wgsl_smith_generated)
                .desired_rows(15)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .interactive(false)); // Read-only display
        });
    
    ui.separator();
    ui.label(&ui_state.wgsl_smith_status);
}

fn draw_diagnostics_tab(ui: &mut egui::Ui, ui_state: &mut EditorUiState) {
    ui.heading("Shader Diagnostics");
    
    if ui.button("Run Diagnostics").clicked() {
        crate::editor_ui::run_wgsl_diagnostics(ui_state);
    }
    
    if ui.button("Clear Diagnostics").clicked() {
        ui_state.diagnostics_messages.clear();
    }
    
    ui.separator();
    
    if ui_state.diagnostics_messages.is_empty() {
        ui.label("No diagnostics to show. Run diagnostics to analyze your shader code.");
    } else {
        egui::ScrollArea::vertical()
            .id_source("diagnostics_output")
            .show(ui, |ui| {
                for diagnostic in &ui_state.diagnostics_messages {
                    match diagnostic.severity {
                        crate::diagnostics_panel::DiagnosticSeverity::Error => {
                            ui.colored_label(egui::Color32::RED, format!("❌ ERROR: {}", diagnostic.message));
                        },
                        crate::diagnostics_panel::DiagnosticSeverity::Warning => {
                            ui.colored_label(egui::Color32::YELLOW, format!("⚠️ WARNING: {}", diagnostic.message));
                        },
                        crate::diagnostics_panel::DiagnosticSeverity::Info => {
                            ui.colored_label(egui::Color32::LIGHT_BLUE, format!("ℹ️ INFO: {}", diagnostic.message));
                        },
                        crate::diagnostics_panel::DiagnosticSeverity::Hint => {
                            ui.colored_label(egui::Color32::LIGHT_GREEN, format!("💡 HINT: {}", diagnostic.message));
                        },
                    }
                    
                    if let Some(line_num) = diagnostic.line_number {
                        ui.label(format!("  Line: {}", line_num));
                    }
                }
            });
    }
}

fn draw_analyzer_tab(ui: &mut egui::Ui, ui_state: &mut EditorUiState) {
    ui.heading("WGSL Code Analyzer");
    
    ui.horizontal(|ui| {
        if ui.button("Run AST Analysis").clicked() {
            run_ast_analysis(ui_state);
        }
        
        if ui.button("Run Validation").clicked() {
            run_shader_validation(ui_state);
        }
        
        if ui.button("Transpile to GLSL").clicked() {
            transpile_to_glsl(ui_state);
        }
        
        if ui.button("Transpile to HLSL").clicked() {
            transpile_to_hlsl(ui_state);
        }
    });
    
    ui.separator();
    
    // AST analysis results
    ui.collapsing("AST Analysis", |ui| {
        if ui_state.ast_ok {
            ui.colored_label(egui::Color32::GREEN, "✓ AST Parse Successful");
        } else {
            ui.colored_label(egui::Color32::RED, "✗ AST Parse Failed");
            ui.label(&ui_state.ast_error);
        }
    });
    
    // Validation results
    ui.collapsing("Validation", |ui| {
        if ui_state.validator_ok {
            ui.colored_label(egui::Color32::GREEN, "✓ Validation Successful");
        } else {
            ui.colored_label(egui::Color32::RED, "✗ Validation Failed");
            ui.label(&ui_state.validator_error);
        }
    });
    
    // Transpilation results
    ui.collapsing("Transpilation", |ui| {
        if !ui_state.transpiled_glsl.is_empty() {
            ui.label("GLSL Output:");
            ui.add(egui::TextEdit::multiline(&mut ui_state.transpiled_glsl)
                .desired_rows(10)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .interactive(false));
        } else {
            ui.label("Run transpilation to see output");
        }
        
        if !ui_state.transpiler_error.is_empty() {
            ui.colored_label(egui::Color32::RED, format!("Transpiler Error: {}", ui_state.transpiler_error));
        }
    });
}

fn generate_shader_with_ai(ui_state: &mut EditorUiState) {
    ui_state.wgsl_smith_status = "Generating shader...".to_string();
    
    // Simulate AI generation (in a real implementation, this would call an actual AI service)
    ui_state.wgsl_smith_generated = crate::editor_ui::generate_shader_with_wgsl_smith(&ui_state.wgsl_smith_prompt);
    
    ui_state.wgsl_smith_status = "Shader generated successfully!".to_string();
}

fn load_shader_file(ui_state: &mut EditorUiState) {
    // In a real implementation, this would open a file dialog
    println!("Loading shader file...");
    // For now, we'll just simulate loading
}

fn save_shader_file(ui_state: &mut EditorUiState) {
    // In a real implementation, this would open a save dialog
    println!("Saving shader file...");
    // For now, we'll just simulate saving
}

fn save_draft_wgsl_to_assets(ui_state: &mut EditorUiState) {
    // In a real implementation, this would save the current draft to the assets directory
    println!("Saving draft WGSL to assets...");
    // For now, we'll just simulate saving
}

fn run_ast_analysis(ui_state: &mut EditorUiState) {
    // Simulate AST analysis
    #[cfg(feature = "naga_integration")] {
        let result = crate::wgsl_ast_parser::WgslAstParser::parse(&ui_state.draft_code);
        match result {
            Ok(_) => {
                ui_state.ast_ok = true;
                ui_state.ast_error = String::new();
            },
            Err(e) => {
                ui_state.ast_ok = false;
                ui_state.ast_error = e.to_string();
            }
        }
    }
    #[cfg(not(feature = "naga_integration"))] {
        ui_state.ast_ok = false;
        ui_state.ast_error = "Naga integration feature is disabled".to_string();
    }
}

fn run_shader_validation(ui_state: &mut EditorUiState) {
    // Simulate shader validation
    #[cfg(feature = "naga_integration")] {
        let result = crate::shader_transpiler::ShaderValidator::validate(&ui_state.draft_code);
        match result {
            Ok(_) => {
                ui_state.validator_ok = true;
                ui_state.validator_error = String::new();
            },
            Err(e) => {
                ui_state.validator_ok = false;
                ui_state.validator_error = e.to_string();
            }
        }
    }
    #[cfg(not(feature = "naga_integration"))] {
        ui_state.validator_ok = false;
        ui_state.validator_error = "Naga integration feature is disabled".to_string();
    }
}

fn transpile_to_glsl(ui_state: &mut EditorUiState) {
    #[cfg(feature = "naga_integration")] {
        let transpiler = crate::shader_transpiler::MultiFormatTranspiler::new();
        let mut options = crate::shader_transpiler::TranspilerOptions::default();
        options.source_language = crate::shader_transpiler::ShaderLanguage::Wgsl;
        options.target_language = crate::shader_transpiler::ShaderLanguage::Glsl;
        
        match transpiler.transpile(&ui_state.draft_code, &options) {
            Ok(result) => {
                ui_state.transpiled_glsl = result.source_code;
                ui_state.transpiler_error = String::new();
            },
            Err(e) => {
                ui_state.transpiled_glsl = String::new();
                ui_state.transpiler_error = e.to_string();
            }
        }
    }
    #[cfg(not(feature = "naga_integration"))] {
        ui_state.transpiled_glsl = String::new();
        ui_state.transpiler_error = "Naga integration feature is disabled".to_string();
    }
}

fn transpile_to_hlsl(ui_state: &mut EditorUiState) {
    #[cfg(feature = "naga_integration")] {
        let transpiler = crate::shader_transpiler::MultiFormatTranspiler::new();
        let mut options = crate::shader_transpiler::TranspilerOptions::default();
        options.source_language = crate::shader_transpiler::ShaderLanguage::Wgsl;
        options.target_language = crate::shader_transpiler::ShaderLanguage::Hlsl;
        
        match transpiler.transpile(&ui_state.draft_code, &options) {
            Ok(result) => {
                ui_state.transpiled_glsl = result.source_code; // Note: using same field for display
                ui_state.transpiler_error = String::new();
            },
            Err(e) => {
                ui_state.transpiled_glsl = String::new();
                ui_state.transpiler_error = e.to_string();
            }
        }
    }
    #[cfg(not(feature = "naga_integration"))] {
        ui_state.transpiled_glsl = String::new();
        ui_state.transpiler_error = "Naga integration feature is disabled".to_string();
    }
}