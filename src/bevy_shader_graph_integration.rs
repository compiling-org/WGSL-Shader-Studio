use bevy::prelude::*;
// FIXED: Removed invalid import - CommandQueue doesn't exist in bevy::ecs::system
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::wgsl_ast_parser::{AstNode, WgslAstParser};
use crate::shader_module_system::{ShaderModule, ModuleId, ShaderModuleSystem};
use crate::shader_transpiler::{ShaderLanguage, TranspilerOptions};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub position: Vec2,
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub properties: HashMap<String, NodeProperty>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Input,
    Output,
    Math(MathOperation),
    Texture,
    Color,
    Vector,
    Matrix,
    Function(String),
    Custom(String),
    Group(NodeGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Sine,
    Cosine,
    Tangent,
    Power,
    Logarithm,
    Minimum,
    Maximum,
    Clamp,
    Mix,
    Step,
    SmoothStep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroup {
    pub name: String,
    pub nodes: Vec<NodeId>,
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePort {
    pub id: PortId,
    pub name: String,
    pub port_type: PortType,
    pub node_id: NodeId,
    pub direction: PortDirection,
    pub default_value: Option<NodeValue>,
    pub connected_to: Vec<PortId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortType {
    Float,
    Float2,
    Float3,
    Float4,
    Int,
    Bool,
    Color,
    Vector2,
    Vector3,
    Vector4,
    Matrix3,
    Matrix4,
    Texture2D,
    Texture3D,
    TextureCube,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeProperty {
    Float(f32),
    Float2(f32, f32),
    Float3(f32, f32, f32),
    Float4(f32, f32, f32, f32),
    Int(i32),
    Bool(bool),
    String(String),
    Color([f32; 4]),
    Vector([f32; 3]),
    Matrix([[f32; 4]; 4]),
    Texture(String),
    Enum(String, Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeValue {
    Float(f32),
    Float2(f32, f32),
    Float3(f32, f32, f32),
    Float4(f32, f32, f32, f32),
    Int(i32),
    Bool(bool),
    Color([f32; 4]),
    Vector([f32; 3]),
    Matrix([[f32; 4]; 4]),
    Texture(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub is_collapsed: bool,
    pub is_selected: bool,
    pub is_preview_enabled: bool,
    pub preview_size: Vec2,
    pub custom_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderGraph {
    pub id: Uuid,
    pub name: String,
    pub nodes: HashMap<NodeId, ShaderNode>,
    pub ports: HashMap<PortId, NodePort>,
    pub connections: Vec<PortConnection>,
    pub entry_points: HashMap<String, NodeId>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConnection {
    pub from: PortId,
    pub to: PortId,
    pub connection_type: ConnectionType,
    pub strength: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionType {
    Direct,
    Interpolated,
    Conditional,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub version: String,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub modified_at: String,
    pub viewport_position: Vec2,
    pub viewport_scale: f32,
    pub grid_enabled: bool,
    pub snap_to_grid: bool,
    pub grid_size: f32,
}

#[derive(Debug, Error)]
pub enum NodeGraphError {
    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),
    
    #[error("Port not found: {0:?}")]
    PortNotFound(PortId),
    
    #[error("Invalid connection: {0:?} -> {1:?}")]
    InvalidConnection(PortId, PortId),
    
    #[error("Type mismatch: {0} != {1}")]
    TypeMismatch(PortType, PortType),
    
    #[error("Circular connection detected")]
    CircularConnection,
    
    #[error("Node validation failed: {0}")]
    ValidationError(String),
    
    #[error("Compilation error: {0}")]
    CompilationError(String),
}

pub type NodeGraphResult<T> = Result<T, NodeGraphError>;

pub struct ShaderNodeGraph {
    graphs: Arc<RwLock<HashMap<Uuid, ShaderGraph>>>,
    active_graph: Arc<RwLock<Option<Uuid>>>,
    node_templates: Arc<RwLock<HashMap<String, NodeTemplate>>>,
    module_system: Arc<ShaderModuleSystem>,
    validation_rules: Vec<Box<dyn Fn(&ShaderGraph) -> NodeGraphResult<()>>>,
}

#[derive(Clone)]
pub struct NodeTemplate {
    pub node_type: NodeType,
    pub name: String,
    pub category: String,
    pub description: String,
    pub inputs: Vec<TemplatePort>,
    pub outputs: Vec<TemplatePort>,
    pub properties: Vec<TemplateProperty>,
    pub icon: String,
    pub color: Color,
}

#[derive(Clone)]
pub struct TemplatePort {
    pub name: String,
    pub port_type: PortType,
    pub default_value: Option<NodeValue>,
    pub description: String,
}

#[derive(Clone)]
pub struct TemplateProperty {
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: NodeProperty,
    pub description: String,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

#[derive(Clone, PartialEq)]
pub enum PropertyType {
    Float,
    Int,
    Bool,
    String,
    Color,
    Vector,
    Enum(Vec<String>),
}

impl ShaderNodeGraph {
    pub fn new(module_system: Arc<ShaderModuleSystem>) -> Self {
        let mut instance = Self {
            graphs: Arc::new(RwLock::new(HashMap::new())),
            active_graph: Arc::new(RwLock::new(None)),
            node_templates: Arc::new(RwLock::new(HashMap::new())),
            module_system,
            validation_rules: Vec::new(),
        };

        instance.initialize_default_templates();
        instance.initialize_validation_rules();
        instance
    }

    pub fn create_graph(&self, name: String) -> NodeGraphResult<Uuid> {
        let graph = ShaderGraph {
            id: Uuid::new_v4(),
            name,
            nodes: HashMap::new(),
            ports: HashMap::new(),
            connections: Vec::new(),
            entry_points: HashMap::new(),
            metadata: GraphMetadata {
                version: "1.0.0".to_string(),
                author: "WGSL Shader Studio".to_string(),
                description: "Shader node graph".to_string(),
                tags: vec!["shader".to_string(), "node_graph".to_string()],
                created_at: chrono::Utc::now().to_rfc3339(),
                modified_at: chrono::Utc::now().to_rfc3339(),
                viewport_position: Vec2::ZERO,
                viewport_scale: 1.0,
                grid_enabled: true,
                snap_to_grid: true,
                grid_size: 20.0,
            },
        };

        let graph_id = graph.id;
        
        let mut graphs = self.graphs.write().unwrap();
        graphs.insert(graph_id, graph);
        
        let mut active = self.active_graph.write().unwrap();
        *active = Some(graph_id);

        Ok(graph_id)
    }

    pub fn add_node(
        &self,
        graph_id: Uuid,
        node_type: NodeType,
        position: Vec2,
    ) -> NodeGraphResult<NodeId> {
        let mut graphs = self.graphs.write().unwrap();
        let graph = graphs.get_mut(&graph_id)
            .ok_or(NodeGraphError::NodeNotFound(NodeId(graph_id.into())))?;

        let node_id = NodeId(Uuid::new_v4());
        let node = self.create_node(node_id, node_type, position)?;

        graph.nodes.insert(node_id, node);
        graph.metadata.modified_at = chrono::Utc::now().to_rfc3339();

        Ok(node_id)
    }

    pub fn connect_ports(
        &self,
        graph_id: Uuid,
        from_port: PortId,
        to_port: PortId,
    ) -> NodeGraphResult<()> {
        let mut graphs = self.graphs.write().unwrap();
        let graph = graphs.get_mut(&graph_id)
            .ok_or(NodeGraphError::NodeNotFound(NodeId(graph_id.into())))?;

        self.validate_connection(graph, from_port, to_port)?;

        let connection = PortConnection {
            from: from_port,
            to: to_port,
            connection_type: ConnectionType::Direct,
            strength: 1.0,
        };

        graph.connections.push(connection);
        
        if let Some(from_port_data) = graph.ports.get_mut(&from_port) {
            from_port_data.connected_to.push(to_port);
        }
        
        if let Some(to_port_data) = graph.ports.get_mut(&to_port) {
            to_port_data.connected_to.push(from_port);
        }

        graph.metadata.modified_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn compile_graph(&self, graph_id: Uuid) -> NodeGraphResult<String> {
        let graphs = self.graphs.read().unwrap();
        let graph = graphs.get(&graph_id)
            .ok_or(NodeGraphError::NodeNotFound(NodeId(graph_id.into())))?;

        self.validate_graph(graph)?;

        let compiler = ShaderGraphCompiler::new(self.module_system.clone());
        compiler.compile(graph)
    }

    pub fn get_node_templates(&self) -> Vec<NodeTemplate> {
        let templates = self.node_templates.read().unwrap();
        templates.values().cloned().collect()
    }

    pub fn validate_graph(&self, graph: &ShaderGraph) -> NodeGraphResult<()> {
        for rule in &self.validation_rules {
            rule(graph)?;
        }
        Ok(())
    }

    fn create_node(&self, node_id: NodeId, node_type: NodeType, position: Vec2) -> NodeGraphResult<ShaderNode> {
        let templates = self.node_templates.read().unwrap();
        let template_name = self.get_template_name(&node_type);
        
        let template = templates.get(&template_name)
            .ok_or_else(|| NodeGraphError::ValidationError(format!("Unknown node type: {:?}", node_type)))?;

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for (i, template_input) in template.inputs.iter().enumerate() {
            let port_id = PortId(Uuid::new_v4());
            inputs.push(port_id);
        }

        for (i, template_output) in template.outputs.iter().enumerate() {
            let port_id = PortId(Uuid::new_v4());
            outputs.push(port_id);
        }

        let mut properties = HashMap::new();
        for template_prop in &template.properties {
            properties.insert(template_prop.name.clone(), template_prop.default_value.clone());
        }

        Ok(ShaderNode {
            id: node_id,
            node_type,
            position,
            inputs,
            outputs,
            properties,
            metadata: NodeMetadata {
                name: template.name.clone(),
                description: template.description.clone(),
                category: template.category.clone(),
                tags: vec![],
                is_collapsed: false,
                is_selected: false,
                is_preview_enabled: false,
                preview_size: Vec2::new(200.0, 150.0),
                custom_data: HashMap::new(),
            },
        })
    }

    fn get_template_name(&self, node_type: &NodeType) -> String {
        match node_type {
            NodeType::Input => "Input".to_string(),
            NodeType::Output => "Output".to_string(),
            NodeType::Math(op) => format!("Math_{:?}", op),
            NodeType::Texture => "Texture".to_string(),
            NodeType::Color => "Color".to_string(),
            NodeType::Vector => "Vector".to_string(),
            NodeType::Matrix => "Matrix".to_string(),
            NodeType::Function(name) => format!("Function_{}", name),
            NodeType::Custom(name) => name.clone(),
            NodeType::Group(_) => "Group".to_string(),
        }
    }

    fn initialize_default_templates(&self) {
        let mut templates = self.node_templates.write().unwrap();

        templates.insert("Input".to_string(), NodeTemplate {
            node_type: NodeType::Input,
            name: "Input".to_string(),
            category: "IO".to_string(),
            description: "Shader input parameter".to_string(),
            inputs: vec![],
            outputs: vec![
                TemplatePort {
                    name: "Value".to_string(),
                    port_type: PortType::Float,
                    default_value: None,
                    description: "Input value".to_string(),
                },
            ],
            properties: vec![
                TemplateProperty {
                    name: "Name".to_string(),
                    property_type: PropertyType::String,
                    default_value: NodeProperty::String("input".to_string()),
                    description: "Input parameter name".to_string(),
                    min: None,
                    max: None,
                },
            ],
            icon: "input".to_string(),
            color: Color::srgb(0.2, 0.8, 0.2),
        });

        templates.insert("Math_Add".to_string(), NodeTemplate {
            node_type: NodeType::Math(MathOperation::Add),
            name: "Add".to_string(),
            category: "Math".to_string(),
            description: "Add two values".to_string(),
            inputs: vec![
                TemplatePort {
                    name: "A".to_string(),
                    port_type: PortType::Float,
                    default_value: Some(NodeValue::Float(0.0)),
                    description: "First operand".to_string(),
                },
                TemplatePort {
                    name: "B".to_string(),
                    port_type: PortType::Float,
                    default_value: Some(NodeValue::Float(0.0)),
                    description: "Second operand".to_string(),
                },
            ],
            outputs: vec![
                TemplatePort {
                    name: "Result".to_string(),
                    port_type: PortType::Float,
                    default_value: None,
                    description: "Sum of A and B".to_string(),
                },
            ],
            properties: vec![],
            icon: "add".to_string(),
            color: Color::srgb(0.8, 0.2, 0.2),
        });

        templates.insert("Texture".to_string(), NodeTemplate {
            node_type: NodeType::Texture,
            name: "Texture Sample".to_string(),
            category: "Texture".to_string(),
            description: "Sample a texture".to_string(),
            inputs: vec![
                TemplatePort {
                    name: "UV".to_string(),
                    port_type: PortType::Float2,
                    default_value: Some(NodeValue::Float2(0.0, 0.0)),
                    description: "Texture coordinates".to_string(),
                },
            ],
            outputs: vec![
                TemplatePort {
                    name: "Color".to_string(),
                    port_type: PortType::Color,
                    default_value: None,
                    description: "Sampled color".to_string(),
                },
            ],
            properties: vec![
                TemplateProperty {
                    name: "Texture Path".to_string(),
                    property_type: PropertyType::String,
                    default_value: NodeProperty::String("".to_string()),
                    description: "Path to texture file".to_string(),
                    min: None,
                    max: None,
                },
            ],
            icon: "texture".to_string(),
            color: Color::srgb(0.2, 0.2, 0.8),
        });
    }

    fn initialize_validation_rules(&self) {
        self.validation_rules.push(Box::new(|graph| {
            if graph.nodes.is_empty() {
                return Err(NodeGraphError::ValidationError("Graph contains no nodes".to_string()));
            }
            Ok(())
        }));

        self.validation_rules.push(Box::new(|graph| {
            for connection in &graph.connections {
                let from_port = graph.ports.get(&connection.from)
                    .ok_or(NodeGraphError::PortNotFound(connection.from))?;
                let to_port = graph.ports.get(&connection.to)
                    .ok_or(NodeGraphError::PortNotFound(connection.to))?;

                if from_port.port_type != to_port.port_type {
                    return Err(NodeGraphError::TypeMismatch(
                        from_port.port_type.clone(),
                        to_port.port_type.clone(),
                    ));
                }
            }
            Ok(())
        }));
    }

    fn validate_connection(&self, graph: &ShaderGraph, from_port: PortId, to_port: PortId) -> NodeGraphResult<()> {
        let from_port_data = graph.ports.get(&from_port)
            .ok_or(NodeGraphError::PortNotFound(from_port))?;
        let to_port_data = graph.ports.get(&to_port)
            .ok_or(NodeGraphError::PortNotFound(to_port))?;

        if from_port_data.direction == to_port_data.direction {
            return Err(NodeGraphError::InvalidConnection(from_port, to_port));
        }

        if from_port_data.port_type != to_port_data.port_type {
            return Err(NodeGraphError::TypeMismatch(
                from_port_data.port_type.clone(),
                to_port_data.port_type.clone(),
            ));
        }

        Ok(())
    }
}

pub struct ShaderGraphCompiler {
    module_system: Arc<ShaderModuleSystem>,
    transpiler_options: TranspilerOptions,
}

impl ShaderGraphCompiler {
    pub fn new(module_system: Arc<ShaderModuleSystem>) -> Self {
        Self {
            module_system,
            transpiler_options: TranspilerOptions::default(),
        }
    }

    pub fn compile(&self, graph: &ShaderGraph) -> NodeGraphResult<String> {
        let mut wgsl_code = String::new();
        
        wgsl_code.push_str(&format!("// Generated from node graph: {}\n", graph.name));
        wgsl_code.push_str(&format!("// Graph ID: {}\n", graph.id));
        wgsl_code.push_str("// Generated by WGSL Shader Studio\n\n");

        let structs = self.generate_structs(graph)?;
        wgsl_code.push_str(&structs);

        let functions = self.generate_functions(graph)?;
        wgsl_code.push_str(&functions);

        let entry_point = self.generate_entry_point(graph)?;
        wgsl_code.push_str(&entry_point);

        Ok(wgsl_code)
    }

    fn generate_structs(&self, graph: &ShaderGraph) -> NodeGraphResult<String> {
        let mut structs = String::new();
        
        structs.push_str("struct VertexInput {\n");
        structs.push_str("    @location(0) position: vec3<f32>,\n");
        structs.push_str("    @location(1) uv: vec2<f32>,\n");
        structs.push_str("};\n\n");

        structs.push_str("struct VertexOutput {\n");
        structs.push_str("    @builtin(position) position: vec4<f32>,\n");
        structs.push_str("    @location(0) uv: vec2<f32>,\n");
        structs.push_str("};\n\n");

        Ok(structs)
    }

    fn generate_functions(&self, graph: &ShaderGraph) -> NodeGraphResult<String> {
        let mut functions = String::new();

        for (node_id, node) in &graph.nodes {
            let function = self.generate_node_function(graph, node_id, node)?;
            functions.push_str(&function);
            functions.push('\n');
        }

        Ok(functions)
    }

    fn generate_node_function(&self, graph: &ShaderGraph, node_id: &NodeId, node: &ShaderNode) -> NodeGraphResult<String> {
        let mut function = String::new();
        
        match &node.node_type {
            NodeType::Math(op) => {
                function.push_str(&format!("fn node_{}(a: f32, b: f32) -> f32 {{\n", node_id.0));
                match op {
                    MathOperation::Add => function.push_str("    return a + b;\n"),
                    MathOperation::Subtract => function.push_str("    return a - b;\n"),
                    MathOperation::Multiply => function.push_str("    return a * b;\n"),
                    MathOperation::Divide => function.push_str("    return a / b;\n"),
                    MathOperation::Sine => function.push_str("    return sin(a);\n"),
                    MathOperation::Cosine => function.push_str("    return cos(a);\n"),
                    MathOperation::Tangent => function.push_str("    return tan(a);\n"),
                    _ => function.push_str("    return a + b;\n"),
                }
                function.push_str("}\n");
            }
            NodeType::Texture => {
                function.push_str(&format!("fn node_{}(uv: vec2<f32>) -> vec4<f32> {{\n", node_id.0));
                function.push_str("    return vec4<f32>(uv.x, uv.y, 0.0, 1.0);\n");
                function.push_str("}\n");
            }
            _ => {
                function.push_str(&format!("fn node_{}() -> f32 {{\n", node_id.0));
                function.push_str("    return 0.0;\n");
                function.push_str("}\n");
            }
        }

        Ok(function)
    }

    fn generate_entry_point(&self, graph: &ShaderGraph) -> NodeGraphResult<String> {
        let mut entry_point = String::new();
        
        entry_point.push_str("@fragment\n");
        entry_point.push_str("fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n");
        entry_point.push_str("    var uv = in.uv;\n");
        entry_point.push_str("    var color = vec4<f32>(uv.x, uv.y, 0.5, 1.0);\n");
        entry_point.push_str("    return color;\n");
        entry_point.push_str("}\n");

        Ok(entry_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_graph() {
        let module_system = Arc::new(ShaderModuleSystem::new(100, std::time::Duration::from_secs(300)));
        let node_graph = ShaderNodeGraph::new(module_system);
        
        let result = node_graph.create_graph("Test Graph".to_string());
        assert!(result.is_ok());
        
        let graph_id = result.unwrap();
        assert!(!graph_id.is_nil());
    }

    #[test]
    fn test_add_node() {
        let module_system = Arc::new(ShaderModuleSystem::new(100, std::time::Duration::from_secs(300)));
        let node_graph = ShaderNodeGraph::new(module_system);
        
        let graph_id = node_graph.create_graph("Test Graph".to_string()).unwrap();
        let result = node_graph.add_node(graph_id, NodeType::Math(MathOperation::Add), Vec2::new(100.0, 100.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_graph() {
        let module_system = Arc::new(ShaderModuleSystem::new(100, std::time::Duration::from_secs(300)));
        let node_graph = ShaderNodeGraph::new(module_system);
        
        let graph_id = node_graph.create_graph("Test Graph".to_string()).unwrap();
        let result = node_graph.compile_graph(graph_id);
        assert!(result.is_ok());
        
        let wgsl_code = result.unwrap();
        assert!(wgsl_code.contains("@fragment"));
        assert!(wgsl_code.contains("fn fs_main"));
    }
}