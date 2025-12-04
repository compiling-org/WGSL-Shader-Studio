use bevy::prelude::*;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rustfft::{FftPlanner, num_complex::Complex};

/// Enhanced audio plugin with advanced analysis features
pub struct EnhancedAudioPlugin;

impl Plugin for EnhancedAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnhancedAudioConfig>()
            .add_systems(Update, update_enhanced_audio_analysis);
    }
}

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
            
            // Log audio metrics for debugging (every 60 frames ~1 second at 60 FPS)
            static mut FRAME_COUNT: u32 = 0;
            unsafe {
                FRAME_COUNT += 1;
                if FRAME_COUNT % 60 == 0 {
                    let data = self.data.lock().unwrap();
                    println!("🎵 Audio Analysis - Volume: {:.3}, Bass: {:.3}, Mid: {:.3}, Treble: {:.3}, Beat: {}, BPM: {:.1}", 
                        data.volume, data.bass_level, data.mid_level, data.treble_level, 
                        if data.beat_detected { "✓" } else { "✗" }, data.tempo);
                }
            }
        }

        self.last_update = Instant::now();
    }

    fn audio_capture_thread(data: Arc<Mutex<AudioData>>, audio_buffer: Arc<Mutex<Vec<f32>>>) {
        // Enhanced synthetic audio generation for testing shader audio reactive features
        let sample_rate: f32 = 44100.0;
        let buffer_size = 1024;
        let mut phase = 0.0f32;
        let mut bass_phase = 0.0f32;
        let mut mid_phase = 0.0f32;
        let mut treble_phase = 0.0f32;
        let mut beat_phase = 0.0f32;

        loop {
            let mut samples = Vec::with_capacity(buffer_size);
            
            // Generate multi-frequency synthetic audio for comprehensive testing
            for i in 0..buffer_size {
                // Bass frequency (80Hz)
                let bass_freq = 80.0;
                let bass_sample = (bass_phase * 2.0 * std::f32::consts::PI / sample_rate * bass_freq).sin() * 0.4;
                
                // Mid frequency (800Hz) 
                let mid_freq = 800.0;
                let mid_sample = (mid_phase * 2.0 * std::f32::consts::PI / sample_rate * mid_freq).sin() * 0.3;
                
                // Treble frequency (4000Hz)
                let treble_freq = 4000.0;
                let treble_sample = (treble_phase * 2.0 * std::f32::consts::PI / sample_rate * treble_freq).sin() * 0.2;
                
                // Beat pattern (2Hz = 120 BPM)
                let beat_freq = 2.0;
                let beat_envelope = (beat_phase * 2.0 * std::f32::consts::PI / sample_rate * beat_freq).sin() * 0.5 + 0.5;
                
                // Combine all frequencies with beat modulation
                let mut sample = bass_sample + mid_sample + treble_sample;
                sample *= (0.7 + beat_envelope * 0.3); // Beat modulation
                
                // Add some noise for realism
                let noise = (rand::random::<f32>() - 0.5) * 0.05;
                sample += noise;
                
                samples.push(sample.clamp(-1.0, 1.0));
                
                // Update phases
                phase += 1.0;
                bass_phase += 1.0;
                mid_phase += 1.0; 
                treble_phase += 1.0;
                beat_phase += 1.0;
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

/// Enhanced audio configuration with advanced analysis parameters
#[derive(Resource, Clone, Debug)]
pub struct EnhancedAudioConfig {
    pub fft_size: u32,
    pub smoothing_time_constant: f64,
    pub min_decibels: f32,
    pub max_decibels: f32,
    pub bass_freq_range: (f32, f32),
    pub mid_freq_range: (f32, f32),
    pub treble_freq_range: (f32, f32),
    pub beat_threshold: f32,
    pub tempo_smoothing: f32,
    pub enable_advanced_analysis: bool,
}

impl Default for EnhancedAudioConfig {
    fn default() -> Self {
        Self {
            fft_size: 2048,
            smoothing_time_constant: 0.8,
            min_decibels: -90.0,
            max_decibels: -10.0,
            bass_freq_range: (20.0, 250.0),
            mid_freq_range: (250.0, 2000.0),
            treble_freq_range: (2000.0, 20000.0),
            beat_threshold: 0.3,
            tempo_smoothing: 0.9,
            enable_advanced_analysis: true,
        }
    }
}

/// Enhanced audio analysis data with advanced features
#[derive(Resource, Clone, Debug)]
pub struct EnhancedAudioData {
    pub volume: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub waveform: Vec<f32>,
    pub frequency_data: Vec<f32>,
    pub tempo: f32,
    pub beat_intensity: f32,
    pub timestamp: f64,
    pub spectral_centroid: f32,
    pub spectral_rolloff: f32,
    pub zero_crossing_rate: f32,
    pub rms_energy: f32,
}

impl Default for EnhancedAudioData {
    fn default() -> Self {
        Self {
            volume: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            waveform: vec![0.0; 512],
            frequency_data: vec![0.0; 1024],
            tempo: 120.0,
            beat_intensity: 0.0,
            timestamp: 0.0,
            spectral_centroid: 0.0,
            spectral_rolloff: 0.0,
            zero_crossing_rate: 0.0,
            rms_energy: 0.0,
        }
    }
}

/// Enhanced audio analyzer with advanced DSP features
#[derive(Resource)]
pub struct EnhancedAudioAnalyzer {
    pub enabled: bool,
    pub config: EnhancedAudioConfig,
    data: Arc<Mutex<EnhancedAudioData>>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    last_update: Instant,
}

impl Default for EnhancedAudioAnalyzer {
    fn default() -> Self {
        Self {
            enabled: false,
            config: EnhancedAudioConfig::default(),
            data: Arc::new(Mutex::new(EnhancedAudioData::default())),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 44100,
            last_update: Instant::now(),
        }
    }
}

impl EnhancedAudioAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn start_analysis(&mut self) {
        if self.enabled {
            return;
        }
        
        self.enabled = true;
        let data = Arc::clone(&self.data);
        let audio_buffer = Arc::clone(&self.audio_buffer);
        let config = self.config.clone();
        
        std::thread::spawn(move || {
            Self::enhanced_audio_capture_thread(data, audio_buffer, config);
        });
    }
    
    pub fn stop_analysis(&mut self) {
        self.enabled = false;
    }
    
    pub fn get_data(&self) -> EnhancedAudioData {
        self.data.lock().unwrap().clone()
    }
    
    pub fn process_frame(&mut self) {
        if !self.enabled || !self.config.enable_advanced_analysis {
            return;
        }
        
        let audio_samples = {
            let mut buffer = self.audio_buffer.lock().unwrap();
            if buffer.is_empty() {
                return;
            }
            std::mem::take(&mut *buffer)
        };
        
        if !audio_samples.is_empty() {
            self.analyze_enhanced_audio(&audio_samples);
        }
        
        self.last_update = Instant::now();
    }
    
    fn enhanced_audio_capture_thread(
        data: Arc<Mutex<EnhancedAudioData>>, 
        audio_buffer: Arc<Mutex<Vec<f32>>>, 
        config: EnhancedAudioConfig
    ) {
        let sample_rate = 44100.0;
        let buffer_size = config.fft_size as usize;
        let mut phase = 0.0f32;
        
        loop {
            let mut samples = Vec::with_capacity(buffer_size);
            
            // Generate enhanced synthetic audio with multiple frequencies
            for i in 0..buffer_size {
                let t = i as f32 / sample_rate;
                let sample = (phase * 0.1 + t * 2.0 * std::f32::consts::PI * 440.0).sin() * 0.3
                    + (phase * 0.3 + t * 2.0 * std::f32::consts::PI * 880.0).sin() * 0.2
                    + (phase * 0.5 + t * 2.0 * std::f32::consts::PI * 1760.0).sin() * 0.1;
                samples.push(sample);
                phase += 0.01;
            }
            
            // Add samples to buffer
            {
                let mut buffer = audio_buffer.lock().unwrap();
                buffer.extend(samples);
                // Keep buffer size reasonable
                if buffer.len() > buffer_size * 4 {
                    let drain_to = buffer.len() - buffer_size * 2;
                    buffer.drain(..drain_to);
                }
            }
            
            std::thread::sleep(Duration::from_millis((buffer_size * 1000) as u64 / sample_rate as u64));
        }
    }
    
    fn analyze_enhanced_audio(&mut self, samples: &[f32]) {
        let mut data = self.data.lock().unwrap();
        let config = &self.config;
        
        // Calculate RMS energy
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        data.rms_energy = (sum_squares / samples.len() as f32).sqrt();
        data.volume = data.rms_energy;
        
        // Advanced frequency analysis
        if samples.len() >= config.fft_size as usize {
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(config.fft_size as usize);
            
            let mut fft_buffer: Vec<Complex<f32>> = samples[..config.fft_size as usize]
                .iter()
                .map(|&x| Complex::new(x, 0.0))
                .collect();
            
            // Apply window function
            for (i, sample) in fft_buffer.iter_mut().enumerate() {
                let window = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / config.fft_size as f32).cos();
                sample.re *= window;
            }
            
            fft.process(&mut fft_buffer);
            
            // Extract frequency data
            data.frequency_data.clear();
            for i in 0..(config.fft_size as usize / 2) {
                let magnitude = fft_buffer[i].norm();
                data.frequency_data.push(magnitude);
            }
            
            // Calculate frequency bands
            let nyquist = self.sample_rate as f32 / 2.0;
            let bin_width = nyquist / (config.fft_size as f32 / 2.0);
            
            let bass_bins = ((config.bass_freq_range.0 / bin_width) as usize)..((config.bass_freq_range.1 / bin_width) as usize).min(data.frequency_data.len());
            let mid_bins = ((config.mid_freq_range.0 / bin_width) as usize)..((config.mid_freq_range.1 / bin_width) as usize).min(data.frequency_data.len());
            let treble_bins = ((config.treble_freq_range.0 / bin_width) as usize)..((config.treble_freq_range.1 / bin_width) as usize).min(data.frequency_data.len());
            
            data.bass = data.frequency_data[bass_bins.clone()].iter().sum::<f32>() / bass_bins.len() as f32;
            data.mid = data.frequency_data[mid_bins.clone()].iter().sum::<f32>() / mid_bins.len() as f32;
            data.treble = data.frequency_data[treble_bins.clone()].iter().sum::<f32>() / treble_bins.len() as f32;
            
            // Calculate spectral features
            self.calculate_spectral_features(&mut data, bin_width);
        }
        
        // Update waveform
        data.waveform.clear();
        let waveform_samples = samples.len().min(512);
        for i in 0..waveform_samples {
            data.waveform.push(samples[i]);
        }
        
        // Advanced beat detection
        self.detect_beats(&mut data);
        
        data.timestamp = self.last_update.elapsed().as_secs_f64();
    }
    
    fn calculate_spectral_features(&self, data: &mut EnhancedAudioData, bin_width: f32) {
        // Spectral centroid (brightness)
        let mut weighted_sum = 0.0;
        let mut magnitude_sum = 0.0;
        
        for (i, &magnitude) in data.frequency_data.iter().enumerate() {
            let freq = i as f32 * bin_width;
            weighted_sum += freq * magnitude;
            magnitude_sum += magnitude;
        }
        
        data.spectral_centroid = if magnitude_sum > 0.0 {
            weighted_sum / magnitude_sum
        } else {
            0.0
        };
        
        // Spectral rolloff (85% of energy)
        let total_energy: f32 = data.frequency_data.iter().sum();
        let threshold = total_energy * 0.85;
        let mut cumulative_energy = 0.0;
        
        for (i, &magnitude) in data.frequency_data.iter().enumerate() {
            cumulative_energy += magnitude;
            if cumulative_energy >= threshold {
                data.spectral_rolloff = i as f32 * bin_width;
                break;
            }
        }
        
        // Zero crossing rate
        let mut zero_crossings = 0;
        for i in 1..data.waveform.len() {
            if (data.waveform[i-1] >= 0.0) != (data.waveform[i] >= 0.0) {
                zero_crossings += 1;
            }
        }
        data.zero_crossing_rate = zero_crossings as f32 / data.waveform.len() as f32;
    }
    
    fn detect_beats(&self, data: &mut EnhancedAudioData) {
        // Enhanced beat detection using spectral flux
        static mut PREVIOUS_ENERGY: f32 = 0.0;
        
        unsafe {
            let current_energy = data.rms_energy;
            let spectral_flux = (current_energy - PREVIOUS_ENERGY).max(0.0);
            PREVIOUS_ENERGY = current_energy;
            
            if spectral_flux > self.config.beat_threshold && current_energy > 0.1 {
                data.beat_intensity = spectral_flux;
                data.tempo = 60.0 / (0.5 + spectral_flux * 2.0); // Estimated BPM
            } else {
                data.beat_intensity *= 0.95; // Decay
            }
        }
    }
}

/// System to update enhanced audio analysis
fn update_enhanced_audio_analysis(
    mut enhanced_analyzer: ResMut<EnhancedAudioAnalyzer>,
    config: Res<EnhancedAudioConfig>,
) {
    if enhanced_analyzer.config.enable_advanced_analysis != config.enable_advanced_analysis {
        enhanced_analyzer.config = config.clone();
    }
    enhanced_analyzer.process_frame();
}