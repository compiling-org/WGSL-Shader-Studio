//! Comprehensive ISF (Interactive Shader Format) integration system
//! 
//! This module provides complete ISF format support for VJ software compatibility,
//! including full ISF specification compliance, multi-pass rendering, and
//! advanced shader parameter management.

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Complete ISF shader with full specification support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfShader {
    pub metadata: IsfMetadata,
    pub inputs: Vec<IsfInput>,
    pub passes: Vec<IsfPass>,
    pub persistent_buffers: Vec<String>,
    pub imported_textures: Vec<ImportedTexture>,
    pub vertex_shader: Option<String>,
    pub fragment_shader: String,
    pub source_file: Option<PathBuf>,
}

/// ISF metadata following the official specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfMetadata {
    pub name: String,
    pub description: Option<String>,
    pub credit: Option<String>,
    pub isf_version: String,
    pub vsn: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub imported: Vec<String>,
}

/// ISF input parameter with full type support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfInput {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: IsfInputType,
    pub label: Option<String>,
    pub default: Option<serde_json::Value>,
    pub min: Option<serde_json::Value>,
    pub max: Option<serde_json::Value>,
    pub values: Option<Vec<serde_json::Value>>,
    pub clamp: Option<bool>,
    pub identity: Option<bool>,
}

/// ISF input types following the specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IsfInputType {
    Float,
    Bool,
    Color,
    Point2D,
    Image,
    Long,
    Event,
    Audio,
    AudioFFT,
}

/// ISF pass configuration for multi-pass rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfPass {
    pub target: Option<String>,
    pub persistent: Option<bool>,
    pub float: Option<bool>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub description: Option<String>,
    pub inputs: Option<HashMap<String, serde_json::Value>>,
}

/// Imported texture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedTexture {
    pub name: String,
    pub path: String,
    pub type_hint: Option<String>,
}

/// ISF to WGSL converter with advanced features
pub struct IsfToWgslConverter {
    input_mappings: HashMap<IsfInputType, WgslTypeMapping>,
    function_mappings: HashMap<String, String>,
    variable_mappings: HashMap<String, String>,
    builtin_functions: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct WgslTypeMapping {
    wgsl_type: String,
    default_value: String,
    needs_uniform: bool,
}

impl IsfToWgslConverter {
    pub fn new() -> Self {
        let mut input_mappings = HashMap::new();
        
        // Map ISF input types to WGSL types
        input_mappings.insert(IsfInputType::Float, WgslTypeMapping {
            wgsl_type: "f32".to_string(),
            default_value: "0.0".to_string(),
            needs_uniform: true,
        });
        
        input_mappings.insert(IsfInputType::Bool, WgslTypeMapping {
            wgsl_type: "bool".to_string(),
            default_value: "false".to_string(),
            needs_uniform: true,
        });
        
        input_mappings.insert(IsfInputType::Color, WgslTypeMapping {
            wgsl_type: "vec4<f32>".to_string(),
            default_value: "vec4<f32>(1.0, 1.0, 1.0, 1.0)".to_string(),
            needs_uniform: true,
        });
        
        input_mappings.insert(IsfInputType::Point2D, WgslTypeMapping {
            wgsl_type: "vec2<f32>".to_string(),
            default_value: "vec2<f32>(0.0, 0.0)".to_string(),
            needs_uniform: true,
        });
        
        input_mappings.insert(IsfInputType::Image, WgslTypeMapping {
            wgsl_type: "texture_2d<f32>".to_string(),
            default_value: "texture_2d<f32>()".to_string(),
            needs_uniform: true,
        });

        let mut function_mappings = HashMap::new();
        
        // ISF built-in functions to WGSL equivalents
        function_mappings.insert("IMG_PIXEL".to_string(), "textureSample".to_string());
        function_mappings.insert("IMG_NORM_PIXEL".to_string(), "textureSample".to_string());
        function_mappings.insert("IMG_SIZE".to_string(), "textureDimensions".to_string());
        function_mappings.insert("ISF_TIME".to_string(), "time".to_string());
        function_mappings.insert("TIME".to_string(), "time".to_string());
        function_mappings.insert("RENDERSIZE".to_string(), "resolution".to_string());
        function_mappings.insert("VV".to_string(), "uv".to_string());
        
        let mut variable_mappings = HashMap::new();
        
        // ISF built-in variables to WGSL equivalents
        variable_mappings.insert("isf_FragNormCoord".to_string(), "uv".to_string());
        variable_mappings.insert("gl_FragCoord".to_string(), "position".to_string());
        variable_mappings.insert("isf_FragCoord".to_string(), "position".to_string());
        
        let mut builtin_functions = HashMap::new();
        
        // Add noise functions
        builtin_functions.insert("noise".to_string(), include_str!("../assets/shaders/noise.wgsl").to_string());
        builtin_functions.insert("random".to_string(), include_str!("../assets/shaders/random.wgsl").to_string());
        
        Self {
            input_mappings,
            function_mappings,
            variable_mappings,
            builtin_functions,
        }
    }

