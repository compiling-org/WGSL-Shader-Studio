use std::fs;
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PanelAudit {
    name: String,
    status: PanelStatus,
    widgets: Vec<String>,
    issues: Vec<String>,
    line_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum PanelStatus {
    Working,
    Stub,
    Missing,
    Placeholder,
}

struct UiAnalyzer {
    panels: HashMap<String, PanelAudit>,
}

impl UiAnalyzer {
    fn new() -> Self {
        Self {
            panels: HashMap::new(),
        }
    }

    fn analyze_file(&mut self, _filename: &str, content: &str) {
        // Define panel patterns to look for
        let panel_patterns = vec![
            ("Menu Bar", r"menu_bar|editor_menu|Menu"),
            ("Shader Browser", r"shader_browser|Shader Browser|shader.*browser"),
            ("Parameter Panel", r"parameters|Parameter|parameter_panel"),
            ("Code Editor", r"code.*editor|Code Editor|editor_code_panel"),
            ("Preview Panel", r"preview|Preview|shader.*preview"),
            ("Node Editor", r"node.*editor|Node Editor|node_studio"),
            ("Timeline", r"timeline|Timeline"),
            ("Audio Panel", r"audio.*panel|Audio.*Panel|audio_analysis"),
            ("MIDI Panel", r"midi.*panel|MIDI.*Panel|midi_mapping"),
            ("Gesture Panel", r"gesture.*panel|Gesture.*Panel|gesture_control"),
        ];

        for (panel_name, pattern) in panel_patterns {
            let regex = Regex::new(pattern).unwrap();
            if regex.is_match(content) {
                let audit = self.analyze_panel(panel_name, content);
                self.panels.insert(panel_name.to_string(), audit);
            }
        }
    }

    fn analyze_panel(&self, panel_name: &str, content: &str) -> PanelAudit {
        let mut audit = PanelAudit {
            name: panel_name.to_string(),
            status: PanelStatus::Missing,
            widgets: vec![],
            issues: vec![],
            line_count: content.lines().count(),
        };

        // Look for specific widget patterns
        let widget_patterns = vec![
            ("button", r"ui\.button\("),
            ("checkbox", r"ui\.checkbox\("),
            ("text_edit", r"ui\.text_edit\("),
            ("drag_value", r"ui\.drag_value\("),
            ("selectable_label", r"ui\.selectable_label\("),
            ("heading", r"ui\.heading\("),
            ("label", r"ui\.label\("),
            ("separator", r"ui\.separator\("),
            ("scroll_area", r"egui::ScrollArea"),
            ("side_panel", r"egui::SidePanel"),
            ("top_panel", r"egui::TopBottomPanel"),
            ("central_panel", r"egui::CentralPanel"),
            ("window", r"egui::Window"),
            ("menu", r"egui::menu::menu"),
            ("slider", r"ui\.add.*Slider"),
            ("color_picker", r"color_edit_button"),
            ("collapsing", r"ui\.collapsing"),
        ];

        for (widget_name, pattern) in widget_patterns {
            let regex = Regex::new(pattern).unwrap();
            let matches: Vec<_> = regex.find_iter(content).collect();
            if !matches.is_empty() {
                audit.widgets.push(format!("{} ({})", widget_name, matches.len()));
            }
        }

        // Determine status based on content analysis - IMPROVED PATTERNS
        let placeholder_patterns = vec![
            r"placeholder",
            r"todo!()",
            r"TODO!()",
            r"unimplemented!()",
            r"not implemented",
            r"stub!()",
            r"mock",
            r"unimplemented!()",
        ];

        // More specific working patterns for each panel
        let working_patterns = match panel_name {
            "Menu Bar" => vec![
                r"render_menu_bar",
                r"File.*menu",
                r"Save.*file",
                r"Open.*file",
                r"ui\.menu_button",
            ],
            "Shader Browser" => vec![
                r"render_shader_browser",
                r"ISF.*Shader.*Browser",
                r"shaders\.iter\(\)",
                r"select_shader",
                r"shader\.name",
            ],
            "Parameter Panel" => vec![
                r"render_parameter_panel",
                r"Parameters",
                r"input\.input_type",
                r"parameter_values",
                r"InputType::",
            ],
            "Code Editor" => vec![
                r"render_code_editor",
                r"WGSL.*Editor",
                r"syntax.*highlighting",
                r"text_edit_multiline",
                r"wgsl_code",
            ],
            "Preview Panel" => vec![
                r"render_live_preview",
                r"Shader.*Preview",
                r"preview.*texture",
                r"preview.*size",
                r"render.*shader",
            ],
            "Node Editor" => vec![
                r"render_node_editor",
                r"Node.*Editor",
                r"nodes\.iter",
                r"node_type",
                r"connections",
            ],
            "Timeline" => vec![
                r"render_timeline",
                r"Timeline",
                r"time_slider",
                r"animation",
                r"keyframe",
            ],
            "Audio Panel" => vec![
                r"render_audio_panel",
                r"Audio.*Analysis",
                r"audio_system",
                r"frequency.*bins",
                r"amplitude",
            ],
            "MIDI Panel" => vec![
                r"render_midi_panel",
                r"MIDI.*Controller",
                r"midi_mapping",
                r"controller.*input",
                r"midi.*device",
            ],
            "Gesture Panel" => vec![
                r"render_gesture_panel",
                r"Gesture.*Control",
                r"gesture_control",
                r"hand.*tracking",
                r"gesture.*recognition",
            ],
            _ => vec![
                r"ui_state\.",
                r"available_shaders",
                r"shader_parameters",
                r"wgsl_code",
                r"search_query",
                r"selected_shader",
            ],
        };

        let has_placeholders = placeholder_patterns.iter()
            .any(|pattern| Regex::new(pattern).unwrap().is_match(content));

        let has_working_code = working_patterns.iter()
            .any(|pattern| Regex::new(pattern).unwrap().is_match(content));

        let has_widgets = !audit.widgets.is_empty();

        // IMPROVED STATUS DETERMINATION
        audit.status = match (has_widgets, has_working_code, has_placeholders) {
            (true, true, false) => PanelStatus::Working,
            (true, true, true) => PanelStatus::Working, // Has both working code and some TODOs
            (true, false, false) => PanelStatus::Working, // Has widgets and no placeholders
            (true, false, true) => PanelStatus::Placeholder,
            (false, true, false) => PanelStatus::Working, // Has working logic but no UI widgets
            (false, _, _) => PanelStatus::Stub,
            _ => PanelStatus::Placeholder,
        };

        // Add specific issues based on actual implementation analysis
        if panel_name == "Preview Panel" {
            if !content.contains("render_live_preview") && !content.contains("preview.*texture") {
                audit.issues.push("No preview rendering implementation found".to_string());
            } else if content.contains("status text only") || content.contains("placeholder preview") {
                audit.issues.push("Preview shows only status text, no actual rendering".to_string());
            }
        }

        if panel_name == "Audio Panel" {
            if !content.contains("render_audio_panel") && !content.contains("audio_system") {
                audit.issues.push("No audio panel implementation found".to_string());
            } else if content.contains("placeholder") && !content.contains("frequency.*bins") {
                audit.issues.push("Audio analysis is placeholder only".to_string());
            }
        }

        if panel_name == "MIDI Panel" {
            if !content.contains("render_midi_panel") && !content.contains("midi_mapping") {
                audit.issues.push("No MIDI panel implementation found".to_string());
            } else if content.contains("placeholder") && !content.contains("controller.*input") {
                audit.issues.push("MIDI mapping is placeholder only".to_string());
            }
        }

        if panel_name == "Node Editor" {
            if !content.contains("render_node_editor") && !content.contains("nodes.iter") {
                audit.issues.push("No node editor implementation found".to_string());
            } else if content.contains("placeholder") && !content.contains("node_type") {
                audit.issues.push("Node editor is placeholder only".to_string());
            }
        }

        if panel_name == "Timeline" {
            if !content.contains("render_timeline") && !content.contains("time_slider") {
                audit.issues.push("No timeline implementation found".to_string());
            } else if content.contains("placeholder") && !content.contains("animation") {
                audit.issues.push("Timeline is placeholder only".to_string());
            }
        }

        if panel_name == "Gesture Panel" {
            if !content.contains("render_gesture_panel") && !content.contains("gesture_control") {
                audit.issues.push("No gesture panel implementation found".to_string());
            } else if content.contains("placeholder") && !content.contains("hand.*tracking") {
                audit.issues.push("Gesture controls are placeholder only".to_string());
            }
        }

        if panel_name == "Parameter Panel" {
            if !content.contains("render_parameter_panel") && !content.contains("parameter_values") {
                audit.issues.push("No parameter panel implementation found".to_string());
            } else if content.contains("placeholder") && !content.contains("InputType::") {
                audit.issues.push("Parameter mapping is placeholder only".to_string());
            }
        }

        if panel_name == "Shader Browser" {
            if !content.contains("render_shader_browser") && !content.contains("ISF.*Shader.*Browser") {
                audit.issues.push("No shader browser implementation found".to_string());
            } else if content.contains("placeholder") && !content.contains("shaders.iter") {
                audit.issues.push("ISF loader is placeholder only".to_string());
            }
        }

        audit
    }

    fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# WGSL Shader Studio - UI Audit Report\n\n");
        report.push_str("Generated by: `cargo run --bin ui_analyzer`\n\n");
        
        // Summary statistics
        let total_panels = self.panels.len();
        let working_panels = self.panels.values()
            .filter(|p| p.status == PanelStatus::Working)
            .count();
        let stub_panels = self.panels.values()
            .filter(|p| p.status == PanelStatus::Stub || p.status == PanelStatus::Placeholder)
            .count();

        report.push_str(&format!("## Summary\n\n"));
        report.push_str(&format!("- **Total Panels Found:** {}\n", total_panels));
        report.push_str(&format!("- **Working Panels:** {}\n", working_panels));
        report.push_str(&format!("- **Stub/Placeholder Panels:** {}\n", stub_panels));
        report.push_str(&format!("- **Implementation Status:** {:.1}%\n\n", 
            (working_panels as f64 / total_panels as f64) * 100.0));

        // Detailed panel breakdown
        report.push_str("## Panel Details\n\n");
        report.push_str("| Panel | Status | Widgets | Issues |\n");
        report.push_str("|-------|--------|---------|--------|\n");

        let mut sorted_panels: Vec<_> = self.panels.iter().collect();
        sorted_panels.sort_by_key(|(name, _)| name.as_str());

        for (name, audit) in sorted_panels {
            let status_icon = match audit.status {
                PanelStatus::Working => "✅",
                PanelStatus::Stub => "❌",
                PanelStatus::Placeholder => "⚠️",
                PanelStatus::Missing => "❓",
            };

            let status_text = match audit.status {
                PanelStatus::Working => "Working",
                PanelStatus::Stub => "Stub",
                PanelStatus::Placeholder => "Placeholder",
                PanelStatus::Missing => "Missing",
            };

            let widgets_text = if audit.widgets.is_empty() {
                "None".to_string()
            } else {
                audit.widgets.join(", ")
            };

            let issues_text = if audit.issues.is_empty() {
                "None".to_string()
            } else {
                audit.issues.join("; ")
            };

            report.push_str(&format!(
                "| {} | {} {} | {} | {} |\n",
                name, status_icon, status_text, widgets_text, issues_text
            ));
        }

        // Missing major features
        report.push_str("\n## Missing Major Features\n\n");
        
        let expected_features = vec![
            ("Shader Preview Rendering", "Preview Panel"),
            ("Audio Analysis System", "Audio Panel"),
            ("MIDI Controller Integration", "MIDI Panel"),
            ("Node-Based Shader Editor", "Node Editor"),
            ("Timeline Animation", "Timeline"),
            ("Gesture Control", "Gesture Panel"),
            ("Parameter Mapping", "Parameter Panel"),
            ("ISF Loader", "Shader Browser"),
        ];

        for (feature, panel) in expected_features {
            if let Some(audit) = self.panels.get(panel) {
                if audit.status != PanelStatus::Working {
                    report.push_str(&format!("- ❌ **{}**: Panel '{}' is not working\n", feature, panel));
                } else {
                    report.push_str(&format!("- ✅ **{}**: Panel '{}' is functional\n", feature, panel));
                }
            } else {
                report.push_str(&format!("- ❓ **{}**: Panel '{}' not found\n", feature, panel));
            }
        }

        // Code quality notes
        report.push_str("\n## Code Quality Notes\n\n");
        
        let total_lines: usize = self.panels.values().map(|p| p.line_count).sum();
        report.push_str(&format!("- Total UI code lines analyzed: {}\n", total_lines));
        
        let panels_with_widgets: Vec<_> = self.panels.values()
            .filter(|p| !p.widgets.is_empty())
            .collect();
        
        if !panels_with_widgets.is_empty() {
            report.push_str(&format!("- Panels with actual widgets: {}\n", panels_with_widgets.len()));
        }

        let panels_with_issues: Vec<_> = self.panels.values()
            .filter(|p| !p.issues.is_empty())
            .collect();
            
        if !panels_with_issues.is_empty() {
            report.push_str(&format!("- Panels with known issues: {}\n", panels_with_issues.len()));
        }

        report.push_str("\n---\n");
        report.push_str("*This report is generated automatically by analyzing the source code patterns.*\n");

        report
    }
}

fn main() {
    println!("🔍 WGSL Shader Studio UI Analyzer");
    println!("=====================================\n");

    let mut analyzer = UiAnalyzer::new();

    // Analyze main UI files (extended to include egui GUI)
    let files_to_analyze = vec![
        ("src/editor_ui.rs", "Editor UI"),
        ("src/bevy_app.rs", "Bevy App"),
        ("src/gui.rs", "Egui GUI"),
    ];

    for (filename, description) in files_to_analyze {
        println!("Analyzing {}...", description);
        
        match fs::read_to_string(filename) {
            Ok(content) => {
                analyzer.analyze_file(filename, &content);
                println!("  ✅ Found {} panels", analyzer.panels.len());
            }
            Err(e) => {
                println!("  ❌ Error reading {}: {}", filename, e);
            }
        }
    }

    // Generate and print report
    println!("\n{}", analyzer.generate_report());
    
    // Save report to file
    match fs::write("UI_AUDIT_REPORT.md", analyzer.generate_report()) {
        Ok(_) => println!("\n✅ Report saved to UI_AUDIT_REPORT.md"),
        Err(e) => println!("\n❌ Error saving report: {}", e),
    }
}