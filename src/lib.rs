//! # Resolume ISF Shaders Rust FFGL
//!
//! FFGL plugin for Resolume with ISF (Interactive Shader Format) support.
//! Professional VJ shader effects for live video performance.

use std::collections::HashMap;

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
pub struct IsfShader {
    name: String,
    source: String,
    inputs: Vec<ShaderInput>,
    outputs: Vec<ShaderOutput>,
}

/// Shader input parameter
pub struct ShaderInput {
    name: String,
    input_type: InputType,
    value: ShaderValue,
}

/// Shader output
pub struct ShaderOutput {
    name: String,
    output_type: OutputType,
}

/// Input parameter types
pub enum InputType {
    Float,
    Bool,
    Color,
    Point2D,
    Image,
}

/// Shader value types
pub enum ShaderValue {
    Float(f32),
    Bool(bool),
    Color([f32; 4]),
    Point2D([f32; 2]),
}

/// Output types
pub enum OutputType {
    Image,
    Float,
}

/// Rendering parameters
pub struct RenderParameters {
    width: u32,
    height: u32,
    time: f32,
    frame_rate: f32,
}

/// Plugin state management
pub struct PluginState {
    current_shader: Option<String>,
    is_enabled: bool,
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
        // Placeholder for ISF parsing logic
        Ok(Self {
            name: name.to_string(),
            source: source.to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
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