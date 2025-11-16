use bevy_egui::egui::*;
use bevy_egui::egui::epaint::CubicBezierShape;
use crate::node_graph::*;
use std::collections::HashMap;

pub struct VisualNodeEditor {
    pan: Vec2,
    zoom: f32,
    selected_node: Option<NodeId>,
    dragging_node: Option<NodeId>,
    drag_offset: Vec2,
    connection_start: Option<(NodeId, PortId, bool)>, // (node_id, port_id, is_output)
    node_positions: HashMap<NodeId, (f32, f32)>,
}

impl Default for VisualNodeEditor {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            selected_node: None,
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            connection_start: None,
            node_positions: HashMap::new(),
        }
    }
}

impl VisualNodeEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Handle pan and zoom
        if response.dragged_by(PointerButton::Middle) || (response.dragged_by(PointerButton::Primary) && ui.input(|i| i.modifiers.ctrl)) {
            self.pan += response.drag_delta();
        }
        
        if response.hovered() {
            ui.input(|i| {
                let zoom_delta = i.zoom_delta();
                if zoom_delta != 1.0 {
                    self.zoom *= zoom_delta;
                    self.zoom = self.zoom.clamp(0.1, 3.0);
                }
            });
        }

        // Draw background grid
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
            if let (Some(from_node), Some(to_node)) = (node_graph.nodes.get(&connection.from_node), node_graph.nodes.get(&connection.to_node)) {
                let from_pos = pos2(from_node.pos.0, from_node.pos.1) + self.pan;
                let to_pos = pos2(to_node.pos.0, to_node.pos.1) + self.pan;
                
                // Find port positions
                let mut from_port_pos = None;
                let mut to_port_pos = None;
                
                // Find output port position
                for (i, output) in from_node.outputs.iter().enumerate() {
                    if output.id == connection.from_port {
                        from_port_pos = Some(pos2(from_pos.x + 180.0 + 8.0, from_pos.y + 40.0 + (i as f32 * 25.0)));
                        break;
                    }
                }
                
                // Find input port position
                for (i, input) in to_node.inputs.iter().enumerate() {
                    if input.id == connection.to_port {
                        to_port_pos = Some(pos2(to_pos.x - 8.0, to_pos.y + 40.0 + (i as f32 * 25.0)));
                        break;
                    }
                }
                
                if let (Some(start), Some(end)) = (from_port_pos, to_port_pos) {
                    // Draw curved connection
                    let control_offset = ((end.x - start.x) * 0.5).max(50.0);
                    let control1 = pos2(start.x + control_offset, start.y);
                    let control2 = pos2(end.x - control_offset, end.y);
                    
                    painter.add(CubicBezierShape {
                        points: [start, control1, control2, end],
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: Stroke::new(2.0, Color32::from_rgb(100, 200, 255)).into(),
                    });
                }
            }
        }
    }

    fn draw_active_connection(&self, ui: &mut Ui, node_graph: &NodeGraph, start_node: NodeId, start_port: PortId, is_output: bool) {
        let painter = ui.painter();
        let mouse_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO));
        
        if let Some(node) = node_graph.nodes.get(&start_node) {
            let node_pos = pos2(node.pos.0, node.pos.1) + self.pan;
            
            let mut start_pos = None;
            
            if is_output {
                // Find output port position
                for (i, output) in node.outputs.iter().enumerate() {
                    if output.id == start_port {
                        start_pos = Some(pos2(node_pos.x + 180.0 + 8.0, node_pos.y + 40.0 + (i as f32 * 25.0)));
                        break;
                    }
                }
            } else {
                // Find input port position
                for (i, input) in node.inputs.iter().enumerate() {
                    if input.id == start_port {
                        start_pos = Some(pos2(node_pos.x - 8.0, node_pos.y + 40.0 + (i as f32 * 25.0)));
                        break;
                    }
                }
            }
            
            if let Some(start) = start_pos {
                // Draw curved connection to mouse
                let control_offset = ((mouse_pos.x - start.x) * 0.5).max(50.0);
                let control1 = pos2(start.x + control_offset * if is_output { 1.0 } else { -1.0 }, start.y);
                let control2 = pos2(mouse_pos.x - control_offset * if is_output { 1.0 } else { -1.0 }, mouse_pos.y);
                
                painter.add(CubicBezierShape {
                    points: [start, control1, control2, mouse_pos],
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(2.0, Color32::from_rgb(255, 200, 100)).into(),
                });
            }
        }
    }

    fn draw_node(&mut self, ui: &mut Ui, node_id: NodeId, node_graph: &mut NodeGraph) {
        if let Some(node) = node_graph.nodes.get_mut(&node_id) {
            // Update node position if being dragged
            if let Some(dragging_id) = self.dragging_node {
                if dragging_id == node_id {
                    let mouse_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO));
                    node.pos = (mouse_pos.x - self.drag_offset.x - self.pan.x, mouse_pos.y - self.drag_offset.y - self.pan.y);
                }
            }
            
            let node_pos = pos2(node.pos.0, node.pos.1) + self.pan;
            let node_size = vec2(180.0, 120.0);
            let node_rect = Rect::from_min_size(node_pos, node_size);
            
            let painter = ui.painter();
            let response = ui.allocate_rect(node_rect, Sense::click_and_drag());
            
            // Handle node selection and dragging
            if response.clicked() {
                self.selected_node = Some(node_id);
            }
            
            if response.drag_started() {
                self.dragging_node = Some(node_id);
                let mouse_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO));
                self.drag_offset = mouse_pos - node_pos;
            }
            
            if response.drag_delta() != Vec2::ZERO {
                self.dragging_node = None;
            }
            
            // Node background with selection highlight
            let mut bg_color = self.get_node_color(&node.kind);
            if self.selected_node == Some(node_id) {
                // Brighten selected node
                bg_color = Color32::from_rgb(
                    (bg_color.r() + 30).min(255),
                    (bg_color.g() + 30).min(255),
                    (bg_color.b() + 30).min(255)
                );
            }
            
            painter.rect_filled(node_rect, 4.0, bg_color);
            
            // Node border
            let border_color = if self.selected_node == Some(node_id) {
                Color32::from_rgb(255, 255, 100)
            } else {
                Color32::from_gray(60)
            };
            let stroke = Stroke::new(2.0, border_color);
            painter.rect_stroke(node_rect, 4.0, stroke);
            
            // Node title
            let title_pos = node_pos + vec2(10.0, 20.0);
            painter.text(
                title_pos,
                Align2::LEFT_CENTER,
                &node.title,
                FontId::proportional(14.0),
                Color32::WHITE
            );
            
            // Draw input ports with interaction
            for (i, input) in node.inputs.iter().enumerate() {
                let port_pos = pos2(node_pos.x - 8.0, node_pos.y + 40.0 + (i as f32 * 25.0));
                let port_response = self.draw_port(ui, painter, port_pos, input, false, node_id);
                
                // Handle port connection
                if port_response.clicked() {
                    if let Some((start_node, start_port, is_output)) = self.connection_start {
                        if !is_output {
                            // Connect output to this input
                            node_graph.connect(start_node, start_port, node_id, input.id);
                            self.connection_start = None;
                        }
                    } else {
                        // Start connection from this input (unusual but allowed)
                        self.connection_start = Some((node_id, input.id, false));
                    }
                }
            }
            
            // Draw output ports with interaction
            for (i, output) in node.outputs.iter().enumerate() {
                let port_pos = pos2(node_pos.x + node_size.x + 8.0, node_pos.y + 40.0 + (i as f32 * 25.0));
                let port_response = self.draw_port(ui, painter, port_pos, output, true, node_id);
                
                // Handle port connection
                if port_response.clicked() {
                    if let Some((start_node, start_port, is_output)) = self.connection_start {
                        if is_output {
                            // Connect this output to input
                            node_graph.connect(node_id, output.id, start_node, start_port);
                            self.connection_start = None;
                        }
                    } else {
                        // Start connection from this output
                        self.connection_start = Some((node_id, output.id, true));
                    }
                }
            }
        }
    }

    fn draw_port(&mut self, ui: &mut Ui, painter: &Painter, pos: Pos2, port: &Port, is_output: bool, _node_id: NodeId) -> Response {
        let port_color = self.get_port_color(&port.kind);
        let port_size = 8.0;
        
        let port_rect = Rect::from_center_size(pos, vec2(port_size * 2.0, port_size * 2.0));
        let response = ui.allocate_rect(port_rect, Sense::click());
        
        // Highlight port on hover
        let final_color = if response.hovered() {
            Color32::from_rgb(
                (port_color.r() + 50).min(255),
                (port_color.g() + 50).min(255),
                (port_color.b() + 50).min(255)
            )
        } else {
            port_color
        };
        
        painter.circle(pos, port_size, final_color, Stroke::new(1.0, Color32::WHITE));
        
        // Port label
        let label_pos = if is_output {
            pos - vec2(15.0, 0.0)
        } else {
            pos + vec2(15.0, 0.0)
        };
        painter.text(
            label_pos,
            if is_output { Align2::RIGHT_CENTER } else { Align2::LEFT_CENTER },
            &port.name,
            FontId::proportional(12.0),
            Color32::WHITE
        );
        
        response
    }

    fn get_node_color(&self, kind: &NodeKind) -> Color32 {
        match kind {
            NodeKind::ConstantFloat(_) | NodeKind::ConstantVec3(_) => Color32::from_rgb(60, 60, 120),
            NodeKind::Time => Color32::from_rgb(120, 60, 60),
            NodeKind::UV => Color32::from_rgb(60, 120, 60),
            NodeKind::Param(_) => Color32::from_rgb(120, 120, 60),
            NodeKind::Add | NodeKind::Multiply => Color32::from_rgb(120, 60, 120),
            NodeKind::Sine => Color32::from_rgb(60, 120, 120),
            NodeKind::TextureSample => Color32::from_rgb(120, 80, 40),
            NodeKind::OutputColor => Color32::from_rgb(180, 60, 60),
        }
    }

    fn get_port_color(&self, kind: &PortKind) -> Color32 {
        match kind {
            PortKind::Float => Color32::from_rgb(100, 150, 200),
            PortKind::Vec2 => Color32::from_rgb(150, 200, 100),
            PortKind::Vec3 => Color32::from_rgb(200, 100, 150),
            PortKind::Vec4 => Color32::from_rgb(200, 150, 100),
            PortKind::Color => Color32::from_rgb(255, 200, 100),
            PortKind::Texture => Color32::from_rgb(150, 100, 200),
        }
    }
}