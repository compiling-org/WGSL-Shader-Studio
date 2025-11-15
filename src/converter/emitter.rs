use std::collections::HashMap;
use anyhow::Result;
use crate::converter::diagnostics::{Diagnostics, DiagnosticSeverity};

/// WGSL code emitter with comment and uniform preservation
pub struct WgslEmitter {
    output: String,
    indent_level: usize,
    preserved_comments: Vec<String>,
    preserved_uniforms: HashMap<String, UniformInfo>,
    diagnostics: Diagnostics,
}

/// Information about a preserved uniform
#[derive(Debug, Clone)]
pub struct UniformInfo {
    pub name: String,
    pub wgsl_type: String,
    pub binding: u32,
    pub group: u32,
    pub original_type: String,
    pub original_declaration: String,
}

/// WGSL shader structure
#[derive(Debug, Clone)]
pub struct WgslShader {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub uniforms: HashMap<String, UniformInfo>,
    pub vertex_attributes: Vec<VertexAttribute>,
    pub fragment_outputs: Vec<FragmentOutput>,
}

/// Vertex attribute information
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub wgsl_type: String,
    pub location: u32,
}

/// Fragment output information
#[derive(Debug, Clone)]
pub struct FragmentOutput {
    pub name: String,
    pub wgsl_type: String,
    pub location: u32,
}

