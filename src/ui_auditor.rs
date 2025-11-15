use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashSet;

/// Resource flag to enable/disable the UI auditor
#[derive(Resource, Default)]
pub struct UiAuditState {
    pub enabled: bool,
    pub trigger_this_frame: bool,
}

/// What we know about a single panel after inspection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PanelAudit {
    pub name: &'static str,
    pub has_real_widgets: bool,
    pub widget_count: usize,
    pub placeholder_indicators: Vec<&'static str>,
}

impl PanelAudit {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            has_real_widgets: false,
            widget_count: 0,
            placeholder_indicators: vec![],
        }
    }

    pub fn mark_real(&mut self) {
        self.has_real_widgets = true;
    }

    pub fn add_placeholder(&mut self, hint: &'static str) {
        self.placeholder_indicators.push(hint);
    }
}

/// Per-frame collector of audit info
#[derive(Resource, Default)]
pub struct UiAuditCollector {
    panels: HashSet<PanelAudit>,
}

impl UiAuditCollector {
    pub fn clear(&mut self) {
        self.panels.clear();
    }

    pub fn record_panel(&mut self, panel: PanelAudit) {
        self.panels.insert(panel);
    }

    pub fn print_report(&self) {
        let real: Vec<_> = self.panels.iter().filter(|p| p.has_real_widgets).collect();
        let dummy: Vec<_> = self.panels.iter().filter(|p| !p.has_real_widgets).collect();

        println!("\n==========  UI AUDIT REPORT (F12)  ==========");
        println!("Total panels scanned: {}", self.panels.len());
        println!("Working panels: {}  |  Placeholder panels: {}\n", real.len(), dummy.len());

        if !real.is_empty() {
            println!("✅ WORKING PANELS (have real widgets):");
            for p in real {
                println!("   - {}  ({} widgets)", p.name, p.widget_count);
            }
        }

        if !dummy.is_empty() {
            println!("\n❌ PLACEHOLDER PANELS (only stub UI):");
            for p in dummy {
                println!("   - {}  hints: {:?}", p.name, p.placeholder_indicators);
            }
        }
        println!("===============================================\n");
    }
}

/// Helper for panels to self-report
pub struct PanelAuditor<'a> {
    audit: &'a mut PanelAudit,
}

impl<'a> PanelAuditor<'a> {
    pub fn new(audit: &'a mut PanelAudit) -> Self {
        Self { audit }
    }

    /// Call when you draw a real widget
    pub fn real_widget(&mut self) {
        self.audit.mark_real();
        self.audit.widget_count += 1;
    }

    /// Call when you only show placeholder text
    pub fn placeholder(&mut self, hint: &'static str) {
        self.audit.add_placeholder(hint);
    }
}

/// System that listens for F12 and prints the audit
pub fn ui_audit_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut audit_state: ResMut<UiAuditState>,
    mut collector: ResMut<UiAuditCollector>,
) {
    if keys.just_pressed(KeyCode::F12) {
        audit_state.trigger_this_frame = true;
    }

    if audit_state.trigger_this_frame {
        audit_state.trigger_this_frame = false;
        if audit_state.enabled {
            collector.print_report();
        } else {
            println!("[UI Auditor] Press F12 again to enable auditor first.");
        }
    }
}

/// Plugin to add the auditor resources and system
pub struct UiAuditorPlugin;

impl Plugin for UiAuditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAuditState>()
            .init_resource::<UiAuditCollector>()
            .add_systems(Update, ui_audit_system);
    }
}