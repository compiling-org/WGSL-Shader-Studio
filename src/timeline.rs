use serde::{Deserialize, Serialize};
use bevy::prelude::Resource;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Shader parameter structure for timeline animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderParameter {
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub binding: u32,
    pub group: u32,
}

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
            track.keyframes.push(keyframe);
            // Sort keyframes by time
            track.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
    }
    
    /// Get keyframes for a parameter
    pub fn get_keyframes(&self, parameter_name: &str) -> Option<&Vec<Keyframe>> {
        self.tracks.get(parameter_name).map(|track| &track.keyframes)
    }
    
    /// Get all keyframes as a map of parameter names to keyframes
    pub fn get_all_keyframes(&self) -> HashMap<String, &Vec<Keyframe>> {
        self.tracks.iter()
            .filter(|(_, track)| !track.keyframes.is_empty())
            .map(|(name, track)| (name.clone(), &track.keyframes))
            .collect()
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
    
    pub fn toggle_track(&mut self, parameter_name: &str) -> bool {
        if let Some(track) = self.tracks.get_mut(parameter_name) {
            track.enabled = !track.enabled;
            track.enabled
        } else {
            false
        }
    }
    
    pub fn set_track_enabled(&mut self, parameter_name: &str, enabled: bool) -> bool {
        if let Some(track) = self.tracks.get_mut(parameter_name) {
            track.enabled = enabled;
            true
        } else {
            false
        }
    }
    
    pub fn get_track_enabled(&self, parameter_name: &str) -> bool {
        self.tracks.get(parameter_name)
            .map(|track| track.enabled)
            .unwrap_or(false)
    }

    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn import_from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Apply timeline animation to shader parameters
    pub fn apply_to_parameters(&self, parameters: &mut [ShaderParameter]) {
        for param in parameters.iter_mut() {
            if let Some(animated_value) = self.get_parameter_at_time(&param.name, self.current_time) {
                param.value = animated_value;
            }
        }
    }
}

// Bevy plugin for timeline animation system
use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct TimelineAnimation {
    pub timeline: Timeline,
    pub playing: bool, // Convenience field for UI binding
}

impl Default for TimelineAnimation {
    fn default() -> Self {
        Self {
            timeline: Timeline::new(),
            playing: false,
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

// UI Functions for Timeline
use bevy_egui::egui::{Color32, Response, RichText, Ui};

pub fn draw_timeline_ui(ui: &mut Ui, timeline: &mut TimelineAnimation) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Timeline Animation").strong());
            
            // Playback controls
            let play_text = match timeline.timeline.playback_state {
                PlaybackState::Playing => "⏸ Pause",
                PlaybackState::Paused => "▶ Resume",
                PlaybackState::Stopped => "▶ Play",
            };
            
            if ui.button(play_text).clicked() {
                match timeline.timeline.playback_state {
                    PlaybackState::Playing => timeline.timeline.pause(),
                    PlaybackState::Paused | PlaybackState::Stopped => timeline.timeline.play(),
                }
                timeline.playing = timeline.timeline.playback_state == PlaybackState::Playing;
            }
            
            if ui.button("⏹ Stop").clicked() {
                timeline.timeline.stop();
                timeline.playing = false;
            }
            
            // Time display
            ui.separator();
            ui.label(format!("Time: {:.2}s / {:.2}s", 
                timeline.timeline.current_time, 
                timeline.timeline.duration
            ));
            
            // Speed control
            ui.separator();
            ui.label("Speed:");
            ui.add(bevy_egui::egui::DragValue::new(&mut timeline.timeline.playback_speed)
                .speed(0.1)
                .range(0.1..=5.0)
                .prefix("x"));
        });
        
        ui.separator();
        
