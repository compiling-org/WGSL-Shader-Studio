//! Enhanced Visual Node Editor
//! Combines concepts from Rust Visual Editor with existing node graph implementation
//! Supports advanced features like undo/redo, copy/paste, compilation checking, and comprehensive node types

use bevy_egui::egui::*;
use bevy_egui::egui::epaint::CubicBezierShape;
use crate::node_graph::{NodeGraph, NodeId, NodeKind, PortId, PortKind};
use crate::visual_language_integration::VisualLanguageIntegration;
use crate::visual_language_manager::VisualLanguageManager;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct EnhancedVisualNodeEditor {
    pan: Vec2,
    pub zoom: f32,
    selected_node: Option<NodeId>,
    dragging_node: Option<NodeId>,
    drag_offset: Vec2,
    connection_start: Option<(NodeId, PortId, bool)>, // (node_id, port_id, is_output)
    node_positions: HashMap<NodeId, (f32, f32)>,
    // Enhanced features
    hovered_node: Option<NodeId>,
    hovered_port: Option<(NodeId, PortId)>,
    drag_box_start: Option<Pos2>,
    selected_nodes: HashSet<NodeId>,
    clipboard: Option<String>, // For copy/paste functionality
    undo_stack: Vec<NodeGraphState>,
    redo_stack: Vec<NodeGraphState>,
    max_undo_steps: usize,
    auto_compile: bool,
    show_grid: bool,
    snap_to_grid: bool,
    grid_size: f32,
    is_dragging_canvas: bool,
    last_mouse_pos: Option<Pos2>,
    // Compilation status
    compilation_status: CompilationStatus,
    // Visual language integration
    visual_language_mode: bool,
    code_generation_enabled: bool,
    pub visual_language_integration: VisualLanguageIntegration,
    visual_language_manager: VisualLanguageManager,
}

#[derive(Debug, Clone)]
struct NodeGraphState {
    nodes: HashMap<NodeId, crate::node_graph::Node>,
    connections: Vec<crate::node_graph::Connection>,
}

#[derive(Debug, Clone)]
pub enum CompilationStatus {
    Idle,
    Compiling,
    Success { message: String },
    Error { message: String },
}

impl Default for EnhancedVisualNodeEditor {
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
            auto_compile: true,
            show_grid: true,
            snap_to_grid: true,
            grid_size: 20.0,
            is_dragging_canvas: false,
            last_mouse_pos: None,
            compilation_status: CompilationStatus::Idle,
            visual_language_mode: false,
            code_generation_enabled: true,
            visual_language_integration: VisualLanguageIntegration::new(),
            visual_language_manager: VisualLanguageManager::new(),
        }
    }
}

