//! # SuperInstance ↔ Control Engine Bridge
//!
//! Connects the Control Engine (prime scheduling + complex math + optional NN)
//! to the SuperInstance infrastructure (FLUX, PLATO, Conservation).
//!
//! ## Architecture
//!
//! The bridge takes control outputs and applies them to SuperInstance resources:
//!
//! - **FluxCompiler**: Control phases determine compile/cache strategy
//! - **BudgetTracker**: Prime schedules influence budget allocation
//! - **PlatoRoom**: Control state updates sensor readings and triggers alarms
//! - **Instance data**: Prime masks select which instances receive modulation
//!
//! All bridge operations are pure Rust — no shader modifications needed.

use crate::control_engine::ControlState;
use crate::superinstance::conservation_integration::{
    BudgetTracker, CompileOperation, ConservationEnforcer,
};
use crate::superinstance::flux_integration::FluxCompiler;
use crate::superinstance::plato_integration::{AlarmSeverity, PlatoRoom, SensorValue};

/// Bridges control engine outputs to SuperInstance infrastructure
pub struct SuperInstanceController {
    /// Budget tracker for cost-aware operations
    pub budget: BudgetTracker,
    /// Conservation enforcer for gamma/eta routing
    pub enforcer: ConservationEnforcer,
    /// Current budget summary
    pub last_summary: BudgetSummary,
}

/// Snapshot of budget state for the control engine to use
#[derive(Debug, Clone)]
pub struct BudgetSummary {
    pub gamma_remaining: f64,
    pub eta_remaining: f64,
    pub total_remaining: f64,
    pub operations_this_session: usize,
}

impl SuperInstanceController {
    /// Create a new controller with default budgets
    pub fn new(daily_budget: f64) -> Self {
        let budget = BudgetTracker::with_budget(daily_budget, 0.8);
        let summary = BudgetSummary {
            gamma_remaining: budget.budget_summary().gamma_remaining,
            eta_remaining: budget.budget_summary().eta_remaining,
            total_remaining: budget.budget_summary().remaining,
            operations_this_session: 0,
        };
        Self {
            budget,
            enforcer: ConservationEnforcer::new(daily_budget),
            last_summary: summary,
        }
    }

    /// Update budget state from control engine outputs
    ///
    /// Prime groups determine which operations get priority budget allocation:
    /// - Group A (prime 2): gets first access to gamma (crystallized) budget
    /// - Group E (prime 11): may need to use eta (live) budget
    pub fn update_from_control_state(&mut self, state: &ControlState) {
        let active_groups = &state.active_groups;

        // Determine budget allocation based on active groups
        for group in active_groups {
            let operation = match group.as_str() {
                "group_a" => CompileOperation::WgslParse,
                "group_b" => CompileOperation::IsfConvert,
                "group_c" => CompileOperation::NodeGraphGenerate,
                "group_d" => CompileOperation::GlslTranspile,
                "group_e" => CompileOperation::LlmOptimize,
                _ => CompileOperation::General,
            };

            // Use prime phase to decide gamma vs eta routing
            let use_gamma = state.prime_phase.cos() > 0.0;
            self.budget.record_compilation(operation, use_gamma);
        }

        // Update summary
        let bs = self.budget.budget_summary();
        self.last_summary = BudgetSummary {
            gamma_remaining: bs.gamma_remaining,
            eta_remaining: bs.eta_remaining,
            total_remaining: bs.remaining,
            operations_this_session: bs.total_operations,
        };
    }

    /// Apply control state to a PLATO room
    ///
    /// Updates room sensors with current control values:
    /// - prime_phase → budget modulation sensor
    /// - active groups → compile activity sensor
    /// - complex latent peaks → alarm triggers
    pub fn update_plato_room(&self, room: &mut PlatoRoom, state: &ControlState) {
        // Update sensors
        room.set_sensor(
            "budget_remaining",
            SensorValue::Float(self.last_summary.gamma_remaining),
        );
        room.set_sensor(
            "compile_count",
            SensorValue::Int(state.frame as i64),
        );
        room.set_sensor(
            "active_groups",
            SensorValue::Int(state.active_groups.len() as i64),
        );

        // Trigger alarms if budget is low
        if self.last_summary.gamma_remaining < 0.01 {
            room.trigger_alarm(
                "low_gamma_budget",
                AlarmSeverity::Warning,
                format!(
                    "Gamma budget low: ${:.4} remaining",
                    self.last_summary.gamma_remaining
                ),
            );
        }

        // Clear alarms if budget recovered
        if self.last_summary.gamma_remaining > 0.05 {
            room.clear_alarm("low_gamma_budget");
        }
    }

    /// Apply control state to a FluxCompiler
    ///
    /// Uses prime phase to enable/disable flux compilation.
    /// When enabled, shader compilation uses the crystallized gamma path.
    pub fn update_flux_compiler(&self, compiler: &mut FluxCompiler, state: &ControlState) {
        // Enable flux compilation when prime_phase is positive
        let should_enable = state.prime_phase.cos() > 0.0;
        compiler.set_enabled(should_enable);
    }