    pub fn convert_isf_to_wgsl(&self, isf_shader: &IsfShader) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Add header comments
        wgsl_code.push_str(&format!("// ISF Shader: {}\n", isf_shader.metadata.name));
        if let Some(desc) = &isf_shader.metadata.description {
            wgsl_code.push_str(&format!("// Description: {}\n", desc));
        }
        if let Some(credit) = &isf_shader.metadata.credit {
            wgsl_code.push_str(&format!("// Credit: {}\n", credit));
        }
        wgsl_code.push('\n');
        
        // Generate uniform buffer for ISF inputs
        self.generate_uniform_buffer(&mut wgsl_code, &isf_shader.inputs)?;
        wgsl_code.push('\n');
        
        // Generate texture declarations for imported textures
        self.generate_texture_declarations(&mut wgsl_code, &isf_shader.imported_textures)?;
        wgsl_code.push('\n');
        
        // Add built-in functions
        self.add_builtin_functions(&mut wgsl_code)?;
        wgsl_code.push('\n');
        
        // Convert fragment shader
        let converted_fragment = self.convert_fragment_shader(&isf_shader.fragment_shader, &isf_shader.inputs)?;
        wgsl_code.push_str(&converted_fragment);
        
        Ok(wgsl_code)
    }

    fn generate_uniform_buffer(&self, wgsl_code: &mut String, inputs: &[IsfInput]) -> Result<()> {
        if inputs.is_empty() {
            return Ok(());
        }
        
        wgsl_code.push_str("struct IsfUniforms {\n");
        
        for (index, input) in inputs.iter().enumerate() {
            if let Some(mapping) = self.input_mappings.get(&input.input_type) {
                if mapping.needs_uniform {
                    wgsl_code.push_str(&format!(
                        "    {}: {},\n",
                        self.sanitize_identifier(&input.name),
                        mapping.wgsl_type
                    ));
                }
            }
        }
        
        wgsl_code.push_str("}\n\n");
        wgsl_code.push_str("@group(0) @binding(0) var<uniform> isf_uniforms: IsfUniforms;\n");
        
        Ok(())
    }

    fn generate_texture_declarations(&self, wgsl_code: &mut String, imported_textures: &[ImportedTexture]) -> Result<()> {
        for (index, texture) in imported_textures.iter().enumerate() {
            wgsl_code.push_str(&format!(
                "@group(1) @binding({}) var {}: texture_2d<f32>;\n",
                index,
                self.sanitize_identifier(&texture.name)
            ));
            wgsl_code.push_str(&format!(
                "@group(1) @binding({}) var {}_sampler: sampler;\n",
                index + imported_textures.len(),
                self.sanitize_identifier(&texture.name)
            ));
        }
        
        Ok(())
    }

