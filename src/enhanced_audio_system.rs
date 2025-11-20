use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use web_sys::{AudioContext, AnalyserNode, MediaDevices, MediaStream, MediaStreamConstraints, Window};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone)]
pub struct AudioAnalysisData {
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub volume: f32,
    pub waveform: Vec<f32>,
    pub frequency_data: Vec<f32>,
    pub tempo: f32,
    pub beat_intensity: f32,
    pub timestamp: f64,
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub fft_size: u32,
    pub smoothing_time_constant: f64,
    pub min_decibels: f32,
    pub max_decibels: f32,
    pub bass_freq_range: (f32, f32),
    pub mid_freq_range: (f32, f32),
    pub treble_freq_range: (f32, f32),
    pub beat_threshold: f32,
    pub tempo_smoothing: f32,
}

impl Default for AudioConfig {
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
        }
    }
}

pub struct EnhancedAudioSystem {
    audio_context: Option<AudioContext>,
    analyser_node: Option<AnalyserNode>,
    source_node: Option<web_sys::MediaStreamAudioSourceNode>,
    media_stream: Option<MediaStream>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    audio_history: Arc<Mutex<VecDeque<AudioAnalysisData>>>,
    config: AudioConfig,
    is_recording: Arc<AtomicBool>,
    beat_history: Arc<Mutex<VecDeque<f32>>>,
    tempo_accumulator: Arc<Mutex<f32>>,
    last_beat_time: Arc<Mutex<Instant>>,
    window: Option<Window>,
}

