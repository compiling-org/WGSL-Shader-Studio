//! Audio analysis and MIDI control for ISF shader parameters

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rustfft::{FftPlanner, num_complex::Complex};
use midir::{MidiInput, MidiInputConnection};

/// Audio analysis data structure
#[derive(Debug, Clone)]
pub struct AudioData {
    pub spectrum: Vec<f32>,
    pub waveform: Vec<f32>,
    pub beat: f32,
    pub bass_level: f32,
    pub mid_level: f32,
    pub treble_level: f32,
    pub volume: f32,
    pub centroid: f32,
    pub rolloff: f32,
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            spectrum: vec![0.0; 256],
            waveform: vec![0.0; 1024],
            beat: 0.0,
            bass_level: 0.0,
            mid_level: 0.0,
            treble_level: 0.0,
            volume: 0.0,
            centroid: 0.0,
            rolloff: 0.0,
        }
    }
}

/// Audio analyzer for real-time audio processing
pub struct AudioAnalyzer {
    pub current_data: Arc<Mutex<AudioData>>,
    pub fft_size: usize,
    pub sample_rate: f32,
    fft_planner: FftPlanner<f32>,
    fft_buffer: Vec<Complex<f32>>,
    window: Vec<f32>,
    previous_spectrum: Vec<f32>,
}

impl AudioAnalyzer {
    pub fn new() -> Self {
        let fft_size = 512;
        let mut fft_planner = FftPlanner::new();
        let fft = fft_planner.plan_fft_forward(fft_size);

        // Hann window for better frequency analysis
        let window: Vec<f32> = (0..fft_size)
            .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32).cos()))
            .collect();

        Self {
            current_data: Arc::new(Mutex::new(AudioData::default())),
            fft_size,
            sample_rate: 44100.0,
            fft_planner,
            fft_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            window,
            previous_spectrum: vec![0.0; fft_size / 2],
        }
    }

    /// Process audio samples and update analysis data
    pub fn process_samples(&mut self, samples: &[f32]) {
        // Compute all analysis data outside the lock to avoid borrow conflicts
        let mut spectrum = vec![0.0; 256];
        self.compute_fft(samples, &mut spectrum);

        let bass_level = self.compute_frequency_band(&spectrum, 0, 64);
        let mid_level = self.compute_frequency_band(&spectrum, 64, 128);
        let treble_level = self.compute_frequency_band(&spectrum, 128, 256);
        let centroid = self.compute_spectral_centroid(&spectrum);
        let rolloff = self.compute_spectral_rolloff(&spectrum, 0.85);
        let volume = (samples.iter().map(|x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
        let beat = self.detect_beat(&spectrum);

        // Update previous spectrum for next iteration
        self.previous_spectrum.copy_from_slice(&spectrum);

        // Update data with locking
        let mut data = self.current_data.lock().unwrap();

        // Update waveform
        data.waveform.clear();
        data.waveform.extend_from_slice(&samples[..samples.len().min(1024)]);

        // Update computed values
        data.spectrum.copy_from_slice(&spectrum);
        data.bass_level = bass_level;
        data.mid_level = mid_level;
        data.treble_level = treble_level;
        data.centroid = centroid;
        data.rolloff = rolloff;
        data.volume = volume;
        data.beat = beat;
    }

    /// Compute FFT with proper windowing
    fn compute_fft(&mut self, samples: &[f32], spectrum: &mut Vec<f32>) {
        let fft = self.fft_planner.plan_fft_forward(self.fft_size);

        // Apply window and convert to complex
        for i in 0..self.fft_size.min(samples.len()) {
            self.fft_buffer[i] = Complex::new(samples[i] * self.window[i], 0.0);
        }

        // Zero-pad if necessary
        for i in samples.len()..self.fft_size {
            self.fft_buffer[i] = Complex::new(0.0, 0.0);
        }

        // Execute FFT
        fft.process(&mut self.fft_buffer);

        // Convert to magnitude spectrum
        spectrum.clear();
        spectrum.resize(self.fft_size / 2, 0.0);

        for i in 0..self.fft_size / 2 {
            let magnitude = (self.fft_buffer[i].re * self.fft_buffer[i].re +
                           self.fft_buffer[i].im * self.fft_buffer[i].im).sqrt();
            spectrum[i] = magnitude;
        }

        // Normalize
        let max_val = spectrum.iter().cloned().fold(0.0f32, f32::max);
        if max_val > 0.0 {
            for val in spectrum.iter_mut() {
                *val /= max_val;
            }
        }
    }

    /// Compute frequency band level
    fn compute_frequency_band(&self, spectrum: &[f32], start: usize, end: usize) -> f32 {
        let end = end.min(spectrum.len());
        if start >= end {
            return 0.0;
        }

        let sum: f32 = spectrum[start..end].iter().sum();
        sum / (end - start) as f32
    }

    /// Compute spectral centroid
    fn compute_spectral_centroid(&self, spectrum: &[f32]) -> f32 {
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &magnitude) in spectrum.iter().enumerate() {
            let frequency = i as f32 * self.sample_rate / (2.0 * spectrum.len() as f32);
            numerator += frequency * magnitude;
            denominator += magnitude;
        }

        if denominator > 0.0 {
            numerator / denominator / (self.sample_rate / 2.0)
        } else {
            0.0
        }
    }

    /// Compute spectral rolloff
    fn compute_spectral_rolloff(&self, spectrum: &[f32], percentile: f32) -> f32 {
        let total_energy: f32 = spectrum.iter().sum();
        let target_energy = total_energy * percentile;
        let mut cumulative_energy = 0.0;

        for (i, &magnitude) in spectrum.iter().enumerate() {
            cumulative_energy += magnitude;
            if cumulative_energy >= target_energy {
                return i as f32 / spectrum.len() as f32;
            }
        }

        1.0
    }

    /// Beat detection using spectral flux
    fn detect_beat(&self, spectrum: &[f32]) -> f32 {
        let mut flux = 0.0;

        for (i, (&current, &previous)) in spectrum.iter().zip(&self.previous_spectrum).enumerate() {
            let diff = current - previous;
            if diff > 0.0 {
                // Rectify (only positive changes)
                flux += diff * diff;
            }
        }

        // Normalize and smooth
        (flux / spectrum.len() as f32).sqrt().min(1.0)
    }

    /// Get current audio data
    pub fn get_data(&self) -> AudioData {
        self.current_data.lock().unwrap().clone()
    }
}

