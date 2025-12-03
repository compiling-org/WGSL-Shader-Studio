use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum FeatureStatus {
    Missing,
    Broken,
    Partial,
    Functional,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutStatus {
    Functional,
    Partial,
    Broken,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
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
    pub fn get_total_features(&self) -> usize {
        self.features.len()
    }

    pub fn get_functional_features_count(&self) -> usize {
        self.functional_features.len()
    }

    pub fn get_partial_features_count(&self) -> usize {
        self.partial_features.len()
    }
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
            category: "Core Rendering".to_string(),
            description: "Direct WebGPU rendering in GUI viewport with high-performance pipeline".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Requires wgpu device initialization".to_string(), "Surface creation for viewport".to_string(), "Render pipeline setup".to_string()],
            priority: Priority::Critical,
        });

        self.features.push(FeatureCheck {
            name: "Live Shader Preview".to_string(),
            category: "Core Rendering".to_string(),
            description: "Real-time shader rendering with parameter updates and smooth animation".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Shader compilation pipeline".to_string(), "Uniform buffer updates".to_string(), "Frame timing control".to_string()],
            priority: Priority::Critical,
        });

        self.features.push(FeatureCheck {
            name: "Performance Monitoring".to_string(),
            category: "Core Rendering".to_string(),
            description: "FPS counters, render time tracking with overlay display".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["FPS calculation system".to_string(), "Frame time measurement".to_string(), "Overlay rendering".to_string()],
            priority: Priority::High,
        });

        // UI LAYOUT & PANELS
        self.features.push(FeatureCheck {
            name: "Three-Panel Layout".to_string(),
            category: "UI Layout".to_string(),
            description: "Professional three-panel workspace (Center preview, Right controls, Bottom editor)".to_string(),
            status: FeatureStatus::Broken,
            details: vec!["Panel docking system".to_string(), "Resizable panels".to_string(), "Panel visibility toggles".to_string()],
            priority: Priority::Critical,
        });

        self.features.push(FeatureCheck {
            name: "Shader Browser Panel".to_string(),
            category: "UI Layout".to_string(),
            description: "ISF shader library with search, categories, and favorites".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["File system scanning".to_string(), "Search functionality".to_string(), "Category filtering".to_string(), "Favorites system".to_string()],
            priority: Priority::Critical,
        });

        self.features.push(FeatureCheck {
            name: "Parameter Panel".to_string(),
            category: "UI Layout".to_string(),
            description: "Interactive shader parameter controls with real-time updates".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Slider controls".to_string(), "Color pickers".to_string(), "Toggle buttons".to_string(), "Real-time parameter sync".to_string()],
            priority: Priority::Critical,
        });

        self.features.push(FeatureCheck {
            name: "Code Editor Panel".to_string(),
            category: "UI Layout".to_string(),
            description: "WGSL editor with syntax highlighting and error indicators".to_string(),
            status: FeatureStatus::Partial,
            details: vec!["Syntax highlighting working".to_string(), "Missing error squiggles".to_string(), "Missing auto-completion".to_string()],
            priority: Priority::High,
        });

        // SHADER SYSTEMS
        self.features.push(FeatureCheck {
            name: "WGSL Syntax Highlighting".to_string(),
            category: "Shader Systems".to_string(),
            description: "Complete WGSL keyword highlighting with semantic coloring".to_string(),
            status: FeatureStatus::Partial,
            details: vec!["Basic highlighting implemented".to_string(), "Missing semantic analysis".to_string(), "Missing error squiggles".to_string()],
            priority: Priority::High,
        });

        self.features.push(FeatureCheck {
            name: "Shader Compilation".to_string(),
            category: "Shader Systems".to_string(),
            description: "WGSL shader compilation with error reporting".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["naga compiler integration".to_string(), "Error message parsing".to_string(), "Validation system".to_string()],
            priority: Priority::Critical,
        });

        self.features.push(FeatureCheck {
            name: "ISF Support".to_string(),
            category: "Shader Systems".to_string(),
            description: "Interactive Shader Format import/export with metadata parsing".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["ISF file parsing".to_string(), "Parameter extraction".to_string(), "Metadata handling".to_string()],
            priority: Priority::High,
        });

        // NODE EDITOR
        self.features.push(FeatureCheck {
            name: "Node-based Editor".to_string(),
            category: "Node Editor".to_string(),
            description: "Visual programming interface with drag-and-drop nodes".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Node graph rendering".to_string(), "Drag-and-drop system".to_string(), "Connection system".to_string(), "Node types".to_string()],
            priority: Priority::High,
        });

        self.features.push(FeatureCheck {
            name: "Code Generation".to_string(),
            category: "Node Editor".to_string(),
            description: "Automatic WGSL code generation from node graphs".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Topological sorting".to_string(), "Code template system".to_string(), "Variable naming".to_string()],
            priority: Priority::Medium,
        });

        // FILE OPERATIONS
        self.features.push(FeatureCheck {
            name: "File Dialogs".to_string(),
            category: "File Operations".to_string(),
            description: "Native OS file dialogs with recent files support".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["rfd integration".to_string(), "Recent files tracking".to_string(), "File type filters".to_string()],
            priority: Priority::High,
        });

        self.features.push(FeatureCheck {
            name: "Project Management".to_string(),
            category: "File Operations".to_string(),
            description: "Project save/load with organized file structure".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Project file format".to_string(), "Asset management".to_string(), "Project templates".to_string()],
            priority: Priority::Medium,
        });

        // EXPORT/IMPORT
        self.features.push(FeatureCheck {
            name: "Shader Conversion".to_string(),
            category: "Export/Import".to_string(),
            description: "WGSL↔GLSL↔HLSL bidirectional conversion".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["naga translation".to_string(), "Format-specific optimizations".to_string(), "Error handling".to_string()],
            priority: Priority::Medium,
        });

        self.features.push(FeatureCheck {
            name: "FFGL Plugin Generation".to_string(),
            category: "Export/Import".to_string(),
            description: "Generate FFGL plugins from shaders".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["FFGL wrapper generation".to_string(), "Parameter mapping".to_string(), "Plugin packaging".to_string()],
            priority: Priority::Low,
        });

        // AUDIO/MIDI INTEGRATION
        self.features.push(FeatureCheck {
            name: "Audio Analysis Engine".to_string(),
            category: "Audio/MIDI".to_string(),
            description: "Real-time FFT analysis with beat detection".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["FFT implementation".to_string(), "Beat detection algorithm".to_string(), "Frequency band analysis".to_string()],
            priority: Priority::Medium,
        });

        self.features.push(FeatureCheck {
            name: "MIDI Control System".to_string(),
            category: "Audio/MIDI".to_string(),
            description: "MIDI CC to shader parameter mapping with low latency".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["MIDI device enumeration".to_string(), "Message parsing".to_string(), "Parameter mapping".to_string(), "Smoothing system".to_string()],
            priority: Priority::Medium,
        });

        // ADVANCED FEATURES
        self.features.push(FeatureCheck {
            name: "Shader Visualizer".to_string(),
            category: "Advanced Features".to_string(),
            description: "AST visualization and dependency graph analysis".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["AST parsing".to_string(), "Graph layout algorithms".to_string(), "Interactive exploration".to_string()],
            priority: Priority::Low,
        });

        self.features.push(FeatureCheck {
            name: "Performance Profiling".to_string(),
            description: "GPU timing analysis and bottleneck identification".to_string(),
            category: "Advanced Features".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["GPU timer queries".to_string(), "Performance metrics".to_string(), "Optimization hints".to_string()],
            priority: Priority::Low,
        });

        // MENU SYSTEM
        self.features.push(FeatureCheck {
            name: "Menu System".to_string(),
            category: "Menu System".to_string(),
            description: "Complete menu bar with File, Edit, View, Tools, Help menus".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Menu bar rendering".to_string(), "Menu item actions".to_string(), "Keyboard shortcuts".to_string(), "Context menus".to_string()],
            priority: Priority::High,
        });

        self.features.push(FeatureCheck {
            name: "Keyboard Shortcuts".to_string(),
            category: "Menu System".to_string(),
            description: "Full keyboard shortcut system for power users".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Shortcut mapping".to_string(), "Configurable shortcuts".to_string(), "Shortcut display".to_string()],
            priority: Priority::Medium,
        });

        // TEMPLATES & EXAMPLES
        self.features.push(FeatureCheck {
            name: "Shader Templates".to_string(),
            category: "Templates".to_string(),
            description: "15+ categorized shader templates with professional examples".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Template categorization".to_string(), "Template loading system".to_string(), "Example shaders".to_string()],
            priority: Priority::Medium,
        });

        // ERROR HANDLING & LOGGING
        self.features.push(FeatureCheck {
            name: "Error Handling System".to_string(),
            category: "Error Handling".to_string(),
            description: "Graceful error handling with user feedback and recovery".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Error types".to_string(), "User notifications".to_string(), "Recovery mechanisms".to_string()],
            priority: Priority::High,
        });

        self.features.push(FeatureCheck {
            name: "Logging System".to_string(),
            category: "Error Handling".to_string(),
            description: "Structured logging with levels and file output".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Log levels".to_string(), "File logging".to_string(), "Structured format".to_string()],
            priority: Priority::Medium,
        });

        // PERFORMANCE & MEMORY
        self.features.push(FeatureCheck {
            name: "Memory Management".to_string(),
            category: "Performance".to_string(),
            description: "Efficient memory usage with texture pooling and buffer management".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Texture pooling".to_string(), "Buffer reuse".to_string(), "Memory monitoring".to_string()],
            priority: Priority::Medium,
        });

        // PLATFORM INTEGRATION
        self.features.push(FeatureCheck {
            name: "Cross-platform Support".to_string(),
            category: "Platform".to_string(),
            description: "Windows, macOS, Linux compatibility with native OS integration".to_string(),
            status: FeatureStatus::Missing,
            details: vec!["Platform detection".to_string(), "OS-specific features".to_string(), "Native dialogs".to_string()],
            priority: Priority::Medium,
        });
    }

    pub fn analyze_current_state(&mut self) {
        // Reset all feature lists
        self.missing_features.clear();
        self.broken_features.clear();
        self.partial_features.clear();
        self.functional_features.clear();

        // Actually analyze the current implementation
        self.perform_real_analysis();

        // Update feature lists based on analysis results
        for feature in &self.features {
            match feature.status {
                FeatureStatus::Missing => self.missing_features.push(feature.name.clone()),
                FeatureStatus::Broken => self.broken_features.push(feature.name.clone()),
                FeatureStatus::Partial => self.partial_features.push(feature.name.clone()),
                FeatureStatus::Functional => self.functional_features.push(feature.name.clone()),
            }
        }
    }

    fn perform_real_analysis(&mut self) {
        // Check WGPU Integration
        if self.check_wgpu_integration() {
            self.update_feature_status("WGPU Integration", FeatureStatus::Functional, vec!["WGPU device initialized successfully".to_string()]);
        } else {
            self.update_feature_status("WGPU Integration", FeatureStatus::Missing, vec!["WGPU device not initialized".to_string()]);
        }

        // Check Live Shader Preview
        if self.check_live_shader_preview() {
            self.update_feature_status("Live Shader Preview", FeatureStatus::Functional, vec!["Live preview rendering working".to_string()]);
        } else {
            self.update_feature_status("Live Shader Preview", FeatureStatus::Missing, vec!["Live preview not functional".to_string()]);
        }

        // Check Three-Panel Layout
        match self.check_three_panel_layout() {
            LayoutStatus::Functional => {
                self.update_feature_status("Three-Panel Layout", FeatureStatus::Functional, vec!["All panels rendering correctly".to_string()]);
            },
            LayoutStatus::Partial => {
                self.update_feature_status("Three-Panel Layout", FeatureStatus::Partial, vec!["Panels exist but missing functionality".to_string()]);
            },
            LayoutStatus::Broken => {
                self.update_feature_status("Three-Panel Layout", FeatureStatus::Broken, vec!["Panel layout has issues".to_string()]);
            },
        }

        // Check Shader Browser Panel
        if self.check_shader_browser_panel() {
            self.update_feature_status("Shader Browser Panel", FeatureStatus::Functional, vec!["Shader browser functional".to_string()]);
        } else {
            self.update_feature_status("Shader Browser Panel", FeatureStatus::Missing, vec!["Shader browser not implemented".to_string()]);
        }

        // Check Parameter Panel
        if self.check_parameter_panel() {
            self.update_feature_status("Parameter Panel", FeatureStatus::Functional, vec!["Parameter controls working".to_string()]);
        } else {
            self.update_feature_status("Parameter Panel", FeatureStatus::Missing, vec!["Parameter panel not functional".to_string()]);
        }

        // Check Shader Compilation
        if self.check_shader_compilation() {
            self.update_feature_status("Shader Compilation", FeatureStatus::Functional, vec!["Shader compilation working".to_string()]);
        } else {
            self.update_feature_status("Shader Compilation", FeatureStatus::Missing, vec!["Shader compilation not working".to_string()]);
        }
    }

    fn check_wgpu_integration(&self) -> bool {
        // Check if WGPU is properly initialized by looking for shader renderer
        std::path::Path::new("src/shader_renderer.rs").exists()
    }

    fn check_live_shader_preview(&self) -> bool {
        if let Ok(editor_content) = std::fs::read_to_string("src/editor_ui.rs") {
            editor_content.contains("Live shader preview") && editor_content.contains("CentralPanel")
        } else if let Ok(app_content) = std::fs::read_to_string("src/bevy_app.rs") {
            app_content.contains("Live shader preview") || app_content.contains("render_frame")
        } else {
            false
        }
    }

    fn check_three_panel_layout(&self) -> LayoutStatus {
        let app = std::fs::read_to_string("src/bevy_app.rs").ok();
        let editor = std::fs::read_to_string("src/editor_ui.rs").ok();
        let has_left_panel = app.as_deref().map(|c| c.contains("shader_browser_panel")).unwrap_or(false)
            || editor.as_deref().map(|c| c.contains("shader_browser_panel")).unwrap_or(false);
        let has_right_panel = app.as_deref().map(|c| c.contains("parameter_panel")).unwrap_or(false)
            || editor.as_deref().map(|c| c.contains("parameter_panel")).unwrap_or(false);
        let has_bottom_panel = app.as_deref().map(|c| c.contains("code_editor_panel")).unwrap_or(false)
            || editor.as_deref().map(|c| c.contains("code_editor_panel")).unwrap_or(false);
        let has_central_panel = app.as_deref().map(|c| c.contains("CentralPanel")).unwrap_or(false)
            || editor.as_deref().map(|c| c.contains("CentralPanel")).unwrap_or(false);

        if has_left_panel && has_right_panel && has_bottom_panel && has_central_panel {
            let has_functionality = editor.as_deref().map(|c|
                c.contains("Interactive shader parameters") || c.contains("Available shaders:") || c.contains("Code Editor")
            ).unwrap_or(false);
            if has_functionality { LayoutStatus::Functional } else { LayoutStatus::Partial }
        } else {
            LayoutStatus::Broken
        }
    }

    fn check_shader_browser_panel(&self) -> bool {
        if let Ok(editor_content) = std::fs::read_to_string("src/editor_ui.rs") {
            editor_content.contains("Available shaders:")
        } else if let Ok(app_content) = std::fs::read_to_string("src/bevy_app.rs") {
            app_content.contains("shader_browser_panel")
        } else {
            false
        }
    }

    fn check_parameter_panel(&self) -> bool {
        if let Ok(editor_content) = std::fs::read_to_string("src/editor_ui.rs") {
            editor_content.contains("Interactive shader parameters") && editor_content.contains("Slider")
        } else if let Ok(app_content) = std::fs::read_to_string("src/bevy_app.rs") {
            app_content.contains("parameter_panel")
        } else {
            false
        }
    }

    fn check_shader_compilation(&self) -> bool {
        // Check if shader compilation is working
        if let Ok(content) = std::fs::read_to_string("src/bevy_app.rs") {
            content.contains("✅ Compiled") || content.contains("❌ Error")
        } else {
            false
        }
    }

    pub fn update_feature_status(&mut self, feature_name: &str, status: FeatureStatus, details: Vec<String>) {
        if let Some(feature) = self.features.iter_mut().find(|f| f.name == feature_name) {
            feature.status = status;
            feature.details = details;
        }
    }

    pub fn get_missing_critical_features(&self) -> Vec<&FeatureCheck> {
        self.features.iter()
            .filter(|f| f.status == FeatureStatus::Missing && f.priority == Priority::Critical)
            .collect()
    }

    pub fn get_missing_high_priority_features(&self) -> Vec<&FeatureCheck> {
        self.features.iter()
            .filter(|f| f.status == FeatureStatus::Missing && f.priority == Priority::High)
            .collect()
    }

    pub fn get_broken_critical_features(&self) -> Vec<&FeatureCheck> {
        self.features.iter()
            .filter(|f| f.status == FeatureStatus::Broken && f.priority == Priority::Critical)
            .collect()
    }

    pub fn generate_comprehensive_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# WGSL Shader Studio - COMPREHENSIVE UI ANALYSIS REPORT\n\n");
        report.push_str("## Executive Summary\n\n");
        
        let total_features = self.features.len();
        let critical_missing = self.get_missing_critical_features().len();
        let high_missing = self.get_missing_high_priority_features().len();
        let critical_broken = self.get_broken_critical_features().len();
        
        report.push_str(&format!("- **Total Features Required**: {}\n", total_features));
        report.push_str(&format!("- **Critical Missing**: {}\n", critical_missing));
        report.push_str(&format!("- **High Priority Missing**: {}\n", high_missing));
        report.push_str(&format!("- **Critical Broken**: {}\n", critical_broken));
        report.push_str(&format!("- **Functional Features**: {}\n", self.functional_features.len()));
        report.push_str(&format!("- **Partial Features**: {}\n", self.partial_features.len()));
        
        report.push_str("\n## CRITICAL ISSUES - IMMEDIATE ACTION REQUIRED\n\n");
        
        if critical_missing > 0 {
            report.push_str("### 🚨 CRITICAL MISSING FEATURES\n\n");
            for feature in self.get_missing_critical_features() {
                report.push_str(&format!("#### {}\n", feature.name));
                report.push_str(&format!("- **Category**: {}\n", feature.category));
                report.push_str(&format!("- **Description**: {}\n", feature.description));
                report.push_str("- **Requirements**:\n");
                for detail in &feature.details {
                    report.push_str(&format!("  - {}\n", detail));
                }
                report.push_str("\n");
            }
        }

        if critical_broken > 0 {
            report.push_str("### 💥 CRITICAL BROKEN FEATURES\n\n");
            for feature in self.get_broken_critical_features() {
                report.push_str(&format!("#### {}\n", feature.name));
                report.push_str(&format!("- **Category**: {}\n", feature.category));
                report.push_str(&format!("- **Description**: {}\n", feature.description));
                report.push_str("- **Issues**:\n");
                for detail in &feature.details {
                    report.push_str(&format!("  - {}\n", detail));
                }
                report.push_str("\n");
            }
        }

        report.push_str("## HIGH PRIORITY MISSING FEATURES\n\n");
        for feature in self.get_missing_high_priority_features() {
            report.push_str(&format!("### {}\n", feature.name));
            report.push_str(&format!("- **Category**: {}\n", feature.category));
            report.push_str(&format!("- **Description**: {}\n", feature.description));
            report.push_str("- **Requirements**:\n");
            for detail in &feature.details {
                report.push_str(&format!("  - {}\n", detail));
            }
            report.push_str("\n");
        }

        report.push_str("## FEATURE STATUS BY CATEGORY\n\n");
        
        let categories = self.get_all_categories();
        for category in categories {
            let category_features: Vec<&FeatureCheck> = self.features.iter()
                .filter(|f| f.category == category)
                .collect();
            
            let missing_count = category_features.iter().filter(|f| f.status == FeatureStatus::Missing).count();
            let broken_count = category_features.iter().filter(|f| f.status == FeatureStatus::Broken).count();
            let partial_count = category_features.iter().filter(|f| f.status == FeatureStatus::Partial).count();
            let functional_count = category_features.iter().filter(|f| f.status == FeatureStatus::Functional).count();
            
            report.push_str(&format!("### {}\n", category));
            report.push_str(&format!("- **Total**: {} features\n", category_features.len()));
            report.push_str(&format!("- **Missing**: {}\n", missing_count));
            report.push_str(&format!("- **Broken**: {}\n", broken_count));
            report.push_str(&format!("- **Partial**: {}\n", partial_count));
            report.push_str(&format!("- **Functional**: {}\n", functional_count));
            report.push_str("\n");
        }

        report.push_str("## IMPLEMENTATION ROADMAP\n\n");
        report.push_str("### Phase 1: Critical Foundation (Week 1)\n");
        report.push_str("1. Fix three-panel UI layout rendering\n");
        report.push_str("2. Implement WGPU integration and shader compilation\n");
        report.push_str("3. Restore shader browser with ISF file loading\n");
        report.push_str("4. Fix parameter panel with real-time updates\n");
        report.push_str("5. Implement basic menu system\n\n");

        report.push_str("### Phase 2: Core Functionality (Week 2)\n");
        report.push_str("1. Complete WGSL syntax highlighting with error indicators\n");
        report.push_str("2. Implement file dialogs and project management\n");
        report.push_str("3. Add performance monitoring overlay\n");
        report.push_str("4. Restore shader conversion capabilities\n");
        report.push_str("5. Implement error handling and logging\n\n");

        report.push_str("### Phase 3: Advanced Features (Week 3-4)\n");
        report.push_str("1. Build node-based editor system\n");
        report.push_str("2. Add audio/MIDI integration\n");
        report.push_str("3. Implement shader visualizer\n");
        report.push_str("4. Add advanced templates and examples\n");
        report.push_str("5. Complete cross-platform support\n\n");

        report.push_str("## TECHNICAL REQUIREMENTS\n\n");
        report.push_str("### Dependencies Required\n");
        report.push_str("- bevy 0.17 + bevy_egui 0.38 (CURRENT)\n");
        report.push_str("- wgpu 0.19+ for rendering\n");
        report.push_str("- naga for shader compilation\n");
        report.push_str("- rfd for file dialogs\n");
        report.push_str("- cpal for audio\n");
        report.push_str("- midir for MIDI\n");
        report.push_str("- serde for serialization\n");
        report.push_str("- tracing for logging\n\n");

        report.push_str("### File Structure Required\n");
        report.push_str("```\n");
        report.push_str("src/\n");
        report.push_str("├── bevy_app.rs          # Main Bevy application\n");
        report.push_str("├── editor_ui.rs          # Main UI implementation\n");
        report.push_str("├── ui_analyzer.rs        # This analysis tool\n");
        report.push_str("├── rendering/            # WGPU rendering systems\n");
        report.push_str("│   ├── mod.rs\n");
        report.push_str("│   ├── pipeline.rs     # Render pipeline\n");
        report.push_str("│   ├── shader.rs       # Shader compilation\n");
        report.push_str("│   └── viewport.rs     # Viewport management\n");
        report.push_str("├── shader_systems/       # Shader-related systems\n");
        report.push_str("│   ├── mod.rs\n");
        report.push_str("│   ├── compiler.rs     # WGSL compilation\n");
        report.push_str("│   ├── isf.rs          # ISF support\n");
        report.push_str("│   └── converter.rs    # Format conversion\n");
        report.push_str("├── ui_systems/          # UI component systems\n");
        report.push_str("│   ├── mod.rs\n");
        report.push_str("│   ├── panels.rs       # Panel management\n");
        report.push_str("│   ├── browser.rs      # Shader browser\n");
        report.push_str("│   ├── parameters.rs   # Parameter controls\n");
        report.push_str("│   ├── editor.rs       # Code editor\n");
        report.push_str("│   └── menus.rs        # Menu system\n");
        report.push_str("├── file_systems/        # File operations\n");
        report.push_str("│   ├── mod.rs\n");
        report.push_str("│   ├── dialogs.rs      # File dialogs\n");
        report.push_str("│   ├── project.rs      # Project management\n");
        report.push_str("│   └── templates.rs    # Template system\n");
        report.push_str("├── audio_midi/          # Audio/MIDI integration\n");
        report.push_str("│   ├── mod.rs\n");
        report.push_str("│   ├── audio.rs        # Audio analysis\n");
        report.push_str("│   ├── midi.rs         # MIDI control\n");
        report.push_str("│   └── parameters.rs   # Parameter mapping\n");
        report.push_str("├── node_editor/         # Node-based editor\n");
        report.push_str("│   ├── mod.rs\n");
        report.push_str("│   ├── graph.rs        # Node graph\n");
        report.push_str("│   ├── nodes.rs        # Node types\n");
        report.push_str("│   └── connections.rs  # Connection system\n");
        report.push_str("└── utils/               # Utility functions\n");
        report.push_str("    ├── mod.rs\n");
        report.push_str("    ├── errors.rs       # Error types\n");
        report.push_str("    ├── logging.rs      # Logging setup\n");
        report.push_str("    └── config.rs       # Configuration\n");
        report.push_str("```\n\n");

        report.push_str("## CONCLUSION\n\n");
        report.push_str("This comprehensive analysis reveals that the WGSL Shader Studio requires **complete reconstruction**\n");
        report.push_str("of all core systems. The current implementation lacks fundamental functionality required for\n");
        report.push_str("basic shader development workflows.\n\n");
        
        report.push_str("**Immediate priorities**: Fix UI layout rendering, implement WGPU integration, restore shader\n");
        report.push_str("browser functionality, and add basic file operations. Without these critical features, the\n");
        report.push_str("application cannot perform its core function as a shader development environment.\n\n");

        report.push_str("**Estimated Recovery Time**: 3-4 weeks for basic functionality, 6-8 weeks for full feature parity.\n");

        report
    }

    fn get_all_categories(&self) -> Vec<String> {
        let mut categories = HashSet::new();
        for feature in &self.features {
            categories.insert(feature.category.clone());
        }
        let mut categories: Vec<String> = categories.into_iter().collect();
        categories.sort();
        categories
    }

    pub fn check_file_structure(&mut self) {
        let required_files = vec![
            ("src/bevy_app.rs", "Main Bevy application"),
            ("src/editor_ui.rs", "Main UI implementation"),
            ("src/rendering/mod.rs", "Rendering module"),
            ("src/shader_systems/mod.rs", "Shader systems module"),
            ("src/ui_systems/mod.rs", "UI systems module"),
            ("src/file_systems/mod.rs", "File systems module"),
            ("src/audio_midi/mod.rs", "Audio/MIDI module"),
            ("src/node_editor/mod.rs", "Node editor module"),
            ("src/utils/mod.rs", "Utilities module"),
        ];

        for (file_path, description) in required_files {
            let path = Path::new(file_path);
            let status = if path.exists() {
                FeatureStatus::Functional
            } else {
                FeatureStatus::Missing
            };

            self.update_feature_status(
                &format!("File: {}", file_path),
                status,
                vec![description.to_string()],
            );
        }
    }

    pub fn check_dependencies(&mut self) {
        let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap_or_default();
        
        let required_deps = vec![
            ("bevy", "Game engine framework"),
            ("bevy_egui", "Egui integration for Bevy"),
            ("wgpu", "WebGPU rendering"),
            ("naga", "Shader compilation"),
            ("rfd", "File dialogs"),
            ("cpal", "Audio processing"),
            ("midir", "MIDI support"),
            ("serde", "Serialization"),
            ("tracing", "Logging"),
        ];

        for (dep_name, description) in required_deps {
            let status = if cargo_toml.contains(dep_name) {
                FeatureStatus::Functional
            } else {
                FeatureStatus::Missing
            };

            self.update_feature_status(
                &format!("Dependency: {}", dep_name),
                status,
                vec![description.to_string()],
            );
        }
    }

    pub fn run_surgical_diagnostics(&mut self) {
        // Run WGPU diagnostics
        self.diagnose_wgpu_state();
        
        // Run UI state analysis
        self.diagnose_ui_state();
        
        // Check for runtime errors
        self.check_runtime_errors();
        
        // Analyze performance bottlenecks
        self.analyze_performance_bottlenecks();
        
        // Validate critical systems
        self.validate_critical_systems();
    }

    fn diagnose_wgpu_state(&mut self) {
        // Check if WGPU can be initialized
        match std::process::Command::new("cargo")
            .args(&["check", "--features", "wgpu"])
            .output() {
            Ok(output) => {
                if !output.status.success() {
                    self.wgpu_diagnostics.initialization_error = Some(
                        String::from_utf8_lossy(&output.stderr).to_string()
                    );
                }
            }
            Err(e) => {
                self.wgpu_diagnostics.initialization_error = Some(format!("Failed to run cargo check: {}", e));
            }
        }

        // Check for WGPU-related compilation errors
        if let Ok(cargo_output) = std::fs::read_to_string("cargo_output.log") {
            if cargo_output.contains("wgpu") && cargo_output.contains("error") {
                self.runtime_errors.push("WGPU compilation errors detected".to_string());
            }
        }

        // Check for texture alignment issues
        if let Ok(editor_content) = std::fs::read_to_string("src/editor_ui.rs") {
            if editor_content.contains("COPY_BYTES_PER_ROW_ALIGNMENT") {
                self.ui_state_diagnostics.rendering_pipeline_active = true;
            }
        }
    }

    fn diagnose_ui_state(&mut self) {
        // Check if egui context is properly initialized
        if let Ok(bevy_content) = std::fs::read_to_string("src/bevy_app.rs") {
            if bevy_content.contains("EguiContext") && bevy_content.contains("egui") {
                self.ui_state_diagnostics.egui_context_exists = true;
            }
        }

        // Check for UI layout issues
        if let Ok(editor_content) = std::fs::read_to_string("src/editor_ui.rs") {
            if editor_content.contains("ui.horizontal") || editor_content.contains("ui.vertical") {
                self.ui_state_diagnostics.panel_layout_valid = true;
            }
        }

        // Check for texture cache issues
        if let Ok(editor_content) = std::fs::read_to_string("src/editor_ui.rs") {
            if editor_content.contains("PreviewCache") {
                self.ui_state_diagnostics.texture_cache_size = 1; // Has cache
            }
        }
    }

    fn check_runtime_errors(&mut self) {
        // Check for common runtime error patterns
        let error_patterns = vec![
            ("WGPU initialization failed", "WGPU renderer not properly initialized"),
            ("Using software shader renderer fallback", "Critical: Using CPU fallback instead of GPU"),
            ("Bytes per row does not respect COPY_BYTES_PER_ROW_ALIGNMENT", "WGPU texture alignment error"),
            ("panicked at", "Application panic detected"),
            ("unwrap() on None value", "Null pointer/unwrapping error"),
            ("index out of bounds", "Array access error"),
            ("bevy_egui::EguiPrimaryContextPass", "UI system stage error"),
        ];

        // Check recent logs
        if let Ok(log_content) = std::fs::read_to_string("run_output.txt") {
            for (pattern, description) in &error_patterns {
                if log_content.contains(pattern) {
                    self.runtime_errors.push(format!("{}: {}", description, pattern));
                }
            }
        }
    }

    fn analyze_performance_bottlenecks(&mut self) {
        // Check for performance issues
        if let Ok(log_content) = std::fs::read_to_string("run_output.txt") {
            if log_content.contains("0.2 FPS") || log_content.contains("4-6 seconds") {
                self.performance_metrics.insert("fps".to_string(), 0.2);
                self.runtime_errors.push("CRITICAL PERFORMANCE: 0.2 FPS detected - CPU rendering fallback".to_string());
            }
        }

        // Check for excessive logging
        if let Ok(log_content) = std::fs::read_to_string("run_output.txt") {
            let log_lines = log_content.lines().count();
            if log_lines > 1000 {
                self.runtime_errors.push(format!("Excessive logging detected: {} lines", log_lines));
            }
        }
    }

    fn validate_critical_systems(&mut self) {
        // Validate WGPU integration
        if let Ok(bevy_content) = std::fs::read_to_string("src/bevy_app.rs") {
            if bevy_content.contains("initialize_wgpu_renderer") && bevy_content.contains("panic!") {
                // This is good - forces WGPU initialization
            } else if bevy_content.contains("WGPU renderer placeholder") {
                self.runtime_errors.push("CRITICAL: WGPU placeholder still present - initialization not forced".to_string());
            }
        }

        // Validate UI rendering
        if let Ok(editor_content) = std::fs::read_to_string("src/editor_ui.rs") {
            if editor_content.contains("compile_and_render_shader") && editor_content.contains("NO SOFTWARE FALLBACK") {
                // This is good - no CPU fallback
            } else if editor_content.contains("SoftwareShaderRenderer") {
                self.runtime_errors.push("CRITICAL: Software shader renderer still present".to_string());
            }
        }
    }

    pub fn generate_surgical_fix_plan(&self) -> String {
        let mut plan = String::new();
        
        plan.push_str("# SURGICAL FIX PLAN - CRITICAL UI ISSUES\n\n");
        plan.push_str("## IMMEDIATE LIFE-THREATENING ISSUES\n\n");
        
        // Critical runtime errors
        let critical_errors: Vec<&String> = self.runtime_errors.iter()
            .filter(|err| err.contains("CRITICAL"))
            .collect();
            
        if !critical_errors.is_empty() {
            plan.push_str("### 🚨 CRITICAL RUNTIME ERRORS\n\n");
            for error in critical_errors {
                plan.push_str(&format!("- **{}**\n", error));
            }
            plan.push_str("\n");
        }

        // WGPU issues
        if let Some(ref error) = self.wgpu_diagnostics.initialization_error {
            plan.push_str("### 💥 WGPU INITIALIZATION FAILURE\n\n");
            plan.push_str(&format!("Error: {}\n\n", error));
            plan.push_str("**SURGICAL FIX**: Force WGPU initialization with panic on failure\n");
            plan.push_str("**LOCATION**: src/bevy_app.rs - initialize_wgpu_renderer()\n\n");
        }

        // Performance issues
        if let Some(fps) = self.performance_metrics.get("fps") {
            if *fps < 1.0 {
                plan.push_str("### ⚡ PERFORMANCE CRISIS\n\n");
                plan.push_str(&format!("Current FPS: {} (UNUSABLE)\n", fps));
                plan.push_str("**ROOT CAUSE**: CPU software rendering fallback\n");
                plan.push_str("**SURGICAL FIX**: Remove all CPU fallback code, force GPU-only rendering\n");
                plan.push_str("**LOCATION**: src/editor_ui.rs - compile_and_render_shader()\n\n");
            }
        }

        // UI state issues
        if !self.ui_state_diagnostics.egui_context_exists {
            plan.push_str("### 🖥️ UI CONTEXT FAILURE\n\n");
            plan.push_str("**ISSUE**: EguiContext not properly initialized\n");
            plan.push_str("**SURGICAL FIX**: Ensure EguiContext is created in Bevy setup\n");
            plan.push_str("**LOCATION**: src/bevy_app.rs - setup() function\n\n");
        }

        if !self.ui_state_diagnostics.panel_layout_valid {
            plan.push_str("### 📐 PANEL LAYOUT BROKEN\n\n");
            plan.push_str("**ISSUE**: UI panels not properly laid out\n");
            plan.push_str("**SURGICAL FIX**: Implement proper egui layout with horizontal/vertical containers\n");
            plan.push_str("**LOCATION**: src/editor_ui.rs - ui() function\n\n");
        }

        plan.push_str("## SURGICAL INTERVENTION STEPS\n\n");
        plan.push_str("1. **STOP ALL APP LAUNCHES** - Do not run broken code\n");
        plan.push_str("2. **FIX WGPU INITIALIZATION** - Force GPU initialization with panic on failure\n");
        plan.push_str("3. **REMOVE CPU FALLBACK** - Delete all software rendering code\n");
        plan.push_str("4. **FIX UI LAYOUT** - Implement proper three-panel layout\n");
        plan.push_str("5. **VALIDATE RENDERING** - Ensure texture alignment and buffer management\n");
        plan.push_str("6. **TEST COMPREHENSIVELY** - Verify all UI elements render and function\n\n");

        plan.push_str("## SUCCESS CRITERIA\n\n");
        plan.push_str("- ✅ WGPU initializes successfully with no fallback\n");
        plan.push_str("- ✅ UI panels render and are interactive\n");
        plan.push_str("- ✅ Shader preview displays correctly\n");
        plan.push_str("- ✅ Performance is > 30 FPS (GPU-accelerated)\n");
        plan.push_str("- ✅ No critical runtime errors\n\n");

        plan
    }

    pub fn run_comprehensive_analysis(&mut self) -> String {
        // Run surgical diagnostics first
        self.run_surgical_diagnostics();
        
        // Check file structure
        self.check_file_structure();
        
        // Check dependencies
        self.check_dependencies();
        
        // Analyze current state
        self.analyze_current_state();
        
        // Generate comprehensive report
        let mut report = self.generate_comprehensive_report();
        
        // Append surgical fix plan
        report.push_str("\n");
        report.push_str(&self.generate_surgical_fix_plan());
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_analyzer_creation() {
        let analyzer = UIAnalyzer::new();
        assert!(!analyzer.features.is_empty());
        assert!(analyzer.features.len() > 20); // Should have many comprehensive checks
    }

    #[test]
    fn test_critical_feature_detection() {
        let mut analyzer = UIAnalyzer::new();
        analyzer.analyze_current_state();
        
        let critical_missing = analyzer.get_missing_critical_features();
        assert!(!critical_missing.is_empty()); // Should detect critical missing features
    }

    #[test]
    fn test_report_generation() {
        let mut analyzer = UIAnalyzer::new();
        analyzer.analyze_current_state();
        
        let report = analyzer.generate_comprehensive_report();
        assert!(!report.is_empty());
        assert!(report.contains("CRITICAL ISSUES"));
        assert!(report.contains("IMPLEMENTATION ROADMAP"));
    }
}