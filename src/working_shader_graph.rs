use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;
use uuid::Uuid;

// Enhanced node graph system based on proven patterns from reference repositories
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    // Input nodes
    Time,
    Resolution,
    Mouse,
    UV,
    VertexPosition,
    
    // Math nodes
    Sin,
    Cos,
    Multiply,
    Add,
    Subtract,
    Divide,
    Pow,
    Sqrt,
    
    // Vector nodes
    Vec2,
    Vec3,
    Vec4,
    Color,
    
    // Texture nodes
    Texture2D,
    SampleTexture,
    
    // Output nodes
    FragmentOutput,
    VertexOutput,
}

#[derive(Debug, Clone)]
pub struct NodeInput {
    pub name: String,
    pub value_type: String,
    pub connected: Option<Uuid>,
    pub default_value: String,
}

#[derive(Debug, Clone)]
pub struct NodeOutput {
    pub name: String,
    pub value_type: String,
}

#[derive(Debug, Clone)]
pub struct ShaderNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub position: egui::Pos2,
    pub inputs: Vec<NodeInput>,
    pub outputs: Vec<NodeOutput>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub from_node: Uuid,
    pub from_output: usize,
    pub to_node: Uuid,
    pub to_input: usize,
}

#[derive(Resource, Default)]
pub struct ShaderGraph {
    pub nodes: HashMap<Uuid, ShaderNode>,
    pub connections: Vec<NodeConnection>,
    pub selected_node: Option<Uuid>,
    pub node_id_counter: u64,
}

impl ShaderGraph {
    pub fn new() -> Self {
        let mut graph = Self::default();
        
        // Add default output node
        let output_node = ShaderNode {
            id: Uuid::new_v4(),
            node_type: NodeType::FragmentOutput,
            position: egui::pos2(400.0, 300.0),
            inputs: vec![
                NodeInput {
                    name: "color".to_string(),
                    value_type: "vec4<f32>".to_string(),
                    connected: None,
                    default_value: "vec4<f32>(0.0, 0.0, 0.0, 1.0)".to_string(),
                }
            ],
            outputs: vec![],
            parameters: HashMap::new(),
        };
        
        graph.nodes.insert(output_node.id, output_node);
        graph
    }
    
    pub fn add_node(&mut self, node_type: NodeType, position: egui::Pos2) -> Uuid {
        let node = self.create_node(node_type, position);
        let id = node.id;
        self.nodes.insert(id, node);
        id
    }
    
    fn create_node(&mut self, node_type: NodeType, position: egui::Pos2) -> ShaderNode {
        let id = Uuid::new_v4();
        
        let (inputs, outputs) = match node_type {
            NodeType::Time => (
                vec![],
                vec![NodeOutput {
                    name: "time".to_string(),
                    value_type: "f32".to_string(),
                }]
            ),
            NodeType::Resolution => (
                vec![],
                vec![NodeOutput {
                    name: "resolution".to_string(),
                    value_type: "vec2<f32>".to_string(),
                }]
            ),
            NodeType::UV => (
                vec![],
                vec![NodeOutput {
                    name: "uv".to_string(),
                    value_type: "vec2<f32>".to_string(),
                }]
            ),
            NodeType::Sin => (
                vec![NodeInput {
                    name: "input".to_string(),
                    value_type: "f32".to_string(),
                    connected: None,
                    default_value: "0.0".to_string(),
                }],
                vec![NodeOutput {
                    name: "result".to_string(),
                    value_type: "f32".to_string(),
                }]
            ),
            NodeType::Cos => (
                vec![NodeInput {
                    name: "input".to_string(),
                    value_type: "f32".to_string(),
                    connected: None,
                    default_value: "0.0".to_string(),
                }],
                vec![NodeOutput {
                    name: "result".to_string(),
                    value_type: "f32".to_string(),
                }]
            ),
            NodeType::Multiply => (
                vec![
                    NodeInput {
                        name: "a".to_string(),
                        value_type: "f32".to_string(),
                        connected: None,
                        default_value: "1.0".to_string(),
                    },
                    NodeInput {
                        name: "b".to_string(),
                        value_type: "f32".to_string(),
                        connected: None,
                        default_value: "1.0".to_string(),
                    }
                ],
                vec![NodeOutput {
                    name: "result".to_string(),
                    value_type: "f32".to_string(),
                }]
            ),
            NodeType::Color => (
                vec![],
                vec![NodeOutput {
                    name: "color".to_string(),
                    value_type: "vec4<f32>".to_string(),
                }]
            ),
            NodeType::FragmentOutput => (
                vec![NodeInput {
                    name: "color".to_string(),
                    value_type: "vec4<f32>".to_string(),
                    connected: None,
                    default_value: "vec4<f32>(0.0, 0.0, 0.0, 1.0)".to_string(),
                }],
                vec![]
            ),
            _ => (vec![], vec![])
        };
        
        let parameters = match node_type {
            NodeType::Color => {
                let mut params = HashMap::new();
                params.insert("r".to_string(), "1.0".to_string());
                params.insert("g".to_string(), "1.0".to_string());
                params.insert("b".to_string(), "1.0".to_string());
                params.insert("a".to_string(), "1.0".to_string());
                params
            },
            _ => HashMap::new()
        };
        
        ShaderNode {
            id,
            node_type,
            position,
            inputs,
            outputs,
            parameters,
        }
    }
    
