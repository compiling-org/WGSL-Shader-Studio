//! WESL (WebGPU Enhanced Shading Language) Integration
//! Provides support for WESL, an enhanced version of WGSL with additional features
//! like imports, conditional compilation, and shader libraries

use std::collections::HashMap;
use anyhow::{Result, bail};
use crate::converter::diagnostics::{Diagnostics, Diagnostic, DiagnosticSeverity};

/// WESL Compiler that handles WESL-specific features
pub struct WeslCompiler {
    diagnostics: Diagnostics,
    /// Cache for compiled shaders
    shader_cache: HashMap<String, String>,
    /// Imported modules cache
    import_cache: HashMap<String, String>,
}

impl WeslCompiler {
    pub fn new() -> Self {
        Self {
            diagnostics: Diagnostics::new(),
            shader_cache: HashMap::new(),
            import_cache: HashMap::new(),
        }
    }

    /// Compile WESL source code to WGSL
    pub fn compile_wesl_to_wgsl(&mut self, source: &str, file_path: &str) -> Result<String> {
        // First, process imports
        let processed_source = self.process_imports(source, file_path)?;
        
        // Process conditional compilation directives
        let processed_source = self.process_conditional_compilation(&processed_source)?;
        
        // Validate the resulting WGSL
        self.validate_wgsl(&processed_source)?;
        
        // Add WESL-specific transformations
        let final_wgsl = self.apply_wesl_transformations(&processed_source)?;
        
        Ok(final_wgsl)
    }

    /// Process WESL import statements
    fn process_imports(&mut self, source: &str, file_path: &str) -> Result<String> {
        let mut result = String::new();
        let mut in_import_section = false;
        
        for line in source.lines() {
            if line.trim_start().starts_with("import") || line.trim_start().starts_with("#import") {
                in_import_section = true;
                
                // Extract import path
                let import_line = line.trim();
                if let Some(import_path) = self.extract_import_path(import_line) {
                    match self.load_import(&import_path, file_path) {
                        Ok(imported_content) => {
                            result.push_str(&imported_content);
                            result.push('\n');
                        }
                        Err(e) => {
                            let diagnostic = Diagnostic {
                                severity: DiagnosticSeverity::Error,
                                code: "WESL_IMPORT_ERROR".to_string(),
                                message: format!("Failed to import '{}': {}", import_path, e),
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
            } else if in_import_section && !line.trim().is_empty() && !line.trim_start().starts_with("import") && !line.trim_start().starts_with("#import") {
                in_import_section = false;
                result.push_str(line);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        
        Ok(result)
    }

    /// Process WESL conditional compilation directives
    fn process_conditional_compilation(&mut self, source: &str) -> Result<String> {
        let mut result = String::new();
        let mut skip_block = false;
        let mut skip_depth = 0;
        
        for line in source.lines() {
            if line.trim_start().starts_with("#if") {
                if skip_block {
                    skip_depth += 1;
                } else {
                    // Evaluate condition (simplified for now)
                    skip_block = !self.evaluate_condition(line);
                }
            } else if line.trim_start().starts_with("#else") && skip_depth == 0 {
                skip_block = !skip_block;
            } else if line.trim_start().starts_with("#endif") {
                if skip_depth > 0 {
                    skip_depth -= 1;
                } else {
                    skip_block = false;
                }
            } else if line.trim_start().starts_with("#ifdef") || line.trim_start().starts_with("#ifndef") {
                if skip_block {
                    skip_depth += 1;
                } else {
                    // Evaluate condition (simplified for now)
                    skip_block = !self.evaluate_condition(line);
                }
            } else if !skip_block {
                result.push_str(line);
                result.push('\n');
            }
        }
        
        Ok(result)
    }

    /// Apply WESL-specific transformations to make it compatible with WGSL
    fn apply_wesl_transformations(&self, source: &str) -> Result<String> {
        let mut result = source.to_string();
        
        // Transform WESL-specific syntax to WGSL-compatible syntax
        // This is a simplified version - in a real implementation, 
        // this would handle more complex transformations
        
        // Example: Transform WESL macro-like features
        result = self.transform_macros(result)?;
        
        // Example: Transform WESL utility functions
        result = self.transform_utilities(result)?;
        
        Ok(result)
    }

    /// Validate the resulting WGSL code
    fn validate_wgsl(&mut self, wgsl: &str) -> Result<()> {
        // Basic validation - in a real implementation, this would use naga or similar
        if wgsl.contains("@vertex") && !wgsl.contains("fn ") {
            let diagnostic = Diagnostic {
                severity: DiagnosticSeverity::Warning,
                code: "WESL_VALIDATION_WARNING".to_string(),
                message: "Vertex shader declared but no functions found".to_string(),
                file_path: "validation".to_string(),
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
        let clean_line = import_line.trim_start();
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

    /// Load an imported module
    fn load_import(&mut self, import_path: &str, base_path: &str) -> Result<String> {
        // In a real implementation, this would load from file system or package manager
        // For now, we'll return an empty string or check the cache
        if let Some(cached) = self.import_cache.get(import_path) {
            return Ok(cached.clone());
        }
        
        // This is where we'd implement actual file loading
        // For now, return an empty string and add a warning
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

    /// Transform WESL macros to WGSL-compatible code
    fn transform_macros(&self, source: String) -> Result<String> {
        // This is a simplified macro transformation
        // In a real implementation, this would handle more complex macro systems
        let mut result = source;
        
        // Example: transform simple macro patterns
        result = result.replace("M_PI", "3.141592653589793");
        result = result.replace("M_TWO_PI", "6.283185307179586");
        
        Ok(result)
    }

    /// Transform WESL utility functions to WGSL-compatible code
    fn transform_utilities(&self, source: String) -> Result<String> {
        let mut result = source;
        
        // Example: transform common utility functions
        result = result.replace("smoothstep01", "smoothstep(0.0, 1.0, ");
        
        Ok(result)
    }

    /// Get diagnostics from the last compilation
    pub fn get_diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Reset the compiler state
    pub fn reset(&mut self) {
        self.diagnostics = Diagnostics::new();
        self.shader_cache.clear();
        self.import_cache.clear();
    }
}

impl Default for WeslCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wesl_compiler_creation() {
        let compiler = WeslCompiler::new();
        assert!(compiler.diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn test_basic_wgsl_passthrough() {
        let mut compiler = WeslCompiler::new();
        let wgsl = "@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }";
        let result = compiler.compile_wesl_to_wgsl(wgsl, "test.wesl").unwrap();
        assert!(result.contains("main"));
        assert!(result.contains("vec4<f32>"));
    }
}