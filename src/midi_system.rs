//! Comprehensive MIDI System for WGSL Shader Studio
//!
//! Provides MIDI device detection, connection management, parameter mapping,
//! and real-time control with learn functionality.

use bevy::prelude::*;
use midir::{MidiInput, MidiInputConnection, Ignore};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::error::Error;
use serde::{Serialize, Deserialize};
use crate::audio_midi_integration::{MidiMessage, MidiAnalyzer};

/// MIDI device information
#[derive(Debug, Clone)]
pub struct MidiDevice {
    pub name: String,
    pub port_name: String,
    pub connected: bool,
}

/// MIDI mapping configuration for parameter control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiMapping {
    pub parameter_name: String,
    pub midi_type: MidiMessageType,
    pub channel: u8,
    pub number: u8, // Note number or controller number
    pub min_value: f32,
    pub max_value: f32,
    pub curve: MidiCurve,
    pub invert: bool,
    pub smoothing: f32,
}

/// Type of MIDI message for mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MidiMessageType {
    Note,
    ControlChange,
    PitchBend,
    Aftertouch,
}

/// MIDI response curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MidiCurve {
    Linear,
    Exponential(f32), // curvature factor
    Logarithmic(f32), // curvature factor
    SCurve(f32), // S-curve factor
}

/// MIDI learn state for automatic mapping
#[derive(Debug, Clone)]
pub struct MidiLearnState {
    pub active: bool,
    pub target_parameter: Option<String>,
    pub learn_timeout: f32,
    pub last_message: Option<(MidiMessageType, u8, u8)>, // type, channel, number
}

/// Main MIDI system resource
#[derive(Resource)]
pub struct MidiSystem {
    pub devices: Vec<MidiDevice>,
    pub active_connections: HashMap<String, Arc<Mutex<MidiInputConnection<()>>>>,
    pub mappings: HashMap<String, MidiMapping>,
    pub learn_state: MidiLearnState,
    pub enabled: bool,
}

impl Default for MidiSystem {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
            active_connections: HashMap::new(),
            mappings: HashMap::new(),
            learn_state: MidiLearnState {
                active: false,
                target_parameter: None,
                learn_timeout: 0.0,
                last_message: None,
            },
            enabled: false,
        }
    }
}