impl EnhancedAudioSystem {
    pub fn new() -> Self {
        let initial_data = AudioAnalysisData {
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            volume: 0.0,
            waveform: vec![0.0; 2048],
            frequency_data: vec![0.0; 1024],
            tempo: 120.0,
            beat_intensity: 0.0,
            timestamp: 0.0,
        };

        Self {
            audio_context: None,
            analyser_node: None,
            source_node: None,
            media_stream: None,
            audio_data: Arc::new(Mutex::new(initial_data.clone())),
            audio_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            config: AudioConfig::default(),
            is_recording: Arc::new(AtomicBool::new(false)),
            beat_history: Arc::new(Mutex::new(VecDeque::with_capacity(32))),
            tempo_accumulator: Arc::new(Mutex::new(0.0)),
            last_beat_time: Arc::new(Mutex::new(Instant::now())),
            window: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("No window available")?;
        self.window = Some(window.clone());

        let audio_context = AudioContext::new()?;
        
        let analyser_node = audio_context.create_analyser()?;
        analyser_node.set_fft_size(self.config.fft_size);
        analyser_node.set_smoothing_time_constant(self.config.smoothing_time_constant);
        analyser_node.set_min_decibels(self.config.min_decibels);
        analyser_node.set_max_decibels(self.config.max_decibels);

        self.audio_context = Some(audio_context);
        self.analyser_node = Some(analyser_node);

        Ok(())
    }

    pub async fn start_audio_capture(&mut self) -> Result<(), JsValue> {
        if self.window.is_none() {
            return Err(JsValue::from_str("Window not initialized"));
        }

        let window = self.window.as_ref().unwrap();
        let navigator = window.navigator();
        let media_devices: MediaDevices = navigator.media_devices()?;

        let mut constraints = MediaStreamConstraints::new();
        let audio_constraints = js_sys::Object::new();
        js_sys::Reflect::set(&audio_constraints, &"echoCancellation".into(), &false.into())?;
        js_sys::Reflect::set(&audio_constraints, &"noiseSuppression".into(), &false.into())?;
        js_sys::Reflect::set(&audio_constraints, &"autoGainControl".into(), &false.into())?;
        constraints.audio(&audio_constraints);

        let promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let media_stream = JsFuture::from(promise).await?;
        let media_stream: MediaStream = media_stream.dyn_into()?;

        if let (Some(audio_context), Some(analyser_node)) = (&self.audio_context, &self.analyser_node) {
            let source_node = audio_context.create_media_stream_source(&media_stream)?;
            source_node.connect_with_audio_node(analyser_node)?;
            
            self.source_node = Some(source_node);
            self.media_stream = Some(media_stream);
            self.is_recording.store(true, Ordering::SeqCst);
            
            self.start_audio_processing();
            Ok(())
        } else {
            Err(JsValue::from_str("Audio context not initialized"))
        }
    }

    pub fn stop_audio_capture(&mut self) {
        self.is_recording.store(false, Ordering::SeqCst);
        
        if let Some(stream) = &self.media_stream {
            stream.get_tracks().for_each(|track| {
                track.stop();
            });
        }
        
        if let Some(source) = &self.source_node {
            source.disconnect();
        }
        
        self.source_node = None;
        self.media_stream = None;
    }

    fn start_audio_processing(&self) {
        let analyser_node = self.analyser_node.clone();
        let audio_data = self.audio_data.clone();
        let audio_history = self.audio_history.clone();
        let config = self.config.clone();
        let is_recording = self.is_recording.clone();
        let beat_history = self.beat_history.clone();
        let tempo_accumulator = self.tempo_accumulator.clone();
        let last_beat_time = self.last_beat_time.clone();

        let process_audio = move || {
            if let Some(analyser) = analyser_node {
                let mut waveform_data = vec![0.0; config.fft_size as usize];
                let mut frequency_data = vec![0.0; (config.fft_size / 2) as usize];
                
                analyser.get_float_time_domain_data(&mut waveform_data);
                analyser.get_float_frequency_data(&mut frequency_data);

                let (bass, mid, treble) = self.calculate_frequency_bands(&frequency_data, &config);
                let volume = self.calculate_volume(&waveform_data);
                let beat_intensity = self.detect_beat(&frequency_data, &config, &beat_history);
                let tempo = self.calculate_tempo(&beat_intensity, &tempo_accumulator, &last_beat_time);

                let analysis_data = AudioAnalysisData {
                    bass,
                    mid,
                    treble,
                    volume,
                    waveform: waveform_data.clone(),
                    frequency_data: frequency_data.clone(),
                    tempo,
                    beat_intensity,
                    timestamp: web_sys::window()
                        .and_then(|w| w.performance())
                        .map(|p| p.now())
                        .unwrap_or(0.0),
                };

                if let Ok(mut data) = audio_data.lock() {
                    *data = analysis_data.clone();
                }

                if let Ok(mut history) = audio_history.lock() {
                    history.push_back(analysis_data);
                    if history.len() > 1000 {
                        history.pop_front();
                    }
                }
            }
        };

        let process_audio_wrapper = Closure::wrap(Box::new(process_audio) as Box<dyn FnMut()>);
        
        if let Some(window) = &self.window {
            let process_fn = process_audio_wrapper.as_ref().unchecked_ref();
            let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
                process_fn, 
                16 // ~60fps
            );
        }
        
        process_audio_wrapper.forget();
    }

    fn calculate_frequency_bands(&self, frequency_data: &[f32], config: &AudioConfig) -> (f32, f32, f32) {
        let sample_rate = 44100.0; // Standard audio sample rate
        let freq_resolution = sample_rate / config.fft_size as f32;
        
        let mut bass_energy = 0.0;
        let mut mid_energy = 0.0;
        let mut treble_energy = 0.0;
        
        let bass_start = (config.bass_freq_range.0 / freq_resolution) as usize;
        let bass_end = (config.bass_freq_range.1 / freq_resolution) as usize;
        let mid_end = (config.mid_freq_range.1 / freq_resolution) as usize;
        let treble_end = std::cmp::min((config.treble_freq_range.1 / freq_resolution) as usize, frequency_data.len());

        for i in bass_start..std::cmp::min(bass_end, frequency_data.len()) {
            bass_energy += frequency_data[i].powi(2);
        }
        
        for i in bass_end..std::cmp::min(mid_end, frequency_data.len()) {
            mid_energy += frequency_data[i].powi(2);
        }
        
        for i in mid_end..treble_end {
            treble_energy += frequency_data[i].powi(2);
        }

        let bass_samples = (bass_end - bass_start).max(1);
        let mid_samples = (mid_end - bass_end).max(1);
        let treble_samples = (treble_end - mid_end).max(1);

        (
            (bass_energy / bass_samples as f32).sqrt(),
            (mid_energy / mid_samples as f32).sqrt(),
            (treble_energy / treble_samples as f32).sqrt(),
        )
    }

