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
            painter.line_segment(
                [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                Stroke::new(1.0, grid_color)
            );
            y += grid_size;
        }
    }
    
    fn draw_connections(&self, ui: &mut Ui, node_graph: &NodeGraph) {
        let painter = ui.painter();
        
        for connection in &node_graph.connections {
            let from_pos = self.get_port_position(ui, node_graph, connection.from_node, connection.from_port, true);
            let to_pos = self.get_port_position(ui, node_graph, connection.to_node, connection.to_port, false);
            
            if let (Some(from), Some(to)) = (from_pos, to_pos) {
                // Draw curved connection line
                let control_offset = vec2((to.x - from.x) * 0.5, 0.0);
                let control1 = from + control_offset;
                let control2 = to - control_offset;
                
                painter.add(Shape::CubicBezier(CubicBezierShape {
                    points: [from, control1, control2, to],
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                }));
            }
        }
    }
    
    fn draw_active_connection(&self, ui: &mut Ui, node_graph: &NodeGraph, start_node: NodeId, start_port: PortId, is_output: bool) {
        let painter = ui.painter();
        
        if let Some(start_pos) = self.get_port_position(ui, node_graph, start_node, start_port, is_output) {
            let mouse_pos = ui.input().pointer.latest_pos().unwrap_or(start_pos);
            
            // Draw straight line from port to mouse
            painter.line_segment(
                [start_pos, mouse_pos],
                Stroke::new(2.0, Color32::from_rgb(255, 200, 100))
            );
        }
    }
    
    fn draw_node(&mut self, ui: &mut Ui, node_id: NodeId, node_graph: &mut NodeGraph) {
        let node = &node_graph.nodes[&node_id];
        let pos = self.node_positions.get(&node_id).copied().unwrap_or((100.0, 100.0));
        let pos = vec2(pos.0, pos.1) + self.pan;
        
        // Node size and appearance
        let node_width = 120.0;
        let node_height = 60.0 + (node.inputs.len() + node.outputs.len()) as f32 * 20.0;
        let node_rect = Rect::from_min_size(pos2(pos.x, pos.y), vec2(node_width, node_height));
        
        let painter = ui.painter();
        
        // Node background
        let bg_color = if self.selected_node == Some(node_id) {
            Color32::from_rgb(60, 60, 80)
        } else {
            Color32::from_rgb(40, 40, 60)
        };
        painter.rect(node_rect, 4.0, bg_color, Stroke::new(1.0, Color32::from_rgb(100, 100, 120)));
        
        // Node title
        let title_pos = pos + vec2(10.0, 5.0);
        painter.text(title_pos, Align2::LEFT_TOP, &node.name, FontId::proportional(14.0), Color32::WHITE);
        
        // Input ports
        for (i, input) in node.inputs.iter().enumerate() {
            let port_pos = pos + vec2(0.0, 30.0 + i as f32 * 20.0);
            self.draw_port(ui, port_pos, false, input.name.as_str(), node_id, input.id, node_graph);
        }
        
        // Output ports
        for (i, output) in node.outputs.iter().enumerate() {
            let port_pos = pos + vec2(node_width, 30.0 + i as f32 * 20.0);
            self.draw_port(ui, port_pos, true, output.name.as_str(), node_id, output.id, node_graph);
        }
        
        // Node interaction
        let node_response = ui.allocate_rect(node_rect, Sense::click_and_drag());
        
        if node_response.clicked() {
            self.selected_node = Some(node_id);
        }
        
        if node_response.dragged() {
            let new_pos = (pos + node_response.drag_delta() - self.pan).to_array();
            self.node_positions.insert(node_id, (new_pos[0], new_pos[1]));
        }
    }
    
    fn draw_port(&mut self, ui: &mut Ui, pos: Pos2, is_output: bool, name: &str, node_id: NodeId, port_id: PortId, node_graph: &mut NodeGraph) {
        let painter = ui.painter();
        let port_radius = 6.0;
        
        // Port circle
        let port_color = if is_output {
            Color32::from_rgb(100, 200, 255)
        } else {
            Color32::from_rgb(255, 100, 200)
        };
        
        painter.circle(pos, port_radius, port_color, Stroke::new(1.0, Color32::WHITE));
        
        // Port label
        let label_pos = if is_output {
            pos - vec2(10.0 + name.len() as f32 * 7.0, 0.0)
        } else {
            pos + vec2(10.0, 0.0)
        };
        painter.text(label_pos, Align2::CENTER_CENTER, name, FontId::proportional(12.0), Color32::WHITE);
        
        // Port interaction
        let port_rect = Rect::from_center_size(pos, vec2(port_radius * 2.0, port_radius * 2.0));
        let port_response = ui.allocate_rect(port_rect, Sense::click());
        
        if port_response.clicked() {
            match self.connection_start {
                None => {
                    // Start new connection
                    self.connection_start = Some((node_id, port_id, is_output));
                }
                Some((start_node, start_port, start_is_output)) => {
                    if start_is_output != is_output && start_node != node_id {
                        // Complete connection
                        if start_is_output {
                            node_graph.connect(start_node, start_port, node_id, port_id);
                        } else {
                            node_graph.connect(node_id, port_id, start_node, start_port);
                        }
                    }
                    self.connection_start = None;
                }
            }
        }
    }
    
    fn get_port_position(&self, ui: &Ui, node_graph: &NodeGraph, node_id: NodeId, port_id: PortId, is_output: bool) -> Option<Pos2> {
        let node_pos = self.node_positions.get(&node_id)?;
        let node = node_graph.nodes.get(&node_id)?;
        
        let node_width = 120.0;
        let base_y = node_pos.1 + 30.0;
        
        if is_output {
            if let Some((i, _)) = node.outputs.iter().enumerate().find(|(_, p)| p.id == port_id) {
                Some(pos2(node_pos.0 + node_width, base_y + i as f32 * 20.0) + self.pan.to_array().into())
            } else {
                None
            }
        } else {
            if let Some((i, _)) = node.inputs.iter().enumerate().find(|(_, p)| p.id == port_id) {
                Some(pos2(node_pos.0, base_y + i as f32 * 20.0) + self.pan.to_array().into())
            } else {
                None
            }
        }
    }
}