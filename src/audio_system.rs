use bevy::prelude::*;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rustfft::{FftPlanner, num_complex::Complex};

#[derive(Resource, Clone, Debug)]
pub struct AudioData {
    pub volume: f32,
    pub bass_level: f32,
    pub mid_level: f32,
    pub treble_level: f32,
    pub beat_detected: bool,
    pub beat_intensity: f32,
    pub tempo: f32,
    pub waveform: Vec<f32>,
    pub frequencies: Vec<f32>,
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            volume: 0.0,
            bass_level: 0.0,
            mid_level: 0.0,
            treble_level: 0.0,
            beat_detected: false,
            beat_intensity: 0.0,
            tempo: 120.0,
            waveform: vec![0.0; 512],
            frequencies: vec![0.0; 256],
        }
    }
}

#[derive(Resource)]
pub struct AudioAnalyzer {
    pub enabled: bool,
    pub gain: f32,
    data: Arc<Mutex<AudioData>>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    last_update: Instant,
}

impl Default for AudioAnalyzer {
    fn default() -> Self {
        Self {
            enabled: false,
            gain: 1.0,
            data: Arc::new(Mutex::new(AudioData::default())),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 44100,
            last_update: Instant::now(),
        }
    }
}

impl AudioAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_audio_capture(&mut self) {
        if self.enabled {
            return;
        }

        self.enabled = true;
        let data = Arc::clone(&self.data);
        let audio_buffer = Arc::clone(&self.audio_buffer);

        // Start audio capture in a separate thread
        std::thread::spawn(move || {
            Self::audio_capture_thread(data, audio_buffer);
        });
    }

    pub fn stop_audio_capture(&mut self) {
        self.enabled = false;
    }

    pub fn get_audio_data(&self) -> AudioData {
        self.data.lock().unwrap().clone()
    }

    pub fn process_audio_frame(&mut self) {
        if !self.enabled {
            return;
        }

        // Get audio samples from buffer
        let audio_samples = {
            let mut buffer = self.audio_buffer.lock().unwrap();
            if buffer.is_empty() {
                return;
            }
            std::mem::take(&mut *buffer)
        };

        if !audio_samples.is_empty() {
            self.analyze_audio(&audio_samples);
        }

        self.last_update = Instant::now();
    }

    fn audio_capture_thread(data: Arc<Mutex<AudioData>>, audio_buffer: Arc<Mutex<Vec<f32>>>) {
        // Synthetic audio generation for testing
        let sample_rate: f32 = 44100.0;
        let buffer_size = 1024;
        let mut phase = 0.0f32;

        loop {
            let mut samples = Vec::with_capacity(buffer_size);
            
            // Generate synthetic audio for testing
            for _ in 0..buffer_size {
                let sample = (phase * 0.1).sin() * 0.5 + (phase * 0.3).sin() * 0.3;
                samples.push(sample);
                phase += 1.0;
            }

            // Add samples to buffer
            {
                let mut buffer = audio_buffer.lock().unwrap();
                buffer.extend(samples);
                // Keep buffer size reasonable
                if buffer.len() > 8192 {
                    let drain_to = buffer.len() - 4096;
                    buffer.drain(..drain_to);
                }
            }

            std::thread::sleep(Duration::from_millis((buffer_size * 1000) as u64 / sample_rate as u64));
        }
    }

    fn analyze_audio(&mut self, samples: &[f32]) {
        let mut data = self.data.lock().unwrap();
        
        // Calculate volume
        let sum: f32 = samples.iter().map(|&x| x * x).sum();
        data.volume = (sum / samples.len() as f32).sqrt() * self.gain;

        // Simple frequency analysis using FFT
        let fft_size = 512;
        if samples.len() >= fft_size {
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(fft_size);
            
            let mut fft_buffer: Vec<Complex<f32>> = samples[..fft_size]
                .iter()
                .map(|&x| Complex::new(x, 0.0))
                .collect();

            // Apply window function
            for (i, sample) in fft_buffer.iter_mut().enumerate() {
                let window = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos();
                sample.re *= window;
            }

            fft.process(&mut fft_buffer);

            // Extract frequency bands
            data.frequencies.clear();
            for i in 0..(fft_size / 2) {
                let magnitude = fft_buffer[i].norm();
                data.frequencies.push(magnitude);
            }

            // Calculate frequency bands
            let bass_end = fft_size / 8;
            let mid_end = fft_size / 2;
            
            data.bass_level = data.frequencies[..bass_end].iter().sum::<f32>() / bass_end as f32;
            data.mid_level = data.frequencies[bass_end..mid_end].iter().sum::<f32>() / (mid_end - bass_end) as f32;
            data.treble_level = data.frequencies[mid_end..].iter().sum::<f32>() / (fft_size / 2 - mid_end) as f32;
        }

        // Update waveform
        data.waveform.clear();
        let waveform_samples = samples.len().min(512);
        for i in 0..waveform_samples {
            data.waveform.push(samples[i] * self.gain);
        }

        // Simple beat detection
        let current_time = Instant::now();
        let time_since_last_beat = current_time.duration_since(self.last_update).as_secs_f32();
        
        if data.volume > 0.3 && time_since_last_beat > 0.2 {
            data.beat_detected = true;
            data.beat_intensity = data.volume;
            data.tempo = 60.0 / time_since_last_beat; // Estimate BPM
        } else {
            data.beat_detected = false;
            data.beat_intensity *= 0.95; // Decay
        }
    }
}

