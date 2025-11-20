use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    pub easing: EasingFunction,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    Bezier(BezierCurve),
    CatmullRom(CatmullRomCurve),
    Hermite(HermiteCurve),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BezierCurve {
    pub control_point_1: [f32; 2],
    pub control_point_2: [f32; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatmullRomCurve {
    pub tension: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermiteCurve {
    pub tangent_1: f32,
    pub tangent_2: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationCurve {
    pub keyframes: Vec<Keyframe>,
    pub name: String,
    pub parameter_name: String,
    pub loop_type: LoopType,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopType {
    None,
    Repeat,
    PingPong,
    Clamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineTrack {
    pub name: String,
    pub target_parameter: String,
    pub curves: Vec<AnimationCurve>,
    pub enabled: bool,
    pub muted: bool,
    pub solo: bool,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineLayer {
    pub name: String,
    pub tracks: Vec<TimelineTrack>,
    pub enabled: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
    Difference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub name: String,
    pub duration: f32,
    pub current_time: f32,
    pub layers: Vec<TimelineLayer>,
    pub frame_rate: f32,
    pub time_signature: (u32, u32),
    pub tempo: f32,
    pub markers: Vec<TimelineMarker>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMarker {
    pub time: f32,
    pub name: String,
    pub color: [f32; 4],
    pub type_: MarkerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarkerType {
    Cue,
    LoopStart,
    LoopEnd,
    Beat,
    Measure,
    Section,
}

pub struct TimelineAnimationSystem {
    timelines: Arc<Mutex<HashMap<String, Timeline>>>,
    active_timeline: Arc<Mutex<String>>,
    is_playing: Arc<Mutex<bool>>,
    playback_speed: Arc<Mutex<f32>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    animation_parameters: Arc<Mutex<HashMap<String, f32>>>,
    parameter_history: Arc<Mutex<HashMap<String, VecDeque<f32>>>>,
    max_history_size: usize,
}

impl TimelineAnimationSystem {
    pub fn new() -> Self {
        Self {
            timelines: Arc::new(Mutex::new(HashMap::new())),
            active_timeline: Arc::new(Mutex::new("default".to_string())),
            is_playing: Arc::new(Mutex::new(false)),
            playback_speed: Arc::new(Mutex::new(1.0)),
            start_time: Arc::new(Mutex::new(None)),
            animation_parameters: Arc::new(Mutex::new(HashMap::new())),
            parameter_history: Arc::new(Mutex::new(HashMap::new())),
            max_history_size: 1000,
        }
    }

    pub fn create_timeline(&self, name: &str, duration: f32) -> Result<(), String> {
        let mut timelines = self.timelines.lock().unwrap();
        
        if timelines.contains_key(name) {
            return Err(format!("Timeline '{}' already exists", name));
        }

        let timeline = Timeline {
            name: name.to_string(),
            duration,
            current_time: 0.0,
            layers: Vec::new(),
            frame_rate: 60.0,
            time_signature: (4, 4),
            tempo: 120.0,
            markers: Vec::new(),
            enabled: true,
        };

        timelines.insert(name.to_string(), timeline);
        Ok(())
    }

    pub fn add_layer_to_timeline(&self, timeline_name: &str, layer: TimelineLayer) -> Result<(), String> {
        let mut timelines = self.timelines.lock().unwrap();
        
        if let Some(timeline) = timelines.get_mut(timeline_name) {
            timeline.layers.push(layer);
            Ok(())
        } else {
            Err(format!("Timeline '{}' not found", timeline_name))
        }
    }

    pub fn add_track_to_layer(&self, timeline_name: &str, layer_name: &str, track: TimelineTrack) -> Result<(), String> {
        let mut timelines = self.timelines.lock().unwrap();
        
        if let Some(timeline) = timelines.get_mut(timeline_name) {
            if let Some(layer) = timeline.layers.iter_mut().find(|l| l.name == layer_name) {
                layer.tracks.push(track);
                Ok(())
            } else {
                Err(format!("Layer '{}' not found in timeline '{}'", layer_name, timeline_name))
            }
        } else {
            Err(format!("Timeline '{}' not found", timeline_name))
        }
    }

    pub fn add_curve_to_track(&self, timeline_name: &str, layer_name: &str, track_name: &str, curve: AnimationCurve) -> Result<(), String> {
        let mut timelines = self.timelines.lock().unwrap();
        
        if let Some(timeline) = timelines.get_mut(timeline_name) {
            if let Some(layer) = timeline.layers.iter_mut().find(|l| l.name == layer_name) {
                if let Some(track) = layer.tracks.iter_mut().find(|t| t.name == track_name) {
                    track.curves.push(curve);
                    Ok(())
                } else {
                    Err(format!("Track '{}' not found in layer '{}'", track_name, layer_name))
                }
            } else {
                Err(format!("Layer '{}' not found in timeline '{}'", layer_name, timeline_name))
            }
        } else {
            Err(format!("Timeline '{}' not found", timeline_name))
        }
    }

    pub fn play(&self) {
        *self.is_playing.lock().unwrap() = true;
        *self.start_time.lock().unwrap() = Some(Instant::now());
    }

    pub fn pause(&self) {
        *self.is_playing.lock().unwrap() = false;
    }

    pub fn stop(&self) {
        *self.is_playing.lock().unwrap() = false;
        *self.start_time.lock().unwrap() = None;
        
        let active_timeline = self.active_timeline.lock().unwrap().clone();
        let mut timelines = self.timelines.lock().unwrap();
        
        if let Some(timeline) = timelines.get_mut(&active_timeline) {
            timeline.current_time = 0.0;
        }
    }

    pub fn set_playback_speed(&self, speed: f32) {
        *self.playback_speed.lock().unwrap() = speed;
    }

    pub fn seek_to_time(&self, time: f32) {
        let active_timeline = self.active_timeline.lock().unwrap().clone();
        let mut timelines = self.timelines.lock().unwrap();
        
        if let Some(timeline) = timelines.get_mut(&active_timeline) {
            timeline.current_time = time.clamp(0.0, timeline.duration);
        }
    }

    pub fn update(&self, delta_time: f32) {
        if !*self.is_playing.lock().unwrap() {
            return;
        }

        let playback_speed = *self.playback_speed.lock().unwrap();
        let active_timeline = self.active_timeline.lock().unwrap().clone();
        let mut timelines = self.timelines.lock().unwrap();
        
        if let Some(timeline) = timelines.get_mut(&active_timeline) {
            if !timeline.enabled {
                return;
            }

            let new_time = timeline.current_time + (delta_time * playback_speed);
            timeline.current_time = self.apply_loop_behavior(new_time, timeline.duration, &timeline.layers);

            // Update all animation parameters
            self.update_animation_parameters(timeline);
        }
    }

    fn apply_loop_behavior(&self, time: f32, duration: f32, layers: &[TimelineLayer]) -> f32 {
        if duration <= 0.0 {
            return 0.0;
        }

        // Find the most restrictive loop behavior from all curves
        let mut loop_type = LoopType::None;
        
        for layer in layers {
            if !layer.enabled {
                continue;
            }
            
            for track in &layer.tracks {
                if !track.enabled {
                    continue;
                }
                
                for curve in &track.curves {
                    match &curve.loop_type {
                        LoopType::None => {},
                        LoopType::Clamp => return time.min(duration),
                        LoopType::Repeat => loop_type = LoopType::Repeat,
                        LoopType::PingPong => loop_type = LoopType::PingPong,
                    }
                }
            }
        }

        match loop_type {
            LoopType::None => time.min(duration),
            LoopType::Repeat => time % duration,
            LoopType::PingPong => {
                let normalized_time = time % (duration * 2.0);
                if normalized_time <= duration {
                    normalized_time
                } else {
                    duration * 2.0 - normalized_time
                }
            },
            LoopType::Clamp => time.min(duration),
        }
    }

    fn update_animation_parameters(&self, timeline: &Timeline) {
        let mut parameters = self.animation_parameters.lock().unwrap();
        let mut parameter_history = self.parameter_history.lock().unwrap();

        for layer in &timeline.layers {
            if !layer.enabled {
                continue;
            }

            for track in &layer.tracks {
                if !track.enabled || track.muted {
                    continue;
                }

                for curve in &track.curves {
                    if let Some(value) = self.evaluate_curve_at_time(curve, timeline.current_time) {
                        let parameter_name = &curve.parameter_name;
                        
                        // Update current value
                        parameters.insert(parameter_name.clone(), value);
                        
                        // Update history
                        let history = parameter_history.entry(parameter_name.clone())
                            .or_insert_with(|| VecDeque::with_capacity(self.max_history_size));
                        
                        history.push_back(value);
                        if history.len() > self.max_history_size {
                            history.pop_front();
                        }
                    }
                }
            }
        }
    }

    fn evaluate_curve_at_time(&self, curve: &AnimationCurve, time: f32) -> Option<f32> {
        if curve.keyframes.is_empty() {
            return None;
        }

        if curve.keyframes.len() == 1 {
            return Some(curve.keyframes[0].value);
        }

        // Find the keyframes to interpolate between
        let mut prev_keyframe = &curve.keyframes[0];
        let mut next_keyframe = &curve.keyframes[curve.keyframes.len() - 1];

        for i in 0..curve.keyframes.len() - 1 {
            if time >= curve.keyframes[i].time && time <= curve.keyframes[i + 1].time {
                prev_keyframe = &curve.keyframes[i];
                next_keyframe = &curve.keyframes[i + 1];
                break;
            }
        }

        if time < prev_keyframe.time {
            return Some(prev_keyframe.value);
        }

        if time > next_keyframe.time {
            return Some(next_keyframe.value);
        }

        // Calculate interpolation factor
        let time_diff = next_keyframe.time - prev_keyframe.time;
        if time_diff <= 0.0 {
            return Some(prev_keyframe.value);
        }

        let t = (time - prev_keyframe.time) / time_diff;
        let eased_t = self.apply_easing(t, &prev_keyframe.easing);

        // Apply interpolation
        match &prev_keyframe.interpolation {
            InterpolationType::Linear => {
                Some(prev_keyframe.value + (next_keyframe.value - prev_keyframe.value) * eased_t)
            },
            InterpolationType::Bezier(bezier) => {
                let bezier_t = self.evaluate_bezier(t, bezier);
                Some(prev_keyframe.value + (next_keyframe.value - prev_keyframe.value) * bezier_t)
            },
            InterpolationType::CatmullRom(catmull_rom) => {
                self.evaluate_catmull_rom(&curve.keyframes, time, catmull_rom.tension)
            },
            InterpolationType::Hermite(hermite) => {
                self.evaluate_hermite(prev_keyframe, next_keyframe, t, hermite.tangent_1, hermite.tangent_2)
            },
        }
    }

    fn apply_easing(&self, t: f32, easing: &EasingFunction) -> f32 {
        match easing {
            EasingFunction::Linear => t,
            EasingFunction::EaseInQuad => t * t,
            EasingFunction::EaseOutQuad => t * (2.0 - t),
            EasingFunction::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            },
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => {
                let t1 = t - 1.0;
                t1 * t1 * t1 + 1.0
            },
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t1 = 2.0 * t - 2.0;
                    (t1 * t1 * t1 + 2.0) / 2.0
                }
            },
            EasingFunction::EaseInSine => {
                1.0 - (t * std::f32::consts::PI / 2.0).cos()
            },
            EasingFunction::EaseOutSine => {
                (t * std::f32::consts::PI / 2.0).sin()
            },
            EasingFunction::EaseInOutSine => {
                -(std::f32::consts::PI * t).cos() / 2.0 + 0.5
            },
            EasingFunction::EaseInExpo => {
                if t == 0.0 { 0.0 } else { 2.0_f32.powf(10.0 * (t - 1.0)) }
            },
            EasingFunction::EaseOutExpo => {
                if t == 1.0 { 1.0 } else { 1.0 - 2.0_f32.powf(-10.0 * t) }
            },
            EasingFunction::EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    2.0_f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            },
            EasingFunction::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            },
            EasingFunction::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                let t1 = t - 1.0;
                1.0 + c3 * t1 * t1 * t1 + c1 * t1 * t1
            },
            EasingFunction::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            },
            _ => t, // Fallback to linear for unimplemented easing functions
        }
    }

    fn evaluate_bezier(&self, t: f32, bezier: &BezierCurve) -> f32 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        let p = uuu * 0.0; // p0 is always 0
        let p = p + 3.0 * uu * t * bezier.control_point_1[1];
        let p = p + 3.0 * u * tt * bezier.control_point_2[1];
        let p = p + ttt * 1.0; // p3 is always 1

        p
    }

    fn evaluate_catmull_rom(&self, keyframes: &[Keyframe], time: f32, tension: f32) -> Option<f32> {
        if keyframes.len() < 2 {
            return None;
        }

        // Find the segment containing the time
        for i in 0..keyframes.len() - 1 {
            if time >= keyframes[i].time && time <= keyframes[i + 1].time {
                let p0 = if i > 0 { keyframes[i - 1].value } else { keyframes[i].value };
                let p1 = keyframes[i].value;
                let p2 = keyframes[i + 1].value;
                let p3 = if i + 2 < keyframes.len() { keyframes[i + 2].value } else { keyframes[i + 1].value };

                let t = (time - keyframes[i].time) / (keyframes[i + 1].time - keyframes[i].time);
                
                let t2 = t * t;
                let t3 = t2 * t;

                let result = 0.5 * (
                    (2.0 * p1) +
                    (-p0 + p2) * t +
                    (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2 +
                    (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
                );

                return Some(result);
            }
        }

        None
    }

    fn evaluate_hermite(&self, prev: &Keyframe, next: &Keyframe, t: f32, tangent_1: f32, tangent_2: f32) -> Option<f32> {
        let h00 = 2.0 * t * t * t - 3.0 * t * t + 1.0;
        let h10 = t * t * t - 2.0 * t * t + t;
        let h01 = -2.0 * t * t * t + 3.0 * t * t;
        let h11 = t * t * t - t * t;

        let duration = next.time - prev.time;
        let result = h00 * prev.value + h10 * duration * tangent_1 + h01 * next.value + h11 * duration * tangent_2;

        Some(result)
    }

    pub fn get_current_time(&self) -> f32 {
        let active_timeline = self.active_timeline.lock().unwrap().clone();
        let timelines = self.timelines.lock().unwrap();
        
        if let Some(timeline) = timelines.get(&active_timeline) {
            timeline.current_time
        } else {
            0.0
        }
    }

    pub fn get_animation_parameter(&self, name: &str) -> Option<f32> {
        self.animation_parameters.lock().unwrap().get(name).copied()
    }

    pub fn get_parameter_history(&self, name: &str) -> Vec<f32> {
        self.parameter_history.lock().unwrap()
            .get(name)
            .map(|history| history.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock().unwrap()
    }

    pub fn get_active_timeline_name(&self) -> String {
        self.active_timeline.lock().unwrap().clone()
    }

    pub fn set_active_timeline(&self, name: &str) -> Result<(), String> {
        let timelines = self.timelines.lock().unwrap();
        
        if timelines.contains_key(name) {
            *self.active_timeline.lock().unwrap() = name.to_string();
            Ok(())
        } else {
            Err(format!("Timeline '{}' not found", name))
        }
    }

    pub fn export_timeline(&self, name: &str) -> Result<String, String> {
        let timelines = self.timelines.lock().unwrap();
        
        if let Some(timeline) = timelines.get(name) {
            serde_json::to_string_pretty(timeline)
                .map_err(|e| format!("Failed to serialize timeline: {}", e))
        } else {
            Err(format!("Timeline '{}' not found", name))
        }
    }

    pub fn import_timeline(&self, json_data: &str) -> Result<(), String> {
        let timeline: Timeline = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to deserialize timeline: {}", e))?;
        
        let name = timeline.name.clone();
        self.timelines.lock().unwrap().insert(name, timeline);
        Ok(())
    }

    pub fn get_shader_uniforms(&self) -> TimelineShaderUniforms {
        let parameters = self.animation_parameters.lock().unwrap();
        let current_time = self.get_current_time();
        let is_playing = self.is_playing();
        
        TimelineShaderUniforms {
            u_timeline_time: current_time,
            u_timeline_progress: if let Some(timeline) = self.timelines.lock().unwrap().get(&self.get_active_timeline_name()) {
                if timeline.duration > 0.0 {
                    current_time / timeline.duration
                } else {
                    0.0
                }
            } else {
                0.0
            },
            u_timeline_playing: if is_playing { 1.0 } else { 0.0 },
            u_timeline_beat: self.time_to_beat(current_time),
            u_timeline_measure: self.time_to_measure(current_time),
            u_timeline_tempo: if let Some(timeline) = self.timelines.lock().unwrap().get(&self.get_active_timeline_name()) {
                timeline.tempo
            } else {
                120.0
            },
            u_animation_params: parameters.clone(),
        }
    }

    fn time_to_beat(&self, time: f32) -> f32 {
        if let Some(timeline) = self.timelines.lock().unwrap().get(&self.get_active_timeline_name()) {
            let beats_per_second = timeline.tempo / 60.0;
            time * beats_per_second
        } else {
            0.0
        }
    }

    fn time_to_measure(&self, time: f32) -> f32 {
        if let Some(timeline) = self.timelines.lock().unwrap().get(&self.get_active_timeline_name()) {
            let (numerator, _denominator) = timeline.time_signature;
            let beats_per_second = timeline.tempo / 60.0;
            let beats = time * beats_per_second;
            beats / numerator as f32
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimelineShaderUniforms {
    pub u_timeline_time: f32,
    pub u_timeline_progress: f32,
    pub u_timeline_playing: f32,
    pub u_timeline_beat: f32,
    pub u_timeline_measure: f32,
    pub u_timeline_tempo: f32,
    pub u_animation_params: HashMap<String, f32>,
}

impl TimelineShaderUniforms {
    pub fn to_wgsl_struct(&self) -> String {
        r#"
struct TimelineData {
    time: f32,
    progress: f32,
    playing: f32,
    beat: f32,
    measure: f32,
    tempo: f32,
}
"#.to_string()
    }

    pub fn to_wgsl_uniforms(&self) -> String {
        let mut uniforms = String::new();
        
        uniforms.push_str(r#"
@group(3) @binding(0) var<uniform> timeline_time: f32;
@group(3) @binding(1) var<uniform> timeline_progress: f32;
@group(3) @binding(2) var<uniform> timeline_playing: f32;
@group(3) @binding(3) var<uniform> timeline_beat: f32;
@group(3) @binding(4) var<uniform> timeline_measure: f32;
@group(3) @binding(5) var<uniform> timeline_tempo: f32;
"#);

        // Add animation parameters as individual uniforms
        for (i, (param_name, _value)) in self.u_animation_params.iter().enumerate() {
            let binding_index = 6 + i;
            uniforms.push_str(&format!(
                "@group(3) @binding({}) var<uniform> {}: f32;\n",
                binding_index, param_name
            ));
        }

        uniforms
    }
}

// Utility functions for creating common animation curves
pub fn create_linear_curve(name: &str, parameter_name: &str, start_value: f32, end_value: f32, duration: f32) -> AnimationCurve {
    AnimationCurve {
        name: name.to_string(),
        parameter_name: parameter_name.to_string(),
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: start_value,
                easing: EasingFunction::Linear,
                interpolation: InterpolationType::Linear,
            },
            Keyframe {
                time: duration,
                value: end_value,
                easing: EasingFunction::Linear,
                interpolation: InterpolationType::Linear,
            },
        ],
        loop_type: LoopType::None,
        duration,
    }
}

pub fn create_ease_in_out_curve(name: &str, parameter_name: &str, start_value: f32, end_value: f32, duration: f32) -> AnimationCurve {
    AnimationCurve {
        name: name.to_string(),
        parameter_name: parameter_name.to_string(),
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: start_value,
                easing: EasingFunction::EaseInOutCubic,
                interpolation: InterpolationType::Linear,
            },
            Keyframe {
                time: duration,
                value: end_value,
                easing: EasingFunction::EaseInOutCubic,
                interpolation: InterpolationType::Linear,
            },
        ],
        loop_type: LoopType::None,
        duration,
    }
}

pub fn create_bounce_curve(name: &str, parameter_name: &str, start_value: f32, end_value: f32, duration: f32) -> AnimationCurve {
    AnimationCurve {
        name: name.to_string(),
        parameter_name: parameter_name.to_string(),
        keyframes: vec![
            Keyframe {
                time: 0.0,
                value: start_value,
                easing: EasingFunction::EaseOutBounce,
                interpolation: InterpolationType::Linear,
            },
            Keyframe {
                time: duration,
                value: end_value,
                easing: EasingFunction::EaseOutBounce,
                interpolation: InterpolationType::Linear,
            },
        ],
        loop_type: LoopType::None,
        duration,
    }
}

pub fn create_oscillation_curve(name: &str, parameter_name: &str, amplitude: f32, frequency: f32, duration: f32) -> AnimationCurve {
    let mut keyframes = Vec::new();
    let samples = (duration * frequency * 4.0) as usize; // 4 samples per cycle
    
    for i in 0..=samples {
        let time = (i as f32 / samples as f32) * duration;
        let value = amplitude * (time * frequency * 2.0 * std::f32::consts::PI).sin();
        
        keyframes.push(Keyframe {
            time,
            value,
            easing: EasingFunction::Linear,
            interpolation: InterpolationType::Linear,
        });
    }

    AnimationCurve {
        name: name.to_string(),
        parameter_name: parameter_name.to_string(),
        keyframes,
        loop_type: LoopType::Repeat,
        duration,
    }
}