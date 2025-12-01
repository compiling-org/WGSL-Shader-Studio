use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::ops::Range;

/// WGSL analyzer integration for inline diagnostics
/// Based on proven patterns from wgsl-analyzer reference repository

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub range: TextRange,
    pub source: DiagnosticSource,
    pub related: Vec<RelatedDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSource {
    WgslAnalyzer,
    Naga,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelatedDiagnostic {
    pub message: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRange {
    pub start: TextPosition,
    pub end: TextPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineIndex {
    pub newlines: Vec<u32>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut newlines = vec![0];
        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                newlines.push((i + 1) as u32);
            }
        }
        Self { newlines }
    }

    pub fn offset_to_position(&self, offset: usize) -> TextPosition {
        let offset = offset as u32;
        match self.newlines.binary_search(&offset) {
            Ok(line) => TextPosition { line: line as u32, character: 0 },
            Err(line) => {
                let line_start = if line > 0 { self.newlines[line - 1] } else { 0 };
                TextPosition { 
                    line: (line - 1) as u32, 
                    character: offset - line_start 
                }
            }
        }
    }

    pub fn position_to_offset(&self, position: &TextPosition) -> usize {
        if position.line as usize >= self.newlines.len() {
            return self.newlines.last().copied().unwrap_or(0) as usize;
        }
        
        let line_start = self.newlines[position.line as usize] as usize;
        line_start + position.character as usize
    }
}

/// Naga error adapter for consistent diagnostic generation
pub trait NagaErrorAdapter {
    fn to_diagnostics(&self, source: &str) -> Vec<Diagnostic>;
}

/// Adapter for Naga parsing errors
pub struct NagaParseErrorAdapter;

impl NagaErrorAdapter for naga::front::wgsl::ParseError {
    fn to_diagnostics(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let line_index = LineIndex::new(source);
        
        // Extract error information from Naga parse error
        if let Some(labels) = self.labels() {
            for label in labels {
                let range = TextRange {
                    start: line_index.offset_to_position(label.offset()),
                    end: line_index.offset_to_position(label.offset() + label.len()),
                };
                
                diagnostics.push(Diagnostic {
                    code: "WGSL_PARSE_ERROR".to_string(),
                    message: label.text().to_string(),
                    severity: DiagnosticSeverity::Error,
                    range,
                    source: DiagnosticSource::Naga,
                    related: Vec::new(),
                });
            }
        }
        
        diagnostics
    }
}

/// Adapter for Naga validation errors
pub struct NagaValidationErrorAdapter;

