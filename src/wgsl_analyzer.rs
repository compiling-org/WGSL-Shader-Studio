use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

/// WGSL analyzer integration for inline diagnostics
/// Standalone tool and integrated app component

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub line: u32,
    pub column: u32,
    pub length: u32,
    pub source: DiagnosticSource,
    pub related_info: Vec<RelatedInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelatedInfo {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSource {
    WgslAnalyzer,
    Naga,
    Custom,
}

/// WGSL analyzer resource for the app
#[derive(Resource, Default)]
pub struct WgslAnalyzer {
    pub diagnostics: HashMap<String, Vec<Diagnostic>>,
    pub analysis_enabled: bool,
    pub real_time_analysis: bool,
}

impl WgslAnalyzer {
    pub fn new() -> Self {
        Self {
            diagnostics: HashMap::new(),
            analysis_enabled: true,
            real_time_analysis: true,
        }
    }

    pub fn analyze_shader(&mut self, file_path: &str, source: &str) {
        if !self.analysis_enabled {
            return;
        }

        let mut diagnostics = Vec::new();

        // Empty shader check
        if source.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "EMPTY_SHADER".to_string(),
                message: "Shader code is empty".to_string(),
                severity: DiagnosticSeverity::Warning,
                line: 1,
                column: 1,
                length: 1,
                source: DiagnosticSource::Custom,
                related_info: vec![],
            });
        }

        // Missing main function check
        if !source.contains("fn main(") && !source.contains("@vertex") && !source.contains("@fragment") {
            diagnostics.push(Diagnostic {
                code: "MISSING_ENTRY_POINT".to_string(),
                message: "Missing entry point function (@vertex, @fragment, or @compute)".to_string(),
                severity: DiagnosticSeverity::Error,
                line: 1,
                column: 1,
                length: 1,
                source: DiagnosticSource::Custom,
                related_info: vec![],
            });
        }

        // Syntax validation with line-by-line analysis
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Missing semicolon detection
            if !trimmed.ends_with(';') 
                && !trimmed.ends_with('{')
                && !trimmed.ends_with('}')
                && !trimmed.starts_with("fn")
                && !trimmed.starts_with("struct")
                && !trimmed.starts_with("let")
                && !trimmed.contains("if")
                && !trimmed.contains("else")
                && !trimmed.contains("return")
                && !trimmed.contains("@")
                && !trimmed.starts_with("var")
                && !trimmed.starts_with("const") {
                
                diagnostics.push(Diagnostic {
                    code: "MISSING_SEMICOLON".to_string(),
                    message: "Statement might be missing semicolon".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    line: (i + 1) as u32,
                    column: line.len() as u32,
                    length: 1,
                    source: DiagnosticSource::Custom,
                    related_info: vec![],
                });
            }

            // Invalid variable declarations
            if trimmed.starts_with("var") && !trimmed.contains(":") {
                diagnostics.push(Diagnostic {
                    code: "INVALID_VAR_DECLARATION".to_string(),
                    message: "Variable declaration missing type annotation".to_string(),
                    severity: DiagnosticSeverity::Error,
                    line: (i + 1) as u32,
                    column: 1,
                    length: 3,
                    source: DiagnosticSource::Custom,
                    related_info: vec![
                        RelatedInfo {
                            message: "Add type annotation like: var x: f32 = 0.0;".to_string(),
                            line: (i + 1) as u32,
                            column: line.len() as u32,
                        }
                    ],
                });
            }

            // Deprecated syntax warnings
            if trimmed.contains("[[") || trimmed.contains("]]") {
                diagnostics.push(Diagnostic {
                    code: "DEPRECATED_ATTRIBUTE_SYNTAX".to_string(),
                    message: "Deprecated attribute syntax [[...]], use @... instead".to_string(),
                    severity: DiagnosticSeverity::Warning,
                    line: (i + 1) as u32,
                    column: line.find("[").unwrap_or(0) as u32 + 1,
                    length: 2,
                    source: DiagnosticSource::Custom,
                    related_info: vec![],
                });
            }

            // Texture sampling validation
            if trimmed.contains("textureSample") && !trimmed.contains("texture_sample") {
                // Check for common texture sampling issues
                if !trimmed.contains(",") {
                    diagnostics.push(Diagnostic {
                        code: "INVALID_TEXTURE_SAMPLE".to_string(),
                        message: "textureSample requires texture and sampler parameters".to_string(),
                        severity: DiagnosticSeverity::Error,
                        line: (i + 1) as u32,
                        column: line.find("textureSample").unwrap_or(0) as u32 + 1,
                        length: 13,
                        source: DiagnosticSource::Custom,
                        related_info: vec![],
                    });
                }
            }

            // Type mismatch detection (basic)
            if trimmed.contains("let") && trimmed.contains("=") {
                if trimmed.contains("let") && !trimmed.contains(":") {
                    diagnostics.push(Diagnostic {
                        code: "IMPLICIT_TYPE_INFERENCE".to_string(),
                        message: "Consider adding explicit type annotation for clarity".to_string(),
                        severity: DiagnosticSeverity::Hint,
                        line: (i + 1) as u32,
                        column: line.find("let").unwrap_or(0) as u32 + 1,
                        length: 3,
                        source: DiagnosticSource::Custom,
                        related_info: vec![],
                    });
                }
            }
        }

        // Advanced validation using Naga when available
        #[cfg(feature = "naga_integration")]
        {
            if let Ok(module) = naga::front::wgsl::parse_str(source) {
                let mut validator = naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga::valid::Capabilities::all(),
                );
                
                match validator.validate(&module) {
                    Ok(_) => {
                        // Validation passed, no additional errors
                    }
                    Err(validation_error) => {
                        // Convert validation error to diagnostic
                        diagnostics.push(Diagnostic {
                            code: "NAGA_VALIDATION_ERROR".to_string(),
                            message: format!("Validation error: {:?}", validation_error),
                            severity: DiagnosticSeverity::Error,
                            line: 1,
                            column: 1,
                            length: 1,
                            source: DiagnosticSource::Naga,
                            related_info: vec![],
                        });
                    }
                }
            }
        }

        self.diagnostics.insert(file_path.to_string(), diagnostics);
    }

    pub fn get_diagnostics(&self, file_path: &str) -> &[Diagnostic] {
        self.diagnostics.get(file_path).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn clear_diagnostics(&mut self, file_path: &str) {
        self.diagnostics.remove(file_path);
    }

    pub fn clear_all_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn toggle_analysis(&mut self) {
        self.analysis_enabled = !self.analysis_enabled;
        if !self.analysis_enabled {
            self.clear_all_diagnostics();
        }
    }

    pub fn set_real_time_analysis(&mut self, enabled: bool) {
        self.real_time_analysis = enabled;
    }
}

