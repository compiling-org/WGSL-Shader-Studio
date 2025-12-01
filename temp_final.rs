//! Enhanced node graph integration based on proven GitHub repositories
//! This incorporates working patterns from bevy_shader_graph, nodus, space_editor, and bevy_animation_graph

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// Main plugin for comprehensive node graph functionality
pub struct BevyNodeGraphPluginEnhanced;

impl Plugin for BevyNodeGraphPluginEnhanced {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodeGraphResource>()
            .add_systems(Update, (
                update_node_graph_enhanced,
                draw_node_graph_ui_enhanced,
                draw_node_graph_toolbar,
                handle_node_interactions,
            ));
    }
}

/// Resource containing the comprehensive node graph system
#[derive(Resource)]
pub struct NodeGraphResource {
    pub graph: ShaderNodeGraph,
    pub selected_node: Option<NodeId>,
    pub selected_nodes: HashSet<NodeId>,
    pub connection_start: Option<(NodeId, usize)>,
    pub drag_state: Option<DragState>,
    pub clipboard: Option<ClipboardData>,
    pub undo_stack: VecDeque<GraphState>,
    pub redo_stack: VecDeque<GraphState>,
    pub max_undo_steps: usize,
    pub auto_layout_enabled: bool,
    pub show_grid: bool,
    pub snap_to_grid: bool,
    pub grid_size: f32,
    pub pan: egui::Vec2,
    pub zoom: f32,
    pub selection_rect: Option<egui::Rect>,
    pub is_selecting: bool,
    pub context_menu_open: bool,
    pub context_menu_pos: egui::Pos2,
}

impl Default for NodeGraphResource {
    fn default() -> Self {
        let mut graph = ShaderNodeGraph::default();
        graph.create_default_shader_graph();
        
        Self {
            graph,
            selected_node: None,
            selected_nodes: HashSet::new(),
            connection_start: None,
            drag_state: None,
            clipboard: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo_steps: 50,
            auto_layout_enabled: false,
            show_grid: true,
            snap_to_grid: true,
            grid_size: 20.0,
            pan: egui::Vec2::new(0.0, 0.0),
            zoom: 1.0,
            selection_rect: None,
            is_selecting: false,
            context_menu_open: false,
            context_menu_pos: egui::Pos2::new(0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub start_pos: egui::Pos2,
    pub nodes_start_pos: HashMap<NodeId, egui::Pos2>,
}

#[derive(Debug, Clone)]
pub struct ClipboardData {
    pub nodes: Vec<ShaderNode>,
    pub connections: Vec<NodeConnection>,
}

#[derive(Debug, Clone)]
pub struct GraphState {
    pub nodes: HashMap<NodeId, ShaderNode>,
    pub connections: Vec<NodeConnection>,
}

/// Unique identifier for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Comprehensive node types based on reference repositories
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderNodeType {
    // Input nodes
    Time,
    Resolution,
    Mouse,
    UV,
    VertexPosition,
    Normal,
    
    // Math operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Sin,
    Cos,
    Tan,
    Pow,
    Sqrt,
    Abs,
    Floor,
    Ceil,
    Fract,
    Min,
    Max,
    Clamp,
    Step,
    SmoothStep,
    
    // Vector operations
    Vec2,
    Vec3,
    Vec4,
    Normalize,
    Length,
    Distance,
    Dot,
    Cross,
    Reflect,
    Refract,
    
    // Color operations
    Color,
    ColorMix,
    ColorAdjust,
    Brightness,
    Contrast,
    Saturation,
    Hue,
    
    // Texture operations
    Texture2D,
    SampleTexture,
    TextureSize,
    
    // Procedural
    Noise2D,
    Noise3D,
    Voronoi,
    
    // Output
    FragmentOutput,
    VertexOutput,
}

/// Enhanced shader node with comprehensive features
#[derive(Debug, Clone)]
pub struct ShaderNode {
    pub id: NodeId,
    pub node_type: ShaderNodeType,
    pub name: String,
    pub inputs: Vec<NodeInput>,
    pub outputs: Vec<NodeOutput>,
    pub position: egui::Pos2,
    pub size: egui::Vec2,
    pub parameters: HashMap<String, f32>,
    pub color: egui::Color32,
    pub collapsed: bool,
    pub selected: bool,
    pub comment: String,
}

#[derive(Debug, Clone)]
pub struct NodeInput {
    pub name: String,
    pub port_type: PortType,
    pub connected: bool,
    pub connection_id: Option<usize>,
    pub position: egui::Pos2,
}

#[derive(Debug, Clone)]
pub struct NodeOutput {
    pub name: String,
    pub port_type: PortType,
    pub connections: Vec<usize>,
    pub position: egui::Pos2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Texture,
    Any,
}

#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub from_node: NodeId,
    pub from_output: usize,
    pub to_node: NodeId,
    pub to_input: usize,
    pub color: egui::Color32,
}

/// Comprehensive shader node graph
#[derive(Debug, Clone)]
pub struct ShaderNodeGraph {
    pub nodes: HashMap<NodeId, ShaderNode>,
    pub connections: Vec<NodeConnection>,
    pub next_node_id: u32,
}

impl Default for ShaderNodeGraph {
    fn default() -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            next_node_id: 1,
        };
        
        graph.create_default_shader_graph();
        graph
    }
}

