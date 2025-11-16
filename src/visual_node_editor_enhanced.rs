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
    // Enhanced features
    show_grid: bool,
    snap_to_grid: bool,
    grid_size: f32,
    node_colors: HashMap<NodeKind, Color32>,
    connection_curves: bool,
    show_node_previews: bool,
    mini_preview_size: f32,
    // UI enhancements
    context_menu_open: bool,
    context_menu_pos: Pos2,
    clipboard_node: Option<(NodeKind, String)>,
    // Animation and effects
    hover_effects: HashMap<NodeId, f32>, // Node hover animation (0.0 to 1.0)
    connection_animation: f32,
    // Performance optimizations
    visible_nodes: Vec<NodeId>,
    culling_enabled: bool,
}

impl VisualNodeEditor {
    pub fn new() -> Self {
        let mut node_colors = HashMap::new();
        node_colors.insert(NodeKind::Time, Color32::from_rgb(255, 100, 100));
        node_colors.insert(NodeKind::UV, Color32::from_rgb(100, 255, 100));
        node_colors.insert(NodeKind::Sin, Color32::from_rgb(100, 100, 255));
        node_colors.insert(NodeKind::Vec2, Color32::from_rgb(255, 255, 100));
        node_colors.insert(NodeKind::Vec3, Color32::from_rgb(255, 100, 255));
        node_colors.insert(NodeKind::Vec4, Color32::from_rgb(100, 255, 255));
        node_colors.insert(NodeKind::Output, Color32::from_rgb(200, 200, 200));
        node_colors.insert(NodeKind::Multiply, Color32::from_rgb(150, 150, 255));
        node_colors.insert(NodeKind::Add, Color32::from_rgb(255, 150, 150));
        node_colors.insert(NodeKind::Subtract, Color32::from_rgb(150, 255, 150));
        node_colors.insert(NodeKind::Texture2d, Color32::from_rgb(255, 200, 100));
        
        Self {
            node_positions: HashMap::new(),
            connection_start: None,
            selected_node: None,
            pan: Vec2::ZERO,
            zoom: 1.0,
            auto_compile: true,
            last_generated_wgsl: None,
            compilation_errors: Vec::new(),
            // Enhanced features
            show_grid: true,
            snap_to_grid: true,
            grid_size: 20.0,
            node_colors,
            connection_curves: true,
            show_node_previews: true,
            mini_preview_size: 60.0,
            // UI enhancements
            context_menu_open: false,
            context_menu_pos: Pos2::ZERO,
            clipboard_node: None,
            // Animation and effects
            hover_effects: HashMap::new(),
            connection_animation: 0.0,
            // Performance optimizations
            visible_nodes: Vec::new(),
            culling_enabled: true,
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
        // Top toolbar with enhanced controls
        ui.horizontal(|ui| {
            ui.label(RichText::new("🎛️ Visual Node Editor").size(16.0).strong());
            ui.separator();
            
            // Compilation controls
            ui.checkbox(&mut self.auto_compile, "Auto Compile");
            
            if ui.button("🔨 Compile Now").clicked() {
                match self.generate_and_compile(node_graph, 512, 512) {
                    Ok(wgsl) => {
                        ui.label(RichText::new(format!("✅ Compiled successfully ({} chars)", wgsl.len())).color(Color32::GREEN));
                    }
                    Err(errors) => {
                        ui.label(RichText::new(format!("❌ {} errors", errors.len())).color(Color32::RED));
                        for error in &errors {
                            ui.label(RichText::new(format!("  • {}", error)).color(Color32::RED).small());
                        }
                    }
                }
            }
            
            ui.separator();
            
            // View controls
            ui.checkbox(&mut self.show_grid, "📐 Grid");
            ui.checkbox(&mut self.snap_to_grid, "🔲 Snap");
            ui.checkbox(&mut self.connection_curves, "〰️ Curves");
            ui.checkbox(&mut self.show_node_previews, "👁️ Previews");
            
            ui.separator();
            
            // Status indicator
            if !self.compilation_errors.is_empty() {
                ui.label(RichText::new(format!("❌ {} errors", self.compilation_errors.len())).color(Color32::RED).strong());
            } else if self.last_generated_wgsl.is_some() {
                ui.label(RichText::new("✅ Ready").color(Color32::GREEN).strong());
            }
        });

        ui.separator();

        // Node library panel
        ui.horizontal(|ui| {
            ui.label(RichText::new("📚 Node Library:").strong());
            
            let node_types = vec![
                (NodeKind::Time, "⏰ Time", "Time-based animation"),
                (NodeKind::UV, "📐 UV", "Texture coordinates"),
                (NodeKind::Sin, "📈 Sin", "Sine wave function"),
                (NodeKind::Vec2, "🎯 Vec2", "2D vector constructor"),
                (NodeKind::Vec3, "🎯 Vec3", "3D vector constructor"),
                (NodeKind::Vec4, "🎯 Vec4", "4D vector constructor"),
                (NodeKind::Multiply, "✖️ Multiply", "Multiply values"),
                (NodeKind::Add, "➕ Add", "Add values"),
                (NodeKind::Subtract, "➖ Subtract", "Subtract values"),
                (NodeKind::Texture2d, "🖼️ Texture", "2D texture sample"),
                (NodeKind::Output, "🎯 Output", "Final output"),
            ];
            
            for (node_kind, emoji_name, tooltip) in node_types {
                let button = ui.button(emoji_name);
                if !tooltip.is_empty() {
                    button.on_hover_text(tooltip);
                }
                if button.clicked() {
                    let pos = self.get_next_node_position();
                    let id = node_graph.add_node(node_kind, &emoji_name.replace("⏰ ", ""), pos);
                    self.node_positions.insert(id, pos);
                }
            }
        });

        ui.separator();

        // Node editor canvas with enhanced rendering
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Handle pan and zoom with smooth animations
        if response.dragged() {
            self.pan += response.drag_delta();
        }
        
        if response.hovered() {
            let zoom_delta = ui.input().scroll_delta.y * 0.01;
            if zoom_delta.abs() > 0.001 {
                let old_zoom = self.zoom;
                self.zoom = (self.zoom + zoom_delta).clamp(0.1, 5.0);
                
                // Zoom towards mouse cursor
                if let Some(mouse_pos) = ui.input().pointer.latest_pos() {
                    let zoom_factor = self.zoom / old_zoom;
                    let mouse_in_canvas = mouse_pos - response.rect.min.to_vec2();
                    self.pan = (self.pan - mouse_in_canvas) * zoom_factor + mouse_in_canvas;
                }
            }
        }

        // Context menu
        if response.secondary_clicked() {
            self.context_menu_open = true;
            self.context_menu_pos = ui.input().pointer.latest_pos().unwrap_or(Pos2::ZERO);
        }

        // Draw enhanced grid
        if self.show_grid {
            self.draw_enhanced_grid(ui, available_rect);
        }

        // Update animations
        self.update_animations(ui);

        // Draw connections with enhanced visuals
        self.draw_enhanced_connections(ui, node_graph);

        // Draw nodes with enhanced visuals and culling
        self.draw_enhanced_nodes(ui, node_graph, available_rect);

        // Draw connection being created
        if let Some((start_node, start_port, is_output)) = self.connection_start {
            self.draw_enhanced_active_connection(ui, node_graph, start_node, start_port, is_output);
        }

        // Draw context menu
        if self.context_menu_open {
            self.draw_context_menu(ui);
        }
    }

    fn get_next_node_position(&self) -> (f32, f32) {
        // Simple auto-layout: place new nodes in a grid pattern
        let node_count = self.node_positions.len();
        let grid_x = (node_count % 4) as f32 * 200.0 + 100.0;
        let grid_y = (node_count / 4) as f32 * 150.0 + 100.0;
        (grid_x, grid_y)
    }

    fn update_animations(&mut self, ui: &Ui) {
        // Update hover effects
        let dt = ui.input().stable_dt;
        for (_, effect) in self.hover_effects.iter_mut() {
            *effect = (*effect - dt * 3.0).max(0.0);
        }
        
        // Update connection animation
        self.connection_animation = (self.connection_animation + dt * 2.0) % 1.0;
    }

    fn draw_enhanced_grid(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        let grid_size = self.grid_size * self.zoom;
        
        if grid_size < 2.0 {
            return; // Grid too dense
        }
        
        // Dynamic grid alpha based on zoom level
        let grid_alpha = (self.zoom * 0.3).clamp(0.05, 0.4);
        let major_grid_color = Color32::from_rgba_premultiplied(60, 60, 80, (grid_alpha * 255.0) as u8);
        let minor_grid_color = Color32::from_rgba_premultiplied(40, 40, 60, (grid_alpha * 128.0) as u8);
        
        // Calculate grid bounds
        let start_x = ((rect.min.x - self.pan.x) / grid_size).floor() * grid_size + self.pan.x;
        let start_y = ((rect.min.y - self.pan.y) / grid_size).floor() * grid_size + self.pan.y;
        
        // Draw minor grid lines
        let mut x = start_x;
        let mut y = start_y;
        
        while x < rect.max.x {
            painter.line_segment(
                [pos2(x, rect.min.y), pos2(x, rect.max.y)],
                Stroke::new(0.5, minor_grid_color)
            );
            x += grid_size;
        }
        
        while y < rect.max.y {
            painter.line_segment(
                [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                Stroke::new(0.5, minor_grid_color)
            );
            y += grid_size;
        }
        
        // Draw major grid lines every 5 units
        let major_grid_size = grid_size * 5.0;
        if major_grid_size >= 10.0 {
            let start_major_x = ((rect.min.x - self.pan.x) / major_grid_size).floor() * major_grid_size + self.pan.x;
            let start_major_y = ((rect.min.y - self.pan.y) / major_grid_size).floor() * major_grid_size + self.pan.y;
            
            let mut major_x = start_major_x;
            let mut major_y = start_major_y;
            
            while major_x < rect.max.x {
                painter.line_segment(
                    [pos2(major_x, rect.min.y), pos2(major_x, rect.max.y)],
                    Stroke::new(1.0, major_grid_color)
                );
                major_x += major_grid_size;
            }
            
            while major_y < rect.max.y {
                painter.line_segment(
                    [pos2(rect.min.x, major_y), pos2(rect.max.x, major_y)],
                    Stroke::new(1.0, major_grid_color)
                );
                major_y += major_grid_size;
            }
        }
        
        // Draw origin axes
        let origin_x = self.pan.x;
        let origin_y = self.pan.y;
        
        if origin_x >= rect.min.x && origin_x <= rect.max.x {
            painter.line_segment(
                [pos2(origin_x, rect.min.y), pos2(origin_x, rect.max.y)],
                Stroke::new(2.0, Color32::from_rgb(100, 100, 140))
            );
        }
        
        if origin_y >= rect.min.y && origin_y <= rect.max.y {
            painter.line_segment(
                [pos2(rect.min.x, origin_y), pos2(rect.max.x, origin_y)],
                Stroke::new(2.0, Color32::from_rgb(100, 100, 140))
            );
        }
    }

    fn draw_enhanced_nodes(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph, viewport_rect: Rect) {
        let node_ids: Vec<NodeId> = node_graph.nodes.keys().copied().collect();
        
        // Frustum culling for performance
        self.visible_nodes.clear();
        if self.culling_enabled {
            for node_id in node_ids {
                if let Some(pos) = self.node_positions.get(&node_id) {
                    let world_pos = vec2(pos.0, pos.1) + self.pan;
                    let node_rect = Rect::from_min_size(pos2(world_pos.x, world_pos.y), vec2(120.0, 60.0));
                    
                    if viewport_rect.intersects(node_rect) {
                        self.visible_nodes.push(node_id);
                    }
                }
            }
        } else {
            self.visible_nodes = node_ids;
        }
        
        // Draw nodes
        for node_id in self.visible_nodes.clone() {
            self.draw_enhanced_node(ui, node_id, node_graph);
        }
    }

    fn draw_enhanced_node(&mut self, ui: &mut Ui, node_id: NodeId, node_graph: &mut NodeGraph) {
        let node = &node_graph.nodes[&node_id];
        let pos = self.node_positions.get(&node_id).copied().unwrap_or((100.0, 100.0));
        let mut world_pos = vec2(pos.0, pos.1) + self.pan;
        
        // Snap to grid if enabled
        if self.snap_to_grid {
            world_pos.x = (world_pos.x / self.grid_size).round() * self.grid_size;
            world_pos.y = (world_pos.y / self.grid_size).round() * self.grid_size;
        }
        
        let node_width = 140.0;
        let node_height = 80.0 + (node.inputs.len() + node.outputs.len()) as f32 * 25.0;
        let node_rect = Rect::from_min_size(pos2(world_pos.x, world_pos.y), vec2(node_width, node_height));
        
        let painter = ui.painter();
        
        // Enhanced node background with hover effects
        let is_selected = self.selected_node == Some(node_id);
        let is_hovered = ui.input().pointer.latest_pos()
            .map(|mouse_pos| node_rect.contains(mouse_pos))
            .unwrap_or(false);
        
        // Update hover effect
        let hover_effect = self.hover_effects.entry(node_id).or_insert(0.0);
        if is_hovered {
            *hover_effect = (1.0 - *hover_effect) * 0.1 + *hover_effect;
        } else {
            *hover_effect = *hover_effect * 0.95;
        }
        
        let base_color = self.node_colors.get(&node.kind).copied().unwrap_or(Color32::from_rgb(80, 80, 100));
        let hover_intensity = *hover_effect;
        
        // Create gradient effect
        let bg_color = if is_selected {
            Color32::from_rgb(
                (base_color.r() as f32 * 1.3).min(255.0) as u8,
                (base_color.g() as f32 * 1.3).min(255.0) as u8,
                (base_color.b() as f32 * 1.3).min(255.0) as u8
            )
        } else {
            Color32::from_rgb(
                (base_color.r() as f32 + hover_intensity * 30.0).min(255.0) as u8,
                (base_color.g() as f32 + hover_intensity * 30.0).min(255.0) as u8,
                (base_color.b() as f32 + hover_intensity * 30.0).min(255.0) as u8
            )
        };
        
        // Draw node shadow
        let shadow_rect = node_rect.expand(2.0).translate(vec2(2.0, 2.0));
        painter.rect(shadow_rect, 6.0, Color32::from_rgba_premultiplied(0, 0, 0, 30), Stroke::NONE);
        
        // Draw main node body
        painter.rect(node_rect, 6.0, bg_color, Stroke::new(2.0, Color32::from_rgb(200, 200, 220)));
        
        // Draw node header with enhanced styling
        let header_rect = Rect::from_min_size(node_rect.min, vec2(node_width, 25.0));
        painter.rect(header_rect, 6.0, Color32::from_rgba_premultiplied(0, 0, 0, 50), Stroke::NONE);
        
        // Node title with enhanced typography
        let title_pos = node_rect.min + vec2(10.0, 5.0);
        let node_title = format!("{} {}", self.get_node_emoji(&node.kind), node.name);
        painter.text(title_pos, Align2::LEFT_TOP, node_title, FontId::proportional(14.0), Color32::WHITE);
        
        // Node type indicator
        let type_indicator_pos = node_rect.min + vec2(node_width - 20.0, 5.0);
        painter.circle(type_indicator_pos, 4.0, base_color, Stroke::new(1.0, Color32::WHITE));
        
        // Input ports with enhanced visuals
        for (i, input) in node.inputs.iter().enumerate() {
            let port_pos = node_rect.min + vec2(0.0, 35.0 + i as f32 * 25.0);
            self.draw_enhanced_port(ui, port_pos, false, &input.name, node_id, input.id, node_graph, node_rect);
        }
        
        // Output ports with enhanced visuals
        for (i, output) in node.outputs.iter().enumerate() {
            let port_pos = node_rect.min + vec2(node_width, 35.0 + i as f32 * 25.0);
            self.draw_enhanced_port(ui, port_pos, true, &output.name, node_id, output.id, node_graph, node_rect);
        }
        
        // Mini preview if enabled
        if self.show_node_previews && node.kind != NodeKind::Output {
            let preview_pos = node_rect.min + vec2(node_width - 70.0, node_height - 35.0);
            let preview_rect = Rect::from_min_size(preview_pos, vec2(self.mini_preview_size, self.mini_preview_size * 0.6));
            painter.rect(preview_rect, 3.0, Color32::from_rgb(20, 20, 30), Stroke::new(1.0, Color32::from_rgb(100, 100, 120)));
            painter.text(preview_rect.center(), Align2::CENTER_CENTER, "👁️", FontId::proportional(16.0), Color32::from_rgb(120, 120, 140));
        }
        
        // Node interaction
        let node_response = ui.allocate_rect(node_rect, Sense::click_and_drag());
        
        if node_response.clicked() {
            self.selected_node = Some(node_id);
        }
        
        if node_response.dragged() {
            let drag_delta = node_response.drag_delta();
            let new_pos = (pos.0 + drag_delta.x, pos.1 + drag_delta.y);
            self.node_positions.insert(node_id, new_pos);
        }
        
        // Context menu on right click
        if node_response.secondary_clicked() {
            self.context_menu_open = true;
            self.context_menu_pos = ui.input().pointer.latest_pos().unwrap_or(Pos2::ZERO);
            self.selected_node = Some(node_id);
        }
    }

    fn draw_enhanced_port(&mut self, ui: &mut Ui, pos: Pos2, is_output: bool, name: &str, node_id: NodeId, port_id: PortId, node_graph: &mut NodeGraph, node_rect: Rect) {
        let painter = ui.painter();
        let port_radius = 8.0;
        
        // Port colors with type indication
        let port_color = if is_output {
            Color32::from_rgb(100, 200, 255)
        } else {
            Color32::from_rgb(255, 100, 200)
        };
        
        // Port glow effect
        let glow_radius = port_radius + 2.0;
        painter.circle(pos, glow_radius, Color32::from_rgba_premultiplied(
            port_color.r(), port_color.g(), port_color.b(), 50
        ), Stroke::NONE);
        
        // Main port circle
        painter.circle(pos, port_radius, port_color, Stroke::new(2.0, Color32::WHITE));
        
        // Connection indicator
        let is_connected = node_graph.connections.iter().any(|conn| {
            (conn.from_node == node_id && conn.from_port == port_id) ||
            (conn.to_node == node_id && conn.to_port == port_id)
        });
        
        if is_connected {
            painter.circle(pos, port_radius - 2.0, Color32::WHITE, Stroke::NONE);
        }
        
        // Port label with enhanced typography
        let label_pos = if is_output {
            pos - vec2(15.0 + name.len() as f32 * 8.0, 0.0)
        } else {
            pos + vec2(15.0, 0.0)
        };
        painter.text(label_pos, Align2::CENTER_CENTER, name, FontId::proportional(11.0), Color32::WHITE);
        
        // Port interaction area
        let port_rect = Rect::from_center_size(pos, vec2(port_radius * 2.5, port_radius * 2.5));
        let port_response = ui.allocate_rect(port_rect, Sense::click());
        
        // Hover effect
        if port_response.hovered() {
            painter.circle(pos, port_radius + 2.0, Color32::TRANSPARENT, Stroke::new(2.0, Color32::YELLOW));
        }
        
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

    fn draw_enhanced_connections(&self, ui: &mut Ui, node_graph: &NodeGraph) {
        let painter = ui.painter();
        
        for (i, connection) in node_graph.connections.iter().enumerate() {
            let from_pos = self.get_port_position(ui, node_graph, connection.from_node, connection.from_port, true);
            let to_pos = self.get_port_position(ui, node_graph, connection.to_node, connection.to_port, false);
            
            if let (Some(from), Some(to)) = (from_pos, to_pos) {
                // Animated connection glow
                let glow_intensity = ((self.connection_animation + i as f32 * 0.1) % 1.0).sin() * 0.3 + 0.7;
                let connection_color = Color32::from_rgba_premultiplied(
                    (100.0 * glow_intensity) as u8,
                    (200.0 * glow_intensity) as u8,
                    (255.0 * glow_intensity) as u8,
                    200
                );
                
                if self.connection_curves {
                    // Enhanced curved connection
                    let control_offset = vec2((to.x - from.x) * 0.5, 0.0);
                    let control1 = from + control_offset;
                    let control2 = to - control_offset;
                    
                    // Draw connection glow
                    painter.add(Shape::CubicBezier(CubicBezierShape {
                        points: [from, control1, control2, to],
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: Stroke::new(6.0, Color32::from_rgba_premultiplied(0, 100, 200, 30)),
                    }));
                    
                    // Draw main connection
                    painter.add(Shape::CubicBezier(CubicBezierShape {
                        points: [from, control1, control2, to],
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: Stroke::new(3.0, connection_color),
                    }));
                } else {
                    // Straight connection with gradient
                    painter.line_segment(
                        [from, to],
                        Stroke::new(3.0, connection_color)
                    );
                }
                
                // Connection direction indicators
                let mid_point = from + (to - from) * 0.5;
                painter.circle(mid_point, 3.0, connection_color, Stroke::new(1.0, Color32::WHITE));
            }
        }
    }

    fn draw_enhanced_active_connection(&self, ui: &mut Ui, node_graph: &NodeGraph, start_node: NodeId, start_port: PortId, is_output: bool) {
        let painter = ui.painter();
        
        if let Some(start_pos) = self.get_port_position(ui, node_graph, start_node, start_port, is_output) {
            let mouse_pos = ui.input().pointer.latest_pos().unwrap_or(start_pos);
            
            // Animated active connection
            let pulse_intensity = (self.connection_animation * 3.0).sin() * 0.5 + 0.5;
            let active_color = Color32::from_rgba_premultiplied(255, 200, 100, (200.0 + pulse_intensity * 55.0) as u8);
            
            if self.connection_curves {
                let control_offset = vec2((mouse_pos.x - start_pos.x) * 0.5, 0.0);
                let control1 = start_pos + control_offset;
                let control2 = mouse_pos - control_offset;
                
                painter.add(Shape::CubicBezier(CubicBezierShape {
                    points: [start_pos, control1, control2, mouse_pos],
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(4.0, active_color),
                }));
            } else {
                painter.line_segment(
                    [start_pos, mouse_pos],
                    Stroke::new(4.0, active_color)
                );
            }
            
            // Pulsing end indicator
            painter.circle(mouse_pos, 5.0 + pulse_intensity * 2.0, active_color, Stroke::new(2.0, Color32::WHITE));
        }
    }

    fn draw_context_menu(&mut self, ui: &mut Ui) {
        if !self.context_menu_open {
            return;
        }
        
        let response = ui.allocate_ui_at_rect(Rect::from_min_size(self.context_menu_pos, vec2(200.0, 300.0)), |ui| {
            ui.set_max_width(200.0);
            Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(200.0);
                
                ui.label(RichText::new("Node Actions").strong());
                ui.separator();
                
                if ui.button("📋 Copy Node").clicked() {
                    if let Some(node_id) = self.selected_node {
                        // Copy node logic would go here
                        self.context_menu_open = false;
                    }
                }
                
                if ui.button("📋 Paste Node").clicked() {
                    // Paste node logic would go here
                    self.context_menu_open = false;
                }
                
                ui.separator();
                
                if ui.button("🗑️ Delete Node").clicked() {
                    if let Some(node_id) = self.selected_node {
                        // Delete node logic would go here
                        self.context_menu_open = false;
                    }
                }
                
                if ui.button("✏️ Rename Node").clicked() {
                    if let Some(node_id) = self.selected_node {
                        // Rename node logic would go here
                        self.context_menu_open = false;
                    }
                }
                
                ui.separator();
                
                if ui.button("🔲 Auto Layout").clicked() {
                    // Auto layout logic would go here
                    self.context_menu_open = false;
                }
                
                if ui.button("🎯 Center View").clicked() {
                    self.pan = Vec2::ZERO;
                    self.zoom = 1.0;
                    self.context_menu_open = false;
                }
            });
        });
        
        // Close context menu if clicked outside
        if ui.input().pointer.any_click() && !response.response.hovered() {
            self.context_menu_open = false;
        }
    }

    fn get_port_position(&self, ui: &Ui, node_graph: &NodeGraph, node_id: NodeId, port_id: PortId, is_output: bool) -> Option<Pos2> {
        let node_pos = self.node_positions.get(&node_id)?;
        let node = node_graph.nodes.get(&node_id)?;
        
        let world_pos = vec2(node_pos.0, node_pos.1) + self.pan;
        let node_width = 140.0;
        let base_y = world_pos.y + 35.0;
        
        if is_output {
            if let Some((i, _)) = node.outputs.iter().enumerate().find(|(_, p)| p.id == port_id) {
                Some(pos2(world_pos.x + node_width, base_y + i as f32 * 25.0))
            } else {
                None
            }
        } else {
            if let Some((i, _)) = node.inputs.iter().enumerate().find(|(_, p)| p.id == port_id) {
                Some(pos2(world_pos.x, base_y + i as f32 * 25.0))
            } else {
                None
            }
        }
    }

    fn get_node_emoji(&self, node_kind: &NodeKind) -> &'static str {
        match node_kind {
            NodeKind::Time => "⏰",
            NodeKind::UV => "📐",
            NodeKind::Sin => "📈",
            NodeKind::Vec2 => "🎯",
            NodeKind::Vec3 => "🎯",
            NodeKind::Vec4 => "🎯",
            NodeKind::Multiply => "✖️",
            NodeKind::Add => "➕",
            NodeKind::Subtract => "➖",
            NodeKind::Texture2d => "🖼️",
            NodeKind::Output => "🎯",
        }
    }
}