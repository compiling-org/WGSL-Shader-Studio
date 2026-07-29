use std::fs;
use std::path::Path;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json::Value;

/// ISF integration for loading and converting ISF shaders
pub struct IsfIntegration {
    isf_directory: String,
    loaded_isfs: HashMap<String, LoadedIsf>,
}

#[derive(Debug, Clone)]
pub struct LoadedIsf {
    pub name: String,
    pub content: String,
    pub metadata: IsfMetadata,
    pub inputs: Vec<IsfInput>,
    pub converted_wgsl: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IsfMetadata {
    pub credit: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub isf_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IsfInput {
    pub name: String,
    pub input_type: String,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub default_value: Option<Value>,
}

impl IsfIntegration {
    pub fn new() -> Self {
        Self {
            isf_directory: "C:\\Program Files\\Magic\\Modules2\\ISF".to_string(),
            loaded_isfs: HashMap::new(),
        }
    }

    pub fn with_directory<P: AsRef<Path>>(path: P) -> Self {
        Self {
            isf_directory: path.as_ref().to_string_lossy().to_string(),
            loaded_isfs: HashMap::new(),
        }
    }

    /// Load all ISF files from the Magic directory
    pub fn load_magic_isfs(&mut self) -> Result<Vec<String>> {
        if !Path::new(&self.isf_directory).exists() {
            // Try alternative paths
            let alt_paths = vec![
                "C:\\Program Files (x86)\\Magic\\Modules2\\ISF",
                "C:\\Magic\\Modules2\\ISF",
                "C:\\Program Files\\Magic\\ISF",
            ];

            for path in alt_paths {
                if Path::new(path).exists() {
                    self.isf_directory = path.to_string();
                    break;
                }
            }

            if !Path::new(&self.isf_directory).exists() {
                return Err(anyhow!("Magic ISF directory not found. Please specify the correct path."));
            }
        }

        self.load_isfs_from_directory(&self.isf_directory)
    }

    /// Load ISF files from a specific directory
    pub fn load_isfs_from_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<String>> {
        let mut loaded_names = Vec::new();
        
        let entries = fs::read_dir(path)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("fs") {
                match self.load_isf_file(&path) {
                    Ok(name) => {
                        loaded_names.push(name.clone());
                        println!("Loaded ISF: {}", name);
                    }
                    Err(e) => {
                        eprintln!("Failed to load ISF file {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(loaded_names)
    }

    /// Load a single ISF file
    fn load_isf_file<P: AsRef<Path>>(&mut self, path: P) -> Result<String> {
        let content = fs::read_to_string(&path)?;
        let name = path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Extract ISF metadata
        let metadata = self.extract_isf_metadata(&content)?;
        let inputs = self.extract_isf_inputs(&content)?;

        let loaded_isf = LoadedIsf {
            name: name.clone(),
            content: content.clone(),
            metadata,
            inputs: inputs.clone(),
            converted_wgsl: None,
        };

        self.loaded_isfs.insert(name.clone(), loaded_isf);
        Ok(name)
    }

    /// Extract ISF metadata from shader content
    fn extract_isf_metadata(&self, content: &str) -> Result<IsfMetadata> {
        // Try to find JSON block at the beginning of the file
        let json_str = self.extract_json_block(content)?;
        let json: Value = serde_json::from_str(&json_str)?;

        Ok(IsfMetadata {
            credit: json.get("CREDIT").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: json.get("DESCRIPTION").and_then(|v| v.as_str()).map(|s| s.to_string()),
            categories: json.get("CATEGORIES").and_then(|cats| {
                cats.as_array().map(|arr| {
                    arr.iter().filter_map(|cat| cat.as_str().map(|s| s.to_string())).collect()
                })
            }),
            isf_version: json.get("ISFVSN").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }

    /// Extract ISF inputs from shader content
    fn extract_isf_inputs(&self, content: &str) -> Result<Vec<IsfInput>> {
        let json_str = self.extract_json_block(content)?;
        let json: Value = serde_json::from_str(&json_str)?;
        
        let mut inputs = Vec::new();
        
        if let Some(inputs_array) = json.get("INPUTS").and_then(|v| v.as_array()) {
            for input in inputs_array {
                if let (Some(name), Some(input_type)) = (
                    input.get("NAME").and_then(|v| v.as_str()),
                    input.get("TYPE").and_then(|v| v.as_str())
                ) {
                    inputs.push(IsfInput {
                        name: name.to_string(),
                        input_type: input_type.to_string(),
                        min_value: input.get("MIN").and_then(|v| v.as_f64()).map(|v| v as f32),
                        max_value: input.get("MAX").and_then(|v| v.as_f64()).map(|v| v as f32),
                        default_value: input.get("DEFAULT").cloned(),
                    });
                }
            }
        }
        
        Ok(inputs)
    }

    /// Extract JSON block from ISF file
    fn extract_json_block(&self, content: &str) -> Result<String> {
        // Look for JSON block between /* and */ at the beginning
        if let Some(start) = content.find("/*") {
            if let Some(end) = content.find("*/") {
                let json_str = content[start + 2..end].trim();
                return Ok(json_str.to_string());
            }
        }
        
        // Look for JSON block between { and } at the beginning
        if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                let json_str = &content[start..=end];
                return Ok(json_str.to_string());
            }
        }
        
        // Return empty JSON object if no block found
        Ok("{}".to_string())
    }

    /// Convert ISF to WGSL
    pub fn convert_isf_to_wgsl(&mut self, name: &str) -> Result<String> {
        let isf = self.loaded_isfs.get(name)
            .ok_or_else(|| anyhow!("ISF '{}' not found", name))?;

        let wgsl_code = self.perform_isf_conversion(&isf.content, &isf.inputs)?;
        
        // Store the converted WGSL
        if let Some(loaded_isf) = self.loaded_isfs.get_mut(name) {
            loaded_isf.converted_wgsl = Some(wgsl_code.clone());
        }

        Ok(wgsl_code)
    }

    /// Perform the actual ISF to WGSL conversion
    fn perform_isf_conversion(&self, isf_content: &str, inputs: &[IsfInput]) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Generate uniform buffer from inputs
        wgsl_code.push_str("struct Uniforms {\n");
        for input in inputs {
            let wgsl_type = self.map_isf_type_to_wgsl(&input.input_type)?;
            let field_name = self.sanitize_identifier(&input.name);
            wgsl_code.push_str(&format!("    {}: {},\n", field_name, wgsl_type));
        }
        
        // Add standard ISF uniforms
        wgsl_code.push_str("    time: f32,\n");
        wgsl_code.push_str("    timeDelta: f32,\n");
        wgsl_code.push_str("    frame: u32,\n");
        wgsl_code.push_str("    fps: f32,\n");
        wgsl_code.push_str("    progress: f32,\n");
        wgsl_code.push_str("    renderSize: vec2<f32>,\n");
        wgsl_code.push_str("    aspectRatio: f32,\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");
        
        // Extract the actual shader code (after the JSON block)
        let shader_code = self.extract_shader_code(isf_content);
        
        // Generate vertex shader
        wgsl_code.push_str("@vertex\n");
        wgsl_code.push_str("fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {\n");
        wgsl_code.push_str("    return vec4<f32>(position, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n\n");
        
        // Generate fragment shader
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {\n");
        wgsl_code.push_str("    let uv = coord.xy / uniforms.renderSize;\n");
        wgsl_code.push_str("    let TIME = uniforms.time;\n");
        wgsl_code.push_str("    let RENDERSIZE = uniforms.renderSize;\n");
        wgsl_code.push_str("    let ASPECT = uniforms.aspectRatio;\n");
        
        // Add input variable mappings
        for input in inputs {
            let var_name = self.sanitize_identifier(&input.name);
            wgsl_code.push_str(&format!("    let {} = uniforms.{};\n", var_name, var_name));
        }
        
        wgsl_code.push_str("\n");
        
        // Convert and insert the ISF shader code
        let converted_code = self.convert_isf_functions(&shader_code);
        wgsl_code.push_str(&converted_code);
        wgsl_code.push_str("\n");
        
        // Default return if not provided in ISF code
        wgsl_code.push_str("    return vec4<f32>(1.0, 0.0, 0.0, 1.0>;\n");
        wgsl_code.push_str("}\n");
        
        Ok(wgsl_code)
    }

    /// Extract shader code from ISF content (after JSON metadata)
    fn extract_shader_code(&self, content: &str) -> String {
        // Remove JSON block and return the remaining shader code
        if let Some(start) = content.find("/*") {
            if let Some(end) = content.find("*/") {
                return content[end + 2..].trim().to_string();
            }
        }
        
        // If no JSON block found, return the whole content
        content.to_string()
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

    /// Get loaded ISF names
    pub fn get_loaded_isfs(&self) -> Vec<String> {
        self.loaded_isfs.keys().cloned().collect()
    }

    /// Get ISF by name
    pub fn get_isf(&self, name: &str) -> Option<&LoadedIsf> {
        self.loaded_isfs.get(name)
    }

    /// Get ISF inputs
    pub fn get_isf_inputs(&self, name: &str) -> Option<&Vec<IsfInput>> {
        self.loaded_isfs.get(name).map(|isf| &isf.inputs)
    }
}

impl Default for IsfIntegration {
    fn default() -> Self {
        Self::new()
    }
}