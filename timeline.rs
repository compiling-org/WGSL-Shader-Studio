use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Timeline animation system for shader parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub tracks: Vec<Track>,
    pub duration: f32,
    pub current_time: f32,
    pub playing: bool,
    pub loop_enabled: bool,
    pub playback_speed: f32,
    start_time: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub parameter_path: String,
    pub keyframes: Vec<Keyframe>,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    pub easing: EasingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    Bezier,
    Step,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            duration: 10.0,
            current_time: 0.0,
            playing: false,
            loop_enabled: true,
            playback_speed: 1.0,
            start_time: None,
        }
    }

    pub fn add_track(&mut self, name: String, parameter_path: String) -> usize {
        let track = Track {
            name,
            parameter_path,
            keyframes: Vec::new(),
            interpolation: InterpolationType::Linear,
        };
        self.tracks.push(track);
        self.tracks.len() - 1
    }

    pub fn add_keyframe(&mut self, track_index: usize, time: f32, value: f32) -> Result<()> {
        if track_index >= self.tracks.len() {
            return Err(anyhow::anyhow!("Track index out of bounds"));
        }

        let keyframe = Keyframe {
            time,
            value,
            easing: EasingType::Linear,
        };

        self.tracks[track_index].keyframes.push(keyframe);
        self.tracks[track_index].keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        Ok(())
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.start_time = Some(Instant::now());
    }

    pub fn pause(&mut self) {
        self.playing = false;
        self.start_time = None;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
        self.start_time = None;
    }

    pub fn seek(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, self.duration);
    }

    pub fn update(&mut self) {
        if !self.playing {
            return;
        }

        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed().as_secs_f32() * self.playback_speed;
            self.current_time += elapsed;

            if self.current_time >= self.duration {
                if self.loop_enabled {
                    self.current_time = self.current_time % self.duration;
                    self.start_time = Some(Instant::now());
                } else {
                    self.current_time = self.duration;
                    self.playing = false;
                    self.start_time = None;
                }
            } else {
                self.start_time = Some(Instant::now());
            }
        }
    }

    pub fn get_parameter_values(&self) -> HashMap<String, f32> {
        let mut values = HashMap::new();

        for track in &self.tracks {
            if let Some(value) = self.interpolate_track(track) {
                values.insert(track.parameter_path.clone(), value);
            }
        }

        values
    }

    fn interpolate_track(&self, track: &Track) -> Option<f32> {
        if track.keyframes.is_empty() {
            return None;
        }

        // Find keyframes to interpolate between
        let mut prev_keyframe = &track.keyframes[0];
        let mut next_keyframe = &track.keyframes[0];

        for i in 0..track.keyframes.len() - 1 {
            if self.current_time >= track.keyframes[i].time && self.current_time <= track.keyframes[i + 1].time {
                prev_keyframe = &track.keyframes[i];
                next_keyframe = &track.keyframes[i + 1];
                break;
            }
        }

        // If we're past the last keyframe, use its value
        if self.current_time > track.keyframes.last().unwrap().time {
            return Some(track.keyframes.last().unwrap().value);
        }

        // If we're before the first keyframe, use its value
        if self.current_time < track.keyframes.first().unwrap().time {
            return Some(track.keyframes.first().unwrap().value);
        }

        // Interpolate between keyframes
        let time_diff = next_keyframe.time - prev_keyframe.time;
        if time_diff == 0.0 {
            return Some(prev_keyframe.value);
        }

        let t = (self.current_time - prev_keyframe.time) / time_diff;
        let eased_t = self.apply_easing(t, &prev_keyframe.easing);

        match track.interpolation {
            InterpolationType::Linear => {
                Some(prev_keyframe.value + (next_keyframe.value - prev_keyframe.value) * eased_t)
            }
            InterpolationType::Step => {
                Some(if t < 0.5 { prev_keyframe.value } else { next_keyframe.value })
            }
            InterpolationType::Bezier => {
                // Simple cubic bezier interpolation
                let t2 = eased_t * eased_t;
                let t3 = t2 * eased_t;
                Some(prev_keyframe.value + (next_keyframe.value - prev_keyframe.value) * (3.0 * t2 - 2.0 * t3))
            }
        }
    }

    fn apply_easing(&self, t: f32, easing: &EasingType) -> f32 {
        match easing {
            EasingType::Linear => t,
            EasingType::EaseIn => t * t,
            EasingType::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingType::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            EasingType::CubicIn => t * t * t,
            EasingType::CubicOut => 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t),
            EasingType::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - 4.0 * (1.0 - t) * (1.0 - t) * (1.0 - t)
                }
            }
        }
    }

    pub fn export_to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize timeline: {}", e))
    }

    pub fn import_from_json(json_str: &str) -> Result<Self> {
        serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize timeline: {}", e))
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Timeline UI controller
pub struct TimelineController {
    timeline: Timeline,
    selected_track: Option<usize>,
    selected_keyframe: Option<(usize, usize)>,
}

impl TimelineController {
    pub fn new() -> Self {
        Self {
            timeline: Timeline::new(),
            selected_track: None,
            selected_keyframe: None,
        }
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    pub fn select_track(&mut self, track_index: Option<usize>) {
        self.selected_track = track_index;
        if track_index.is_none() {
            self.selected_keyframe = None;
        }
    }

    pub fn select_keyframe(&mut self, track_index: Option<usize>, keyframe_index: Option<usize>) {
        self.selected_keyframe = match (track_index, keyframe_index) {
            (Some(track), Some(keyframe)) => Some((track, keyframe)),
            _ => None,
        };
    }

    pub fn get_selected_track(&self) -> Option<usize> {
        self.selected_track
    }

    pub fn get_selected_keyframe(&self) -> Option<(usize, usize)> {
        self.selected_keyframe
    }

    pub fn delete_selected_keyframe(&mut self) -> Result<()> {
        if let Some((track_idx, keyframe_idx)) = self.selected_keyframe {
            if track_idx < self.timeline.tracks.len() {
                let track = &mut self.timeline.tracks[track_idx];
                if keyframe_idx < track.keyframes.len() {
                    track.keyframes.remove(keyframe_idx);
                    self.selected_keyframe = None;
                    return Ok(());
                }
            }
        }
        Err(anyhow::anyhow!("No keyframe selected"))
    }

    pub fn duplicate_selected_keyframe(&mut self) -> Result<()> {
        if let Some((track_idx, keyframe_idx)) = self.selected_keyframe {
            if track_idx < self.timeline.tracks.len() {
                let track = &self.timeline.tracks[track_idx];
                if keyframe_idx < track.keyframes.len() {
                    let keyframe = track.keyframes[keyframe_idx].clone();
                    let new_time = keyframe.time + 0.1;
                    
                    self.timeline.add_keyframe(track_idx, new_time, keyframe.value)?;
                    return Ok(());
                }
            }
        }
        Err(anyhow::anyhow!("No keyframe selected"))
    }
}

impl Default for TimelineController {
    fn default() -> Self {
        Self::new()
    }
}