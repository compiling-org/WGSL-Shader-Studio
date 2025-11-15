//! WGSL Diagnostics Module
//! Provides real-time shader validation and error reporting using naga

use naga::{front::wgsl, valid::{Validator, ValidationFlags, Capabilities}};
use std::collections::HashMap;

/// Diagnostic severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// A single diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub code: Option<String>,
}

/// WGSL diagnostics analyzer
pub struct WgslDiagnostics {
    validator: Validator,
}

impl WgslDiagnostics {
    pub fn new() -> Self {
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        Self {
            validator,
        }
    }

    /// Analyze WGSL code and return diagnostics
    pub fn analyze(&mut self, wgsl_code: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // First, try to parse the WGSL code
        match wgsl::parse_str(wgsl_code) {
            Ok(module) => {
                // If parsing succeeded, validate the module
                match self.validator.validate(&module) {
                    Ok(_) => {
                        // No validation errors
                    }
                    Err(validation_error) => {
                        // Convert validation error to diagnostic
                        diagnostics.push(self.validation_error_to_diagnostic(&validation_error));
                    }
                }
            }
            Err(parse_error) => {
                // Convert parse error to diagnostic
                diagnostics.push(self.parse_error_to_diagnostic(&parse_error));
            }
        }

        diagnostics
    }

    /// Convert a naga parse error to a diagnostic
    fn parse_error_to_diagnostic(&self, error: &wgsl::ParseError) -> Diagnostic {
        // For now, use a simple approach since the exact API may vary
        let message = format!("Parse error: {:?}", error);
        
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            message,
            line: 0,
            column: 0,
            length: 1, // Default length for parse errors
            code: Some("WGSL_PARSE_ERROR".to_string()),
        }
    }

    /// Convert a naga validation error to a diagnostic
    fn validation_error_to_diagnostic(&self, error: &naga::WithSpan<naga::valid::ValidationError>) -> Diagnostic {
        let message = format!("Validation error: {:?}", error);
        
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            message,
            line: 0,
            column: 0,
            length: 1,
            code: Some("WGSL_VALIDATION_ERROR".to_string()),
        }
    }

    /// Convert byte offset to line and column numbers
    fn offset_to_line_column(&self, offset: usize) -> (usize, usize) {
        // This is a simplified implementation
        // In a real implementation, you'd parse the entire source and build a proper offset map
        (offset / 50, offset % 50) // Rough approximation
    }

    /// Get syntax highlighting information
    pub fn get_syntax_info(&self, wgsl_code: &str) -> HashMap<String, Vec<(usize, usize)>> {
        let mut syntax_info: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
        
        // Keywords
        let keywords = vec![
            "struct", "fn", "let", "var", "const", "if", "else", "for", "while", "loop",
            "break", "continue", "return", "discard", "continue", "fallthrough",
            "case", "default", "switch",
        ];
        
        // Types
        let types = vec![
            "bool", "i32", "u32", "f32", "f16", "vec2", "vec3", "vec4",
            "mat2x2", "mat2x3", "mat2x4", "mat3x2", "mat3x3", "mat3x4", "mat4x2", "mat4x3", "mat4x4",
            "atomic", "array", "ptr", "sampler", "sampler_comparison", "texture_1d", "texture_2d",
            "texture_2d_array", "texture_3d", "texture_cube", "texture_cube_array", "texture_multisampled_2d",
        ];
        
        // Built-in functions
        let builtin_functions = vec![
            "abs", "acos", "asin", "atan", "atan2", "ceil", "clamp", "cos", "cosh", "cross",
            "distance", "dot", "exp", "exp2", "floor", "fract", "length", "log", "log2",
            "max", "min", "mix", "normalize", "pow", "reflect", "refract", "round", "sign",
            "sin", "sinh", "smoothstep", "sqrt", "step", "tan", "tanh", "trunc",
        ];
        
        // Attributes
        let attributes = vec![
            "@builtin", "@location", "@group", "@binding", "@stage", "@workgroup_size",
            "@vertex", "@fragment", "@compute", "@const", "@id", "@size", "@align",
        ];
        
        // Find all occurrences
        for (line_idx, line) in wgsl_code.lines().enumerate() {
            let line_start = wgsl_code.lines().take(line_idx).map(|l| l.len() + 1).sum::<usize>();
            
            // Check keywords
            for keyword in &keywords {
                if let Some(pos) = line.find(keyword) {
                    syntax_info.entry("keyword".to_string()).or_default().push((line_start + pos, keyword.len()));
                }
            }
            
            // Check types
            for type_name in &types {
                if let Some(pos) = line.find(type_name) {
                    syntax_info.entry("type".to_string()).or_default().push((line_start + pos, type_name.len()));
                }
            }
            
            // Check builtin functions
            for func in &builtin_functions {
                if let Some(pos) = line.find(func) {
                    syntax_info.entry("function".to_string()).or_default().push((line_start + pos, func.len()));
                }
            }
            
            // Check attributes
            for attr in &attributes {
                if let Some(pos) = line.find(attr) {
                    syntax_info.entry("attribute".to_string()).or_default().push((line_start + pos, attr.len()));
                }
            }
        }
        
        syntax_info
    }

    /// Quick syntax check - returns true if code appears to be valid WGSL
    pub fn quick_check(&self, wgsl_code: &str) -> bool {
        // Basic checks for required WGSL structure
        let has_struct_or_fn = wgsl_code.contains("struct") || wgsl_code.contains("fn");
        let has_valid_braces = self.check_brace_balance(wgsl_code);
        let has_semicolons = wgsl_code.contains(';');
        
        has_struct_or_fn && has_valid_braces && has_semicolons
    }

    /// Check if braces are balanced
    fn check_brace_balance(&self, code: &str) -> bool {
        let mut brace_count = 0;
        let mut in_string = false;
        let mut in_comment = false;
        
        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let ch = chars[i];
            
            if in_comment {
                if ch == '\n' {
                    in_comment = false;
                }
                i += 1;
                continue;
            }
            
            if in_string {
                if ch == '"' && (i == 0 || chars[i-1] != '\\') {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            
            match ch {
                '"' => in_string = true,
                '/' if i + 1 < chars.len() && chars[i + 1] == '/' => in_comment = true,
                '{' => brace_count += 1,
                '}' => {
                    if brace_count == 0 {
                        return false; // Unmatched closing brace
                    }
                    brace_count -= 1;
                }
                _ => {}
            }
            
            i += 1;
        }
        
        brace_count == 0 && !in_string
    }
}

