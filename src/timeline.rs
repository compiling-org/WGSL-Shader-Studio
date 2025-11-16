use serde::{Deserialize, Serialize};
use bevy::prelude::Resource;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Step,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineTrack {
    pub parameter_name: String,
    pub keyframes: Vec<Keyframe>,
    pub enabled: bool,
    pub color: [f32; 4], // RGBA for UI visualization
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub tracks: HashMap<String, TimelineTrack>,
    pub current_time: f32,
    pub duration: f32,
    pub playback_state: PlaybackState,
    pub playback_speed: f32,
    pub loop_enabled: bool,
    pub loop_start: f32,
    pub loop_end: f32,
    pub snap_to_grid: bool,
    pub grid_division: f32, // seconds per grid division
    pub selected_track: Option<String>,
    pub selected_keyframe: Option<(String, usize)>, // (track_name, keyframe_index)
    
    // Internal timing
    #[serde(skip)]
    pub last_update: Option<Instant>,
    #[serde(skip)]
    pub playback_start_time: Option<f32>,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            tracks: HashMap::new(),
            current_time: 0.0,
            duration: 60.0, // 60 seconds default
            playback_state: PlaybackState::Stopped,
            playback_speed: 1.0,
            loop_enabled: false,
            loop_start: 0.0,
            loop_end: 60.0,
            snap_to_grid: true,
            grid_division: 0.1, // 100ms grid
            selected_track: None,
            selected_keyframe: None,
            last_update: None,
            playback_start_time: None,
        }
    }
}

impl Timeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_track(&mut self, parameter_name: String, color: [f32; 4]) {
        let track = TimelineTrack {
            parameter_name: parameter_name.clone(),
            keyframes: Vec::new(),
            enabled: true,
            color,
        };
        self.tracks.insert(parameter_name, track);
    }

    pub fn add_keyframe(&mut self, parameter_name: &str, time: f32, value: f32, interpolation: InterpolationType) {
        if !self.tracks.contains_key(parameter_name) {
            // Create track with a default color if it doesn't exist
            self.create_track(parameter_name.to_string(), [0.2, 0.6, 1.0, 1.0]);
        }

        if let Some(track) = self.tracks.get_mut(parameter_name) {
            let keyframe = Keyframe { time, value, interpolation };
            
            // Find insertion point to maintain sorted order
            let insert_pos = track.keyframes.binary_search_by(|k| k.time.partial_cmp(&time).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or_else(|pos| pos);
            
            track.keyframes.insert(insert_pos, keyframe);
        }
    }

    pub fn remove_keyframe(&mut self, parameter_name: &str, keyframe_index: usize) -> bool {
        if let Some(track) = self.tracks.get_mut(parameter_name) {
            if keyframe_index < track.keyframes.len() {
                track.keyframes.remove(keyframe_index);
                return true;
            }
        }
        false
    }

    pub fn move_keyframe(&mut self, parameter_name: &str, keyframe_index: usize, new_time: f32, new_value: f32) -> bool {
        if let Some(track) = self.tracks.get_mut(parameter_name) {
            if keyframe_index < track.keyframes.len() {
                let keyframe = &mut track.keyframes[keyframe_index];
                keyframe.time = new_time;
                keyframe.value = new_value;
                
                // Re-sort keyframes to maintain order
                track.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
                return true;
            }
        }
        false
    }

    pub fn evaluate(&self, parameter_name: &str, time: f32, default: f32) -> f32 {
        let Some(track) = self.tracks.get(parameter_name) else { return default; };
        if !track.enabled || track.keyframes.is_empty() { return default; }

        // Find bracketing keyframes
        let mut prev = None;
        let mut next = None;
        
        for (i, keyframe) in track.keyframes.iter().enumerate() {
            if keyframe.time <= time {
                prev = Some((i, keyframe.clone()));
            } else {
                next = Some((i, keyframe.clone()));
                break;
            }
        }

        match (prev, next) {
            (Some((_, p)), Some((_, n))) => {
                let t = ((time - p.time) / (n.time - p.time)).clamp(0.0, 1.0);
                self.interpolate_values(p.value, n.value, t, p.interpolation)
            }
            (Some((_, p)), None) => p.value,
            (None, Some((_, n))) => n.value,
            _ => default,
        }
    }

    fn interpolate_values(&self, start: f32, end: f32, t: f32, interpolation: InterpolationType) -> f32 {
        let t = t.clamp(0.0, 1.0);
        
        match interpolation {
            InterpolationType::Linear => start * (1.0 - t) + end * t,
            InterpolationType::EaseIn => {
                let eased = t * t;
                start * (1.0 - eased) + end * eased
            }
            InterpolationType::EaseOut => {
                let eased = 1.0 - (1.0 - t) * (1.0 - t);
                start * (1.0 - eased) + end * eased
            }
            InterpolationType::EaseInOut => {
                let eased = if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                };
                start * (1.0 - eased) + end * eased
            }
            InterpolationType::Step => if t < 0.5 { start } else { end },
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        if self.playback_state != PlaybackState::Playing {
            return;
        }

        let delta_seconds = delta_time.as_secs_f32() * self.playback_speed;
        self.current_time += delta_seconds;

        // Handle looping
        if self.loop_enabled {
            if self.current_time >= self.loop_end {
                self.current_time = self.loop_start;
            }
        } else if self.current_time >= self.duration {
            self.current_time = self.duration;
            self.playback_state = PlaybackState::Stopped;
        }
    }

    pub fn play(&mut self) {
        self.playback_state = PlaybackState::Playing;
        self.last_update = Some(Instant::now());
    }

    pub fn pause(&mut self) {
        self.playback_state = PlaybackState::Paused;
    }

    pub fn stop(&mut self) {
        self.playback_state = PlaybackState::Stopped;
        self.current_time = 0.0;
    }

    pub fn seek(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, self.duration);
    }

    pub fn get_all_parameters(&self) -> Vec<&str> {
        self.tracks.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_parameter_at_time(&self, parameter_name: &str, time: f32) -> Option<f32> {
        self.tracks.get(parameter_name)
            .filter(|track| track.enabled)
            .map(|_| self.evaluate(parameter_name, time, 0.0))
    }

    pub fn get_keyframe_at_time(&self, parameter_name: &str, time: f32, tolerance: f32) -> Option<usize> {
        self.tracks.get(parameter_name).and_then(|track| {
            track.keyframes.iter().position(|kf| (kf.time - time).abs() < tolerance)
        })
    }

    pub fn snap_time_to_grid(&self, time: f32) -> f32 {
        if self.snap_to_grid {
            (time / self.grid_division).round() * self.grid_division
        } else {
            time
        }
    }

    pub fn clear_track(&mut self, parameter_name: &str) {
        if let Some(track) = self.tracks.get_mut(parameter_name) {
            track.keyframes.clear();
        }
    }

    pub fn clear_all_tracks(&mut self) {
        for track in self.tracks.values_mut() {
            track.keyframes.clear();
        }
    }

    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn import_from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// Bevy plugin for timeline animation system
use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct TimelineAnimation {
    pub timeline: Timeline,
}

impl Default for TimelineAnimation {
    fn default() -> Self {
        Self {
            timeline: Timeline::new(),
        }
    }
}

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(TimelineAnimation::default())
            .add_systems(Update, update_timeline_animation);
    }
}

fn update_timeline_animation(
    mut timeline: ResMut<TimelineAnimation>,
    time: Res<Time>,
) {
    if timeline.timeline.playback_state == PlaybackState::Playing {
        timeline.timeline.update(time.delta());
    }
}