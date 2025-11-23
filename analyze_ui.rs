use std::collections::{HashMap, HashSet};

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
}

#[derive(Debug, Clone)]
pub struct WgpuDiagnostics {
    pub adapter_info: Option<String>,
    pub device_limits: Option<String>,
    pub surface_capabilities: Option<String>,
    pub backend_type: Option<String>,
    pub initialization_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UiStateDiagnostics {
    pub egui_context_exists: bool,
    pub viewport_size: Option<(f32, f32)>,
    pub panel_layout_valid: bool,
    pub rendering_pipeline_active: bool,
    pub texture_cache_size: usize,
    pub shader_compilation_errors: Vec<String>,
}

pub struct UIAnalyzer {
    features: Vec<FeatureCheck>,
    missing_features: Vec<String>,
    broken_features: Vec<String>,
    partial_features: Vec<String>,
    functional_features: Vec<String>,
    wgpu_diagnostics: WgpuDiagnostics,
    ui_state_diagnostics: UiStateDiagnostics,
    runtime_errors: Vec<String>,
    performance_metrics: HashMap<String, f64>,
}

impl UIAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            features: Vec::new(),
            missing_features: Vec::new(),
            broken_features: Vec::new(),
            partial_features: Vec::new(),
            functional_features: Vec::new(),
            wgpu_diagnostics: WgpuDiagnostics {
                adapter_info: None,
                device_limits: None,
                surface_capabilities: None,
                backend_type: None,
                initialization_error: None,
            },
            ui_state_diagnostics: UiStateDiagnostics {
                egui_context_exists: false,
                viewport_size: None,
                panel_layout_valid: false,
                rendering_pipeline_active: false,
                texture_cache_size: 0,
                shader_compilation_errors: Vec::new(),
            },
            runtime_errors: Vec::new(),
            performance_metrics: HashMap::new(),
        };
        
        analyzer.initialize_comprehensive_checks();
        analyzer
    }

    fn initialize_comprehensive_checks(&mut self) {
        // CORE RENDERING SYSTEMS
        self.features.push(FeatureCheck {
            name: "WGPU Integration".to_string(),
            category: "Core Rendering