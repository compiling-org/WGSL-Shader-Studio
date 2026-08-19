// Fosfora effect loader
// Handles .pfx (Pyroformer) effect file parsing and parameter extraction
// Implements parsing of the proper JSON-based Fosfora .pfx format
// Using the same format as reference_repos/fosfora/crates/fosfora-app/src/effect/format.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use log;

// Import the ParamDef from the params module
use crate::fosfora::params::ParamDef;

/// Visual classification of an effect for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffectType {
    /// Fragment shader only, no particles.
    Shader,
    /// Compute particles + background shader.
    Particle,
    /// Particles + accumulated-state feedback (trails, RD, N-body, etc.).
    Feedback,}

/// Loop-export contract (overlay initiative): how an effect's motion relates to time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoopMode {
    /// Ordinary effect: free-running time, feedback and history allowed.
    #[default]
    Free,
    /// Pure function of the uniform block; all motion phase-derived.
    PhaseLocked,
    /// Explicitly disabled loop mode.
    Disabled,
}

impl LoopMode {
    /// Check if this is the default Free mode.
    pub fn is_free(&self) -> bool {
        matches!(self, LoopMode::Free)
    }
}

/// A render pass definition within a multi-pass effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassDef {
    pub name: String,
    pub shader: String,
    #[serde(default = "default_scale")]
    pub scale: f32,
    /// Names of earlier passes whose current-frame outputs this pass samples as
    /// `input0..` (in declared order). Forward/unknown references are a hard error.
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Names of feedback passes whose previous-frame outputs this pass samples,
    /// appended after `inputs` in the `input0..` numbering.
    #[serde(default)]
    pub prev_inputs: Vec<String>,
    /// Number of times to run this pass per frame, ping-ponging its own target between
    /// runs (Jacobi/relaxation loops). Only meaningful for `feedback: true` passes;
    /// defaults to 1 (single draw, legacy behavior).
    #[serde(default = "default_one")]
    pub iterations: u32,
    /// Whether this pass reads its own previous frame (ping-pong feedback).
    /// Defaults to true (matches legacy single-shader behavior); set false to disable.
    #[serde(default = "default_true")]
    pub feedback: bool,
}
fn default_scale() -> f32 {
    1.0
}
fn default_one() -> u32 {
    1
}

/// Per-effect post-processing overrides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostProcessDef {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_bloom_threshold")]
    pub bloom_threshold: f32,
    #[serde(default = "default_bloom_intensity")]
    pub bloom_intensity: f32,
    #[serde(default = "default_vignette")]
    pub vignette: f32,
    #[serde(default = "default_half")]
    pub ca_intensity: f32,
    #[serde(default = "default_half")]
    pub grain_intensity: f32,
    /// Film-grain updates per second. The grain deliberately runs slower than
    /// the display so a repeated frame does not freeze a boiling noise field
    /// into a visible flash (#1983). 0 = update every frame (pre-1.26 look).
    #[serde(default = "default_grain_rate")]
    pub grain_rate: f32,
    #[serde(default = "default_true")]
    pub bloom_enabled: bool,
    #[serde(default = "default_true")]
    pub ca_enabled: bool,
    #[serde(default = "default_true")]
    pub vignette_enabled: bool,
    #[serde(default = "default_true")]
    pub grain_enabled: bool,
    /// Tonemap operator: "aces" (default, Phosphor house look) or "linear"
    /// (raw passthrough clamp, matching SuperSplat for the Splat effect).
    #[serde(default = "default_tonemap")]
    pub tonemap: String,
}
fn default_true() -> bool {
    true
}

fn default_tonemap() -> String {
    "aces".to_string()
}

fn default_bloom_threshold() -> f32 {
    0.8
}
fn default_bloom_intensity() -> f32 {
    0.35
}
fn default_vignette() -> f32 {
    0.3
}
fn default_half() -> f32 {
    0.5
}
fn default_grain_rate() -> f32 {
    24.0
}
impl Default for PostProcessDef {
    fn default() -> Self {
        Self {
            enabled: true,
            bloom_threshold: 0.8,
            bloom_intensity: 0.35,
            vignette: 0.3,
            ca_intensity: 0.5,
            grain_intensity: 0.5,
            grain_rate: 24.0,
            bloom_enabled: true,
            ca_enabled: true,
            vignette_enabled: true,
            grain_enabled: true,
            tonemap: "aces".to_string(),
        }
    }
}

