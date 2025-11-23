use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Enforcement system for preventing psychotic loops and maintaining discipline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementSystem {
    pub session_start: DateTime<Utc>,
    pub violations: Vec<Violation>,
    pub last_activity: DateTime<Utc>,
    pub file_modification_counts: std::collections::HashMap<String, u32>,
    pub max_file_changes_per_session: u32,
    pub max_violations_before_reset: u32,
    pub is_locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub timestamp: DateTime<Utc>,
    pub violation_type: ViolationType,
    pub description: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    ExcessiveFileModifications,
    FrameworkViolation,
    CompilationError,
    FalseCompletionClaim,
    ReferenceIntegrationFailure,
    PsychoticLoopDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Critical,
    Fatal,
}

impl EnforcementSystem {
    pub fn new() -> Self {
        Self {
            session_start: Utc::now(),
            violations: Vec::new(),
            last_activity: Utc::now(),
            file_modification_counts: std::collections::HashMap::new(),
            max_file_changes_per_session: 10,
            max_violations_before_reset: 5,
            is_locked: false,
        }
    }

    pub fn record_file_modification(&mut self, file_path: &str) -> Result<()> {
        let count = self.file_modification_counts.entry(file_path.to_string()).or_insert(0);
        *count += 1;
        self.last_activity = Utc::now();

        if *count > self.max_file_changes_per_session {
            self.add_violation(ViolationType::ExcessiveFileModifications, 
                             format!("File '{}' modified {} times (max: {})", 
                                   file_path, *count, self.max_file_changes_per_session),
                             ViolationSeverity::Critical);
        }

        Ok(())
    }

    pub fn check_framework_compliance(&mut self, code_content: &str) -> Result<bool> {
        // Check for eframe violations
        if code_content.contains("eframe::egui") || code_content.contains("use eframe") {
            self.add_violation(ViolationType::FrameworkViolation,
                             "eframe usage detected - this violates Bevy architecture".to_string(),
                             ViolationSeverity::Fatal);
            self.lock_system();
            return Ok(false);
        }

        // Check for correct bevy_egui usage
        if !code_content.contains("bevy_egui") && code_content.contains("egui") {
            self.add_violation(ViolationType::FrameworkViolation,
                             "egui usage without bevy_egui wrapper detected".to_string(),
                             ViolationSeverity::Critical);
            return Ok(false);
        }

        Ok(true)
    }

    pub fn check_compilation_status(&mut self, has_errors: bool, error_count: usize) -> Result<()> {
        if has_errors {
            self.add_violation(ViolationType::CompilationError,
                             format!("{} compilation errors detected", error_count),
                             if error_count > 10 { ViolationSeverity::Fatal } else { ViolationSeverity::Critical });
        }
        Ok(())
    }

    pub fn check_psychotic_loop_patterns(&mut self, recent_actions: &[String]) -> Result<()> {
        // Detect psychotic loop patterns
        if recent_actions.len() >= 3 {
            let last_three = &recent_actions[recent_actions.len()-3..];
            if last_three.iter().all(|action| action.contains("visual_node_editor")) {
                self.add_violation(ViolationType::PsychoticLoopDetected,
                                 "Visual node editor obsession pattern detected".to_string(),
                                 ViolationSeverity::Fatal);
                self.lock_system();
            }
        }

        // Check for repeated false completion claims
        let false_claims = self.violations.iter()
            .filter(|v| matches!(v.violation_type, ViolationType::FalseCompletionClaim))
            .count();

        if false_claims > 3 {
            self.add_violation(ViolationType::PsychoticLoopDetected,
                             "Repeated false completion claims pattern".to_string(),
                             ViolationSeverity::Fatal);
            self.lock_system();
        }

        Ok(())
    }

    pub fn check_reference_integration(&mut self, has_references: bool) -> Result<()> {
        if !has_references {
            self.add_violation(ViolationType::ReferenceIntegrationFailure,
                             "Missing reference repository integration".to_string(),
                             ViolationSeverity::Critical);
        }
        Ok(())
    }