/// Audio MIDI system for comprehensive audio analysis
#[derive(Clone)]
pub struct AudioMidiSystem {
    pub audio_analyzer: Arc<Mutex<AudioAnalyzer>>,
}

impl AudioMidiSystem {
    pub fn new() -> Self {
        Self {
            audio_analyzer: Arc::new(Mutex::new(AudioAnalyzer::new())),
        }
    }

    pub fn start_audio_analysis(&self) {
        if let Ok(mut analyzer) = self.audio_analyzer.lock() {
            analyzer.start_audio_capture();
        }
    }

    pub fn stop_audio_analysis(&self) {
        if let Ok(mut analyzer) = self.audio_analyzer.lock() {
            analyzer.stop_audio_capture();
        }
    }

    pub fn get_parameter(&self, name: &str, base_value: f32) -> f32 {
        // Get current audio data and apply modulation based on parameter name
        if let Ok(analyzer) = self.audio_analyzer.lock() {
            let audio_data = analyzer.get_audio_data();
            
            match name.to_lowercase().as_str() {
                name if name.contains("volume") || name.contains("amp") => {
                    base_value * (1.0 + audio_data.volume * 0.5)
                }
                name if name.contains("bass") || name.contains("low") => {
                    base_value * (1.0 + audio_data.bass_level * 0.8)
                }
                name if name.contains("mid") || name.contains("midrange") => {
                    base_value * (1.0 + audio_data.mid_level * 0.8)
                }
                name if name.contains("treble") || name.contains("high") => {
                    base_value * (1.0 + audio_data.treble_level * 0.8)
                }
                name if name.contains("beat") => {
                    if audio_data.beat_detected {
                        base_value * (1.0 + audio_data.beat_intensity)
                    } else {
                        base_value
                    }
                }
                _ => base_value, // No modulation for unknown parameters
            }
        } else {
            base_value
        }
    }
}

impl Default for AudioMidiSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Plugin for audio system
pub struct AudioAnalysisPlugin;

impl Plugin for AudioAnalysisPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioAnalyzer::new())
            .add_systems(Update, update_audio_analysis);
    }
}

fn update_audio_analysis(mut audio_analyzer: ResMut<AudioAnalyzer>) {
    audio_analyzer.process_audio_frame();
}