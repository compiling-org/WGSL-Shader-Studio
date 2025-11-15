use serde::{Deserialize, Serialize};
use bevy::prelude::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Resource)]
pub struct Timeline {
    pub tracks: HashMap<String, Vec<Keyframe>>, // parameter -> keyframes
}

impl Timeline {
    pub fn new() -> Self { Self::default() }

    pub fn add_keyframe(&mut self, param: &str, time: f32, value: f32) {
        let kf = Keyframe { time, value };
        let entry = self.tracks.entry(param.to_string()).or_default();
        entry.push(kf);
        entry.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn evaluate(&self, param: &str, time: f32, default: f32) -> f32 {
        let Some(track) = self.tracks.get(param) else { return default; };
        if track.is_empty() { return default; }
        // Find bracketing keyframes
        let mut prev = None;
        let mut next = None;
        for k in track {
            if k.time <= time { prev = Some(k.clone()); } else { next = Some(k.clone()); break; }
        }
        match (prev, next) {
            (Some(p), Some(n)) => {
                let t = ((time - p.time) / (n.time - p.time)).clamp(0.0, 1.0);
                p.value * (1.0 - t) + n.value * t
            }
            (Some(p), None) => p.value,
            (None, Some(n)) => n.value,
            _ => default,
        }
    }
}