impl ShaderNodeGraph {
    /// Create a comprehensive default shader graph
    pub fn create_default_shader_graph(&mut self) {
        self.clear();
        
        // Add time node
        let time_node = self.add_node(ShaderNodeType::Time, "Time", 
            vec![], vec!["time".to_string()]);
        
        // Add resolution node  
        let resolution_node = self.add_node(ShaderNodeType::Resolution, "Resolution",
            vec![], vec!["resolution".to_string()]);
        
        // Add UV node
        let uv_node = self.add_node(ShaderNodeType::UV, "UV",
            vec![], vec!["uv".to_string()]);
        
        // Add sine wave generator
        let sin_node = self.add_node(ShaderNodeType::Sin, "Sin",
            vec!["input".to_string()], vec!["result".to_string()]);
        
        // Add color mixing
        let color_mix_node = self.add_node(ShaderNodeType::ColorMix, "Color Mix",
            vec!["a".to_string(), "b".to_string(), "factor".to_string()], 
            vec!["result".to_string()]);
        
        // Add final output
        let output_node = self.add_node(ShaderNodeType::FragmentOutput, "Fragment Output",
            vec!["color".to_string()], vec![]);
        
        // Connect nodes for animated gradient
        self.connect(time_node, 0, sin_node, 0);
        self.connect(sin_node, 0, color_mix_node, 2); // Use sin(time) as mix factor
        self.connect(uv_node, 0, color_mix_node, 0); // UV as first color
        self.connect(time_node, 0, color_mix_node, 1); // Time as second color
        self.connect(color_mix_node, 0, output_node, 0);
    }
    