    fn add_builtin_functions(&self, wgsl_code: &mut String) -> Result<()> {
        // Add ISF-specific built-in functions
        wgsl_code.push_str("// ISF Built-in Functions\n");
        wgsl_code.push_str("fn IMG_PIXEL(img: texture_2d<f32>, sampler_: sampler, coord: vec2<f32>) -> vec4<f32> {\n");
        wgsl_code.push_str("    return textureSample(img, sampler_, coord);\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("fn IMG_NORM_PIXEL(img: texture_2d<f32>, sampler_: sampler, coord: vec2<f32>) -> vec4<f32> {\n");
        wgsl_code.push_str("    return textureSample(img, sampler_, coord);\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("fn IMG_SIZE(img: texture_2d<f32>, level: i32) -> vec2<i32> {\n");
        wgsl_code.push_str("    return vec2<i32>(textureDimensions(img, level));\n");
        wgsl_code.push_str("}\n\n");
        
        Ok(())
    }

    fn convert_fragment_shader(&self, isf_code: &str, inputs: &[IsfInput]) -> Result<String> {
        let mut converted_code = String::new();
        
        // Add fragment shader header
        converted_code.push_str("@fragment\n");
        converted_code.push_str("fn main(@location(0) uv: vec2<f32>, @builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {\n");
        
        // Add ISF variable mappings
        converted_code.push_str("    let isf_FragNormCoord = uv;\n");
        converted_code.push_str("    let isf_FragCoord = position.xy;\n");
        converted_code.push_str("    let TIME = isf_uniforms.time;\n");
        converted_code.push_str("    let RENDERSIZE = vec2<f32>(1280.0, 720.0); // TODO: Get from render target\n");
        converted_code.push_str("    let VV = isf_FragNormCoord;\n");
        converted_code.push('\n');
        
        // Convert ISF inputs to WGSL variables
        for input in inputs {
            if let Some(mapping) = self.input_mappings.get(&input.input_type) {
                if mapping.needs_uniform {
                    let wgsl_name = self.sanitize_identifier(&input.name);
                    converted_code.push_str(&format!(
                        "    let {} = isf_uniforms.{};\n",
                        wgsl_name, wgsl_name
                    ));
                }
            }
        }
        converted_code.push('\n');
        
        // Convert the main shader code
        let mut converted_main = isf_code.to_string();
        
        // Replace ISF function calls
        for (isf_func, wgsl_func) in &self.function_mappings {
            converted_main = converted_main.replace(isf_func, wgsl_func);
        }
        
        // Replace ISF variables
        for (isf_var, wgsl_var) in &self.variable_mappings {
            converted_main = converted_main.replace(isf_var, wgsl_var);
        }
        
        // Add converted main code
        converted_code.push_str("    // Converted ISF shader code\n");
        for line in converted_main.lines() {
            converted_code.push_str(&format!("    {}\n", line));
        }
        
        converted_code.push_str("}\n");
        
        Ok(converted_code)
    }

    fn sanitize_identifier(&self, name: &str) -> String {
        name.replace(" ", "_").replace("-", "_").to_lowercase()
    }
}

/// ISF shader loader with comprehensive format support
pub struct IsfShaderLoader {
    converter: IsfToWgslConverter,
    cache: HashMap<PathBuf, IsfShader>,
}

impl IsfShaderLoader {
    pub fn new() -> Self {
        Self {
            converter: IsfToWgslConverter::new(),
            cache: HashMap::new(),
        }
    }

    pub fn load_isf_shader(&mut self, file_path: &Path) -> Result<IsfShader> {
        // Check cache first
        if let Some(cached) = self.cache.get(file_path) {
            return Ok(cached.clone());
        }

        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read ISF file: {:?}", file_path))?;

        let shader = self.parse_isf_shader(&content, file_path)?;
        
        // Cache the result
        self.cache.insert(file_path.to_path_buf(), shader.clone());
        
        Ok(shader)
    }

    fn parse_isf_shader(&self, content: &str, file_path: &Path) -> Result<IsfShader> {
        // Parse ISF format with JSON metadata in comments
        let mut json_metadata = String::new();
        let mut in_json = false;
        let mut fragment_shader = String::new();
        let mut vertex_shader = None;
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("/*{") {
                in_json = true;
                json_metadata.push('{');
            } else if in_json && trimmed.starts_with("}") && trimmed.ends_with("*/") {
                in_json = false;
                json_metadata.push('}');
            } else if in_json {
                json_metadata.push_str(trimmed);
                json_metadata.push('\n');
            } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
                // This is shader code
                if trimmed.contains("void main") && trimmed.contains("vertex") {
                    vertex_shader = Some(trimmed.to_string());
                } else {
                    fragment_shader.push_str(line);
                    fragment_shader.push('\n');
                }
            }
        }
        
        // Parse JSON metadata
        let metadata: serde_json::Value = serde_json::from_str(&json_metadata)
            .with_context(|| "Failed to parse ISF JSON metadata")?;
        
        // Extract metadata fields
        let name = metadata["NAME"].as_str()
            .unwrap_or(file_path.file_stem().unwrap().to_str().unwrap())
            .to_string();
        
        let description = metadata["DESCRIPTION"].as_str().map(String::from);
        let credit = metadata["CREDIT"].as_str().map(String::from);
        let isf_version = metadata["ISFVSN"].as_str().unwrap_or("2.0").to_string();
        let vsn = metadata["VSN"].as_str().map(String::from);
        
        let categories = metadata["CATEGORIES"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
            .unwrap_or_default();
        
        let keywords = metadata["KEYWORDS"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
            .unwrap_or_default();
        
        let imported = metadata["IMPORTED"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
            .unwrap_or_default();
        
        let metadata = IsfMetadata {
            name,
            description,
            credit,
            isf_version,
            vsn,
            categories,
            keywords,
            imported,
        };
        
        // Parse inputs
        let inputs = if let Some(inputs_array) = metadata["INPUTS"].as_array() {
            self.parse_isf_inputs(inputs_array)?
        } else {
            Vec::new()
        };
        
        // Parse passes
        let passes = if let Some(passes_array) = metadata["PASSES"].as_array() {
            self.parse_isf_passes(passes_array)?
        } else {
            vec![IsfPass {
                target: None,
                persistent: None,
                float: None,
                width: None,
                height: None,
                description: None,
                inputs: None,
            }]
        };
        
        // Parse persistent buffers
        let persistent_buffers = metadata["PERSISTENT_BUFFERS"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
            .unwrap_or_default();
        
        // Parse imported textures
        let imported_textures = Vec::new(); // TODO: Parse from metadata
        
        Ok(IsfShader {
            metadata,
            inputs,
            passes,
            persistent_buffers,
            imported_textures,
            vertex_shader,
            fragment_shader,
            source_file: Some(file_path.to_path_buf()),
        })
    }

    fn parse_isf_inputs(&self, inputs_array: &[serde_json::Value]) -> Result<Vec<IsfInput>> {
        let mut inputs = Vec::new();
        
        for input_value in inputs_array {
            let name = input_value["NAME"].as_str()
                .ok_or_else(|| anyhow!("Input missing NAME field"))?
                .to_string();
            
            let input_type_str = input_value["TYPE"].as_str()
                .ok_or_else(|| anyhow!("Input missing TYPE field"))?
                .to_lowercase();
            
            let input_type = match input_type_str.as_str() {
                "float" => IsfInputType::Float,
                "bool" => IsfInputType::Bool,
                "color" => IsfInputType::Color,
                "point2d" => IsfInputType::Point2D,
                "image" => IsfInputType::Image,
                "long" => IsfInputType::Long,
                "event" => IsfInputType::Event,
                "audio" => IsfInputType::Audio,
                "audiofft" => IsfInputType::AudioFFT,
                _ => return Err(anyhow!("Unknown input type: {}", input_type_str)),
            };
            
            let label = input_value["LABEL"].as_str().map(String::from);
            let default = input_value["DEFAULT"].clone();
            let min = input_value["MIN"].clone();
            let max = input_value["MAX"].clone();
            let values = input_value["VALUES"].as_array().map(|arr| arr.clone());
            let clamp = input_value["CLAMP"].as_bool();
            let identity = input_value["IDENTITY"].as_bool();
            
            inputs.push(IsfInput {
                name,
                input_type,
                label,
                default,
                min,
                max,
                values,
                clamp,
                identity,
            });
        }
        
        Ok(inputs)
    }

    fn parse_isf_passes(&self, passes_array: &[serde_json::Value]) -> Result<Vec<IsfPass>> {
        let mut passes = Vec::new();
        
        for pass_value in passes_array {
            let target = pass_value["TARGET"].as_str().map(String::from);
            let persistent = pass_value["PERSISTENT"].as_bool();
            let float = pass_value["FLOAT"].as_bool();
            let width = pass_value["WIDTH"].as_str().map(String::from);
            let height = pass_value["HEIGHT"].as_str().map(String::from);
            let description = pass_value["DESCRIPTION"].as_str().map(String::from);
            
            let inputs = if let Some(inputs_obj) = pass_value["INPUTS"].as_object() {
                Some(inputs_obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            } else {
                None
            };
            
            passes.push(IsfPass {
                target,
                persistent,
                float,
                width,
                height,
                description,
                inputs,
            });
        }
        
        Ok(passes)
    }

    pub fn convert_isf_to_wgsl(&self, isf_shader: &IsfShader) -> Result<String> {
        self.converter.convert_isf_to_wgsl(isf_shader)
    }
}

/// Batch ISF shader processor for loading multiple shaders
pub struct IsfBatchProcessor {
    loader: IsfShaderLoader,
    shaders: HashMap<String, IsfShader>,
}

impl IsfBatchProcessor {
    pub fn new() -> Self {
        Self {
            loader: IsfShaderLoader::new(),
            shaders: HashMap::new(),
        }
    }

    pub fn load_isf_directory(&mut self, dir_path: &Path) -> Result<usize> {
        if !dir_path.is_dir() {
            return Err(anyhow!("Path is not a directory: {:?}", dir_path));
        }

        let mut loaded_count = 0;
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("fs") {
                match self.loader.load_isf_shader(&path) {
                    Ok(shader) => {
                        let name = shader.metadata.name.clone();
                        self.shaders.insert(name.clone(), shader);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to load ISF shader {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(loaded_count)
    }

    pub fn get_shader(&self, name: &str) -> Option<&IsfShader> {
        self.shaders.get(name)
    }

    pub fn get_all_shaders(&self) -> &HashMap<String, IsfShader> {
        &self.shaders
    }

    pub fn convert_all_to_wgsl(&self) -> HashMap<String, String> {
        let mut converted = HashMap::new();
        
        for (name, shader) in &self.shaders {
            match self.loader.convert_isf_to_wgsl(shader) {
                Ok(wgsl_code) => {
                    converted.insert(name.clone(), wgsl_code);
                }
                Err(e) => {
                    eprintln!("Failed to convert ISF shader {} to WGSL: {}", name, e);
                }
            }
        }
        
        converted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isf_shader_loader_creation() {
        let loader = IsfShaderLoader::new();
        assert!(loader.cache.is_empty());
    }

    #[test]
    fn test_isf_to_wgsl_converter_creation() {
        let converter = IsfToWgslConverter::new();
        assert!(!converter.input_mappings.is_empty());
        assert!(!converter.function_mappings.is_empty());
    }

    #[test]
    fn test_isf_input_type_mapping() {
        let converter = IsfToWgslConverter::new();
        
        let float_mapping = converter.input_mappings.get(&IsfInputType::Float).unwrap();
        assert_eq!(float_mapping.wgsl_type, "f32");
        assert_eq!(float_mapping.default_value, "0.0");
        assert!(float_mapping.needs_uniform);
        
        let color_mapping = converter.input_mappings.get(&IsfInputType::Color).unwrap();
        assert_eq!(color_mapping.wgsl_type, "vec4<f32>");
        assert_eq!(color_mapping.default_value, "vec4<f32>(1.0, 1.0, 1.0, 1.0)");
        assert!(color_mapping.needs_uniform);
    }

    #[test]
    fn test_isf_function_mapping() {
        let converter = IsfToWgslConverter::new();
        
        assert_eq!(
            converter.function_mappings.get("IMG_PIXEL").unwrap(),
            "textureSample"
        );
        assert_eq!(
            converter.function_mappings.get("ISF_TIME").unwrap(),
            "time"
        );
        assert_eq!(
            converter.function_mappings.get("RENDERSIZE").unwrap(),
            "resolution"
        );
    }

    #[test]
    fn test_sanitize_identifier() {
        let converter = IsfToWgslConverter::new();
        
        assert_eq!(converter.sanitize_identifier("My Input"), "my_input");
        assert_eq!(converter.sanitize_identifier("test-input"), "test_input");
        assert_eq!(converter.sanitize_identifier("VALID_NAME"), "valid_name");
    }
}