    fn calculate_volume(&self, waveform: &[f32]) -> f32 {
        let sum: f32 = waveform.iter().map(|x| x.abs()).sum();
        sum / waveform.len() as f32
    }

    fn detect_beat(&self, frequency_data: &[f32], config: &AudioConfig, beat_history: &Arc<Mutex<VecDeque<f32>>>) -> f32 {
        let energy: f32 = frequency_data.iter().map(|x| x.powi(2)).sum::<f32>() / frequency_data.len() as f32;
        let instant_energy = energy.sqrt();

        if let Ok(mut history) = beat_history.lock() {
            if history.len() >= 32 {
                history.pop_front();
            }
            history.push_back(instant_energy);

            if history.len() >= 32 {
                let average_energy: f32 = history.iter().sum::<f32>() / history.len() as f32;
                let variance: f32 = history.iter().map(|x| (x - average_energy).powi(2)).sum::<f32>() / history.len() as f32;
                let standard_deviation = variance.sqrt();
                
                let beat_threshold = average_energy + config.beat_threshold * standard_deviation;
                
                if instant_energy > beat_threshold {
                    return 1.0; // Strong beat
                } else if instant_energy > average_energy {
                    return 0.5; // Weak beat
                }
            }
        }
        
        0.0 // No beat
    }

    fn calculate_tempo(&self, beat_intensity: f32, tempo_accumulator: &Arc<Mutex<f32>>, last_beat_time: &Arc<Mutex<Instant>>) -> f32 {
        if beat_intensity > 0.5 {
            let current_time = Instant::now();
            
            if let (Ok(mut last_time), Ok(mut accumulator)) = (last_beat_time.lock(), tempo_accumulator.lock()) {
                let time_since_last_beat = current_time.duration_since(*last_time).as_secs_f32();
                
                if time_since_last_beat > 0.1 && time_since_last_beat < 2.0 { // Reasonable beat interval
                    let instant_tempo = 60.0 / time_since_last_beat;
                    *accumulator = *accumulator * 0.9 + instant_tempo * 0.1;
                }
                
                *last_time = current_time;
            }
        }

        if let Ok(accumulator) = tempo_accumulator.lock() {
            *accumulator
        } else {
            120.0 // Default tempo
        }
    }

    pub fn get_current_audio_data(&self) -> AudioAnalysisData {
        self.audio_data.lock().unwrap().clone()
    }

    pub fn get_audio_history(&self) -> Vec<AudioAnalysisData> {
        self.audio_history.lock().unwrap().iter().cloned().collect()
    }

