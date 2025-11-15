use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfMetadata {
    pub name: String,
    pub description: String,
    pub credit: Option<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfInput {
    pub name: String,
    pub input_type: String,
    pub default: Option<serde_json::Value>,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfConversionResult {
    pub metadata: IsfMetadata,
    pub inputs: Vec<IsfInput>,
    pub wgsl_code: String,
    pub uniforms: Vec<String>,
    pub textures: Vec<String>,
}

pub struct SimpleIsfConverter {
    type_mappings: HashMap<String, String>,
    function_mappings: HashMap<String, String>,
}

impl SimpleIsfConverter {
    pub fn new() -> Self {
        let mut type_mappings = HashMap::new();
        type_mappings.insert("float".to_string(), "f32".to_string());
        type_mappings.insert("vec2".to_string(), "vec2<f32>".to_string());
        type_mappings.insert("vec3".to_string(), "vec3<f32>".to_string());
        type_mappings.insert("vec4".to_string(), "vec4<f32>".to_string());
        type_mappings.insert("mat2".to_string(), "mat2x2<f32>".to_string());
        type_mappings.insert("mat3".to_string(), "mat3x3<f32>".to_string());
        type_mappings.insert("mat4".to_string(), "mat4x4<f32>".to_string());
        
        let mut function_mappings = HashMap::new();
        function_mappings.insert("sin".to_string(), "sin".to_string());
        function_mappings.insert("cos".to_string(), "cos".to_string());
        function_mappings.insert("tan".to_string(), "tan".to_string());
        function_mappings.insert("pow".to_string(), "pow".to_string());
        function_mappings.insert("sqrt".to_string(), "sqrt".to_string());
        function_mappings.insert("mix".to_string(), "mix".to_string());
        function_mappings.insert("clamp".to_string(), "clamp".to_string());
        function_mappings.insert("smoothstep".to_string(), "smoothstep".to_string());
        function_mappings.insert("step".to_string(), "step".to_string());
        function_mappings.insert("length".to_string(), "length".to_string());
        function_mappings.insert("distance".to_string(), "distance".to_string());
        function_mappings.insert("dot".to_string(), "dot".to_string());
        function_mappings.insert("cross".to_string(), "cross".to_string());
        function_mappings.insert("normalize".to_string(), "normalize".to_string());
        function_mappings.insert("texture".to_string(), "textureSample".to_string());
        
        Self {
            type_mappings,
            function_mappings,
        }
    }
    
    pub fn convert_isf_to_wgsl(&self, isf_code: &str) -> Result<IsfConversionResult> {
        // Parse ISF metadata and inputs from JSON comment
        let (metadata, inputs, glsl_code) = self.parse_isf_header(isf_code)?;
        
        // Convert GLSL code to WGSL
        let wgsl_code = self.convert_glsl_to_wgsl(&glsl_code, &inputs)?;
        
        // Extract uniforms and textures
        let uniforms = self.extract_uniforms(&inputs);
        let textures = self.extract_textures(&glsl_code);
        
        Ok(IsfConversionResult {
            metadata,
            inputs,
            wgsl_code,
            uniforms,
            textures,
        })
    }
    
    fn parse_isf_header(&self, isf_code: &str) -> Result<(IsfMetadata, Vec<IsfInput>, String)> {
        // Look for JSON metadata block
        if let Some(start) = isf_code.find("/*{") {
            if let Some(end) = isf_code.find("}*/") {
                let json_str = &isf_code[start + 2..end + 1];
                let metadata: serde_json::Value = serde_json::from_str(json_str)?;
                
                let name = metadata["NAME"].as_str().unwrap_or("Unnamed Shader").to_string();
                let description = metadata["DESCRIPTION"].as_str().unwrap_or("").to_string();
                let credit = metadata["CREDIT"].as_str().map(|s| s.to_string());
                let categories = metadata["CATEGORIES"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                
                let inputs = if let Some(inputs_array) = metadata["INPUTS"].as_array() {
                    inputs_array
                        .iter()
                        .filter_map(|input| {
                            Some(IsfInput {
                                name: input["NAME"].as_str()?.to_string(),
                                input_type: input["TYPE"].as_str()?.to_string(),
                                default: input.get("DEFAULT").cloned(),
                                min: input["MIN"].as_f64().map(|f| f as f32),
                                max: input["MAX"].as_f64().map(|f| f as f32),
                            })
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                
                let glsl_code = isf_code[end + 3..].trim().to_string();
                
                Ok((
                    IsfMetadata {
                        name,
                        description,
                        credit,
                        categories,
                    },
                    inputs,
                    glsl_code,
                ))
            } else {
                Err(anyhow!("Missing closing }*/ in ISF header"))
            }
        } else {
            // No metadata block, treat entire code as GLSL
            Ok((
                IsfMetadata {
                    name: "Unnamed Shader".to_string(),
                    description: String::new(),
                    credit: None,
                    categories: Vec::new(),
                },
                Vec::new(),
                isf_code.to_string(),
            ))
        }
    }
    
    fn convert_glsl_to_wgsl(&self, glsl_code: &str, inputs: &[IsfInput]) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Add struct for uniforms
        if !inputs.is_empty() {
            wgsl_code.push_str("struct Uniforms {\n");
            for input in inputs {
                let wgsl_type = self.type_mappings.get(&input.input_type)
                    .unwrap_or(&"f32".to_string());
                wgsl_code.push_str(&format!("    {}: {},\n", input.name, wgsl_type));
            }
            wgsl_code.push_str("}\n\n");
            wgsl_code.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");
        }
        
        // Add texture and sampler declarations
        if glsl_code.contains("IMG_PIXEL") || glsl_code.contains("texture") {
            wgsl_code.push_str("@group(0) @binding(1) var inputImage: texture_2d<f32>;\n");
            wgsl_code.push_str("@group(0) @binding(2) var inputImage_sampler: sampler;\n