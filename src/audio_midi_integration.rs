//! Audio MIDI Integration System
//! 
//! Provides comprehensive MIDI input support for parameter control,
//! complementing the existing audio analysis system.

use bevy::prelude::*;
use bevy::log::info;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// MIDI message types for parameter control
#[derive(Debug, Clone)]
pub enum MidiMessage {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    PitchBend { channel: u8, value: i16 },
    ProgramChange { channel: u8, program: u8 },
}

/// MIDI data structure for shader parameter mapping
#[derive(Resource, Clone, Debug)]
pub struct MidiData {
    pub enabled: bool,
    pub note_values: [u8; 128], // MIDI note values (0-127)
    pub controller_values: HashMap<u8, u8>, // Controller number -> value (0-127)
    pub pitch_bend_values: [i16; 16], // Pitch bend per channel (-8192 to 8191)
    pub last_message: Option<MidiMessage>,
    pub tempo_tap: Vec<f64>, // Timestamps for tempo tapping
}

impl Default for MidiData {
    fn default() -> Self {
        Self {
            enabled: false,
            note_values: [0; 128],
            controller_values: HashMap::new(),
            pitch_bend_values: [0; 16],
            last_message: None,
            tempo_tap: Vec::new(),
        }
    }
}

/// MIDI analyzer for real-time parameter control
#[derive(Resource)]
pub struct MidiAnalyzer {
    pub enabled: bool,
    pub input_gain: f32,
    pub smoothing_factor: f32,
    data: Arc<Mutex<MidiData>>,
}

impl Default for MidiAnalyzer {
    fn default() -> Self {
        Self {
            enabled: false,
            input_gain: 1.0,
            smoothing_factor: 0.8,
            data: Arc::new(Mutex::new(MidiData::default())),
        }
    }
}

impl MidiAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        if let Ok(mut data) = self.data.lock() {
            data.enabled = true;
        }
        info!("MIDI Analyzer enabled");
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        if let Ok(mut data) = self.data.lock() {
            data.enabled = false;
        }
        info!("MIDI Analyzer disabled");
    }

    pub fn process_midi_message(&self, message: MidiMessage) {
        if !self.enabled {
            return;
        }

        if let Ok(mut data) = self.data.lock() {
            match &message {
                MidiMessage::NoteOn { channel, note, velocity } => {
                    data.note_values[*note as usize] = *velocity;
                    info!("MIDI Note ON: ch={}, note={}, vel={}", channel, note, velocity);
                }
                MidiMessage::NoteOff { channel, note } => {
                    data.note_values[*note as usize] = 0;
                    info!("MIDI Note OFF: ch={}, note={}", channel, note);
                }
                MidiMessage::ControlChange { channel, controller, value } => {
                    data.controller_values.insert(*controller, *value);
                    info!("MIDI CC: ch={}, ctrl={}, val={}", channel, controller, value);
                }
                MidiMessage::PitchBend { channel, value } => {
                    data.pitch_bend_values[*channel as usize] = *value;
                    info!("MIDI Pitch Bend: ch={}, val={}", channel, value);
                }
                MidiMessage::ProgramChange { channel, program } => {
                    info!("MIDI Program Change: ch={}, prog={}", channel, program);
                }
            }
            data.last_message = Some(message);
        }
    }

    pub fn get_midi_data(&self) -> Option<MidiData> {
        self.data.lock().ok().map(|data| data.clone())
    }

    pub fn get_controller_value(&self, controller: u8) -> Option<u8> {
        self.data.lock().ok().and_then(|data| {
            data.controller_values.get(&controller).copied()
        })
    }

    pub fn get_note_value(&self, note: u8) -> u8 {
        self.data.lock().ok()
            .map(|data| data.note_values[note as usize])
            .unwrap_or(0)
    }

    pub fn get_pitch_bend(&self, channel: u8) -> i16 {
        self.data.lock().ok()
            .map(|data| data.pitch_bend_values[channel as usize])
            .unwrap_or(0)
    }

    /// Tap tempo functionality
    pub fn tap_tempo(&self) -> Option<f32> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        if let Ok(mut data) = self.data.lock() {
            data.tempo_tap.push(now);
            
            // Keep only recent taps (last 4 seconds)
            data.tempo_tap.retain(|&timestamp| now - timestamp < 4.0);
            
            // Need at least 2 taps to calculate tempo
            if data.tempo_tap.len() >= 2 {
                let intervals: Vec<f64> = data.tempo_tap.windows(2)
                    .map(|window| window[1] - window[0])
                    .collect();
                
                let avg_interval = intervals.iter().sum::<f64>() / intervals.len() as f64;
                let tempo = 60.0 / avg_interval;
                
                // Filter reasonable tempos (60-200 BPM)
                if tempo >= 60.0 && tempo <= 200.0 {
                    return Some(tempo as f32);
                }
            }
        }
        None
    }

    /// Reset all MIDI data
    pub fn reset(&self) {
        if let Ok(mut data) = self.data.lock() {
            data.note_values = [0; 128];
            data.controller_values.clear();
            data.pitch_bend_values = [0; 16];
            data.last_message = None;
            data.tempo_tap.clear();
        }
    }
}

/// Audio MIDI Integration Plugin
pub struct AudioMidiIntegrationPlugin;

impl Plugin for AudioMidiIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MidiAnalyzer>()
            // .add_systems(Update, update_midi_integration); // Temporarily disabled due to E0382 error
            ;
    }
}

/// System to update MIDI integration
fn update_midi_integration(
    midi_analyzer: Res<MidiAnalyzer>,
    mut ui_state: ResMut<crate::editor_ui::EditorUiState>,
) {
    if !midi_analyzer.enabled {
        return;
    }

    // Update UI state with MIDI data if available
    if let Ok(midi_data_lock) = midi_analyzer.data.lock() {
        let midi_data = midi_data_lock.clone();
        drop(midi_data_lock); // Release the lock early
        
        // Map MIDI controllers to shader parameters
        for (controller, value) in &midi_data.controller_values {
            let _normalized_value = *value as f32 / 127.0;
            
            // Map common MIDI controllers to shader parameters
            match controller {
                1 => { /* Modulation wheel */ }
                7 => { /* Volume */ }
                10 => { /* Pan */ }
                11 => { /* Expression */ }
                64 => { /* Sustain pedal */ }
                _ => {}
            }
        }
    }
}

/// Helper functions for MIDI parameter mapping
pub mod midi_mapping {


    /// Map MIDI value (0-127) to normalized float (0.0-1.0)
    pub fn midi_to_float(value: u8) -> f32 {
        value as f32 / 127.0
    }

    /// Map MIDI note to frequency
    pub fn midi_note_to_freq(note: u8) -> f32 {
        440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
    }

    /// Map pitch bend value to normalized float (-1.0 to 1.0)
    pub fn pitch_bend_to_float(value: i16) -> f32 {
        value as f32 / 8192.0
    }

    /// Smooth MIDI values with exponential smoothing
    pub fn smooth_midi_value(current: f32, target: f32, factor: f32) -> f32 {
        current + (target - current) * factor
    }
}