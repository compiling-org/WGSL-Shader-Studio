//! WGSL Bindgen Integration Module
//! Provides uniform layout analysis and code generation using naga for proper WGSL parsing

use naga::{Module, GlobalVariable, Type, TypeInner, AddressSpace};
use std::collections::HashMap;
use anyhow::Result;

/// Structure to hold uniform layout information
#[derive(Debug, Clone)]
pub struct UniformLayout {
    pub name: String,
    pub binding: u32,
    pub group: u32,
    pub size: usize,
    pub fields: Vec<UniformField>,
}

#[derive(Debug, Clone)]
pub struct UniformField {
    pub name: String,
    pub ty: String,
    pub offset: usize,
    pub size: usize,
    pub alignment: usize,
}

/// WGSL Bindgen analyzer for shader uniform layouts
pub struct WgslBindgenAnalyzer {
    layouts: HashMap<String, UniformLayout>,
}

impl WgslBindgenAnalyzer {
    pub fn new() -> Self {
        Self {
            layouts: HashMap::new(),
        }
    }

    /// Analyze WGSL code and extract uniform layouts using naga
    pub fn analyze_shader(&mut self, wgsl_code: &str, shader_name: &str) -> Result<Vec<UniformLayout>> {
        // Parse WGSL using naga for proper analysis
        let module = naga::front::wgsl::parse_str(wgsl_code)?;
        let layouts = self.extract_uniforms_from_module(&module)?;
        
        // Store layouts for this shader
        for layout in &layouts {
            self.layouts.insert(format!("{}::{}", shader_name, layout.name), layout.clone());
        }
        
        Ok(layouts)
    }
    
    /// Extract uniform layouts from naga module
    fn extract_uniforms_from_module(&self, module: &Module) -> Result<Vec<UniformLayout>> {
        let mut layouts = Vec::new();
        
        // Iterate through global variables to find uniforms
        for (handle, var) in module.global_variables.iter() {
            if var.space == AddressSpace::Uniform {
                if let Some(type_info) = self.extract_uniform_info(module, var) {
                    layouts.push(type_info);
                }
            }
        }
        
        Ok(layouts)
    }
    
    /// Extract uniform information from a global variable
    fn extract_uniform_info(&self, module: &Module, var: &GlobalVariable) -> Option<UniformLayout> {
        let type_handle = var.ty;
        let type_info = &module.types[type_handle];
        
        match &type_info.inner {
            TypeInner::Struct { members, .. } => {
                let mut fields = Vec::new();
                
                for (i, member) in members.iter().enumerate() {
                    let member_type = &module.types[member.ty];
                    let (ty, size) = self.get_type_info(member_type)?;
                    
                    fields.push(UniformField {
                        name: member.name.clone().unwrap_or_else(|| format!("field_{}", i)),
                        ty,
                        offset: 0, // Will be calculated later
                        size,
                        alignment: size,
                    });
                }
                
                let total_size = fields.iter().map(|f| f.size).sum::<usize>().max(256);
                
                Some(UniformLayout {
                    name: var.name.clone().unwrap_or_else(|| "Uniforms".to_string()),
                    binding: var.binding.as_ref()?.binding,
                    group: var.binding.as_ref()?.group,
                    size: total_size,
                    fields,
                })
            }
            _ => None,
        }
    }
    
    /// Get type information from naga type
    fn get_type_info(&self, type_info: &Type) -> Option<(String, usize)> {
        match &type_info.inner {
            TypeInner::Scalar(scalar) => {
                match scalar.kind {
                    naga::ScalarKind::Float => Some(("f32".to_string(), scalar.width as usize)),
                    naga::ScalarKind::Sint => Some(("i32".to_string(), scalar.width as usize)),
                    naga::ScalarKind::Uint => Some(("u32".to_string(), scalar.width as usize)),
                    naga::ScalarKind::Bool => Some(("bool".to_string(), scalar.width as usize)),
                    naga::ScalarKind::AbstractInt => Some(("i32".to_string(), scalar.width as usize)),
                    naga::ScalarKind::AbstractFloat => Some(("f32".to_string(), scalar.width as usize)),
                }
            }
            TypeInner::Vector { size, scalar } => {
                let base_type = match scalar.kind {
                    naga::ScalarKind::Float => "f32",
                    naga::ScalarKind::Sint => "i32",
                    naga::ScalarKind::Uint => "u32",
                    naga::ScalarKind::Bool => "bool",
                    naga::ScalarKind::AbstractInt => "i32",
                    naga::ScalarKind::AbstractFloat => "f32",
                };
                let vec_size = match *size {
                    naga::VectorSize::Bi => 2,
                    naga::VectorSize::Tri => 3,
                    naga::VectorSize::Quad => 4,
                };
                Some((format!("vec{}<{}>", vec_size, base_type), (scalar.width as usize) * vec_size))
            }
            TypeInner::Matrix { columns, rows, scalar } => {
                let total_size = (*columns as usize) * (*rows as usize) * (scalar.width as usize);
                Some((format!("mat{}x{}<f32>", *columns as usize, *rows as usize), total_size))
            }
            _ => None,
        }
    }
    