/// MIDI controller for parameter mapping
pub struct MidiController {
    pub mappings: HashMap<(u8, u8), MidiMapping>, // (channel, controller) -> mapping
    pub current_values: HashMap<String, f32>,
    _connection: Option<MidiInputConnection<()>>,
}

#[derive(Debug, Clone)]
pub struct MidiMapping {
    pub parameter_name: String,
    pub channel: u8,
    pub controller: u8,
    pub min_value: f32,
    pub max_value: f32,
    pub smoothing: f32, // 0.0 = no smoothing, 1.0 = heavy smoothing
}

impl MidiController {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            current_values: HashMap::new(),
            _connection: None,
        }
    }

    /// Initialize MIDI input connection
    pub fn init_midi(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let midi_in = MidiInput::new("ISF Shader MIDI Input")?;
        let in_ports = midi_in.ports();

        if in_ports.is_empty() {
            println!("No MIDI input ports available");
            return Ok(());
        }

        // Connect to first available port
        let in_port = &in_ports[0];
        let port_name = midi_in.port_name(in_port)?;

        println!("Connecting to MIDI port: {}", port_name);

        let mappings = self.mappings.clone();
        let current_values = Arc::new(Mutex::new(self.current_values.clone()));

        let connection = midi_in.connect(
            in_port,
            "isf-shader-input",
            move |stamp, message, _| {
                Self::process_midi_message_static(message, &mappings, &current_values);
            },
            (),
        )?;

        self._connection = Some(connection);
        Ok(())
    }

    /// Process MIDI message (static version for closure)
    fn process_midi_message_static(
        message: &[u8],
        mappings: &HashMap<(u8, u8), MidiMapping>,
        current_values: &Arc<Mutex<HashMap<String, f32>>>,
    ) {
        if message.len() >= 3 {
            let status = message[0];
            let data1 = message[1];
            let data2 = message[2];

            // Check for control change messages (0xB0-0xBF)
            if (0xB0..=0xBF).contains(&status) {
                let channel = status & 0x0F;
                let controller = data1;
                let value = data2;

                if let Some(mapping) = mappings.get(&(channel, controller)) {
                    let normalized_value = value as f32 / 127.0;
                    let mapped_value = mapping.min_value +
                        (mapping.max_value - mapping.min_value) * normalized_value;

                    let mut values = current_values.lock().unwrap();
                    values.insert(mapping.parameter_name.clone(), mapped_value);
                }
            }
        }
    }

    /// Add a MIDI mapping
    pub fn add_mapping(&mut self, mapping: MidiMapping) {
        self.mappings.insert((mapping.channel, mapping.controller), mapping);
    }

    /// Process MIDI message (instance method)
    pub fn process_midi_message(&mut self, channel: u8, controller: u8, value: u8) {
        if let Some(mapping) = self.mappings.get(&(channel, controller)) {
            let normalized_value = value as f32 / 127.0;
            let mapped_value = mapping.min_value +
                (mapping.max_value - mapping.min_value) * normalized_value;
            self.current_values.insert(mapping.parameter_name.clone(), mapped_value);
        }
    }

    /// Get parameter value
    pub fn get_parameter(&self, name: &str) -> f32 {
        *self.current_values.get(name).unwrap_or(&0.0)
    }

    /// Create default mappings for common ISF parameters
    pub fn create_default_mappings(&mut self) {
        let mappings = vec![
            MidiMapping {
                parameter_name: "Speed".to_string(),
                channel: 0,
                controller: 1, // Mod wheel
                min_value: 0.0,
                max_value: 5.0,
                smoothing: 0.1,
            },
            MidiMapping {
                parameter_name: "Zoom".to_string(),
                channel: 0,
                controller: 2,
                min_value: 0.2,
                max_value: 5.0,
                smoothing: 0.1,
            },
            MidiMapping {
                parameter_name: "Brightness".to_string(),
                channel: 0,
                controller: 3,
                min_value: 0.0,
                max_value: 20.0,
                smoothing: 0.2,
            },
            MidiMapping {
                parameter_name: "Contrast".to_string(),
                channel: 0,
                controller: 4,
                min_value: 0.0,
                max_value: 3.0,
                smoothing: 0.2,
            },
            MidiMapping {
                parameter_name: "Glow".to_string(),
                channel: 0,
                controller: 5,
                min_value: 0.0,
                max_value: 5.0,
                smoothing: 0.1,
            },
            MidiMapping {
                parameter_name: "ChaosIntensity".to_string(),
                channel: 0,
                controller: 6,
                min_value: 0.0,
                max_value: 1.0,
                smoothing: 0.1,
            },
            MidiMapping {
                parameter_name: "ColorPaletteMode".to_string(),
                channel: 0,
                controller: 7,
                min_value: 0.0,
                max_value: 10.0,
                smoothing: 0.0,
            },
            MidiMapping {
                parameter_name: "TransformMode".to_string(),
                channel: 0,
                controller: 8,
                min_value: 0.0,
                max_value: 5.0,
                smoothing: 0.0,
            },
        ];

        for mapping in mappings {
            self.add_mapping(mapping);
        }
    }

    /// Get all available mappings
    pub fn get_mappings(&self) -> &HashMap<(u8, u8), MidiMapping> {
        &self.mappings
    }
}

