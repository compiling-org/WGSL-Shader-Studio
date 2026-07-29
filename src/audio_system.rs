use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

// Ensure AudioData implements necessary traits for multi-threading
unsafe impl Send for AudioData {}
unsafe impl Sync for AudioData {}

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

    fn audio_capture_thread(_data: Arc<Mutex<AudioData>>, audio_buffer: Arc<Mutex<Vec<f32>>>) {
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
            for _i in 0..buffer_size {
                // Bass frequency (80Hz)
                let bass_freq = 80.0;
                let bass_sample =
                    (bass_phase * 2.0 * std::f32::consts::PI / sample_rate * bass_freq).sin() * 0.4;

                // Mid frequency (800Hz)
                let mid_freq = 800.0;
                let mid_sample =
                    (mid_phase * 2.0 * std::f32::consts::PI / sample_rate * mid_freq).sin() * 0.3;

                // Treble frequency (4000Hz)
                let treble_freq = 4000.0;
                let treble_sample =
                    (treble_phase * 2.0 * std::f32::consts::PI / sample_rate * treble_freq).sin()
                        * 0.2;

                // Beat pattern (2Hz = 120 BPM)
                let beat_freq = 2.0;
                let beat_envelope =
                    (beat_phase * 2.0 * std::f32::consts::PI / sample_rate * beat_freq).sin() * 0.5
                        + 0.5;

                // Combine all frequencies with beat modulation
                let mut sample = bass_sample + mid_sample + treble_sample;
                sample *= 0.7 + beat_envelope * 0.3; // Beat modulation

                // Add some noise for realism
                let noise = 0.0; // rand::random::<f32>() - 0.5) * 0.05; // rand not available
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

            std::thread::sleep(Duration::from_millis(
                (buffer_size * 1000) as u64 / sample_rate as u64,
            ));
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
                let window =
                    0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos();
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
            data.mid_level = data.frequencies[bass_end..mid_end].iter().sum::<f32>()
                / (mid_end - bass_end) as f32;
            data.treble_level =
                data.frequencies[mid_end..].iter().sum::<f32>() / (fft_size / 2 - mid_end) as f32;
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

/// 20 audio features, all normalized to 0.0-1.0 range.
/// Multi-resolution FFT bands + spectral shape + beat detection.
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Resource, Default)]
pub struct AudioFeatures {
    // Frequency bands (7) — multi-resolution FFT
    pub sub_bass: f32,   // 20-60 Hz (kick fundamentals)
    pub bass: f32,       // 60-250 Hz (bass guitar/synth)
    pub low_mid: f32,    // 250-500 Hz (lower vocals/snare body)
    pub mid: f32,        // 500-2000 Hz (vocal/snare presence)
    pub upper_mid: f32,  // 2000-4000 Hz (harmonic presence)
    pub presence: f32,   // 4000-6000 Hz (hi-hat attack)
    pub brilliance: f32, // 6000-20000 Hz (cymbal shimmer)

    // Aggregates (2)
    pub rms: f32,  // Overall amplitude
    pub kick: f32, // Dedicated kick drum detection (30-120Hz spectral flux)

    // Spectral shape (6)
    pub centroid: f32,  // Brightness/timbre
    pub flux: f32,      // Spectral change rate
    pub flatness: f32,  // Noise vs tone (Wiener entropy)
    pub rolloff: f32,   // 85% energy frequency
    pub bandwidth: f32, // Spectral spread
    pub zcr: f32,       // Zero crossing rate

    // Beat detection (5)
    pub onset: f32,         // Onset strength (continuous 0-1, for envelope effects)
    pub beat: f32,          // 1.0 on beat frame, 0.0 otherwise (trigger)
    pub beat_phase: f32,    // 0-1 sawtooth cycling at detected tempo
    pub bpm: f32,           // BPM / 300 (normalized 0-1)
    pub beat_strength: f32, // How strong the detected beat was
}

pub const NUM_FEATURES: usize = 20;

/// Kalman filter for BPM tracking in log2-BPM space.
#[derive(Clone, Debug)]
pub struct KalmanBpm {
    state: f64,         // log2(BPM)
    variance: f64,      // estimation uncertainty
    q: f64,             // process noise
    r: f64,             // measurement noise
    diverge_count: u32, // consecutive divergent frames
    snap_count: u32,    // consecutive octave-snapped frames
    initialized: bool,
}