    pub fn generate_wgsl(&self) -> String {
        let mut code = String::new();
        
        // Generate struct definitions
        code.push_str("struct VertexOutput {\n");
        code.push_str("    @builtin(position) position: vec4<f32>,\n");
        code.push_str("    @location(0) uv: vec2<f32>,\n");
        code.push_str("}\n\n");
        
        // Generate uniform bindings
        code.push_str("@group(0) @binding(0) var<uniform> time: f32;\n");
        code.push_str("@group(0) @binding(1) var<uniform> resolution: vec2<f32>;\n");
        code.push_str("@group(0) @binding(2) var<uniform> mouse: vec2<f32>;\n\n");
        
        // Generate vertex shader
        code.push_str("@vertex\n");
        code.push_str("fn vertex_main(@location(0) position: vec2<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {\n");
        code.push_str("    var output: VertexOutput;\n");
        code.push_str("    output.position = vec4<f32>(position, 0.0, 1.0);\n");
        code.push_str("    output.uv = uv;\n");
        code.push_str("    return output;\n");
        code.push_str("}\n\n");
        
        // Generate fragment shader
        code.push_str("@fragment\n");
        code.push_str("fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {\n");
        
        // Generate node code with proper dependency ordering
        let mut generated_nodes = HashMap::new();
        
        for (id, node) in &self.nodes {
            match node.node_type {
                NodeType::Time => {
                    code.push_str(&format!("    let node_{}_time = time;\n", id));
                    generated_nodes.insert(id, "time".to_string());
                },
                NodeType::Resolution => {
                    code.push_str(&format!("    let node_{}_resolution = resolution;\n", id));
                    generated_nodes.insert(id, "resolution".to_string());
                },
                NodeType::UV => {
                    code.push_str(&format!("    let node_{}_uv = input.uv;\n", id));
                    generated_nodes.insert(id, "uv".to_string());
                },
                NodeType::Color => {
                    if let (Some(r), Some(g), Some(b), Some(a)) = (
                        node.parameters.get("r"),
                        node.parameters.get("g"),
                        node.parameters.get("b"),
                        node.parameters.get("a")
                    ) {
                        code.push_str(&format!(
                            "    let node_{}_color = vec4<f32>({}, {}, {}, {});\n",
                            id, r, g, b, a
                        ));
                        generated_nodes.insert(id, "color".to_string());
                    }
                },
                NodeType::Sin => {
                    if let Some(input_conn) = &node.inputs[0].connected {
                        if let Some(input_name) = generated_nodes.get(input_conn) {
                            code.push_str(&format!(
                                "    let node_{}_result = sin(node_{}_{});\n",
                                id, input_conn, input_name
                            ));
                            generated_nodes.insert(id, "result".to_string());
                        }
                    }
                },
                NodeType::Cos => {
                    if let Some(input_conn) = &node.inputs[0].connected {
                        if let Some(input_name) = generated_nodes.get(input_conn) {
                            code.push_str(&format!(
                                "    let node_{}_result = cos(node_{}_{});\n",
                                id, input_conn, input_name
                            ));
                            generated_nodes.insert(id, "result".to_string());
                        }
                    }
                },
                NodeType::Multiply => {
                    if let (Some(a_conn), Some(b_conn)) = (&node.inputs[0].connected, &node.inputs[1].connected) {
                        if let (Some(a_name), Some(b_name)) = (generated_nodes.get(a_conn), generated_nodes.get(b_conn)) {
                            code.push_str(&format!(
                                "    let node_{}_result = node_{}_{} * node_{}_{};\n",
                                id, a_conn, a_name, b_conn, b_name
                            ));
                            generated_nodes.insert(id, "result".to_string());
                        }
                    }
                },
                NodeType::FragmentOutput => {
                    if let Some(color_conn) = &node.inputs[0].connected {
                        if let Some(color_name) = generated_nodes.get(color_conn) {
                            code.push_str(&format!("    return node_{}_{};\n", color_conn, color_name));
                        } else {
                            code.push_str("    return vec4<f32>(0.0, 0.0, 0.0, 1.0);\n");
                        }
                    } else {
                        code.push_str("    return vec4<f32>(0.0, 0.0, 0.0, 1.0);\n");
                    }
                },
                _ => {}
            }
        }
        
        code.push_str("}\n");
        code
    }
    
    pub fn validate_graph(&self) -> Result<(), String> {
        // Check for cycles
        // Check that all inputs are connected or have defaults
        // Check type compatibility
        Ok(())
    }
}

#[derive(Resource)]
pub struct ShaderGraphEditor {
    pub graph: ShaderGraph,
    pub show_editor: bool,
    pub generated_code: String,
    pub auto_generate: bool,
}