/// Describes which audio feature drives which visual aspect of an effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioMapping {
    pub feature: String,
    pub target: String,
}

/// A .pfx effect definition (JSON format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PfxEffect {
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    /// Single-pass shader (backward compatible). Ignored if `passes` is non-empty.
    #[serde(default)]
    pub shader: String,
    /// Inputs (parameters) for the effect.
    #[serde(default)]
    pub inputs: Vec<ParamDef>,
    /// Multi-pass pipeline definition. If empty, `shader` field is used as a single pass.
    #[serde(default)]
    pub passes: Vec<PassDef>,
    /// Per-effect post-processing overrides.
    #[serde(default)]
    pub postprocess: Option<PostProcessDef>,
    /// GPU particle system definition.
    #[serde(default)]
    pub particles: Option<ParticleDef>,
    /// Audio feature → visual target mappings (read-only display in UI).
    #[serde(default)]
    pub audio_mappings: Vec<AudioMapping>,
    /// If true, effect is hidden from UI (not shown in effects panel or next/prev cycling).
    #[serde(default)]
    pub hidden: bool,
    /// Browser grouping bucket: `"effect"` (default) lists normally, `"overlay"` groups
    /// under the Overlay section.
    #[serde(
        default = "default_category",
        skip_serializing_if = "is_default_category"
    )]
    pub category: String,
    /// The effect emits a meaningful alpha channel.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub alpha: bool,
    /// Spelled `loop` in the JSON (`loop` is a Rust keyword).
    #[serde(rename = "loop", default, skip_serializing_if = "LoopMode::is_free")]
    pub loop_mode: LoopMode,
    /// Explicit effect type override (shader/particle/feedback).
    /// If absent, auto-detected: no particles → Shader, particles → Particle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_type: Option<EffectType>,
    /// Path to the .pfx file on disk (not serialized).
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

/// Describes what changed between two versions of a PfxEffect.
#[derive(Debug, Clone, PartialEq)]
pub struct PfxDiff {
    pub metadata_changed: bool,
    pub inputs_changed: bool,
    pub postprocess_changed: bool,
    pub passes_changed: bool,
    pub particles_changed: bool,
}
impl PfxDiff {
    pub fn is_empty(&self) -> bool {
        !self.metadata_changed
            && !self.inputs_changed
            && !self.postprocess_changed
            && !self.passes_changed
            && !self.particles_changed
    }
    pub fn needs_rebuild(&self) -> bool {
        self.passes_changed || self.particles_changed
    }
}
fn default_category() -> String {
    "effect".to_string()
}
fn is_default_category(c: &str) -> bool {
    c == "effect"
}
impl PfxEffect {
    /// Returns the effect type: explicit if set, otherwise auto-detected.
    pub fn effect_type(&self) -> EffectType {
        if let Some(et) = self.effect_type {
            return et;
        }
        if self.particles.is_some() {
            EffectType::Particle
        } else {
            EffectType::Shader
        }
    }

    /// Normalize: if `passes` is empty but `shader` is set, create a single-pass definition.
    /// Single-pass effects get feedback enabled by default (matches legacy behavior).
    pub fn normalized_passes(&self) -> Vec<PassDef> {
        if !self.passes.is_empty() {
            return self.passes.clone();
        }
        if !self.shader.is_empty() {
            vec![PassDef {
                name: "main".to_string(),
                shader: self.shader.clone(),
                scale: 1.0,
                inputs: vec![],
                prev_inputs: vec![],
                iterations: 1,
                feedback: true,
            }]
        } else {
            vec![]
        }
    }

    /// Compare two PfxEffect versions and identify what changed.
    pub fn diff(&self, other: &PfxEffect) -> PfxDiff {
        PfxDiff {
            metadata_changed: self.name != other.name
                || self.author != other.author
                || self.description != other.description
                || self.hidden != other.hidden
                || self.audio_mappings != other.audio_mappings
                || self.category != other.category
                || self.alpha != other.alpha
                || self.loop_mode != other.loop_mode,
            inputs_changed: self.inputs != other.inputs,
            postprocess_changed: self.postprocess != other.postprocess,
            passes_changed: self.normalized_passes() != other.normalized_passes(),
            particles_changed: self.particles != other.particles,
        }
    }

