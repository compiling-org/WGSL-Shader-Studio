use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context, bail};
use crate::converter::diagnostics::{Diagnostic, DiagnosticSeverity, Diagnostics, DiagnosticHelpers};

/// ISF 1.2 specification structures
/// Based on the Interactive Shader Format specification

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISFShader {
    pub metadata: ISFMetadata,
    pub vertex_shader: Option<String>,
    pub fragment_shader: String,
    pub inputs: Vec<ISFInput>,
    pub outputs: Vec<ISFOutput>,
    pub imports: Vec<String>,
    pub passes: Vec<ISFPass>,
    pub persistent_buffers: Vec<String>,
    pub custom_functions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISFMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub credit: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub version: String,
    pub isf_version: String,
    pub webgl: bool,
    pub platforms: Vec<String>,
    pub imported: Vec<String>,
    pub passes: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub credit_url: Option<String>,
    pub description_extended: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISFInput {
    pub name: String,
    pub input_type: ISFInputType,
    pub default: Option<serde_json::Value>,
    pub min: Option<serde_json::Value>,
    pub max: Option<serde_json::Value>,
    pub identity: Option<serde_json::Value>,
    pub values: Option<Vec<serde_json::Value>>,
    pub label: Option<String>,
    pub functional: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ISFInputType {
    Event,
    Bool,
    Long,
    Float,
    Point2D,
    Color,
    Image,
    Audio,
    AudioFFT,
    Cube,
    AudioWaveform,
    AudioFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISFOutput {
    pub name: String,
    pub output_type: ISFOutputType,
    pub width: Option<usize>,
    pub height: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ISFOutputType {
    Image,
    Buffer,
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISFPass {
    pub target: Option<String>,
    pub persistent: bool,
    pub float: bool,
    pub width: Option<String>,
    pub height: Option<String>,
    pub format: Option<String>,
    pub description: Option<String>,
    pub inputs: HashMap<String, serde_json::Value>,
}

/// ISF file parser and converter
pub struct ISFParser {
    diagnostics: Diagnostics,
}

impl ISFParser {
    pub fn new() -> Self {
        Self {
            diagnostics: Diagnostics::new(),
        }
    }
    
    /// Parse an ISF file from JSON content
    pub fn parse_isf(&mut self, content: &str, file_path: &str) -> Result<ISFShader> {
        let json_data: serde_json::Value = serde_json::from_str(content)
            .with_context(|| format!("Failed to parse ISF JSON in {}", file_path))?;
        
        self.validate_isf_schema(&json_data, file_path)?;
        
        let metadata = self.parse_metadata(&json_data, file_path)?;
        let inputs = self.parse_inputs(&json_data, file_path)?;
        let outputs = self.parse_outputs(&json_data, file_path)?;
        let passes = self.parse_passes(&json_data, file_path)?;
        
        // Extract vertex and fragment shaders
        let vertex_shader = json_data.get("VERTEX_SHADER")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let fragment_shader = json_data.get("FRAGMENT_SHADER")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing FRAGMENT_SHADER in ISF file"))?
            .to_string();
        
        // Extract imports and persistent buffers
        let imports = json_data.get("IMPORTED")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
            
        let persistent_buffers = json_data.get("PERSISTENT_BUFFERS")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
        
        Ok(ISFShader {
            metadata,
            vertex_shader,
            fragment_shader,
            inputs,
            outputs,
            imports,
            passes,
            persistent_buffers,
            custom_functions: HashMap::new(),
        })
    }
    
    /// Validate ISF JSON schema
    fn validate_isf_schema(&mut self, json_data: &serde_json::Value, file_path: &str) -> Result<()> {
        // Check required fields
        if !json_data.is_object() {
            self.diagnostics.add_diagnostic(
                DiagnosticHelpers::validation_error(
                    "ISF file must be a JSON object",
                    1,
                    1
                ).with_file_path(file_path.to_string())
            );
            return Err(anyhow::anyhow!("Invalid ISF JSON structure"));
        }
        
        // Check for FRAGMENT_SHADER
        if json_data.get("FRAGMENT_SHADER").is_none() {
            self.diagnostics.add_diagnostic(
                DiagnosticHelpers::validation_error(
                    "Missing required FRAGMENT_SHADER field",
                    1,
                    1
                ).with_file_path(file_path.to_string())
            );
        }
        
        // Check ISF version
        if let Some(isf_version) = json_data.get("ISF_VERSION").and_then(|v| v.as_str()) {
            if !isf_version.starts_with("1.") {
                self.diagnostics.add_diagnostic(
                    DiagnosticHelpers::compatibility_warning(
                        format!("ISF version {} may not be fully supported", isf_version),
                        1,
                        1
                    ).with_file_path(file_path.to_string())
                );
            }
        } else {
            self.diagnostics.add_diagnostic(
                DiagnosticHelpers::validation_error(
                    "Missing ISF_VERSION field",
                    1,
                    1
                ).with_file_path(file_path.to_string())
            );
        }
        
        Ok(())
    }
    
    /// Parse ISF metadata
    fn parse_metadata(&mut self, json_data: &serde_json::Value, file_path: &str) -> Result<ISFMetadata> {
        let description = json_data.get("DESCRIPTION")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let author = json_data.get("AUTHOR")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let credit = json_data.get("CREDIT")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let categories = json_data.get("CATEGORIES")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
            
        let tags = json_data.get("TAGS")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
            
        let version = json_data.get("VERSION")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0")
            .to_string();
            
        let isf_version = json_data.get("ISF_VERSION")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0")
            .to_string();
            
        let webgl = json_data.get("WEBGL")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let platforms = json_data.get("PLATFORMS")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
            
        let imported = json_data.get("IMPORTED")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
            
        let passes = json_data.get("PASSES")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
            
        let inputs = json_data.get("INPUTS")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.get("NAME"))
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
            
        let outputs = json_data.get("OUTPUTS")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.get("NAME"))
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();
            
        let credit_url = json_data.get("CREDIT_URL")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let description_extended = json_data.get("DESCRIPTION_EXTENDED")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(ISFMetadata {
            description,
            author,
            credit,
            categories,
            tags,
            version,
            isf_version,
            webgl,
            platforms,
            imported,
            passes,
            inputs,
            outputs,
            credit_url,
            description_extended,
        })
    }
    
    /// Parse ISF inputs
    fn parse_inputs(&mut self, json_data: &serde_json::Value, file_path: &str) -> Result<Vec<ISFInput>> {
        let mut inputs = Vec::new();
        
        if let Some(inputs_array) = json_data.get("INPUTS").and_then(|v| v.as_array()) {
            for (index, input_value) in inputs_array.iter().enumerate() {
                if let Some(input) = self.parse_single_input(input_value, index, file_path)? {
                    inputs.push(input);
                }
            }
        }
        
        Ok(inputs)
    }
    
    /// Parse a single ISF input
    fn parse_single_input(&mut self, input_value: &serde_json::Value, index: usize, file_path: &str) -> Result<Option<ISFInput>> {
        if !input_value.is_object() {
            self.diagnostics.add_diagnostic(
                DiagnosticHelpers::validation_error(
                    format!("Input at index {} must be an object", index),
                    1,
                    1
                ).with_file_path(file_path.to_string())
            );
            return Ok(None);
        }
        
        let name = input_value.get("NAME")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Input at index {} missing NAME field", index))?
            .to_string();
            
        let input_type_str = input_value.get("TYPE")
            .and_then(|v| v.as_str())
            .unwrap_or("float")
            .to_lowercase();
            
        let input_type = match input_type_str.as_str() {
            "event" => ISFInputType::Event,
            "bool" => ISFInputType::Bool,
            "long" | "int" => ISFInputType::Long,
            "float" => ISFInputType::Float,
            "point2d" | "vec2" => ISFInputType::Point2D,
            "color" | "vec4" => ISFInputType::Color,
            "image" | "texture" => ISFInputType::Image,
            "audio" => ISFInputType::Audio,
            "audiofft" => ISFInputType::AudioFFT,
            "cube" => ISFInputType::Cube,
            "audiowaveform" => ISFInputType::AudioWaveform,
            "audiofrequency" => ISFInputType::AudioFrequency,
            _ => {
                self.diagnostics.add_diagnostic(
                    DiagnosticHelpers::validation_error(
                        format!("Unknown input type '{}' for input '{}'", input_type_str, name),
                        1,
                        1
                    ).with_file_path(file_path.to_string())
                );
                ISFInputType::Float // Default fallback
            }
        };
        
        let default = input_value.get("DEFAULT").cloned();
        let min = input_value.get("MIN").cloned();
        let max = input_value.get("MAX").cloned();
        let identity = input_value.get("IDENTITY").cloned();
        
        let values = input_value.get("VALUES")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().cloned().collect());
            
        let label = input_value.get("LABEL")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let functional = input_value.get("FUNCTIONAL")
            .and_then(|v| v.as_bool());
            
        let description = input_value.get("DESCRIPTION")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(Some(ISFInput {
            name,
            input_type,
            default,
            min,
            max,
            identity,
            values,
            label,
            functional,
            description,
        }))
    }
    
    /// Parse ISF outputs
    fn parse_outputs(&mut self, json_data: &serde_json::Value, file_path: &str) -> Result<Vec<ISFOutput>> {
        let mut outputs = Vec::new();
        
        if let Some(outputs_array) = json_data.get("OUTPUTS").and_then(|v| v.as_array()) {
            for (index, output_value) in outputs_array.iter().enumerate() {
                if let Some(output) = self.parse_single_output(output_value, index, file_path)? {
                    outputs.push(output);
                }
            }
        }
        
        Ok(outputs)
    }
    
    /// Parse a single ISF output
    fn parse_single_output(&mut self, output_value: &serde_json::Value, index: usize, file_path: &str) -> Result<Option<ISFOutput>> {
        if !output_value.is_object() {
            self.diagnostics.add_diagnostic(
                DiagnosticHelpers::validation_error(
                    format!("Output at index {} must be an object", index),
                    1,
                    1
                ).with_file_path(file_path.to_string())
            );
            return Ok(None);
        }
        
        let name = output_value.get("NAME")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Output at index {} missing NAME field", index))?
            .to_string();
            
        let output_type_str = output_value.get("TYPE")
            .and_then(|v| v.as_str())
            .unwrap_or("image")
            .to_lowercase();
            
        let output_type = match output_type_str.as_str() {
            "image" => ISFOutputType::Image,
            "buffer" => ISFOutputType::Buffer,
            "audio" => ISFOutputType::Audio,
            _ => {
                self.diagnostics.add_diagnostic(
                    DiagnosticHelpers::validation_error(
                        format!("Unknown output type '{}' for output '{}'", output_type_str, name),
                        1,
                        1
                    ).with_file_path(file_path.to_string())
                );
                ISFOutputType::Image // Default fallback
            }
        };
        
        let width = output_value.get("WIDTH")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
            
        let height = output_value.get("HEIGHT")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        
        Ok(Some(ISFOutput {
            name,
            output_type,
            width,
            height,
        }))
    }
    
    /// Parse ISF passes
    fn parse_passes(&mut self, json_data: &serde_json::Value, file_path: &str) -> Result<Vec<ISFPass>> {
        let mut passes = Vec::new();
        
        if let Some(passes_array) = json_data.get("PASSES").and_then(|v| v.as_array()) {
            for (index, pass_value) in passes_array.iter().enumerate() {
                if let Some(pass) = self.parse_single_pass(pass_value, index, file_path)? {
                    passes.push(pass);
                }
            }
        }
        
        Ok(passes)
    }
    
    /// Parse a single ISF pass
    fn parse_single_pass(&mut self, pass_value: &serde_json::Value, index: usize, file_path: &str) -> Result<Option<ISFPass>> {
        if !pass_value.is_object() {
            self.diagnostics.add_diagnostic(
                DiagnosticHelpers::validation_error(
                    format!("Pass at index {} must be an object", index),
                    1,
                    1
                ).with_file_path(file_path.to_string())
            );
            return Ok(None);
        }
        
        let target = pass_value.get("TARGET")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let persistent = pass_value.get("PERSISTENT")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let float = pass_value.get("FLOAT")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let width = pass_value.get("WIDTH")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let height = pass_value.get("HEIGHT")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let format = pass_value.get("FORMAT")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let description = pass_value.get("DESCRIPTION")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Parse inputs for this pass
        let mut inputs = HashMap::new();
        if let Some(inputs_obj) = pass_value.get("INPUTS").and_then(|v| v.as_object()) {
            for (key, value) in inputs_obj {
                inputs.insert(key.clone(), value.clone());
            }
        }
        
        Ok(Some(ISFPass {
            target,
            persistent,
            float,
            width,
            height,
            format,
            description,
            inputs,
        }))
    }
    
    /// Convert ISF shader to WGSL
    pub fn convert_to_wgsl(&mut self, isf_shader: &ISFShader) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Add header comments
        wgsl_code.push_str("// Converted from ISF shader\n");
        if let Some(ref description) = isf_shader.metadata.description {
            wgsl_code.push_str(&format!("// Description: {}\n", description));
        }
        if let Some(ref author) = isf_shader.metadata.author {
            wgsl_code.push_str(&format!("// Author: {}\n", author));
        }
        wgsl_code.push('\n');
        
        // Convert inputs to WGSL uniforms
        self.convert_inputs_to_wgsl(&isf_shader.inputs, &mut wgsl_code)?;
        
        // Convert fragment shader to WGSL
        let converted_fragment = self.convert_fragment_shader_to_wgsl(&isf_shader.fragment_shader)?;
        wgsl_code.push_str(&converted_fragment);
        
        Ok(wgsl_code)
    }
    
    /// Convert ISF inputs to WGSL uniforms
    fn convert_inputs_to_wgsl(&mut self, inputs: &[ISFInput], wgsl_code: &mut String) -> Result<()> {
        if inputs.is_empty() {
            return Ok(());
        }
        
        wgsl_code.push_str("struct Uniforms {\n");
        
        for input in inputs {
            let wgsl_type = self.isf_input_type_to_wgsl(&input.input_type);
            let field_name = self.sanitize_identifier(&input.name);
            
            wgsl_code.push_str(&format!("    {}: {},\n", field_name, wgsl_type));
            
            // Add comment with metadata
            if let Some(ref description) = input.description {
                wgsl_code.push_str(&format!("    // {}\n", description));
            }
            
            // Add range information if available
            if let (Some(min), Some(max)) = (&input.min, &input.max) {
                wgsl_code.push_str(&format!("    // Range: {} to {}\n", min, max));
            }
        }
        
        wgsl_code.push_str("}\n\n");
        wgsl_code.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");
        
        Ok(())
    }
    
    /// Convert ISF input type to WGSL type
    fn isf_input_type_to_wgsl(&self, input_type: &ISFInputType) -> &'static str {
        match input_type {
            ISFInputType::Event => "bool",
            ISFInputType::Bool => "bool",
            ISFInputType::Long => "i32",
            ISFInputType::Float => "f32",
            ISFInputType::Point2D => "vec2<f32>",
            ISFInputType::Color => "vec4<f32>",
            ISFInputType::Image => "texture_2d<f32>",
            ISFInputType::Audio => "texture_2d<f32>",
            ISFInputType::AudioFFT => "texture_2d<f32>",
            ISFInputType::Cube => "texture_cube<f32>",
            ISFInputType::AudioWaveform => "texture_2d<f32>",
            ISFInputType::AudioFrequency => "texture_2d<f32>",
        }
    }
    
    /// Convert fragment shader to WGSL
    fn convert_fragment_shader_to_wgsl(&mut self, fragment_shader: &str) -> Result<String> {
        // This is a simplified conversion - in a real implementation,
        // you would need a full GLSL to WGSL transpiler
        let mut wgsl_code = String::new();
        
        // Add common WGSL functions that map to ISF functions
        wgsl_code.push_str("// Common ISF functions\n");
        wgsl_code.push_str("fn isf_fragCoord() -> vec2<f32> {\n");
        wgsl_code.push_str("    return vec2<f32>(0.0, 0.0); // Will be replaced with actual frag coord\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("fn isf_resolution() -> vec2<f32> {\n");
        wgsl_code.push_str("    return vec2<f32>(800.0, 600.0); // Will be replaced with actual resolution\n");
        wgsl_code.push_str("}\n\n");
        
        // Add the main fragment function
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {\n");
        wgsl_code.push_str("    // Original ISF fragment shader would be converted here\n");
        wgsl_code.push_str("    // For now, return a placeholder color\n");
        wgsl_code.push_str("    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n");
        
        Ok(wgsl_code)
    }
    
    /// Sanitize identifier for WGSL
    fn sanitize_identifier(&self, name: &str) -> String {
        // Replace invalid characters and ensure it starts with a letter
        let sanitized = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>();
            
        // Ensure it starts with a letter or underscore
        if sanitized.chars().next().map_or(true, |c| c.is_numeric()) {
            format!("_{}", sanitized)
        } else {
            sanitized
        }
    }
    
    /// Get diagnostics from parsing
    pub fn get_diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
    
    /// Take ownership of diagnostics
    pub fn take_diagnostics(self) -> Diagnostics {
        self.diagnostics
    }
}

impl Default for ISFParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isf_parser_creation() {
        let parser = ISFParser::new();
        assert_eq!(parser.diagnostics.count(), 0);
    }
    
    #[test]
    fn test_simple_isf_parsing() {
        let isf_json = r#"{
            "DESCRIPTION": "Test shader",
            "ISF_VERSION": "1.2",
            "FRAGMENT_SHADER": "void main() { gl_FragColor = vec4(1.0); }"
        }"#;
        
        let mut parser = ISFParser::new();
        let result = parser.parse_isf(isf_json, "test.fs");
        
        assert!(result.is_ok());
        let shader = result.unwrap();
        assert_eq!(shader.metadata.description, Some("Test shader".to_string()));
        assert_eq!(shader.metadata.isf_version, "1.2");
        assert!(shader.fragment_shader.contains("gl_FragColor"));
    }
    
    #[test]
    fn test_isf_with_inputs() {
        let isf_json = r#"{
            "DESCRIPTION": "Test with inputs",
            "ISF_VERSION": "1.2",
            "INPUTS": [
                {
                    "NAME": "color",
                    "TYPE": "color",
                    "DEFAULT": [1.0, 0.0, 0.0, 1.0]
                },
                {
                    "NAME": "speed",
                    "TYPE": "float",
                    "DEFAULT": 1.0,
                    "MIN": 0.0,
                    "MAX": 10.0
                }
            ],
            "FRAGMENT_SHADER": "void main() { gl_FragColor = vec4(1.0); }"
        }"#;
        
        let mut parser = ISFParser::new();
        let result = parser.parse_isf(isf_json, "test.fs");
        
        assert!(result.is_ok());
        let shader = result.unwrap();
        assert_eq!(shader.inputs.len(), 2);
        assert_eq!(shader.inputs[0].name, "color");
        assert_eq!(shader.inputs[1].name, "speed");
    }
    
    #[test]
    fn test_isf_validation_errors() {
        let isf_json = r#"{
            "DESCRIPTION": "Invalid ISF"
        }"#;
        
        let mut parser = ISFParser::new();
        let result = parser.parse_isf(isf_json, "test.fs");
        
        assert!(result.is_err());
        assert!(parser.diagnostics.has_errors());
    }
    
    #[test]
    fn test_wgsl_conversion() {
        let isf_shader = ISFShader {
            metadata: ISFMetadata {
                description: Some("Test shader".to_string()),
                author: Some("Test Author".to_string()),
                credit: None,
                categories: vec![],
                tags: vec![],
                version: "1.0".to_string(),
                isf_version: "1.2".to_string(),
                webgl: false,
                platforms: vec![],
                imported: vec![],
                passes: vec![],
                inputs: vec![],
                outputs: vec![],
                credit_url: None,
                description_extended: None,
            },
            vertex_shader: None,
            fragment_shader: "void main() { gl_FragColor = vec4(1.0); }".to_string(),
            inputs: vec![],
            outputs: vec![],
            imports: vec![],
            passes: vec![],
            persistent_buffers: vec![],
            custom_functions: HashMap::new(),
        };
        
        let mut parser = ISFParser::new();
        let result = parser.convert_to_wgsl(&isf_shader);
        
        assert!(result.is_ok());
        let wgsl = result.unwrap();
        assert!(wgsl.contains("// Converted from ISF shader"));
        assert!(wgsl.contains("// Description: Test shader"));
        assert!(wgsl.contains("// Author: Test Author"));
        assert!(wgsl.contains("@fragment"));
    }
}