impl WgslEmitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            preserved_comments: Vec::new(),
            preserved_uniforms: HashMap::new(),
            diagnostics: Diagnostics::new(),
        }
    }
    
    /// Set preserved comments from original shader
    pub fn set_preserved_comments(&mut self, comments: Vec<String>) {
        self.preserved_comments = comments;
    }
    
    /// Add preserved uniform
    pub fn add_preserved_uniform(
        &mut self,
        name: String,
        wgsl_type: String,
        binding: u32,
        group: u32,
        original_type: String,
        original_declaration: String,
    ) {
        let info = UniformInfo {
            name: name.clone(),
            wgsl_type,
            binding,
            group,
            original_type,
            original_declaration,
        };
        
        self.preserved_uniforms.insert(name, info);
    }
    
    /// Emit WGSL shader from conversion result
    pub fn emit_shader(
        &mut self,
        shader_type: ShaderType,
        uniforms: &HashMap<String, UniformInfo>,
        functions: &[FunctionInfo],
        global_vars: &[GlobalVarInfo],
        main_body: &str,
    ) -> Result<String> {
        self.output.clear();
        self.indent_level = 0;
        
        // Emit header with preserved comments
        self.emit_header(shader_type)?;
        
        // Emit uniforms
        self.emit_uniforms(uniforms)?;
        
        // Emit global variables
        self.emit_global_vars(global_vars)?;
        
        // Emit functions
        self.emit_functions(functions)?;
        
        // Emit main function
        self.emit_main_function(shader_type, main_body)?;
        
        Ok(self.output.clone())
    }
    
    /// Emit shader header with comments
    fn emit_header(&mut self, shader_type: ShaderType) -> Result<()> {
        // Add auto-generated comment
        self.output.push_str("// Auto-generated WGSL shader\n");
        self.output.push_str(&format!("// Type: {:?}\n", shader_type));
        self.output.push_str("// Converted from ISF/GLSL/HLSL\n\n");
        
        // Add preserved comments
        for comment in &self.preserved_comments {
            self.output.push_str(comment);
            self.output.push('\n');
        }
        
        if !self.preserved_comments.is_empty() {
            self.output.push('\n');
        }
        
        Ok(())
    }
    
    /// Emit uniform declarations
    fn emit_uniforms(&mut self, uniforms: &HashMap<String, UniformInfo>) -> Result<()> {
        if uniforms.is_empty() && self.preserved_uniforms.is_empty() {
            return Ok(());
        }
        
        self.output.push_str("// Uniforms\n");
        
        // Group uniforms by binding group
        let mut groups: HashMap<u32, Vec<&UniformInfo>> = HashMap::new();
        
        for uniform in uniforms.values() {
            groups.entry(uniform.group).or_default().push(uniform);
        }
        
        for uniform in self.preserved_uniforms.values() {
            groups.entry(uniform.group).or_default().push(uniform);
        }
        
        // Emit each group
        let mut sorted_groups: Vec<_> = groups.into_iter().collect();
        sorted_groups.sort_by_key(|(group, _)| *group);
        
        for (group, uniforms) in sorted_groups {
            self.output.push_str(&format!("@group({}) ", group));
            
            let mut sorted_uniforms = uniforms;
            sorted_uniforms.sort_by_key(|u| u.binding);
            
            for (i, uniform) in sorted_uniforms.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.output.push_str(&format!(
                    "@binding({}) var<uniform> {}: {}",
                    uniform.binding,
                    uniform.name,
                    uniform.wgsl_type
                ));
            }
            
            self.output.push_str(";\n");
        }
        
        self.output.push('\n');
        Ok(())
    }
    
    /// Emit global variables
    fn emit_global_vars(&mut self, global_vars: &[GlobalVarInfo]) -> Result<()> {
        if global_vars.is_empty() {
            return Ok(());
        }
        
        self.output.push_str("// Global variables\n");
        
        for var in global_vars {
            self.output.push_str(&format!(
                "var<{}> {}: {};\n",
                var.storage_class,
                var.name,
                var.wgsl_type
            ));
        }
        
        self.output.push('\n');
        Ok(())
    }
    
    /// Emit functions
    fn emit_functions(&mut self, functions: &[FunctionInfo]) -> Result<()> {
        if functions.is_empty() {
            return Ok(());
        }
        
        self.output.push_str("// Functions\n");
        
        for function in functions {
            self.emit_function(function)?;
            self.output.push('\n');
        }
        
        Ok(())
    }
    
    /// Emit a single function
    fn emit_function(&mut self, function: &FunctionInfo) -> Result<()> {
        // Function header
        self.output.push_str(&format!("fn {}(", function.name));
        
        // Parameters
        for (i, param) in function.parameters.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&format!("{}: {}", param.name, param.wgsl_type));
        }
        
        self.output.push_str(&format!(") -> {} {{\n",
            function.return_type
        ));
        
        // Function body
        self.indent_level += 1;
        for line in function.body.lines() {
            self.output.push_str(&"    ".repeat(self.indent_level));
            self.output.push_str(line);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        
        self.output.push_str("}\n");
        
        Ok(())
    }
    
    /// Emit main function
    fn emit_main_function(&mut self, shader_type: ShaderType, main_body: &str) -> Result<()> {
        match shader_type {
            ShaderType::Vertex => {
                self.output.push_str("// Vertex shader main function\n");
                self.output.push_str("@vertex\n");
                self.output.push_str("fn main(\n");
                self.output.push_str("    @location(0) position: vec3<f32>,\n");
                self.output.push_str("    @location(1) uv: vec2<f32>,\n");
                self.output.push_str(") -> @builtin(position) vec4<f32> {\n");
                
                self.indent_level += 1;
                self.output.push_str(&"    ".repeat(self.indent_level));
                self.output.push_str("// Vertex shader logic\n");
                
                // Add main body with proper indentation
                for line in main_body.lines() {
                    self.output.push_str(&"    ".repeat(self.indent_level));
                    self.output.push_str(line);
                    self.output.push('\n');
                }
                
                self.output.push_str(&"    ".repeat(self.indent_level));
                self.output.push_str("return vec4<f32>(position, 1.0);\n");
                self.indent_level -= 1;
                
                self.output.push_str("}\n");
            }
            ShaderType::Fragment => {
                self.output.push_str("// Fragment shader main function\n");
                self.output.push_str("@fragment\n");
                self.output.push_str("fn main(\n");
                self.output.push_str("    @location(0) uv: vec2<f32>,\n");
                self.output.push_str(") -> @location(0) vec4<f32> {\n");
                
                self.indent_level += 1;
                self.output.push_str(&"    ".repeat(self.indent_level));
                self.output.push_str("// Fragment shader logic\n");
                
                // Add main body with proper indentation
                for line in main_body.lines() {
                    self.output.push_str(&"    ".repeat(self.indent_level));
                    self.output.push_str(line);
                    self.output.push('\n');
                }
                
                self.output.push_str(&"    ".repeat(self.indent_level));
                self.output.push_str("return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Default red color\n");
                self.indent_level -= 1;
                
                self.output.push_str("}\n");
            }
        }
        
        Ok(())
    }
    
    /// Convert GLSL type to WGSL type
    pub fn glsl_type_to_wgsl(glsl_type: &str) -> String {
        match glsl_type {
            "void" => "()".to_string(),
            "bool" => "bool".to_string(),
            "int" => "i32".to_string(),
            "uint" => "u32".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            
            // Vector types
            "vec2" => "vec2<f32>".to_string(),
            "vec3" => "vec3<f32>".to_string(),
            "vec4" => "vec4<f32>".to_string(),
            "bvec2" => "vec2<bool>".to_string(),
            "bvec3" => "vec3<bool>".to_string(),
            "bvec4" => "vec4<bool>".to_string(),
            "ivec2" => "vec2<i32>".to_string(),
            "ivec3" => "vec3<i32>".to_string(),
            "ivec4" => "vec4<i32>".to_string(),
            "uvec2" => "vec2<u32>".to_string(),
            "uvec3" => "vec3<u32>".to_string(),
            "uvec4" => "vec4<u32>".to_string(),
            "dvec2" => "vec2<f64>".to_string(),
            "dvec3" => "vec3<f64>".to_string(),
            "dvec4" => "vec4<f64>".to_string(),
            
            // Matrix types
            "mat2" => "mat2x2<f32>".to_string(),
            "mat3" => "mat3x3<f32>".to_string(),
            "mat4" => "mat4x4<f32>".to_string(),
            "mat2x2" => "mat2x2<f32>".to_string(),
            "mat2x3" => "mat2x3<f32>".to_string(),
            "mat2x4" => "mat2x4<f32>".to_string(),
            "mat3x2" => "mat3x2<f32>".to_string(),
            "mat3x3" => "mat3x3<f32>".to_string(),
            "mat3x4" => "mat3x4<f32>".to_string(),
            "mat4x2" => "mat4x2<f32>".to_string(),
            "mat4x3" => "mat4x3<f32>".to_string(),
            "mat4x4" => "mat4x4<f32>".to_string(),
            
            // Sampler types
            "sampler2D" => "texture_2d<f32>".to_string(),
            "samplerCube" => "texture_cube<f32>".to_string(),
            "sampler2DArray" => "texture_2d_array<f32>".to_string(),
            "sampler3D" => "texture_3d<f32>".to_string(),
            
            // Default fallback
            _ => {
                if glsl_type.starts_with("sampler") {
                    "texture_2d<f32>".to_string()
                } else {
                    format!("/* Unknown GLSL type: {} */ f32", glsl_type)
                }
            }
        }
    }
    
    /// Convert HLSL type to WGSL type
    pub fn hlsl_type_to_wgsl(hlsl_type: &str) -> String {
        match hlsl_type {
            "void" => "()".to_string(),
            "bool" => "bool".to_string(),
            "int" => "i32".to_string(),
            "uint" => "u32".to_string(),
            "dword" => "u32".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            
            // Vector types
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
            "double2" => "vec2<f64>".to_string(),
            "double3" => "vec3<f64>".to_string(),
            "double4" => "vec4<f64>".to_string(),
            
            // Matrix types
            "float2x2" => "mat2x2<f32>".to_string(),
            "float2x3" => "mat2x3<f32>".to_string(),
            "float2x4" => "mat2x4<f32>".to_string(),
            "float3x2" => "mat3x2<f32>".to_string(),
            "float3x3" => "mat3x3<f32>".to_string(),
            "float3x4" => "mat3x4<f32>".to_string(),
            "float4x2" => "mat4x2<f32>".to_string(),
            "float4x3" => "mat4x3<f32>".to_string(),
            "float4x4" => "mat4x4<f32>".to_string(),
            
            // Texture types
            "Texture2D" => "texture_2d<f32>".to_string(),
            "TextureCube" => "texture_cube<f32>".to_string(),
            "Texture2DArray" => "texture_2d_array<f32>".to_string(),
            "Texture3D" => "texture_3d<f32>".to_string(),
            
            // Default fallback
            _ => format!("/* Unknown HLSL type: {} */ f32", hlsl_type),
        }
    }
    
    /// Convert ISF type to WGSL type
    pub fn isf_type_to_wgsl(isf_type: &str) -> String {
        match isf_type {
            "event" => "f32".to_string(),
            "bool" => "bool".to_string(),
            "long" => "i32".to_string(),
            "float" => "f32".to_string(),
            "point2D" => "vec2<f32>".to_string(),
            "color" => "vec4<f32>".to_string(),
            "image" => "texture_2d<f32>".to_string(),
            "audio" => "texture_2d<f32>".to_string(),
            "audiofft" => "texture_2d<f32>".to_string(),
            _ => Self::glsl_type_to_wgsl(isf_type),
        }
    }
    
    /// Get the diagnostics collected during emission
    pub fn get_diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
    
    /// Get mutable diagnostics
    pub fn get_diagnostics_mut(&mut self) -> &mut Diagnostics {
        &mut self.diagnostics
    }
}

