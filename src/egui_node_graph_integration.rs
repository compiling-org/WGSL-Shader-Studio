use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2 as EguiVec2, Widget, WidgetInfo, WidgetText};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::bevy_shader_graph_integration::{NodeId, PortId, ShaderNode, NodeType, PortType, NodeProperty};
use crate::shader_transpiler::{ShaderLanguage, TranspilerOptions};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphWidgetId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWidget {
    pub id: NodeId,
    pub position: Pos2,
    pub size: EguiVec2,
    pub title: String,
    pub color: Color32,
    pub border_color: Color32,
    pub header_height: f32,
    pub corner_radius: f32,
    pub shadow_offset: EguiVec2,
    pub shadow_blur: f32,
    pub shadow_color: Color32,
    pub is_selected: bool,
    pub is_hovered: bool,
    pub is_dragging: bool,
    pub is_collapsed: bool,
    pub opacity: f32,
    pub animation_state: AnimationState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationState {
    Idle,
    Hovering(f32),
    Selecting(f32),
    Dragging(f32),
    Collapsing(f32),
    Expanding(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortWidget {
    pub id: PortId,
    pub node_id: NodeId,
    pub position: Pos2,
    pub radius: f32,
    pub color: Color32,
    pub border_color: Color32,
    pub port_type: PortType,
    pub is_connected: bool,
    pub is_hovered: bool,
    pub is_active: bool,
    pub animation_offset: f32,
    pub pulse_phase: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionWidget {
    pub from_port: PortId,
    pub to_port: PortId,
    pub from_pos: Pos2,
    pub to_pos: Pos2,
    pub color: Color32,
    pub thickness: f32,
    pub is_selected: bool,
    pub is_hovered: bool,
    pub animation_offset: f32,
    pub flow_direction: ConnectionFlow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionFlow {
    None,
    Forward(f32),
    Backward(f32),
    Bidirectional(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphViewport {
    pub offset: EguiVec2,
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub pan_speed: f32,
    pub zoom_speed: f32,
    pub is_panning: bool,
    pub pan_start: Option<Pos2>,
    pub bounds: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphGrid {
    pub enabled: bool,
    pub size: f32,
    pub color: Color32,
    pub subdivisions: u32,
    pub subdivision_color: Color32,
    pub snap_enabled: bool,
    pub snap_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSelection {
    pub selected_nodes: HashSet<NodeId>,
    pub selected_connections: HashSet<(PortId, PortId)>,
    pub selection_rect: Option<Rect>,
    pub selection_start: Option<Pos2>,
    pub is_selecting: bool,
    pub selection_mode: SelectionMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectionMode {
    Replace,
    Add,
    Remove,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphClipboard {
    pub copied_nodes: Vec<ShaderNode>,
    pub copied_connections: Vec<(PortId, PortId)>,
    pub offset: EguiVec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraphWidget {
    pub id: GraphWidgetId,
    pub nodes: HashMap<NodeId, NodeWidget>,
    pub ports: HashMap<PortId, PortWidget>,
    pub connections: Vec<ConnectionWidget>,
    pub viewport: GraphViewport,
    pub grid: GraphGrid,
    pub selection: GraphSelection,
    pub clipboard: GraphClipboard,
    pub interaction_state: InteractionState,
    pub visual_settings: VisualSettings,
    pub performance_stats: PerformanceStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionState {
    Idle,
    DraggingNode(NodeId, Pos2),
    DraggingPort(PortId, Pos2),
    Panning(EguiVec2),
    Zooming(f32, Pos2),
    Selecting(Rect),
    Connecting(PortId, Pos2),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualSettings {
    pub node_opacity: f32,
    pub connection_opacity: f32,
    pub grid_opacity: f32,
    pub selection_opacity: f32,
    pub animation_speed: f32,
    pub pulse_speed: f32,
    pub glow_intensity: f32,
    pub antialiasing: bool,
    pub high_dpi_support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub frame_time_ms: f32,
    pub node_count: usize,
    pub connection_count: usize,
    pub render_time_ms: f32,
    pub interaction_time_ms: f32,
    pub memory_usage_mb: f32,
}

#[derive(Debug, Error)]
pub enum NodeGraphWidgetError {
    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),
    
    #[error("Port not found: {0:?}")]
    PortNotFound(PortId),
    
    #[error("Invalid connection: {0:?} -> {1:?}")]
    InvalidConnection(PortId, PortId),
    
    #[error("Widget error: {0}")]
    WidgetError(String),
}

pub type NodeGraphWidgetResult<T> = Result<T, NodeGraphWidgetError>;

impl NodeGraphWidget {
    pub fn new() -> Self {
        Self {
            id: GraphWidgetId(Uuid::new_v4()),
            nodes: HashMap::new(),
            ports: HashMap::new(),
            connections: Vec::new(),
            viewport: GraphViewport {
                offset: EguiVec2::ZERO,
                zoom: 1.0,
                min_zoom: 0.1,
                max_zoom: 5.0,
                pan_speed: 1.0,
                zoom_speed: 0.1,
                is_panning: false,
                pan_start: None,
                bounds: Rect::EVERYTHING,
            },
            grid: GraphGrid {
                enabled: true,
                size: 20.0,
                color: Color32::from_gray(60),
                subdivisions: 4,
                subdivision_color: Color32::from_gray(40),
                snap_enabled: true,
                snap_strength: 0.5,
            },
            selection: GraphSelection {
                selected_nodes: HashSet::new(),
                selected_connections: HashSet::new(),
                selection_rect: None,
                selection_start: None,
                is_selecting: false,
                selection_mode: SelectionMode::Replace,
            },
            clipboard: GraphClipboard {
                copied_nodes: Vec::new(),
                copied_connections: Vec::new(),
                offset: EguiVec2::new(50.0, 50.0),
            },
            interaction_state: InteractionState::Idle,
            visual_settings: VisualSettings {
                node_opacity: 0.95,
                connection_opacity: 0.8,
                grid_opacity: 0.3,
                selection_opacity: 0.2,
                animation_speed: 1.0,
                pulse_speed: 2.0,
                glow_intensity: 0.3,
                antialiasing: true,
                high_dpi_support: true,
            },
            performance_stats: PerformanceStats {
                frame_time_ms: 0.0,
                node_count: 0,
                connection_count: 0,
                render_time_ms: 0.0,
                interaction_time_ms: 0.0,
                memory_usage_mb: 0.0,
            },
        }
    }

    pub fn add_node(&mut self, node_id: NodeId, position: Pos2) -> NodeGraphWidgetResult<()> {
        let node_widget = NodeWidget {
            id: node_id,
            position,
            size: EguiVec2::new(200.0, 150.0),
            title: format!("Node {:?}", node_id.0),
            color: Color32::from_rgb(70, 70, 70),
            border_color: Color32::from_rgb(100, 100, 100),
            header_height: 30.0,
            corner_radius: 5.0,
            shadow_offset: EguiVec2::new(2.0, 2.0),
            shadow_blur: 8.0,
            shadow_color: Color32::from_rgba_unmultiplied(0, 0, 0, 100),
            is_selected: false,
            is_hovered: false,
            is_dragging: false,
            is_collapsed: false,
            opacity: self.visual_settings.node_opacity,
            animation_state: AnimationState::Idle,
        };

        self.nodes.insert(node_id, node_widget);
        self.update_performance_stats();
        Ok(())
    }

    pub fn add_port(&mut self, port_id: PortId, node_id: NodeId, port_type: PortType, position: Pos2) -> NodeGraphWidgetResult<()> {
        let port_widget = PortWidget {
            id: port_id,
            node_id,
            position,
            radius: 6.0,
            color: self.get_port_color(&port_type),
            border_color: Color32::from_rgb(50, 50, 50),
            port_type,
            is_connected: false,
            is_hovered: false,
            is_active: false,
            animation_offset: 0.0,
            pulse_phase: 0.0,
        };

        self.ports.insert(port_id, port_widget);
        Ok(())
    }

    pub fn add_connection(&mut self, from_port: PortId, to_port: PortId) -> NodeGraphWidgetResult<()> {
        let from_pos = self.get_port_position(from_port)?;
        let to_pos = self.get_port_position(to_port)?;

        let connection = ConnectionWidget {
            from_port,
            to_port,
            from_pos,
            to_pos,
            color: Color32::from_rgb(150, 150, 150),
            thickness: 2.0,
            is_selected: false,
            is_hovered: false,
            animation_offset: 0.0,
            flow_direction: ConnectionFlow::None,
        };

        self.connections.push(connection);
        self.update_performance_stats();
        Ok(())
    }

    pub fn update(&mut self, ui: &mut Ui) -> Response {
        let start_time = std::time::Instant::now();
        
        let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());
        
        self.handle_interaction(&response);
        self.update_animations();
        self.render(ui, &response);
        
        self.performance_stats.frame_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
        response
    }

    fn handle_interaction(&mut self, response: &Response) {
        if response.clicked() {
            self.handle_click(response.interact_pointer_pos());
        }

        if response.dragged() {
            self.handle_drag(response.interact_pointer_pos(), response.drag_delta());
        }

        if response.hovered() {
            self.handle_hover(response.interact_pointer_pos());
        }

        if response.double_clicked() {
            self.handle_double_click(response.interact_pointer_pos());
        }
    }

    fn handle_click(&mut self, pos: Option<Pos2>) {
        if let Some(click_pos) = pos {
            let graph_pos = self.screen_to_graph(click_pos);
            
            if let Some(node_id) = self.find_node_at_position(graph_pos) {
                self.select_node(node_id, SelectionMode::Toggle);
            } else if let Some((from_port, to_port)) = self.find_connection_at_position(graph_pos) {
                self.select_connection(from_port, to_port);
            } else {
                self.clear_selection();
            }
        }
    }

    fn handle_drag(&mut self, pos: Option<Pos2>, delta: EguiVec2) {
        match &self.interaction_state {
            InteractionState::DraggingNode(node_id, start_pos) => {
                if let Some(current_pos) = pos {
                    let graph_delta = self.screen_to_graph_delta(delta);
                    self.move_node(*node_id, graph_delta);
                }
            }
            InteractionState::Panning(_) => {
                let graph_delta = self.screen_to_graph_delta(delta);
                self.pan_viewport(graph_delta);
            }
            InteractionState::Connecting(from_port, start_pos) => {
                // Update connection preview
            }
            _ => {
                if let Some(drag_pos) = pos {
                    let graph_delta = self.screen_to_graph_delta(delta);
                    if graph_delta.length() > 5.0 {
                        self.interaction_state = InteractionState::Panning(graph_delta);
                    }
                }
            }
        }
    }

    fn handle_hover(&mut self, pos: Option<Pos2>) {
        if let Some(hover_pos) = pos {
            let graph_pos = self.screen_to_graph(hover_pos);
            
            self.update_hover_states(graph_pos);
            
            if let Some(node_id) = self.find_node_at_position(graph_pos) {
                self.nodes.get_mut(&node_id).unwrap().is_hovered = true;
            }
        }
    }

    fn handle_double_click(&mut self, pos: Option<Pos2>) {
        if let Some(click_pos) = pos {
            let graph_pos = self.screen_to_graph(click_pos);
            
            if let Some(node_id) = self.find_node_at_position(graph_pos) {
                self.toggle_node_collapse(node_id);
            }
        }
    }

    fn update_animations(&mut self) {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        for node in self.nodes.values_mut() {
            match &mut node.animation_state {
                AnimationState::Hovering(phase) => {
                    *phase = (*phase + 0.1 * self.visual_settings.animation_speed).min(1.0);
                }
                AnimationState::Dragging(phase) => {
                    *phase = (*phase + 0.15 * self.visual_settings.animation_speed).min(1.0);
                }
                AnimationState::Selecting(phase) => {
                    *phase = (*phase + 0.2 * self.visual_settings.animation_speed).min(1.0);
                }
                _ => {}
            }
        }

        for port in self.ports.values_mut() {
            port.pulse_phase = (port.pulse_phase + 0.05 * self.visual_settings.pulse_speed) % (2.0 * std::f32::consts::PI);
        }

        for connection in &mut self.connections {
            connection.animation_offset = (connection.animation_offset + 0.02) % 1.0;
        }
    }

    fn render(&self, ui: &mut Ui, response: &Response) {
        let painter = ui.painter();
        let clip_rect = response.rect;
        
        painter.set_clip_rect(clip_rect);

        if self.grid.enabled {
            self.render_grid(painter, clip_rect);
        }

        self.render_connections(painter);
        self.render_nodes(painter);
        self.render_ports(painter);
        
        if self.selection.is_selecting {
            self.render_selection_rect(painter);
        }

        self.render_ui_overlay(ui);
    }

    fn render_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = self.grid.size * self.viewport.zoom;
        let subdiv_size = grid_size / self.grid.subdivisions as f32;
        
        let min_x = rect.min.x - (rect.min.x % grid_size);
        let min_y = rect.min.y - (rect.min.y % grid_size);
        
        let mut x = min_x;
        while x < rect.max.x {
            painter.line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                (1.0, self.grid.color),
            );
            x += grid_size;
        }
        
        let mut y = min_y;
        while y < rect.max.y {
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                (1.0, self.grid.color),
            );
            y += grid_size;
        }
    }

    fn render_nodes(&self, painter: &egui::Painter) {
        for node in self.nodes.values() {
            let screen_pos = self.graph_to_screen(node.position);
            let rect = Rect::from_min_size(screen_pos, node.size);
            
            let mut node_color = node.color;
            node_color[3] = (node.opacity * 255.0) as u8;
            
            painter.rect(
                rect,
                egui::Rounding::same(node.corner_radius),
                node_color,
                egui::Stroke::new(1.0, node.border_color),
            );
            
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &node.title,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
        }
    }

    fn render_connections(&self, painter: &egui::Painter) {
        for connection in &self.connections {
            let mut color = connection.color;
            color[3] = (self.visual_settings.connection_opacity * 255.0) as u8;
            
            painter.line_segment(
                [connection.from_pos, connection.to_pos],
                (connection.thickness, color),
            );
        }
    }

    fn render_ports(&self, painter: &egui::Painter) {
        for port in self.ports.values() {
            let screen_pos = self.graph_to_screen(port.position);
            
            let mut port_color = port.color;
            let pulse_intensity = (port.pulse_phase.sin() + 1.0) * 0.5;
            let glow_intensity = self.visual_settings.glow_intensity * pulse_intensity;
            
            painter.circle(
                screen_pos,
                port.radius + glow_intensity,
                port_color,
                egui::Stroke::new(1.0, port.border_color),
            );
        }
    }

    fn render_selection_rect(&self, painter: &egui::Painter) {
        if let Some(rect) = self.selection.selection_rect {
            painter.rect(
                rect,
                egui::Rounding::same(0.0),
                Color32::from_rgba_unmultiplied(100, 100, 255, (self.visual_settings.selection_opacity * 255.0) as u8),
                egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 255)),
            );
        }
    }

    fn render_ui_overlay(&self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.label(format!("Nodes: {}", self.performance_stats.node_count));
            ui.label(format!("Connections: {}", self.performance_stats.connection_count));
            ui.label(format!("Zoom: {:.1}x", self.viewport.zoom));
            ui.label(format!("Frame: {:.1}ms", self.performance_stats.frame_time_ms));
        });
    }

    fn screen_to_graph(&self, screen_pos: Pos2) -> Pos2 {
        Pos2::new(
            (screen_pos.x - self.viewport.offset.x) / self.viewport.zoom,
            (screen_pos.y - self.viewport.offset.y) / self.viewport.zoom,
        )
    }

    fn graph_to_screen(&self, graph_pos: Pos2) -> Pos2 {
        Pos2::new(
            graph_pos.x * self.viewport.zoom + self.viewport.offset.x,
            graph_pos.y * self.viewport.zoom + self.viewport.offset.y,
        )
    }

    fn screen_to_graph_delta(&self, delta: EguiVec2) -> EguiVec2 {
        EguiVec2::new(delta.x / self.viewport.zoom, delta.y / self.viewport.zoom)
    }

    fn find_node_at_position(&self, pos: Pos2) -> Option<NodeId> {
        for (node_id, node) in &self.nodes {
            let node_rect = Rect::from_min_size(node.position, node.size);
            if node_rect.contains(pos) {
                return Some(*node_id);
            }
        }
        None
    }

    fn find_connection_at_position(&self, pos: Pos2) -> Option<(PortId, PortId)> {
        for connection in &self.connections {
            let line_start = connection.from_pos;
            let line_end = connection.to_pos;
            
            let distance = self.point_to_line_distance(pos, line_start, line_end);
            if distance < 5.0 {
                return Some((connection.from_port, connection.to_port));
            }
        }
        None
    }

    fn find_port_at_position(&self, pos: Pos2) -> Option<PortId> {
        for (port_id, port) in &self.ports {
            let distance = (port.position - pos).length();
            if distance < port.radius + 2.0 {
                return Some(*port_id);
            }
        }
        None
    }

    fn point_to_line_distance(&self, point: Pos2, line_start: Pos2, line_end: Pos2) -> f32 {
        let line_vec = line_end - line_start;
        let point_vec = point - line_start;
        
        let line_len_sq = line_vec.length_sq();
        if line_len_sq == 0.0 {
            return point_vec.length();
        }
        
        let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
        let projection = line_start + line_vec * t;
        (point - projection).length()
    }

    fn select_node(&mut self, node_id: NodeId, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.selection.selected_nodes.clear();
                self.selection.selected_nodes.insert(node_id);
            }
            SelectionMode::Add => {
                self.selection.selected_nodes.insert(node_id);
            }
            SelectionMode::Remove => {
                self.selection.selected_nodes.remove(&node_id);
            }
            SelectionMode::Toggle => {
                if self.selection.selected_nodes.contains(&node_id) {
                    self.selection.selected_nodes.remove(&node_id);
                } else {
                    self.selection.selected_nodes.insert(node_id);
                }
            }
        }
    }

    fn select_connection(&mut self, from_port: PortId, to_port: PortId) {
        self.selection.selected_connections.insert((from_port, to_port));
    }

    fn clear_selection(&mut self) {
        self.selection.selected_nodes.clear();
        self.selection.selected_connections.clear();
    }

    fn move_node(&mut self, node_id: NodeId, delta: EguiVec2) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.position.x += delta.x;
            node.position.y += delta.y;
            
            if self.grid.snap_enabled {
                node.position.x = (node.position.x / self.grid.size).round() * self.grid.size;
                node.position.y = (node.position.y / self.grid.size).round() * self.grid.size;
            }
        }
    }

    fn pan_viewport(&mut self, delta: EguiVec2) {
        self.viewport.offset += delta;
    }

    fn toggle_node_collapse(&mut self, node_id: NodeId) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.is_collapsed = !node.is_collapsed;
        }
    }

    fn update_hover_states(&mut self, pos: Pos2) {
        for node in self.nodes.values_mut() {
            node.is_hovered = false;
        }
        
        for port in self.ports.values_mut() {
            port.is_hovered = false;
        }
        
        for connection in &mut self.connections {
            connection.is_hovered = false;
        }
    }

    fn get_port_position(&self, port_id: PortId) -> NodeGraphWidgetResult<Pos2> {
        self.ports.get(&port_id)
            .map(|port| port.position)
            .ok_or(NodeGraphWidgetError::PortNotFound(port_id))
    }

    fn get_port_color(&self, port_type: &PortType) -> Color32 {
        match port_type {
            PortType::Float => Color32::from_rgb(100, 150, 200),
            PortType::Float2 => Color32::from_rgb(150, 100, 200),
            PortType::Float3 => Color32::from_rgb(200, 100, 150),
            PortType::Float4 => Color32::from_rgb(200, 150, 100),
            PortType::Color => Color32::from_rgb(255, 100, 100),
            PortType::Texture2D => Color32::from_rgb(100, 255, 100),
            _ => Color32::from_rgb(150, 150, 150),
        }
    }

    fn update_performance_stats(&mut self) {
        self.performance_stats.node_count = self.nodes.len();
        self.performance_stats.connection_count = self.connections.len();
    }

    pub fn export_to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn import_from_json(&mut self, json: &str) -> NodeGraphWidgetResult<()> {
        match serde_json::from_str(json) {
            Ok(widget) => {
                *self = widget;
                self.update_performance_stats();
                Ok(())
            }
            Err(e) => Err(NodeGraphWidgetError::WidgetError(format!("Failed to import: {}", e)))
        }
    }
}