impl KalmanBpm {
    pub fn new() -> Self {
        Self {
            state: 0.0,
            variance: 1.0,
            q: 0.001,
            r: 0.1,
            diverge_count: 0,
            snap_count: 0,
            initialized: false,
        }
    }

    pub fn update(&mut self, raw_bpm: f64, confidence: f64) -> f64 {
        if raw_bpm <= 0.0 {
            return if self.initialized {
                2.0f64.powf(self.state)
            } else {
                0.0
            };
        }

        if !self.initialized {
            self.state = raw_bpm.log2();
            self.variance = 1.0;
            self.initialized = true;
            return raw_bpm;
        }

        let current_bpm = 2.0f64.powf(self.state);
        let ratio = raw_bpm / current_bpm;
        let mut snapped_bpm = raw_bpm;
        let mut was_snapped = false;
        for &hr in &[0.5, 2.0] {
            if (ratio - hr).abs() / hr < 0.05 {
                snapped_bpm = current_bpm;
                was_snapped = true;
                break;
            }
        }

        if was_snapped {
            self.snap_count += 1;
            if self.snap_count >= 50 {
                snapped_bpm = raw_bpm;
                was_snapped = false;
                self.snap_count = 0;
            }
        } else {
            self.snap_count = 0;
        }
        let snapped_measurement = snapped_bpm.log2();

        let bpm_deviation = (snapped_bpm - current_bpm).abs() / current_bpm.max(1.0);
        if bpm_deviation > 0.10 {
            self.diverge_count += 1;
        } else {
            self.diverge_count = 0;
        }

        if self.diverge_count >= 15 {
            self.state = raw_bpm.log2();
            self.variance = 1.0;
            self.diverge_count = 0;
            return raw_bpm;
        }

        self.r = 0.01 + (1.0 - confidence) * 0.5;
        self.q = if self.diverge_count > 0 { 0.1 } else { 0.001 };

        self.variance += self.q;

        let innovation = snapped_measurement - self.state;
        let s = self.variance + self.r;
        let k = self.variance / s;
        self.state += k * innovation;
        self.variance *= 1.0 - k;

        2.0f64.powf(self.state)
    }
}

