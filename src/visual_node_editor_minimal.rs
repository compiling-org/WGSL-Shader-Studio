use bevy_egui::egui::*;
use crate::node_graph::*;

pub struct VisualNodeEditor {
    pub auto_compile: bool,
}

impl Default for VisualNodeEditor {
    fn default() -> Self {
        Self {
            auto_compile: true,
        }
    }
}

impl VisualNodeEditor {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn ui(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        ui.label("Visual Node Editor");
    }
}