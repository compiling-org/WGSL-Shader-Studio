use egui::*;
use crate::node_graph::{NodeGraph, NodeId, NodeKind, PortId};
use std::collections::HashMap;

/// A visual node editor for creating shader graphs with drag-and-drop interface
pub struct VisualNodeEditor {
    node_positions: HashMap<NodeId, (f32, f32)>,
    connection_start: Option<(NodeId, PortId, bool)>, // node_id, port_id, is_output
    selected_node: Option<NodeId>,
    pan: Vec2,
    zoom: f32,
    auto_compile: bool,
    last_generated_wgsl: Option<String>,
    compilation_errors: Vec<String>,
    show_grid: bool,
    grid_size: f32,
    node_drag_state: Option<NodeDragState>,
}

#[derive(Debug, Clone)]
struct NodeDragState {
    node_id: NodeId,
    drag_offset: Vec2,
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
            show_grid: true,
            grid_size: 20.0,
            node_drag_state: None,
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
        self.draw_control_panel(ui, node_graph);
        
        ui.separator();

        // Node editor canvas
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Handle canvas interactions (pan, zoom, click)
        self.handle_canvas_interactions(&response, available_rect);
        
        // Draw grid if enabled
        if self.show_grid {
            self.draw_grid(ui, available_rect);
        }

        // Draw connections first (behind nodes)
        self.draw_connections(ui, node_graph);

        // Draw nodes with proper layering
        self.draw_nodes(ui, node_graph, available_rect);

        // Draw connection being created
        if let Some((start_node, start_port, is_output)) = self.connection_start {
            self.draw_active_connection(ui, node_graph, start_node, start_port, is_output);
        }
        
        // Handle node dragging
        self.handle_node_dragging(ui, node_graph, &response);
        
