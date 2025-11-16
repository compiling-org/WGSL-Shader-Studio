#!/usr/bin/env rust-script
//! Test ISF loader functionality
//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! anyhow = "1.0"
//! ```

use std::path::Path;
use std::fs;

// Copy the ISF loader types directly for testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IsfShader {
    pub name: String,
    pub source: String,
    pub inputs: Vec<ShaderInput>,
    pub outputs: Vec<ShaderOutput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShaderInput {
    pub name: String,
    pub input_type: InputType,
    pub value: ShaderValue,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub default: Option<f32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShaderOutput {
    pub name: String,
    pub output_type: OutputType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InputType {
    Float,
    Bool,
    Color,
    Point2D,
    Image,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ShaderValue {
    Float(f32),
    Bool(bool),
    Color([f32; 4]),
    Point2D([f32; 2]),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OutputType {
    Image,
    Float,
}

/// Load ISF shaders from a directory
pub fn load_isf_shaders_from_directory(dir_path: &str) -> Result<Vec<IsfShader>, Box<dyn std::error::Error>> {
    let mut shaders = Vec::new();
    let path = Path::new(dir_path);

    if !path.exists() {
        return Err(format!("Directory {} does not exist", dir_path).into());
    }

    // Recursively find .fs files (ISF fragment shaders)
    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively load from subdirectories
            let mut sub_shaders = load_isf_shaders_from_directory(path.to_str().unwrap())?;
            shaders.append(&mut sub_shaders);
        } else if let Some(extension) = path.extension() {
            if extension == "fs" {
                if let Some(file_name) = path.file_stem() {
                    let name = file_name.to_string_lossy().to_string();
                    match load_isf_shader(&path) {
                        Ok(shader) => shaders.push(shader),
                        Err(e) => eprintln!("Failed to load shader {}: {}", name, e),
                    }
                }
            }
        }
    }

    Ok(shaders)
}

/// Load a single ISF shader from file
pub fn load_isf_shader(file_path: &Path) -> Result<IsfShader, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let name = file_path.file_stem()
        .ok_or("Invalid file name")?
        .to_string_lossy()
        .to_string();

    IsfShader::parse(&name, &content)
}

impl IsfShader {
    pub fn parse(name: &str, source: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Basic ISF parsing - look for JSON metadata in comments
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // Parse JSON metadata from comments
        if let Some(json_start) = source.find("/*{") {
            if let Some(json_end) = source[json_start..].find("}*/") {
                let json_str = &source[json_start + 2..json_start + json_end + 1];
                if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(json_str) {
                    // Parse inputs
                    if let Some(inputs_json) = metadata.get("INPUTS") {
                        if let Some(inputs_array) = inputs_json.as_array() {
                            for input_json in inputs_array {
                                if let Some(name) = input_json.get("NAME").and_then(|n| n.as_str()) {
                                    let input_type = match input_json.get("TYPE").and_then(|t| t.as_str()) {
                                        Some("float") => InputType::Float,
                                        Some("bool") => InputType::Bool,
                                        Some("color") => InputType::Color,
                                        Some("point2D") => InputType::Point2D,
                                        Some("image") => InputType::Image,
                                        _ => InputType::Float,
                                    };

                                    let default = input_json.get("DEFAULT")
                                        .and_then(|d| d.as_f64())
                                        .map(|d| d as f32);

                                    let min = input_json.get("MIN")
                                        .and_then(|m| m.as_f64())
                                        .map(|m| m as f32);

                                    let max = input_json.get("MAX")
                                        .and_then(|m| m.as_f64())
                                        .map(|m| m as f32);

                                    let value = match input_type {
                                        InputType::Float => ShaderValue::Float(default.unwrap_or(0.0)),
                                        InputType::Bool => ShaderValue::Bool(default.map(|d| d > 0.0).unwrap_or(false)),
                                        InputType::Color => ShaderValue::Color([1.0, 1.0, 1.0, 1.0]),
                                        InputType::Point2D => ShaderValue::Point2D([0.0, 0.0]),
                                        InputType::Image => ShaderValue::Float(0.0), // Placeholder
                                    };

                                    inputs.push(ShaderInput {
                                        name: name.to_string(),
                                        input_type,
                                        value,
                                        min,
                                        max,
                                        default,
                                    });
                                }
                            }
                        }
                    }

                    // Parse outputs
                    if let Some(outputs_json) = metadata.get("OUTPUTS") {
                        if let Some(outputs_array) = outputs_json.as_array() {
                            for output_json in outputs_array {
                                if let Some(name) = output_json.get("NAME").and_then(|n| n.as_str()) {
                                    let output_type = match output_json.get("TYPE").and_then(|t| t.as_str()) {
                                        Some("image") => OutputType::Image,
                                        Some("float") => OutputType::Float,
                                        _ => OutputType::Image,
                                    };

                                    outputs.push(ShaderOutput {
                                        name: name.to_string(),
                                        output_type,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            name: name.to_string(),
            source: source.to_string(),
            inputs,
            outputs,
        })
    }
}

fn main() {
    println!("Testing ISF loader functionality...");
    
    // Test local ISF directory
    let local_dir = "./isf-shaders";
    if Path::new(local_dir).exists() {
        println!("\nTesting load from local ISF directory: {}", local_dir);
        match load_isf_shaders_from_directory(local_dir) {
            Ok(shaders) => {
                println!("✓ Successfully loaded {} shaders from local directory", shaders.len());
                
                // Show first few shaders as examples
                for (i, shader) in shaders.iter().take(3).enumerate() {
                    println!("\nShader {}: {} (inputs: {})", 
                        i + 1, 
                        shader.name, 
                        shader.inputs.len()
                    );
                    
                    // Show input details
                    for input in &shader.inputs {
                        println!("  - Input: {} (type: {:?}, default: {:?})", 
                            input.name, 
                            input.input_type,
                            input.default
                        );
                    }
                }
                
                if shaders.len() > 3 {
                    println!("\n... and {} more shaders", shaders.len() - 3);
                }
            }
            Err(e) => {
                println!("✗ Failed to load from local directory: {}", e);
            }
        }
    } else {
        println!("\n✗ Local ISF directory not found at: {}", local_dir);
    }
    
    // Test Magic directory
    let magic_dir = r"C:\Program Files\Magic\Modules2\ISF";
    if Path::new(magic_dir).exists() {
        println!("\nTesting load from Magic directory: {}", magic_dir);
        match load_isf_shaders_from_directory(magic_dir) {
            Ok(shaders) => {
                println!("✓ Successfully loaded {} shaders from Magic directory", shaders.len());
            }
            Err(e) => {
                println!("✗ Failed to load from Magic directory: {}", e);
            }
        }
    } else {
        println!("\n✗ Magic directory not found at: {}", magic_dir);
    }
    
    println!("\nISF loader test completed!");
}