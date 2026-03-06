use bevy_egui::egui;
use std::sync::Arc;
use super::state::{EditorUiState, CodeEditorTab, DiagnosticSeverity};

pub fn draw_editor_code_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    if !ui_state.show_code_editor { return; }
    egui::TopBottomPanel::bottom("code_editor_panel")
        .resizable(false)
        .default_height(240.0)
        .min_height(160.0)
        .max_height(280.0)
        .show(ctx, |ui| {
        ui.horizontal(|ui| {
            let tabs = [
                (CodeEditorTab::Editor, "Editor"),
                (CodeEditorTab::AI, "AI"),
                (CodeEditorTab::Diagnostics, "Diagnostics"),
                (CodeEditorTab::Analyzer, "Analyzer"),
            ];
            for (tab, label) in tabs {
                let sel = ui_state.code_editor_tab == tab;
                if ui.selectable_label(sel, label).clicked() {
                    ui_state.code_editor_tab = tab;
                }
            }
        });
        ui.separator();
        match ui_state.code_editor_tab {
            CodeEditorTab::Editor => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut edit = egui::TextEdit::multiline(&mut ui_state.draft_code)
                        .code_editor()
                        .desired_rows(12)
                        .lock_focus(true)
                        .hint_text("Paste or write WGSL here...");
                    
                    let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| -> Arc<egui::Galley> {
                        crate::editor_ui::highlight_wgsl(ui, text.as_str(), wrap_width)
                    };
                    edit = edit.layouter(&mut layouter);
                    let fixed_height = 180.0;
                    ui.add_sized(egui::vec2(ui.available_width(), fixed_height), edit);
                });
                ui.horizontal(|ui| {
                    if ui.button("Apply to Preview").clicked() {
                        ui_state.apply_requested = true;
                    }
                    ui.checkbox(&mut ui_state.auto_apply, "Auto Apply");
                });
            }
            CodeEditorTab::AI => {
                ui.heading("AI-Assisted Shader Generation");
                ui.horizontal(|ui| {
                    ui.label("Prompt:");
                    ui.text_edit_multiline(&mut ui_state.wgsl_smith_prompt);
                });
                if ui.button("Generate Shader").clicked() {
                    ui_state.wgsl_smith_generated = crate::editor_ui::generate_shader_with_wgsl_smith(&ui_state.wgsl_smith_prompt);
                }
            }
            CodeEditorTab::Diagnostics => {
                ui.heading("Shader Compilation Diagnostics");
                if ui.button("Check Current Shader").clicked() {
                    ui_state.diagnostics_messages = crate::editor_ui::check_wgsl_diagnostics(&ui_state.draft_code);
                }
                egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                    for diagnostic in &ui_state.diagnostics_messages {
                        let color = match diagnostic.severity {
                            DiagnosticSeverity::Error => egui::Color32::RED,
                            DiagnosticSeverity::Warning => egui::Color32::YELLOW,
                            DiagnosticSeverity::Info => egui::Color32::BLUE,
                        };
                        ui.colored_label(color, &diagnostic.message);
                    }
                });
            }
            CodeEditorTab::Analyzer => {
                ui.heading("WGSL Code Analysis");
                ui.label("Analyzer tools integrated via editor_ui");
            }
        }
    });
}