        // Handle context menu for adding nodes
        self.handle_context_menu(ui, node_graph, &response);
    }

    fn draw_control_panel(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        ui.horizontal(|ui| {
            if ui.button("▶ Compile").clicked() {
                match self.generate_and_compile(node_graph, 1920, 1080) {
                    Ok(wgsl) => {
                        println!("✅ Compilation successful! Generated {} characters of WGSL", wgsl.len());
                    }
                    Err(errors) => {
                        println!("❌ Compilation failed with {} errors:", errors.len());
                        for error in &errors {
                            println!("  - {}", error);
                        }
                    }
                }
            }
            
            ui.checkbox(&mut self.auto_compile, "Auto-compile");
            ui.checkbox(&mut self.show_grid, "Show grid");
            
            ui.separator();
            
            if ui.button("🗑 Clear").clicked() {
                node_graph.clear();
                self.node_positions.clear();
                self.selected_node = None;
                self.connection_start = None;
                self.compilation_errors.clear();
                self.last_generated_wgsl = None;
            }
            
            if ui.button("💾 Save").clicked() {
                match node_graph.save_to_file("node_graph.json") {
                    Ok(_) => println!("✅ Node graph saved successfully"),
                    Err(e) => println!("❌ Failed to save node graph: {}", e),
                }
            }
            
            if ui.button("📁 Load").clicked() {
                match NodeGraph::load_from_file("node_graph.json") {
                    Ok(loaded_graph) => {
                        *node_graph = loaded_graph;
                        println!("✅ Node graph loaded successfully");
                    }
                    Err(e) => println!("❌ Failed to load node graph: {}", e),
                }
            }
            
            ui.separator();
            
            ui.label(format!("Nodes: {}", node_graph.nodes.len()));
            ui.label(format!("Connections: {}", node_graph.connections.len()));
            
            if !self.compilation_errors.is_empty() {
                ui.colored_label(egui::Color32::RED, format!("❌ {} errors", self.compilation_errors.len()));
            } else if self.last_generated_wgsl.is_some() {
                ui.colored_label(egui::Color32::GREEN, "✅ Compiled");
            }
        });
    }

    fn handle_canvas_interactions(&mut self, response: &egui::Response, rect: Rect) {
        // Handle pan with middle mouse button
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan += response.drag_delta();
        }

        // Handle zoom with mouse wheel
        if let Some(hover_pos) = response.hover_pos() {
            let zoom_delta = response.ctx.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.zoom *= zoom_delta;
                self.zoom = self.zoom.clamp(0.1, 5.0);
            }
        }

        // Handle click to deselect
        if response.clicked() {
            self.selected_node = None;
            self.connection_start = None;
        }
    }

    fn draw_grid(&self, ui: &mut Ui, rect: Rect) {
        let grid_size = self.grid_size * self.zoom;
        let grid_alpha = (0.5 * self.zoom).clamp(0.1, 0.5);
        let grid_color = Color32::from_gray((30.0 * grid_alpha) as u8);
        
        let painter = ui.painter();
        
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
            if let (Some(from_pos, from_node), Some(to_pos, to_node)) = (
                self.get_port_position(ui, node_graph, &connection.from_node, &connection.from_port, true),
                self.get_port_position(ui, node_graph, &connection.to_node, &connection.to_port, false)
            ) {
                let stroke = Stroke::new(2.0, Color32::from_rgb(100, 200, 255));
                painter.line_segment([from_pos, to_pos], stroke);
                
                // Draw direction arrow
                let direction = (to_pos - from_pos).normalized();
                let arrow_pos = to_pos - direction * 8.0;
                let arrow_size = 6.0;
                let perpendicular = vec2(-direction.y, direction.x);
                
                painter.line_segment(
                    [arrow_pos, arrow_pos + direction * arrow_size - perpendicular * arrow_size/2.0],
                    stroke
                );
                painter.line_segment(
                    [arrow_pos, arrow_pos + direction * arrow_size + perpendicular * arrow_size/2.0],
                    stroke
                );
            }
        }
    }

    fn draw_nodes(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph, rect: Rect) {
        let node_ids: Vec<NodeId> = node_graph.nodes.keys().cloned().collect();
        
        for node_id in node_ids {
            let node = node_graph.nodes.get_mut(&node_id).unwrap();
            let position = self.node_positions.entry(node_id.clone()).or_insert((100.0, 100.0));
            
            // Create node UI
            let node_rect = self.create_node_ui(ui, node, *position, rect);
            
            // Update position if node was dragged
            if let Some(drag_state) = &self.node_drag_state {
                if drag_state.node_id == node_id {
                    *position = (node_rect.center().x, node_rect.center().y);
                }
            }
        }
    }

    fn create_node_ui(&mut self, ui: &mut Ui, node: &mut crate::node_graph::Node, position: (f32, f32), rect: Rect) -> Rect {
        let node_size = vec2(180.0, 120.0);
        let node_pos = pos2(position.0, position.1);
        let node_rect = Rect::from_min_size(node_pos - node_size/2.0, node_size);
        
        // Ensure node stays within bounds
        let clamped_rect = node_rect.intersect(rect);
        
        let response = ui.allocate_rect(clamped_rect, Sense::click_and_drag());
        let painter = ui.painter();
        
        // Node background
        let bg_color = if response.hovered() {
            Color32::from_rgb(60, 60, 80)
        } else {
            Color32::from_rgb(40, 40, 60)
        };
        painter.rect(clamped_rect, 4.0, bg_color, Stroke::new(1.0, Color32::from_rgb(100, 100, 150)));
        
        // Node header
        let header_rect = Rect::from_min_max(clamped_rect.min, pos2(clamped_rect.max.x, clamped_rect.min.y + 25.0));
        painter.rect(header_rect, 4.0, Color32::from_rgb(70, 70, 100), Stroke::NONE);
        
        // Node title
        let title_pos = clamped_rect.min + vec2(8.0, 4.0);
        painter.text(title_pos, Align2::LEFT_TOP, &node.name, FontId::proportional(12.0), Color32::WHITE);
        
        // Input/output ports
        self.draw_ports(ui, node, &clamped_rect);
        
        // Handle node selection and dragging
        if response.clicked() {
            self.selected_node = Some(node.id.clone());
        }
        
        if response.dragged() {
            if self.node_drag_state.is_none() {
                self.node_drag_state = Some(NodeDragState {
                    node_id: node.id.clone(),
                    drag_offset: response.drag_delta(),
                });
            } else if let Some(drag_state) = &mut self.node_drag_state {
                if drag_state.node_id == node.id {
                    drag_state.drag_offset += response.drag_delta();
                }
            }
        } else if self.node_drag_state.as_ref().map(|s| s.node_id == node.id).unwrap_or(false) {
            self.node_drag_state = None;
        }
        
        clamped_rect
    }

    fn draw_ports(&self, ui: &mut Ui, node: &crate::node_graph::Node, node_rect: &Rect) {
        let painter = ui.painter();
        let port_radius = 4.0;
        
        // Input ports (left side)
        for (i, input) in node.inputs.iter().enumerate() {
            let y = node_rect.min.y + 35.0 + (i as f32 * 20.0);
            let port_pos = pos2(node_rect.min.x - port_radius, y);
            
            let port_color = match input.port_type {
                crate::node_graph::PortType::Float => Color32::from_rgb(255, 150, 0),
                crate::node_graph::PortType::Vec2 => Color32::from_rgb(0, 255, 150),
                crate::node_graph::PortType::Vec3 => Color32::from_rgb(0, 150, 255),
                crate::node_graph::PortType::Vec4 => Color32::from_rgb(255, 0, 255),
                _ => Color32::from_rgb(200, 200, 200),
            };
            
            painter.circle_filled(port_pos, port_radius, port_color);
            painter.text(port_pos + vec2(8.0, 0.0), Align2::LEFT_CENTER, &input.name, 
                        FontId::proportional(10.0), Color32::WHITE);
        }
        
        // Output ports (right side)
        for (i, output) in node.outputs.iter().enumerate() {
            let y = node_rect.min.y + 35.0 + (i as f32 * 20.0);
            let port_pos = pos2(node_rect.max.x + port_radius, y);
            
            let port_color = match output.port_type {
                crate::node_graph::PortType::Float => Color32::from_rgb(255, 150, 0),
                crate::node_graph::PortType::Vec2 => Color32::from_rgb(0, 255, 150),
                crate::node_graph::PortType::Vec3 => Color32::from_rgb(0, 150, 255),
                crate::node_graph::PortType::Vec4 => Color32::from_rgb(255, 0, 255),
                _ => Color32::from_rgb(200, 200, 200),
            };
            
            painter.circle_filled(port_pos, port_radius, port_color);
            painter.text(port_pos - vec2(8.0, 0.0), Align2::RIGHT_CENTER, &output.name, 
                        FontId::proportional(10.0), Color32::WHITE);
        }
    }

    fn get_port_position(&self, ui: &Ui, node_graph: &NodeGraph, node_id: &NodeId, port_id: &PortId, is_output: bool) -> Option<Pos2> {
        if let Some(position) = self.node_positions.get(node_id) {
            if let Some(node) = node_graph.nodes.get(node_id) {
                let node_size = vec2(180.0, 120.0);
                let node_pos = pos2(position.0, position.1) - node_size/2.0;
                
                let ports = if is_output { &node.outputs } else { &node.inputs };
                if let Some(port_index) = ports.iter().position(|p| p.id == *port_id) {
                    let y = node_pos.y + 35.0 + (port_index as f32 * 20.0);
                    let x = if is_output { node_pos.x + node_size.x } else { node_pos.x };
                    return Some(pos2(x, y));
                }
            }
        }
        None
    }

    fn draw_active_connection(&self, ui: &mut Ui, node_graph: &NodeGraph, start_node: NodeId, start_port: PortId, is_output: bool) {
        if let Some(start_pos) = self.get_port_position(ui, node_graph, &start_node, &start_port, is_output) {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.latest_pos()) {
                let painter = ui.painter();
                let stroke = Stroke::new(2.0, Color32::from_rgb(255, 200, 100));
                painter.line_segment([start_pos, mouse_pos], stroke);
            }
        }
    }

    fn handle_node_dragging(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph, response: &egui::Response) {
        // Handle port connections
        if let Some(click_pos) = response.interact_pointer_pos() {
            // Check for port clicks
            for (node_id, position) in &self.node_positions {
                if let Some(node) = node_graph.nodes.get(node_id) {
                    let node_size = vec2(180.0, 120.0);
                    let node_pos = pos2(position.0, position.1) - node_size/2.0;
                    let node_rect = Rect::from_min_size(node_pos, node_size);
                    
                    if node_rect.contains(click_pos) {
                        // Check input ports
                        for (i, input) in node.inputs.iter().enumerate() {
                            let y = node_pos.y + 35.0 + (i as f32 * 20.0);
                            let port_pos = pos2(node_pos.x - 4.0, y);
                            let port_rect = Rect::from_center_size(port_pos, vec2(8.0, 8.0));
                            
                            if port_rect.contains(click_pos) {
                                if let Some((start_node, start_port, true)) = self.connection_start {
                                    // Complete connection
                                    node_graph.add_connection(start_node, start_port, node_id.clone(), input.id.clone());
                                    self.connection_start = None;
                                } else {
                                    // Start connection from input
                                    self.connection_start = Some((node_id.clone(), input.id.clone(), false));
                                }
                                return;
                            }
                        }
                        
                        // Check output ports
                        for (i, output) in node.outputs.iter().enumerate() {
                            let y = node_pos.y + 35.0 + (i as f32 * 20.0);
                            let port_pos = pos2(node_pos.x + node_size.x + 4.0, y);
                            let port_rect = Rect::from_center_size(port_pos, vec2(8.0, 8.0));
                            
                            if port_rect.contains(click_pos) {
                                if let Some((start_node, start_port, false)) = self.connection_start {
                                    // Complete connection
                                    node_graph.add_connection(node_id.clone(), output.id.clone(), start_node, start_port);
                                    self.connection_start = None;
                                } else {
                                    // Start connection from output
                                    self.connection_start = Some((node_id.clone(), output.id.clone(), true));
                                }
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_context_menu(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph, response: &egui::Response) {
        if response.secondary_clicked() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                egui::Window::new("Add Node")
                    .fixed_pos(click_pos)
                    .show(ui.ctx(), |ui| {
                        ui.set_max_width(200.0);
                        
                        ui.heading("Math Nodes");
                        if ui.button("Add").clicked() {
                            let node_id = node_graph.add_node("Add", NodeKind::Math);
                            self.node_positions.insert(node_id, (click_pos.x, click_pos.y));
                        }
                        if ui.button("Multiply").clicked() {
                            let node_id = node_graph.add_node("Multiply", NodeKind::Math);
                            self.node_positions.insert(node_id, (click_pos.x, click_pos.y));
                        }
                        if ui.button("Sine").clicked() {
                            let node_id = node_graph.add_node("Sine", NodeKind::Math);
                            self.node_positions.insert(node_id, (click_pos.x, click_pos.y));
                        }
                        
                        ui.separator();
                        ui.heading("Input Nodes");
                        if ui.button("Time").clicked() {
                            let node_id = node_graph.add_node("Time", NodeKind::Input);
                            self.node_positions.insert(node_id, (click_pos.x, click_pos.y));
                        }
                        if ui.button("Resolution").clicked() {
                            let node_id = node_graph.add_node("Resolution", NodeKind::Input);
                            self.node_positions.insert(node_id, (click_pos.x, click_pos.y));
                        }
                        if ui.button("Mouse").clicked() {
                            let node_id = node_graph.add_node("Mouse", NodeKind::Input);
                            self.node_positions.insert(node_id, (click_pos.x, click_pos.y));
                        }
                        
                        ui.separator();
                        ui.heading("Output Nodes");
                        if ui.button("Color Output").clicked() {
                            let node_id = node_graph.add_node("Color Output", NodeKind::Output);
                            self.node_positions.insert(node_id, (click_pos.x, click_pos.y));
                        }
                        if ui.button("UV Coordinates").clicked() {
                            let node_id = node_graph.add_node("UV Coordinates", NodeKind::Utility);
                            self.node_positions.insert(node_id, (click_pos.x, click_pos.y));
                        }
                    });
            }
        }
    }
}

impl Default for VisualNodeEditor {
    fn default() -> Self {
        Self::new()
    }
}