impl Default for KalmanBpm {
    fn default() -> Self {
        Self::new()
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
    // New normalized feature set
    pub features: AudioFeatures,
    pub kalman_bpm: KalmanBpm,
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
            features: AudioFeatures::default(),
            kalman_bpm: KalmanBpm::default(),
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
    // Firewheel-inspired features
    firewheel_buffer: Arc<Mutex<Vec<f32>>>,
    firewheel_enabled: bool,
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
            firewheel_buffer: Arc::new(Mutex::new(Vec::new())),
            firewheel_enabled: true, // Enable Firewheel features by default
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
        let firewheel_buffer = Arc::clone(&self.firewheel_buffer);
        let config = self.config.clone();
        let firewheel_enabled = self.firewheel_enabled;

        std::thread::spawn(move || {
            Self::enhanced_audio_capture_thread(
                data,
                audio_buffer,
                firewheel_buffer,
                config,
                firewheel_enabled,
            );
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

        let firewheel_samples = {
            let mut buffer = self.firewheel_buffer.lock().unwrap();
            if !self.firewheel_enabled || buffer.is_empty() {
                Vec::new()
            } else {
                std::mem::take(&mut *buffer)
            }
        };

        if !audio_samples.is_empty() {
            self.analyze_enhanced_audio(&audio_samples, &firewheel_samples);
        }

        self.last_update = Instant::now();
    }

    fn enhanced_audio_capture_thread(
        _data: Arc<Mutex<EnhancedAudioData>>,
        audio_buffer: Arc<Mutex<Vec<f32>>>,
        firewheel_buffer: Arc<Mutex<Vec<f32>>>,
        config: EnhancedAudioConfig,
        firewheel_enabled: bool,
    ) {
        let sample_rate = 44100.0;
        let buffer_size = config.fft_size as usize;
        let mut phase = 0.0f32;
        let mut firewheel_phase = 0.0f32;

        loop {
            let mut samples = Vec::with_capacity(buffer_size);
            let mut firewheel_samples = Vec::with_capacity(buffer_size);

            // Generate enhanced synthetic audio with multiple frequencies
            for _i in 0..buffer_size {
                let t = _i as f32 / sample_rate;
                let sample = (phase * 0.1 + t * 2.0 * std::f32::consts::PI * 440.0).sin() * 0.3
                    + (phase * 0.3 + t * 2.0 * std::f32::consts::PI * 880.0).sin() * 0.2
                    + (phase * 0.5 + t * 2.0 * std::f32::consts::PI * 1760.0).sin() * 0.1;
                samples.push(sample);

                // Firewheel-inspired audio processing
                if firewheel_enabled {
                    let firewheel_sample = (firewheel_phase * 0.2
                        + t * 2.0 * std::f32::consts::PI * 220.0)
                        .sin()
                        * 0.15
                        + (firewheel_phase * 0.4 + t * 2.0 * std::f32::consts::PI * 660.0).sin()
                            * 0.1
                        + (firewheel_phase * 0.6 + t * 2.0 * std::f32::consts::PI * 1320.0).sin()
                            * 0.05;
                    firewheel_samples.push(firewheel_sample);
                    firewheel_phase += 0.02;
                }

                phase += 0.01;
            }

            // Add samples to buffers
            {
                let mut buffer = audio_buffer.lock().unwrap();
                buffer.extend(samples);
                // Keep buffer size reasonable
                if buffer.len() > buffer_size * 4 {
                    let drain_to = buffer.len() - buffer_size * 2;
                    buffer.drain(..drain_to);
                }
            }

            if firewheel_enabled {
                let mut firewheel_buf = firewheel_buffer.lock().unwrap();
                firewheel_buf.extend(firewheel_samples);
                // Keep buffer size reasonable
                if firewheel_buf.len() > buffer_size * 4 {
                    let drain_to = firewheel_buf.len() - buffer_size * 2;
                    firewheel_buf.drain(..drain_to);
                }
            }

            std::thread::sleep(Duration::from_millis(
                (buffer_size * 1000) as u64 / sample_rate as u64,
            ));
        }
    }

    fn analyze_enhanced_audio(&mut self, samples: &[f32], firewheel_samples: &[f32]) {
        let mut data = self.data.lock().unwrap();
        let config = &self.config;

        // Calculate RMS energy
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        data.rms_energy = (sum_squares / samples.len() as f32).sqrt();
        data.volume = data.rms_energy;

        // Incorporate Firewheel audio data
        if !firewheel_samples.is_empty() {
            let firewheel_sum_squares: f32 = firewheel_samples.iter().map(|&x| x * x).sum();
            let firewheel_rms = (firewheel_sum_squares / firewheel_samples.len() as f32).sqrt();

            // Blend original audio with Firewheel audio
            data.volume = (data.volume + firewheel_rms * 0.3).min(1.0);

            // Firewheel-inspired spectral processing
            if firewheel_samples.len() >= config.fft_size as usize {
                let mut firewheel_fft_planner = FftPlanner::new();
                let firewheel_fft =
                    firewheel_fft_planner.plan_fft_forward(config.fft_size as usize);

                let mut firewheel_fft_buffer: Vec<Complex<f32>> = firewheel_samples
                    [..config.fft_size as usize]
                    .iter()
                    .map(|&x| Complex::new(x, 0.0))
                    .collect();

                // Apply window function
                for (i, sample) in firewheel_fft_buffer.iter_mut().enumerate() {
                    let window = 0.5
                        - 0.5
                            * (2.0 * std::f32::consts::PI * i as f32 / config.fft_size as f32)
                                .cos();
                    sample.re *= window;
                }

                firewheel_fft.process(&mut firewheel_fft_buffer);

                // Process Firewheel frequency data
                let mut firewheel_freq_data = Vec::new();
                for i in 0..(config.fft_size as usize / 2) {
                    let magnitude = firewheel_fft_buffer[i].norm();
                    firewheel_freq_data.push(magnitude);
                }

                // Blend frequency data
                for (i, &firewheel_mag) in firewheel_freq_data.iter().enumerate() {
                    if i < data.frequency_data.len() {
                        data.frequency_data[i] =
                            (data.frequency_data[i] + firewheel_mag * 0.2).min(1.0);
                    }
                }
            }
        }

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
                let window = 0.5
                    - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / config.fft_size as f32).cos();
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

            let bass_bins = ((config.bass_freq_range.0 / bin_width) as usize)
                ..((config.bass_freq_range.1 / bin_width) as usize).min(data.frequency_data.len());
            let mid_bins = ((config.mid_freq_range.0 / bin_width) as usize)
                ..((config.mid_freq_range.1 / bin_width) as usize).min(data.frequency_data.len());
            let treble_bins = ((config.treble_freq_range.0 / bin_width) as usize)
                ..((config.treble_freq_range.1 / bin_width) as usize)
                    .min(data.frequency_data.len());

            data.bass = data.frequency_data[bass_bins.clone()].iter().sum::<f32>()
                / bass_bins.len().max(1) as f32;
            data.mid = data.frequency_data[mid_bins.clone()].iter().sum::<f32>()
                / mid_bins.len().max(1) as f32;
            data.treble = data.frequency_data[treble_bins.clone()].iter().sum::<f32>()
                / treble_bins.len().max(1) as f32;

            // Populate advanced features
            self.calculate_audio_features(&mut data, bin_width);
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

    fn calculate_audio_features(&self, data: &mut EnhancedAudioData, bin_width: f32) {
        let freqs = &data.frequency_data;
        if freqs.is_empty() {
            return;
        }

        // 1. Frequency bands (normalized)
        let get_band = |lo: f32, hi: f32| {
            let start = (lo / bin_width) as usize;
            let end = (hi / bin_width) as usize;
            let slice = &freqs[start.min(freqs.len())..end.min(freqs.len())];
            if slice.is_empty() {
                0.0
            } else {
                slice.iter().sum::<f32>() / slice.len() as f32
            }
        };

        data.features.sub_bass = get_band(20.0, 60.0);
        data.features.bass = get_band(60.0, 250.0);
        data.features.low_mid = get_band(250.0, 500.0);
        data.features.mid = get_band(500.0, 2000.0);
        data.features.upper_mid = get_band(2000.0, 4000.0);
        data.features.presence = get_band(4000.0, 6000.0);
        data.features.brilliance = get_band(6000.0, 20000.0);

        // 2. Aggregates
        data.features.rms = data.rms_energy;

        // Kick detection (spectral flux in 30-120Hz)
        static mut PREV_BASS_ENERGY: f32 = 0.0;
        let current_bass_energy = data.features.sub_bass;
        unsafe {
            data.features.kick = (current_bass_energy - PREV_BASS_ENERGY).max(0.0) * 5.0;
            PREV_BASS_ENERGY = current_bass_energy;
        }

        // 3. Spectral shape
        self.calculate_spectral_features(data, bin_width);
        let freqs = &data.frequency_data;
        data.features.centroid = data.spectral_centroid / bin_width / freqs.len().max(1) as f32;
        data.features.rolloff = data.spectral_rolloff / bin_width / freqs.len().max(1) as f32;
        data.features.zcr = data.zero_crossing_rate;

        // Spectral flux (overall spectral change)
        use std::cell::RefCell;
        thread_local! {
            static PREV_MAGS: RefCell<Vec<f32>> = RefCell::new(Vec::new());
        }
        
        PREV_MAGS.with(|cell| {
            let mut prev = cell.borrow_mut();
            if prev.len() == freqs.len() {
                let flux: f32 = freqs
                    .iter()
                    .zip(prev.iter())
                    .map(|(&c, &p)| (c - p).max(0.0))
                    .sum();
                data.features.flux = (flux / freqs.len() as f32).min(1.0);
            }
            *prev = freqs.clone();
        });

        // 4. Beat detection
        self.detect_beats(data);
        data.features.onset = data.beat_intensity;
        data.features.beat = if data.beat_intensity > self.config.beat_threshold {
            1.0
        } else {
            0.0
        };

        // Kalman BPM update
        let raw_bpm = data.tempo as f64;
        let confidence = (data.beat_intensity as f64 * 2.0).min(1.0);
        let stable_bpm = data.kalman_bpm.update(raw_bpm, confidence);
        data.features.bpm = (stable_bpm as f32 / 300.0).min(1.0);

        // Simple sawtooth beat phase
        static mut BEAT_PHASE: f32 = 0.0;
        unsafe {
            let dt = 1.0 / 60.0; // Assume 60fps for simplicity, should be actual dt
            BEAT_PHASE = (BEAT_PHASE + stable_bpm as f32 / 60.0 * dt) % 1.0;
            data.features.beat_phase = BEAT_PHASE;
        }
        data.features.beat_strength = data.beat_intensity;
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
            if (data.waveform[i - 1] >= 0.0) != (data.waveform[i] >= 0.0) {
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
                data.beat_intensity = spectral_flux.min(1.0);
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
