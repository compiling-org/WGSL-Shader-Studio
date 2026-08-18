// Fosfora parameter definitions
// Matches the structure from Fosfora's params/types.rs

use serde::{Deserialize, Serialize};

/// A parameter definition for a .pfx effect.
/// This matches the Fosfora `ParamDef` enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParamDef {
    /// Float parameter with min/max/default values.
    Float {
        name: String,
        default: f32,
        min: f32,
        max: f32,
    },
    /// Boolean parameter (true/false).
    Bool {
        name: String,
        default: bool,
    },
    /// Color parameter (RGBA).
    Color {
        name: String,
        default: [f32; 4],
    },
    /// 2D point parameter.
    Point2D {
        name: String,
        default: [f32; 2],
        min: [f32; 2],
        max: [f32; 2],
    },
    /// Integer parameter.
    Int {
        name: String,
        default: i32,
        min: i32,
        max: i32,
    },
    /// Event parameter (trigger).
    Event {
        name: String,
    },
}

impl ParamDef {
    /// Get the parameter name.
    pub fn name(&self) -> &str {
        match self {
            ParamDef::Float { name, .. } => name,
            ParamDef::Bool { name, .. } => name,
            ParamDef::Color { name, .. } => name,
            ParamDef::Point2D { name, .. } => name,
            ParamDef::Int { name, .. } => name,
            ParamDef::Event { name } => name,
        }
    }
}