    /// Parse WGSL code manually to extract uniform information (fallback method)
    fn parse_wgsl_for_uniforms(&self, wgsl_code: &str) -> Result<Vec<UniformLayout>> {
        let mut layouts = Vec::new();
        let lines: Vec<&str> = wgsl_code.lines().collect();
        
        let mut current_struct: Option<(String, u32, u32)> = None;
        let mut current_fields = Vec::new();
        
        for line in lines {
            let trimmed = line.trim();
            
            // Look for struct definitions
            if trimmed.starts_with("struct ") {
                let struct_name = trimmed[7..].trim().trim_end_matches('{').trim();
                current_struct = Some((struct_name.to_string(), 0, 0));
                current_fields.clear();
            }
            // Look for struct fields
            else if let Some((ref struct_name, group, binding)) = current_struct {
                if trimmed.contains(':') && !trimmed.starts_with("//") {
                    let parts: Vec<&str> = trimmed.split(':').collect();
                    if parts.len() == 2 {
                        let field_name = parts[0].trim();
                        let field_type = parts[1].trim().trim_end_matches(',').trim();
                        
                        current_fields.push(UniformField {
                            name: field_name.to_string(),
                            ty: field_type.to_string(),
                            offset: current_fields.len() * 16, // Simplified offset calculation
                            size: self.get_type_size(field_type),
                            alignment: self.get_type_alignment(field_type),
                        });
                    }
                }
                // Look for uniform variable declarations
                else if trimmed.starts_with("@group(") && trimmed.contains("var<uniform>") {
                    // Extract group and binding info
                    if let Some(group_match) = self.extract_group_binding(trimmed) {
                        if let Some((struct_name, _, _)) = current_struct.take() {
                            let total_size = current_fields.iter().map(|f| f.size).sum::<usize>().max(256);
                            
                            layouts.push(UniformLayout {
                                name: struct_name.clone(),
                                binding: group_match.1,
                                group: group_match.0,
                                size: total_size,
                                fields: current_fields.clone(),
                            });
                        }
                    }
                }
            }
        }
        
        // Handle any remaining struct
        if let Some((struct_name, _, _)) = current_struct {
            if !current_fields.is_empty() {
                let total_size = current_fields.iter().map(|f| f.size).sum::<usize>().max(256);
                
                layouts.push(UniformLayout {
                    name: struct_name,
                    binding: 0,
                    group: 0,
                    size: total_size,
                    fields: current_fields,
                });
            }
        }
        
        Ok(layouts)
    }
    
    fn extract_group_binding(&self, line: &str) -> Option<(u32, u32)> {
        // Simple regex-like parsing for @group(X) @binding(Y)
        let mut group = 0;
        let mut binding = 0;
        
        if let Some(start) = line.find("@group(") {
            if let Some(end) = line[start..].find(')') {
                if let Ok(g) = line[start + 7..start + end].parse::<u32>() {
                    group = g;
                }
            }
        }
        
        if let Some(start) = line.find("@binding(") {
            if let Some(end) = line[start..].find(')') {
                if let Ok(b) = line[start + 9..start + end].parse::<u32>() {
                    binding = b;
                }
            }
        }
        
        Some((group, binding))
    }
    
    fn get_type_size(&self, wgsl_type: &str) -> usize {
        match wgsl_type {
            "f32" | "i32" | "u32" => 4,
            "vec2<f32>" | "vec2<i32>" | "vec2<u32>" => 8,
            "vec3<f32>" | "vec3<i32>" | "vec3<u32>" => 12,
            "vec4<f32>" | "vec4<i32>" | "vec4<u32>" => 16,
            "mat2x2<f32>" => 16,
            "mat3x3<f32>" => 48,
            "mat4x4<f32>" => 64,
            _ => 16, // Default to 16 bytes
        }
    }
    
    fn get_type_alignment(&self, wgsl_type: &str) -> usize {
        match wgsl_type {
            "f32" | "i32" | "u32" => 4,
            "vec2<f32>" | "vec2<i32>" | "vec2<u32>" => 8,
            "vec3<f32>" | "vec3<i32>" | "vec3<u32>" => 16, // vec3 has 16-byte alignment
            "vec4<f32>" | "vec4<i32>" | "vec4<u32>" => 16,
            "mat2x2<f32>" => 8,
            "mat3x3<f32>" => 16,
            "mat4x4<f32>" => 16,
            _ => 16,
        }
    }



