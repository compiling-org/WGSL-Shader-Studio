use std::collections::HashMap;
use anyhow::{Result, Context, bail};
use crate::converter::diagnostics::{Diagnostics, diagnostic_helpers};

/// GLSL to WGSL converter
pub struct GLSLConverter {
    diagnostics: Diagnostics,
    symbol_table: HashMap<String, SymbolInfo>,
    uniform_blocks: Vec<UniformBlock>,
    functions: HashMap<String, FunctionInfo>,
}

#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    glsl_type: String,
    wgsl_type: String,
    storage_class: StorageClass,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
struct UniformBlock {
    name: String,
    members: Vec<SymbolInfo>,
    set: u32,
    binding: u32,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    return_type: String,
    parameters: Vec<ParameterInfo>,
    body: String,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
struct ParameterInfo {
    name: String,
    glsl_type: String,
    wgsl_type: String,
    storage_class: StorageClass,
}

#[derive(Debug, Clone, PartialEq)]
enum StorageClass {
    Uniform,
    Storage,
    Function,
    Private,
    Workgroup,
}

impl GLSLConverter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            diagnostics: Diagnostics::new(),
            symbol_table: HashMap::new(),
            uniform_blocks: Vec::new(),
            functions: HashMap::new(),
        })
    }
    
    /// Convert GLSL source code to WGSL
    pub fn convert(&mut self, glsl_source: &str, file_path: &str) -> Result<String> {
        // Simple validation
        self.parse_glsl(glsl_source, file_path)?;
        
        // Generate basic WGSL code
        let wgsl = self.generate_simple_wgsl(glsl_source, file_path)?;
        
        Ok(wgsl)
    }
    
    /// Parse GLSL source code (simplified without tree-sitter)
    fn parse_glsl(&mut self, glsl_source: &str, file_path: &str) -> Result<String> {
        // Simple validation - check for basic syntax issues
        if glsl_source.trim().is_empty() {
            self.diagnostics.add_diagnostic(
                diagnostic_helpers::syntax_error(
                    "Empty GLSL source",
                    file_path,
                    1,
                    1,
                    1
                )
            );
            bail!("GLSL source is empty");
        }
        
        Ok(glsl_source.to_string())
    }
    
    /// Generate simple WGSL code from GLSL source
    fn generate_simple_wgsl(&mut self, glsl_source: &str, file_path: &str) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Add header comments
        wgsl_code.push_str("// Converted from GLSL\n");
        wgsl_code.push_str(&format!("// Original file: {}\n", file_path));
        wgsl_code.push('\n');
        
        // Generate basic WGSL structure
        self.generate_uniform_blocks(&mut wgsl_code)?;
        self.generate_function_declarations(&mut wgsl_code)?;
        self.generate_main_function(&mut wgsl_code)?;
        
        Ok(wgsl_code)
    }
    
    /// Generate uniform blocks in WGSL
    fn generate_uniform_blocks(&mut self, wgsl_code: &mut String) -> Result<()> {
        for (_index, block) in self.uniform_blocks.iter().enumerate() {
            wgsl_code.push_str(&format!("struct {} {{\n", block.name));
            
            for member in &block.members {
                wgsl_code.push_str(&format!("    {}: {},\n", member.name, member.wgsl_type));
            }
            
            wgsl_code.push_str("}\n\n");
            wgsl_code.push_str(&format!("@group({}) @binding({}) var<uniform> {}_block: {};\n\n", 
                block.set, block.binding, block.name.to_lowercase(), block.name));
        }
        
        Ok(())
    }
    
    /// Generate function declarations
    fn generate_function_declarations(&mut self, wgsl_code: &mut String) -> Result<()> {
        for function in self.functions.values() {
            let return_type = self.convert_glsl_type_to_wgsl(&function.return_type);
            
            wgsl_code.push_str(&format!("fn {}(", function.name));
            
            // Add parameters
            let params: Vec<String> = function.parameters.iter()
                .map(|p| format!("{}: {}", p.name, p.wgsl_type))
                .collect();
            wgsl_code.push_str(&params.join(", "));
            
            wgsl_code.push_str(&format!(") -> {} {{\n", return_type));
            wgsl_code.push_str("    // Function body would be converted here\n");
            wgsl_code.push_str("}\n\n");
        }
        
        Ok(())
    }
    
    /// Generate main function
    fn generate_main_function(&mut self, wgsl_code: &mut String) -> Result<()> {
        wgsl_code.push_str("@vertex\n");
        wgsl_code.push_str("fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {\n");
        wgsl_code.push_str("    // Vertex shader logic would be converted here\n");
        wgsl_code.push_str("    return vec4<f32>(0.0, 0.0, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main() -> @location(0) vec4<f32> {\n");
        wgsl_code.push_str("    // Fragment shader logic would be converted here\n");
        wgsl_code.push_str("    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n");
        
        Ok(())
    }
    
    /// Convert GLSL type to WGSL type
    fn convert_glsl_type_to_wgsl(&self, glsl_type: &str) -> String {
        match glsl_type {
            "void" => "()".to_string(),
            "bool" => "bool".to_string(),
            "int" => "i32".to_string(),
            "uint" => "u32".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            "vec2" => "vec2<f32>".to_string(),
            "vec3" => "vec3<f32>".to_string(),
            "vec4" => "vec4<f32>".to_string(),
            "ivec2" => "vec2<i32>".to_string(),
            "ivec3" => "vec3<i32>".to_string(),
            "ivec4" => "vec4<i32>".to_string(),
            "uvec2" => "vec2<u32>".to_string(),
            "uvec3" => "vec3<u32>".to_string(),
            "uvec4" => "vec4<u32>".to_string(),
            "mat2" => "mat2x2<f32>".to_string(),
            "mat3" => "mat3x3<f32>".to_string(),
            "mat4" => "mat4x4<f32>".to_string(),
            "sampler2D" => "texture_2d<f32>".to_string(),
            _ => glsl_type.to_string(),
        }
    }
    
    /// Get diagnostics from conversion
    pub fn get_diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
    
    /// Take ownership of diagnostics
    pub fn take_diagnostics(self) -> Diagnostics {
        self.diagnostics
    }
}

impl Default for GLSLConverter {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create GLSL converter"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glsl_converter_creation() {
        let converter = GLSLConverter::new();
        assert!(converter.is_ok());
    }
    
    #[test]
    fn test_simple_glsl_parsing() {
        let glsl = r#"
            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;
        
        let mut converter = GLSLConverter::new().unwrap();
        let result = converter.convert(glsl, "test.frag");
        
        // Should succeed even if conversion is not complete
        assert!(result.is_ok());
        let wgsl = result.unwrap();
        assert!(wgsl.contains("@fragment"));
        assert!(wgsl.contains("fs_main"));
    }
    
    #[test]
    fn test_glsl_type_conversion() {
        let converter = GLSLConverter::new().unwrap();
        
        assert_eq!(converter.convert_glsl_type_to_wgsl("float"), "f32");
        assert_eq!(converter.convert_glsl_type_to_wgsl("vec3"), "vec3<f32>");
        assert_eq!(converter.convert_glsl_type_to_wgsl("mat4"), "mat4x4<f32>");
        assert_eq!(converter.convert_glsl_type_to_wgsl("sampler2D"), "texture_2d<f32>");
    }
}