use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};
use serde_json::Value;

/// ISF loader for local ISF files
pub struct IsfLoader {
    isf_directory: String,
}

impl IsfLoader {
    pub fn new() -> Self {
        Self {
            isf_directory: "C:\\Program Files\\Magic\\Modules2\\ISF".to_string(),
        }
    }

    pub fn with_directory<P: AsRef<Path>>(path: P) -> Self {
        Self {
            isf_directory: path.as_ref().to_string_lossy().to_string(),
        }
    }

    /// Load all ISF files from the directory
    pub fn load_all_isfs(&self) -> Result<Vec<(String, String, Value)>> {
        let mut isf_files = Vec::new();
        
        if !Path::new(&self.isf_directory).exists() {
            return Err(anyhow!("ISF directory not found: {}", self.isf_directory));
        }

        let entries = fs::read_dir(&self.isf_directory)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("fs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(isf_json) = self.extract_isf_json(&content) {
                        let name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        isf_files.push((name, content, isf_json));
                    }
                }
            }
        }
        
        Ok(isf_files)
    }

    /// Extract ISF JSON from shader file
    fn extract_isf_json(&self, content: &str) -> Result<Value> {
        // Look for ISF JSON block (between /* and */ at the beginning)
        if let Some(start) = content.find("/*") {
            if let Some(end) = content.find("*/") {
                let json_str = &content[start + 2..end];
                let parsed: Value = serde_json::from_str(json_str)?;
                return Ok(parsed);
            }
        }
        
        // Look for ISF JSON block (between { and } at the beginning)
        if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                let json_str = &content[start..=end];
                let parsed: Value = serde_json::from_str(json_str)?;
                return Ok(parsed);
            }
        }
        
        Err(anyhow!("No ISF JSON found in file"))
    }

    /// Parse ISF inputs from JSON
    pub fn parse_isf_inputs(&self, isf_json: &Value) -> Result<Vec<IsfInput>> {
        let mut inputs = Vec::new();
        
        if let Some(inputs_array) = isf_json.get("INPUTS").and_then(|v| v.as_array()) {
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

    /// Get ISF metadata
    pub fn get_isf_metadata(&self, isf_json: &Value) -> IsfMetadata {
        IsfMetadata {
            credit: isf_json.get("CREDIT").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: isf_json.get("DESCRIPTION").and_then(|v| v.as_str()).map(|s| s.to_string()),
            categories: isf_json.get("CATEGORIES").and_then(|cats| {
                cats.as_array().map(|arr| {
                    arr.iter().filter_map(|cat| cat.as_str().map(|s| s.to_string())).collect()
                })
            }),
            isf_version: isf_json.get("ISFVSN").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

impl Default for IsfLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct IsfInput {
    pub name: String,
    pub input_type: String,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct IsfMetadata {
    pub credit: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub isf_version: Option<String>,
}

/// Load ISF files from Magic installation directory
pub fn load_magic_isfs() -> Result<Vec<(String, String, Value)>> {
    let loader = IsfLoader::new();
    loader.load_all_isfs()
}

/// Load ISF files from custom directory
pub fn load_isfs_from_directory<P: AsRef<Path>>(path: P) -> Result<Vec<(String, String, Value)>> {
    let loader = IsfLoader::with_directory(path);
    loader.load_all_isfs()
}