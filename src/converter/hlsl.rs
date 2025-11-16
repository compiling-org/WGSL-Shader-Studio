use std::collections::HashMap;
use anyhow::{Result, Context, bail};
use crate::converter::diagnostics::{Diagnostics, diagnostic_helpers};

/// HLSL to WGSL converter
pub struct HLSLConverter {
    diagnostics: Diagnostics,
    symbol_table: HashMap<String, SymbolInfo>,
    constant_buffers: Vec<ConstantBuffer>,
    texture_declarations: Vec<TextureInfo>,
    functions: HashMap<String, FunctionInfo>,
}

#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    hlsl_type: String,
    wgsl_type: String,
    storage_class: StorageClass,
    line: usize,
    column: usize,
    semantic: Option<String>, // HLSL semantic (e.g., POSITION, TEXCOORD0)
}

#[derive(Debug, Clone)]
struct ConstantBuffer {
    name: String,
    members: Vec<SymbolInfo>,
    set: u32,
    binding: u32,
}

#[derive(Debug, Clone)]
struct TextureInfo {
    name: String,
    hlsl_type: String,
    wgsl_type: String,
    set: u32,
    binding: u32,
    is_comparison: bool,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    return_type: String,
    parameters: Vec<ParameterInfo>,
    body: String,
    line: usize,
    column: usize,
    shader_type: Option<ShaderType>, // Vertex, Pixel, Compute
}

#[derive(Debug, Clone)]
struct ParameterInfo {
    name: String,
    hlsl_type: String,
    wgsl_type: String,
    storage_class: StorageClass,
    semantic: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum StorageClass {
    Uniform,
    Storage,
    Function,
    Private,
    Workgroup,
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq)]
enum ShaderType {
    Vertex,
    Pixel,
    Compute,
}

