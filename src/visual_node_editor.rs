use bevy_egui::egui::*;
use crate::node_graph::{NodeGraph, NodeId, NodeKind, PortId, Connection};
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
        let wgsl = node_graph.generate_wgsl(width, height);
        self.last_generated_wgsl = Some(wgsl.clone());
        self.compilation_errors.clear();
        Ok(wgsl)
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
                let id = node_graph.add_node(NodeKind::Sine, "Sin", (300.0, 100.0));
                self.node_positions.insert(id, (300.0, 100.0));
            }
            if ui.button("Vec2").clicked() {
                let id = node_graph.add_node(NodeKind::ConstantVec2([0.0, 0.0]), "Vec2", (400.0, 100.0));
                self.node_positions.insert(id, (400.0, 100.0));
            }
            if ui.button("Output").clicked() {
                let id = node_graph.add_node(NodeKind::OutputColor, "Output", (500.0, 100.0));
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
            let zoom_delta = ui.input(|i| i.raw_scroll_delta.y) * 0.01;
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
            painter.line_segment(
                [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                Stroke::new(1.0, grid_color)
            );
            y += grid_size;
        }
    }
    
    fn draw_connections(&mut self, ui: &mut Ui, node_graph: &NodeGraph) {
        let painter = ui.painter();
        
        // Draw all connections in the graph
        for connection in &node_graph.connections {
            let output_node_id = connection.from_node;
            let input_node_id = connection.to_node;
            
            // Get positions for both nodes
            if let (Some(input_pos), Some(output_pos)) = (self.node_positions.get(&input_node_id), self.node_positions.get(&output_node_id)) {
                // Calculate port positions (simplified - just use node centers for now)
                let input_port_pos = pos2(input_pos.0 + 100.0, input_pos.1 + 50.0);
                let output_port_pos = pos2(output_pos.0, output_pos.1 + 50.0);
                
                // Draw connection line
                painter.line_segment(
                    [output_port_pos, input_port_pos],
                    Stroke::new(2.0, Color32::from_rgb(100, 200, 255))
                );
            }
        }
    }
    
    fn draw_node(&mut self, ui: &mut Ui, node_id: NodeId, node_graph: &NodeGraph) {
        if let Some(node) = node_graph.nodes.get(&node_id) {
            if let Some(pos) = self.node_positions.get(&node_id) {
                let node_pos = pos2(pos.0, pos.1);
                let node_size = vec2(120.0, 80.0);
                
                // Draw node background
                let painter = ui.painter();
                let rect = Rect::from_min_size(node_pos, node_size);
                painter.rect_filled(rect, 4.0, Color32::from_rgb(60, 60, 60));
                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(120, 120, 120)), StrokeKind::Inside);
                
                // Draw node title
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{:?}", node.kind),
                    FontId::proportional(12.0),
                    Color32::WHITE
                );
                
                // Draw input/output ports (simplified)
                for (i, _input) in node.inputs.iter().enumerate() {
                    let port_pos = pos2(rect.min.x, rect.min.y + 20.0 + (i as f32 * 15.0));
                    painter.circle(port_pos, 4.0, Color32::from_rgb(255, 100, 100), Stroke::new(1.0, Color32::WHITE));
                }
                
                for (i, _output) in node.outputs.iter().enumerate() {
                    let port_pos = pos2(rect.max.x, rect.min.y + 20.0 + (i as f32 * 15.0));
                    painter.circle(port_pos, 4.0, Color32::from_rgb(100, 255, 100), Stroke::new(1.0, Color32::WHITE));
                }
            }
        }
    }
    
    fn draw_active_connection(&mut self, ui: &mut Ui, _node_graph: &NodeGraph, start_node: NodeId, start_port: PortId, is_output: bool) {
        if let Some(start_pos) = self.node_positions.get(&start_node) {
            let painter = ui.painter();
            let start_node_pos = pos2(start_pos.0, start_pos.1);
            
            // Calculate start port position (simplified)
            let start_port_pos = if is_output {
                pos2(start_node_pos.x + 120.0, start_node_pos.y + 50.0)
            } else {
                pos2(start_node_pos.x, start_node_pos.y + 50.0)
            };
            
            // Draw line to mouse cursor
            if let Some(mouse_pos) = ui.input(|i| i.pointer.latest_pos()) {
                painter.line_segment(
                    [start_port_pos, mouse_pos],
                    Stroke::new(2.0, Color32::from_rgb(255, 200, 100))
                );
            }
        }
    }
}
