//! Visual Node Editor Adapter
//! Bridges the gap between the restored visual editor and actual node graph implementation

use bevy_egui::egui::*;
use crate::node_graph::{NodeGraph, NodeId, NodeKind as ActualNodeKind, PortId};
use std::collections::HashMap;

/// Adapter to map between visual editor expectations and actual node graph
pub struct NodeEditorAdapter {
    node_positions: HashMap<NodeId, (f32, f32)>,
    connection_start: Option<(NodeId, PortId, bool)>,
    selected_node: Option<NodeId>,
    pan: Vec2,
    zoom: f32,
    auto_compile: bool,
    last_generated_wgsl: Option<String>,
    compilation_errors: Vec<String>,
}

/// Node types expected by the visual editor (from restored version)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualNodeKind {
    Time,
    UV,
    Sin,
    Vec2,
    Add,
    Multiply,
    Color,
    Output,
}

impl NodeEditorAdapter {
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
    
    /// Map visual node types to actual node graph types
    fn visual_to_actual_kind(visual: VisualNodeKind) -> ActualNodeKind {
        match visual {
            VisualNodeKind::Time => ActualNodeKind::Time,
            VisualNodeKind::UV => ActualNodeKind::UV,
            VisualNodeKind::Sin => ActualNodeKind::Sine,
            VisualNodeKind::Vec2 => ActualNodeKind::ConstantVec2([0.0, 0.0]),
            VisualNodeKind::Add => ActualNodeKind::Add,
            VisualNodeKind::Multiply => ActualNodeKind::Multiply,
            VisualNodeKind::Color => ActualNodeKind::ConstantVec4([1.0, 1.0, 1.0, 1.0]),
            VisualNodeKind::Output => ActualNodeKind::OutputColor,
        }
    }
    
