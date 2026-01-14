//! WESL (WebGPU Enhanced Shading Language) converter module
//! Provides conversion between WESL and other shader formats

use std::collections::HashMap;
use anyhow::{Result, bail};
use crate::converter::diagnostics::{Diagnostics, Diagnostic, DiagnosticSeverity};

/// WESL to other formats converter
pub struct WESLConverter {
    diagnostics: Diagnostics,
    /// Track imported modules
    imported_modules: HashMap<String, String>,
}

impl WESLConverter {
    pub fn new() -> Self {
        Self {
            diagnostics: Diagnostics::new(),
            imported_modules: HashMap::new(),
        }
    }

    /// Convert WESL source to WGSL
    pub fn convert_to_wgsl(&mut self, wesl_source: &str, file_path: &str) -> Result<String> {
        // Process WESL-specific features and convert to WGSL
        let mut wgsl_code = String::new();
        
        // Add file header
        wgsl_code.push_str(&format!("// Converted from WESL: {}\n", file_path));
        wgsl_code.push_str("// WESL features processed by WESLConverter\n\n");
        
        // Process the WESL source
        let processed_source = self.process_wesl_features(wesl_source, file_path)?;
        
        // Append the processed source
        wgsl_code.push_str(&processed_source);
        
        Ok(wgsl_code)
    }

    /// Process WESL-specific features like imports, conditional compilation, etc.
    fn process_wesl_features(&mut self, source: &str, file_path: &str) -> Result<String> {
        let mut result = String::new();
        let mut lines = source.lines().peekable();
        
        while let Some(line) = lines.next() {
            let trimmed = line.trim();
            
            // Handle WESL import statements
            if trimmed.starts_with("import") || trimmed.starts_with("#import") {
                self.handle_import(line, file_path)?;
                continue; // Skip import line in output, as content is included separately
            }
            // Handle WESL conditional compilation
            else if trimmed.starts_with("#if") || 
                   trimmed.starts_with("#ifdef") || 
                   trimmed.starts_with("#ifndef") {
                self.handle_conditional_compilation(&mut lines, &mut result, file_path)?;
                continue;
            }
            // Handle other WESL-specific directives
            else if trimmed.starts_with("#define") || trimmed.starts_with("#pragma") {
                // Process preprocessor directives
                self.handle_preprocessor_directive(line)?;
                continue; // Skip preprocessor directives in output
            }
            
            // Add regular line to result
            result.push_str(line);
            result.push('\n');
        }
        
        Ok(result)
    }