    /// Clear all nodes and connections
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.connections.clear();
    }
    
    /// Add a new node to the graph
    pub fn add_node(&mut self, node_type: ShaderNodeType, name: &str,
                    inputs: Vec<String>, outputs: Vec<String>) -> NodeId {
        let id = NodeId::new();
        let node = ShaderNode {
            id,
            node_type: node_type.clone(),
            name: name.to_string(),
            inputs: inputs.into_iter().enumerate().map(|(i, name)| NodeInput {
                name,
                port_type: self.get_default_port_type(&node_type),
                connected: false,
                connection_id: None,
                position: egui::Pos2::new(0.0, 0.0),
            }).collect(),
            outputs: outputs.into_iter().enumerate().map(|(i, name)| NodeOutput {
                name,
                port_type: self.get_default_port_type(&node_type),
                connections: Vec::new(),
                position: egui::Pos2::new(0.0, 0.0),
            }).collect(),
            position: egui::Pos2::new(100.0 + (self.nodes.len() % 5) as f32 * 200.0, 
                                      100.0 + (self.nodes.len() / 5) as f32 * 150.0),
            size: egui::Vec2::new(150.0, 80.0),
            parameters: HashMap::new(),
            color: self.get_node_color(&node_type),
            collapsed: false,
            selected: false,
            comment: String::new(),
        };
        
        self.nodes.insert(id, node);
        id
    }
    
    fn get_default_port_type(&self, node_type: &ShaderNodeType) -> PortType {
        match node_type {
            ShaderNodeType::Time | ShaderNodeType::Resolution | ShaderNodeType::Mouse => PortType::Vec2,
            ShaderNodeType::UV => PortType::Vec2,
            ShaderNodeType::Add | ShaderNodeType::Subtract | ShaderNodeType::Multiply | ShaderNodeType::Divide => PortType::Float,
            ShaderNodeType::Sin | ShaderNodeType::Cos | ShaderNodeType::Tan => PortType::Float,
            ShaderNodeType::Color | ShaderNodeType::ColorMix => PortType::Vec3,
            ShaderNodeType::Texture2D | ShaderNodeType::SampleTexture => PortType::Texture,
            _ => PortType::Float,
        }
    }
    
    fn get_node_color(&self, node_type: &ShaderNodeType) -> egui::Color32 {
        match node_type {
            ShaderNodeType::Time | ShaderNodeType::Resolution | ShaderNodeType::Mouse | ShaderNodeType::UV => 
                egui::Color32::from_rgb(100, 150, 200), // Input nodes - blue
            ShaderNodeType::Add | ShaderNodeType::Subtract | ShaderNodeType::Multiply | ShaderNodeType::Divide |
            ShaderNodeType::Sin | ShaderNodeType::Cos | ShaderNodeType::Tan | ShaderNodeType::Pow |
            ShaderNodeType::Sqrt | ShaderNodeType::Abs | ShaderNodeType::Floor | ShaderNodeType::Ceil |
            ShaderNodeType::Fract | ShaderNodeType::Min | ShaderNodeType::Max | ShaderNodeType::Clamp |
            ShaderNodeType::Step | ShaderNodeType::SmoothStep => 
                egui::Color32::from_rgb(150, 100, 200), // Math nodes - purple
            ShaderNodeType::Vec2 | ShaderNodeType::Vec3 | ShaderNodeType::Vec4 | ShaderNodeType::Normalize |
            ShaderNodeType::Length | ShaderNodeType::Distance | ShaderNodeType::Dot | ShaderNodeType::Cross |
            ShaderNodeType::Reflect | ShaderNodeType::Refract => 
                egui::Color32::from_rgb(200, 150, 100), // Vector nodes - orange
            ShaderNodeType::Color | ShaderNodeType::ColorMix | ShaderNodeType::ColorAdjust |
            ShaderNodeType::Brightness | ShaderNodeType::Contrast | ShaderNodeType::Saturation |
            ShaderNodeType::Hue => 
                egui::Color32::from_rgb(200, 100, 150), // Color nodes - pink
            ShaderNodeType::Texture2D | ShaderNodeType::SampleTexture | ShaderNodeType::TextureSize |
            ShaderNodeType::Noise2D | ShaderNodeType::Noise3D | ShaderNodeType::Voronoi => 
                egui::Color32::from_rgb(100, 200, 150), // Texture/Procedural nodes - green
            ShaderNodeType::FragmentOutput | ShaderNodeType::VertexOutput => 
                egui::Color32::from_rgb(200, 200, 100), // Output nodes - yellow
            _ => egui::Color32::from_rgb(150, 150, 150), // Default - gray
        }
    }
    
    /// Connect two nodes
    pub fn connect(&mut self, from_node: NodeId, from_output: usize, 
                   to_node: NodeId, to_input: usize) -> Result<(), String> {
        // Validate nodes exist
        if !self.nodes.contains_key(&from_node) || !self.nodes.contains_key(&to_node) {
            return Err("Invalid node ID".to_string());
        }
        
        // Validate port indices
        let from_node_ref = &self.nodes[&from_node];
        let to_node_ref = &self.nodes[&to_node];
        
        if from_output >= from_node_ref.outputs.len() {
            return Err("Invalid output port index".to_string());
        }
        
        if to_input >= to_node_ref.inputs.len() {
            return Err("Invalid input port index".to_string());
        }
        
        // Check type compatibility
        let from_type = &from_node_ref.outputs[from_output].port_type;
        let to_type = &to_node_ref.inputs[to_input].port_type;
        
        if !self.are_types_compatible(from_type, to_type) {
            return Err(format!("Type mismatch: {:?} -> {:?}", from_type, to_type));
        }
        
        let connection = NodeConnection {
            from_node,
            from_output,
            to_node,
            to_input,
            color: self.get_connection_color(from_type),
        };
        
        self.connections.push(connection);
        
        // Update node connection states
        if let Some(node) = self.nodes.get_mut(&to_node) {
            node.inputs[to_input].connected = true;
        }
        
        if let Some(node) = self.nodes.get_mut(&from_node) {
            node.outputs[from_output].connections.push(self.connections.len() - 1);
        }
        
        Ok(())
    }
    
    fn are_types_compatible(&self, from: &PortType, to: &PortType) -> bool {
        match (from, to) {
            (PortType::Any, _) | (_, PortType::Any) => true,
            (a, b) => a == b,
        }
    }
    
    fn get_connection_color(&self, port_type: &PortType) -> egui::Color32 {
        match port_type {
            PortType::Float => egui::Color32::from_rgb(100, 200, 100),
            PortType::Vec2 => egui::Color32::from_rgb(100, 150, 250),
            PortType::Vec3 => egui::Color32::from_rgb(250, 150, 100),
            PortType::Vec4 => egui::Color32::from_rgb(250, 100, 250),
            PortType::Color => egui::Color32::from_rgb(250, 200, 100),
            PortType::Texture => egui::Color32::from_rgb(150, 250, 150),
            PortType::Any => egui::Color32::from_rgb(200, 200, 200),
        }
    }
    
    /// Generate comprehensive WGSL code
    pub fn generate_wgsl(&self) -> Result<String, String> {
        let mut wgsl = String::new();
        
        // Add struct definitions
        wgsl.push_str("struct VertexOutput {\n");
        wgsl.push_str("    @builtin(position) position: vec4<f32>,\n");
        wgsl.push_str("    @location(0) uv: vec2<f32>,\n");
        wgsl.push_str("}\n\n");
        
        // Add uniform struct
        wgsl.push_str("struct Uniforms {\n");
        wgsl.push_str("    time: f32,\n");
        wgsl.push_str("    resolution: vec2<f32>,\n");
        wgsl.push_str("    mouse: vec2<f32>,\n");
        wgsl.push_str("}\n\n");
        
        // Add uniforms binding
        wgsl.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");
        
        // Add vertex shader
        wgsl.push_str("@vertex\n");
        wgsl.push_str("fn vertex_main(@location(0) position: vec2<f32>) -> VertexOutput {\n");
        wgsl.push_str("    var output: VertexOutput;\n");
        wgsl.push_str("    output.position = vec4<f32>(position, 0.0, 1.0);\n");
        wgsl.push_str("    output.uv = position * 0.5 + 0.5;\n");
        wgsl.push_str("    return output;\n");
        wgsl.push_str("}\n\n");
        
        // Generate fragment shader
        wgsl.push_str("@fragment\n");
        wgsl.push_str("fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {\n");
        wgsl.push_str("    let uv = input.uv;\n");
        wgsl.push_str("    var color = vec3<f32>(0.0);\n\n");
        
        // Generate node variables
        let mut node_vars = HashMap::new();
        let mut processed = HashSet::new();
        
        // Process nodes in topological order
        for node_id in self.nodes.keys() {
            self.generate_node_variables(*node_id, &mut node_vars, &mut processed)?;
        }
        
        // Add generated variables to WGSL
        for (var_name, var_code) in node_vars {
            wgsl.push_str(&format!("    {}\n", var_code));
        }
        
        wgsl.push_str("\n    return vec4<f32>(color, 1.0);\n");
        wgsl.push_str("}\n");
        
        Ok(wgsl)
    }
    
    fn generate_node_variables(&self, node_id: NodeId, node_vars: &mut HashMap<String, String>, 
                              processed: &mut HashSet<NodeId>) -> Result<(), String> {
        if processed.contains(&node_id) {
            return Ok(());
        }
        
        processed.insert(node_id);
        
        if let Some(node) = self.nodes.get(&node_id) {
            // Generate dependencies first
            for connection in &self.connections {
                if connection.to_node == node_id {
                    self.generate_node_variables(connection.from_node, node_vars, processed)?;
                }
            }
            
            // Generate this node's variable
            let var_code = self.generate_single_node_variable(node)?;
            if !var_code.is_empty() {
                node_vars.insert(format!("node_{}", node_id.0), var_code);
            }
        }
        
        Ok(())
    }
    
    fn generate_single_node_variable(&self, node: &ShaderNode) -> Result<String, String> {
        match &node.node_type {
            ShaderNodeType::Time => Ok(format!("let node_{}_time = uniforms.time;", node.id.0)),
            ShaderNodeType::Resolution => Ok(format!("let node_{}_resolution = uniforms.resolution;", node.id.0)),
            ShaderNodeType::UV => Ok(format!("let node_{}_uv = input.uv;", node.id.0)),
            ShaderNodeType::Sin => {
                if let Some(input_conn) = self.get_input_connection(node.id, 0) {
                    Ok(format!("let node_{}_result = sin({});", 
                        node.id.0, self.get_connection_variable(&input_conn)))
                } else {
                    Ok(format!("let node_{}_result = sin(uniforms.time);", node.id.0))
                }
            }
            ShaderNodeType::Color => {
                Ok(format!("let node_{}_color = vec3<f32>(0.5 + 0.5 * sin(uniforms.time), 0.5 + 0.5 * cos(uniforms.time), 0.5 + 0.5 * sin(uniforms.time * 1.3));", node.id.0))
            }
            ShaderNodeType::ColorMix => {
                if let (Some(a_conn), Some(b_conn), Some(factor_conn)) = (
                    self.get_input_connection(node.id, 0),
                    self.get_input_connection(node.id, 1),
                    self.get_input_connection(node.id, 2)
                ) {
                    Ok(format!("let node_{}_result = mix({}, {}, {});", 
                        node.id.0, 
                        self.get_connection_variable(&a_conn),
                        self.get_connection_variable(&b_conn),
                        self.get_connection_variable(&factor_conn)))
                } else {
                    Ok(format!("let node_{}_result = vec3<f32>(0.5, 0.5, 0.5);", node.id.0))
                }
            }
            _ => Ok(String::new()),
        }
    }
    
    fn get_input_connection(&self, node_id: NodeId, input_index: usize) -> Option<NodeConnection> {
        self.connections.iter()
            .find(|conn| conn.to_node == node_id && conn.to_input == input_index)
            .cloned()
    }
    
    fn get_connection_variable(&self, connection: &NodeConnection) -> String {
        let from_node = &self.nodes[&connection.from_node];
        match &from_node.node_type {
            ShaderNodeType::Time => format!("node_{}_time", connection.from_node.0),
            ShaderNodeType::Resolution => format!("node_{}_resolution", connection.from_node.0),
            ShaderNodeType::UV => format!("node_{}_uv", connection.from_node.0),
            ShaderNodeType::Sin => format!("node_{}_result", connection.from_node.0),
            ShaderNodeType::Color => format!("node_{}_color", connection.from_node.0),
            ShaderNodeType::ColorMix => format!("node_{}_result", connection.from_node.0),
            _ => format!("node_{}_out", connection.from_node.0),
        }
    }
}