/// Standalone WGSL analyzer tool
pub struct StandaloneWgslAnalyzer {
    analyzer: Arc<Mutex<WgslAnalyzer>>,
}

impl StandaloneWgslAnalyzer {
    pub fn new() -> Self {
        Self {
            analyzer: Arc::new(Mutex::new(WgslAnalyzer::new())),
        }
    }

    pub fn analyze_file(&self, file_path: &str, source: &str) -> Vec<Diagnostic> {
        let mut analyzer = self.analyzer.lock().unwrap();
        analyzer.analyze_shader(file_path, source);
        analyzer.get_diagnostics(file_path).to_vec()
    }

    pub fn analyze_files(&self, files: Vec<(&str, &str)>) -> HashMap<String, Vec<Diagnostic>> {
        let mut results = HashMap::new();
        let mut analyzer = self.analyzer.lock().unwrap();
        
        for (file_path, source) in files {
            analyzer.analyze_shader(file_path, source);
            if let Some(diagnostics) = analyzer.diagnostics.get(file_path) {
                results.insert(file_path.to_string(), diagnostics.clone());
            }
        }
        
        results
    }

    pub fn format_diagnostics(&self, diagnostics: &[Diagnostic]) -> String {
        let mut output = String::new();
        
        for diagnostic in diagnostics {
            let severity_str = match diagnostic.severity {
                DiagnosticSeverity::Error => "ERROR",
                DiagnosticSeverity::Warning => "WARNING",
                DiagnosticSeverity::Info => "INFO",
                DiagnosticSeverity::Hint => "HINT",
            };

            let source_str = match diagnostic.source {
                DiagnosticSource::WgslAnalyzer => "WGSL-Analyzer",
                DiagnosticSource::Naga => "Naga",
                DiagnosticSource::Custom => "Custom",
            };

            output.push_str(&format!(
                "{}: {} ({}:{}): {} [{}]\n",
                severity_str,
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
                diagnostic.message,
                source_str
            ));

            for related in &diagnostic.related_info {
                output.push_str(&format!(
                    "  → {} ({}:{})\n",
                    related.message,
                    related.line,
                    related.column
                ));
            }
        }
        
        output
    }
}

/// UI component for rendering diagnostics
pub struct DiagnosticRenderer;

impl DiagnosticRenderer {
    pub fn render_diagnostics_panel(ui: &mut egui::Ui, diagnostics: &[Diagnostic]) {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for diagnostic in diagnostics {
                    ui.horizontal(|ui| {
                        // Severity icon
                        match diagnostic.severity {
                            DiagnosticSeverity::Error => {
                                ui.colored(egui::Color32::RED, "●");
                            }
                            DiagnosticSeverity::Warning => {
                                ui.colored(egui::Color32::YELLOW, "▲");
                            }
                            DiagnosticSeverity::Info => {
                                ui.colored(egui::Color32::BLUE, "ℹ");
                            }
                            DiagnosticSeverity::Hint => {
                                ui.colored(egui::Color32::GRAY, "💡");
                            }
                        }
                        
                        // Diagnostic message
                        ui.label(&diagnostic.message);
                        
                        // Position info
                        ui.weak(format!(
                            " ({}:{})",
                            diagnostic.line,
                            diagnostic.column
                        ));
                        
                        // Source
                        match diagnostic.source {
                            DiagnosticSource::WgslAnalyzer => ui.weak("[WGSL-Analyzer]"),
                            DiagnosticSource::Naga => ui.weak("[Naga]"),
                            DiagnosticSource::Custom => ui.weak("[Custom]"),
                        }
                    });
                    
                    // Show related information
                    for related in &diagnostic.related_info {
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            ui.weak("→");
                            ui.label(&related.message);
                            ui.weak(format!(
                                " ({}:{})",
                                related.line,
                                related.column
                            ));
                        });
                    }
                    
