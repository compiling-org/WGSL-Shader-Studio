use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, anyhow};

/// ISF (Interactive Shader Format) structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfShader {
    #[serde(rename = "CREDIT")]
    pub credit: Option<String>,
    #[serde(rename = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(rename = "CATEGORIES")]
    pub categories: Option<Vec<String>>,
    #[serde(rename = "INPUTS")]
    pub inputs: Option<Vec<IsfInput>>,
    #[serde(rename = "PASSES")]
    pub passes: Option<Vec<IsfPass>>,
    #[serde(rename = "PERSISTENT_BUFFERS")]
    pub persistent_buffers: Option<Vec<String>>,
    #[serde(rename = "IMPORTED")]
    pub imported: Option<HashMap<String, String>>,
    #[serde(rename = "ISFVSN")]
    pub isf_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfInput {
    pub name: String,
    #[serde(rename = "TYPE")]
    pub input_type: String,
    #[serde(rename = "MIN")]
    pub min_value: Option<f32>,
    #[serde(rename = "MAX")]
    pub max_value: Option<f32>,
    #[serde(rename = "DEFAULT")]
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfPass {
    #[serde(rename = "TARGET")]
    pub target: Option<String>,
    #[serde(rename = "PERSISTENT")]
    pub persistent: Option<bool>,
}

/// ISF to WGSL converter
pub struct IsfToWgslConverter;

impl IsfToWgslConverter {
    pub fn new() -> Self {
        Self
    }

    /// Load ISF shader from JSON file
    pub fn load_isf_from_file<P: AsRef<Path>>(&self, path: P) -> Result<IsfShader> {
        let content = std::fs::read_to_string(path)?;
        self.parse_isf(&content)
    }

    /// Parse ISF shader from JSON string
    pub fn parse_isf(&self, json_str: &str) -> Result<IsfShader> {
        let shader: IsfShader = serde_json::from_str(json_str)?;
        Ok(shader)
    }

    /// Convert ISF to WGSL fragment shader
    pub fn convert_fragment_shader(&self, isf: &IsfShader, isf_code: &str) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Generate uniform buffer
        let uniform_buffer = self.generate_uniform_buffer(isf)?;
        wgsl_code.push_str(&uniform_buffer);
        wgsl_code.push_str("\n");
        
        // Generate main function
        wgsl_code.push_str("@vertex\n");
        wgsl_code.push_str("fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {\n");
        wgsl_code.push_str("    return vec4<f32>(position, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {\n");
        wgsl_code.push_str("    let uv = (coord.xy / uniforms.renderSize);\n");
        wgsl_code.push_str("    let TIME = uniforms.time;\n");
        wgsl_code.push_str("    let RENDERSIZE = uniforms.renderSize;\n");
        wgsl_code.push_str("    let ASPECT = uniforms.aspectRatio;\n");
        
        // Add input mappings
        if let Some(inputs) = &isf.inputs {
            for input in inputs {
                let wgsl_name = self.sanitize_identifier(&input.name);
                let uniform_access = format!("uniforms.{}", wgsl_name);
                wgsl_code.push_str(&format!("    let {} = {};\n", wgsl_name, uniform_access));
            }
        }
        
        // Convert and add ISF code
        let converted_code = self.convert_isf_functions(isf_code);
        wgsl_code.push_str(&converted_code);
        wgsl_code.push_str("}\n");
        
        Ok(wgsl_code)
    }

    /// Generate uniform buffer layout from ISF inputs
    fn generate_uniform_buffer(&self, isf: &IsfShader) -> Result<String> {
        let mut buffer_code = String::new();
        buffer_code.push_str("struct Uniforms {\n");
        
        if let Some(inputs) = &isf.inputs {
            for input in inputs {
                let field_type = self.map_isf_type_to_wgsl(&input.input_type)?;
                let field_name = self.sanitize_identifier(&input.name);
                
                buffer_code.push_str(&format!("    {}: {},\n", field_name, field_type));
            }
        }
        
        // Add standard ISF uniforms
        buffer_code.push_str("    time: f32,\n");
        buffer_code.push_str("    timeDelta: f32,\n");
        buffer_code.push_str("    frame: u32,\n");
        buffer_code.push_str("    fps: f32,\n");
        buffer_code.push_str("    progress: f32,\n");
        buffer_code.push_str("    renderSize: vec2<f32>,\n");
        buffer_code.push_str("    aspectRatio: f32,\n");
        
        buffer_code.push_str("}\n");
        buffer_code.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n");
        
        Ok(buffer_code)
    }

    /// Convert ISF-specific functions to WGSL equivalents
    fn convert_isf_functions(&self, code: &str) -> String {
        let mut converted = code.to_string();
        
        // Convert ISF built-in functions to WGSL
        converted = converted.replace("IMG_PIXEL", "textureSample");
        converted = converted.replace("IMG_NORM_PIXEL", "textureSample");
        converted = converted.replace("IMG_THIS_PIXEL", "textureSample");
        converted = converted.replace("IMG_SIZE", "textureDimensions");
        
        // Convert ISF variables to WGSL uniforms
        converted = converted.replace("TIME", "uniforms.time");
        converted = converted.replace("TIMEDELTA", "uniforms.timeDelta");
        converted = converted.replace("FRAME", "f32(uniforms.frame)");
        converted = converted.replace("FPS", "uniforms.fps");
        converted = converted.replace("PROGRESS", "uniforms.progress");
        converted = converted.replace("RENDERSIZE", "uniforms.renderSize");
        converted = converted.replace("ASPECT", "uniforms.aspectRatio");
        
        // Convert vec macros to WGSL
        converted = converted.replace("vec2(", "vec2<f32>(");
        converted = converted.replace("vec3(", "vec3<f32>(");
        converted = converted.replace("vec4(", "vec4<f32>(");
        
        converted
    }

    /// Map ISF input types to WGSL types
    fn map_isf_type_to_wgsl(&self, isf_type: &str) -> Result<String> {
        match isf_type.to_lowercase().as_str() {
            "float" | "double" => Ok("f32".to_string()),
            "bool" | "event" => Ok("bool".to_string()),
            "long" | "integer" => Ok("i32".to_string()),
            "point2d" => Ok("vec2<f32>".to_string()),
            "color" => Ok("vec4<f32>".to_string()),
            "image" => Ok("texture_2d<f32>".to_string()),
            "audio" => Ok("texture_2d<f32>".to_string()),
            "audiofft" => Ok("texture_2d<f32>".to_string()),
            _ => Err(anyhow!("Unsupported ISF input type: {}", isf_type)),
        }
    }

    /// Sanitize identifier for WGSL
    fn sanitize_identifier(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
    }
}