// System functions
fn update_node_graph_enhanced(
    mut node_graph: ResMut<NodeGraphResource>,
    time: Res<Time>,
) {
    // Update any time-dependent node parameters
    for node in node_graph.graph.nodes.values_mut() {
        if let ShaderNodeType::Time = node.node_type {
            if let Some(time_param) = node.parameters.get_mut("time_scale") {
                *time_param = time.elapsed_seconds();
            }
        }
    }
}

fn draw_node_graph_ui_enhanced(
    mut node_graph: ResMut<NodeGraphResource>,
    mut egui_ctx: EguiContexts,
) {
    let ctx = egui_ctx.ctx_mut();
    
    egui::Window::new("Shader Graph Editor")
        .default_size([1200.0, 800.0])
        .resizable(true)
        .show(ctx, |ui| {
            draw_node_graph_canvas(ui, &mut node_graph);
        });
}

fn draw_node_graph_canvas(ui: &mut egui::Ui, node_graph: &mut NodeGraphResource) {
    let available_size = ui.available_size();
    
    // Create a scroll area for the node graph canvas
    egui::ScrollArea::both()
        .auto_shrink([false; 2])
        .show_viewport(ui, |ui, viewport| {
            // Draw grid if enabled
            if node_graph.show_grid {
                draw_grid(ui, viewport, node_graph.grid_size);
            }
            
            // Draw connections
            draw_connections(ui, node_graph);
            
            // Draw nodes
            draw_nodes(ui, node_graph);
            
            // Handle interactions
            handle_canvas_interactions(ui, node_graph);
        });
}