    /// Load a .pfx effect from a file path
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse .pfx effect from JSON string content
    pub fn parse(content: &str) -> Result<Self> {
        serde_json::from_str(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse .pfx file: {}", e))
    }

    /// Get a parameter (input) value by name
    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        for input in &self.inputs {
            match input {
                ParamDef::Float { name: n, .. } if n == name => {
                    if let ParamDef::Float { default, .. } = input {
                        return Some(*default);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Get pass by name
    pub fn get_pass(&self, name: &str) -> Option<&PassDef> {
        self.passes.iter().find(|pass| pass.name == name)
    }
}

/// Effect loader that manages multiple .pfx effects
pub struct EffectLoader {
    /// Loaded effects indexed by name
    pub effects: HashMap<String, PfxEffect>,
    /// Current active effect index
    pub active_effect: Option<String>,
    /// Parameter override map (UI overrides)
    pub parameter_overrides: HashMap<String, f32>,
}

impl EffectLoader {
    pub fn new() -> Self {
        EffectLoader {
            effects: HashMap::new(),
            active_effect: None,
            parameter_overrides: HashMap::new(),
        }
    }

    /// Load an effect from file path
    pub fn load_effect(&mut self, path: &Path) -> Result<()> {
        let effect = PfxEffect::load(path)?;
        self.effects.insert(effect.name.clone(), effect);
        Ok(())
    }

    /// Load all .pfx effects from a directory
    pub fn load_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only load .pfx files
            if path.extension().map_or(false, |ext| ext == "pfx") {
                let effect = PfxEffect::load(&path)?;
                self.effects.insert(effect.name.clone(), effect);
            }
        }
        
        Ok(())
    }

    /// Get current effect's parameter value
    pub fn get_parameter_value(&self, effect_name: &str, param_name: &str) -> Option<f32> {
        self.effects.get(effect_name)
            .and_then(|eff| eff.get_parameter(param_name))
    }

    /// Apply UI parameter overrides
    pub fn override_parameter(&mut self, param_name: &str, value: f32) {
        self.parameter_overrides.insert(param_name.to_string(), value);
    }

    /// Get effective parameter value (considering overrides)
    pub fn get_effective_parameter(&self, effect_name: &str, param_name: &str) -> f32 {
        // Check overrides first, then effect parameters, then default
        if let Some(&overridden) = self.parameter_overrides.get(param_name) {
            return overridden;
        }
        
        self.effects.get(effect_name)
            .and_then(|eff| eff.get_parameter(param_name))
            .unwrap_or(0.5) // Default fallback
    }

    /// Map Fosfora audio features to effect parameters
    pub fn map_audio_to_parameters(&mut self, audio_features: &crate::audio_system::AudioFeatures) {
        // Map key Fosfora features to .pfx parameters
        let mappings: [(&str, fn(&crate::audio_system::AudioFeatures) -> f32); 12] = [
            ("loudness", |f| f.loudness_m),
            ("key_root", |f| f.key_class),
            ("key_minor", |f| f.key_is_minor),
            ("downbeat", |f| f.downbeat),
            ("bar_phase", |f| f.bar_phase),
            ("beat_phase", |f| f.beat_phase),
            ("stereo_width", |f| f.stereo_width),
            ("contrast_mean", |f| f.contrast_mean),
            ("timbral_flux", |f| f.timbre_flux),
            ("percussive_energy", |f| f.percussive_energy),
            ("harmonic_ratio", |f| f.harmonic_ratio),
            ("pitch", |f| f.pitch),
        ];
        
        for (param_name, feature_fn) in mappings.iter() {
            let value = feature_fn(audio_features);
            self.override_parameter(param_name, value);
        }
    }
}

impl Default for EffectLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder for ParticleDef - should be replaced with actual implementation
// This allows the loader.rs to compile while particles are not yet integrated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ParticleDef {
    pub max_count: u32,
    #[serde(default)]
    pub max_scaled_count: u32,
    #[serde(default)]
    pub render_mode: String,
    #[serde(default)]
    pub compute_shader: String,
    #[serde(default)]
    pub trail_length: u32,
    #[serde(default)]
    pub trail_width: f32,
    #[serde(default)]
    pub blend: String,
    #[serde(default)]
    pub interaction: bool,
}

fn is_free(loop_mode: &LoopMode) -> bool {
    matches!(loop_mode, LoopMode::Free)
}
