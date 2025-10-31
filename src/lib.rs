//! # Resolume ISF Shaders Rust FFGL
//!
//! FFGL plugin for Resolume with ISF (Interactive Shader Format) support.
//! Professional VJ shader effects for live video performance.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Main FFGL ISF shader plugin structure
pub struct ResolumeIsfShadersRustFfgl {
    // ISF shader collection
    shaders: HashMap<String, IsfShader>,

    // Rendering parameters
    render_params: RenderParameters,

    // FFGL plugin state
    plugin_state: PluginState,
}

/// ISF shader representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfShader {
    pub name: String,
    pub source: String,
    pub inputs: Vec<ShaderInput>,
    pub outputs: Vec<ShaderOutput>,
}

/// Shader input parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderInput {
    pub name: String,
    pub input_type: InputType,
    pub value: ShaderValue,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub default: Option<f32>,
}

/// Shader output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderOutput {
    pub name: String,
    pub output_type: OutputType,
}

/// Input parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    Float,
    Bool,
    Color,
    Point2D,
    Image,
}

/// Shader value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderValue {
    Float(f32),
    Bool(bool),
    Color([f32; 4]),
    Point2D([f32; 2]),
}

/// Output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    Image,
    Float,
}

/// Rendering parameters
pub struct RenderParameters {
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub frame_rate: f32,
}

/// Plugin state management
pub struct PluginState {
    pub current_shader: Option<String>,
    pub is_enabled: bool,
}

impl Default for ResolumeIsfShadersRustFfgl {
    fn default() -> Self {
        Self {
            shaders: HashMap::new(),
            render_params: RenderParameters::default(),
            plugin_state: PluginState::default(),
        }
    }
}

impl Default for RenderParameters {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            time: 0.0,
            frame_rate: 30.0,
        }
    }
}

impl Default for PluginState {
    fn default() -> Self {
        Self {
            current_shader: None,
            is_enabled: true,
        }
    }
}

impl ResolumeIsfShadersRustFfgl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_isf_shader(&mut self, name: &str, source: &str) -> Result<(), Box<dyn std::error::Error>> {
        let shader = IsfShader::parse(name, source)?;
        self.shaders.insert(name.to_string(), shader);
        Ok(())
    }

    pub fn set_current_shader(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.shaders.contains_key(name) {
            self.plugin_state.current_shader = Some(name.to_string());
            Ok(())
        } else {
            Err("Shader not found".into())
        }
    }

    pub fn render_frame(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for FFGL rendering logic
        self.render_params.time += 1.0 / self.render_params.frame_rate;

        // Simple pass-through for now
        output.copy_from_slice(input);
        Ok(())
    }

    pub fn set_parameter(&mut self, shader_name: &str, param_name: &str, value: ShaderValue) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(shader) = self.shaders.get_mut(shader_name) {
            shader.set_parameter(param_name, value)?;
        }
        Ok(())
    }

    pub fn get_parameter(&self, shader_name: &str, param_name: &str) -> Option<&ShaderValue> {
        self.shaders.get(shader_name)?.get_parameter(param_name)
    }
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

    pub fn set_parameter(&mut self, name: &str, value: ShaderValue) -> Result<(), Box<dyn std::error::Error>> {
        for input in &mut self.inputs {
            if input.name == name {
                input.value = value;
                return Ok(());
            }
        }
        Err("Parameter not found".into())
    }

    pub fn get_parameter(&self, name: &str) -> Option<&ShaderValue> {
        for input in &self.inputs {
            if input.name == name {
                return Some(&input.value);
            }
        }
        None
    }
}

/// Simple test function to verify the library compiles
pub fn hello_resolume_isf_shaders_rust_ffgl() -> &'static str {
    "Hello from Resolume ISF Shaders Rust FFGL! Professional VJ shader effects."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello_resolume_isf_shaders_rust_ffgl(), "Hello from Resolume ISF Shaders Rust FFGL! Professional VJ shader effects.");
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = ResolumeIsfShadersRustFfgl::new();
        assert!(plugin.shaders.is_empty());
        assert!(plugin.plugin_state.current_shader.is_none());
    }

    #[test]
    fn test_load_isf_shader() {
        let mut plugin = ResolumeIsfShadersRustFfgl::new();
        let result = plugin.load_isf_shader("test", "shader source");
        assert!(result.is_ok());
        assert!(plugin.shaders.contains_key("test"));
    }

    #[test]
    fn test_set_current_shader() {
        let mut plugin = ResolumeIsfShadersRustFfgl::new();
        plugin.load_isf_shader("test", "shader source").unwrap();
        let result = plugin.set_current_shader("test");
        assert!(result.is_ok());
        assert_eq!(plugin.plugin_state.current_shader, Some("test".to_string()));
    }

    #[test]
    fn test_set_current_shader_not_found() {
        let mut plugin = ResolumeIsfShadersRustFfgl::new();
        let result = plugin.set_current_shader("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_render_frame() {
        let mut plugin = ResolumeIsfShadersRustFfgl::new();
        let input = vec![255u8; 1920 * 1080 * 4];
        let mut output = vec![0u8; 1920 * 1080 * 4];
        let result = plugin.render_frame(&input, &mut output);
        assert!(result.is_ok());
        assert_eq!(output.len(), input.len());
    }

    #[test]
    fn test_isf_shader_parse() {
        let shader = IsfShader::parse("test", "source");
        assert!(shader.is_ok());
        let shader = shader.unwrap();
        assert_eq!(shader.name, "test");
        assert_eq!(shader.source, "source");
    }

    #[test]
    fn test_shader_parameter_operations() {
        let mut shader = IsfShader::parse("test", "source").unwrap();
        shader.inputs.push(ShaderInput {
            name: "param1".to_string(),
            input_type: InputType::Float,
            value: ShaderValue::Float(0.0),
        });

        let result = shader.set_parameter("param1", ShaderValue::Float(1.0));
        assert!(result.is_ok());

        let value = shader.get_parameter("param1");
        assert!(matches!(value, Some(ShaderValue::Float(1.0))));
    }
}

// Module declarations
pub mod audio;
pub mod gesture_control;
pub mod shader_converter;
pub mod shader_renderer;
pub mod real_shader_renderer;
pub mod wgpu_renderer;
pub mod isf_loader;
pub mod ffgl_plugin;
pub mod ui;

// Re-export main types for easier use
pub use audio::*;
pub use gesture_control::*;
pub use shader_converter::*;
pub use shader_renderer::*;
pub use isf_loader::*;
pub use ffgl_plugin::*;
pub use ui::*;