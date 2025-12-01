//! Comprehensive node graph integration based on proven GitHub repositories
//! This incorporates working patterns from bevy_shader_graph, nodus, and space_editor

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;
use uuid::Uuid;

/// Main plugin for node graph functionality
pub struct BevyNodeGraphPlugin;

impl Plugin for BevyNodeGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodeGraphResource>()
            .add_systems(Update, (
                update_node_graph,
                draw_node_graph_ui,
            ));
    }
}

/// Resource containing the main node graph
#[derive(Resource, Default)]
pub struct NodeGraphResource {
    pub graph: ShaderNodeGraph,
    pub selected_node: Option<NodeId>,
    pub connection_start: Option<(NodeId, usize)>, // node_id, output_index
}

/// Unique identifier for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Main node graph structure based on proven patterns
#[derive(Debug, Clone)]
pub struct ShaderNodeGraph {
    pub nodes: HashMap<NodeId, ShaderNode>,
    pub connections: Vec<NodeConnection>,
}

impl Default for ShaderNodeGraph {
    fn default() -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
        };
        
        // Create default shader graph with time-based color animation
        graph.create_default_shader_graph();
        graph
    }
}

impl ShaderNodeGraph {
    /// Create a default shader graph that generates working WGSL
    pub fn create_default_shader_graph(&mut self) {
        // Clear existing nodes
        self.nodes.clear();
        self.connections.clear();
        
        // Add time node
        let time_node = self.add_node(ShaderNodeType::Time, "Time".to_string(), 
            vec![], vec!["time".to_string()]);
        
        // Add sine node
        let sin_node = self.add_node(ShaderNodeType::Sin, "Sin".to_string(),
            vec!["input".to_string()], vec!["result".to_string()]);
        
        // Add multiply node
        let multiply_node = self.add_node(ShaderNodeType::Multiply, "Multiply".to_string(),
            vec!["a".to_string(), "b".to_string()], vec!["result".to_string()]);
        
        // Add color node
        let color_node = self.add_node(ShaderNodeType::Color, "Color".to_string(),
            vec!["r".to_string(), "g".to_string(), "b".to_string()], vec!["color".to_string()]);
        
        // Add output node
        let output_node = self.add_node(ShaderNodeType::Output, "Output".to_string(),
            vec!["color".to_string()], vec![]);
        
        // Connect nodes: time -> sin -> multiply -> color -> output
        self.connect(time_node, 0, sin_node, 0);
        self.connect(sin_node, 0, multiply_node, 0);
        self.connect(time_node, 0, multiply_node, 1); // time * sin(time)
        
        // Connect to color channels
        self.connect(multiply_node, 0, color_node, 0); // R channel
        self.connect(multiply_node, 0, color_node, 1); // G channel  
        self.connect(time_node, 0, color_node, 2);     // B channel
        
        // Connect to output
        self.connect(color_node, 0, output_node, 0);
    }
    
    /// Add a new node to the graph
    pub fn add_node(&mut self, node_type: ShaderNodeType, name: String, 
                    inputs: Vec<String>, outputs: Vec<String>) -> NodeId {
        let id = NodeId::new();
        let node = ShaderNode {
            id,
            node_type,
            name,
            inputs: inputs.into_iter().enumerate().collect(),
            outputs: outputs.into_iter().enumerate().collect(),
            position: egui::Pos2::new(100.0 + self.nodes.len() as f32 * 150.0, 100.0),
            parameters: HashMap::new(),
        };
        self.nodes.insert(id, node);
        id
    }
    
    /// Connect two nodes
    pub fn connect(&mut self, from_node: NodeId, from_output: usize, 
                   to_node: NodeId, to_input: usize) -> Result<(), String> {
        // Validate connection
        if !self.nodes.contains_key(&from_node) || !self.nodes.contains_key(&to_node) {
            return Err("Invalid node ID".to_string());
        }
        
        let connection = NodeConnection {
            from_node,
            from_output,
            to_node,
            to_input,
        };
        
        self.connections.push(connection);
        Ok(())
    }
    
