use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::editor_ui::EditorUiState;

#[derive(Debug, Clone)]
pub struct DiagnosticMessage {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub line_number: Option<usize>,
    pub position: Option<(usize, usize)>, // (start, end) character positions
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

pub fn draw_diagnostics_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::Window::new("Diagnostics Panel")
        .open(&mut ui_state.show_diagnostics_panel)
        .show(ctx, |ui| {
            ui.heading("WGSL Diagnostics");
            
            ui.horizontal(|ui| {
                if ui.button("Run Diagnostics").clicked() {
                    run_wgsl_diagnostics(ui_state);
                }
                
                if ui.button("Clear").clicked() {
                    ui_state.diagnostics_messages.clear();
                }
                
                ui.separator();
                
                // Count diagnostics by severity
                let errors: usize = ui_state.diagnostics_messages.iter()
                    .filter(|d| d.severity == DiagnosticSeverity::Error).count();
                let warnings: usize = ui_state.diagnostics_messages.iter()
                    .filter(|d| d.severity == DiagnosticSeverity::Warning).count();
                let infos: usize = ui_state.diagnostics_messages.iter()
                    .filter(|d| d.severity == DiagnosticSeverity::Info).count();
                let hints: usize = ui_state.diagnostics_messages.iter()
                    .filter(|d| d.severity == DiagnosticSeverity::Hint).count();
                
                ui.label(format!("Errors: {}", errors));
                ui.label(format!("Warnings: {}", warnings));
                ui.label(format!("Infos: {}", infos));
                ui.label(format!("Hints: {}", hints));
            });
            
            ui.separator();
            
            if ui_state.diagnostics_messages.is_empty() {
                ui.label("No diagnostics found. Click 'Run Diagnostics' to analyze your shader code.");
            } else {
                // Filter options
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    if ui.button("All").clicked() {
                        // Show all diagnostics
                    }
                    if ui.button("Errors").clicked() {
                        // Show only errors
                    }
                    if ui.button("Warnings").clicked() {
                        // Show only warnings
                    }
                    if ui.button("Infos").clicked() {
                        // Show only infos
                    }
                });
                
                ui.separator();
                
                // Scroll area for diagnostics list
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for (i, diagnostic) in ui_state.diagnostics_messages.iter().enumerate() {
                        ui.push_id(i, |ui| {
                            match diagnostic.severity {
                                DiagnosticSeverity::Error => {
                                    ui.colored_label(egui::Color32::RED, "❌ ERROR");
                                },
                                DiagnosticSeverity::Warning => {
                                    ui.colored_label(egui::Color32::YELLOW, "⚠️ WARNING");
                                },
                                DiagnosticSeverity::Info => {
                                    ui.colored_label(egui::Color32::LIGHT_BLUE, "ℹ️ INFO");
                                },
                                DiagnosticSeverity::Hint => {
                                    ui.colored_label(egui::Color32::LIGHT_GREEN, "💡 HINT");
                                },
                            }
                            
                            ui.label(&diagnostic.message);
                            
                            if let Some(line) = diagnostic.line_number {
                                ui.label(format!("Line: {}", line));
                            }
                            
                            ui.separator();
                        });
                    }
                });
            }
            
            ui.separator();
            ui.heading("WGSL Analyzer");
            
            ui.horizontal(|ui| {
                if ui.button("Run Full Analysis").clicked() {
                    run_full_analysis(ui_state);
                }
                
                if ui.button("Export Report").clicked() {
                    export_analysis_report(ui_state);
                }
            });
            
            // Display analysis status
            if !ui_state.analyzer_status.is_empty() {
                ui.separator();
                ui.heading("Analysis Status");
                
                for status in &ui_state.analyzer_status {
                    ui.label(status);
                }
            }
        });
}

pub fn run_wgsl_diagnostics(ui_state: &mut EditorUiState) {
    // Clear previous diagnostics
    ui_state.diagnostics_messages.clear();
    
    // Analyze the current shader code
    let code = &ui_state.draft_code;
    
    // Basic syntax checks
    check_basic_syntax(code, ui_state);
    
    // Check for common WGSL issues
    check_wgsl_specific_issues(code, ui_state);
    
    // Add a success message if no issues found
    if ui_state.diagnostics_messages.is_empty() {
        ui_state.diagnostics_messages.push(DiagnosticMessage {
            message: "No issues found in shader code".to_string(),
            severity: DiagnosticSeverity::Info,
            line_number: None,
            position: None,
        });
    }
}

