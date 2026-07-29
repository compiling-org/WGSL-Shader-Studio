//! # Control Engine — Prime + Complex Orchestration Layer
//!
//! Acts as a higher-level orchestration layer over SuperInstance.
//! All logic runs in Rust host and pipes results to WGSL via uniforms/buffers.
//! No NN or prime logic runs per-fragment — this is a control engine, not a shader.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                  Control Engine                      │
//! │  ┌──────────┐  ┌──────────────┐  ┌───────────────┐  │
//! │  │  primes   │  │ complex_math │  │  nn_bridge    │  │
//! │  │ (struct)  │  │  (signal→    │  │  (optional)   │  │
//! │  │ schedule  │  │   control)   │  │  (ONNX/ort)   │  │
//! │  └─────┬─────┘  └──────┬───────┘  └──────┬────────┘  │
//! │        └───────────────┬┴─────────────────┘           │
//! │                        ▼                              │
//! │              ┌─────────────────┐                      │
//! │              │  ControlState   │                      │
//! │              │  (frame output) │                      │
//! │              └────────┬────────┘                      │
//! │                       │                               │
//! │              ┌────────▼────────┐                      │
//! │              │ superinstance_  │                      │
//! │              │ bridge          │                      │
//! │              │ (→ Flux, Plato, │                      │
//! │              │  BudgetTracker)  │                      │
//! │              └─────────────────┘                      │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## Roles
//!
//! - **Complex NN role** (signal + phase): Map audio/MIDI/OSC/sensor streams into
//!   latent control states (phase, magnitude, envelopes). Model oscillatory fields
//!   that drive shader parameters: rotations, offsets, noise warps, flow fields.
//!
//! - **Prime role** (structure + scheduling): Use primes as discrete structure —
//!   time signatures, channel indexing, sampling windows — to prevent repetitive
//!   lock-in and create pseudo-non-repeating patterns.

pub mod primes;
pub mod complex_math;
pub mod nn_bridge;
pub mod superinstance_bridge;

use crate::superinstance::SuperInstanceConfig;
use std::collections::HashMap;

/// Configuration for the Control Engine
#[derive(Debug, Clone)]
pub struct ControlEngineConfig {
    /// Enable the control engine entirely
    pub enabled: bool,
    /// Enable prime-structured scheduling
    pub prime_scheduling_enabled: bool,
    /// Prime intervals for parameter group updates (e.g., [2, 3, 5, 7, 11])
    pub prime_intervals: Vec<u64>,
    /// Enable complex-valued signal processing
    pub complex_math_enabled: bool,
    /// Enable NN bridge (requires feature flag)
    pub nn_bridge_enabled: bool,
    /// Path to optional ONNX model
    pub nn_model_path: Option<String>,
    /// Number of control vector dimensions output
    pub control_vector_size: usize,
    /// Number of instance modulation dimensions
    pub instance_mod_size: usize,
}

impl Default for ControlEngineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            prime_scheduling_enabled: true,
            prime_intervals: vec![2, 3, 5, 7, 11],
            complex_math_enabled: true,
            nn_bridge_enabled: false,
            nn_model_path: None,
            control_vector_size: 16,
            instance_mod_size: 8,
        }
    }
}

/// Per-frame control state produced by the Control Engine.
/// This is the single output struct that feeds into SuperInstance and WGSL uniforms.
#[derive(Debug, Clone)]
pub struct ControlState {
    /// Current frame number (monotonically increasing)
    pub frame: u64,
    /// Prime phase — a value in [0.0, 1.0) derived from prime-scheduled frame cycling
    pub prime_phase: f32,
    /// Active prime intervals for this frame (which parameter groups should update)
    pub active_groups: Vec<String>,
    /// Complex latent vector — output of complex-valued signal processing
    pub complex_latent: Vec<f32>,
    /// Per-instance modulations — derived from prime mask × complex latent
    pub instance_mod: Vec<f32>,
    /// Global uniform values — directly mappable to WGSL uniform blocks
    pub global_uniforms: Vec<f32>,
    /// Named parameter values for the node graph
    pub node_graph_params: HashMap<String, f32>,
}

impl ControlState {
    /// Create a zero-initialized control state
    pub fn new(config: &ControlEngineConfig) -> Self {
        Self {
            frame: 0,
            prime_phase: 0.0,
            active_groups: Vec::new(),
            complex_latent: vec![0.0; config.control_vector_size],
            instance_mod: vec![0.0; config.instance_mod_size],
            global_uniforms: vec![0.0; 16],
            node_graph_params: HashMap::new(),
        }
    }
}

