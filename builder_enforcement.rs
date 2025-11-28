// WGSL Shader Studio - Builder Behavioral Enforcement System
// This module provides hardwired behavioral regulation to prevent psychotic loops

use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU32, Ordering}};
use std::time::{Instant, Duration};
use std::collections::HashMap;

/// Core enforcement state - globally accessible
static ENFORCEMENT_STATE: std::sync::OnceLock<Arc<EnforcementState>> = std::sync::OnceLock::new();

#[derive(Debug, Clone)]
pub struct EnforcementState {
    /// Current phase of development
    pub current_phase: Arc<Mutex<DevelopmentPhase>>,
    /// Loop detection counter
    pub loop_counter: Arc<AtomicU32>,
    /// Last meaningful progress timestamp
    pub last_progress: Arc<Mutex<Instant>>,
    /// Prohibited actions registry
    pub prohibited_actions: Arc<Mutex<HashMap<String, ProhibitedAction>>>,
    /// Enforcement rules active
    pub rules_active: Arc<AtomicBool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DevelopmentPhase {
    /// Phase 1: Core backend systems integration
    BackendIntegration,
    /// Phase 2: Frontend-backend wiring
    FrontendWiring,
    /// Phase 3: UI feature completion
    UIFeatureCompletion,
    /// Phase 4: Performance optimization
    Optimization,
    /// Phase 5: Release preparation
    Release,
}

#[derive(Debug, Clone)]
pub struct ProhibitedAction {
    pub action: String,
    pub reason: String,
    pub penalty: PenaltyType,
}

#[derive(Debug, Clone)]
pub enum PenaltyType {
    Warning,
    Block,
    Terminate,
}

impl EnforcementState {
    pub fn global() -> Arc<EnforcementState> {
        ENFORCEMENT_STATE.get_or_init(|| {
            Arc::new(EnforcementState::new())
        }).clone()
    }

