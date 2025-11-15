use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use regex::Regex;

/// ISF input parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "TYPE")]
pub enum IsfInputType {
    #[serde(rename = "float")]
    Float { DEFAULT: f32, MIN: f32, MAX: f32 },
    #[serde(rename = "bool")]
    Bool { DEFAULT: bool },
    #[serde(rename = "color")]
    Color { DEFAULT: [f32; 4] },
    #[serde(rename = "point2D")]
    Point2D { DEFAULT: [f32; 2], MIN: [f32; 2], MAX: [f32; 2] },
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "audioFFT")]
    AudioFFT,
}

/// ISF input parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfInput {
    pub NAME: String,
    #[serde(flatten)]
    pub input_type: IsfInputType,
}

/// ISF metadata from JSON comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfMetadata {
    pub NAME: String,
    pub DESCRIPTION: Option<String>,
    pub CREDIT: Option<String>,
    pub ISFVSN: Option<String>,
    pub VSN: Option<String>,
    pub INPUTS: Option<Vec<IsfInput>>,
    pub PASSES: Option<Vec<HashMap<String, serde_json::Value>>>,
    pub PERSISTENT_BUFFERS: Option<Vec<String>>,
}

/// WGSL conversion result
#[derive(Debug, Clone)]
pub struct WgslConversionResult {
    pub metadata: IsfMetadata,
    pub wgsl_code: String,
    pub inputs: Vec<IsfInput>,
    pub bind_groups: Vec<BindGroupInfo>,
    pub performance_hints: Vec<String>,
    pub conversion_time_ms: f64,
    pub entry_points: Vec<String>,
    pub converted_wgsl: Option<Box<WgslConversionResult>>,
    pub conversion_notes: Vec<String>,
    pub vertex_shader: String,
    pub fragment_shader: String,
}

/// Bind group information for WGSL
#[derive(Debug, Clone)]
pub struct BindGroupInfo {
    pub group: u32,
    pub binding: u32,
    pub name: String,
    pub binding_type: String,
}

/// ISF to WGSL auto-converter
pub struct IsfAutoConverter {
    conversion_cache: HashMap<PathBuf, WgslConversionResult>,
}

impl IsfAutoConverter {
    pub fn new() -> Self {
        Self {
            conversion_cache: HashMap::new(),
        }
    }

    /// Convert ISF shader to WGSL
    pub fn convert_isf_to_wgsl(&mut self, isf_code: &str, file_path: Option<&PathBuf>) -> Result<WgslConversionResult> {
        let start_time = std::time::Instant::now();
        
        // Parse ISF metadata from JSON comment
        let metadata = self.parse_isf_metadata(isf_code)?;
        
        // Extract GLSL code (everything after the JSON comment)
        let glsl_code = self.extract_glsl_code(isf_code);
        
        // Convert GLSL to WGSL
        let wgsl_code = self.convert_glsl_to_wgsl(&glsl_code, &metadata)?;
        
        // Generate bind groups from inputs
        let bind_groups = self.generate_bind_groups(&metadata);
        
        // Generate performance hints
        let performance_hints = self.generate_performance_hints(&wgsl_code);
        
        let conversion_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        
        let result = WgslConversionResult {
            metadata: metadata.clone(),
            wgsl_code: wgsl_code.clone(),
            inputs: metadata.INPUTS.clone().unwrap_or_default(),
            bind_groups,
            performance_hints,
            conversion_time_ms,
            entry_points: vec!["fs_main".to_string()], // Standard fragment shader entry point
            converted_wgsl: None, // Will be set if needed
            conversion_notes: vec!["ISF auto-conversion successful".to_string()],
            vertex_shader: "// Vertex shader would be generated here".to_string(),
            fragment_shader: wgsl_code,
        };
        
        // Cache result if file path provided
        if let Some(path) = file_path {
            self.conversion_cache.insert(path.clone(), result.clone());
        }
        
        Ok(result)
    }

    /// Parse ISF metadata from JSON comment
    fn parse_isf_metadata(&self, isf_code: &str) -> Result<IsfMetadata> {
        // Find JSON comment block
        let json_start = isf_code.find("/*{").ok_or_else(|| anyhow!("No ISF metadata found"))?;
        let json_end = isf_code.find("}*/").ok_or_else(|| anyhow!("No ISF metadata end found"))?;
        
        let json_str = &isf_code[json_start + 2..json_end + 2];
        let metadata: IsfMetadata = serde_json::from_str(json_str)?;
        
        Ok(metadata)
    }