/// The Control Engine orchestrator.
/// Runs an update cycle each frame, compositing prime + complex signals.
pub struct ControlEngine {
    /// Configuration
    config: ControlEngineConfig,
    /// Current control state
    pub state: ControlState,
    /// Prime scheduler
    pub prime_schedule: primes::PrimeSchedule,
    /// Complex math processor (oscillator bank)
    pub oscillator_bank: complex_math::OscillatorBank,
    /// Optional NN bridge
    nn_bridge: Option<nn_bridge::NNBridge>,
    /// SuperInstance bridge
    pub si_bridge: superinstance_bridge::SuperInstanceController,
    /// Frame counter
    frame: u64,
}

impl ControlEngine {
    /// Create a new Control Engine with the given configuration and SuperInstance config
    pub fn new(
        config: ControlEngineConfig,
        superinstance_config: &SuperInstanceConfig,
    ) -> Self {
        let state = ControlState::new(&config);
        let prime_intervals = config.prime_intervals.clone();

        // Create oscillator bank with harmonic ratios
        let oscillator_bank = complex_math::OscillatorBank::new(4, 1.0);

        Self {
            prime_schedule: primes::PrimeSchedule::with_groups({
                let mut groups = HashMap::new();
                for (i, &interval) in prime_intervals.iter().enumerate() {
                    let name = format!("group_{}", (b'a' + i as u8) as char);
                    groups.insert(name, interval as u32);
                }
                groups
            }),
            oscillator_bank,
            nn_bridge: if config.nn_bridge_enabled {
                let nn_config = nn_bridge::NNBridgeConfig {
                    enabled: true,
                    model_path: config.nn_model_path.clone().unwrap_or_default(),
                    input_size: 37,
                    output_size: config.control_vector_size,
                    ..Default::default()
                };
                Some(nn_bridge::NNBridge::new(nn_config))
            } else {
                None
            },
            si_bridge: superinstance_bridge::SuperInstanceController::new(
                superinstance_config.daily_budget,
            ),
            frame: 0,
            config,
            state,
        }
    }

    /// Update the control engine for a new frame.
    /// This is the main entry point, called once per frame.
    pub fn update(
        &mut self,
        audio_inputs: Option<&[f32]>,
        _external_inputs: Option<&[f32]>,
    ) -> &ControlState {
        self.frame += 1;
        self.state.frame = self.frame;

        // 1. Update prime schedule
        if self.config.prime_scheduling_enabled {
            self.prime_schedule.tick();
            self.state.active_groups = self.prime_schedule.active_groups().to_vec();

            // Compute prime phase (0.0 - 1.0) based on frame cycling
            let total_groups = self.prime_schedule.groups.len() as f32;
            let active_count = self.state.active_groups.len() as f32;
            self.state.prime_phase = if total_groups > 0.0 {
                active_count / total_groups
            } else {
                0.0
            };
        }

        // 2. Update complex latent from inputs
        if self.config.complex_math_enabled {
            // Use oscillator bank with or without external modulation
            let dt = 1.0 / 60.0; // Assume 60 FPS
            let raw = self.oscillator_bank.tick_all(dt);

            // Apply audio modulation if available
            let modulated: Vec<f32> = if let Some(audio) = audio_inputs {
                raw.iter()
                    .enumerate()
                    .map(|(i, &v)| {
                        let audio_mod = audio.get(i % audio.len()).copied().unwrap_or(0.0);
                        v * (1.0 + audio_mod * 0.5)
                    })
                    .collect()
            } else {
                raw
            };

            // Copy to complex latent, resizing as needed
            self.state.complex_latent.clear();
            self.state
                .complex_latent
                .extend(modulated.iter().take(self.config.control_vector_size));
            while self.state.complex_latent.len() < self.config.control_vector_size {
                self.state.complex_latent.push(0.0);
            }
        }

        // 3. Compute instance modulations
        let instance_count = self.state.instance_mod.len();
        for i in 0..instance_count {
            // Prime-based base modulation
            let prime_mod = if self.state.prime_phase > 0.0 {
                (self.state.prime_phase * std::f32::consts::TAU * (i as f32 + 1.0)).sin()
            } else {
                (self.frame as f32 * 0.01 + i as f32 * 0.1).sin()
            };
            // Complex latent modulation
            let lat_idx = i % self.state.complex_latent.len().max(1);
            let lat_mod = self.state.complex_latent[lat_idx];
            // Combine
            self.state.instance_mod[i] = prime_mod * 0.5 + lat_mod * 0.5;
        }

        // 4. Compute global uniforms
        self.state.global_uniforms.clear();
        self.state.global_uniforms.push(self.state.prime_phase);
        self.state
            .global_uniforms
            .push((self.state.frame % 1000) as f32 / 1000.0);
        if !self.state.complex_latent.is_empty() {
            let peak = self.state.complex_latent.iter().cloned().fold(0.0f32, f32::max);
            let mean = self.state.complex_latent.iter().sum::<f32>()
                / self.state.complex_latent.len() as f32;
            self.state.global_uniforms.push(peak);
            self.state.global_uniforms.push(mean);
        }
        self.state
            .global_uniforms
            .extend_from_slice(&self.state.complex_latent);

        // 5. Update SuperInstance bridge
        self.si_bridge.update_from_control_state(&self.state);

        &self.state
    }

