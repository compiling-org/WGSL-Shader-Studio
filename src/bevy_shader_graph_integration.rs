use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;
// use uuid::Uuid; // Comment out for now, use simple ID instead

/// Main plugin for shader graph integration
pub struct ShaderGraphIntegrationPlugin;

impl Plugin for ShaderGraphIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShaderGraphResource>()
            .add_systems(Update, (
                update_shader_graph,
                draw_shader_graph_ui,
            ));
    }
}

/// Resource containing the main shader graph
#[derive(Resource, Default)]
pub struct ShaderGraphResource {
    pub graph: ShaderGraph,
    pub selected_node: Option<NodeId>,
    pub connection_start: Option<(NodeId, usize)>, // node_id, output_index
    pub generated_code: String,
    pub show_editor: bool,
}

/// Unique identifier for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Main shader graph structure
#[derive(Debug, Clone)]
pub struct ShaderGraph {
    pub nodes: HashMap<NodeId, ShaderNode>,
    pub connections: Vec<NodeConnection>,
}

impl Default for ShaderGraph {
    fn default() -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
        };
        
        // Add default output node
        let output_id = NodeId::new();
        graph.nodes.insert(output_id, ShaderNode {
            id: output_id,
            node_type: NodeType::FragmentOutput,
            position: egui::pos2(400.0, 300.0),
            inputs: vec![InputPort {
                name: "color".to_string(),
                port_type: PortType::Vec4,
                connected_from: None,
            }],
            outputs: vec![],
        });
        
        graph
    }
}

/// Shader node with inputs and outputs
#[derive(Debug, Clone)]
pub struct ShaderNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub position: egui::Pos2,
    pub inputs: Vec<InputPort>,
    pub outputs: Vec<OutputPort>,
}

/// Input port for connecting nodes
#[derive(Debug, Clone)]
pub struct InputPort {
    pub name: String,
    pub port_type: PortType,
    pub connected_from: Option<(NodeId, usize)>, // (source_node_id, output_index)
}

/// Output port for connecting nodes
#[derive(Debug, Clone)]
pub struct OutputPort {
    pub name: String,
    pub port_type: PortType,
}

/// Connection between nodes
#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub from: (NodeId, usize), // (node_id, output_index)
    pub to: (NodeId, usize),   // (node_id, input_index)
}

/// Available node types for shader graph
#[derive(Debug, Clone)]
pub enum NodeType {
    // Input nodes
    Time,
    Resolution,
    Mouse,
    UV,
    
    // Math nodes
    Sin,
    Cos,
    Multiply,
    Add,
    Subtract,
    Divide,
    
    // Vector nodes
    Vec2,
    Vec3,
    Vec4,
    
    // Color nodes
    Color,
    
    // Output nodes
    FragmentOutput,
}

/// Port types for type-safe connections
#[derive(Debug, Clone, PartialEq)]
pub enum PortType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
}

/// Update the shader graph resource
fn update_shader_graph(
    mut graph_resource: ResMut<ShaderGraphResource>,
) {
    // Generate WGSL code from the current graph
    graph_resource.generated_code = generate_wgsl_code(&graph_resource.graph);
}

/// Draw the shader graph UI
fn draw_shader_graph_ui(
    mut egui_ctx: EguiContexts,
    mut graph_resource: ResMut<ShaderGraphResource>,
) {
    if !graph_resource.show_editor {
        return;
    }
    
    let ctx = egui_ctx.ctx_mut().unwrap();
    
    egui::Window::new("Shader Graph Editor")
        .default_size([800.0, 600.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Add Time Node").clicked() {
                    let node_id = NodeId::new();
                    let node = ShaderNode {
                        id: node_id,
                        node_type: NodeType::Time,
                        position: egui::pos2(100.0, 100.0),
                        inputs: vec![],
                        outputs: vec![OutputPort {
                            name: "time".to_string(),
                            port_type: PortType::Float,
                        }],
                    };
                    graph_resource.graph.nodes.insert(node_id, node);
                }
                
                if ui.button("Add Sin Node").clicked() {
                    let node_id = NodeId::new();
                    let node = ShaderNode {
                        id: node_id,
                        node_type: NodeType::Sin,
                        position: egui::pos2(200.0, 100.0),
                        inputs: vec![InputPort {
                            name: "input".to_string(),
                            port_type: PortType::Float,
                            connected_from: None,
                        }],
                        outputs: vec![OutputPort {
                            name: "result".to_string(),
                            port_type: PortType::Float,
                        }],
                    };
                    graph_resource.graph.nodes.insert(node_id, node);
                }
                
                if ui.button("Add Color Node").clicked() {
                    let node_id = NodeId::new();
                    let node = ShaderNode {
                        id: node_id,
                        node_type: NodeType::Color,
                        position: egui::pos2(300.0, 100.0),
                        inputs: vec![],
                        outputs: vec![OutputPort {
                            name: "color".to_string(),
                            port_type: PortType::Color,
                        }],
                    };
                    graph_resource.graph.nodes.insert(node_id, node);
                }
            });
            
            ui.separator();
            
            // Display generated code
            ui.heading("Generated WGSL Code:");
            ui.monospace(&graph_resource.generated_code);
        });
}

/// Generate WGSL code from the shader graph
fn generate_wgsl_code(graph: &ShaderGraph) -> String {
    let mut code = String::new();
    
    // Find the output node
    let output_node = graph.nodes.values()
        .find(|node| matches!(node.node_type, NodeType::FragmentOutput));
    
    if let Some(output) = output_node {
        code.push_str("@fragment\n");
        code.push_str("fn fragment_main() -> @location(0) vec4<f32> {\n");
        
        // Generate code based on connected inputs
        if let Some(input) = output.inputs.first() {
            if let Some((source_node_id, output_index)) = input.connected_from {
                if let Some(source_node) = graph.nodes.get(&source_node_id) {
                    match source_node.node_type {
                        NodeType::Color => {
                            code.push_str("    return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red color\n");
                        }
                        NodeType::Time => {
                            code.push_str("    let time = 0.0;\n");
                            code.push_str("    return vec4<f32>(sin(time), 0.0, 0.0, 1.0);\n");
                        }
                        _ => {
                            code.push_str("    return vec4<f32>(0.5, 0.5, 0.5, 1.0); // Gray\n");
                        }
                    }
                } else {
                    code.push_str("    return vec4<f32>(0.5, 0.5, 0.5, 1.0); // Gray\n");
                }
            } else {
                code.push_str("    return vec4<f32>(0.5, 0.5, 0.5, 1.0); // Gray\n");
            }
        } else {
            code.push_str("    return vec4<f32>(0.5, 0.5, 0.5, 1.0); // Gray\n");
        }
        
        code.push_str("}\n");
    } else {
        code.push_str("@fragment\n");
        code.push_str("fn fragment_main() -> @location(0) vec4<f32> {\n");
        code.push_str("    return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black\n");
        code.push_str("}\n");
    }
    
    code
}

/// System to toggle shader graph editor
pub fn toggle_shader_graph_editor(
    mut graph_resource: ResMut<ShaderGraphResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyN) {
        graph_resource.show_editor = !graph_resource.show_editor;
    }
}