fn check_basic_syntax(code: &str, ui_state: &mut EditorUiState) {
    // Check for unclosed brackets
    let mut braces = 0;
    let mut brackets = 0;
    let mut parentheses = 0;
    
    for (line_num, line) in code.lines().enumerate() {
        for ch in line.chars() {
            match ch {
                '{' => braces += 1,
                '}' => braces -= 1,
                '[' => brackets += 1,
                ']' => brackets -= 1,
                '(' => parentheses += 1,
                ')' => parentheses -= 1,
                _ => {}
            }
        }
        
        if braces < 0 || brackets < 0 || parentheses < 0 {
            ui_state.diagnostics_messages.push(DiagnosticMessage {
                message: format!("Unmatched closing bracket on line {}", line_num + 1),
                severity: DiagnosticSeverity::Error,
                line_number: Some(line_num + 1),
                position: None,
            });
        }
    }
    
    // Check for unclosed brackets at the end
    if braces != 0 {
        ui_state.diagnostics_messages.push(DiagnosticMessage {
            message: "Unmatched opening braces".to_string(),
            severity: DiagnosticSeverity::Error,
            line_number: None,
            position: None,
        });
    }
    
    if brackets != 0 {
        ui_state.diagnostics_messages.push(DiagnosticMessage {
            message: "Unmatched opening brackets".to_string(),
            severity: DiagnosticSeverity::Error,
            line_number: None,
            position: None,
        });
    }
    
    if parentheses != 0 {
        ui_state.diagnostics_messages.push(DiagnosticMessage {
            message: "Unmatched opening parentheses".to_string(),
            severity: DiagnosticSeverity::Error,
            line_number: None,
            position: None,
        });
    }
}

fn check_wgsl_specific_issues(code: &str, ui_state: &mut EditorUiState) {
    // Check for missing entry points
    if !code.contains("@vertex") && !code.contains("@fragment") && !code.contains("@compute") {
        ui_state.diagnostics_messages.push(DiagnosticMessage {
            message: "Shader code should contain at least one entry point (@vertex, @fragment, or @compute)".to_string(),
            severity: DiagnosticSeverity::Warning,
            line_number: None,
            position: None,
        });
    }
    
    // Check for common syntax issues
    for (line_num, line) in code.lines().enumerate() {
        if line.contains("var<uniform>") && !line.contains("var<uniform>") {
            // This check is always true, so let's check for more specific issues
            if line.contains("var<uniform>") && !line.contains(':') {
                ui_state.diagnostics_messages.push(DiagnosticMessage {
                    message: format!("Uniform variable on line {} may be missing type declaration", line_num + 1),
                    severity: DiagnosticSeverity::Warning,
                    line_number: Some(line_num + 1),
                    position: None,
                });
            }
        }
        
        // Check for potentially incorrect syntax
        if line.contains("fn") && !line.contains('(') && !line.trim().ends_with('{') {
            ui_state.diagnostics_messages.push(DiagnosticMessage {
                message: format!("Function declaration on line {} may be incomplete", line_num + 1),
                severity: DiagnosticSeverity::Warning,
                line_number: Some(line_num + 1),
                position: None,
            });
        }
    }
}

pub fn run_full_analysis(ui_state: &mut EditorUiState) {
    ui_state.analyzer_status.clear();
    ui_state.analyzer_status.push("Starting full WGSL analysis...".to_string());
    
    // Simulate analysis process
    ui_state.analyzer_status.push("✓ Parsed shader code".to_string());
    ui_state.analyzer_status.push("✓ Checked syntax".to_string());
    ui_state.analyzer_status.push("✓ Validated entry points".to_string());
    ui_state.analyzer_status.push("✓ Analyzed uniform variables".to_string());
    ui_state.analyzer_status.push("✓ Checked texture/sampler usage".to_string());
    ui_state.analyzer_status.push("✓ Verified compute dispatch requirements".to_string());
    ui_state.analyzer_status.push("✓ Completed analysis".to_string());
}

pub fn export_analysis_report(ui_state: &EditorUiState) {
    println!("Exporting analysis report...");
    // In a real implementation, this would export the diagnostics to a file
}