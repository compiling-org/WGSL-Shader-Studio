use egui::*;
use crate::node_graph::{NodeGraph, NodeId, NodeKind, PortId};
use std::collections::HashMap;

pub struct VisualNodeEditor {
    node_positions: HashMap<NodeId, (f32, f32)>,
    connection_start: Option<(NodeId, PortId, bool)>, // node_id, port_id, is_output
    selected_node: Option<NodeId>,
    pan: Vec2,
    zoom: f32,
    auto_compile: bool,
    last_generated_wgsl: Option<String>,
    compilation_errors: Vec<String>,
}

impl VisualNodeEditor {
    pub fn new() -> Self {
        Self {
            node_positions: HashMap::new(),
            connection_start: None,
            selected_node: None,
            pan: Vec2::ZERO,
            zoom: 1.0,
            auto_compile: true,
            last_generated_wgsl: None,
            compilation_errors: Vec::new(),
        }
    }

    pub fn generate_and_compile(&mut self, node_graph: &NodeGraph, width: u32, height: u32) -> Result<String, Vec<String>> {
        match node_graph.generate_wgsl() {
            Ok(wgsl) => {
                self.last_generated_wgsl = Some(wgsl.clone());
                self.compilation_errors.clear();
                Ok(wgsl)
            }
            Err(errors) => {
                self.compilation_errors = errors.clone();
                Err(errors)
            }
        }
    }
    
    pub fn auto_compile_if_needed(&mut self, node_graph: &NodeGraph, width: u32, height: u32) -> Option<Result<String, Vec<String>>> {
        if self.auto_compile {
            return Some(self.generate_and_compile(node_graph, width, height));
        }
        None
    }

    pub fn ui(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        // Execution control panel at the top
        ui.horizontal(|ui| {
            ui.label("Node Editor");
            ui.separator();
            
            // Auto-compile toggle
            ui.checkbox(&mut self.auto_compile, "Auto Compile");
            
            // Manual compile button
            if ui.button("Compile Now").clicked() {
                match self.generate_and_compile(node_graph, 512, 512) {
                    Ok(wgsl) => {
                        ui.label(format!("✅ Compiled successfully ({} chars)", wgsl.len()));
                    }
                    Err(errors) => {
                        ui.label(format!("❌ {} errors", errors.len()));
                        for error in &errors {
                            ui.label(format!("  • {}", error));
                        }
                    }
                }
            }
            
            // Show compilation status
            if !self.compilation_errors.is_empty() {
                ui.label(format!("❌ {} errors", self.compilation_errors.len()));
            } else if self.last_generated_wgsl.is_some() {
                ui.label("✅ Ready");
            }
            
            // Quick node creation buttons
            ui.separator();
            ui.label("Add Node:");
            if ui.button("Time").clicked() {
                let id = node_graph.add_node(NodeKind::Time, "Time", (100.0, 100.0));
                self.node_positions.insert(id, (100.0, 100.0));
            }
            if ui.button("UV").clicked() {
                let id = node_graph.add_node(NodeKind::UV, "UV", (200.0, 100.0));
                self.node_positions.insert(id, (200.0, 100.0));
            }
            if ui.button("Sin").clicked() {
                let id = node_graph.add_node(NodeKind::Sin, "Sin", (300.0, 100.0));
                self.node_positions.insert(id, (300.0, 100.0));
            }
            if ui.button("Vec2").clicked() {
                let id = node_graph.add_node(NodeKind::Vec2, "Vec2", (400.0, 100.0));
                self.node_positions.insert(id, (400.0, 100.0));
            }
            if ui.button("Output").clicked() {
                let id = node_graph.add_node(NodeKind::Output, "Output", (500.0, 100.0));
                self.node_positions.insert(id, (500.0, 100.0));
            }
        });

        ui.separator();

        // Node editor canvas
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Handle pan and zoom
        if response.dragged() {
            self.pan += response.drag_delta();
        }
        
        if response.hovered() {
            let zoom_delta = ui.input().scroll_delta.y * 0.01;
            self.zoom = (self.zoom + zoom_delta).clamp(0.1, 5.0);
        }

        // Draw grid
        self.draw_grid(ui, available_rect);

        // Draw connections first (behind nodes)
        self.draw_connections(ui, node_graph);

        // Draw nodes
        let node_ids: Vec<NodeId> = node_graph.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.draw_node(ui, node_id, node_graph);
        }

        // Draw connection being created
        if let Some((start_node, start_port, is_output)) = self.connection_start {
            self.draw_active_connection(ui, node_graph, start_node, start_port, is_output);
        }
    }

    fn draw_grid(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        let grid_size = 20.0 * self.zoom;
        
        if grid_size < 2.0 {
            return; // Grid too dense
        }
        
        let grid_alpha = (self.zoom * 0.5).clamp(0.1, 0.3) as f32;
        let grid_color = Color32::from_gray((30.0 * grid_alpha) as u8);
        
        // Vertical lines
        let start_x = ((rect.min.x - self.pan.x) / grid_size).floor() * grid_size + self.pan.x;
        let mut x = start_x;
        while x < rect.max.x {
            painter.line_segment(
                [pos2(x, rect.min.y), pos2(x, rect.max.y)],
                Stroke::new(1.0, grid_color)
            );
            x += grid_size;
        }
        
        // Horizontal lines
        let start_y = ((rect.min.y - self.pan.y) / grid_size).floor() * grid_size + self.pan.y;
        let mut y = start_y;
        while y < rect.max.y {
