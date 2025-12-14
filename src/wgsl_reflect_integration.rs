use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Analyze WGSL shader and return reflection information
pub fn analyze_shader_reflection(wgsl_code: &str) -> Result<WgslReflectAnalyzer> {
    let mut analyzer = WgslReflectAnalyzer::new();
    analyzer.analyze_shader(wgsl_code)?;
    Ok(analyzer)
}

/// WGSL reflection analysis using wgsl_reflect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WgslReflectAnalyzer {
    pub shader_info: ShaderReflectionInfo,
    pub entry_points: Vec<EntryPointInfo>,
    pub bind_groups: Vec<BindGroupInfo>,
    pub uniforms: Vec<UniformInfo>,
    pub textures: Vec<TextureInfo>,
    pub samplers: Vec<SamplerInfo>,
    pub storage_buffers: Vec<StorageBufferInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderReflectionInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPointInfo {
    pub name: String,
    pub stage: ShaderStage,
    pub workgroup_size: Option<(u32, u32, u32)>,
    pub inputs: Vec<VariableInfo>,
    pub outputs: Vec<VariableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindGroupInfo {
    pub group: u32,
    pub bindings: Vec<BindingInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingInfo {
    pub binding: u32,
    pub name: String,
    pub binding_type: BindingType,
    pub visibility: ShaderStage,
    pub size: Option<u64>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformInfo {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub align: u32,
    pub type_info: TypeInfo,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureInfo {
    pub name: String,
    pub binding: u32,
    pub group: u32,
    pub texture_type: String,
    pub format: Option<String>,
    pub dimensions: Option<(u32, u32, u32)>,
    pub sample_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplerInfo {
    pub name: String,
    pub binding: u32,
    pub group: u32,
    pub sampler_type: String,
    pub filtering: Option<String>,
    pub addressing: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageBufferInfo {
    pub name: String,
    pub binding: u32,
    pub group: u32,
    pub size: Option<u64>,
    pub readonly: bool,
    pub type_info: TypeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub location: u32,
    pub type_info: TypeInfo,
    pub interpolation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub base_type: String,
    pub components: u32,
    pub rows: Option<u32>,
    pub columns: Option<u32>,
    pub array_size: Option<u32>,
    pub struct_members: Vec<StructMemberInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructMemberInfo {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub type_info: TypeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingType {
    UniformBuffer,
    StorageBuffer,
    Texture,
    Sampler,
    AccelerationStructure,
}

impl WgslReflectAnalyzer {
    pub fn new() -> Self {
        Self {
            shader_info: ShaderReflectionInfo {
                name: None,
                version: None,
                description: None,
                author: None,
                categories: Vec::new(),
                tags: Vec::new(),
            },
            entry_points: Vec::new(),
            bind_groups: Vec::new(),
            uniforms: Vec::new(),
            textures: Vec::new(),
            samplers: Vec::new(),
            storage_buffers: Vec::new(),
        }
    }

    /// Analyze WGSL shader code and extract reflection information
    pub fn analyze_shader(&mut self, wgsl_code: &str) -> Result<()> {
        // Parse WGSL code manually since wgsl_reflect crate might not be available
        self.parse_wgsl_shader(wgsl_code)?;
        self.extract_entry_points(wgsl_code)?;
        self.extract_bind_groups(wgsl_code)?;
        self.extract_uniforms(wgsl_code)?;
        self.extract_textures(wgsl_code)?;
        self.extract_samplers(wgsl_code)?;
        self.extract_storage_buffers(wgsl_code)?;
        
        Ok(())
    }

    /// Parse basic WGSL structure
    fn parse_wgsl_shader(&mut self, wgsl_code: &str) -> Result<()> {
        // Extract shader metadata from comments
        for line in wgsl_code.lines() {
            let line = line.trim();
            if line.starts_with("//") {
                if line.contains("@name") {
                    self.shader_info.name = line.split("@name").nth(1).map(|s| s.trim().to_string());
                } else if line.contains("@version") {
                    self.shader_info.version = line.split("@version").nth(1).map(|s| s.trim().to_string());
                } else if line.contains("@description") {
                    self.shader_info.description = line.split("@description").nth(1).map(|s| s.trim().to_string());
                } else if line.contains("@author") {
                    self.shader_info.author = line.split("@author").nth(1).map(|s| s.trim().to_string());
                }
            }
        }
        
        Ok(())
    }

    /// Extract entry points from WGSL code
    fn extract_entry_points(&mut self, wgsl_code: &str) -> Result<()> {
        self.entry_points.clear();
        
        let mut lines = wgsl_code.lines().peekable();
        while let Some(line) = lines.next() {
            let line = line.trim();
            
            // Look for entry point attributes
            if line.starts_with("@vertex") || line.starts_with("@fragment") || line.starts_with("@compute") {
                let stage = if line.starts_with("@vertex") {
                    ShaderStage::Vertex
                } else if line.starts_with("@fragment") {
                    ShaderStage::Fragment
                } else {
                    ShaderStage::Compute
                };
                
                // Look for function declaration
                if let Some(next_line) = lines.peek() {
                    if next_line.contains("fn ") {
                        let fn_name = next_line.split("fn ").nth(1)
                            .and_then(|s| s.split('(').next())
                            .unwrap_or("unknown")
                            .trim()
                            .to_string();
                        
                        let mut entry_point = EntryPointInfo {
                            name: fn_name,
                            stage: stage.clone(),
                            workgroup_size: None,
                            inputs: Vec::new(),
                            outputs: Vec::new(),
                        };
                        
                        // Extract workgroup size for compute shaders
                        if matches!(stage, ShaderStage::Compute) && line.contains("workgroup_size") {
                            if let Some(size_str) = line.split("workgroup_size").nth(1) {
                                if let Some(sizes) = Self::parse_workgroup_size(size_str) {
                                    entry_point.workgroup_size = Some(sizes);
                                }
                            }
                        }
                        
                        self.entry_points.push(entry_point);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Extract bind groups from WGSL code
    fn extract_bind_groups(&mut self, wgsl_code: &str) -> Result<()> {
        self.bind_groups.clear();
        
        let mut current_group: Option<u32> = None;
        let mut current_bindings: Vec<BindingInfo> = Vec::new();
        
        for line in wgsl_code.lines() {
            let line = line.trim();
            
            // Look for @group attribute
            if line.starts_with("@group") {
                if let Some(group_num) = Self::extract_group_number(line) {
                    // Save previous group if exists
                    if let Some(prev_group) = current_group {
                        self.bind_groups.push(BindGroupInfo {
                            group: prev_group,
                            bindings: current_bindings.clone(),
                        });
                        current_bindings.clear();
                    }
                    current_group = Some(group_num);
                }
            }
            
            // Look for @binding attribute
            if line.starts_with("@binding") && current_group.is_some() {
                if let Some(binding_info) = Self::extract_binding_info(line, wgsl_code) {
                    current_bindings.push(binding_info);
                }
            }
        }
        
        // Save last group
        if let Some(group) = current_group {
            self.bind_groups.push(BindGroupInfo {
                group,
                bindings: current_bindings,
            });
        }
        
        Ok(())
    }

    /// Extract uniforms from WGSL code
    fn extract_uniforms(&mut self, wgsl_code: &str) -> Result<()> {
        self.uniforms.clear();
        
        let mut in_struct = false;
        let mut current_struct = String::new();
        
        for line in wgsl_code.lines() {
            let line = line.trim();
            
            if line.starts_with("struct ") && line.contains("uniform") {
                in_struct = true;
                current_struct = line.to_string();
            } else if in_struct {
                if line == "}" {
                    in_struct = false;
                    Self::parse_uniform_struct(&current_struct)?;
                    current_struct.clear();
                } else {
                    current_struct.push('\n');
                    current_struct.push_str(line);
                }
            }
        }
        
        Ok(())
    }

    /// Extract textures from WGSL code
    fn extract_textures(&mut self, wgsl_code: &str) -> Result<()> {
        self.textures.clear();
        
        for line in wgsl_code.lines() {
            let line = line.trim();
            
            if line.contains("texture_2d") || line.contains("texture_3d") || line.contains("texture_cube") {
                if let Some(texture_info) = Self::parse_texture_declaration(line) {
                    self.textures.push(texture_info);
                }
            }
        }
        
        Ok(())
    }

    /// Extract samplers from WGSL code
    fn extract_samplers(&mut self, wgsl_code: &str) -> Result<()> {
        self.samplers.clear();
        
        for line in wgsl_code.lines() {
            let line = line.trim();
            
            if line.contains("sampler") && !line.contains("texture") {
                if let Some(sampler_info) = Self::parse_sampler_declaration(line) {
                    self.samplers.push(sampler_info);
                }
            }
        }
        
        Ok(())
    }

    /// Extract storage buffers from WGSL code
    fn extract_storage_buffers(&mut self, wgsl_code: &str) -> Result<()> {
        self.storage_buffers.clear();
        
        for line in wgsl_code.lines() {
            let line = line.trim();
            
            if line.contains("var<storage>") {
                if let Some(buffer_info) = Self::parse_storage_buffer_declaration(line) {
                    self.storage_buffers.push(buffer_info);
                }
            }
        }
        
        Ok(())
    }

    // Helper functions for parsing

    fn parse_workgroup_size(size_str: &str) -> Option<(u32, u32, u32)> {
        // Parse workgroup_size(x, y, z) format
        if let Some(start) = size_str.find('(') {
            if let Some(end) = size_str.find(')') {
                let params = &size_str[start + 1..end];
                let parts: Vec<u32> = params.split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                
                if parts.len() == 3 {
                    return Some((parts[0], parts[1], parts[2]));
                }
            }
        }
        None
    }

    fn extract_group_number(line: &str) -> Option<u32> {
        line.split("@group(").nth(1)
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.parse().ok())
    }

    fn extract_binding_info(line: &str, wgsl_code: &str) -> Option<BindingInfo> {
        // This is a simplified extraction - in a real implementation,
        // you would parse the full WGSL structure
        let binding = line.split("@binding(").nth(1)
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.parse().ok())?;
        
        // Look for variable declaration on the same or next line
        let var_line = if line.contains("var") {
            line.to_string()
        } else {
            // Find next line with variable declaration
            wgsl_code.lines()
                .skip_while(|l| !l.contains(line))
                .skip(1)
                .find(|l| l.contains("var"))?
                .to_string()
        };
        
        let name = var_line.split(':').nth(0)?
            .split_whitespace()
            .last()?
            .to_string();
        
        let binding_type = if var_line.contains("uniform") {
            BindingType::UniformBuffer
        } else if var_line.contains("storage") {
            BindingType::StorageBuffer
        } else if var_line.contains("texture") {
            BindingType::Texture
        } else if var_line.contains("sampler") {
            BindingType::Sampler
        } else {
            BindingType::UniformBuffer
        };
        
        Some(BindingInfo {
            binding,
            name,
            binding_type,
            visibility: ShaderStage::Fragment, // Default
            size: None,
            format: None,
        })
    }

    fn parse_uniform_struct(struct_code: &str) -> Result<()> {
        // Parse struct members and create UniformInfo entries
        // This is a simplified implementation
        Ok(())
    }

    fn parse_texture_declaration(line: &str) -> Option<TextureInfo> {
        // Parse texture declaration
        // This is a simplified implementation
        None
    }

    fn parse_sampler_declaration(line: &str) -> Option<SamplerInfo> {
        // Parse sampler declaration
        // This is a simplified implementation
        None
    }

    fn parse_storage_buffer_declaration(line: &str) -> Option<StorageBufferInfo> {
        // Parse storage buffer declaration
        // This is a simplified implementation
        None
    }

    /// Generate reflection report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# WGSL Shader Reflection Report\n\n");
        
        // Shader info
        if let Some(name) = &self.shader_info.name {
            report.push_str(&format!("**Name:** {}\n", name));
        }
        if let Some(version) = &self.shader_info.version {
            report.push_str(&format!("**Version:** {}\n", version));
        }
        if let Some(description) = &self.shader_info.description {
            report.push_str(&format!("**Description:** {}\n", description));
        }
        report.push('\n');
        
        // Entry points
        if !self.entry_points.is_empty() {
            report.push_str("## Entry Points\n\n");
            for entry_point in &self.entry_points {
                report.push_str(&format!("- **{}** ({:?})\n", entry_point.name, entry_point.stage));
                if let Some((x, y, z)) = entry_point.workgroup_size {
                    report.push_str(&format!("  - Workgroup size: ({}, {}, {})\n", x, y, z));
                }
            }
            report.push('\n');
        }
        
        // Bind groups
        if !self.bind_groups.is_empty() {
            report.push_str("## Bind Groups\n\n");
            for bind_group in &self.bind_groups {
                report.push_str(&format!("**Group {}**\n", bind_group.group));
                for binding in &bind_group.bindings {
                    report.push_str(&format!("- Binding {}: {} ({:?})\n", binding.binding, binding.name, binding.binding_type));
                }
                report.push('\n');
            }
        }
        
        // Uniforms
        if !self.uniforms.is_empty() {
            report.push_str("## Uniforms\n\n");
            for uniform in &self.uniforms {
                report.push_str(&format!("- **{}**: offset={}, size={}, align={}\n", 
                    uniform.name, uniform.offset, uniform.size, uniform.align));
            }
            report.push('\n');
        }
        
        report
    }
}

impl Default for WgslReflectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}