fn draw_grid(ui: &mut egui::Ui, viewport: &egui::Rect, grid_size: f32) {
    let painter = ui.painter();
    let grid_color = egui::Color32::from_gray(40);
    
    // Calculate grid lines
    let min_x = viewport.min.x.floor();
    let max_x = viewport.max.x.ceil();
    let min_y = viewport.min.y.floor();
    let max_y = viewport.max.y.ceil();
    
    // Vertical lines
    let start_x = (min_x / grid_size).floor() * grid_size;
    let mut x = start_x;
    while x <= max_x {
        painter.line_segment(
            [egui::Pos2::new(x, min_y), egui::Pos2::new(x, max_y)],
            (1.0, grid_color),
        );
        x += grid_size;
    }
    
    // Horizontal lines
    let start_y = (min_y / grid_size).floor() * grid_size;
    let mut y = start_y;
    while y <= max_y {
        painter.line_segment(
            [egui::Pos2::new(min_x, y), egui::Pos2::new(max_x, y)],
            (1.0, grid_color),
        );
        y += grid_size;
    }
}

fn draw_connections(ui: &mut egui::Ui, node_graph: &NodeGraphResource) {
    let painter = ui.painter();
    
    for connection in &node_graph.graph.connections {
        if let (Some(from_node), Some(to_node)) = (
            node_graph.graph.nodes.get(&connection.from_node),
            node_graph.graph.nodes.get(&connection.to_node)
        ) {
            let from_pos = from_node.position + egui::Vec2::new(from_node.size.x, 
                (connection.from_output as f32 + 0.5) * 20.0);
            let to_pos = to_node.position + egui::Vec2::new(0.0, 
                (connection.to_input as f32 + 0.5) * 20.0);
            
            // Draw bezier curve
            let control_offset = (to_pos.x - from_pos.x) * 0.5;
            let control1 = from_pos + egui::Vec2::new(control_offset, 0.0);
            let control2 = to_pos - egui::Vec2::new(control_offset, 0.0);
            
            painter.add(egui::Shape::CubicBezier(CubicBezierShape::from_points(
                [from_pos, control1, control2, to_pos],
                false,
                connection.color,
                (2.0, connection.color),
            )));
            
            // Draw connection points
            painter.circle_filled(from_pos, 4.0, connection.color);
            painter.circle_filled(to_pos, 4.0, connection.color);
        }
    }
}