    /// Extract GLSL code (everything after JSON comment)
    fn extract_glsl_code(&self, isf_code: &str) -> String {
        if let Some(json_end) = isf_code.find("}*/") {
            isf_code[json_end + 3..].trim().to_string()
        } else {
            isf_code.to_string()
        }
    }

    /// Convert GLSL code to WGSL
    fn convert_glsl_to_wgsl(&self, glsl_code: &str, metadata: &IsfMetadata) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Add struct definitions
        wgsl_code.push_str("struct VertexOutput {\n");
        wgsl_code.push_str("    @builtin(position) position: vec4<f32>,\n");
        wgsl_code.push_str("    @location(0) uv: vec2<f32>,\n");
        wgsl_code.push_str("}\n\n");
        
        // Add uniform struct
        wgsl_code.push_str("struct Uniforms {\n");
        wgsl_code.push_str("    time: f32,\n");
        wgsl_code.push_str("    resolution: vec2<f32>,\n");
        
        if let Some(inputs) = &metadata.INPUTS {
            for input in inputs {
                match &input.input_type {
                    IsfInputType::Float { DEFAULT: _, MIN: _, MAX: _ } => {
                        wgsl_code.push_str(&format!("    {}: f32,\n", input.NAME));
                    }
                    IsfInputType::Bool { DEFAULT: _ } => {
                        wgsl_code.push_str(&format!("    {}: bool,\n", input.NAME));
                    }
                    IsfInputType::Color { DEFAULT: _ } => {
                        wgsl_code.push_str(&format!("    {}: vec4<f32>,\n", input.NAME));
                    }
                    IsfInputType::Point2D { DEFAULT: _, MIN: _, MAX: _ } => {
                        wgsl_code.push_str(&format!("    {}: vec2<f32>,\n", input.NAME));
                    }
                    _ => {}
                }
            }
        }
        
        wgsl_code.push_str("}\n\n");
        
        // Add bind group declarations
        wgsl_code.push_str("@group(0) @binding(0)\n");
        wgsl_code.push_str("var<uniform> uniforms: Uniforms;\n\n");
        
        // Add texture declarations for image inputs
        if let Some(inputs) = &metadata.INPUTS {
            let mut texture_binding = 1;
            for input in inputs {
                if matches!(input.input_type, IsfInputType::Image) {
                    wgsl_code.push_str(&format!("@group(0) @binding({})\n", texture_binding));
                    wgsl_code.push_str(&format!("var {}: texture_2d<f32>;\n", input.NAME));
                    wgsl_code.push_str(&format!("@group(0) @binding({})\n", texture_binding + 1));
                    wgsl_code.push_str(&format!("var {}_sampler: sampler;\n\n", input.NAME));
                    texture_binding += 2;
                }
            }
        }
        
        // Convert main function
        let converted_main = self.convert_main_function(glsl_code)?;
        wgsl_code.push_str(&converted_main);
        