impl Default for WgslDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_wgsl() {
        let mut diagnostics = WgslDiagnostics::new();
        let valid_wgsl = r#"
            struct VertexOutput {
                @builtin(position) position: vec4<f32>,
                @location(0) color: vec4<f32>,
            }
            
            @vertex
            fn vs_main(@location(0) position: vec3<f32>, @location(1) color: vec3<f32>) -> VertexOutput {
                var output: VertexOutput;
                output.position = vec4<f32>(position, 1.0);
                output.color = vec4<f32>(color, 1.0);
                return output;
            }
            
            @fragment
            fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
                return input.color;
            }
        "#;
        
        let result = diagnostics.analyze(valid_wgsl);
        assert!(result.is_empty(), "Valid WGSL should have no diagnostics: {:?}", result);
    }

    #[test]
    fn test_invalid_wgsl() {
        let mut diagnostics = WgslDiagnostics::new();
        let invalid_wgsl = r#"
            @vertex
            fn vs_main() -> vec4<f32> {
                return vec4<f32>(1.0, 2.0, 3.0); // Wrong number of components
            }
        "#;
        
        let result = diagnostics.analyze(invalid_wgsl);
        assert!(!result.is_empty(), "Invalid WGSL should have diagnostics");
    }

    #[test]
    fn test_brace_balance() {
        let diagnostics = WgslDiagnostics::new();
        
        assert!(diagnostics.check_brace_balance("fn main() { return 1.0; }"));
        assert!(!diagnostics.check_brace_balance("fn main() { return 1.0;"));
        assert!(!diagnostics.check_brace_balance("fn main() return 1.0; }"));
        assert!(diagnostics.check_brace_balance("fn main() { if (true) { return 1.0; } }"));
    }

    #[test]
    fn test_quick_check() {
        let diagnostics = WgslDiagnostics::new();
        
        assert!(diagnostics.quick_check("fn main() { return 1.0; }"));
        assert!(!diagnostics.quick_check("fn main() { return 1.0")); // Missing closing brace
        assert!(!diagnostics.quick_check("main() { return 1.0; }")); // Missing 'fn'
    }
}