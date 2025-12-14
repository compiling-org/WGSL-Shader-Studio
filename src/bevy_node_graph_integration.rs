use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;

pub struct BevyNodeGraphPlugin;

impl Plugin for BevyNodeGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodeGraphResource>()
            .add_systems(Update, draw_node_graph_ui);
    }
}

// Node graph types
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderNodeType {
    Time,
    Sin,
    Color,
    Multiply,
    Add,
    Texture,
    UV,
    Constant(f32),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u32,
    pub node_type: ShaderNodeType,
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub position: egui::Pos2,
}

#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub from_node: u32,
    pub from_output: usize,
    pub to_node: u32,
    pub to_input: usize,
}

#[derive(Resource, Default)]
pub struct NodeGraph {
    pub nodes: HashMap<u32, Node>,
    pub connections: Vec<NodeConnection>,
    pub next_node_id: u32,
}

impl NodeGraph {
    pub fn add_node(
        &mut self,
        node_type: ShaderNodeType,
        name: String,
        inputs: Vec<String>,
        outputs: Vec<String>,
    ) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        
        let node = Node {
            id,
            node_type,
            name,
            inputs,
            outputs,
            position: egui::Pos2::new(100.0 + (id as f32 * 50.0), 100.0),
        };
        
        self.nodes.insert(id, node);
        id
    }
    
    pub fn generate_wgsl(&self) -> Result<String, String> {
        // Simple WGSL generation logic
        let mut wgsl = String::new();
        wgsl.push_str("@vertex\n");
        wgsl.push_str("fn vertex_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {\n");
        wgsl.push_str("    return vec4<f32>(position, 1.0);\n");
        wgsl.push_str("}\n\n");
        
        wgsl.push_str("@fragment\n");
        wgsl.push_str("fn fragment_main() -> @location(0) vec4<f32> {\n");
        wgsl.push_str("    return vec4<f32>(0.5, 0.5, 0.5, 1.0);\n");
        wgsl.push_str("}\n");
        
        Ok(wgsl)
    }
}

#[derive(Resource, Default)]
pub struct NodeGraphResource {
    pub graph: NodeGraph,
    pub selected_node: Option<u32>,
}

pub fn draw_node_graph_ui(
    mut contexts: EguiContexts,
    mut node_graph: ResMut<NodeGraphResource>,
) {
    let ctx = contexts.ctx_mut().unwrap();
    
    egui::Window::new("Shader Node Graph")
        .default_size([800.0, 600.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Add Time Node").clicked() {
                    let id = node_graph.graph.add_node(
                        ShaderNodeType::Time,
                        "Time".to_string(),
                        vec![],
                        vec!["time".to_string()]
                    );
                    node_graph.selected_node = Some(id);
                }
                
                if ui.button("Add Sin Node").clicked() {
                    let id = node_graph.graph.add_node(
                        ShaderNodeType::Sin,
                        "Sin".to_string(),
                        vec!["input".to_string()],
                        vec!["result".to_string()]
                    );
                    node_graph.selected_node = Some(id);
                }
                
                if ui.button("Add Color Node").clicked() {
                    let id = node_graph.graph.add_node(
                        ShaderNodeType::Color,
                        "Color".to_string(),
                        vec!["r".to_string(), "g".to_string(), "b".to_string()],
                        vec!["color".to_string()]
                    );
                    node_graph.selected_node = Some(id);
                }
                
                if ui.button("Generate WGSL").clicked() {
                    match node_graph.graph.generate_wgsl() {
                        Ok(wgsl) => {
                            println!("Generated WGSL:\n{}", wgsl);
                            // TODO: Send to shader compiler
                        }
                        Err(e) => {
                            eprintln!("Failed to generate WGSL: {}", e);
                        }
                    }
                }
            });
            
            ui.separator();
            
            // Node graph canvas
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
                    
                    // Draw grid
                    let painter = ui.painter();
                    let rect = response.rect;
                    
                    // Simple grid drawing
                    for x in (rect.left() as i32..rect.right() as i32).step_by(20) {
                        painter.line_segment(
                            [egui::pos2(x as f32, rect.top()), egui::pos2(x as f32, rect.bottom())],
                            (1.0, egui::Color32::from_gray(40))
                        );
                    }
                    
                    for y in (rect.top() as i32..rect.bottom() as i32).step_by(20) {
                        painter.line_segment(
                            [egui::pos2(rect.left(), y as f32), egui::pos2(rect.right(), y as f32)],
                            (1.0, egui::Color32::from_gray(40))
                        );
                    }
                    
                    // Draw nodes
                    for node in node_graph.graph.nodes.values() {
                        let node_rect = egui::Rect::from_min_size(
                            node.position,
                            egui::vec2(150.0, 80.0)
                        );
                        
                        painter.rect_filled(
                            node_rect,
                            egui::Rounding::same(4),
                            egui::Color32::from_gray(60)
                        );
                        painter.rect_stroke(
                            node_rect,
                            egui::Rounding::same(4),
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                            egui::StrokeKind::Outside
                        );
                        
                        painter.text(
                            node_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &node.name,
                            egui::FontId::proportional(12.0),
                            egui::Color32::WHITE
                        );
                    }
                });
        });
}