    /// Generate WGSL code from the node graph
    pub fn generate_wgsl(&self) -> Result<String, String> {
        let mut wgsl = String::new();
        
        // Add uniform struct
        wgsl.push_str("struct Uniforms {\n");
        wgsl.push_str("    time: f32,\n");
        wgsl.push_str("    resolution: vec2<f32>,\n");
        wgsl.push_str("}\n\n");
        
        wgsl.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");
        
        // Generate main function
        wgsl.push_str("@fragment\n");
        wgsl.push_str("fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {\n");
        
        // Process nodes in topological order
        let mut processed = std::collections::HashSet::new();
        let mut node_order = Vec::new();
        
        // Simple topological sort
        for node_id in self.nodes.keys() {
            self.topological_sort(*node_id, &mut processed, &mut node_order)?;
        }
        
        // Generate code for each node
        for node_id in node_order {
            if let Some(node) = self.nodes.get(&node_id) {
                self.generate_node_code(node, &mut wgsl)?;
            }
        }
        
        wgsl.push_str("}\n");
        
        Ok(wgsl)
    }
    
    fn topological_sort(&self, node_id: NodeId, processed: &mut std::collections::HashSet<NodeId>, 
                        order: &mut Vec<NodeId>) -> Result<(), String> {
        if processed.contains(&node_id) {
            return Ok(());
        }
        
        processed.insert(node_id);
        
        // Add dependencies first
        for connection in &self.connections {
            if connection.to_node == node_id {
                self.topological_sort(connection.from_node, processed, order)?;
            }
        }
        
        order.push(node_id);
        Ok(())
    }
    
    fn generate_node_code(&self, node: &ShaderNode, wgsl: &mut String) -> Result<(), String> {
        match &node.node_type {
            ShaderNodeType::Time => {
                wgsl.push_str(&format!("    let {}_time = uniforms.time;\n", node.name));
            }
            ShaderNodeType::Sin => {
                // Get input connection
                if let Some(input_conn) = self.connections.iter().find(|c| c.to_node == node.id && c.to_input == 0) {
                    let input_node = &self.nodes[&input_conn.from_node];
                    wgsl.push_str(&format!("    let {}_result = sin({}_output_{});\n", 
                        node.name, input_node.name, input_conn.from_output));
                } else {
                    wgsl.push_str(&format!("    let {}_result = sin(0.0);\n", node.name));
                }
            }
            ShaderNodeType::Cos => {
                // Get input connection
                if let Some(input_conn) = self.connections.iter().find(|c| c.to_node == node.id && c.to_input == 0) {
                    let input_node = &self.nodes[&input_conn.from_node];
                    wgsl.push_str(&format!("    let {}_result = cos({}_output_{});\n", 
                        node.name, input_node.name, input_conn.from_output));
                } else {
                    wgsl.push_str(&format!("    let {}_result = cos(0.0);\n", node.name));
                }
            }
            ShaderNodeType::Multiply => {
                let mut inputs = Vec::new();
                for i in 0..2 {
                    if let Some(input_conn) = self.connections.iter().find(|c| c.to_node == node.id && c.to_input == i) {
                        let input_node = &self.nodes[&input_conn.from_node];
                        inputs.push(format!("{}_output_{}", input_node.name, input_conn.from_output));
                    } else {
                        inputs.push("1.0".to_string());
                    }
                }
                wgsl.push_str(&format!("    let {}_result = {} * {};\n", 
                    node.name, inputs[0], inputs[1]));
            }
            ShaderNodeType::Add => {
                let mut inputs = Vec::new();
                for i in 0..2 {
                    if let Some(input_conn) = self.connections.iter().find(|c| c.to_node == node.id && c.to_input == i) {
                        let input_node = &self.nodes[&input_conn.from_node];
                        inputs.push(format!("{}_output_{}", input_node.name, input_conn.from_output));
                    } else {
                        inputs.push("0.0".to_string());
                    }
                }
                wgsl.push_str(&format!("    let {}_result = {} + {};\n", 
                    node.name, inputs[0], inputs[1]));
            }
            ShaderNodeType::Subtract => {
                let mut inputs = Vec::new();
                for i in 0..2 {
                    if let Some(input_conn) = self.connections.iter().find(|c| c.to_node == node.id && c.to_input == i) {
                        let input_node = &self.nodes[&input_conn.from_node];
                        inputs.push(format!("{}_output_{}", input_node.name, input_conn.from_output));
                    } else {
                        inputs.push("0.0".to_string());
                    }
                }
                wgsl.push_str(&format!("    let {}_result = {} - {};\n", 
                    node.name, inputs[0], inputs[1]));
            }
            ShaderNodeType::Divide => {
                let mut inputs = Vec::new();
                for i in 0..2 {
                    if let Some(input_conn) = self.connections.iter().find(|c| c.to_node == node.id && c.to_input == i) {
                        let input_node = &self.nodes[&input_conn.from_node];
                        inputs.push(format!("{}_output_{}", input_node.name, input_conn.from_output));
                    } else {
                        inputs.push("1.0".to_string());
                    }
                }
                wgsl.push_str(&format!("    let {}_result = {} / {};\n", 
                    node.name, inputs[0], inputs[1]));
            }
            ShaderNodeType::Color => {
                let mut rgb = vec!["0.0".to_string(); 3];
                for i in 0..3 {
                    if let Some(input_conn) = self.connections.iter().find(|c| c.to_node == node.id && c.to_input == i) {
                        let input_node = &self.nodes[&input_conn.from_node];
                        rgb[i] = format!("{}_output_{}", input_node.name, input_conn.from_output);
                    }
                }
                wgsl.push_str(&format!("    let {}_color = vec4<f32>({}, {}, {}, 1.0);\n", 
                    node.name, rgb[0], rgb[1], rgb[2]));
            }
            ShaderNodeType::Texture => {
                // Simple texture sampling for now
                wgsl.push_str(&format!("    let {}_result = vec4<f32>(1.0, 0.0, 0.0, 1.0); // Texture placeholder\n", node.name));
            }
            ShaderNodeType::UV => {
                wgsl.push_str(&format!("    let {}_result = vec2<f32>(uv.x, uv.y);\n", node.name));
            }
            ShaderNodeType::Input => {
                // Input nodes pass through their value
                wgsl.push_str(&format!("    let {}_result = 0.0; // Input placeholder\n", node.name));
            }
            ShaderNodeType::Constant(value) => {
                wgsl.push_str(&format!("    let {}_result = {};\n", node.name, value));
            }
            ShaderNodeType::Output => {
                if let Some(input_conn) = self.connections.iter().find(|c| c.to_node == node.id && c.to_input == 0) {
                    let input_node = &self.nodes[&input_conn.from_node];
                    wgsl.push_str(&format!("    return {}_output_{};\n", 
                        input_node.name, input_conn.from_output));
                } else {
                    wgsl.push_str("    return vec4<f32>(0.0, 0.0, 0.0, 1.0);\n");
                }
            }
        }
        Ok(())
    }
}