    /// Handle WESL import statements
    fn handle_import(&mut self, import_line: &str, file_path: &str) -> Result<()> {
        // Extract import path from import statement
        if let Some(import_path) = self.extract_import_path(import_line) {
            match self.load_imported_module(&import_path, file_path) {
                Ok(content) => {
                    self.imported_modules.insert(import_path.clone(), content);
                }
                Err(e) => {
                    let diagnostic = Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: "WESL_IMPORT_ERROR".to_string(),
                        message: format!("Failed to load import '{}': {}", import_path, e),
                        file_path: file_path.to_string(),
                        line: 0, // Should track actual line
                        column: 0,
                        length: 1,
                        suggestions: vec![],
                        related_information: vec![],
                        quick_fix_available: false,
                    };
                    self.diagnostics.add_diagnostic(diagnostic);
                }
            }
        }
        Ok(())
    }

    /// Handle WESL conditional compilation directives
    fn handle_conditional_compilation(
        &mut self, 
        lines: &mut std::iter::Peekable<std::str::Lines>, 
        result: &mut String,
        file_path: &str
    ) -> Result<()> {
        let mut condition_stack = Vec::new();
        let mut skip_block = false;
        let mut skip_depth = 0;
        
        // For now, we'll just skip conditional compilation blocks
        // In a real implementation, this would evaluate conditions
        while let Some(line) = lines.next() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("#if") || trimmed.starts_with("#ifdef") || trimmed.starts_with("#ifndef") {
                condition_stack.push(skip_block);
                if skip_block {
                    skip_depth += 1;
                } else {
                    // Evaluate condition - for now we'll assume true
                    skip_block = !self.evaluate_condition(trimmed);
                }
            } else if trimmed.starts_with("#elif") || trimmed.starts_with("#else") {
                if skip_depth == 0 {
                    skip_block = !skip_block;
                }
            } else if trimmed.starts_with("#endif") {
                if skip_depth > 0 {
                    skip_depth -= 1;
                } else {
                    if let Some(prev_skip) = condition_stack.pop() {
                        skip_block = prev_skip;
                    } else {
                        skip_block = false;
                    }
                }
            }
            
            if !skip_block && skip_depth == 0 {
                result.push_str(line);
                result.push('\n');
            }
        }
        
        Ok(())
    }

    /// Handle WESL preprocessor directives
    fn handle_preprocessor_directive(&mut self, directive: &str) -> Result<()> {
        // Process preprocessor directives like #define, #pragma, etc.
        // For now, we just log them as warnings
        if directive.trim().starts_with("#define") {
            let diagnostic = Diagnostic {
                severity: DiagnosticSeverity::Info,
                code: "WESL_DEFINE_INFO".to_string(),
                message: format!("Preprocessor directive '{}' processed", directive.trim()),
                file_path: "preprocessor".to_string(),
                line: 0,
                column: 0,
                length: 1,
                suggestions: vec![],
                related_information: vec![],
                quick_fix_available: false,
            };
            self.diagnostics.add_diagnostic(diagnostic);
        }
        
        Ok(())
    }

    /// Extract import path from an import statement
    fn extract_import_path(&self, import_line: &str) -> Option<String> {
        // Look for import "path" or import 'path' patterns
        let clean_line = import_line.trim();
        if clean_line.starts_with("import") {
            let after_import = clean_line[6..].trim_start();
            if after_import.starts_with('"') || after_import.starts_with('\'') {
                let quote = after_import.chars().next().unwrap();
                if let Some(end_pos) = after_import[1..].find(quote) {
                    return Some(after_import[1..end_pos + 1].to_string());
                }
            }
        } else if clean_line.starts_with("#import") {
            let after_import = clean_line[7..].trim_start();
            if after_import.starts_with('"') || after_import.starts_with('\'') {
                let quote = after_import.chars().next().unwrap();
                if let Some(end_pos) = after_import[1..].find(quote) {
                    return Some(after_import[1..end_pos + 1].to_string());
                }
            }
        }
        None
    }

    /// Load an imported WESL module
    fn load_imported_module(&mut self, import_path: &str, base_path: &str) -> Result<String> {
        // In a real implementation, this would load from file system or package manager
        // For now, return an empty string with a warning
        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: "WESL_IMPORT_NOT_FOUND".to_string(),
            message: format!("Import '{}' not found, using empty placeholder", import_path),
            file_path: base_path.to_string(),
            line: 0,
            column: 0,
            length: 1,
            suggestions: vec![],
            related_information: vec![],
            quick_fix_available: false,
        };
        self.diagnostics.add_diagnostic(diagnostic);
        
        Ok(String::new())
    }

    /// Evaluate a conditional compilation condition
    fn evaluate_condition(&self, condition_line: &str) -> bool {
        // Simplified condition evaluation
        // In a real implementation, this would be more sophisticated
        if condition_line.contains("DEBUG") {
            // Example: return true if in debug mode
            cfg!(debug_assertions)
        } else {
            // Default to true for unknown conditions
            true
        }
    }

    /// Get diagnostics from the last conversion
    pub fn get_diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}

impl Default for WESLConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wesl_converter_creation() {
        let converter = WESLConverter::new();
        assert!(converter.diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn test_basic_wgsl_conversion() {
        let mut converter = WESLConverter::new();
        let wesl_source = "@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }";
        let result = converter.convert_to_wgsl(wesl_source, "test.wesl").unwrap();
        assert!(result.contains("Converted from WESL"));
        assert!(result.contains("main"));
    }
}