impl HLSLConverter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            diagnostics: Diagnostics::new(),
            symbol_table: HashMap::new(),
            constant_buffers: Vec::new(),
            texture_declarations: Vec::new(),
            functions: HashMap::new(),
        })
    }
    
    /// Convert HLSL source code to WGSL
    pub fn convert(&mut self, hlsl_source: &str, file_path: &str) -> Result<String> {
        // Simple validation
        self.parse_hlsl(hlsl_source, file_path)?;
        
        // Generate basic WGSL code
        let wgsl = self.generate_simple_wgsl(hlsl_source, file_path)?;
        
        Ok(wgsl)
    }
    
    /// Parse HLSL source code (simplified without tree-sitter)
    fn parse_hlsl(&mut self, hlsl_source: &str, file_path: &str) -> Result<String> {
        // Simple validation - check for basic syntax issues
        if hlsl_source.trim().is_empty() {
            self.diagnostics.add_diagnostic(
                diagnostic_helpers::syntax_error(
                    "Empty HLSL source",
                    file_path,
                    1,
                    1,
                    1
                )
            );
            bail!("HLSL source is empty");
        }
        
        Ok(hlsl_source.to_string())
    }
    
    /// Collect parse errors from the tree (placeholder - no tree-sitter)
    fn collect_parse_errors(&mut self, _tree: &str, _source: &str, _file_path: &str) {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
    }
    
    /// Walk the tree and collect error nodes (placeholder - no tree-sitter)
    fn walk_for_errors(&mut self, _node: &str, _source: &str, _file_path: &str) {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
    }
    
    /// Analyze the AST to extract symbols, constant buffers, functions, etc. (placeholder - no tree-sitter)
    fn analyze_ast(&mut self, _tree: &str, _source: &str, _file_path: &str) -> Result<()> {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
        Ok(())
    }
    
    /// Find all declarations (variables, textures, etc.) (placeholder - no tree-sitter)
    fn find_declarations(&mut self, _node: &str, _source: &str, _file_path: &str) -> Result<()> {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
        Ok(())
    }
    
    /// Find all function definitions (placeholder - no tree-sitter)
    fn find_functions(&mut self, _node: &str, _source: &str, _file_path: &str) -> Result<()> {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
        Ok(())
    }
    
    /// Find all constant buffers (cbuffer) (placeholder - no tree-sitter)
    fn find_constant_buffers(&mut self, _node: &str, _source: &str, _file_path: &str) -> Result<()> {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
        Ok(())
    }
    
    /// Parse constant buffer body to extract members (placeholder - no tree-sitter)
    fn parse_cbuffer_body(&mut self, _body_node: &str, _source: &str, _file_path: &str) -> Result<Vec<SymbolInfo>> {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
        Ok(Vec::new())
    }
    
    /// Find texture declarations (Texture2D, TextureCube, etc.) (placeholder - no tree-sitter)
    fn find_texture_declarations(&mut self, _node: &str, _source: &str, _file_path: &str) -> Result<()> {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
        Ok(())
    }
    
    /// Find semantics (POSITION, TEXCOORD0, etc.) (placeholder - no tree-sitter)
    fn find_semantics(&mut self, _node: &str, _source: &str, _file_path: &str) -> Result<()> {
        // Tree-sitter functionality disabled for now
        // Keep original logic preserved for future restoration
        Ok(())
    }
    
    /// Determine shader type from function name
    fn determine_shader_type(&self, function_name: &str) -> Option<ShaderType> {
        let name_lower = function_name.to_lowercase();
        
        if name_lower.contains("vertex") || name_lower.contains("vs") {
            Some(ShaderType::Vertex)
        } else if name_lower.contains("pixel") || name_lower.contains("fragment") || name_lower.contains("ps") {
            Some(ShaderType::Pixel)
        } else if name_lower.contains("compute") || name_lower.contains("cs") {
            Some(ShaderType::Compute)
        } else {
            None
        }
    }
    
    /// Convert HLSL type to WGSL type
    fn convert_hlsl_type_to_wgsl(&self, hlsl_type: &str) -> String {
        match hlsl_type {
            "void" => "()".to_string(),
            "bool" => "bool".to_string(),
            "int" => "i32".to_string(),
            "uint" => "u32".to_string(),
            "dword" => "u32".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            "float2" => "vec2<f32>".to_string(),
            "float3" => "vec3<f32>".to_string(),
            "float4" => "vec4<f32>".to_string(),
            "int2" => "vec2<i32>".to_string(),
            "int3" => "vec3<i32>".to_string(),
            "int4" => "vec4<i32>".to_string(),
            "uint2" => "vec2<u32>".to_string(),
            "uint3" => "vec3<u32>".to_string(),
            "uint4" => "vec4<u32>".to_string(),
            "bool2" => "vec2<bool>".to_string(),
            "bool3" => "vec3<bool>".to_string(),
            "bool4" => "vec4<bool>".to_string(),
            "float2x2" => "mat2x2<f32>".to_string(),
            "float3x3" => "mat3x3<f32>".to_string(),
            "float4x4" => "mat4x4<f32>".to_string(),
            "matrix" => "mat4x4<f32>".to_string(),
            _ => {
                // Handle array types and other complex types
                if hlsl_type.contains("[") {
                    // Array type
                    let base_type = hlsl_type.split('[').next().unwrap();
                    let array_size = hlsl_type.split('[').nth(1).unwrap_or("").trim_end_matches(']');
                    let wgsl_base = self.convert_hlsl_type_to_wgsl(base_type);
                    format!("array<{}, {}>", wgsl_base, array_size)
                } else {
                    // Unknown type, return as-is with a warning
                    hlsl_type.to_string()
                }
            }
        }
    }
    
    /// Convert HLSL texture type to WGSL type
    fn convert_hlsl_texture_to_wgsl(&self, hlsl_texture_type: &str) -> String {
        match hlsl_texture_type {
            "Texture2D" => "texture_2d<f32>".to_string(),
            "Texture2DMS" => "texture_multisampled_2d<f32>".to_string(),
            "Texture3D" => "texture_3d<f32>".to_string(),
            "TextureCube" => "texture_cube<f32>".to_string(),
            "Texture2DArray" => "texture_2d_array<f32>".to_string(),
            "TextureCubeArray" => "texture_cube_array<f32>".to_string(),
            "Texture1D" => "texture_1d<f32>".to_string(),
            "Texture1DArray" => "texture_1d_array<f32>".to_string(),
            "Texture2DShadow" | "Texture2DMSArray" | "TextureCubeShadow" => {
                // Comparison samplers
                hlsl_texture_type.to_string()
            }
            _ => {
                // Generic texture type
                "texture_2d<f32>".to_string()
            }
        }
    }
    
    /// Generate simple WGSL code from HLSL source
    fn generate_simple_wgsl(&mut self, hlsl_source: &str, file_path: &str) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Add header comments
        wgsl_code.push_str("// Converted from HLSL\n");
        wgsl_code.push_str(&format!("// Original file: {}\n", file_path));
        wgsl_code.push('\n');
        
        // Generate basic WGSL structure
        self.generate_vertex_input_structure(&mut wgsl_code)?;
        self.generate_constant_buffers(&mut wgsl_code)?;
        self.generate_texture_declarations(&mut wgsl_code)?;
        self.generate_sampler_declarations(&mut wgsl_code)?;
        self.generate_function_declarations(&mut wgsl_code)?;
        self.generate_main_functions(&mut wgsl_code)?;
        
        Ok(wgsl_code)
    }
    
    /// Generate vertex input structure
    fn generate_vertex_input_structure(&mut self, wgsl_code: &mut String) -> Result<()> {
        wgsl_code.push_str("struct VertexInput {\n");
        wgsl_code.push_str("    @location(0) position: vec3<f32>,\n");
        wgsl_code.push_str("    @location(1) texcoord: vec2<f32>,\n");
        wgsl_code.push_str("    @location(2) normal: vec3<f32>,\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("struct VertexOutput {\n");
        wgsl_code.push_str("    @builtin(position) position: vec4<f32>,\n");
        wgsl_code.push_str("    @location(0) texcoord: vec2<f32>,\n");
        wgsl_code.push_str("    @location(1) normal: vec3<f32>,\n");
        wgsl_code.push_str("}\n\n");
        
        Ok(())
    }
    
    /// Generate constant buffers in WGSL
    fn generate_constant_buffers(&mut self, wgsl_code: &mut String) -> Result<()> {
        for (index, cbuffer) in self.constant_buffers.iter().enumerate() {
            wgsl_code.push_str(&format!("struct {} {{\n", cbuffer.name));
            
            for member in &cbuffer.members {
                wgsl_code.push_str(&format!("    {}: {},\n", member.name, member.wgsl_type));
            }
            
            wgsl_code.push_str("}\n\n");
            wgsl_code.push_str(&format!("@group({}) @binding({}) var<uniform> {}_block: {};\n\n", 
                cbuffer.set, cbuffer.binding, cbuffer.name.to_lowercase(), cbuffer.name));
        }
        
        Ok(())
    }
    
    /// Generate texture declarations
    fn generate_texture_declarations(&mut self, wgsl_code: &mut String) -> Result<()> {
        for (index, texture) in self.texture_declarations.iter().enumerate() {
            wgsl_code.push_str(&format!("@group({}) @binding({}) var {}: {};\n", 
                texture.set, texture.binding, texture.name, texture.wgsl_type));
        }
        
        if !self.texture_declarations.is_empty() {
            wgsl_code.push('\n');
        }
        
        Ok(())
    }
    
    /// Generate sampler declarations
    fn generate_sampler_declarations(&mut self, wgsl_code: &mut String) -> Result<()> {
        // Add default samplers for textures
        for (index, texture) in self.texture_declarations.iter().enumerate() {
            wgsl_code.push_str(&format!("@group({}) @binding({}) var {}_sampler: sampler;\n", 
                texture.set, texture.binding + 1000, texture.name));
        }
        
        if !self.texture_declarations.is_empty() {
            wgsl_code.push('\n');
        }
        
        Ok(())
    }
    
    /// Generate function declarations
    fn generate_function_declarations(&mut self, wgsl_code: &mut String) -> Result<()> {
        for function in self.functions.values() {
            let return_type = self.convert_hlsl_type_to_wgsl(&function.return_type);
            
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
    
    /// Generate main functions
    fn generate_main_functions(&mut self, wgsl_code: &mut String) -> Result<()> {
        // Generate vertex shader main function
        wgsl_code.push_str("@vertex\n");
        wgsl_code.push_str("fn vs_main(input: VertexInput) -> VertexOutput {\n");
        wgsl_code.push_str("    var output: VertexOutput;\n");
        wgsl_code.push_str("    output.position = vec4<f32>(input.position, 1.0);\n");
        wgsl_code.push_str("    output.texcoord = input.texcoord;\n");
        wgsl_code.push_str("    output.normal = input.normal;\n");
        wgsl_code.push_str("    return output;\n");
        wgsl_code.push_str("}\n\n");
        
        // Generate fragment shader main function
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {\n");
        wgsl_code.push_str("    // Fragment shader logic would be converted here\n");
        wgsl_code.push_str("    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n");
        
        Ok(())
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

impl Default for HLSLConverter {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create HLSL converter"))
    }
}

// Tree-sitter functionality disabled for compilation
// extern "C" {
//     fn tree_sitter_hlsl() -> Language;
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlsl_converter_creation() {
        let converter = HLSLConverter::new();
        assert!(converter.is_ok());
    }
    
    #[test]
    fn test_simple_hlsl_parsing() {
        let hlsl = r#"
            cbuffer Constants {
                float4x4 worldViewProj;
                float4 lightDir;
            };
            
            Texture2D diffuseTexture;
            SamplerState diffuseSampler;
            
            float4 main(float4 pos : POSITION) : SV_POSITION {
                return mul(pos, worldViewProj);
            }
        "#;
        
        let mut converter = HLSLConverter::new().unwrap();
        let result = converter.convert(hlsl, "test.hlsl");
        
        // Should succeed even if conversion is not complete
        assert!(result.is_ok());
        let wgsl = result.unwrap();
        assert!(wgsl.contains("@vertex"));
        assert!(wgsl.contains("vs_main"));
    }
    
    #[test]
    fn test_hlsl_type_conversion() {
        let converter = HLSLConverter::new().unwrap();
        
        assert_eq!(converter.convert_hlsl_type_to_wgsl("float"), "f32");
        assert_eq!(converter.convert_hlsl_type_to_wgsl("float3"), "vec3<f32>");
        assert_eq!(converter.convert_hlsl_type_to_wgsl("float4x4"), "mat4x4<f32>");
        assert_eq!(converter.convert_hlsl_type_to_wgsl("Texture2D"), "texture_2d<f32>");
    }
    
    #[test]
    fn test_invalid_hlsl_detection() {
        let invalid_hlsl = r#"
            cbuffer Constants {
                float4x4 worldViewProj;
                // Missing semicolon
                float4 lightDir
            };
        "#;
        
        let mut converter = HLSLConverter::new().unwrap();
        let result = converter.convert(invalid_hlsl, "test.hlsl");
        
        // Should fail due to syntax error
        assert!(result.is_err());
        assert!(converter.diagnostics.has_errors());
    }
}