impl EnhancedVisualNodeEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Store current mouse position
        self.last_mouse_pos = response.interact_pointer_pos();
        
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ui, node_graph);
        
        // Handle canvas panning
        if response.dragged_by(PointerButton::Middle) || 
           (response.dragged_by(PointerButton::Primary) && ui.input(|i| i.modifiers.ctrl)) {
            self.pan += response.drag_delta();
            self.is_dragging_canvas = true;
        } else if self.is_dragging_canvas && !response.dragged() {
            self.is_dragging_canvas = false;
        }
        
        // Handle selection box
        if response.dragged_by(PointerButton::Primary) && 
           !ui.input(|i| i.modifiers.ctrl) && 
           self.dragging_node.is_none() && 
           !self.is_dragging_canvas {
            if self.drag_box_start.is_none() {
                self.drag_box_start = Some(response.interact_pointer_pos().unwrap_or(Pos2::ZERO));
            }
        } else {
            if let Some(start_pos) = self.drag_box_start {
                if let Some(current_pos) = response.interact_pointer_pos() {
                    self.update_selection_from_box(start_pos, current_pos, node_graph);
                }
                self.drag_box_start = None;
            }
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
        if self.show_grid {
            self.draw_grid(ui, available_rect);
        }

        // Draw selection box if active
        if let Some(start_pos) = self.drag_box_start {
            if let Some(current_pos) = response.interact_pointer_pos() {
                self.draw_selection_box(ui, start_pos, current_pos);
            }
        }

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
        
        // Handle node dragging
        self.handle_node_dragging(ui, node_graph);
        
        // Handle connection creation
        self.handle_connection_creation(ui, node_graph);
    }

    fn draw_grid(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        let grid_size = self.grid_size * self.zoom;
        
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
                    
                    painter.add(Shape::CubicBezier(CubicBezierShape::from_points_stroke(
                        [start, control1, control2, end],
                        false,
                        Color32::TRANSPARENT,
                        Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                    )));
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
                
                painter.add(Shape::CubicBezier(CubicBezierShape::from_points_stroke(
                    [start, control1, control2, mouse_pos],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(2.0, Color32::from_rgb(255, 200, 100)),
                )));
            }
        }
    }

    fn draw_node(&mut self, ui: &mut Ui, node_id: NodeId, node_graph: &mut NodeGraph) {
        // Now work with immutable access for rendering
        if let Some(node) = node_graph.nodes.get(&node_id) {
            let node_pos = pos2(node.pos.0, node.pos.1) + self.pan;
            let node_size = vec2(180.0, 120.0);
            let node_rect = Rect::from_min_size(node_pos, node_size);
            
            let response = ui.allocate_rect(node_rect, Sense::click_and_drag());
            
            // Update hover state
            if response.hovered() {
                self.hovered_node = Some(node_id);
            } else if self.hovered_node == Some(node_id) {
                self.hovered_node = None;
            }
            
            // Handle node selection and dragging
            if response.clicked() {
                if ui.input(|i| i.modifiers.shift) {
                    // Toggle selection with shift
                    if self.selected_nodes.contains(&node_id) {
                        self.selected_nodes.remove(&node_id);
                    } else {
                        self.selected_nodes.insert(node_id);
                    }
                } else {
                    // Single selection
                    self.selected_nodes.clear();
                    self.selected_nodes.insert(node_id);
                    self.selected_node = Some(node_id);
                }
            }
            
            if response.drag_started() && !ui.input(|i| i.pointer.secondary_down()) {
                self.dragging_node = Some(node_id);
                let mouse_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO));
                self.drag_offset = mouse_pos - node_pos;
                
                // If dragging a node that's not selected, select it
                if !self.selected_nodes.contains(&node_id) {
                    self.selected_nodes.clear();
                    self.selected_nodes.insert(node_id);
                    self.selected_node = Some(node_id);
                }
            }
            
            if response.drag_stopped() {
                self.dragging_node = None;
            }
            
            // Node background with selection and hover highlight
            let mut bg_color = self.get_node_color(&node.kind);
            if self.selected_nodes.contains(&node_id) {
                // Brighten selected node
                bg_color = Color32::from_rgb(
                    (bg_color.r() + 40).min(255),
                    (bg_color.g() + 40).min(255),
                    (bg_color.b() + 40).min(255)
                );
            } else if self.hovered_node == Some(node_id) {
                // Slight highlight for hover
                bg_color = Color32::from_rgb(
                    (bg_color.r() + 20).min(255),
                    (bg_color.g() + 20).min(255),
                    (bg_color.b() + 20).min(255)
                );
            }
            
            ui.painter().rect_filled(node_rect, CornerRadius::same(4u8), bg_color);
            
            // Node border with enhanced selection
            let border_color = if self.selected_nodes.contains(&node_id) {
                Color32::from_rgb(255, 255, 100)
            } else if self.hovered_node == Some(node_id) {
                Color32::from_rgb(200, 200, 200)
            } else {
                Color32::from_gray(60)
            };
            let stroke = Stroke::new(2.0, border_color);
            ui.painter().rect_stroke(node_rect, Rounding::ZERO, stroke, StrokeKind::Outside);
            
            // Node title with enhanced styling
            let title_pos = node_pos + vec2(10.0, 20.0);
            let title_color = if self.selected_nodes.contains(&node_id) {
                Color32::from_rgb(255, 255, 150)
            } else {
                Color32::WHITE
            };
            ui.painter().text(
                title_pos,
                Align2::LEFT_CENTER,
                &node.title,
                FontId::proportional(14.0),
                title_color
            );
            
            // Collect port click events first to avoid borrow checker issues
            let mut input_clicks = Vec::new();
            let mut output_clicks = Vec::new();
            
            // Draw input ports and collect clicks
            for (i, input) in node.inputs.iter().enumerate() {
                let port_pos = pos2(node_pos.x - 8.0, node_pos.y + 40.0 + (i as f32 * 25.0));
                let port_rect = Rect::from_center_size(port_pos, vec2(16.0, 16.0));
                let port_response = ui.allocate_rect(port_rect, Sense::click());
                
                if port_response.clicked() {
                    input_clicks.push(input.id);
                }
                
                // Draw the port visually
                self.draw_port_visual(ui.painter(), port_pos, input, false, node_id, node_graph);
            }
            
            // Draw output ports and collect clicks
            for (i, output) in node.outputs.iter().enumerate() {
                let port_pos = pos2(node_pos.x + node_size.x + 8.0, node_pos.y + 40.0 + (i as f32 * 25.0));
                let port_rect = Rect::from_center_size(port_pos, vec2(16.0, 16.0));
                let port_response = ui.allocate_rect(port_rect, Sense::click());
                
                if port_response.clicked() {
                    output_clicks.push(output.id);
                }
                
                // Draw the port visually
                self.draw_port_visual(ui.painter(), port_pos, output, true, node_id, node_graph);
            }
            
            // Handle port connections after drawing
            for input_id in input_clicks {
                if let Some((start_node, start_port, is_output)) = self.connection_start {
                    if !is_output {
                        // Connect output to this input
                        let _ = node_graph.connect(start_node, start_port, node_id, input_id);
                        self.connection_start = None;
                    }
                } else {
                    // Start connection from this input (unusual but allowed)
                    self.connection_start = Some((node_id, input_id, false));
                }
            }
            
            for output_id in output_clicks {
                if let Some((start_node, start_port, is_output)) = self.connection_start {
                    if is_output {
                        // Connect this output to input
                        let _ = node_graph.connect(node_id, output_id, start_node, start_port);
                        self.connection_start = None;
                    }
                } else {
                    // Start connection from this output
                    self.connection_start = Some((node_id, output_id, true));
                }
            }
        }
    }

    fn draw_port_visual(&mut self, painter: &Painter, pos: Pos2, port: &crate::node_graph::Port, is_output: bool, node_id: NodeId, node_graph: &NodeGraph) {
        let port_color = self.get_port_color(&port.kind);
        let port_size = 8.0;
        
        // Enhanced port visual feedback
        let is_connected = self.is_port_connected(node_graph, node_id, port.id);
        let is_hovered = self.hovered_port == Some((node_id, port.id));
        let is_active_connection = self.connection_start.map_or(false, |(start_node, start_port, _)| {
            start_node == node_id && start_port == port.id
        });
        
        // Determine final port appearance
        let mut final_color = port_color;
        let mut border_color = Color32::WHITE;
        let mut port_size_final = port_size;
        
        if is_hovered || is_active_connection {
            // Brighten on hover or when active
            final_color = Color32::from_rgb(
                (port_color.r() + 60).min(255),
                (port_color.g() + 60).min(255),
                (port_color.b() + 60).min(255)
            );
            port_size_final = port_size * 1.2; // Slightly larger
        }
        
        if is_connected {
            // Connected ports get a stronger border
            border_color = Color32::from_rgb(255, 255, 200);
        }
        
        // Draw port with enhanced visuals
        painter.circle_filled(pos, port_size_final, final_color);
        painter.circle_stroke(pos, port_size_final, Stroke::new(2.0, border_color));
        
        // Add glow effect for hovered/connected ports
        if is_hovered || is_connected {
            painter.circle_stroke(pos, port_size_final + 3.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 50)));
        }
        
        // Port label
        let label_pos = if is_output {
            pos - vec2(20.0, 0.0)
        } else {
            pos + vec2(20.0, 0.0)
        };
        
        let label_color = if is_hovered {
            Color32::from_rgb(255, 255, 150)
        } else {
            Color32::WHITE
        };
        
        painter.text(
            label_pos,
            if is_output { Align2::RIGHT_CENTER } else { Align2::LEFT_CENTER },
            &port.name,
            FontId::proportional(11.0),
            label_color
        );
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
            
            // Output
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

    fn handle_keyboard_shortcuts(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        ui.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(Key::A) {
                    // Select all nodes
                    self.selected_nodes.clear();
                    for &node_id in node_graph.nodes.keys() {
                        self.selected_nodes.insert(node_id);
                    }
                }
                if i.key_pressed(Key::D) {
                    // Deselect all
                    self.selected_nodes.clear();
                    self.selected_node = None;
                }
                if i.key_pressed(Key::C) && !self.selected_nodes.is_empty() {
                    // Copy selected nodes
                    self.copy_selected_nodes(node_graph);
                }
                if i.key_pressed(Key::V) && self.clipboard.is_some() {
                    // Paste nodes
                    self.paste_nodes(node_graph);
                }
                if i.key_pressed(Key::Z) && !i.modifiers.shift {
                    // Undo
                    self.undo(node_graph);
                }
                if i.key_pressed(Key::Z) && i.modifiers.shift || i.key_pressed(Key::Y) {
                    // Redo
                    self.redo(node_graph);
                }
                if i.key_pressed(Key::Delete) && !self.selected_nodes.is_empty() {
                    // Delete selected nodes
                    self.delete_selected_nodes(node_graph);
                }
            }
        });
    }

    fn save_state_for_undo(&mut self, node_graph: &NodeGraph) {
        if self.undo_stack.len() >= self.max_undo_steps {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(NodeGraphState {
            nodes: node_graph.nodes.clone(),
            connections: node_graph.connections.clone(),
        });
        self.redo_stack.clear();
    }

    fn undo(&mut self, node_graph: &mut NodeGraph) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(NodeGraphState {
                nodes: node_graph.nodes.clone(),
                connections: node_graph.connections.clone(),
            });
            node_graph.nodes = state.nodes;
            node_graph.connections = state.connections;
        }
    }

    fn redo(&mut self, node_graph: &mut NodeGraph) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(NodeGraphState {
                nodes: node_graph.nodes.clone(),
                connections: node_graph.connections.clone(),
            });
            node_graph.nodes = state.nodes;
            node_graph.connections = state.connections;
        }
    }

    fn copy_selected_nodes(&mut self, node_graph: &NodeGraph) {
        let selected_data = serde_json::json!({
            "nodes": self.selected_nodes.iter().filter_map(|&id| node_graph.nodes.get(&id)).collect::<Vec<_>>(),
            "connections": node_graph.connections.iter().filter(|conn| {
                self.selected_nodes.contains(&conn.from_node) && self.selected_nodes.contains(&conn.to_node)
            }).collect::<Vec<_>>()
        });
        self.clipboard = Some(selected_data.to_string());
    }

    fn paste_nodes(&mut self, node_graph: &mut NodeGraph) {
        if let Some(ref clipboard_data) = self.clipboard {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(clipboard_data) {
                // Simple paste implementation - would need proper offset logic
                self.save_state_for_undo(node_graph);
                // Implementation would go here
            }
        }
    }

    fn delete_selected_nodes(&mut self, node_graph: &mut NodeGraph) {
        self.save_state_for_undo(node_graph);
        for &node_id in &self.selected_nodes.clone() {
            node_graph.nodes.remove(&node_id);
            node_graph.connections.retain(|conn| conn.from_node != node_id && conn.to_node != node_id);
        }
        self.selected_nodes.clear();
        self.selected_node = None;
    }

    fn update_selection_from_box(&mut self, start_pos: Pos2, end_pos: Pos2, node_graph: &NodeGraph) {
        let rect = Rect::from_two_pos(start_pos, end_pos);
        self.selected_nodes.clear();
        
        for (&node_id, node) in &node_graph.nodes {
            let node_screen_pos = pos2(node.pos.0, node.pos.1) + self.pan;
            let node_rect = Rect::from_min_size(node_screen_pos, vec2(180.0, 120.0));
            if rect.intersects(node_rect) {
                self.selected_nodes.insert(node_id);
            }
        }
    }

    fn draw_selection_box(&self, ui: &mut Ui, start_pos: Pos2, current_pos: Pos2) {
        let rect = Rect::from_two_pos(start_pos, current_pos);
        ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, Color32::from_rgb(100, 150, 255)), StrokeKind::Outside);
        ui.painter().rect_filled(rect, CornerRadius::same(0u8), Color32::from_rgba_unmultiplied(100, 150, 255, 30));
    }

    fn handle_node_dragging(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        if let Some(dragging_id) = self.dragging_node {
            if let Some(mouse_pos) = self.last_mouse_pos {
                if let Some(node) = node_graph.nodes.get_mut(&dragging_id) {
                    let new_x = mouse_pos.x - self.drag_offset.x - self.pan.x;
                    let new_y = mouse_pos.y - self.drag_offset.y - self.pan.y;
                    
                    if self.snap_to_grid {
                        let snapped_x = (new_x / self.grid_size).round() * self.grid_size;
                        let snapped_y = (new_y / self.grid_size).round() * self.grid_size;
                        node.pos = (snapped_x, snapped_y);
                    } else {
                        node.pos = (new_x, new_y);
                    }
                }
            }
        }
    }

    fn handle_connection_creation(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        if ui.input(|i| i.pointer.any_released()) {
            if let Some((from_node, from_port, is_output)) = self.connection_start.take() {
                if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    for (to_id, to_node) in node_graph.nodes.iter() {
                        for (idx, input) in to_node.inputs.iter().enumerate() {
                            let port_pos = pos2(to_node.pos.0 - 8.0, to_node.pos.1 + 40.0 + (idx as f32 * 25.0));
                            if mouse_pos.distance(port_pos) < 10.0 {
                                let _ = node_graph.connect(from_node, from_port, *to_id, input.id);
                                return;
                            }
                        }
                    }
                }
            }
        }
        
        if ui.input(|i| i.pointer.primary_pressed()) {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                for (nid, node) in node_graph.nodes.iter() {
                    for (idx, output) in node.outputs.iter().enumerate() {
                        let port_pos = pos2(node.pos.0 + 180.0 + 8.0, node.pos.1 + 40.0 + (idx as f32 * 25.0));
                        if mouse_pos.distance(port_pos) < 10.0 {
                            self.connection_start = Some((*nid, output.id, true));
                            return;
                        }
                    }
                }
            }
        }
    }

    // Generate WGSL code from the node graph
    pub fn generate_wgsl(&self, node_graph: &NodeGraph) -> Result<String, String> {
        // Use the existing node graph's generate_wgsl method
        Ok(node_graph.generate_wgsl(800, 600))
    }

    // Get/set auto compile
    pub fn auto_compile(&self) -> bool {
        self.auto_compile
    }

    pub fn set_auto_compile(&mut self, auto_compile: bool) {
        self.auto_compile = auto_compile;
    }

    // Get/set grid visibility
    pub fn show_grid(&self) -> bool {
        self.show_grid
    }

    pub fn set_show_grid(&mut self, show_grid: bool) {
        self.show_grid = show_grid;
    }

    // Get/set snap to grid
    pub fn snap_to_grid(&self) -> bool {
        self.snap_to_grid
    }

    pub fn set_snap_to_grid(&mut self, snap_to_grid: bool) {
        self.snap_to_grid = snap_to_grid;
    }

    // Node position methods
    pub fn set_node_position(&mut self, node_id: NodeId, position: (f32, f32)) {
        self.node_positions.insert(node_id, position);
    }
    
    pub fn get_node_position(&self, node_id: NodeId) -> Option<(f32, f32)> {
        self.node_positions.get(&node_id).copied()
    }
    
    // Visual language integration methods
    pub fn visual_language_mode(&self) -> bool {
        self.visual_language_mode
    }

    pub fn set_visual_language_mode(&mut self, enabled: bool) {
        self.visual_language_mode = enabled;
    }

    pub fn code_generation_enabled(&self) -> bool {
        self.code_generation_enabled
    }

    pub fn set_code_generation_enabled(&mut self, enabled: bool) {
        self.code_generation_enabled = enabled;
    }

    pub fn get_compilation_status(&self) -> &CompilationStatus {
        &self.compilation_status
    }

    pub fn compile_node_graph(&mut self, node_graph: &NodeGraph) -> Result<String, String> {
        self.compilation_status = CompilationStatus::Compiling;
        
        // Use the visual language integration for compilation with validation
        match self.visual_language_integration.compile_node_graph(node_graph) {
            Ok(wgsl) => {
                self.compilation_status = CompilationStatus::Success {
                    message: format!("Generated {} characters of WGSL", wgsl.len())
                };
                Ok(wgsl)
            }
            Err(e) => {
                self.compilation_status = CompilationStatus::Error {
                    message: e.join("\n")
                };
                Err(e.join("\n"))
            }
        }
    }
}