use bevy_egui::egui::*;
use crate::node_graph::*;

pub struct VisualNodeEditor {
    pan: Vec2,
    zoom: f32,
}

impl Default for VisualNodeEditor {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
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

        // Draw nodes
        let node_ids: Vec<NodeId> = node_graph.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.draw_node(ui, node_id, node_graph);
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

    fn draw_node(&self, ui: &mut Ui, node_id: NodeId, node_graph: &mut NodeGraph) {
        if let Some(node) = node_graph.nodes.get(&node_id) {
            let node_pos = pos2(node.pos.0, node.pos.1) + self.pan;
            let node_size = vec2(180.0, 120.0);
            let node_rect = Rect::from_min_size(node_pos, node_size);
            
            let painter = ui.painter();
            
            // Node background
            let bg_color = self.get_node_color(&node.kind);
            painter.rect_filled(node_rect, 4.0, bg_color);
            let stroke = Stroke::new(2.0, Color32::from_gray(60));
            painter.rect_stroke(node_rect, 4.0, stroke, egui::epaint::RectShape::default());
            
            // Node title
            let title_pos = node_pos + vec2(10.0, 20.0);
            painter.text(
                title_pos,
                Align2::LEFT_CENTER,
                &node.title,
                FontId::proportional(14.0),
                Color32::WHITE
            );
            
            // Draw input ports
            for (i, input) in node.inputs.iter().enumerate() {
                let port_pos = pos2(node_pos.x - 8.0, node_pos.y + 40.0 + (i as f32 * 25.0));
                self.draw_port(painter, port_pos, input, false);
            }
            
            // Draw output ports
            for (i, output) in node.outputs.iter().enumerate() {
                let port_pos = pos2(node_pos.x + node_size.x + 8.0, node_pos.y + 40.0 + (i as f32 * 25.0));
                self.draw_port(painter, port_pos, output, true);
            }
        }
    }

    fn draw_port(&self, painter: &Painter, pos: Pos2, port: &Port, is_output: bool) {
        let port_color = self.get_port_color(&port.kind);
        let port_size = 8.0;
        
        painter.circle(pos, port_size, port_color, Stroke::new(1.0, Color32::WHITE));
        
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