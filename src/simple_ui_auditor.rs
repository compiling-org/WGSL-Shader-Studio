use bevy::prelude::*;
use bevy_egui::EguiContexts;
use std::collections::HashMap;

/// Simple UI auditor that tracks what's actually rendered vs placeholders
#[derive(Resource, Default)]
pub struct SimpleUiAuditor {
    pub enabled: bool,
    pub panels_found: HashMap<String, PanelInfo>,
}

#[derive(Default, Clone)]
pub struct PanelInfo {
    pub has_real_content: bool,
    pub widget_count: usize,
    pub placeholder_reasons: Vec<String>,
}

impl SimpleUiAuditor {
    pub fn new() -> Self {
        Self {
            enabled: true,
            panels_found: HashMap::new(),
        }
    }

    pub fn record_panel(&mut self, name: &str, has_content: bool, reason: Option<String>) {
        let info = self.panels_found.entry(name.to_string()).or_default();
        info.has_real_content = has_content;
        if let Some(r) = reason {
            info.placeholder_reasons.push(r);
        }
        info.widget_count += 1;
    }

    pub fn print_report(&self) {
        println!("\n==========  UI AUDIT REPORT (F12)  ==========");
        println!("Total panels found: {}", self.panels_found.len());
        
        let working: Vec<_> = self.panels_found.iter()
            .filter(|(_, info)| info.has_real_content)
            .collect();
        let dummy: Vec<_> = self.panels_found.iter()
            .filter(|(_, info)| !info.has_real_content)
            .collect();
            
        println!("Working panels: {}  |  Placeholder panels: {}\n", working.len(), dummy.len());

        if !working.is_empty() {
            println!("✅ WORKING PANELS:");
            for (name, info) in working {
                println!("   - {}  ({} widgets)", name, info.widget_count);
            }
        }

        if !dummy.is_empty() {
            println!("\n❌ PLACEHOLDER PANELS:");
            for (name, info) in dummy {
                println!("   - {}  reasons: {:?}", name, info.placeholder_reasons);
            }
        }
        println!("===============================================\n");
    }

    pub fn clear(&mut self) {
        self.panels_found.clear();
    }
}

/// System that listens for F12 and prints audit report
pub fn ui_audit_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut auditor: ResMut<SimpleUiAuditor>,
) {
    if keys.just_pressed(KeyCode::F12) {
        auditor.print_report();
        auditor.clear(); // Reset for next audit
    }
}

/// Plugin to add the simple auditor
pub struct SimpleUiAuditorPlugin;

impl Plugin for SimpleUiAuditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimpleUiAuditor>()
            .add_systems(Update, ui_audit_system);
    }
}