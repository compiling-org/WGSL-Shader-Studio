use bevy_egui::egui::*;
use bevy_egui::egui::epaint::{CubicBezierShape, StrokeKind};
use crate::node_graph::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct VisualNodeEditor {
    pan: Vec2,
    zoom: f32,
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
    // Execution state
    pub auto_compile: bool,
    pub last_generated_wgsl: Option<String>,
    pub compilation_errors: Vec<String>,
    pub is_compiling: Arc<AtomicBool>,
    pub last_graph_hash: Option<String>,
}

#[derive(Clone)]
struct NodeGraphState {
    nodes: HashMap<NodeId, Node>,
    connections: Vec<Connection>,
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
            // Execution state
            auto_compile: true,
            last_generated_wgsl: None,
            compilation_errors: Vec::new(),
            is_compiling: Arc::new(AtomicBool::new(false)),
            last_graph_hash: None,
        }
    }
}

impl VisualNodeEditor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate WGSL code from the current node graph and update execution state
    pub fn generate_and_compile(&mut self, node_graph: &NodeGraph, width: u32, height: u32) -> Result<String, Vec<String>> {
        if self.is_compiling.load(Ordering::Relaxed) {
            return Err(vec!["Compilation already in progress".to_string()]);
        }
        
        self.is_compiling.store(true, Ordering::Relaxed);
        self.compilation_errors.clear();
        
        // Generate WGSL code
        let wgsl_code = node_graph.generate_wgsl(width, height);
        self.last_generated_wgsl = Some(wgsl_code.clone());
        
        // Basic validation
        let mut errors = Vec::new();
        
        // Check if there's an output node
        let has_output = node_graph.nodes.values().any(|n| matches!(n.kind, NodeKind::OutputColor));
        if !has_output {
            errors.push("No OutputColor node found in graph".to_string());
        }
        
        // Check for unconnected required inputs
        for (node_id, node) in &node_graph.nodes {
            for input in &node.inputs {
                let has_connection = node_graph.connections.iter().any(|c| c.to_node == *node_id && c.to_port == input.id);
                if !has_connection && !self.is_input_optional(&node.kind, input) {
                    errors.push(format!("Node '{}' has unconnected input '{}'", node.title, input.name));
                }
            }
        }
        
        self.is_compiling.store(false, Ordering::Relaxed);
        
        if errors.is_empty() {
            Ok(wgsl_code)
        } else {
            self.compilation_errors = errors.clone();
            Err(errors)
        }
    }
    
    /// Check if an input port is optional for a given node kind
    fn is_input_optional(&self, node_kind: &NodeKind, input: &Port) -> bool {
        match node_kind {
            NodeKind::ConstantFloat(_) | NodeKind::ConstantVec2(_) | NodeKind::ConstantVec3(_) | NodeKind::ConstantVec4(_) |
            NodeKind::Time | NodeKind::UV | NodeKind::Resolution | NodeKind::Mouse => true,
            _ => false,
        }
    }
    
    /// Auto-compile if enabled and graph has changed
    pub fn auto_compile_if_needed(&mut self, node_graph: &NodeGraph, width: u32, height: u32) -> Option<Result<String, Vec<String>>> {
        if self.auto_compile && !self.is_compiling.load(Ordering::Relaxed) {
            // Simple change detection - in a real implementation, you'd track graph version
            let current_hash = self.calculate_graph_hash(node_graph);
            let last_hash = self.last_graph_hash.as_ref();
            
            if last_hash.map_or(true, |h| h != &current_hash) {
                self.last_graph_hash = Some(current_hash);
                return Some(self.generate_and_compile(node_graph, width, height));
            }
        }
        None
    }
    
    /// Calculate a simple hash of the graph for change detection
    fn calculate_graph_hash(&self, node_graph: &NodeGraph) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        node_graph.nodes.len().hash(&mut hasher);
        node_graph.connections.len().hash(&mut hasher);
        format!("{:x}", hasher.finish())
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
            if self.is_compiling.load(Ordering::Relaxed) {
                ui.label("⏳ Compiling...");
            } else if !self.compilation_errors.is_empty() {
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
            if ui.button("Output").clicked() {
                let id = node_graph.add_node(NodeKind::OutputColor, "Output Color", (400.0, 300.0));
                self.node_positions.insert(id, (400.0, 300.0));
            }
        });
        
        ui.separator();
        
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
        self.draw_grid(ui, available_rect);

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
    }

    fn draw_grid(&self, ui: &mut Ui, rect: Rect) {
        // Create painter after port allocation to avoid borrow checker issues
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
        // Handle node dragging first (this needs mutable access)
        if let Some(dragging_id) = self.dragging_node {
            if dragging_id == node_id {
                if let Some(node) = node_graph.nodes.get_mut(&node_id) {
                    let mouse_pos = ui.input(|i| i.pointer.latest_pos().unwrap_or(Pos2::ZERO));
                    node.pos = (mouse_pos.x - self.drag_offset.x - self.pan.x, mouse_pos.y - self.drag_offset.y - self.pan.y);
                }
            }
        }
        
        // Now work with immutable access for rendering
        if let Some(node) = node_graph.nodes.get(&node_id) {
            let node_pos = pos2(node.pos.0, node.pos.1) + self.pan;
            let node_size = vec2(180.0, 120.0);
            let node_rect = Rect::from_min_size(node_pos, node_size);
            
            let response = ui.allocate_rect(node_rect, Sense::click_and_drag());
            let painter = ui.painter();
            
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
            
            if response.drag_started() {
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
            
            if response.drag_delta() != Vec2::ZERO {
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
            
            painter.rect_filled(node_rect, 4.0, bg_color);
            
            // Node border with enhanced selection
            let border_color = if self.selected_nodes.contains(&node_id) {
                Color32::from_rgb(255, 255, 100)
            } else if self.hovered_node == Some(node_id) {
                Color32::from_rgb(200, 200, 200)
            } else {
                Color32::from_gray(60)
            };
            let stroke = Stroke::new(2.0, border_color);
            painter.rect_stroke(node_rect, 4.0, stroke, StrokeKind::Inside);
            
            // Node title with enhanced styling
            let title_pos = node_pos + vec2(10.0, 20.0);
            let title_color = if self.selected_nodes.contains(&node_id) {
                Color32::from_rgb(255, 255, 150)
            } else {
                Color32::WHITE
            };
            painter.text(
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
                
                // Draw the port visually - pass ui instead of painter to avoid borrow issues
                self.draw_port_visual(ui, port_pos, input, false, node_id, node_graph);
            }
            
            // Draw output ports and collect clicks
            for (i, output) in node.outputs.iter().enumerate() {
                let port_pos = pos2(node_pos.x + node_size.x + 8.0, node_pos.y + 40.0 + (i as f32 * 25.0));
                let port_rect = Rect::from_center_size(port_pos, vec2(16.0, 16.0));
                let port_response = ui.allocate_rect(port_rect, Sense::click());
                
                if port_response.clicked() {
                    output_clicks.push(output.id);
                }
                
                // Draw the port visually - pass ui instead of painter to avoid borrow issues
                self.draw_port_visual(ui, port_pos, output, true, node_id, node_graph);
            }
            
            // Handle port connections after drawing
            for input_id in input_clicks {
                if let Some((start_node, start_port, is_output)) = self.connection_start {
                    if !is_output {
                        // Connect output to this input
                        node_graph.connect(start_node, start_port, node_id, input_id);
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
                        node_graph.connect(node_id, output_id, start_node, start_port);
                        self.connection_start = None;
                    }
                } else {
                    // Start connection from this output
                    self.connection_start = Some((node_id, output_id, true));
                }
            }
        }
    }

    fn draw_port_visual(&mut self, ui: &mut Ui, pos: Pos2, port: &Port, is_output: bool, node_id: NodeId, node_graph: &NodeGraph) {
        let port_color = self.get_port_color(&port.kind);
        let port_size = 8.0;
        let painter = ui.painter();
        
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
        painter.circle(pos, port_size_final, final_color, Stroke::new(2.0, border_color));
        
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
        let response = ui.allocate_rect(port_rect, Sense::click());
        
        // Update hover state
        if response.hovered() {
            self.hovered_port = Some((node_id, port.id));
        } else if self.hovered_port == Some((node_id, port.id)) {
            self.hovered_port = None;
        }
        
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
        painter.circle(pos, port_size_final, final_color, Stroke::new(2.0, border_color));
        
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
        
        response
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
        let painter = ui.painter();
        let rect = Rect::from_two_pos(start_pos, current_pos);
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(100, 150, 255)), StrokeKind::Inside);
        painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(100, 150, 255, 30));
    }
}