    /// Map actual node types back to visual types for display
    fn actual_to_visual_kind(actual: &ActualNodeKind) -> Option<VisualNodeKind> {
        match actual {
            ActualNodeKind::Time => Some(VisualNodeKind::Time),
            ActualNodeKind::UV => Some(VisualNodeKind::UV),
            ActualNodeKind::Sine => Some(VisualNodeKind::Sin),
            ActualNodeKind::ConstantVec2(_) => Some(VisualNodeKind::Vec2),
            ActualNodeKind::Add => Some(VisualNodeKind::Add),
            ActualNodeKind::Multiply => Some(VisualNodeKind::Multiply),
            ActualNodeKind::ConstantVec4(_) => Some(VisualNodeKind::Color),
            ActualNodeKind::OutputColor => Some(VisualNodeKind::Output),
            _ => None, // Complex nodes that don't have visual equivalents
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
    
    /// Add a node with visual type mapping
    pub fn add_visual_node(&mut self, node_graph: &mut NodeGraph, kind: VisualNodeKind, title: &str, position: (f32, f32)) -> NodeId {
        let actual_kind = Self::visual_to_actual_kind(kind);
        let node_id = node_graph.add_node(actual_kind, title, position);
        self.node_positions.insert(node_id, position);
        node_id
    }
    
    /// Get visual representation of a node
    pub fn get_node_visual_kind(&self, _node_graph: &NodeGraph, _node_id: NodeId) -> Option<VisualNodeKind> {
        // For now, return None as we don't have a get_node method
        // This is a limitation we'll address in the future
        None
    }
    
    pub fn ui(&mut self, ui: &mut Ui, node_graph: &mut NodeGraph) {
        // Execution control panel at the top
        ui.horizontal(|ui| {
            ui.label("Node Editor (Adapted)");
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
                self.add_visual_node(node_graph, VisualNodeKind::Time, "Time", (100.0, 100.0));
            }
            if ui.button("UV").clicked() {
                self.add_visual_node(node_graph, VisualNodeKind::UV, "UV", (200.0, 100.0));
            }
            if ui.button("Sin").clicked() {
                self.add_visual_node(node_graph, VisualNodeKind::Sin, "Sin", (300.0, 100.0));
            }
            if ui.button("Vec2").clicked() {
                self.add_visual_node(node_graph, VisualNodeKind::Vec2, "Vec2", (400.0, 100.0));
            }
            if ui.button("Add").clicked() {
                self.add_visual_node(node_graph, VisualNodeKind::Add, "Add", (500.0, 100.0));
            }
            if ui.button("Multiply").clicked() {
                self.add_visual_node(node_graph, VisualNodeKind::Multiply, "Multiply", (600.0, 100.0));
            }
            if ui.button("Color").clicked() {
                self.add_visual_node(node_graph, VisualNodeKind::Color, "Color", (700.0, 100.0));
            }
            if ui.button("Output").clicked() {
                self.add_visual_node(node_graph, VisualNodeKind::Output, "Output", (800.0, 100.0));
            }
        });
        
        ui.separator();
        
        // Main node editor area
        let response = ui.allocate_response(ui.available_size(), Sense::drag());
        
        // Handle pan and zoom
        if response.dragged() {
            self.pan += response.drag_delta();
        }
        
        // Draw grid
        let painter = ui.painter_at(response.rect);
        let grid_spacing = 50.0 * self.zoom;
        let grid_color = Color32::from_gray(40);
        
        let min_x = response.rect.left() - self.pan.x;
        let max_x = response.rect.right() - self.pan.x;
        let min_y = response.rect.top() - self.pan.y;
        let max_y = response.rect.bottom() - self.pan.y;
        
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let start_y = (min_y / grid_spacing).floor() * grid_spacing;
        
        let mut x = start_x;
        while x < max_x {
            painter.line_segment(
                [pos2(x + self.pan.x, response.rect.top()), pos2(x + self.pan.x, response.rect.bottom())],
                (1.0, grid_color),
            );
            x += grid_spacing;
        }
        
        let mut y = start_y;
        while y < max_y {
            painter.line_segment(
                [pos2(response.rect.left(), y + self.pan.y), pos2(response.rect.right(), y + self.pan.y)],
                (1.0, grid_color),
            );
            y += grid_spacing;
        }
        
        // Draw nodes
        for (node_id, position) in &self.node_positions {
            // For now, draw placeholder nodes since we don't have get_node method
            self.draw_placeholder_node(ui, &painter, *node_id, *position);
        }
        
        // Show generated WGSL if available
        if let Some(wgsl) = &self.last_generated_wgsl {
            ui.collapsing("Generated WGSL", |ui| {
                ui.monospace(wgsl);
            });
        }
        
        // Show compilation errors
        if !self.compilation_errors.is_empty() {
            ui.collapsing("Compilation Errors", |ui| {
                for error in &self.compilation_errors {
                    ui.label(RichText::new(error).color(Color32::RED));
                }
            });
        }
    }
    
    fn draw_placeholder_node(&self, _ui: &Ui, painter: &Painter, _node_id: NodeId, position: (f32, f32)) {
        let node_rect = Rect::from_min_size(
            pos2(position.0, position.1),
            vec2(120.0, 80.0)
        );
        
        // Node background
        painter.rect(
            node_rect,
            5.0,
            Color32::from_gray(60),
            Stroke::new(2.0, Color32::from_gray(120)),
            StrokeKind::Inside
        );
        
        // Node title - placeholder
        painter.text(
            node_rect.center_top() + vec2(0.0, 10.0),
            Align2::CENTER_TOP,
            "Node",
            FontId::proportional(12.0),
            Color32::WHITE
        );
        
        // Node type indicator - placeholder
        let type_text = "🔧"; // Placeholder icon
        
        painter.text(
            node_rect.center(),
            Align2::CENTER_CENTER,
            type_text,
            FontId::proportional(20.0),
            Color32::WHITE
        );
        
        // Input/output ports - simplified placeholders
        painter.circle(pos2(node_rect.left() - 8.0, node_rect.center().y), 6.0, Color32::from_gray(100), Stroke::new(2.0, Color32::WHITE));
        painter.circle(pos2(node_rect.right() + 8.0, node_rect.center().y), 6.0, Color32::from_gray(100), Stroke::new(2.0, Color32::WHITE));
    }
}