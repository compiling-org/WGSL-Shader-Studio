//! # SuperInstance Integration Module
//!
//! Infrastructure layer integrating the SuperInstance ecosystem into WGSL Shader Studio.
//! Provides:
//! - FLUX VM bytecode compilation for deterministic shader compilation
//! - PLATO constraint engine coordination for agent pipelines
//! - Conservation Theory (gamma + eta = C) budget enforcement
//!
//! This is a backend/infrastructure module, not a UI module.
//! It provides CLI flags and programmatic APIs for budget-constrained shader operations.
//!
//! ## CLI Usage
//! ```bash
//! # Compile shader via FLUX bytecode (crystallized path)
//! cargo run -- --flux-compile shaders/my_shader.wgsl
//!
//! # Track budget usage during compilation
//! cargo run -- --budget-track --budget-daily 0.10
//!
//! # Run with PLATO room coordination
//! cargo run -- --plato-room "shader_pipeline" --plato-port 8847
//! ```

pub mod flux_integration;
pub mod plato_integration;
pub mod conservation_integration;

use std::path::PathBuf;

/// Configuration for the SuperInstance integration layer
#[derive(Debug, Clone)]
pub struct SuperInstanceConfig {
    /// Enable FLUX bytecode compilation
    pub flux_enabled: bool,
    /// Path to FLUX cache directory
    pub flux_cache_dir: PathBuf,
    /// Enable PLATO room coordination
    pub plato_enabled: bool,
    /// PLATO room name for agent coordination
    pub plato_room: String,
    /// PLATO server port
    pub plato_port: u16,
    /// Enable Conservation Theory budget enforcement
    pub conservation_enabled: bool,
    /// Daily budget in USD for compilation operations
    pub daily_budget: f64,
    /// Ratio of crystallized (gamma) to total budget (0.0 - 1.0)
    pub crystallized_ratio: f64,
}

impl Default for SuperInstanceConfig {
    fn default() -> Self {
        Self {
            flux_enabled: false,
            flux_cache_dir: PathBuf::from(".flux-cache"),
            plato_enabled: false,
            plato_room: "shader_compiler".to_string(),
            plato_port: 8847,
            conservation_enabled: false,
            daily_budget: 0.10, // $0.10/day default
            crystallized_ratio: 0.8, // 80% crystallized, 20% live
        }
    }
}

/// Result of a budget-constrained compilation operation
#[derive(Debug)]
pub enum CompilationResult {
    /// Compiled via FLUX bytecode (crystallized gamma path)
    Crystallized {
        bytecode: Vec<u8>,
        cost: f64,
        duration_ms: u64,
    },
    /// Compiled via fallback LLM (live eta path)
    Live {
        output: Vec<u8>,
        cost: f64,
        duration_ms: u64,
    },
    /// Budget exceeded, compilation denied
    BudgetExceeded {
        available: f64,
        required: f64,
    },
}

impl CompilationResult {
    pub fn cost(&self) -> f64 {
        match self {
            CompilationResult::Crystallized { cost, .. } => *cost,
            CompilationResult::Live { cost, .. } => *cost,
            CompilationResult::BudgetExceeded { .. } => 0.0,
        }
    }

    pub fn is_allowed(&self) -> bool {
        !matches!(self, CompilationResult::BudgetExceeded { .. })
    }
}

/// Parse CLI arguments related to SuperInstance features
pub fn parse_superinstance_args(args: &[String]) -> SuperInstanceConfig {
    let mut config = SuperInstanceConfig::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--flux-compile" => config.flux_enabled = true,
            "--budget-track" => config.conservation_enabled = true,
            "--budget-daily" => {
                if i + 1 < args.len() {
                    if let Ok(budget) = args[i + 1].parse::<f64>() {
                        config.daily_budget = budget;
                    }
                    i += 1;
                }
            }
            "--plato-room" => {
                if i + 1 < args.len() {
                    config.plato_room = args[i + 1].clone();
                    config.plato_enabled = true;
                    i += 1;
                }
            }
            "--plato-port" => {
                if i + 1 < args.len() {
                    if let Ok(port) = args[i + 1].parse::<u16>() {
                        config.plato_port = port;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SuperInstanceConfig::default();
        assert!(!config.flux_enabled);
        assert!(!config.plato_enabled);
        assert!(!config.conservation_enabled);
        assert_eq!(config.daily_budget, 0.10);
        assert_eq!(config.crystallized_ratio, 0.8);
    }

    #[test]
    fn test_parse_args_flux() {
        let args = vec![
            "program".to_string(),
            "--flux-compile".to_string(),
            "shader.wgsl".to_string(),
        ];
        let config = parse_superinstance_args(&args[1..]);
        assert!(config.flux_enabled);
        assert!(!config.plato_enabled);
    }

    #[test]
    fn test_parse_args_budget() {
        let args = vec![
            "program".to_string(),
            "--budget-track".to_string(),
            "--budget-daily".to_string(),
            "0.50".to_string(),
        ];
        let config = parse_superinstance_args(&args[1..]);
        assert!(config.conservation_enabled);
        assert_eq!(config.daily_budget, 0.50);
    }

    #[test]
    fn test_parse_args_plato() {
        let args = vec![
            "program".to_string(),
            "--plato-room".to_string(),
            "test_room".to_string(),
            "--plato-port".to_string(),
            "9000".to_string(),
        ];
        let config = parse_superinstance_args(&args[1..]);
        assert!(config.plato_enabled);
        assert_eq!(config.plato_room, "test_room");
        assert_eq!(config.plato_port, 9000);
    }

    #[test]
    fn test_compilation_result_crystallized() {
        let result = CompilationResult::Crystallized {
            bytecode: vec![0, 1, 2],
            cost: 0.0001,
            duration_ms: 10,
        };
        assert!(result.is_allowed());
        assert_eq!(result.cost(), 0.0001);
    }

    #[test]
    fn test_compilation_result_budget_exceeded() {
        let result = CompilationResult::BudgetExceeded {
            available: 0.05,
            required: 0.10,
        };
        assert!(!result.is_allowed());
        assert_eq!(result.cost(), 0.0);
    }
}