        // Timeline ruler and tracks
        ui.vertical(|ui| {
            let timeline_width = ui.available_width() - 20.0;
            let timeline_height = 200.0;
            let ruler_height = 30.0;
            let track_height = 40.0;
            
            // Timeline ruler
            ui.horizontal(|ui| {
                ui.label("Time:");
                ui.allocate_ui(bevy_egui::egui::vec2(timeline_width, ruler_height), |ui| {
                    let painter = ui.painter();
                    let rect = bevy_egui::egui::Rect::from_min_size(ui.cursor().min, bevy_egui::egui::vec2(timeline_width, ruler_height));
                    
                    // Draw ruler background
                    painter.rect_filled(rect, bevy_egui::egui::CornerRadius::same(0u8), Color32::from_gray(40));
                    
                    // Draw time markers
                    let time_per_pixel = timeline.timeline.duration / timeline_width;
                    let marker_interval = if timeline.timeline.duration > 10.0 { 1.0 } else { 0.5 };
                    
                    let mut time = 0.0;
                    while time <= timeline.timeline.duration {
                        let x = rect.left() + (time / time_per_pixel);
                        if x <= rect.right() {
                            painter.line_segment(
                                [bevy_egui::egui::pos2(x, rect.top()), bevy_egui::egui::pos2(x, rect.bottom())],
                                (1.0, Color32::from_gray(100))
                            );
                            
                            painter.text(
                                bevy_egui::egui::pos2(x + 2.0, rect.top() + 2.0),
                                bevy_egui::egui::Align2::LEFT_TOP,
                                format!("{:.1}s", time),
                                bevy_egui::egui::FontId::monospace(10.0),
                                Color32::WHITE,
                            );
                        }
                        time += marker_interval;
                    }
                    
                    // Draw current time indicator
                    let current_x = rect.left() + (timeline.timeline.current_time / time_per_pixel);
                    painter.line_segment(
                        [bevy_egui::egui::pos2(current_x, rect.top()), bevy_egui::egui::pos2(current_x, rect.bottom() + track_height * timeline.timeline.tracks.len() as f32)],
                        (2.0, Color32::from_rgb(255, 100, 100))
                    );
                });
            });
            
            // Track list
            ui.separator();
            
            // Create a vector of track names to avoid borrowing issues
            let track_names: Vec<String> = timeline.timeline.tracks.keys().cloned().collect();
            
            for track_name in &track_names {
                let (track_enabled_snapshot, track_color_snapshot, keyframes_snapshot) = if let Some(track) = timeline.timeline.tracks.get(track_name) {
                    (track.enabled, track.color, track.keyframes.clone())
                } else {
                    (true, [0.2, 0.2, 0.2, 1.0], Vec::new())
                };
                ui.horizontal(|ui| {
                    // Track header - get mutable access to the track
                    ui.vertical(|ui| {
                        if let Some(track) = timeline.timeline.tracks.get_mut(track_name) {
                            ui.checkbox(&mut track.enabled, "");
                            ui.label(track_name);
                        } else {
                            ui.label(format!("{} (error)", track_name));
                        }
                    });
                    
                    let track_enabled = track_enabled_snapshot;
                    
                    ui.allocate_ui(bevy_egui::egui::vec2(timeline_width - 60.0, track_height), |ui| {
                        ui.scope(|ui| {
                            let painter = ui.painter();
                            let rect = bevy_egui::egui::Rect::from_min_size(ui.cursor().min, bevy_egui::egui::vec2(timeline_width - 60.0, track_height));
                            
                            // Draw track background with enabled/disabled state
                            let bg_color = if track_enabled {
                                Color32::from_gray(30)
                            } else {
                                Color32::from_gray(15) // Darker when disabled
                            };
                            painter.rect_filled(rect, bevy_egui::egui::CornerRadius::same(2u8), bg_color);
                            
                            // Draw track color indicator (dimmed when disabled)
                            let color_alpha = if track_enabled { track_color_snapshot[3] } else { track_color_snapshot[3] * 0.3 };
                            let color_rect = bevy_egui::egui::Rect::from_min_max(
                                rect.left_top(),
                                bevy_egui::egui::pos2(rect.left() + 4.0, rect.bottom())
                            );
                            painter.rect_filled(color_rect, bevy_egui::egui::CornerRadius::same(0u8), Color32::from_rgba_unmultiplied(
                                (track_color_snapshot[0] * 255.0) as u8,
                                (track_color_snapshot[1] * 255.0) as u8,
                                (track_color_snapshot[2] * 255.0) as u8,
                                (color_alpha * 255.0) as u8
                            ));
                            
                            // Draw keyframes (dimmed when disabled)
                            if track_enabled {
                                let time_per_pixel = timeline.timeline.duration / (timeline_width - 60.0);
                                for keyframe in keyframes_snapshot.iter() {
                                    let x = rect.left() + 10.0 + (keyframe.time / time_per_pixel);
                                    let y = rect.center().y;
                                    
                                    let keyframe_color = match keyframe.interpolation {
                                        InterpolationType::Linear => Color32::from_rgb(100, 200, 255),
                                        InterpolationType::EaseIn => Color32::from_rgb(255, 200, 100),
                                        InterpolationType::EaseOut => Color32::from_rgb(200, 255, 100),
                                        InterpolationType::EaseInOut => Color32::from_rgb(255, 100, 200),
                                        InterpolationType::Step => Color32::from_rgb(200, 100, 255),
                                    };
                                    
                                    painter.circle_filled(bevy_egui::egui::pos2(x, y), 4.0, keyframe_color);
                                    painter.circle_stroke(bevy_egui::egui::pos2(x, y), 5.0, (1.0, Color32::WHITE));
                                }
                            }
                        });
                        
                        // Keyframe tooltips (only when enabled and outside painter scope)
                        if track_enabled {
                            let time_per_pixel = timeline.timeline.duration / (timeline_width - 60.0);
                            for keyframe in keyframes_snapshot.iter() {
                                let x = ui.cursor().min.x + 10.0 + (keyframe.time / time_per_pixel);
                                let y = ui.cursor().min.y + track_height / 2.0;
                                
                                let _keyframe_rect = bevy_egui::egui::Rect::from_center_size(bevy_egui::egui::pos2(x, y), bevy_egui::egui::vec2(10.0, 10.0));
                                // Tooltip temporarily disabled for compilation
                                // if ui.rect_contains_pointer(keyframe_rect) {
                                //     bevy_egui::egui::show_tooltip_at_pointer(ui.ctx(), bevy_egui::egui::Id::new(format!("keyframe_tooltip_{}", track_name)), |ui| {
                                //         ui.label(format!("Time: {:.2}s\nValue: {:.3}", keyframe.time, keyframe.value));
                                //     });
                                // }
                            }
                        }
                    });
                });
            }
            
            if timeline.timeline.tracks.is_empty() {
                ui.label("No tracks added yet. Add keyframes to create tracks.");
            }
        });
        