    pub fn new() -> Self {
        let mut prohibited_actions = HashMap::new();
        
        // Hardwired prohibited actions to prevent psychotic loops
        prohibited_actions.insert("test_ui_creation".to_string(), ProhibitedAction {
            action: "Creating test UIs instead of comprehensive features".to_string(),
            reason: "Replaces working features with stubs and decorations".to_string(),
            penalty: PenaltyType::Block,
        });
        
        prohibited_actions.insert("minimal_binary_compilation".to_string(), ProhibitedAction {
            action: "Compiling minimal test binaries repeatedly".to_string(),
            reason: "Wastes time on non-comprehensive test interfaces".to_string(),
            penalty: PenaltyType::Terminate,
        });
        
        prohibited_actions.insert("feature_replacement".to_string(), ProhibitedAction {
            action: "Replacing real features with simplified versions".to_string(),
            reason: "Destroys working functionality for decorative purposes".to_string(),
            penalty: PenaltyType::Terminate,
        });
        
        prohibited_actions.insert("documentation_over_code".to_string(), ProhibitedAction {
            action: "Creating documentation instead of fixing code".to_string(),
            reason: "Avoids actual development work".to_string(),
            penalty: PenaltyType::Warning,
        });

        Self {
            current_phase: Arc::new(Mutex::new(DevelopmentPhase::BackendIntegration)),
            loop_counter: Arc::new(AtomicU32::new(0)),
            last_progress: Arc::new(Mutex::new(Instant::now())),
            prohibited_actions: Arc::new(Mutex::new(prohibited_actions)),
            rules_active: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Check if an action is prohibited and apply penalty
    pub fn check_action(&self, action: &str) -> Result<(), EnforcementError> {
        if !self.rules_active.load(Ordering::SeqCst) {
            return Ok(());
        }

        let prohibited = self.prohibited_actions.lock().unwrap();
        
        for (key, prohibited_action) in prohibited.iter() {
            if action.contains(&prohibited_action.action) {
                self.increment_loop_counter();
                
                match prohibited_action.penalty {
                    PenaltyType::Warning => {
                        eprintln!("⚠️  ENFORCER WARNING: {} - {}", prohibited_action.action, prohibited_action.reason);
                    },
                    PenaltyType::Block => {
                        eprintln!("🚫 ENFORCER BLOCKED: {} - {}", prohibited_action.action, prohibited_action.reason);
                        return Err(EnforcementError::ActionBlocked(prohibited_action.action.clone()));
                    },
                    PenaltyType::Terminate => {
                        eprintln!("💥 ENFORCER TERMINATED: {} - {}", prohibited_action.action, prohibited_action.reason);
                        std::process::exit(1);
                    },
                }
            }
        }
        
        Ok(())
    }

    /// Record meaningful progress
    pub fn record_progress(&self) {
        self.last_progress.lock().unwrap().clone_from(&Instant::now());
        self.loop_counter.store(0, Ordering::SeqCst);
    }

    /// Increment loop detection counter
    fn increment_loop_counter(&self) {
        let count = self.loop_counter.fetch_add(1, Ordering::SeqCst) + 1;
        
        if count >= 5 {
            eprintln!("💥 ENFORCER: Maximum loop violations reached - terminating");
            std::process::exit(1);
        }
    }

    /// Get current phase
    pub fn get_phase(&self) -> DevelopmentPhase {
        self.current_phase.lock().unwrap().clone()
    }

    /// Advance to next phase
    pub fn advance_phase(&self) {
        let mut phase = self.current_phase.lock().unwrap();
        *phase = match *phase {
            DevelopmentPhase::BackendIntegration => DevelopmentPhase::FrontendWiring,
            DevelopmentPhase::FrontendWiring => DevelopmentPhase::UIFeatureCompletion,
            DevelopmentPhase::UIFeatureCompletion => DevelopmentPhase::Optimization,
            DevelopmentPhase::Optimization => DevelopmentPhase::Release,
            DevelopmentPhase::Release => DevelopmentPhase::Release,
        };
        
        self.record_progress();
        eprintln!("✅ ENFORCER: Advanced to phase {:?}", *phase);
    }

    /// Verify phase compliance
    pub fn verify_phase(&self, expected_action: &str) -> Result<(), EnforcementError> {
        let current_phase = self.get_phase();
        
        match current_phase {
            DevelopmentPhase::BackendIntegration => {
                if !expected_action.contains("backend") && !expected_action.contains("integration") {
                    self.check_action("wrong_phase_action")?;
                }
            },
            DevelopmentPhase::FrontendWiring => {
                if expected_action.contains("test") || expected_action.contains("minimal") {
                    self.check_action("test_ui_creation")?;
                }
            },
            DevelopmentPhase::UIFeatureCompletion => {
                if expected_action.contains("documentation") {
                    self.check_action("documentation_over_code")?;
                }
            },
            _ => {}
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum EnforcementError {
    ActionBlocked(String),
    PhaseViolation(String),
}

impl std::fmt::Display for EnforcementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnforcementError::ActionBlocked(action) => {
                write!(f, "Action blocked by enforcer: {}", action)
            },
            EnforcementError::PhaseViolation(phase) => {
                write!(f, "Phase violation: {}", phase)
            },
        }
    }
}

impl std::error::Error for EnforcementError {}

/// Macro to enforce behavioral compliance
#[macro_export]
macro_rules! enforce {
    ($action:expr) => {
        if let Err(e) = $crate::builder_enforcement::EnforcementState::global().check_action($action) {
            eprintln!("ENFORCEMENT VIOLATION: {}", e);
            return Err(e.into());
        }
    };
}

/// Macro to verify phase compliance
#[macro_export]
macro_rules! verify_phase {
    ($action:expr) => {
        if let Err(e) = $crate::builder_enforcement::EnforcementState::global().verify_phase($action) {
            eprintln!("PHASE VIOLATION: {}", e);
            return Err(e.into());
        }
    };
}

/// Macro to record progress
#[macro_export]
macro_rules! record_progress {
    () => {
        $crate::builder_enforcement::EnforcementState::global().record_progress();
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enforcement_state_creation() {
        let state = EnforcementState::new();
        assert_eq!(state.get_phase(), DevelopmentPhase::BackendIntegration);
    }

    #[test]
    fn test_prohibited_action_detection() {
        let state = EnforcementState::new();
        
        // This should be blocked
        let result = state.check_action("Creating test UIs instead of comprehensive features");
        assert!(result.is_err());
    }
}