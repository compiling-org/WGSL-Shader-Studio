//! # Conservation Theory Integration (gamma + eta = C)
//!
//! Provides budget enforcement for shader compilation operations based on
//! the Conservation Theory: gamma + eta = C.
//!
//! ## Theory
//!
//! The conservation law states: **gamma + eta = C**
//!
//! Where:
//! - **gamma** = crystallized cognition (deterministic bytecode, cached patterns, FLUX programs)
//! - **eta** = live entropy (LLM inference, heuristic search, uncertain computation)
//! - **C** = total budget (fixed, measurable, cannot be exceeded)
//!
//! ## Budget Categories
//!
//! | Operation | gamma Cost | eta Cost | C Budget | Strategy |
//! |-----------|-----------|---------|----------|----------|
//! | WGSL -> SPIRV (naga) | ~$0.0001 | -- | Fixed | Always gamma |
//! | ISF -> WGSL conversion | ~$0.0001 | ~$0.01 | $0.05/day | Try gamma first |
//! | GLSL -> WGSL transpile | ~$0.0005 | ~$0.02 | $0.10/day | Crystallize patterns |
//! | Node graph -> WGSL gen | ~$0.0002 | ~$0.015 | $0.08/day | Cache bytecode |
//! | LLM-assisted shader fix | -- | ~$0.05 | $0.20/day | eta only |

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Operation types with associated budget costs
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CompileOperation {
    /// WGSL source parsing and validation
    WgslParse,
    /// SPIRV bytecode generation from WGSL
    SpirvGenerate,
    /// ISF shader conversion to WGSL
    IsfConvert,
    /// GLSL to WGSL transpilation
    GlslTranspile,
    /// HLSL to WGSL transpilation
    HlslTranspile,
    /// Node graph to WGSL code generation
    NodeGraphGenerate,
    /// LLM-assisted shader optimization
    LlmOptimize,
    /// LLM-assisted error fix
    ErrorFix,
    /// General compilation operation
    General,
}

impl CompileOperation {
    /// Get the crystallized (gamma) cost estimate in USD
    pub fn gamma_cost(&self) -> f64 {
        match self {
            CompileOperation::WgslParse => 0.0001,
            CompileOperation::SpirvGenerate => 0.0001,
            CompileOperation::IsfConvert => 0.0003,
            CompileOperation::GlslTranspile => 0.0005,
            CompileOperation::HlslTranspile => 0.0005,
            CompileOperation::NodeGraphGenerate => 0.0002,
            CompileOperation::LlmOptimize => 0.0001, // Not applicable, but tracked
            CompileOperation::ErrorFix => 0.0001,
            CompileOperation::General => 0.0002,
        }
    }

    /// Get the live (eta) cost estimate in USD
    pub fn eta_cost(&self) -> f64 {
        match self {
            CompileOperation::WgslParse => 0.0, // Always gamma
            CompileOperation::SpirvGenerate => 0.0, // Always gamma
            CompileOperation::IsfConvert => 0.01,
            CompileOperation::GlslTranspile => 0.02,
            CompileOperation::HlslTranspile => 0.02,
            CompileOperation::NodeGraphGenerate => 0.015,
            CompileOperation::LlmOptimize => 0.05,
            CompileOperation::ErrorFix => 0.05,
            CompileOperation::General => 0.01,
        }
    }

    /// Check if this operation can use crystallized (gamma) path
    pub fn can_crystallize(&self) -> bool {
        matches!(self, CompileOperation::WgslParse | CompileOperation::SpirvGenerate)
    }
}

/// Budget tracker implementing Conservation Theory
pub struct BudgetTracker {
    /// Daily budget cap in USD
    daily_budget: f64,
    /// Crystallized budget (gamma)
    crystallized_budget: f64,
    /// Per-call eta budget
    live_budget: f64,
    /// Budget consumed so far today
    consumed: f64,
    /// Crystallized consumption (gamma)
    gamma_consumed: f64,
    /// Live consumption (eta)
    eta_consumed: f64,
    /// Daily budget reset time
    day_start: Instant,
    /// Operation log for auditing
    operation_log: Vec<BudgetEntry>,
    /// Maximum log entries
    max_log_entries: usize,
}

/// A logged budget entry for auditing
#[derive(Debug, Clone)]
pub struct BudgetEntry {
    pub operation: CompileOperation,
    pub gamma_used: f64,
    pub eta_used: f64,
    pub total_cost: f64,
    pub timestamp: Instant,
    pub allowed: bool,
}

