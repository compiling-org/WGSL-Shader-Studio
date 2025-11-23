use egui::*;
use std::collections::{HashMap, HashSet};
use crate::node_graph::*;

#[derive(Clone)]
struct NodeGraphState {
    nodes: HashMap<NodeId, Node>,
    connections: Vec<Connection>,
}

pub struct VisualNodeEditor {
    pan: Vec2,
    zoom: f32,
    selected_node: Option<NodeId>,
    dragging_node: Option<NodeId>,
    drag_offset: Vec2,
    connection_start: Option<(NodeId, PortId, bool)>, // (node_id, port_id, is_output)
    node_positions: HashMap<NodeId, Pos2>,
    // Enhanced features
    hovered_node: Option<NodeId>,
    hovered_port: Option<(NodeId, PortId, bool)>,
    drag_box_start: Option<Pos2>,
    selected_nodes: HashSet<NodeId>,
    clipboard: Option<Vec<(NodeId, Node, Pos2)>>,
    undo_stack: Vec<NodeGraphState>,
    redo_stack: Vec<NodeGraphState>,
    max_undo_steps: usize,
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
            // Enhanced features
            hovered_node: None,
            hovered_port: None,
            drag_box_start: None,
            selected_nodes: HashSet::new(),
            clipboard: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_steps: 50,
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
        
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ui, node_graph);
        
        // Handle pan and zoom
        if response.dragged_by(PointerButton::Middle) || (response.dragged_by(PointerButton::Primary) && ui.input(|i| i.modifiers.ctrl)) {
            self.pan += response.drag_delta();
        }
        
        // Handle selection box
        if response.dragged_by(PointerButton::Primary) && !ui.input(|i| i.modifiers.ctrl) && self.dragging_node.is_none() {
            if self.drag_box_start.is_none() {
                self.drag_box_start = Some(response.interact_pointer_pos().unwrap_or(Pos2::ZERO));
            }
        } else {
            if let Some(start_pos) = self.drag_box_start {
                let end_pos = response.interact_pointer_pos().unwrap_or(Pos2::ZERO);
                self.update_selection_from_box(start_pos, end_pos, node_graph);
                self.drag_box_start = None;
            }
        }
        
        // Handle zoom
        if response.hovered() {
            let zoom_delta = ui.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                self.zoom = (self.zoom * zoom_delta).clamp(0.1, 5.0);
            }
        }
        
        let painter = ui.painter();
        
        // Draw grid
        self.draw_grid(ui, available_rect);
        
        // Draw selection box if active
        if let Some(start_pos) = self.drag_box_start {
            if let Some(current_pos) = response.interact_pointer_pos() {
                self.draw_selection_box(ui, start_pos, current_pos);
            }
        }
        
        // Draw connections first (behind nodes)
        self.draw_connections(ui, node_graph);
        self.draw_active_connection(ui, node_graph, response.hover_pos());
        
        // Draw nodes
        let mut nodes_to_draw: Vec<(NodeId, Node)> = node_graph.nodes.iter()
            .map(|(&id, node)| (id, node.clone()))
            .collect();
        nodes_to_draw.sort_by_key(|(id, _)| *id);
        
        for (node_id, node) in nodes_to_draw {
            self.draw_node(ui, node_id, node_graph);
        }
        
        // Handle interactions
        if response.clicked() && !response.drag_delta().any() {
            if !ui.input(|i| i.modifiers.ctrl || i.modifiers.shift) {
                self.selected_nodes.clear();
            }
        }
        
        response
    }

    fn draw_grid(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        let grid_color = Color32::from_gray(40);
        let grid_size = 20.0 * self.zoom;
        
        let start_x = ((rect.min.x - self.pan.x) / grid_size).floor() * grid_size + self.pan.x;
        let start_y = ((rect.min.y - self.pan.y) / grid_size).floor() * grid_size + self.pan.y;
        
        let end_x = rect.max.x;
        let end_y = rect.max.y;
        
        let mut x = start_x;
        while x < end_x {
            painter.line_segment(
                [pos2(x, rect.min.y), pos2(x, rect.max.y)],
                Stroke::new(0.5, grid_color)
            );
            x += grid_size;
        }
        
        let mut y = start_y;
        while y < end_y {
            painter.line_segment(
                [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                Stroke::new(0.5, grid_color)
            );
            y += grid_size;
        }
    }

    fn draw_connections(&self, ui: &mut Ui, node_graph: &NodeGraph) {
        let painter = ui.painter();
        
        for connection in &node_graph.connections {
            if let (Some(from_node), Some(to_node)) = (
                node_graph.nodes.get(&connection.from_node),
                node_graph.nodes.get(&connection.to_node)
            ) {
                let from_pos = self.get_output_port_pos(connection.from_node, connection.from_port, from_node);
                let to_pos = self.get_input_port_pos(connection.to_node, connection.to_port, to_node);
                
                if let (Some(from), Some(to)) = (from_pos, to_pos) {
                    let stroke = Stroke::new(2.0, Color32::from_rgb(100, 150, 255));
                    painter.line_segment([from, to], stroke);
                    
                    // Draw arrowhead
                    let direction = (to - from).normalized();
                    let arrow_size = 8.0;
                    let arrow_pos = to - direction * 5.0;
                    let perpendicular = vec2(-direction.y, direction.x) * (arrow_size / 2.0);
                    
                    painter.add(Shape::convex_polygon(
                        vec![
                            arrow_pos,
                            arrow_pos - direction * arrow_size + perpendicular,
                            arrow_pos - direction * arrow_size - perpendicular,
                        ],
                        Color32::from_rgb(100, 150, 255),
                        Stroke::NONE
                    ));
                }
            }
        }
    }

    fn draw_active_connection(&self, ui: &mut Ui, node_graph: &NodeGraph, mouse_pos: Option<Pos2>) {
        if let Some((start_node, start_port, is_output)) = self.connection_start {
            if let Some(start_node_data) = node_graph.nodes.get(&start_node) {
                let start_pos = if is_output {
                    self.get_output_port_pos(start_node, start_port, start_node_data)
                } else {
                    self.get_input_port_pos(start_node, start_port, start_node_data)
                };
                
                if let (Some(start), Some(mouse)) = (start_pos, mouse_pos) {
                    let painter = ui.painter();
                    let stroke = Stroke::new(2.0, Color32::from_rgb(150, 200, 255));
                    painter.line_segment([start, mouse], stroke);
                }
            }
        }
    }

    fn draw_node(&mut self, ui: &mut Ui, node_id: NodeId, node_graph: &mut NodeGraph) {
        let node = node_graph.nodes.get(&node_id).unwrap().clone();
        let node_screen_pos = self.node_positions.get(&node_id).copied().unwrap_or_else(|| {
            let pos = pos2(100.0, 100.0);
            self.node_positions.insert(node_id, pos);
            pos
        }) + self.pan;
        
        let node_size = vec2(180.0, 120.0);
        let node_rect = Rect::from_min_size(node_screen_pos, node_size);
        
        let painter = ui.painter();
        
        // Node background
        let node_color = self.get_node_color(&node.kind);
        let border_color = if self.selected_nodes.contains(&node_id) {
            Color32::from_rgb(255, 200, 0)
        } else if self.hovered_node == Some(node_id) {
            Color32::from_rgb(200, 200, 200)
        } else {
            Color32::from_gray(100)
        };
        
        painter.rect_filled(node_rect, 4.0, node_color);
        painter.rect_stroke(node_rect, 4.0, Stroke::new(2.0, border_color));
        
        // Node title
        let title = format!("{:?}", node.kind);
        painter.text(
            node_screen_pos + vec2(10.0, 15.0),
            Align2::LEFT_CENTER,
            &title,
            FontId::proportional(12.0),
            Color32::WHITE
        );
        
        // Input ports
        let mut input_y = 40.0;
        for (i, port) in node.inputs.iter().enumerate() {
            let port_pos = node_screen_pos + vec2(10.0, input_y);
            self.draw_port_visual(painter, port_pos, port, false, node_id, node_graph);
            input_y += 25.0;
        }
        
        // Output ports
        let mut output_y = 40.0;
        for (i, port) in node.outputs.iter().enumerate() {
            let port_pos = node_screen_pos + vec2(node_size.x - 10.0, output_y);
            self.draw_port_visual(painter, port_pos, port, true, node_id, node_graph);
            output_y += 25.0;
        }
        
        // Node interaction
        let node_response = ui.allocate_rect(node_rect, Sense::click_and_drag());
        
        if node_response.hovered() {
            self.hovered_node = Some(node_id);
        }
        
        if node_response.dragged_by(PointerButton::Primary) && self.connection_start.is_none() {
            if self.dragging_node.is_none() {
                self.dragging_node = Some(node_id);
                self.drag_offset = node_response.interact_pointer_pos().unwrap_or(Pos2::ZERO) - node_screen_pos;
            }
        }
        
        if self.dragging_node == Some(node_id) {
            let new_pos = node_response.interact_pointer_pos().unwrap_or(Pos2::ZERO) - self.drag_offset - self.pan;
            self.node_positions.insert(node_id, new_pos);
        }
        
        if !node_response.dragged() {
            if self.dragging_node == Some(node_id) {
                self.dragging_node = None;
            }
        }
        
        if node_response.clicked() {
            if ui.input(|i| i.modifiers.ctrl) {
                if self.selected_nodes.contains(&node_id) {
                    self.selected_nodes.remove(&node_id);
                } else {
                    self.selected_nodes.insert(node_id);
                }
            } else if !ui.input(|i| i.modifiers.shift) {
                self.selected_nodes.clear();
                self.selected_nodes.insert(node_id);
            }
        }
    }

    fn draw_port_visual(&mut self, painter: &Painter, pos: Pos2, port: &Port, is_output: bool, node_id: NodeId, node_graph: &NodeGraph) {
        let port_color = self.get_port_color(&port.kind);
        let is_connected = self.is_port_connected(node_graph, node_id, port.id);
        let is_hovered = self.hovered_port == Some((node_id, port.id, is_output));
        
        let port_size = if is_connected { 8.0 } else { 6.0 };
        let port_stroke = if is_hovered {
            Stroke::new(2.0, Color32::WHITE)
        } else {
            Stroke::new(1.0, Color32::from_gray(150))
        };
        
        painter.circle_filled(pos, port_size, port_color);
        painter.circle_stroke(pos, port_size, port_stroke);
        
        // Port label
        let label_pos = if is_output {
            pos - vec2(15.0, 0.0)
        } else {
            pos + vec2(15.0, 0.0)
        };
        
        let label_color = if is_hovered {
            Color32::WHITE
        } else {
            Color32::from_gray(200)
        };
        
        painter.text(
            label_pos,
            if is_output { Align2::RIGHT_CENTER } else { Align2::LEFT_CENTER },
            &port.name,
            FontId::proportional(11.0),
            label_color
        );
        
        // Check for port hover
        let port_rect = Rect::from_center_size(pos, vec2(20.0, 20.0));
        if let Some(mouse_pos) = painter.ctx().input(|i| i.pointer.latest_pos()) {
            if port_rect.contains(mouse_pos) {
                self.hovered_port = Some((node_id, port.id, is_output));
            }
        }
    }

    fn is_port_connected(&self, node_graph: &NodeGraph, node_id: NodeId, port_id: PortId) -> bool {
        self.connection_start.map_or(false, |(start_node, start_port, _)| {
            start_node == node_id && start_port == port_id
        }) || self.connection_start.is_none() && node_graph.connections.iter().any(|conn| {
            (conn.from_node == node_id && conn.from_port == port_id) ||
            (conn.to_node == node_id && conn.to_port == port_id)
        })
    }

    fn get_node_color(&self, kind: &NodeKind) -> Color32 {
        match kind {
            // Constants
            NodeKind::ConstantFloat(_) | NodeKind::ConstantVec2(_) | NodeKind::ConstantVec3(_) | NodeKind::ConstantVec4(_) => Color32::from_rgb(60, 60, 120),
            
            // Input/Time
            NodeKind::Time => Color32::from_rgb(120, 60, 60),
            NodeKind::UV => Color32::from_rgb(60, 120, 60),
            NodeKind::Param(_) => Color32::from_rgb(120, 120, 60),
            NodeKind::Resolution => Color32::from_rgb(100, 100, 60),
            NodeKind::Mouse => Color32::from_rgb(120, 80, 60),
            
            // Math Operations
            NodeKind::Add | NodeKind::Subtract | NodeKind::Multiply | NodeKind::Divide => Color32::from_rgb(120, 60, 120),
            
            // Trigonometry
            NodeKind::Sine | NodeKind::Cosine | NodeKind::Tangent => Color32::from_rgb(60, 120, 120),
            
            // Vector Operations
            NodeKind::Length | NodeKind::Normalize | NodeKind::Distance | NodeKind::Dot | NodeKind::Cross | NodeKind::Reflect | NodeKind::Refract => Color32::from_rgb(80, 100, 140),
            
            // Interpolation & Utility
            NodeKind::Mix | NodeKind::Step | NodeKind::Smoothstep | NodeKind::Clamp => Color32::from_rgb(140, 100, 80),
            NodeKind::Fract | NodeKind::Floor | NodeKind::Ceil | NodeKind::Abs | NodeKind::Min | NodeKind::Max | NodeKind::Pow | NodeKind::Sqrt | NodeKind::Sign => Color32::from_rgb(100, 80, 120),
            
            // Color Operations
            NodeKind::RGB | NodeKind::HSV | NodeKind::ColorMix | NodeKind::ColorAdjust => Color32::from_rgb(180, 120, 60),
            
            // Noise & Procedural
            NodeKind::Noise2D | NodeKind::Noise3D | NodeKind::Voronoi => Color32::from_rgb(120, 140, 80),
            
            // Texture
            NodeKind::TextureSample | NodeKind::TextureSampleLod | NodeKind::TextureSize => Color32::from_rgb(120, 80, 40),
        }
    }

    fn get_port_color(&self, kind: &PortKind) -> Color32 {
        match kind {
            PortKind::Float => Color32::from_rgb(100, 150, 255),
            PortKind::Vec2 => Color32::from_rgb(150, 100, 255),
            PortKind::Vec3 => Color32::from_rgb(255, 100, 150),
            PortKind::Vec4 => Color32::from_rgb(255, 150, 100),
            PortKind::Texture => Color32::from_rgb(100, 255, 150),
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        if ui.input(|i| i.key_pressed(Key::Delete)) {
            self.delete_selected_nodes(node_graph);
        }
        
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(Key::Z)) {
            if ui.input(|i| i.modifiers.shift) {
                self.redo(node_graph);
            } else {
                self.undo(node_graph);
            }
        }
        
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(Key::C)) {
            self.copy_selected_nodes(node_graph);
        }
        
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(Key::V)) {
            self.paste_nodes(node_graph);
        }
        
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(Key::S)) {
            self.save_state_for_undo(node_graph);
        }
    }

    fn save_state_for_undo(&mut self, node_graph: &NodeGraph) {
        let state = NodeGraphState {
            nodes: node_graph.nodes.clone(),
            connections: node_graph.connections.clone(),
        };
        
        self.undo_stack.push(state);
        if self.undo_stack.len() > self.max_undo_steps {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self, node_graph: &mut NodeGraph) {
        if let Some(state) = self.undo_stack.pop() {
            let current_state = NodeGraphState {
                nodes: node_graph.nodes.clone(),
                connections: node_graph.connections.clone(),
            };
            self.redo_stack.push(current_state);
            
            node_graph.nodes = state.nodes;
            node_graph.connections = state.connections;
        }
    }

    fn redo(&mut self, node_graph: &mut NodeGraph) {
        if let Some(state) = self.redo_stack.pop() {
            let current_state = NodeGraphState {
                nodes: node_graph.nodes.clone(),
                connections: node_graph.connections.clone(),
            };
            self.undo_stack.push(current_state);
            
            node_graph.nodes = state.nodes;
            node_graph.connections = state.connections;
        }
    }

    fn copy_selected_nodes(&mut self, node_graph: &NodeGraph) {
        if !self.selected_nodes.is_empty() {
            let mut nodes_to_copy = Vec::new();
            
            for &node_id in &self.selected_nodes {
                if let Some(node) = node_graph.nodes.get(&node_id) {
                    let pos = self.node_positions.get(&node_id).copied().unwrap_or(Pos2::ZERO);
                    nodes_to_copy.push((node_id, node.clone(), pos));
                }
            }
            
            self.clipboard = Some(nodes_to_copy);
        }
    }

    fn paste_nodes(&mut self, node_graph: &mut NodeGraph) {
        if let Some(nodes_to_copy) = &self.clipboard {
            self.save_state_for_undo(node_graph);
            
            self.selected_nodes.clear();
            
            for (_, node, original_pos) in nodes_to_copy {
                let new_node_id = NodeId::new();
                let new_pos = *original_pos + vec2(20.0, 20.0);
                
                node_graph.nodes.insert(new_node_id, node.clone());
                self.node_positions.insert(new_node_id, new_pos);
                self.selected_nodes.insert(new_node_id);
            }
        }
    }

    fn delete_selected_nodes(&mut self, node_graph: &mut NodeGraph) {
        if !self.selected_nodes.is_empty() {
            self.save_state_for_undo(node_graph);
            
            for &node_id in &self.selected_nodes {
                node_graph.nodes.remove(&node_id);
                self.node_positions.remove(&node_id);
                
                // Remove connections involving this node
                node_graph.connections.retain(|conn| {
                    conn.from_node != node_id && conn.to_node != node_id
                });
            }
            
            self.selected_nodes.clear();
        }
    }

    fn update_selection_from_box(&mut self, start_pos: Pos2, end_pos: Pos2, node_graph: &NodeGraph) {
        let rect = Rect::from_two_pos(start_pos, end_pos);
        self.selected_nodes.clear();
        
        for (&node_id, node) in &node_graph.nodes {
            let node_screen_pos = self.node_positions.get(&node_id).copied().unwrap_or(Pos2::ZERO) + self.pan;
            let node_rect = Rect::from_min_size(node_screen_pos, vec2(180.0, 120.0));
            if rect.intersects(node_rect) {
                self.selected_nodes.insert(node_id);
            }
        }
    }

    fn draw_selection_box(&self, ui: &mut Ui, start_pos: Pos2, current_pos: Pos2) {
        let painter = ui.painter();
        let rect = Rect::from_two_pos(start_pos, current_pos);
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(100, 150, 255)));
        painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(100, 150, 255, 30));
    }

    // Helper functions for port positions
    fn get_output_port_pos(&self, node_id: NodeId, port_id: PortId, node: &Node) -> Option<Pos2> {
        if let Some(node_pos) = self.node_positions.get(&node_id) {
            let node_screen_pos = *node_pos + self.pan;
            for (i, port) in node.outputs.iter().enumerate() {
                if port.id == port_id {
                    return Some(node_screen_pos + vec2(170.0, 40.0 + (i as f32 * 25.0)));
                }
            }
        }
        None
    }

    fn get_input_port_pos(&self, node_id: NodeId, port_id: PortId, node: &Node) -> Option<Pos2> {
        if let Some(node_pos) = self.node_positions.get(&node_id) {
            let node_screen_pos = *node_pos + self.pan;
            for (i, port) in node.inputs.iter().enumerate() {
                if port.id == port_id {
                    return Some(node_screen_pos + vec2(10.0, 40.0 + (i as f32 * 25.0)));
                }
            }
        }
        None
    }
}