    /// Get instance modulation values from a prime mask and control state
    ///
    /// Returns per-instance modulation values that can be written to a storage buffer.
    /// Uses the complex latent vector + prime mask to produce varied modulations.
    pub fn get_instance_modulations(
        &self,
        instance_count: usize,
        prime_mask: &[bool],
        state: &ControlState,
    ) -> Vec<f32> {
        let mut mods = Vec::with_capacity(instance_count);
        let latent = &state.complex_latent;

        for i in 0..instance_count {
            let base_mod = if i < prime_mask.len() && prime_mask[i] {
                // Selected instances get full modulation
                let idx = i % latent.len().max(1);
                let lat_val = latent.get(idx).copied().unwrap_or(0.0);
                // Combine with prime phase
                let phase_mod = (state.prime_phase + i as f32 * 0.1).sin();
                lat_val * 0.5 + phase_mod * 0.5
            } else {
                // Non-selected instances get reduced modulation
                let phase_mod = (state.prime_phase * 0.5 + i as f32 * 0.05).sin();
                phase_mod * 0.25 + 0.5
            };
            mods.push(base_mod);
        }

        mods
    }

    /// Get global uniform values from control state
    ///
    /// Returns a flat Vec<f32> suitable for writing to a uniform buffer:
    /// [prime_phase, frame_fract, complex_peak, complex_mean, ...latent_values]
    pub fn get_global_uniforms(&self, state: &ControlState) -> Vec<f32> {
        let mut uniforms = Vec::new();
        uniforms.push(state.prime_phase);
        uniforms.push((state.frame % 1000) as f32 / 1000.0);
        // Compute peak and mean of complex latent
        let peak = state
            .complex_latent
            .iter()
            .cloned()
            .fold(0.0f32, f32::max);
        let mean = if !state.complex_latent.is_empty() {
            state.complex_latent.iter().sum::<f32>() / state.complex_latent.len() as f32
        } else {
            0.0
        };
        uniforms.push(peak);
        uniforms.push(mean);
        // Append complex latent values
        uniforms.extend_from_slice(&state.complex_latent);
        uniforms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control_engine::ControlState;

    fn test_state() -> ControlState {
        use std::collections::HashMap;
        ControlState {
            frame: 42,
            prime_phase: 1.0,
            complex_latent: vec![0.5, -0.3, 0.8, -0.1],
            instance_mod: vec![0.1; 10],
            global_uniforms: vec![0.0; 16],
            active_groups: vec!["group_a".to_string(), "group_c".to_string()],
            node_graph_params: HashMap::new(),
        }
    }

    #[test]
    fn test_controller_creation() {
        let controller = SuperInstanceController::new(0.10);
        assert_eq!(controller.last_summary.gamma_remaining, 0.08);
        assert_eq!(controller.last_summary.eta_remaining, 0.02);
    }

    #[test]
    fn test_update_from_control_state() {
        let mut controller = SuperInstanceController::new(0.10);
        let state = test_state();
        controller.update_from_control_state(&state);
        assert!(controller.last_summary.operations_this_session > 0);
    }

    #[test]
    fn test_update_plato_room() {
        let controller = SuperInstanceController::new(0.10);
        let state = test_state();
        let mut room = PlatoRoom::shader_compiler_room();

        controller.update_plato_room(&mut room, &state);

        // Budget sensor should have been updated
        match room.get_sensor("budget_remaining") {
            Some(SensorValue::Float(v)) => assert!(*v > 0.0),
            _ => panic!("budget_remaining sensor not found or wrong type"),
        }
    }

    #[test]
    fn test_update_flux_compiler() {
        let controller = SuperInstanceController::new(0.10);
        let state = test_state();
        let mut compiler = FluxCompiler::new();

        controller.update_flux_compiler(&mut compiler, &state);
        // prime_phase.cos() = 1.0.cos() > 0.0 → enabled
        assert!(compiler.is_enabled());
    }

    #[test]
    fn test_get_instance_modulations() {
        let controller = SuperInstanceController::new(0.10);
        let state = test_state();
        let mask = vec![true, false, true, false, true];

        let mods = controller.get_instance_modulations(5, &mask, &state);
        assert_eq!(mods.len(), 5);

        // Selected instances (even indices) should have different modulation
        assert!(mods[0] != mods[1]);
    }

    #[test]
    fn test_get_global_uniforms() {
        let controller = SuperInstanceController::new(0.10);
        let state = test_state();

        let uniforms = controller.get_global_uniforms(&state);
        // 4 base uniforms + 4 latent values = 8
        assert_eq!(uniforms.len(), 8);
        assert_eq!(uniforms[0], 1.0); // prime_phase
    }
}