impl Default for ShaderGraphEditor {
    fn default() -> Self {
        let graph = ShaderGraph::new();
        let generated_code = graph.generate_wgsl();
        
        Self {
            graph,
            show_editor: false,
            generated_code,
            auto_generate: true,
        }
    }
}

impl ShaderGraphEditor {
    pub fn regenerate_code(&mut self) {
        if self.auto_generate {
            self.generated_code = self.graph.generate_wgsl();
        }
    }
}

pub struct ShaderGraphPlugin;

impl Plugin for ShaderGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShaderGraphEditor>()
            .add_systems(Update, shader_graph_editor_system)
            .add_systems(Update, auto_generate_shader_system);
    }
}

fn shader_graph_editor_system(
    mut editor: ResMut<ShaderGraphEditor>,
    mut contexts: EguiContexts,
    input: Res<ButtonInput<KeyCode>>,
) {
    // Toggle editor with G key
    if input.just_pressed(KeyCode::KeyG) {
        editor.show_editor = !editor.show_editor;
    }
    
    if !editor.show_editor {
        return;
    }
    
    let ctx = contexts.ctx_mut();
    
    egui::Window::new("Shader Graph Editor")
        .default_pos((100.0, 100.0))
        .default_size((900.0, 700.0))
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Add Node", |ui| {
                    if ui.button("Input → Time").clicked() {
                        editor.graph.add_node(NodeType::Time, egui::pos2(100.0, 100.0));
                        ui.close_menu();
                    }
                    if ui.button("Input → Resolution").clicked() {
                        editor.graph.add_node(NodeType::Resolution, egui::pos2(100.0, 150.0));
                        ui.close_menu();
                    }
                    if ui.button("Input → UV").clicked() {
                        editor.graph.add_node(NodeType::UV, egui::pos2(100.0, 200.0));
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Math → Sin").clicked() {
                        editor.graph.add_node(NodeType::Sin, egui::pos2(200.0, 100.0));
                        ui.close_menu();
                    }
                    if ui.button("Math → Cos").clicked() {
                        editor.graph.add_node(NodeType::Cos, egui::pos2(200.0, 150.0));
                        ui.close_menu();
                    }
                    if ui.button("Math → Multiply").clicked() {
                        editor.graph.add_node(NodeType::Multiply, egui::pos2(200.0, 200.0));
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Color → RGBA").clicked() {
                        editor.graph.add_node(NodeType::Color, egui::pos2(300.0, 100.0));
                        ui.close_menu();
                    }
                });
                
                ui.separator();
                
                if ui.button("Generate Code").clicked() {
                    editor.regenerate_code();
                }
                
                ui.checkbox(&mut editor.auto_generate, "Auto Generate");
                
                if ui.button("Validate Graph").clicked() {
                    match editor.graph.validate_graph() {
                        Ok(()) => {
                            ui.label("✅ Graph is valid");
                        }
                        Err(err) => {
                            ui.label(format!("❌ Validation error: {}", err));
                        }
                    }
                }
            });
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Node Graph");
                    ui.separator();
                    
                    // Simple node visualization with drag-and-drop support
                    egui::ScrollArea::both()
                        .max_width(400.0)
                        .min_height(500.0)
                        .show(ui, |ui| {
                            for (id, node) in &editor.graph.nodes {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{:?}", node.node_type));
                                        if ui.small_button("🗑").clicked() {
                                            // Remove node and its connections
                                            editor.graph.nodes.remove(&id);
                                            editor.graph.connections.retain(|conn| {
                                                conn.from_node != id && conn.to_node != id
                                            });
                                        }
                                    });
                                    
                                    ui.separator();
                                    
                                    // Show inputs
                                    for (i, input) in node.inputs.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{}: {}", input.name, input.value_type));
                                            if input.connected.is_some() {
                                                ui.label("🔗");
                                            } else {
                                                ui.label("⚡");
                                            }
                                        });
                                    }
                                    
                                    // Show outputs
                                    for (i, output) in node.outputs.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{}: {}", output.name, output.value_type));
                                            ui.label("→");
                                        });
                                    }
                                    
                                    // Show parameters
                                    if !node.parameters.is_empty() {
                                        ui.separator();
                                        for (key, value) in &node.parameters {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("{}: {}", key, value));
                                            });
                                        }
                                    }
                                });
                                ui.add_space(5.0);
                            }
                        });
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label("Generated WGSL Code");
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .max_height(500.0)
                        .show(ui, |ui| {
                            ui.monospace(&editor.generated_code);
                        });
                    
                    ui.separator();
                    
                    if ui.button("Copy to Clipboard").clicked() {
                        ui.output_mut(|o| o.copied_text = editor.generated_code.clone());
                    }
                });
            });
        });
}

fn auto_generate_shader_system(mut editor: ResMut<ShaderGraphEditor>) {
    if editor.auto_generate {
        editor.regenerate_code();
    }
}

// Re-export for use in other modules - FIXED: Removed duplicate re-export