impl Widget for &mut NodeGraphWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        self.update(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_graph_widget_creation() {
        let widget = NodeGraphWidget::new();
        assert_eq!(widget.nodes.len(), 0);
        assert_eq!(widget.connections.len(), 0);
        assert_eq!(widget.viewport.zoom, 1.0);
    }

    #[test]
    fn test_add_node() {
        let mut widget = NodeGraphWidget::new();
        let node_id = NodeId(Uuid::new_v4());
        let result = widget.add_node(node_id, Pos2::new(100.0, 100.0));
        assert!(result.is_ok());
        assert_eq!(widget.nodes.len(), 1);
    }

    #[test]
    fn test_add_connection() {
        let mut widget = NodeGraphWidget::new();
        let node_id = NodeId(Uuid::new_v4());
        let port1 = PortId(Uuid::new_v4());
        let port2 = PortId(Uuid::new_v4());
        
        widget.add_node(node_id, Pos2::new(100.0, 100.0)).unwrap();
        widget.add_port(port1, node_id, PortType::Float, Pos2::new(50.0, 50.0)).unwrap();
        widget.add_port(port2, node_id, PortType::Float, Pos2::new(150.0, 150.0)).unwrap();
        
        let result = widget.add_connection(port1, port2);
        assert!(result.is_ok());
        assert_eq!(widget.connections.len(), 1);
    }

    #[test]
    fn test_screen_graph_conversion() {
        let widget = NodeGraphWidget::new();
        let screen_pos = Pos2::new(200.0, 200.0);
        let graph_pos = widget.screen_to_graph(screen_pos);
        let back_to_screen = widget.graph_to_screen(graph_pos);
        
        assert!((back_to_screen.x - screen_pos.x).abs() < 0.01);
        assert!((back_to_screen.y - screen_pos.y).abs() < 0.01);
    }

    #[test]
    fn test_json_export_import() {
        let mut widget = NodeGraphWidget::new();
        let node_id = NodeId(Uuid::new_v4());
        widget.add_node(node_id, Pos2::new(100.0, 100.0)).unwrap();
        
        let json = widget.export_to_json();
        assert!(!json.is_empty());
        
        let mut new_widget = NodeGraphWidget::new();
        let result = new_widget.import_from_json(&json);
        assert!(result.is_ok());
        assert_eq!(new_widget.nodes.len(), 1);
    }
}