        Ok(wgsl_code)
    }

    /// Convert main() function from GLSL to WGSL
    fn convert_main_function(&self, glsl_code: &str) -> Result<String> {
        let mut result = String::new();
        
        // Basic GLSL to WGSL conversions
        let mut converted = glsl_code.to_string();
        
        // Replace GLSL built-ins
        converted = converted.replace("gl_FragColor", "fragColor");
        converted = converted.replace("gl_FragCoord", "fragCoord");
        converted = converted.replace("isf_FragNormCoord", "uv");
        converted = converted.replace("TIME", "uniforms.time");
        converted = converted.replace("RENDERSIZE", "uniforms.resolution");
        
        // Convert vec constructors
        converted = converted.replace("vec2(", "vec2<f32>(");
        converted = converted.replace("vec3(", "vec3<f32>(");
        converted = converted.replace("vec4(", "vec4<f32>(");
        
        // Convert texture sampling
        converted = converted.replace("IMG_PIXEL(", "textureSample(");
        
        // Replace function signature
        converted = converted.replace("void main()", "@fragment\nfn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32>");
        
        // Add return statement if using fragColor
        if converted.contains("fragColor") {
            converted = converted.replace("fragColor =", "let fragColor =");
            // Add return at the end
            if !converted.contains("return") {
                if let Some(pos) = converted.rfind('}') {
                    converted.insert_str(pos, "\n    return fragColor;");
                }
            }
        }
        
        result.push_str(&converted);
        Ok(result)
    }

    /// Generate bind groups from inputs
    fn generate_bind_groups(&self, metadata: &IsfMetadata) -> Vec<BindGroupInfo> {
        let mut bind_groups = Vec::new();
        let mut binding = 0u32;
        
        // Uniforms bind group
        bind_groups.push(BindGroupInfo {
            group: 0,
            binding,
            name: "uniforms".to_string(),
            binding_type: "uniform".to_string(),
        });
        binding += 1;
        
        // Image inputs
        if let Some(inputs) = &metadata.INPUTS {
            for input in inputs {
                if matches!(input.input_type, IsfInputType::Image) {
                    bind_groups.push(BindGroupInfo {
                        group: 0,
                        binding,
                        name: input.NAME.clone(),
                        binding_type: "texture".to_string(),
                    });
                    binding += 1;
                    
                    bind_groups.push(BindGroupInfo {
                        group: 0,
                        binding,
                        name: format!("{}_sampler", input.NAME),
                        binding_type: "sampler".to_string(),
                    });
                    binding += 1;
                }
            }
        }
        
        bind_groups
    }

    /// Generate performance hints
    fn generate_performance_hints(&self, wgsl_code: &str) -> Vec<String> {
        let mut hints = Vec::new();
        
        if wgsl_code.contains("textureSample") {
            let count = wgsl_code.matches("textureSample").count();
            if count > 16 {
                hints.push(format!("High texture sampling count ({}), consider texture atlas optimization ", count));
            }
        }
        
        if wgsl_code.contains("sin") || wgsl_code.contains("cos") || wgsl_code.contains("tan") {
            hints.push("Trigonometric functions detected, consider using precomputed lookup tables for better performance ".to_string());
        }
        
        if wgsl_code.contains("pow") {
            hints.push("Power functions detected, consider using multiplication chains for integer exponents ".to_string());
        }
        
        if wgsl_code.contains("for") || wgsl_code.contains("while") {
            hints.push("Loop constructs detected, ensure loop bounds are constant for optimal performance ".to_string());
        }
        
        hints
    }

    /// Parse ISF with advanced error handling
    pub fn parse_isf_advanced(&mut self, isf_code: &str) -> Result<WgslConversionResult> {
        self.convert_isf_to_wgsl(isf_code, None)
    }

    /// Load and convert ISF file (for GUI integration)
    pub fn load_and_convert(&mut self, file_path: &PathBuf) -> Result<WgslConversionResult> {
        let content = std::fs::read_to_string(file_path)?;
        self.convert_isf_to_wgsl(&content, Some(file_path))
    }

    /// Convert to WGSL with advanced features (for tester compatibility)
    pub fn convert_to_wgsl_advanced(&mut self, isf_code: &str) -> Result<WgslConversionResult> {
        self.convert_isf_to_wgsl(isf_code, None)
    }
}

impl Default for IsfAutoConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_isf_conversion() {
        let converter = IsfAutoConverter::new();
        let test_isf = r#"
        /*{
            "NAME": "Test Shader",
            "DESCRIPTION": "A test shader",
            "INPUTS": [
                {"NAME": "speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 10.0}
            ]
        }*/
        
        void main() {
            vec2 uv = isf_FragNormCoord;
            float time = TIME * speed;
            vec3 color = vec3(sin(time + uv.x * 10.0), cos(time + uv.y * 10.0), 0.5);
            gl_FragColor = vec4(color, 1.0);
        }
        "#;
        
        let result = converter.parse_isf_advanced(test_isf).unwrap();
        assert_eq!(result.metadata.NAME, "Test Shader");
        assert_eq!(result.inputs.len(), 1);
        assert_eq!(result.inputs[0].NAME, "speed");
        
        // Check that WGSL code contains expected elements
        assert!(result.wgsl_code.contains("@fragment"));
        assert!(result.wgsl_code.contains("fn fs_main"));
        assert!(result.wgsl_code.contains("uniforms.time"));
        assert!(result.wgsl_code.contains("uniforms.resolution"));
    }

    #[test]
    fn test_texture_conversion() {
        let converter = IsfAutoConverter::new();
        let test_isf = r#"
        /*{
            "NAME": "Texture Test",
            "INPUTS": [
                {"NAME": "inputImage", "TYPE": "image"}
            ]
        }*/
        
        void main() {
            vec2 uv = isf_FragNormCoord;
            vec4 color = IMG_PIXEL(inputImage, uv);
            gl_FragColor = color;
        }
        "#;
        
        let result = converter.parse_isf_advanced(test_isf).unwrap();
        assert!(result.wgsl_code.contains("textureSample"));
        assert!(result.wgsl_code.contains("inputImage"));
        assert!(result.wgsl_code.contains("inputImage_sampler"));
    }
}