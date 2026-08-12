// Fosfora effect loader
// Handles .pfx (Pyroformer) effect file parsing and parameter extraction

use std::collections::HashMap;
use std::path::Path;

/// Fosfora .pfx effect format structure
#[derive(Debug, Clone)]
pub struct PfxEffect {
    /// Effect name from the .pfx file
    pub name: String,
    /// Number of channels in the effect
    pub num_channels: u32,
    /// Sample rate of the audio
    pub sample_rate: u32,
    /// Number of audio samples per channel
    pub num_samples: u32,
    /// Effect type/classification
    pub effect_type: String,
    /// Duration in seconds
    pub duration_secs: f32,
    /// Parameter definitions with default values
    pub parameters: HashMap<String, f32>,
    /// Effect pass structure with parameters per pass
    pub passes: Vec<PfxPass>,
}

/// Individual pass within a .pfx effect
#[derive(Debug, Clone)]
pub struct PfxPass {
    /// Pass name
    pub name: String,
    /// Blend mode (additive, multiply, etc.)
    pub blend_mode: String,
    /// Uniform parameters for this pass
    pub uniforms: HashMap<String, f32>,
    /// Shader code associated with this pass
    pub shader_code: String,
    /// Audio modulation parameters for this pass
    pub audio_mod: PfxAudioMod,
}

/// Audio modulation parameters for a pass
#[derive(Debug, Clone)]
pub struct PfxAudioMod {
    /// Enable audio-driven modulation
    pub enabled: bool,
    /// Modulation source (loudness, beat, etc.)
    pub source: String,
    /// Modulation depth (0.0-1.0)
    pub depth: f32,
    /// Modulation rate in Hz
    pub rate: f32,
    /// Attack time in seconds
    pub attack: f32,
    /// Release time in seconds
    pub release: f32,
    /// Target parameter to modulate
    pub target_param: String,
    /// Target value range [min, max]
    pub target_range: (f32, f32),
}

impl PfxEffect {
    /// Load a .pfx effect from a file path
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse .pfx effect from string content
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut lines = content.lines();
        let mut effect = PfxEffect {
            name: String::new(),
            num_channels: 0,
            sample_rate: 44100,
            num_samples: 0,
            effect_type: String::new(),
            duration_secs: 0.0,
            parameters: HashMap::new(),
            passes: Vec::new(),
        };

        // Parse header: name, num_channels, sample_rate, num_samples, effect_type, duration
        if let Some(line) = lines.next() {
            if line.starts_with("#Pfx") {
                // Valid .pfx header
                for line in &mut lines {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    
                    let mut parts = trimmed.splitn(2, ':');
                    let key = parts.next().unwrap_or("").trim();
                    let value = parts.next().unwrap_or("").trim();
                    
                    match key {
                        "name" => effect.name = value.to_string(),
                        "channels" => {
                            effect.num_channels = value.parse().unwrap_or(0);
                        }
                        "rate" => {
                            effect.sample_rate = value.parse().unwrap_or(44100);
                        }
                        "samples" => {
                            effect.num_samples = value.parse().unwrap_or(0);
                        }
                        "type" => effect.effect_type = value.to_string(),
                        "duration" => {
                            effect.duration_secs = value.parse().unwrap_or(0.0);
                        }
                        "parameters" => {
                            // Parse key=value pairs until empty line or next section
                            for param_line in &mut lines {
                                let param_trimmed = param_line.trim();
                                if param_trimmed.is_empty() || param_trimmed.starts_with('#') {
                                    break;
                                }
                                if let Some(param_parts) = param_trimmed.splitn(2, '=') {
                                    let param_key = param_parts.next().unwrap().trim();
                                    let param_val = param_parts.next().unwrap().trim();
                                    if let Ok(fval) = param_val.parse::<f32>() {
                                        effect.parameters.insert(param_key.to_string(), fval);
                                    }
                                }
                            }
                        }
                        "effect_type" => effect.effect_type = value.to_string(),
                        "duration_secs" => effect.duration_secs = value.parse().unwrap_or(0.0),
                        "" => break, // Empty line ends header
                        _ => {}
                    }
                }
            }
        }

        // Parse passes (simplified)
        // In full implementation, parse pass sections with uniforms and shader code
        // For now, add a default pass if none found
        if effect.passes.is_empty() {
            effect.passes.push(PfxPass {
                name: effect.name.clone() + "_pass",
                blend_mode: "additive".to_string(),
                uniforms: HashMap::new(),
                shader_code: "// Default pass shader\nfn main() {\n    // shader code would go here\n}\n".to_string(),
                audio_mod: PfxAudioMod {
                    enabled: false,
                    source: "loudness".to_string(),
                    depth: 0.5,
                    rate: 1.0,
                    attack: 0.01,
                    release: 0.1,
                    target_param: "volume".to_string(),
                    target_range: (0.0, 1.0),
                },
            });
        }

        Ok(effect)
    }

    /// Get a parameter value by name
    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        self.parameters.get(name).copied()
    }

    /// Get pass by name
    pub fn get_pass(&self, name: &str) -> Option<&PfxPass> {
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
    pub fn load_effect(&mut self, path: &Path) -> anyhow::Result<()> {
        let effect = PfxEffect::load(path)?;
        self.effects.insert(effect.name.clone(), effect);
        Ok(())
    }

    /// Load all .pfx effects from a directory
    pub fn load_directory(&mut self, dir: &Path) -> anyhow::Result<()> {
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
        self.effects.get(effect_name).and_then(|eff| eff.get_parameter(param_name))
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
    pub fn map_audio_to_parameters(&mut self, audio_features: &crate::AudioFeatures) {
        // Map key Fosfora features to .pfx parameters
        let mappings: [(&str, fn(&AudioFeatures) -> f32); 12] = [
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