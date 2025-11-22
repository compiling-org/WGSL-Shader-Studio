use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum FeatureStatus {
    Missing,
    Broken,
    Partial,
    Functional,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct FeatureCheck {
    pub name: String,
    pub category: String,
    pub description: String,
    pub status: FeatureStatus,
    pub details: Vec<String>,
    pub priority: Priority,
    pub dependencies: Vec<String>,
    pub file_locations: Vec<String>,
    pub test_commands: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticIssue {
    pub severity: String,
    pub category: String,
    pub description: String,
    pub location: String,
    pub suggested_fix: String,
    pub related_files: Vec<String>,
}

pub struct UIAnalyzerEnhanced {
    features: Vec<FeatureCheck>,
    diagnostic_issues: Vec<DiagnosticIssue>,
    missing_features: Vec<String>,
    broken_features: Vec<String>,
    partial_features: Vec<String>,
    functional_features: Vec<String>,
}

impl UIAnalyzerEnhanced {
    pub fn new() -> Self {
        let mut analyzer = Self {
            features: Vec::new(),
            diagnostic_issues: Vec::new(),
            missing_features: Vec::new(),
            broken_features: Vec::new(),
            partial_features: Vec::new(),
            functional_features: Vec::new(),
        };
        
        analyzer.initialize_comprehensive_checks();
        analyzer
    }

    fn initialize_comprehensive_checks(&mut self) {
        // CORE RENDERING SYSTEMS
        self.features.push(FeatureCheck {
            name: "WGPU Integration".to_string(),
            category: "Core Rendering".to_string(),
            description: "Direct WebGPU rendering in GUI viewport with high-performance pipeline".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Requires wgpu device initialization".to_string(), "Surface creation for viewport".to_string(), "Render pipeline setup".to_string()],
            priority: Priority::Critical,
            dependencies: vec!["wgpu".to_string(), "bevy".to_string(), "bevy_egui".to_string()],
            file_locations: vec!["src/bevy_app.rs".to_string(), "src/shader_renderer.rs".to_string()],
            test_commands: vec!["cargo check --lib".to_string(), "cargo test wgpu".to_string()],
        });

        self.features.push(FeatureCheck {
            name: "Live Shader Preview".to_string(),
            category: "Core Rendering".to_string(),
            description: "Real-time shader rendering with parameter updates and smooth animation".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Shader compilation pipeline".to_string(), "Uniform buffer updates".to_string(), "Frame timing control".to_string()],
            priority: Priority::Critical,
            dependencies: vec!["WGPU Integration".to_string(), "Shader Compilation".to_string()],
            file_locations: vec!["src/editor_ui.rs".to_string(), "src/shader_renderer.rs".to_string()],
            test_commands: vec!["cargo run --example shader_preview".to_string()],
        });

        self.features.push(FeatureCheck {
            name: "Performance Monitoring".to_string(),
            category: "Core Rendering".to_string(),
            description: "FPS counters, render time tracking with overlay display".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["FPS calculation system".to_string(), "Frame time measurement".to_string(), "Overlay rendering".to_string()],
            priority: Priority::High,
            dependencies: vec!["WGPU Integration".to_string()],
            file_locations: vec!["src/editor_ui.rs".to_string()],
            test_commands: vec!["cargo run --release".to_string()],
        });

        // UI LAYOUT & PANELS
        self.features.push(FeatureCheck {
            name: "Three-Panel Layout".to_string(),
            category: "UI Layout".to_string(),
            description: "Professional three-panel workspace (Center preview, Right controls, Bottom editor)".to_string(),
            status: FeatureStatus::Broken,
            details: vec!["Panel docking system".to_string(), "Resizable panels".to_string(), "Panel visibility toggles".to_string()],
            priority: Priority::Critical,
            dependencies: vec!["bevy_egui".to_string()],
            file_locations: vec!["src/editor_ui.rs".to_string()],
            test_commands: vec!["cargo run --bin layout_test".to_string()],
        });

        self.features.push(FeatureCheck {
            name: "Shader Browser Panel".to_string(),
            category: "UI Layout".to_string(),
            description: "ISF shader library with search, categories, and favorites".to_string(),
            status: FeatureStatus::Missing,
            details: vec![