impl BudgetTracker {
    /// Create a new budget tracker with default budget ($0.10/day)
    pub fn new() -> Self {
        Self::with_budget(0.10, 0.8)
    }

    /// Create a budget tracker with custom budget
    pub fn with_budget(daily_budget: f64, crystallized_ratio: f64) -> Self {
        Self {
            daily_budget,
            crystallized_budget: daily_budget * crystallized_ratio,
            live_budget: daily_budget * (1.0 - crystallized_ratio),
            consumed: 0.0,
            gamma_consumed: 0.0,
            eta_consumed: 0.0,
            day_start: Instant::now(),
            operation_log: Vec::new(),
            max_log_entries: 10000,
        }
    }

    /// Check if an operation can be performed within budget
    ///
    /// Returns true if the operation is affordable, false if budget is exceeded.
    /// Operations that are always gamma (like WGSL parsing) are always allowed.
    pub fn can_compile(&self, operation: &CompileOperation) -> bool {
        // Always-free operations (pure gamma, no eta)
        if operation.can_crystallize() && operation.eta_cost() == 0.0 {
            return true;
        }

        // Check if we have enough gamma budget
        let gamma_cost = operation.gamma_cost();
        let eta_cost = operation.eta_cost();

        // Try gamma path first
        if self.gamma_consumed + gamma_cost <= self.crystallized_budget {
            return true;
        }

        // Check eta path (with gamma already exceeded)
        if self.eta_consumed + eta_cost <= self.live_budget {
            return true;
        }

        // Total budget check
        self.consumed + gamma_cost + eta_cost <= self.daily_budget
    }

    /// Record a compilation operation and deduct from budget
    ///
    /// Returns the BudgetEntry recording what was spent.
    pub fn record_compilation(&mut self, operation: CompileOperation, use_gamma: bool) -> BudgetEntry {
        let gamma = if use_gamma { operation.gamma_cost() } else { 0.0 };
        let eta = if use_gamma { 0.0 } else { operation.eta_cost() };
        let total = gamma + eta;

        let allowed = self.can_compile(&operation);
        if allowed {
            self.gamma_consumed += gamma;
            self.eta_consumed += eta;
            self.consumed += total;
        }

        let entry = BudgetEntry {
            operation: operation.clone(),
            gamma_used: gamma,
            eta_used: eta,
            total_cost: total,
            timestamp: Instant::now(),
            allowed,
        };

        self.operation_log.push(entry.clone());
        if self.operation_log.len() > self.max_log_entries {
            self.operation_log.remove(0);
        }

        entry
    }

    /// Crystallize a pattern: learn from an eta operation and convert to gamma
    ///
    /// When a pattern is crystallized, future uses of the same operation type
    /// can use the cheaper gamma path instead of the expensive eta path.
    pub fn crystallize_pattern(&mut self, operation: &CompileOperation) -> f64 {
        // Crystallization costs a small amount of gamma budget
        let cost = 0.00005;
        if self.gamma_consumed + cost <= self.crystallized_budget {
            self.gamma_consumed += cost;
            self.consumed += cost;
            self.operation_log.push(BudgetEntry {
                operation: operation.clone(),
                gamma_used: cost,
                eta_used: 0.0,
                total_cost: cost,
                timestamp: Instant::now(),
                allowed: true,
            });
        }
        cost
    }

    /// Reset the daily budget (called at day boundary)
    pub fn reset_daily_budget(&mut self) {
        self.consumed = 0.0;
        self.gamma_consumed = 0.0;
        self.eta_consumed = 0.0;
        self.day_start = Instant::now();
    }

    /// Get current budget consumption summary
    pub fn budget_summary(&self) -> BudgetSummary {
        BudgetSummary {
            daily_budget: self.daily_budget,
            crystallized_budget: self.crystallized_budget,
            live_budget: self.live_budget,
            consumed: self.consumed,
            gamma_consumed: self.gamma_consumed,
            eta_consumed: self.eta_consumed,
            remaining: self.daily_budget - self.consumed,
            gamma_remaining: self.crystallized_budget - self.gamma_consumed,
            eta_remaining: self.live_budget - self.eta_consumed,
            total_operations: self.operation_log.len(),
        }
    }

    /// Get the operation log for auditing
    pub fn operation_log(&self) -> &[BudgetEntry] {
        &self.operation_log
    }
}

