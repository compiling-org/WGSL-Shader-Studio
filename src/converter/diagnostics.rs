use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

/// Severity levels for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Info => "info",
            DiagnosticSeverity::Hint => "hint",
        }
    }
    
    pub fn to_char(&self) -> char {
        match self {
            DiagnosticSeverity::Error => 'E',
            DiagnosticSeverity::Warning => 'W',
            DiagnosticSeverity::Info => 'I',
            DiagnosticSeverity::Hint => 'H',
        }
    }
}

/// A single diagnostic message with location and suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub suggestions: Vec<Suggestion>,
    pub related_information: Vec<RelatedInformation>,
    pub quick_fix_available: bool,
}

impl Diagnostic {
    /// Set the file path for this diagnostic
    pub fn with_file_path(mut self, file_path: String) -> Self {
        self.file_path = file_path;
        self
    }
}

/// A suggested fix for a diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub title: String,
    pub replacement: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

/// Related information for a diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedInformation {
    pub message: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
}

/// Collection of diagnostics for a conversion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    pub diagnostics: Vec<Diagnostic>,
    pub has_errors: bool,
    pub has_warnings: bool,
    pub summary: DiagnosticSummary,
}

/// Summary of diagnostics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub hint_count: usize,
    pub total_count: usize,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            has_errors: false,
            has_warnings: false,
            summary: DiagnosticSummary::default(),
        }
    }
    
    /// Add a diagnostic
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        match diagnostic.severity {
            DiagnosticSeverity::Error => {
                self.has_errors = true;
                self.summary.error_count += 1;
            }
            DiagnosticSeverity::Warning => {
                self.has_warnings = true;
                self.summary.warning_count += 1;
            }
            DiagnosticSeverity::Info => {
                self.summary.info_count += 1;
            }
            DiagnosticSeverity::Hint => {
                self.summary.hint_count += 1;
            }
        }
        
        self.summary.total_count += 1;
        self.diagnostics.push(diagnostic);
    }
    
    /// Add an error diagnostic
    pub fn add_error(
        &mut self,
        code: &str,
        message: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) {
        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: code.to_string(),
            message: message.to_string(),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: Vec::new(),
            related_information: Vec::new(),
            quick_fix_available: false,
        };
        
        self.add_diagnostic(diagnostic);
    }
    
    /// Add a warning diagnostic
    pub fn add_warning(
        &mut self,
        code: &str,
        message: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) {
        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: code.to_string(),
            message: message.to_string(),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: Vec::new(),
            related_information: Vec::new(),
            quick_fix_available: false,
        };
        
        self.add_diagnostic(diagnostic);
    }
    
    /// Add an info diagnostic
    pub fn add_info(
        &mut self,
        code: &str,
        message: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) {
        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Info,
            code: code.to_string(),
            message: message.to_string(),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: Vec::new(),
            related_information: Vec::new(),
            quick_fix_available: false,
        };
        
        self.add_diagnostic(diagnostic);
    }
    
    /// Add a hint diagnostic
    pub fn add_hint(
        &mut self,
        code: &str,
        message: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) {
        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Hint,
            code: code.to_string(),
            message: message.to_string(),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: Vec::new(),
            related_information: Vec::new(),
            quick_fix_available: false,
        };
        
        self.add_diagnostic(diagnostic);
    }
    
    /// Add a suggestion to an existing diagnostic
    pub fn add_suggestion_to_diagnostic(
        &mut self,
        index: usize,
        title: &str,
        replacement: &str,
        line: usize,
        column: usize,
        length: usize,
    ) -> Result<()> {
        if let Some(diagnostic) = self.diagnostics.get_mut(index) {
            let suggestion = Suggestion {
                title: title.to_string(),
                replacement: replacement.to_string(),
                line,
                column,
                length,
            };
            
            diagnostic.suggestions.push(suggestion);
            diagnostic.quick_fix_available = true;
            Ok(())
        } else {
            anyhow::bail!("Diagnostic index {} out of range", index)
        }
    }
    
    /// Add related information to an existing diagnostic
    pub fn add_related_information_to_diagnostic(
        &mut self,
        index: usize,
        message: &str,
        file_path: &str,
        line: usize,
        column: usize,
    ) -> Result<()> {
        if let Some(diagnostic) = self.diagnostics.get_mut(index) {
            let info = RelatedInformation {
                message: message.to_string(),
                file_path: file_path.to_string(),
                line,
                column,
            };
            
            diagnostic.related_information.push(info);
            Ok(())
        } else {
            anyhow::bail!("Diagnostic index {} out of range", index)
        }
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.has_errors
    }
    
    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.has_warnings
    }
    
    /// Get all diagnostics
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// Get diagnostics by severity
    pub fn get_diagnostics_by_severity(&self, severity: DiagnosticSeverity) -> Vec<&Diagnostic> {
        self.diagnostics.iter()
            .filter(|d| d.severity == severity)
            .collect()
    }
    
    /// Get diagnostics by file path
    pub fn get_diagnostics_by_file(&self, file_path: &str) -> Vec<&Diagnostic> {
        self.diagnostics.iter()
            .filter(|d| d.file_path == file_path)
            .collect()
    }
    
    /// Get diagnostics by line number
    pub fn get_diagnostics_by_line(&self, line: usize) -> Vec<&Diagnostic> {
        self.diagnostics.iter()
            .filter(|d| d.line == line)
            .collect()
    }
    
    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.has_errors = false;
        self.has_warnings = false;
        self.summary = DiagnosticSummary::default();
    }
    
    /// Merge another diagnostics into this one
    pub fn merge(&mut self, other: Diagnostics) {
        for diagnostic in other.diagnostics {
            self.add_diagnostic(diagnostic);
        }
    }
    
    /// Format diagnostics as a string
    pub fn format(&self) -> String {
        if self.diagnostics.is_empty() {
            return "No diagnostics found.".to_string();
        }
        
        let mut result = String::new();
        
        // Add summary
        result.push_str(&format!(
            "Found {} diagnostic(s): {} error(s), {} warning(s), {} info(s), {} hint(s)\n\n",
            self.summary.total_count,
            self.summary.error_count,
            self.summary.warning_count,
            self.summary.info_count,
            self.summary.hint_count
        ));
        
        // Add individual diagnostics
        for diagnostic in &self.diagnostics {
            result.push_str(&format!(
                "[{}] {}:{}:{} - {}\n",
                diagnostic.severity.to_char(),
                diagnostic.file_path,
                diagnostic.line,
                diagnostic.column,
                diagnostic.message
            ));
            
            if !diagnostic.suggestions.is_empty() {
                result.push_str("  Suggestions:\n");
                for suggestion in &diagnostic.suggestions {
                    result.push_str(&format!(
                        "    - {} (replace with: {})\n",
                        suggestion.title,
                        suggestion.replacement
                    ));
                }
            }
            
            if !diagnostic.related_information.is_empty() {
                result.push_str("  Related information:\n");
                for info in &diagnostic.related_information {
                    result.push_str(&format!(
                        "    - {} ({}:{}:{})\n",
                        info.message,
                        info.file_path,
                        info.line,
                        info.column
                    ));
                }
            }
            
            result.push('\n');
        }
        
        result
    }
    
    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .context("Failed to serialize diagnostics to JSON")
    }
    
    /// Create from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .context("Failed to deserialize diagnostics from JSON")
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for common diagnostic patterns
pub mod diagnostic_helpers {
    use super::*;
    