                    ui.separator();
                }
            });
    }
    
    pub fn render_inline_diagnostics(
        ui: &mut egui::Ui,
        diagnostics: &[Diagnostic],
        line_number: usize,
    ) {
        let line_diagnostics: Vec<&Diagnostic> = diagnostics
            .iter()
            .filter(|d| d.line as usize == line_number)
            .collect();
            
        if !line_diagnostics.is_empty() {
            ui.horizontal(|ui| {
                for diagnostic in line_diagnostics {
                    match diagnostic.severity {
                        DiagnosticSeverity::Error => {
                            ui.colored(egui::Color32::RED, "●");
                        }
                        DiagnosticSeverity::Warning => {
                            ui.colored(egui::Color32::YELLOW, "▲");
                        }
                        DiagnosticSeverity::Info => {
                            ui.colored(egui::Color32::BLUE, "ℹ");
                        }
                        DiagnosticSeverity::Hint => {
                            ui.colored(egui::Color32::GRAY, "💡");
                        }
                    }
                    
                    if ui.add(egui::Button::new("").sense(egui::Sense::hover())).hovered() {
                        egui::show_tooltip(ui.ctx(), egui::Id::new(&diagnostic.code), |ui| {
                            ui.label(&diagnostic.message);
                            ui.weak(format!("Code: {}", diagnostic.code));
                            ui.weak(format!("Line: {}:{}", diagnostic.line, diagnostic.column));
                            match diagnostic.source {
                                DiagnosticSource::WgslAnalyzer => ui.weak("Source: WGSL-Analyzer"),
                                DiagnosticSource::Naga => ui.weak("Source: Naga"),
                                DiagnosticSource::Custom => ui.weak("Source: Custom"),
                            }
                            
                            if !diagnostic.related_info.is_empty() {
                                ui.separator();
                                ui.label("Related:");
                                for related in &diagnostic.related_info {
                                    ui.weak(format!("  → {}", related.message));
                                }
                            }
                        });
                    }
                }
            });
        }
    }
    
    pub fn render_diagnostic_summary(ui: &mut egui::Ui, diagnostics: &[Diagnostic]) {
        let error_count = diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Error)).count();
        let warning_count = diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Warning)).count();
        let info_count = diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Info)).count();
        let hint_count = diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Hint)).count();
        
        ui.horizontal(|ui| {
            if error_count > 0 {
                ui.colored(egui::Color32::RED, format!("● {} errors", error_count));
            }
            if warning_count > 0 {
                ui.colored(egui::Color32::YELLOW, format!("▲ {} warnings", warning_count));
            }
            if info_count > 0 {
                ui.colored(egui::Color32::BLUE, format!("ℹ {} info", info_count));
            }
            if hint_count > 0 {
                ui.colored(egui::Color32::GRAY, format!("💡 {} hints", hint_count));
            }
            
            if diagnostics.is_empty() {
                ui.colored(egui::Color32::GREEN, "✓ No issues");
            }
        });
    }
}

/// System to analyze shaders in real-time
pub fn analyze_shader_system(
    mut analyzer: ResMut<WgslAnalyzer>,
    editor_state: Res<crate::editor_ui::EditorUiState>,
) {
    if analyzer.real_time_analysis && editor_state.code_changed {
        if let Some(current_file) = &editor_state.current_file {
            analyzer.analyze_shader(current_file, &editor_state.code);
        }
    }
}

/// Plugin for WGSL analyzer integration
pub struct WgslAnalyzerPlugin;

impl Plugin for WgslAnalyzerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WgslAnalyzer>()
            .add_systems(Update, analyze_shader_system);
    }
}

/// CLI interface for standalone analyzer
pub fn run_standalone_analyzer(files: Vec<String>) {
    let analyzer = StandaloneWgslAnalyzer::new();
    
    println!("WGSL Analyzer - Standalone Tool");
    println!("================================");
    
    for file_path in files {
        match std::fs::read_to_string(&file_path) {
            Ok(source) => {
                println!("\nAnalyzing: {}", file_path);
                let diagnostics = analyzer.analyze_file(&file_path, &source);
                
                if diagnostics.is_empty() {
                    println!("✓ No issues found");
                } else {
                    println!("{}", analyzer.format_diagnostics(&diagnostics));
                }
            }
            Err(e) => {
                println!("✗ Error reading file {}: {}", file_path, e);
            }
        }
    }
    
    println!("\nAnalysis complete.");
}