/// Combined audio and MIDI system
pub struct AudioMidiSystem {
    pub audio_analyzer: AudioAnalyzer,
    pub midi_controller: MidiController,
    pub audio_reactivity_enabled: bool,
    pub midi_enabled: bool,
}

impl AudioMidiSystem {
    pub fn new() -> Self {
        let mut midi_controller = MidiController::new();
        midi_controller.create_default_mappings();

        // Try to initialize MIDI (ignore errors in standalone mode)
        let _ = midi_controller.init_midi();

        Self {
            audio_analyzer: AudioAnalyzer::new(),
            midi_controller,
            audio_reactivity_enabled: true,
            midi_enabled: true,
        }
    }

    /// Update system with new audio samples
    pub fn update_audio(&mut self, samples: &[f32]) {
        self.audio_analyzer.process_samples(samples);
    }

    /// Process MIDI message
    pub fn process_midi(&mut self, channel: u8, controller: u8, value: u8) {
        if self.midi_enabled {
            self.midi_controller.process_midi_message(channel, controller, value);
        }
    }

    /// Get combined parameter value (MIDI + audio reactivity)
    pub fn get_parameter(&self, name: &str, base_value: f32) -> f32 {
        let midi_value = if self.midi_enabled {
            self.midi_controller.get_parameter(name)
        } else {
            0.0
        };

        if !self.audio_reactivity_enabled {
            return base_value + midi_value;
        }

        let audio_data = self.audio_analyzer.get_data();

        // Audio-reactive parameter modulation
        match name {
            "Speed" => base_value + midi_value + audio_data.beat * 2.0,
            "Brightness" => base_value + midi_value + audio_data.volume * 10.0,
            "Glow" => base_value + midi_value + audio_data.treble_level * 3.0,
            "ChaosIntensity" => base_value + midi_value + audio_data.centroid * 0.5,
            "Zoom" => base_value + midi_value * (1.0 + audio_data.bass_level * 0.5),
            "Contrast" => base_value + midi_value + audio_data.mid_level * 2.0,
            _ => base_value + midi_value,
        }
    }

    /// Get current audio data
    pub fn get_audio_data(&self) -> AudioData {
        self.audio_analyzer.get_data()
    }

    /// Get MIDI controller reference
    pub fn get_midi_controller(&self) -> &MidiController {
        &self.midi_controller
    }

    /// Toggle audio reactivity
    pub fn toggle_audio_reactivity(&mut self) {
        self.audio_reactivity_enabled = !self.audio_reactivity_enabled;
    }

    /// Toggle MIDI control
    pub fn toggle_midi(&mut self) {
        self.midi_enabled = !self.midi_enabled;
    }
}