impl NagaErrorAdapter for naga::valid::ValidationError {
    fn to_diagnostics(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let line_index = LineIndex::new(source);
        
        // Map validation errors to diagnostics
        let (message, severity) = match self {
            naga::valid::ValidationError::InvalidHandle(_) => 
                ("Invalid handle reference".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidAccess(_) => 
                ("Invalid access operation".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidExpression(_) => 
                ("Invalid expression".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidStatement(_) => 
                ("Invalid statement".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidFunction(_) => 
                ("Invalid function".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidEntryPoint(_) => 
                ("Invalid entry point".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidType(_) => 
                ("Invalid type".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidVariable(_) => 
                ("Invalid variable".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidConstant(_) => 
                ("Invalid constant".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidAtomic(_) => 
                ("Invalid atomic operation".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidBarrier(_) => 
                ("Invalid barrier".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidWorkGroupSize(_) => 
                ("Invalid workgroup size".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidImageFormat(_) => 
                ("Invalid image format".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidSampleType(_) => 
                ("Invalid sample type".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidInterpolation(_) => 
                ("Invalid interpolation".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidStorageFormat(_) => 
                ("Invalid storage format".to_string(), DiagnosticSeverity::Error),
            naga::valid::ValidationError::InvalidConservativeDepth(_) => 
                ("Invalid conservative depth".to_string(), DiagnosticSeverity::Warning),
            naga::valid::ValidationError::InvalidConservativeRaster(_) => 
                ("Invalid conservative raster".to_string(), DiagnosticSeverity::Warning),
            _ => ("Unknown validation error".to_string(), DiagnosticSeverity::Error),
        };
        
        // For now, use a default range since validation errors don't have spans
        // In a real implementation, we'd track the source location
        let range = TextRange {
            start: TextPosition { line: 0, character: 0 },
            end: TextPosition { line: 0, character: 0 },
        };
        
        diagnostics.push(Diagnostic {
            code: "WGSL_VALIDATION_ERROR".to_string(),
            message,
            severity,
            range,
            source: DiagnosticSource::Naga,
            related: Vec::new(),
        });
        
        diagnostics
    }
}

/// WGSL analyzer resource for managing diagnostics
#[derive(Resource, Default)]
pub struct WgslAnalyzer {
    pub diagnostics: HashMap<String, Vec<Diagnostic>>,
    pub analysis_enabled: bool,
}

impl WgslAnalyzer {
    pub fn new() -> Self {
        Self {
            diagnostics: HashMap::new(),
            analysis_enabled: true,
        }
    }

    pub fn analyze_shader(&mut self, file_path: &str, source: &str) {
        if !self.analysis_enabled {
            return;
        }

        let mut diagnostics = Vec::new();

        // Parse the WGSL source
        match naga::front::wgsl::parse_str(source) {
            Ok(module) => {
                // Validate the module
                let mut validator = naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga::valid::Capabilities::all(),
                );
                
                match validator.validate(&module) {
                    Ok(_) => {
                        // No validation errors
                    }
                    Err(validation_error) => {
                        // Convert validation errors to diagnostics
                        let validation_diagnostics = validation_error.to_diagnostics(source);
                        diagnostics.extend(validation_diagnostics);
                    }
                }
            }
            Err(parse_error) => {
                // Convert parse errors to diagnostics
                let parse_diagnostics = parse_error.to_diagnostics(source);
                diagnostics.extend(parse_diagnostics);
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
}

/// System to analyze shaders and update diagnostics
pub fn analyze_shader_system(
    mut analyzer: ResMut<WgslAnalyzer>,
    editor_state: Res<crate::editor_ui::EditorUiState>,
) {
    if editor_state.code_changed {
        if let Some(current_file) = &editor_state.current_file {
            analyzer.analyze_shader(current_file, &editor_state.code);
        }
    }
}

/// UI component for rendering inline diagnostics
pub struct DiagnosticRenderer;

impl DiagnosticRenderer {
    pub fn render_diagnostics(
        ui: &mut egui::Ui,
        diagnostics: &[Diagnostic],
        source: &str,
    ) {
        let line_index = LineIndex::new(source);
        
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
                }
                
                // Diagnostic message
                ui.label(&diagnostic.message);
                
                // Position info
                ui.weak(format!(
                    " ({}:{})",
                    diagnostic.range.start.line + 1,
                    diagnostic.range.start.character + 1
                ));
                
                // Source
                match diagnostic.source {
                    DiagnosticSource::WgslAnalyzer => ui.weak("[WGSL-Analyzer]"),
                    DiagnosticSource::Naga => ui.weak("[Naga]"),
                    DiagnosticSource::Custom => ui.weak("[Custom]"),
                }
            });
            
            // Show related diagnostics
            for related in &diagnostic.related {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.weak("→");
                    ui.label(&related.message);
                    ui.weak(format!(
                        " ({}:{})",
                        related.range.start.line + 1,
                        related.range.start.character + 1
                    ));
                });
            }
        }
    }
    
    pub fn render_inline_diagnostics(
        ui: &mut egui::Ui,
        diagnostics: &[Diagnostic],
        line_number: usize,
    ) {
        let line_diagnostics: Vec<&Diagnostic> = diagnostics
            .iter()
            .filter(|d| d.range.start.line as usize == line_number)
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
                    }
                    
                    if ui.add(egui::Button::new("").sense(egui::Sense::hover())).hovered() {
                        egui::show_tooltip(ui.ctx(), egui::Id::new(&diagnostic.code), |ui| {
                            ui.label(&diagnostic.message);
                            ui.weak(format!("Code: {}", diagnostic.code));
                            match diagnostic.source {
                                DiagnosticSource::WgslAnalyzer => ui.weak("Source: WGSL-Analyzer"),
                                DiagnosticSource::Naga => ui.weak("Source: Naga"),
                                DiagnosticSource::Custom => ui.weak("Source: Custom"),
                            }
                        });
                    }
                }
            });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_index() {
        let text = "line 1\nline 2\nline 3";
        let line_index = LineIndex::new(text);
        
        assert_eq!(line_index.offset_to_position(0).line, 0);
        assert_eq!(line_index.offset_to_position(7).line, 1);
        assert_eq!(line_index.offset_to_position(14).line, 2);
    }

    #[test]
    fn test_diagnostic_creation() {
        let diagnostic = Diagnostic {
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            severity: DiagnosticSeverity::Error,
            range: TextRange {
                start: TextPosition { line: 0, character: 5 },
                end: TextPosition { line: 0, character: 10 },
            },
            source: DiagnosticSource::WgslAnalyzer,
            related: Vec::new(),
        };
        
        assert_eq!(diagnostic.code, "TEST_ERROR");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    }
}