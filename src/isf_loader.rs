//! ISF shader loading utilities

// Define the types locally for the binary
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
use std::fs;
use std::path::Path;

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

    pub fn from_wgsl(name: String, source: String) -> Self {
        Self {
            name,
            source,
            inputs: Vec::new(),
            outputs: vec![ShaderOutput { name: "image".to_string(), output_type: OutputType::Image }],
        }
    }
}

/// Load ISF shaders from the specified Resolume directories
pub fn load_resolume_isf_shaders() -> Result<Vec<IsfShader>, Box<dyn std::error::Error>> {
    let mut all_shaders = Vec::new();

    // Standard Resolume ISF directories
    let directories = vec![
        r"C:\Program Files\Magic\Modules2\ISF\fractal",
        r"C:\Program Files\Magic\Modules2\ISF\fractal 2",
        r"C:\Program Files\Magic\Modules2\ISF\final",
    ];

    for dir in directories {
        match load_isf_shaders_from_directory(dir) {
            Ok(mut shaders) => {
                all_shaders.append(&mut shaders);
                println!("Loaded {} shaders from {}", shaders.len(), dir);
            }
            Err(e) => {
                eprintln!("Failed to load shaders from {}: {}", dir, e);
            }
        }
    }

    // Also load local project assets
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let local_isf_dir = project_root.join("assets").join("isf");
    if local_isf_dir.exists() {
        if let Ok(mut shaders) = load_isf_shaders_from_directory(local_isf_dir.to_str().unwrap()) {
            println!("Loaded {} local ISF shaders from {}", shaders.len(), local_isf_dir.display());
            all_shaders.append(&mut shaders);
        }
    }
    let local_wgsl_dir = project_root.join("assets").join("shaders");
    if local_wgsl_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&local_wgsl_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "wgsl" {
                        if let Ok(contents) = std::fs::read_to_string(&path) {
                            let name = path.file_stem().unwrap().to_string_lossy().to_string();
                            all_shaders.push(IsfShader::from_wgsl(name, contents));
                        }
                    }
                }
            }
        }
    }

    println!("Total ISF shaders loaded: {}", all_shaders.len());
    Ok(all_shaders)
}

/// Validate ISF shader syntax and structure
pub fn validate_isf_shader(shader: &IsfShader) -> Result<(), Box<dyn std::error::Error>> {
    // Check for required fields
    if shader.name.is_empty() {
        return Err("Shader name cannot be empty".into());
    }

    if shader.source.is_empty() {
        return Err("Shader source cannot be empty".into());
    }

    // Check for GLSL main function
    if !shader.source.contains("void main(") {
        return Err("Shader must contain a main function".into());
    }

    // Validate inputs
    for input in &shader.inputs {
        if input.name.is_empty() {
            return Err("Input parameter name cannot be empty".into());
        }

        // Check value ranges
        if let (Some(min), Some(max)) = (input.min, input.max) {
            if min > max {
                return Err(format!("Invalid range for parameter {}: min > max", input.name).into());
            }
        }
    }

    // Validate outputs
    if shader.outputs.is_empty() {
        return Err("Shader must have at least one output".into());
    }

    Ok(())
}

/// Get shader metadata for UI display
pub fn get_shader_metadata(shader: &IsfShader) -> ShaderMetadata {
    ShaderMetadata {
        name: shader.name.clone(),
        description: extract_description(&shader.source),
        category: extract_category(&shader.source),
        author: extract_author(&shader.source),
        inputs: shader.inputs.iter().map(|input| InputMetadata {
            name: input.name.clone(),
            input_type: input.input_type.clone(),
            min: input.min,
            max: input.max,
            default: input.default,
        }).collect(),
        outputs: shader.outputs.iter().map(|output| OutputMetadata {
            name: output.name.clone(),
            output_type: output.output_type.clone(),
        }).collect(),
    }
}

/// Extract description from ISF shader comments
fn extract_description(source: &str) -> Option<String> {
    // Look for description in JSON metadata
    if let Some(start) = source.find("/*{") {
        if let Some(end) = source[start..].find("}*/") {
            let json_str = &source[start + 2..start + end + 1];
            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(desc) = metadata.get("DESCRIPTION").and_then(|d| d.as_str()) {
                    return Some(desc.to_string());
                }
            }
        }
    }
    None
}

/// Extract category from ISF shader
fn extract_category(source: &str) -> Option<String> {
    // Look for category in JSON metadata
    if let Some(start) = source.find("/*{") {
        if let Some(end) = source[start..].find("}*/") {
            let json_str = &source[start + 2..start + end + 1];
            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(cat) = metadata.get("CATEGORIES").and_then(|c| c.as_str()) {
                    return Some(cat.to_string());
                }
            }
        }
    }
    None
}

/// Extract author from ISF shader
fn extract_author(source: &str) -> Option<String> {
    // Look for author in JSON metadata
    if let Some(start) = source.find("/*{") {
        if let Some(end) = source[start..].find("}*/") {
            let json_str = &source[start + 2..start + end + 1];
            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(auth) = metadata.get("CREDIT").and_then(|c| c.as_str()) {
                    return Some(auth.to_string());
                }
            }
        }
    }
    None
}

/// Shader metadata for UI display
#[derive(Debug, Clone)]
pub struct ShaderMetadata {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub author: Option<String>,
    pub inputs: Vec<InputMetadata>,
    pub outputs: Vec<OutputMetadata>,
}

/// Input parameter metadata
#[derive(Debug, Clone)]
pub struct InputMetadata {
    pub name: String,
    pub input_type: InputType,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub default: Option<f32>,
}

/// Output metadata
#[derive(Debug, Clone)]
pub struct OutputMetadata {
    pub name: String,
    pub output_type: OutputType,
}