    /// Check if the control engine is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the current frame number
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Reset the engine state
    pub fn reset(&mut self) {
        self.frame = 0;
        self.state = ControlState::new(&self.config);
        self.prime_schedule.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_engine_config_default() {
        let config = ControlEngineConfig::default();
        assert!(!config.enabled);
        assert!(config.prime_scheduling_enabled);
        assert_eq!(config.prime_intervals, vec![2, 3, 5, 7, 11]);
        assert_eq!(config.control_vector_size, 16);
    }

    #[test]
    fn test_control_state_creation() {
        let config = ControlEngineConfig::default();
        let state = ControlState::new(&config);
        assert_eq!(state.frame, 0);
        assert_eq!(state.complex_latent.len(), 16);
        assert_eq!(state.instance_mod.len(), 8);
    }

    #[test]
    fn test_control_engine_creation() {
        let si_config = SuperInstanceConfig::default();
        let ce_config = ControlEngineConfig::default();
        let engine = ControlEngine::new(ce_config, &si_config);
        assert!(!engine.is_enabled());
        assert_eq!(engine.frame(), 0);
    }

    #[test]
    fn test_control_engine_update() {
        let si_config = SuperInstanceConfig::default();
        let mut ce_config = ControlEngineConfig::default();
        ce_config.enabled = true;
        let mut engine = ControlEngine::new(ce_config, &si_config);

        // Force enable so update runs
        engine.config.enabled = true;
        let state = engine.update(None, None);
        assert_eq!(state.frame, 1);

        // After update, state should have been populated
        assert!(!state.complex_latent.is_empty());
        assert!(!state.instance_mod.is_empty());
        assert!(!state.global_uniforms.is_empty());
    }

    #[test]
    fn test_control_engine_with_audio() {
        let si_config = SuperInstanceConfig::default();
        let mut ce_config = ControlEngineConfig::default();
        ce_config.enabled = true;
        let mut engine = ControlEngine::new(ce_config, &si_config);

        let audio = [0.5, 0.3, 0.8, 0.1];
        let state = engine.update(Some(&audio), None);
        assert_eq!(state.frame, 1);
    }

    #[test]
    fn test_control_engine_reset() {
        let si_config = SuperInstanceConfig::default();
        let mut ce_config = ControlEngineConfig::default();
        ce_config.enabled = true;
        let mut engine = ControlEngine::new(ce_config, &si_config);

        engine.config.enabled = true;
        engine.update(None, None);
        assert_eq!(engine.frame(), 1);

        engine.reset();
        assert_eq!(engine.frame(), 0);
        // Silently drop the warning
        let _ = engine.state;
    }

    #[test]
    fn test_prime_schedule_integration() {
        let mut engine = {
            let si_config = SuperInstanceConfig::default();
            let mut ce_config = ControlEngineConfig::default();
            ce_config.enabled = true;
            ce_config.prime_scheduling_enabled = true;
            ControlEngine::new(ce_config, &si_config)
        };

        // Tick a few frames
        for _ in 0..10 {
            engine.update(None, None);
        }

        // Prime schedule should have been active
        assert_eq!(engine.frame(), 10);
        // Active groups should be populated by now
        // (frame 2: group_a active, frame 3: group_b, frame 4: group_a, frame 5: group_c, etc.)
        assert!(!engine.state.active_groups.is_empty() || engine.state.prime_phase > 0.0);
    }
}