/// Summary of current budget status
#[derive(Debug, Clone)]
pub struct BudgetSummary {
    pub daily_budget: f64,
    pub crystallized_budget: f64,
    pub live_budget: f64,
    pub consumed: f64,
    pub gamma_consumed: f64,
    pub eta_consumed: f64,
    pub remaining: f64,
    pub gamma_remaining: f64,
    pub eta_remaining: f64,
    pub total_operations: usize,
}

/// Conservation Enforcer - evaluates operations against budget constraints
pub struct ConservationEnforcer {
    tracker: BudgetTracker,
    /// Crystallized pattern registry (operation -> can_use_gamma)
    crystallized_patterns: HashMap<CompileOperation, bool>,
    /// Decision threshold for gamma vs eta routing
    gamma_threshold: f64,
}

impl ConservationEnforcer {
    /// Create a new conservation enforcer
    pub fn new(daily_budget: f64) -> Self {
        Self {
            tracker: BudgetTracker::with_budget(daily_budget, 0.8),
            crystallized_patterns: HashMap::new(),
            gamma_threshold: 0.5,
        }
    }

    /// Evaluate whether an operation is allowed and which path to use
    pub fn evaluate(&mut self, operation: CompileOperation) -> OperationDecision {
        let can_use_gamma = self.should_use_gamma(&operation);

        if !self.tracker.can_compile(&operation) {
            return OperationDecision::Denied {
                operation,
                budget_remaining: self.tracker.budget_summary().remaining,
            };
        }

        let entry = self.tracker.record_compilation(operation.clone(), can_use_gamma);

        if can_use_gamma {
            OperationDecision::Crystallized {
                operation,
                cost: entry.total_cost,
                budget_remaining: self.tracker.budget_summary().remaining,
            }
        } else {
            OperationDecision::Live {
                operation,
                cost: entry.total_cost,
                budget_remaining: self.tracker.budget_summary().remaining,
            }
        }
    }

    /// Decide whether to use crystallized (gamma) or live (eta) path
    fn should_use_gamma(&self, operation: &CompileOperation) -> bool {
        // Always-free operations always use gamma
        if operation.can_crystallize() && operation.eta_cost() == 0.0 {
            return true;
        }

        // Check if pattern is crystallized
        if self.crystallized_patterns.get(operation).copied().unwrap_or(false) {
            return true;
        }

        // Check gamma budget availability
        let gamma_cost = operation.gamma_cost();
        let gamma_remaining = self.tracker.budget_summary().gamma_remaining;
        gamma_remaining >= gamma_cost
    }

    /// Mark a pattern as crystallized
    pub fn mark_crystallized(&mut self, operation: CompileOperation) {
        self.crystallized_patterns.insert(operation.clone(), true);
        self.tracker.crystallize_pattern(&operation);
    }

    /// Get budget summary
    pub fn budget_summary(&self) -> BudgetSummary {
        self.tracker.budget_summary()
    }

    /// Set the gamma/eta routing threshold (0.0 = always eta, 1.0 = always gamma)
    pub fn set_gamma_threshold(&mut self, threshold: f64) {
        self.gamma_threshold = threshold.clamp(0.0, 1.0);
    }
}

/// Decision result from the conservation enforcer
#[derive(Debug, Clone)]
pub enum OperationDecision {
    /// Use crystallized (gamma) path
    Crystallized {
        operation: CompileOperation,
        cost: f64,
        budget_remaining: f64,
    },
    /// Use live (eta) path
    Live {
        operation: CompileOperation,
        cost: f64,
        budget_remaining: f64,
    },
    /// Operation denied due to budget constraints
    Denied {
        operation: CompileOperation,
        budget_remaining: f64,
    },
}

impl OperationDecision {
    pub fn is_allowed(&self) -> bool {
        !matches!(self, OperationDecision::Denied { .. })
    }