    pub fn update_config(&mut self, new_config: AudioConfig) {
        self.config = new_config;
        
        if let Some(analyser) = &self.analyser_node {
            analyser.set_fft_size(self.config.fft_size);
            analyser.set_smoothing_time_constant(self.config.smoothing_time_constant);
            analyser.set_min_decibels(self.config.min_decibels);
            analyser.set_max_decibels(self.config.max_decibels);
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn get_shader_uniforms(&self) -> AudioShaderUniforms {
        let data = self.get_current_audio_data();
        
        AudioShaderUniforms {
            u_audio_bass: data.bass,
            u_audio_mid: data.mid,
            u_audio_treble: data.treble,
            u_audio_volume: data.volume,
            u_audio_tempo: data.tempo,
            u_audio_beat_intensity: data.beat_intensity,
            u_audio_time: data.timestamp as f32 * 0.001,
            u_audio_waveform: data.waveform,
            u_audio_spectrum: data.frequency_data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioShaderUniforms {
    pub u_audio_bass: f32,
    pub u_audio_mid: f32,
    pub u_audio_treble: f32,
    pub u_audio_volume: f32,
    pub u_audio_tempo: f32,
    pub u_audio_beat_intensity: f32,
    pub u_audio_time: f32,
    pub u_audio_waveform: Vec<f32>,
    pub u_audio_spectrum: Vec<f32>,
}

impl AudioShaderUniforms {
    pub fn to_wgsl_struct(&self) -> String {
        format!(
            r#"
struct AudioData {{
    bass: f32,
    mid: f32,
    treble: f32,
    volume: f32,
    tempo: f32,
    beat_intensity: f32,
    time: f32,
    waveform: array<f32, {}>,
    spectrum: array<f32, {}>,
}}
"#,
            self.u_audio_waveform.len(),
            self.u_audio_spectrum.len()
        )
    }

    pub fn to_wgsl_uniforms(&self) -> String {
        format!(
            r#"
@group(0) @binding(0) var<uniform> audio_bass: f32;
@group(0) @binding(1) var<uniform> audio_mid: f32;
@group(0) @binding(2) var<uniform> audio_treble: f32;
@group(0) @binding(3) var<uniform> audio_volume: f32;
@group(0) @binding(4) var<uniform> audio_tempo: f32;
@group(0) @binding(5) var<uniform> audio_beat_intensity: f32;
@group(0) @binding(6) var<uniform> audio_time: f32;
@group(0) @binding(7) var<storage> audio_waveform: array<f32>;
@group(0) @binding(8) var<storage> audio_spectrum: array<f32>;
"#
        )
    }
}

pub struct AudioMidiIntegration {
    audio_system: EnhancedAudioSystem,
    midi_inputs: Vec<String>,
    midi_data: Arc<Mutex<Vec<MidiMessage>>>,
}

#[derive(Debug, Clone)]
pub struct MidiMessage {
    pub timestamp: f64,
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
    pub message_type: MidiMessageType,
}

#[derive(Debug, Clone)]
pub enum MidiMessageType {
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
    PitchBend,
}

impl AudioMidiIntegration {
    pub fn new() -> Self {
        Self {
            audio_system: EnhancedAudioSystem::new(),
            midi_inputs: Vec::new(),
            midi_data: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), JsValue> {
        self.audio_system.initialize().await
    }

    pub fn get_combined_shader_uniforms(&self) -> CombinedAudioMidiUniforms {
        let audio_uniforms = self.audio_system.get_shader_uniforms();
        let midi_data = self.get_midi_shader_data();
        
        CombinedAudioMidiUniforms {
            audio: audio_uniforms,
            midi: midi_data,
        }
    }

    fn get_midi_shader_data(&self) -> MidiShaderData {
        if let Ok(midi_messages) = self.midi_data.lock() {
            let recent_notes: Vec<u8> = midi_messages
                .iter()
                .rev()
                .take(8)
                .filter(|msg| matches!(msg.message_type, MidiMessageType::NoteOn))
                .map(|msg| msg.note)
                .collect();

            let total_velocity: u32 = midi_messages
                .iter()
                .filter(|msg| matches!(msg.message_type, MidiMessageType::NoteOn))
                .map(|msg| msg.velocity as u32)
                .sum();

            MidiShaderData {
                active_notes: recent_notes,
                total_velocity: total_velocity as f32 / 127.0,
                note_count: midi_messages.len() as f32,
            }
        } else {
            MidiShaderData::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct CombinedAudioMidiUniforms {
    pub audio: AudioShaderUniforms,
    pub midi: MidiShaderData,
}

#[derive(Debug, Clone, Default)]
pub struct MidiShaderData {
    pub active_notes: Vec<u8>,
    pub total_velocity: f32,
    pub note_count: f32,
}

impl MidiShaderData {
    pub fn to_wgsl_uniforms(&self) -> String {
        format!(
            r#"
@group(1) @binding(0) var<uniform> midi_active_notes: array<f32, {}>;
@group(1) @binding(1) var<uniform> midi_total_velocity: f32;
@group(1) @binding(2) var<uniform> midi_note_count: f32;
"#,
            self.active_notes.len()
        )
    }
}