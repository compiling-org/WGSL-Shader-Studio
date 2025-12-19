//! FFGL plugin interface implementation

// FIXED: Removed invalid imports - these don't exist in root crate
use crate::ShaderValue;
use crate::audio_system::AudioMidiSystem;
use bevy::prelude::Resource;
use crate::ResolumeIsfShadersRustFfgl;
use std::os::raw::{c_char, c_void, c_int, c_uint, c_float};
use std::ptr;
use std::collections::HashMap;
use std::sync::Arc;

/// FFGL plugin instance
pub struct FfglPlugin {
    plugin: ResolumeIsfShadersRustFfgl,
    current_shader: Option<String>,
    audio_midi_system: Arc<AudioMidiSystem>,
    parameter_cache: HashMap<String, f32>,
}

/// FFGL plugin info structure (matches FFGL SDK)
#[repr(C)]
pub struct PluginInfoStruct {
    pub api_major_version: c_uint,
    pub api_minor_version: c_uint,
    pub plugin_unique_id: [c_char; 4],
    pub plugin_name: [c_char; 16],
    pub plugin_type: [c_char; 16],
}

impl Default for PluginInfoStruct {
    fn default() -> Self {
        let mut plugin_name = [0i8; 16];
        let name = b"ISF Shaders\0";
        for (i, &byte) in name.iter().enumerate() {
            if i < plugin_name.len() {
                plugin_name[i] = byte as i8;
            }
        }

        let mut plugin_type = [0i8; 16];
        let ptype = b"Effect\0";
        for (i, &byte) in ptype.iter().enumerate() {
            if i < plugin_type.len() {
                plugin_type[i] = byte as i8;
            }
        }

        Self {
            api_major_version: 1,
            api_minor_version: 5,
            plugin_unique_id: [b'I' as i8, b'S' as i8, b'F' as i8, b'S' as i8],
            plugin_name,
            plugin_type,
        }
    }
}

/// FFGL parameter structure
#[repr(C)]
pub struct ParameterStruct {
    pub name: [c_char; 16],
    pub default_value: c_float,
    pub min_value: c_float,
    pub max_value: c_float,
    pub type_: c_int,
}

impl FfglPlugin {
    pub fn new() -> Self {
        Self {
            plugin: ResolumeIsfShadersRustFfgl::new(),
            current_shader: None,
            audio_midi_system: Arc::new(AudioMidiSystem::new()),
            parameter_cache: HashMap::new(),
        }
    }