/// Shader type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

/// Function information
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<ParameterInfo>,
    pub body: String,
}

/// Parameter information
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub wgsl_type: String,
}

/// Global variable information
#[derive(Debug, Clone)]
pub struct GlobalVarInfo {
    pub name: String,
    pub wgsl_type: String,
    pub storage_class: String,
    pub initial_value: Option<String>,
}

impl Default for WgslEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glsl_type_conversion() {
        assert_eq!(WgslEmitter::glsl_type_to_wgsl("void"), "()");
        assert_eq!(WgslEmitter::glsl_type_to_wgsl("float"), "f32");
        assert_eq!(WgslEmitter::glsl_type_to_wgsl("vec3"), "vec3<f32>");
        assert_eq!(WgslEmitter::glsl_type_to_wgsl("mat4"), "mat4x4<f32>");
        assert_eq!(WgslEmitter::glsl_type_to_wgsl("sampler2D"), "texture_2d<f32>");
    }
    
    #[test]
    fn test_hlsl_type_conversion() {
        assert_eq!(WgslEmitter::hlsl_type_to_wgsl("void"), "()");
        assert_eq!(WgslEmitter::hlsl_type_to_wgsl("float"), "f32");
        assert_eq!(WgslEmitter::hlsl_type_to_wgsl("float3"), "vec3<f32>");
        assert_eq!(WgslEmitter::hlsl_type_to_wgsl("float4x4"), "mat4x4<f32>");
        assert_eq!(WgslEmitter::hlsl_type_to_wgsl("Texture2D"), "texture_2d<f32>");
    }
    
    #[test]
    fn test_isf_type_conversion() {
        assert_eq!(WgslEmitter::isf_type_to_wgsl("event"), "f32");
        assert_eq!(WgslEmitter::isf_type_to_wgsl("point2D"), "vec2<f32>");
        assert_eq!(WgslEmitter::isf_type_to_wgsl("color"), "vec4<f32>");
        assert_eq!(WgslEmitter::isf_type_to_wgsl("image"), "texture_2d<f32>");
    }
    
    #[test]
    fn test_emitter_creation() {
        let emitter = WgslEmitter::new();
        assert!(emitter.preserved_comments.is_empty());
        assert!(emitter.preserved_uniforms.is_empty());
        assert!(!emitter.get_diagnostics().has_errors());
    }
}