    /// Create a syntax error diagnostic
    pub fn syntax_error(
        message: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) -> Diagnostic {
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: "SYNTAX_ERROR".to_string(),
            message: message.to_string(),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: Vec::new(),
            related_information: Vec::new(),
            quick_fix_available: false,
        }
    }
    
    /// Create a type mismatch error
    pub fn type_mismatch(
        expected: &str,
        found: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) -> Diagnostic {
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: "TYPE_MISMATCH".to_string(),
            message: format!("Expected type '{}', found '{}'", expected, found),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: vec![
                Suggestion {
                    title: format!("Change to '{}'", expected),
                    replacement: expected.to_string(),
                    line,
                    column,
                    length,
                }
            ],
            related_information: Vec::new(),
            quick_fix_available: true,
        }
    }
    
    /// Create an undefined variable error
    pub fn undefined_variable(
        name: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) -> Diagnostic {
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: "UNDEFINED_VARIABLE".to_string(),
            message: format!("Undefined variable '{}'", name),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: vec![],
            related_information: Vec::new(),
            quick_fix_available: false,
        }
    }
    
    /// Create a deprecated feature warning
    pub fn deprecated_feature(
        feature: &str,
        replacement: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) -> Diagnostic {
        Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: "DEPRECATED_FEATURE".to_string(),
            message: format!("Deprecated feature '{}' - use '{}' instead", feature, replacement),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: vec![
                Suggestion {
                    title: format!("Replace with '{}'", replacement),
                    replacement: replacement.to_string(),
                    line,
                    column,
                    length,
                }
            ],
            related_information: Vec::new(),
            quick_fix_available: true,
        }
    }
    
    /// Create a performance hint
    pub fn performance_hint(
        message: &str,
        file_path: &str,
        line: usize,
        column: usize,
        length: usize,
    ) -> Diagnostic {
        Diagnostic {
            severity: DiagnosticSeverity::Hint,
            code: "PERFORMANCE_HINT".to_string(),
            message: message.to_string(),
            file_path: file_path.to_string(),
            line,
            column,
            length,
            suggestions: Vec::new(),
            related_information: Vec::new(),
            quick_fix_available: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diagnostic_helpers::*;

    #[test]
    fn test_diagnostics_creation() {
        let diagnostics = Diagnostics::new();
        assert!(!diagnostics.has_errors());
        assert!(!diagnostics.has_warnings());
        assert_eq!(diagnostics.get_diagnostics().len(), 0);
    }
    
    #[test]
    fn test_adding_diagnostics() {
        let mut diagnostics = Diagnostics::new();
        
        diagnostics.add_error(
            "TEST_ERROR",
            "Test error message",
            "test.glsl",
            10,
            5,
            3,
        );
        
        diagnostics.add_warning(
            "TEST_WARNING",
            "Test warning message",
            "test.glsl",
            15,
            8,
            5,
        );
        
        assert!(diagnostics.has_errors());
        assert!(diagnostics.has_warnings());
        assert_eq!(diagnostics.get_diagnostics().len(), 2);
        assert_eq!(diagnostics.summary.error_count, 1);
        assert_eq!(diagnostics.summary.warning_count, 1);
    }
    
    #[test]
    fn test_diagnostic_helpers() {
        let mut diagnostics = Diagnostics::new();
        
        let error = syntax_error("Unexpected token", "test.glsl", 1, 1, 1);
        diagnostics.add_diagnostic(error);
        
        let warning = deprecated_feature("old_feature", "new_feature", "test.glsl", 2, 1, 10);
        diagnostics.add_diagnostic(warning);
        
        let hint = performance_hint("Consider using a more efficient algorithm", "test.glsl", 3, 1, 5);
        diagnostics.add_diagnostic(hint);
        
        assert_eq!(diagnostics.get_diagnostics().len(), 3);
        assert_eq!(diagnostics.summary.error_count, 1);
        assert_eq!(diagnostics.summary.warning_count, 1);
        assert_eq!(diagnostics.summary.hint_count, 1);
    }
    
    #[test]
    fn test_formatting() {
        let mut diagnostics = Diagnostics::new();
        
        diagnostics.add_error(
            "TEST_ERROR",
            "Test error message",
            "test.glsl",
            10,
            5,
            3,
        );
        
        let formatted = diagnostics.format();
        assert!(formatted.contains("Found 1 diagnostic(s): 1 error(s), 0 warning(s)"));
        assert!(formatted.contains("[E] test.glsl:10:5"));
        assert!(formatted.contains("Test error message"));
    }
    
    #[test]
    fn test_json_serialization() {
        let mut diagnostics = Diagnostics::new();
        
        diagnostics.add_error(
            "TEST_ERROR",
            "Test error message",
            "test.glsl",
            10,
            5,
            3,
        );
        
        let json = diagnostics.to_json().unwrap();
        let deserialized = Diagnostics::from_json(&json).unwrap();
        
        assert_eq!(deserialized.summary.total_count, 1);
        assert_eq!(deserialized.summary.error_count, 1);
        assert!(deserialized.has_errors());
    }
}