    /// Initialize the plugin
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load ISF shaders from Resolume directories
        let shaders = crate::isf_loader::load_resolume_isf_shaders()?;
        for shader in shaders {
            self.plugin.load_isf_shader(&shader.name, &shader.source)?;
        }
        Ok(())
    }

    /// Process a video frame
    pub fn process_frame(&mut self, input: &[u8], output: &mut [u8], width: u32, height: u32, time: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Update render parameters
        self.plugin.render_params.width = width;
        self.plugin.render_params.height = height;
        self.plugin.render_params.time = time;

        // Render frame
        self.plugin.render_frame(input, output)?;
        Ok(())
    }

    /// Set parameter value
    pub fn set_parameter(&mut self, param_index: usize, value: f32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(shader_name) = &self.current_shader {
            // Map parameter index to shader parameter name
            if let Some(shader) = self.plugin.shaders.get_mut(shader_name) {
                if let Some(input) = shader.inputs.get(param_index) {
                    let shader_value = match input.input_type {
                        crate::InputType::Float => ShaderValue::Float(value),
                        crate::InputType::Bool => ShaderValue::Bool(value > 0.0),
                        _ => ShaderValue::Float(value), // Default to float for now
                    };
                    let input_name = input.name.clone();
                    shader.set_parameter(&input_name, shader_value)?;
                }
            }
        }
        Ok(())
    }

    /// Get parameter value with audio/MIDI modulation
    pub fn get_parameter(&self, param_index: usize) -> f32 {
        if let Some(shader_name) = &self.current_shader {
            if let Some(shader) = self.plugin.shaders.get(shader_name) {
                if let Some(input) = shader.inputs.get(param_index) {
                    let base_value = match self.plugin.get_parameter(shader_name, &input.name) {
                        Some(ShaderValue::Float(val)) => *val,
                        Some(ShaderValue::Bool(val)) => if *val { 1.0 } else { 0.0 },
                        _ => input.default.unwrap_or(0.0),
                    };

                    // Apply audio/MIDI modulation
                    self.audio_midi_system.get_parameter(&input.name, base_value)
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get number of parameters for current shader
    pub fn get_num_parameters(&self) -> usize {
        if let Some(shader_name) = &self.current_shader {
            if let Some(shader) = self.plugin.shaders.get(shader_name) {
                shader.inputs.len()
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Get parameter info
    pub fn get_parameter_info(&self, param_index: usize) -> Option<ParameterStruct> {
        if let Some(shader_name) = &self.current_shader {
            if let Some(shader) = self.plugin.shaders.get(shader_name) {
                if let Some(input) = shader.inputs.get(param_index) {
                    let mut name = [0i8; 16];
                    let param_name = input.name.as_bytes();
                    for (i, &byte) in param_name.iter().enumerate() {
                        if i < name.len() {
                            name[i] = byte as i8;
                        }
                    }

                    let (min_val, max_val, default_val) = match input.input_type {
                        crate::InputType::Float => (
                            input.min.unwrap_or(0.0),
                            input.max.unwrap_or(1.0),
                            input.default.unwrap_or(0.0),
                        ),
                        crate::InputType::Bool => (0.0, 1.0, input.default.unwrap_or(0.0)),
                        _ => (0.0, 1.0, 0.0),
                    };

                    Some(ParameterStruct {
                        name,
                        default_value: default_val,
                        min_value: min_val,
                        max_value: max_val,
                        type_: match input.input_type {
                            crate::InputType::Float => 0, // FF_TYPE_STANDARD
                            crate::InputType::Bool => 1,  // FF_TYPE_BOOLEAN
                            _ => 0,
                        },
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Set current shader
    pub fn set_current_shader(&mut self, shader_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.plugin.set_current_shader(shader_name)?;
        self.current_shader = Some(shader_name.to_string());
        Ok(())
    }

    /// Get list of available shaders
    pub fn get_available_shaders(&self) -> Vec<String> {
        self.plugin.shaders.keys().cloned().collect()
    }
}

impl bevy::prelude::Plugin for FfglPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[derive(Resource, Clone)]
        struct AudioMidiSystemResource(Arc<AudioMidiSystem>);
        app.insert_resource(AudioMidiSystemResource(self.audio_midi_system.clone()));
    }
}

/// Global plugin instance
static mut PLUGIN_INSTANCE: Option<FfglPlugin> = None;

/// Initialize the plugin (FFGL entry point)
#[no_mangle]
pub extern "C" fn plugMain() -> *mut c_void {
    unsafe {
        PLUGIN_INSTANCE = Some(FfglPlugin::new());
        if let Some(ref mut plugin) = PLUGIN_INSTANCE {
            if let Err(e) = plugin.init() {
                eprintln!("Failed to initialize plugin: {}", e);
                return ptr::null_mut();
            }
        }
        PLUGIN_INSTANCE.as_mut().unwrap() as *mut _ as *mut c_void
    }
}

/// Get plugin info (FFGL entry point)
#[no_mangle]
pub extern "C" fn getInfo() -> PluginInfoStruct {
    PluginInfoStruct::default()
}

/// Process frame (FFGL entry point)
#[no_mangle]
pub extern "C" fn processFrame(
    input: *const u8,
    output: *mut u8,
    width: c_uint,
    height: c_uint,
    time: c_float,
) {
    unsafe {
        if let Some(ref mut plugin) = PLUGIN_INSTANCE {
            let input_slice = std::slice::from_raw_parts(input, (width * height * 4) as usize);
            let output_slice = std::slice::from_raw_parts_mut(output, (width * height * 4) as usize);

            if let Err(e) = plugin.process_frame(input_slice, output_slice, width as u32, height as u32, time as f32) {
                eprintln!("Frame processing error: {}", e);
            }
        }
    }
}

/// Set parameter (FFGL entry point)
#[no_mangle]
pub extern "C" fn setParameter(index: c_int, value: c_float) {
    unsafe {
        if let Some(ref mut plugin) = PLUGIN_INSTANCE {
            let _ = plugin.set_parameter(index as usize, value as f32);
        }
    }
}

/// Get parameter (FFGL entry point)
#[no_mangle]
pub extern "C" fn getParameter(index: c_int) -> c_float {
    unsafe {
        if let Some(ref plugin) = PLUGIN_INSTANCE {
            plugin.get_parameter(index as usize) as c_float
        } else {
            0.0
        }
    }
}

/// Get number of parameters (FFGL entry point)
#[no_mangle]
pub extern "C" fn getNumParameters() -> c_int {
    unsafe {
        if let Some(ref plugin) = PLUGIN_INSTANCE {
            plugin.get_num_parameters() as c_int
        } else {
            0
        }
    }
}

/// Get parameter info (FFGL entry point)
#[no_mangle]
pub extern "C" fn getParameterInfo(index: c_int) -> ParameterStruct {
    unsafe {
        if let Some(ref plugin) = PLUGIN_INSTANCE {
            if let Some(info) = plugin.get_parameter_info(index as usize) {
                info
            } else {
                ParameterStruct {
                    name: [0; 16],
                    default_value: 0.0,
                    min_value: 0.0,
                    max_value: 1.0,
                    type_: 0,
                }
            }
        } else {
            ParameterStruct {
                name: [0; 16],
                default_value: 0.0,
                min_value: 0.0,
                max_value: 1.0,
                type_: 0,
            }
        }
    }
}