use egui::CubicBezierShape;

fn draw_nodes(ui: &mut egui::Ui, node_graph: &mut NodeGraphResource) {
    for node in node_graph.graph.nodes.values_mut() {
        draw_single_node(ui, node, node_graph);
    }
}

fn draw_single_node(ui: &mut egui::Ui, node: &mut ShaderNode, node_graph: &NodeGraphResource) {
    let response = ui.allocate_ui_at_rect(
        egui::Rect::from_min_size(node.position, node.size),
        |ui| {
            let frame = egui::Frame::none()
                .fill(node.color)
                .stroke(egui::Stroke::new(2.0, if node.selected { 
                    egui::Color32::YELLOW 
                } else { 
                    egui::Color32::GRAY 
                }))
                .rounding(8.0);
            
            frame.show(ui, |ui| {
                ui.vertical(|ui| {
                    // Node header
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&node.name).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✕").clicked() {
                                // Mark for deletion
                            }
                        });
                    });
                    
                    if !node.collapsed {
                        ui.separator();
                        
                        // Inputs
                        for (i, input) in node.inputs.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let color = match input.port_type {
                                    PortType::Float => egui::Color32::GREEN,
                                    PortType::Vec2 => egui::Color32::BLUE,
                                    PortType::Vec3 => egui::Color32::RED,
                                    PortType::Vec4 => egui::Color32::YELLOW,
                                    PortType::Color => egui::Color32::from_rgb(255, 128, 0),
                                    PortType::Texture => egui::Color32::from_rgb(128, 255, 128),
                                    PortType::Any => egui::Color32::WHITE,
                                };
                                
                                ui.painter().circle_filled(ui.cursor().left_top() + egui::Vec2::new(8.0, 8.0), 6.0, color);
                                ui.label(&input.name);
                            });
                        }
                        
                        ui.separator();
                        
                        // Outputs
                        for (i, output) in node.outputs.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let color = match output.port_type {
                                        PortType::Float => egui::Color32::GREEN,
                                        PortType::Vec2 => egui::Color32::BLUE,
                                        PortType::Vec3 => egui::Color32::RED,
                                        PortType::Vec4 => egui::Color32::YELLOW,
                                        PortType::Color => egui::Color32::from_rgb(255, 128, 0),
                                        PortType::Texture => egui::Color32::from_rgb(128, 255, 128),
                                        PortType::Any => egui::Color32::WHITE,
                                    };
                                    
                                    ui.painter().circle_filled(ui.cursor().right_top() + egui::Vec2::new(-8.0, 8.0), 6.0, color);
                                    ui.label(&output.name);
                                });
                            });
                        }
                    }
                });
            });
        },
    );
    
    // Handle node dragging
    if response.response.dragged() {
        if let Some(drag_state) = &node_graph.drag_state {
            let delta = response.response.drag_delta();
            node.position += delta;
            
            if node_graph.snap_to_grid {
                node.position.x = (node.position.x / node_graph.grid_size).round() * node_graph.grid_size;
                node.position.y = (node.position.y / node_graph.grid_size).round() * node_graph.grid_size;
            }
        }
    }
}

