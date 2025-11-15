use bevy::prelude::*;

/// Simple UI auditor that tracks what's actually rendered vs placeholders
#[derive(Resource, Default)]
pub struct MinimalUiAuditor {
    pub enabled: bool,
    pub panels_found: Vec<String>,
    pub working_panels: Vec<String>,
    pub placeholder_panels: Vec<String>,
}

impl MinimalUiAuditor {
    pub fn new() -> Self {
        Self {
            enabled: true,
            panels_found: Vec::new(),
            working_panels: Vec::new(),
            placeholder_panels: Vec::new(),
        }
    }

    pub fn record_panel(&mut self, name: &str, has_content: bool) {
        self.panels_found.push(name.to_string());
        if has_content {
            self.working_panels.push(name.to_string());
        } else {
            self.placeholder_panels.push(name.to_string());
        }
    }

    pub fn print_report(&self) {
        println!("\n==========  UI AUDIT REPORT (F12)  ==========");
        println!("Total panels found: {}", self.panels_found.len());
        println!("Working panels: {}  |  Placeholder panels: {}\n", 
                 self.working_panels.len(), self.placeholder_panels.len());

        if !self.working_panels.is_empty() {
            println!("✅ WORKING PANELS:");
            for panel in &self.working_panels {
                println!("   - {}", panel);
            }
        }

        if !self.placeholder_panels.is_empty() {
            println!("\n❌ PLACEHOLDER PANELS:");
            for panel in &self.placeholder_panels {
                println!("   - {}", panel);
            }
        }
        println!("===============================================\n");
    }

    pub fn clear(&mut self) {
        self.panels_found.clear();
        self.working_panels.clear();
        self.placeholder_panels.clear();
    }
}

/// System that listens for F12 and prints audit report
pub fn ui_audit_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    auditor: Res<MinimalUiAuditor>,
) {
    if keys.just_pressed(KeyCode::F12) {
        auditor.print_report();
    }
}

/// System that audits UI panels based on EditorUiState
pub fn ui_audit_system(
    ui_state: Res<crate::editor_ui::EditorUiState>,
    mut auditor: ResMut<MinimalUiAuditor>,
) {
    auditor.clear();
    
    // Audit Menu Bar - always present
    auditor.record_panel("Menu Bar", true);
    
    // Audit Shader Browser
    if ui_state.show_shader_browser {
        if !ui_state.available_shaders_all.is_empty() {
            auditor.record_panel("Shader Browser", true);
        } else {
            auditor.record_panel("Shader Browser", false);
        }
    }
    
    // Audit Parameter Panel
    if ui_state.show_parameter_panel {
        if ui_state.selected_shader.is_some() && !ui_state.shader_parameters.is_empty() {
            auditor.record_panel("Parameter Panel", true);
        } else {
            auditor.record_panel("Parameter Panel", false);
        }
    }
    
    // Audit Code Editor
    if ui_state.show_code_editor {
        if !ui_state.wgsl_code.is_empty() {
            auditor.record_panel("Code Editor", true);
        } else {
            auditor.record_panel("Code Editor", false);
        }
    }
    
    // Audit Preview Panel
    if ui_state.show_preview {
        auditor.record_panel("Preview Panel", false); // Not implemented
    }
    
    // Audit Node Editor
    if ui_state.show_node_studio {
        auditor.record_panel("Node Editor", false); // Not implemented
    }
    
    // Audit Timeline
    if ui_state.show_timeline {
        auditor.record_panel("Timeline", false); // Not implemented
    }
    
    // Audit Audio Panel
    if ui_state.show_audio_panel {
        auditor.record_panel("Audio Panel", false); // Not implemented
    }
}

/// Plugin to add the minimal auditor
pub struct MinimalUiAuditorPlugin;

impl Plugin for MinimalUiAuditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimalUiAuditor>()
            .add_systems(Update, ui_audit_keyboard_system)
            .add_systems(Update, ui_audit_system);
    }
}