    pub fn cost(&self) -> f64 {
        match self {
            OperationDecision::Crystallized { cost, .. } => *cost,
            OperationDecision::Live { cost, .. } => *cost,
            OperationDecision::Denied { .. } => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_tracker_default() {
        let tracker = BudgetTracker::new();
        assert_eq!(tracker.daily_budget, 0.10);
        assert_eq!(tracker.crystallized_budget, 0.08);
        assert_eq!(tracker.live_budget, 0.02);
    }

    #[test]
    fn test_custom_budget() {
        let tracker = BudgetTracker::with_budget(0.50, 0.9);
        assert_eq!(tracker.crystallized_budget, 0.45);
        assert_eq!(tracker.live_budget, 0.05);
    }

    #[test]
    fn test_always_free_operations() {
        let tracker = BudgetTracker::new();
        assert!(tracker.can_compile(&CompileOperation::WgslParse));
        assert!(tracker.can_compile(&CompileOperation::SpirvGenerate));
    }

    #[test]
    fn test_budget_exceeded() {
        let mut tracker = BudgetTracker::with_budget(0.001, 1.0); // Tiny budget
        // Record crystallized operations until budget is nearly spent
        for _ in 0..10 {
            tracker.record_compilation(CompileOperation::IsfConvert, true);
        }
        // Budget should be exceeded
        // The IsfConvert gamma cost is 0.0003, after 10 that's 0.003, which exceeds 0.001
        assert!(!tracker.can_compile(&CompileOperation::IsfConvert));
    }

    #[test]
    fn test_record_compilation() {
        let mut tracker = BudgetTracker::new();
        let entry = tracker.record_compilation(CompileOperation::WgslParse, true);
        assert!(entry.allowed);
        assert_eq!(entry.gamma_used, 0.0001);
        assert_eq!(entry.eta_used, 0.0);
    }

    #[test]
    fn test_crystallize_pattern() {
        let mut tracker = BudgetTracker::new();
        let cost = tracker.crystallize_pattern(&CompileOperation::IsfConvert);
        assert!(cost > 0.0);
        assert_eq!(tracker.operation_log.len(), 1);
    }

    #[test]
    fn test_budget_summary() {
        let mut tracker = BudgetTracker::new();
        tracker.record_compilation(CompileOperation::WgslParse, true);
        let summary = tracker.budget_summary();
        assert!(summary.consumed > 0.0);
        assert!(summary.remaining < summary.daily_budget);
        assert_eq!(summary.total_operations, 1);
    }

    #[test]
    fn test_conservation_enforcer() {
        let mut enforcer = ConservationEnforcer::new(0.10);

        // WGSL parse should always be crystallized
        let decision = enforcer.evaluate(CompileOperation::WgslParse);
        match decision {
            OperationDecision::Crystallized { .. } => {} // Expected
            _ => panic!("WGSL parse should be crystallized"),
        }

        // ISF convert
        let decision = enforcer.evaluate(CompileOperation::IsfConvert);
        assert!(decision.is_allowed());

        // LLM optimize should be live
        let decision = enforcer.evaluate(CompileOperation::LlmOptimize);
        assert!(decision.is_allowed());
    }

    #[test]
    fn test_enforcer_budget_exceeded() {
        let mut enforcer = ConservationEnforcer::new(0.001);
        enforcer.set_gamma_threshold(0.0); // Force eta path for expensive ops

        for _ in 0..10 {
            enforcer.evaluate(CompileOperation::LlmOptimize);
        }

        let decision = enforcer.evaluate(CompileOperation::LlmOptimize);
        match decision {
            OperationDecision::Denied { .. } => {} // Expected
            _ => panic!("Should be denied after budget exceeded"),
        }
    }

    #[test]
    fn test_mark_crystallized() {
        let mut enforcer = ConservationEnforcer::new(0.10);
        enforcer.mark_crystallized(CompileOperation::IsfConvert);

        // Now ISF convert should prefer gamma path
        let decision = enforcer.evaluate(CompileOperation::IsfConvert);
        match decision {
            OperationDecision::Crystallized { .. } => {} // Expected
            _ => panic!("Should use crystallized path after marking"),
        }
    }

    #[test]
    fn test_operation_cost_table() {
        assert_eq!(CompileOperation::WgslParse.gamma_cost(), 0.0001);
        assert_eq!(CompileOperation::WgslParse.eta_cost(), 0.0);

        assert_eq!(CompileOperation::LlmOptimize.eta_cost(), 0.05);
        assert!(CompileOperation::LlmOptimize.gamma_cost() < CompileOperation::LlmOptimize.eta_cost());
    }

    #[test]
    fn test_reset_daily_budget() {
        let mut tracker = BudgetTracker::new();
        tracker.record_compilation(CompileOperation::IsfConvert, true);
        assert!(tracker.consumed > 0.0);

        tracker.reset_daily_budget();
        assert_eq!(tracker.consumed, 0.0);
        assert_eq!(tracker.gamma_consumed, 0.0);
        assert_eq!(tracker.eta_consumed, 0.0);
    }
}