fn handle_canvas_interactions(ui: &mut egui::Ui, node_graph: &mut NodeGraphResource) {
    let response = ui.interact(ui.clip_rect(), ui.id(), egui::Sense::click_and_drag());
    
    if response.dragged() {
        // Handle canvas panning
        if node_graph.drag_state.is_none() {
            // Start dragging canvas
        }
    }
    
    if response.clicked() {
        // Deselect all nodes
        node_graph.selected_nodes.clear();
        for node in node_graph.graph.nodes.values_mut() {
            node.selected = false;
        }
    }
}

fn draw_node_graph_toolbar(
    mut node_graph: ResMut<NodeGraphResource>,
    mut egui_ctx: EguiContexts,
) {
    let ctx = egui_ctx.ctx_mut();
    
    egui::TopBottomPanel::top("node_graph_toolbar")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Node Graph Toolbar");
                ui.separator();
                
                // Node creation buttons
                ui.menu_button("Add Node", |ui| {
                    if ui.button("Input → Time").clicked() {
                        node_graph.graph.add_node(ShaderNodeType::Time, "Time", vec![], vec!["time".to_string()]);
                        ui.close_menu();
                    }
                    if ui.button("Input → Resolution").clicked() {
                        node_graph.graph.add_node(ShaderNodeType::Resolution, "Resolution", vec![], vec!["resolution".to_string()]);
                        ui.close_menu();
                    }
                    if ui.button("Input → UV").clicked() {
                        node_graph.graph.add_node(ShaderNodeType::UV, "UV", vec![], vec!["uv".to_string()]);
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Math → Add").clicked() {
                        node_graph.graph.add_node(ShaderNodeType::Add, "Add", vec!["a".to_string(), "b".to_string()], vec!["result".to_string()]);
                        ui.close_menu();
                    }
                    if ui.button("Math → Multiply").clicked() {
                        node_graph.graph.add_node(ShaderNodeType::Multiply, "Multiply", vec!["a".to_string(), "b".to_string()], vec!["result".to_string()]);
                        ui.close_menu();
                    }
                    if ui.button("Math → Sin").clicked() {
                        node_graph.graph.add_node(ShaderNodeType::Sin, "Sin", vec!["input".to_string()], vec!["result".to_string()]);
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Color → Color Mix").clicked() {
                        node_graph.graph.add_node(ShaderNodeType::ColorMix, "Color Mix", vec!["a".to_string(), "b".to_string(), "factor".to_string()], vec!["result".to_string()]);
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Output → Fragment Output").clicked() {
                        node_graph.graph.add_node(ShaderNodeType::FragmentOutput, "Fragment Output", vec!["color".to_string()], vec![]);
                        ui.close_menu();
                    }
                });
                
                ui.separator();
                
                // Graph operations
                if ui.button("Generate WGSL").clicked() {
                    match node_graph.graph.generate_wgsl() {
                        Ok(wgsl_code) => {
                            println!("Generated WGSL code:");
                            println!("{}", wgsl_code);
                        }
                        Err(e) => {
                            eprintln!("Failed to generate WGSL: {}", e);
                        }
                    }
                }
                
                if ui.button("Clear Graph").clicked() {
                    node_graph.graph.clear();
                }
                
                if ui.button("Auto Layout").clicked() {
                    node_graph.auto_layout_enabled = !node_graph.auto_layout_enabled;
                }
                
                ui.separator();
                
                // View options
                ui.checkbox(&mut node_graph.show_grid, "Show Grid");
                ui.checkbox(&mut node_graph.snap_to_grid, "Snap to Grid");
                
                ui.separator();
                
                // Undo/Redo
                if ui.button("Undo").clicked() {
                    // Implement undo
                }
                if ui.button("Redo").clicked() {
                    // Implement redo
                }
            });
        });
}

fn handle_node_interactions(
    mut node_graph: ResMut<NodeGraphResource>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Handle keyboard shortcuts
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        // Delete selected nodes
        let selected: Vec<_> = node_graph.selected_nodes.iter().copied().collect();
        for node_id in selected {
            node_graph.graph.nodes.remove(&node_id);
            node_graph.connections.retain(|conn| {
                conn.from_node != node_id && conn.to_node != node_id
            });
        }
        node_graph.selected_nodes.clear();
    }
    
    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
        if keys.just_pressed(KeyCode::KeyA) {
            // Select all nodes
            node_graph.selected_nodes = node_graph.graph.nodes.keys().copied().collect();
            for node in node_graph.graph.nodes.values_mut() {
                node.selected = true;
            }
        }
        
        if keys.just_pressed(KeyCode::KeyC) {
            // Copy selected nodes
            // Implement copy functionality
        }
        
        if keys.just_pressed(KeyCode::KeyV) {
            // Paste nodes
            // Implement paste functionality
        }
    }
}

/// Update function for node graph