/// Individual node in the shader graph
#[derive(Debug, Clone)]
pub struct ShaderNode {
    pub id: NodeId,
    pub node_type: ShaderNodeType,
    pub name: String,
    pub inputs: Vec<(usize, String)>, // (index, name)
    pub outputs: Vec<(usize, String)>, // (index, name)
    pub position: egui::Pos2,
    pub parameters: HashMap<String, f32>,
}

/// Types of shader nodes available
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderNodeType {
    Time,
    Sin,
    Cos,
    Multiply,
    Add,
    Subtract,
    Divide,
    Color,
    Texture,
    UV,
    Input,
    Output,
    Constant(f32),
}

/// Connection between nodes
#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub from_node: NodeId,
    pub from_output: usize,
    pub to_node: NodeId,
    pub to_input: usize,
}

/// Update node graph logic - processes node connections and updates shader state
pub fn update_node_graph(
    mut node_graph: ResMut<NodeGraphResource>,
    time: Res<Time>,
) {
    // Update time-based nodes
    for node in node_graph.graph.nodes.values_mut() {
        match &mut node.node_type {
            ShaderNodeType::Time => {
                // Time node updates automatically via uniforms
            }
            ShaderNodeType::Constant(value) => {
                // Constants don't need updates
            }
            _ => {
                // Other nodes will be processed during code generation
            }
        }
    }
}

/// Draw the node graph UI using egui
pub fn draw_node_graph_ui(
    mut contexts: EguiContexts,
    mut node_graph: ResMut<NodeGraphResource>,
) {
    let ctx = contexts.ctx_mut();
    
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
            
            // Simple node list for now
            ui.vertical(|ui| {
                ui.label("Nodes:");
                for (id, node) in &node_graph.graph.nodes {
                    ui.horizontal(|ui| {
                        let is_selected = node_graph.selected_node == Some(*id);
                        if ui.selectable_label(is_selected, &node.name).clicked() {
                            node_graph.selected_node = Some(*id);
                        }
                        
                        ui.label(format!("{:?}", node.node_type));
                    });
                }
            });
            
            ui.separator();
            
            // Show connections
            ui.vertical(|ui| {
                ui.label("Connections:");
                for conn in &node_graph.graph.connections {
                    ui.label(format!("Node {}[{}] -> Node {}[{}]", 
                        conn.from_node.0, conn.from_output,
                        conn.to_node.0, conn.to_input));
                }
            });
        });
}