    /// Generate Rust code for uniform buffer structures
    pub fn generate_rust_code(&self, shader_name: &str) -> Result<String> {
        let mut code = String::new();
        
        code.push_str(&format!("// Auto-generated uniform structures for {}\n\n", shader_name));
        code.push_str("use bytemuck::{Pod, Zeroable};\n");
        code.push_str("use std::mem;\n\n");
        
        // Generate structures for each uniform layout
        for (key, layout) in &self.layouts {
            if key.starts_with(shader_name) {
                code.push_str(&format!("#[repr(C)]\n"));
                code.push_str(&format!("#[derive(Debug, Clone, Copy, Pod, Zeroable)]\n"));
                code.push_str(&format!("pub struct {} {{\n", 
                    convert_case::Casing::to_case(&layout.name, convert_case::Case::Pascal)));
                
                for field in &layout.fields {
                    let rust_type = self.wgsl_type_to_rust(&field.ty);
                    code.push_str(&format!("    pub {}: {},\n", field.name, rust_type));
                }
                
                code.push_str("}\n\n");
                
                // Add implementation with size and alignment constants
                code.push_str(&format!("impl {} {{\n", convert_case::Casing::to_case(&layout.name, convert_case::Case::Pascal)));
                code.push_str(&format!("    pub const SIZE: usize = {};\n", layout.size));
                code.push_str(&format!("    pub const ALIGNMENT: usize = {};\n", 
                    layout.fields.iter().map(|f| f.alignment).max().unwrap_or(1)));
                code.push_str("}\n\n");
            }
        }
        
        Ok(code)
    }

    /// Convert WGSL types to Rust types
    fn wgsl_type_to_rust(&self, wgsl_type: &str) -> String {
        match wgsl_type {
            "f32" => "f32".to_string(),
            "i32" => "i32".to_string(),
            "u32" => "u32".to_string(),
            "bool" => "bool".to_string(),
            "vec2<f32>" => "[f32; 2]".to_string(),
            "vec3<f32>" => "[f32; 3]".to_string(),
            "vec4<f32>" => "[f32; 4]".to_string(),
            "vec2<i32>" => "[i32; 2]".to_string(),
            "vec3<i32>" => "[i32; 3]".to_string(),
            "vec4<i32>" => "[i32; 4]".to_string(),
            "vec2<u32>" => "[u32; 2]".to_string(),
            "vec3<u32>" => "[u32; 3]".to_string(),
            "vec4<u32>" => "[u32; 4]".to_string(),
            "mat2x2<f32>" => "[[f32; 2]; 2]".to_string(),
            "mat3x3<f32>" => "[[f32; 3]; 3]".to_string(),
            "mat4x4<f32>" => "[[f32; 4]; 4]".to_string(),
            _ => "[u8; 16]".to_string(), // Default to 16 bytes for unknown types
        }
    }

    /// Get uniform layout for a specific shader
    pub fn get_layout(&self, shader_name: &str, layout_name: &str) -> Option<&UniformLayout> {
        let key = format!("{}::{}", shader_name, layout_name);
        self.layouts.get(&key)
    }

    /// Get all layouts for a shader
    pub fn get_shader_layouts(&self, shader_name: &str) -> Vec<&UniformLayout> {
        self.layouts
            .iter()
            .filter(|(key, _)| key.starts_with(shader_name))
            .map(|(_, layout)| layout)
            .collect()
    }

    /// Validate uniform buffer data against layout
    pub fn validate_uniform_data(&self, layout: &UniformLayout, data: &[u8]) -> Result<bool> {
        if data.len() != layout.size {
            anyhow::bail!("Data size {} doesn't match expected size {}", data.len(), layout.size);
        }
        
        // Basic validation - could be enhanced with more sophisticated checks
        Ok(true)
    }

    /// Generate binding layout for WGPU
    pub fn generate_bind_group_layout(&self, shader_name: &str) -> Result<Vec<wgpu::BindGroupLayoutEntry>> {
        let layouts = self.get_shader_layouts(shader_name);
        let mut entries = Vec::new();
        
        for layout in layouts {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: layout.binding,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(layout.size as u64),
                },
                count: None,
            });
        }
        
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_layout_analysis() {
        let wgsl_code = r#"
            struct Uniforms {
                time: f32,
                resolution: vec2<f32>,
                mouse: vec2<f32>,
            }
            
            @group(0) @binding(0) var<uniform> uniforms: Uniforms;
            
            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(uniforms.time, 0.0, 0.0, 1.0);
            }
        "#;
        
        let mut analyzer = WgslBindgenAnalyzer::new();
        let layouts = analyzer.analyze_shader(wgsl_code, "test_shader").unwrap();
        
        assert!(!layouts.is_empty());
        assert_eq!(layouts[0].binding, 0);
        assert_eq!(layouts[0].group, 0);
    }

    #[test]
    fn test_rust_code_generation() {
        let mut analyzer = WgslBindgenAnalyzer::new();
        
        // Add a test layout
        let layout = UniformLayout {
            name: "test_uniforms".to_string(),
            binding: 0,
            group: 0,
            size: 32,
            fields: vec![
                UniformField {
                    name: "time".to_string(),
                    ty: "f32".to_string(),
                    offset: 0,
                    size: 4,
                    alignment: 4,
                },
                UniformField {
                    name: "resolution".to_string(),
                    ty: "vec2<f32>".to_string(),
                    offset: 8,
                    size: 8,
                    alignment: 8,
                },
            ],
        };
        
        analyzer.layouts.insert("test_shader::test_uniforms".to_string(), layout);
        
        let code = analyzer.generate_rust_code("test_shader").unwrap();
        assert!(code.contains("struct TestUniforms"));
        assert!(code.contains("time: f32"));
        assert!(code.contains("resolution: [f32; 2]"));
    }
}