    fn add_violation(&mut self, violation_type: ViolationType, description: String, severity: ViolationSeverity) {
        let violation = Violation {
            timestamp: Utc::now(),
            violation_type,
            description,
            severity,
        };

        println!("🚨 ENFORCEMENT VIOLATION: {:?} - {}", violation_type, violation.description);
        
        self.violations.push(violation);
        self.last_activity = Utc::now();

        // Check if system should be locked
        let critical_count = self.violations.iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Critical | ViolationSeverity::Fatal))
            .count();

        if critical_count >= self.max_violations_before_reset as usize {
            self.lock_system();
        }
    }

    fn lock_system(&mut self) {
        self.is_locked = true;
        println!("🔒 SYSTEM LOCKED: Too many violations detected. Manual intervention required.");
    }

    pub fn generate_report(&self) -> EnforcementReport {
        let session_duration = Utc::now() - self.session_start;
        let warning_count = self.violations.iter().filter(|v| matches!(v.severity, ViolationSeverity::Warning)).count();
        let critical_count = self.violations.iter().filter(|v| matches!(v.severity, ViolationSeverity::Critical)).count();
        let fatal_count = self.violations.iter().filter(|v| matches!(v.severity, ViolationSeverity::Fatal)).count();

        EnforcementReport {
            session_start: self.session_start,
            session_duration,
            total_violations: self.violations.len(),
            warning_count,
            critical_count,
            fatal_count,
            is_system_locked: self.is_locked,
            top_violations: self.violations.iter().rev().take(5).cloned().collect(),
            file_modification_summary: self.file_modification_counts.clone(),
        }
    }

    pub fn should_reset(&self) -> bool {
        self.is_locked || self.violations.len() > (self.max_violations_before_reset * 2) as usize
    }

    pub fn reset(&mut self) {
        println!("🔄 ENFORCEMENT SYSTEM RESET");
        self.session_start = Utc::now();
        self.violations.clear();
        self.file_modification_counts.clear();
        self.is_locked = false;
        self.last_activity = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementReport {
    pub session_start: DateTime<Utc>,
    pub session_duration: chrono::Duration,
    pub total_violations: usize,
    pub warning_count: usize,
    pub critical_count: usize,
    pub fatal_count: usize,
    pub is_system_locked: bool,
    pub top_violations: Vec<Violation>,
    pub file_modification_summary: std::collections::HashMap<String, u32>,
}

impl std::fmt::Display for EnforcementReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "🔍 ENFORCEMENT REPORT")?;
        writeln!(f, "Session Start: {}", self.session_start.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(f, "Session Duration: {} minutes", self.session_duration.num_minutes())?;
        writeln!(f, "Total Violations: {}", self.total_violations)?;
        writeln!(f, "Warnings: {}", self.warning_count)?;
        writeln!(f, "Critical: {}", self.critical_count)?;
        writeln!(f, "Fatal: {}", self.fatal_count)?;
        writeln!(f, "System Locked: {}", if self.is_system_locked { "YES" } else { "NO" })?;
        
        if !self.top_violations.is_empty() {
            writeln!(f, "\nTop Recent Violations:")?;
            for (i, violation) in self.top_violations.iter().enumerate() {
                writeln!(f, "{}. {:?} - {}", i + 1, violation.violation_type, violation.description)?;
            }
        }

        if !self.file_modification_summary.is_empty() {
            writeln!(f, "\nFile Modification Summary:")?;
            for (file, count) in &self.file_modification_summary {
                writeln!(f, "  {}: {} modifications", file, count)?;
            }
        }

        Ok(())
    }
}

/// Global enforcement system instance
pub static ENFORCEMENT_SYSTEM: once_cell::sync::Lazy<Arc<RwLock<EnforcementSystem>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(EnforcementSystem::new())));

/// Initialize the enforcement system
pub async fn initialize_enforcement() -> Result<()> {
    let system = ENFORCEMENT_SYSTEM.read().await;
    println!("🔍 Enforcement system initialized");
    println!("{}", system.generate_report());
    Ok(())
}

/// Check if current code changes comply with enforcement rules
pub async fn check_code_compliance(code_content: &str, file_path: &str) -> Result<bool> {
    let mut system = ENFORCEMENT_SYSTEM.write().await;
    
    // Record file modification
    system.record_file_modification(file_path)?;
    
    // Check framework compliance
    let is_compliant = system.check_framework_compliance(code_content)?;
    
    Ok(is_compliant)
}

/// Report compilation status
pub async fn report_compilation_status(has_errors: bool, error_count: usize) -> Result<()> {
    let mut system = ENFORCEMENT_SYSTEM.write().await;
    system.check_compilation_status(has_errors, error_count)?;
    Ok(())
}

/// Generate and display enforcement report
pub async fn generate_enforcement_report() -> Result<EnforcementReport> {
    let system = ENFORCEMENT_SYSTEM.read().await;
    Ok(system.generate_report())
}

/// Reset enforcement system if needed
pub async fn reset_enforcement_if_needed() -> Result<bool> {
    let mut system = ENFORCEMENT_SYSTEM.write().await;
    if system.should_reset() {
        system.reset();
        Ok(true)
    } else {
        Ok(false)
    }
}