impl Default for IsfToWgslConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Main ISF converter that wraps the functionality
pub struct IsfConverter {
    converter: IsfToWgslConverter,
}

impl IsfConverter {
    pub fn new() -> Self {
        Self {
            converter: IsfToWgslConverter::new(),
        }
    }
    
    /// Convert ISF shader to WGSL
    pub fn convert_to_wgsl(&mut self, isf_shader: &crate::isf_loader::IsfShader) -> Result<String, Box<dyn std::error::Error>> {
        // Convert the ISF loader format to our internal format
        let internal_isf = IsfShader {
            credit: None,
            description: None,
            categories: None,
            inputs: if isf_shader.inputs.is_empty() {
                None
            } else {
                Some(isf_shader.inputs.iter().map(|input| IsfInput {
                    name: input.name.clone(),
                    input_type: match input.input_type {
                        crate::isf_loader::InputType::Float => "float".to_string(),
                        crate::isf_loader::InputType::Bool => "bool".to_string(),
                        crate::isf_loader::InputType::Color => "color".to_string(),
                        crate::isf_loader::InputType::Point2D => "point2D".to_string(),
                        crate::isf_loader::InputType::Image => "image".to_string(),
                    },
                    min_value: input.min,
                    max_value: input.max,
                    default_value: input.default.map(|v| serde_json::Value::Number(serde_json::Number::from_f64(v as f64).unwrap())),
                }).collect())
            },
            passes: None,
            persistent_buffers: None,
            imported: None,
            isf_version: Some("2.0".to_string()),
        };
        
        // Use the converter to convert to WGSL
        Ok(self.converter.convert_fragment_shader(&internal_isf, &isf_shader.source)?)
    }
}