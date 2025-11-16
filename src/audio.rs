use bevy::prelude::*;
use std::time::Instant;

#[derive(Resource)]
pub struct AudioAnalyzer {
    pub enabled: bool,
    pub gain: f32,
    pub bass_level: f32,
    pub mid_level: f32,
    pub treble_level: f32,
    pub overall_level: f32,
    pub beat_detected: bool,
    pub beat_intensity: f32,
}

impl Default for AudioAnalyzer {
    fn default() -> Self {
        Self {
            enabled: false,
            gain: 1.0,
            bass_level: 0.0,
            mid_level: 0.0,
            treble_level: 0.0,
            overall_level: 0.0,
            beat_detected: false,
            beat_intensity: 0.0,
        }
    }
}

impl AudioAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        info!("Audio analyzer enabled");
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        info!("Audio analyzer disabled");
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 5.0);
    }

    pub fn process_audio_frame(&mut self) {
        if !self.enabled {
            return;
        }

        let time = Instant::now().elapsed().as_secs_f64();
        
        self.bass_level = ((time * 2.0).sin() as f32 * 0.5 + 0.5) * self.gain;
        self.mid_level = ((time * 4.0).sin() as f32 * 0.3 + 0.3) * self.gain;
        self.treble_level = ((time * 8.0).sin() as f32 * 0.2 + 0.2) * self.gain;
        self.overall_level = (self.bass_level + self.mid_level + self.treble_level) / 3.0;
        
        self.beat_detected = self.bass_level > 0.4;
        if self.beat_detected {
            self.beat_intensity = self.bass_level.min(1.0);
        } else {
            self.beat_intensity *= 0.95;
        }
    }
}

pub struct AudioAnalysisPlugin;

impl Plugin for AudioAnalysisPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioAnalyzer::new())
            .add_systems(Update, update_audio_analysis);
    }
}

#[derive(Clone, Debug)]
pub struct AudioData {
    pub enabled: bool,
    pub bass_level: f32,
    pub mid_level: f32,
    pub treble_level: f32,
    pub overall_level: f32,
    pub beat_detected: bool,
    pub beat_intensity: f32,
    pub volume: f32, // Overall volume level for compatibility
}

impl AudioAnalyzer {
    pub fn get_audio_data(&self) -> AudioData {
        AudioData {
            enabled: self.enabled,
            bass_level: self.bass_level,
            mid_level: self.mid_level,
            treble_level: self.treble_level,
            overall_level: self.overall_level,
            beat_detected: self.beat_detected,
            beat_intensity: self.beat_intensity,
            volume: self.overall_level, // Use overall level as volume
        }
    }
}

pub struct AudioMidiSystem {
    pub audio_analyzer: AudioAnalyzer,
}

impl AudioMidiSystem {
    pub fn new() -> Self {
        Self {
            audio_analyzer: AudioAnalyzer::new(),
        }
    }

    pub fn get_parameter(&self, name: &str, default: f32) -> f32 {
        match name {
            "u_reactive_bass" => self.audio_analyzer.bass_level,
            "u_reactive_mid" => self.audio_analyzer.mid_level,
            "u_reactive_treble" => self.audio_analyzer.treble_level,
            "u_reactive_beat" => if self.audio_analyzer.beat_detected { self.audio_analyzer.beat_intensity } else { 0.0 },
            "u_reactive_overall" => self.audio_analyzer.overall_level,
            _ => default,
        }
    }
}

impl Default for AudioMidiSystem {
    fn default() -> Self {
        Self::new()
    }
}

fn update_audio_analysis(mut audio_analyzer: ResMut<AudioAnalyzer>) {
    audio_analyzer.process_audio_frame();
}