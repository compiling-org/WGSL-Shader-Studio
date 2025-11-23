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
    pub volume: f32,
    pub beat_detected: bool,
    pub beat_intensity: f32,
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            enabled: false,
            bass_level: 0.0,
            mid_level: 0.0,
            treble_level: 0.0,
            overall_level: 0.0,
            volume: 0.0,
            beat_detected: false,
            beat_intensity: 0.0,
        }
    }
}

impl AudioData {
    pub fn from_analyzer(analyzer: &AudioAnalyzer) -> Self {
        Self {
            enabled: analyzer.enabled,
            bass_level: analyzer.bass_level,
            mid_level: analyzer.mid_level,
            treble_level: analyzer.treble_level,
            overall_level: analyzer.overall_level,
            volume: analyzer.overall_level,
            beat_detected: analyzer.beat_detected,
            beat_intensity: analyzer.beat_intensity,
        }
    }
}

fn update_audio_analysis(mut analyzer: ResMut<AudioAnalyzer>) {
    analyzer.process_audio_frame();
}