        ui.separator();
        
        // Controls
        ui.horizontal(|ui| {
            // Loop controls
            ui.checkbox(&mut timeline.timeline.loop_enabled, "Loop");
            if timeline.timeline.loop_enabled {
                ui.label("Start:");
                ui.add(bevy_egui::egui::DragValue::new(&mut timeline.timeline.loop_start)
                    .speed(0.1)
                    .range(0.0..=timeline.timeline.loop_end));
                ui.label("End:");
                ui.add(bevy_egui::egui::DragValue::new(&mut timeline.timeline.loop_end)
                    .speed(0.1)
                    .range(timeline.timeline.loop_start..=timeline.timeline.duration));
            }
            
            ui.separator();
            
            // Grid controls
            ui.checkbox(&mut timeline.timeline.snap_to_grid, "Snap to Grid");
            if timeline.timeline.snap_to_grid {
                ui.label("Division:");
                ui.add(bevy_egui::egui::DragValue::new(&mut timeline.timeline.grid_division)
                    .speed(0.01)
                    .range(0.01..=1.0)
                    .suffix("s"));
            }
        });
        
        ui.separator();
        
        // Track management
        ui.horizontal(|ui| {
            if ui.button("Add Track").clicked() {
                let track_name = format!("Parameter_{}", timeline.timeline.tracks.len() + 1);
                let color = [
                    (timeline.timeline.tracks.len() as f32 * 0.3) % 1.0,
                    (timeline.timeline.tracks.len() as f32 * 0.7) % 1.0,
                    (timeline.timeline.tracks.len() as f32 * 0.9) % 1.0,
                    1.0
                ];
                timeline.timeline.create_track(track_name, color);
            }
            
            if ui.button("Clear All Tracks").clicked() {
                timeline.timeline.clear_all_tracks();
            }
            
            if ui.button("Export Timeline").clicked() {
                if let Ok(json) = timeline.timeline.export_to_json() {
                    // ui.output_mut(|o| o.copied_text = json); // TODO: Fix PlatformOutput field access
                    println!("Timeline exported to clipboard");
                }
            }
            
            if ui.button("Import Timeline").clicked() {
                // This would need file dialog integration
                println!("Timeline import would show file dialog");
            }
        });
    });
}

pub fn draw_timeline_controls(ui: &mut Ui, timeline: &mut TimelineAnimation) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Playback").strong());
        
        // Compact playback controls
        let play_icon = match timeline.timeline.playback_state {
            PlaybackState::Playing => "⏸",
            PlaybackState::Paused => "▶",
            PlaybackState::Stopped => "▶",
        };
        
        if ui.button(play_icon).clicked() {
            match timeline.timeline.playback_state {
                PlaybackState::Playing => timeline.timeline.pause(),
                PlaybackState::Paused | PlaybackState::Stopped => timeline.timeline.play(),
            }
            timeline.playing = timeline.timeline.playback_state == PlaybackState::Playing;
        }
        
        if ui.button("⏹").clicked() {
            timeline.timeline.stop();
            timeline.playing = false;
        }
        
        ui.separator();
        
        // Time scrubber
        let time_before = timeline.timeline.current_time;
        ui.add(bevy_egui::egui::Slider::new(&mut timeline.timeline.current_time, 0.0..=timeline.timeline.duration)
            .text("Time")
            .suffix("s"));
        
        if (timeline.timeline.current_time - time_before).abs() > 0.001 {
            timeline.timeline.playback_state = PlaybackState::Paused;
            timeline.playing = false;
        }
        
        ui.separator();
        
        // Duration control
        ui.label("Duration:");
        ui.add(bevy_egui::egui::DragValue::new(&mut timeline.timeline.duration)
            .speed(0.1)
            .range(1.0..=300.0)
            .suffix("s"));
    });
}