impl MidiSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan for available MIDI input devices
    pub fn scan_devices(&mut self) -> Result<Vec<MidiDevice>, Box<dyn Error>> {
        let midi_input = MidiInput::new("WGSL Shader Studio MIDI Scanner")?;
        let mut devices = Vec::new();

        for (i, port) in midi_input.ports().iter().enumerate() {
            if let Ok(name) = midi_input.port_name(port) {
                devices.push(MidiDevice {
                    name: name.clone(),
                    port_name: format!("{} (Port {})", name, i),
                    connected: false,
                });
            }
        }

        self.devices = devices.clone();
        Ok(devices)
    }

    /// Connect to a MIDI device
    pub fn connect_device(&mut self, device_index: usize) -> Result<(), Box<dyn Error>> {
        if device_index >= self.devices.len() {
            return Err("Invalid device index".into());
        }

        let device_name = self.devices[device_index].name.clone();
        
        // Disconnect if already connected
        if self.active_connections.contains_key(&device_name) {
            self.disconnect_device(device_index)?;
        }

        let midi_input = MidiInput::new("WGSL Shader Studio MIDI Input")?;
        let ports = midi_input.ports();
        
        if device_index >= ports.len() {
            return Err("Device port not available".into());
        }

        let port = &ports[device_index];
        let port_name = midi_input.port_name(port)?;
        
        info!("Connecting to MIDI device: {}", port_name);

        // Set up MIDI message handler
        let device_name_clone = device_name.clone();
        let learn_state = Arc::new(Mutex::new(self.learn_state.clone()));
        let mappings = Arc::new(Mutex::new(self.mappings.clone()));

        let connection = midi_input.connect(
            port,
            &port_name,
            move |_timestamp, message, _| {
                Self::handle_midi_message(&device_name_clone, message, &learn_state, &mappings);
            },
            (),
        )?;

        // Store the connection
        self.active_connections.insert(
            device_name.clone(),
            Arc::new(Mutex::new(connection)),
        );

        self.devices[device_index].connected = true;
        info!("Successfully connected to MIDI device: {}", port_name);

        Ok(())
    }

    /// Disconnect from a MIDI device
    pub fn disconnect_device(&mut self, device_index: usize) -> Result<(), Box<dyn Error>> {
        if device_index >= self.devices.len() {
            return Err("Invalid device index".into());
        }

        let device_name = self.devices[device_index].name.clone();
        
        if let Some(_connection) = self.active_connections.remove(&device_name) {
            self.devices[device_index].connected = false;
            info!("Disconnected from MIDI device: {}", device_name);
        }

        Ok(())
    }

    /// Handle incoming MIDI messages
    fn handle_midi_message(
        _device_name: &str,
        message: &[u8],
        learn_state: &Arc<Mutex<MidiLearnState>>,
        _mappings: &Arc<Mutex<HashMap<String, MidiMapping>>>,
    ) {
        if message.is_empty() {
            return;
        }

        let status = message[0];
        let channel = (status & 0x0F) + 1; // 1-16
        let message_type = status >> 4;

        match message_type {
            0x8 => { // Note Off
                if message.len() >= 3 {
                    let note = message[1];
                    let velocity = message[2];
                    
                    let _midi_message = MidiMessage::NoteOff { channel, note };
                    
                    // Check for learn mode
                    if let Ok(mut learn) = learn_state.lock() {
                        if learn.active && learn.target_parameter.is_some() {
                            learn.last_message = Some((MidiMessageType::Note, channel, note));
                            learn.active = false;
                            info!("MIDI Learn: Note {} on channel {} mapped to {} (velocity: {})", 
                                  note, channel, learn.target_parameter.as_ref().unwrap(), velocity);
                        }
                    }
                }
            }
            0x9 => { // Note On
                if message.len() >= 3 {
                    let note = message[1];
                    let velocity = message[2];
                    
                    if velocity > 0 {
                        let _midi_message = MidiMessage::NoteOn { channel, note, velocity };
                        
                        // Check for learn mode
                        if let Ok(mut learn) = learn_state.lock() {
                            if learn.active && learn.target_parameter.is_some() {
                                learn.last_message = Some((MidiMessageType::Note, channel, note));
                                learn.active = false;
                                info!("MIDI Learn: Note {} on channel {} mapped to {} (velocity: {})", 
                                      note, channel, learn.target_parameter.as_ref().unwrap(), velocity);
                            }
                        }
                    }
                }
            }
            0xB => { // Control Change
                if message.len() >= 3 {
                    let controller = message[1];
                    let value = message[2];
                    
                    let _midi_message = MidiMessage::ControlChange { channel, controller, value };
                    
                    // Check for learn mode
                    if let Ok(mut learn) = learn_state.lock() {
                        if learn.active && learn.target_parameter.is_some() {
                            learn.last_message = Some((MidiMessageType::ControlChange, channel, controller));
                            learn.active = false;
                            info!("MIDI Learn: CC {} on channel {} mapped to {} (value: {})", 
                                  controller, channel, learn.target_parameter.as_ref().unwrap(), value);
                        }
                    }
                }
            }
            0xC => { // Program Change
                if message.len() >= 2 {
                    let program = message[1];
                    let _midi_message = MidiMessage::ProgramChange { channel, program };
                }
            }
            0xE => { // Pitch Bend
                if message.len() >= 3 {
                    let lsb = message[1] as i16;
                    let msb = message[2] as i16;
                    let value = ((msb << 7) | lsb) as i16 - 8192; // Center at 0
                    
                    let _midi_message = MidiMessage::PitchBend { channel, value };
                    
                    // Check for learn mode
                    if let Ok(mut learn) = learn_state.lock() {
                        if learn.active && learn.target_parameter.is_some() {
                            learn.last_message = Some((MidiMessageType::PitchBend, channel, 0));
                            learn.active = false;
                            info!("MIDI Learn: Pitch Bend on channel {} mapped to {} (value: {})", 
                                  channel, learn.target_parameter.as_ref().unwrap(), value);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Start MIDI learn mode for a parameter
    pub fn start_midi_learn(&mut self, parameter_name: &str) {
        self.learn_state.active = true;
        self.learn_state.target_parameter = Some(parameter_name.to_string());
        self.learn_state.learn_timeout = 5.0; // 5 second timeout
        info!("MIDI Learn started for parameter: {}", parameter_name);
    }

    /// Stop MIDI learn mode
    pub fn stop_midi_learn(&mut self) {
        self.learn_state.active = false;
        self.learn_state.target_parameter = None;
        self.learn_state.learn_timeout = 0.0;
        info!("MIDI Learn stopped");
    }

    /// Add a MIDI mapping
    pub fn add_mapping(&mut self, mapping: MidiMapping) {
        let parameter_name = mapping.parameter_name.clone();
        self.mappings.insert(parameter_name.clone(), mapping);
        info!("Added MIDI mapping for parameter: {}", parameter_name);
    }

    /// Remove a MIDI mapping
    pub fn remove_mapping(&mut self, parameter_name: &str) {
        if self.mappings.remove(parameter_name).is_some() {
            info!("Removed MIDI mapping for parameter: {}", parameter_name);
        }
    }

    /// Get MIDI mapping for a parameter
    pub fn get_mapping(&self, parameter_name: &str) -> Option<&MidiMapping> {
        self.mappings.get(parameter_name)
    }

    /// Apply MIDI mappings to get parameter values
    pub fn apply_mappings(&self, midi_analyzer: &MidiAnalyzer) -> HashMap<String, f32> {
        let mut parameter_values = HashMap::new();

        for (parameter_name, mapping) in &self.mappings {
            if let Some(value) = self.apply_mapping(mapping, midi_analyzer) {
                parameter_values.insert(parameter_name.clone(), value);
            }
        }

        parameter_values
    }

    /// Apply a single mapping
    fn apply_mapping(&self, mapping: &MidiMapping, midi_analyzer: &MidiAnalyzer) -> Option<f32> {
        match mapping.midi_type {
            MidiMessageType::ControlChange => {
                if let Some(value) = midi_analyzer.get_controller_value(mapping.number) {
                    Some(self.map_midi_value(value as f32, mapping))
                } else {
                    None
                }
            }
            MidiMessageType::Note => {
                let value = midi_analyzer.get_note_value(mapping.number);
                if value > 0 {
                    Some(self.map_midi_value(value as f32, mapping))
                } else {
                    None
                }
            }
            MidiMessageType::PitchBend => {
                let value = midi_analyzer.get_pitch_bend(mapping.channel);
                let normalized = (value as f32 + 8192.0) / 16383.0; // Normalize to 0-1
                Some(self.apply_curve(normalized, &mapping.curve) * 
                     (mapping.max_value - mapping.min_value) + mapping.min_value)
            }
            MidiMessageType::Aftertouch => {
                // For now, use note values as aftertouch
                let value = midi_analyzer.get_note_value(mapping.number);
                Some(self.map_midi_value(value as f32, mapping))
            }
        }
    }

    /// Map MIDI value (0-127) to parameter range with curve
    fn map_midi_value(&self, midi_value: f32, mapping: &MidiMapping) -> f32 {
        let normalized = midi_value / 127.0;
        let curved = self.apply_curve(normalized, &mapping.curve);
        
        let mut result = curved * (mapping.max_value - mapping.min_value) + mapping.min_value;
        
        if mapping.invert {
            result = mapping.max_value - (result - mapping.min_value);
        }
        
        result
    }

    /// Apply response curve to normalized value
    fn apply_curve(&self, value: f32, curve: &MidiCurve) -> f32 {
        match curve {
            MidiCurve::Linear => value,
            MidiCurve::Exponential(factor) => value.powf(*factor),
            MidiCurve::Logarithmic(factor) => (1.0 + value * (*factor - 1.0)).ln() / (*factor).ln(),
            MidiCurve::SCurve(factor) => {
                let x = value * std::f32::consts::PI * *factor;
                (x.sin() + 1.0) / 2.0
            }
        }
    }

    /// Enable/disable MIDI system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("MIDI System enabled");
            // Rescan devices when enabling
            let _ = self.scan_devices();
        } else {
            info!("MIDI System disabled");
            // Disconnect all devices when disabling
            for i in 0..self.devices.len() {
                let _ = self.disconnect_device(i);
            }
        }
    }
}

/// MIDI System Plugin
pub struct MidiSystemPlugin;

impl Plugin for MidiSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MidiSystem>()
            .add_systems(Update, update_midi_system)
            .add_systems(Update, update_midi_learn);
    }
}

/// System to update MIDI system
fn update_midi_system(
    mut midi_system: ResMut<MidiSystem>,
    midi_analyzer: Res<MidiAnalyzer>,
    mut ui_state: ResMut<crate::editor_ui::EditorUiState>,
) {
    if !midi_system.enabled {
        return;
    }

    // Apply MIDI mappings to shader parameters
    let parameter_values = midi_system.apply_mappings(&midi_analyzer);
    
    for (parameter_name, value) in parameter_values {
        ui_state.set_parameter_value(&parameter_name, value);
    }
}

/// System to handle MIDI learn timeout
fn update_midi_learn(
    mut midi_system: ResMut<MidiSystem>,
    time: Res<Time>,
) {
    if midi_system.learn_state.active && midi_system.learn_state.learn_timeout > 0.0 {
        midi_system.learn_state.learn_timeout -= time.delta_secs();
        
        if midi_system.learn_state.learn_timeout <= 0.0 {
            midi_system.stop_midi_learn();
            warn!("MIDI Learn timed out");
        }
    }
}
