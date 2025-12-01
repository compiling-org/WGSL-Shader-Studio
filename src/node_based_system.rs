use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use std::fmt;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePort {
    pub id: String,
    pub name: String,
    pub port_type: PortType,
    pub value: PortValue,
    pub connected: bool,
    pub connection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Texture,
    Boolean,
    Integer,
    String,
    Shader,
    Time,
    Audio,
    Midi,
    Geometry,
    Material,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color([f32; 4]),
    Texture(String),
    Boolean(bool),
    Integer(i32),
    String(String),
    Shader(String),
    Time(f32),
    Audio(Vec<f32>),
    Midi(Vec<u8>),
    Geometry(String),
    Material(String),
}

impl PortValue {
    pub fn default_for_type(port_type: &PortType) -> Self {
        match port_type {
            PortType::Float => PortValue::Float(0.0),
            PortType::Vec2 => PortValue::Vec2([0.0, 0.0]),
            PortType::Vec3 => PortValue::Vec3([0.0, 0.0, 0.0]),
            PortType::Vec4 => PortValue::Vec4([0.0, 0.0, 0.0, 0.0]),
            PortType::Color => PortValue::Color([0.0, 0.0, 0.0, 1.0]),
            PortType::Texture => PortValue::Texture("".to_string()),
            PortType::Boolean => PortValue::Boolean(false),
            PortType::Integer => PortValue::Integer(0),
            PortType::String => PortValue::String("".to_string()),
            PortType::Shader => PortValue::Shader("".to_string()),
            PortType::Time => PortValue::Time(0.0),
            PortType::Audio => PortValue::Audio(vec![]),
            PortType::Midi => PortValue::Midi(vec![]),
            PortType::Geometry => PortValue::Geometry("".to_string()),
            PortType::Material => PortValue::Material("".to_string()),
        }
    }

    pub fn to_wgsl_string(&self) -> String {
        match self {
            PortValue::Float(f) => f.to_string(),
            PortValue::Vec2(v) => format!("vec2<f32>({}, {})", v[0], v[1]),
            PortValue::Vec3(v) => format!("vec3<f32>({}, {}, {})", v[0], v[1], v[2]),
            PortValue::Vec4(v) => format!("vec4<f32>({}, {}, {}, {})", v[0], v[1], v[2], v[3]),
            PortValue::Color(c) => format!("vec4<f32>({}, {}, {}, {})", c[0], c[1], c[2], c[3]),
            PortValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            PortValue::Integer(i) => i.to_string(),
            PortValue::Time(t) => t.to_string(),
            _ => "0.0".to_string(), // Default for unsupported types
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub name: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub inputs: HashMap<String, NodePort>,
    pub outputs: HashMap<String, NodePort>,
    pub properties: HashMap<String, PropertyValue>,
    pub enabled: bool,
    pub collapsed: bool,
    pub color: [f32; 4],
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    // Input Nodes
    TimeNode,
    AudioInputNode,
    MidiInputNode,
    TextureInputNode,
    ColorInputNode,
    FloatInputNode,
    Vec2InputNode,
    Vec3InputNode,
    Vec4InputNode,
    
    // Math Nodes
    AddNode,
    SubtractNode,
    MultiplyNode,
    DivideNode,
    SineNode,
    CosineNode,
    TangentNode,
    PowerNode,
    SquareRootNode,
    LogarithmNode,
    MinimumNode,
    MaximumNode,
    ClampNode,
    MixNode,
    StepNode,
    SmoothStepNode,
    
    // Vector Nodes
    Vec2ComposeNode,
    Vec3ComposeNode,
    Vec4ComposeNode,
    Vec2DecomposeNode,
    Vec3DecomposeNode,
    Vec4DecomposeNode,
    Vec2LengthNode,
    Vec3LengthNode,
    Vec4LengthNode,
    Vec2NormalizeNode,
    Vec3NormalizeNode,
    Vec4NormalizeNode,
    DotProductNode,
    CrossProductNode,
    
    // Color Nodes
    ColorComposeNode,
    ColorDecomposeNode,
    ColorMixNode,
    ColorInvertNode,
    ColorBrightnessNode,
    ColorContrastNode,
    ColorSaturationNode,
    ColorHueNode,
    
    // Texture Nodes
    TextureSampleNode,
    TextureUVNode,
    TextureTransformNode,
    TextureNoiseNode,
    TextureGradientNode,
    TextureCheckerboardNode,
    TextureCircleNode,
    TextureRectangleNode,
    
    // Filter Nodes
    BlurNode,
    SharpenNode,
    EdgeDetectNode,
    EmbossNode,
    DistortNode,
    WarpNode,
    
    // Generator Nodes
    NoiseNode,
    FractalNoiseNode,
    CellularNoiseNode,
    ValueNoiseNode,
    GradientNoiseNode,
    VoronoiNode,
    
    // Time Nodes
    TimeSpeedNode,
    TimeOffsetNode,
    TimeLoopNode,
    TimePingPongNode,
    TimeReverseNode,
    
    // Output Nodes
    ColorOutputNode,
    FloatOutputNode,
    Vec2OutputNode,
    Vec3OutputNode,
    Vec4OutputNode,
    ShaderOutputNode,
    
    // Utility Nodes
    CommentNode,
    GroupNode,
    SwitchNode,
    RouterNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Boolean(bool),
    Integer(i32),
    String(String),
    Color([f32; 4]),
    Texture(String),
    Enum(String, Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub source_node_id: NodeId,
    pub source_port: String,
    pub target_node_id: NodeId,
    pub target_port: String,
    pub connection_type: PortType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraph {
    pub id: String,
    pub name: String,
    pub nodes: HashMap<NodeId, Node>,
    pub connections: HashMap<String, Connection>,
    pub metadata: HashMap<String, String>,
    pub viewport_position: [f32; 2],
    pub viewport_zoom: f32,
}

pub struct NodeBasedSystem {
    graphs: Arc<Mutex<HashMap<String, NodeGraph>>>,
    active_graph: Arc<Mutex<String>>,
    node_templates: Arc<Mutex<HashMap<NodeType, NodeTemplate>>>,
    execution_cache: Arc<Mutex<HashMap<String, ExecutionCache>>>,
    shader_generator: Arc<Mutex<ShaderCodeGenerator>>,
}

#[derive(Debug, Clone)]
struct NodeTemplate {
    node_type: NodeType,
    name: String,
    category: String,
    inputs: Vec<NodePort>,
    outputs: Vec<NodePort>,
    properties: Vec<PropertyDefinition>,
    color: [f32; 4],
}

#[derive(Debug, Clone)]
struct PropertyDefinition {
    name: String,
    property_type: PropertyType,
    default_value: PropertyValue,
    min_value: Option<PropertyValue>,
    max_value: Option<PropertyValue>,
    description: String,
}

#[derive(Debug, Clone)]
enum PropertyType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Boolean,
    Integer,
    String,
    Color,
    Texture,
    Enum,
}

#[derive(Debug, Clone)]
struct ExecutionCache {
    node_results: HashMap<NodeId, PortValue>,
    execution_order: Vec<NodeId>,
    last_execution_time: f32,
    cache_valid: bool,
}

pub struct ShaderCodeGenerator {
    uniform_declarations: Vec<String>,
    function_definitions: Vec<String>,
    main_function_body: Vec<String>,
    output_assignments: Vec<String>,
}

impl NodeBasedSystem {
    pub fn new() -> Self {
        let system = Self {
            graphs: Arc::new(Mutex::new(HashMap::new())),
            active_graph: Arc::new(Mutex::new("main".to_string())),
            node_templates: Arc::new(Mutex::new(HashMap::new())),
            execution_cache: Arc::new(Mutex::new(HashMap::new())),
            shader_generator: Arc::new(Mutex::new(ShaderCodeGenerator::new())),
        };

        system.initialize_node_templates();
        system
    }

    fn initialize_node_templates(&self) {
        let mut templates = self.node_templates.lock().unwrap();

        // Input Nodes
        templates.insert(NodeType::TimeNode, NodeTemplate {
            node_type: NodeType::TimeNode,
            name: "Time".to_string(),
            category: "Input".to_string(),
            inputs: vec![],
            outputs: vec![
                NodePort {
                    id: "time".to_string(),
                    name: "Time".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![],
            color: [0.2, 0.8, 0.2, 1.0],
        });

        templates.insert(NodeType::FloatInputNode, NodeTemplate {
            node_type: NodeType::FloatInputNode,
            name: "Float Input".to_string(),
            category: "Input".to_string(),
            inputs: vec![],
            outputs: vec![
                NodePort {
                    id: "value".to_string(),
                    name: "Value".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![
                PropertyDefinition {
                    name: "value".to_string(),
                    property_type: PropertyType::Float,
                    default_value: PropertyValue::Float(0.0),
                    min_value: Some(PropertyValue::Float(-1000.0)),
                    max_value: Some(PropertyValue::Float(1000.0)),
                    description: "Input value".to_string(),
                },
            ],
            color: [0.2, 0.8, 0.2, 1.0],
        });

        // Math Nodes
        templates.insert(NodeType::AddNode, NodeTemplate {
            node_type: NodeType::AddNode,
            name: "Add".to_string(),
            category: "Math".to_string(),
            inputs: vec![
                NodePort {
                    id: "a".to_string(),
                    name: "A".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "b".to_string(),
                    name: "B".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![
                NodePort {
                    id: "result".to_string(),
                    name: "Result".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![],
            color: [0.8, 0.2, 0.2, 1.0],
        });

        templates.insert(NodeType::MultiplyNode, NodeTemplate {
            node_type: NodeType::MultiplyNode,
            name: "Multiply".to_string(),
            category: "Math".to_string(),
            inputs: vec![
                NodePort {
                    id: "a".to_string(),
                    name: "A".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "b".to_string(),
                    name: "B".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![
                NodePort {
                    id: "result".to_string(),
                    name: "Result".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![],
            color: [0.8, 0.2, 0.2, 1.0],
        });

        templates.insert(NodeType::SineNode, NodeTemplate {
            node_type: NodeType::SineNode,
            name: "Sine".to_string(),
            category: "Math".to_string(),
            inputs: vec![
                NodePort {
                    id: "input".to_string(),
                    name: "Input".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![
                NodePort {
                    id: "result".to_string(),
                    name: "Result".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![],
            color: [0.8, 0.2, 0.2, 1.0],
        });

        // Vector Nodes
        templates.insert(NodeType::Vec2ComposeNode, NodeTemplate {
            node_type: NodeType::Vec2ComposeNode,
            name: "Vec2 Compose".to_string(),
            category: "Vector".to_string(),
            inputs: vec![
                NodePort {
                    id: "x".to_string(),
                    name: "X".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "y".to_string(),
                    name: "Y".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![
                NodePort {
                    id: "vector".to_string(),
                    name: "Vector".to_string(),
                    port_type: PortType::Vec2,
                    value: PortValue::Vec2([0.0, 0.0]),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![],
            color: [0.2, 0.2, 0.8, 1.0],
        });

        templates.insert(NodeType::Vec3ComposeNode, NodeTemplate {
            node_type: NodeType::Vec3ComposeNode,
            name: "Vec3 Compose".to_string(),
            category: "Vector".to_string(),
            inputs: vec![
                NodePort {
                    id: "x".to_string(),
                    name: "X".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "y".to_string(),
                    name: "Y".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "z".to_string(),
                    name: "Z".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![
                NodePort {
                    id: "vector".to_string(),
                    name: "Vector".to_string(),
                    port_type: PortType::Vec3,
                    value: PortValue::Vec3([0.0, 0.0, 0.0]),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![],
            color: [0.2, 0.2, 0.8, 1.0],
        });

        // Color Nodes
        templates.insert(NodeType::ColorComposeNode, NodeTemplate {
            node_type: NodeType::ColorComposeNode,
            name: "Color Compose".to_string(),
            category: "Color".to_string(),
            inputs: vec![
                NodePort {
                    id: "r".to_string(),
                    name: "Red".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "g".to_string(),
                    name: "Green".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "b".to_string(),
                    name: "Blue".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "a".to_string(),
                    name: "Alpha".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(1.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![
                NodePort {
                    id: "color".to_string(),
                    name: "Color".to_string(),
                    port_type: PortType::Color,
                    value: PortValue::Color([0.0, 0.0, 0.0, 1.0]),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![],
            color: [0.8, 0.8, 0.2, 1.0],
        });

        // Output Nodes
        templates.insert(NodeType::ColorOutputNode, NodeTemplate {
            node_type: NodeType::ColorOutputNode,
            name: "Color Output".to_string(),
            category: "Output".to_string(),
            inputs: vec![
                NodePort {
                    id: "color".to_string(),
                    name: "Color".to_string(),
                    port_type: PortType::Color,
                    value: PortValue::Color([0.0, 0.0, 0.0, 1.0]),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![],
            properties: vec![],
            color: [0.8, 0.2, 0.8, 1.0],
        });

        templates.insert(NodeType::FloatOutputNode, NodeTemplate {
            node_type: NodeType::FloatOutputNode,
            name: "Float Output".to_string(),
            category: "Output".to_string(),
            inputs: vec![
                NodePort {
                    id: "value".to_string(),
                    name: "Value".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![],
            properties: vec![],
            color: [0.8, 0.2, 0.8, 1.0],
        });

        // Generator Nodes
        templates.insert(NodeType::NoiseNode, NodeTemplate {
            node_type: NodeType::NoiseNode,
            name: "Noise".to_string(),
            category: "Generator".to_string(),
            inputs: vec![
                NodePort {
                    id: "position".to_string(),
                    name: "Position".to_string(),
                    port_type: PortType::Vec2,
                    value: PortValue::Vec2([0.0, 0.0]),
                    connected: false,
                    connection_id: None,
                },
                NodePort {
                    id: "scale".to_string(),
                    name: "Scale".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(1.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            outputs: vec![
                NodePort {
                    id: "value".to_string(),
                    name: "Value".to_string(),
                    port_type: PortType::Float,
                    value: PortValue::Float(0.0),
                    connected: false,
                    connection_id: None,
                },
            ],
            properties: vec![],
            color: [0.2, 0.8, 0.8, 1.0],
        });
    }

    pub fn create_graph(&self, name: &str) -> Result<(), String> {
        let mut graphs = self.graphs.lock().unwrap();
        
        if graphs.contains_key(name) {
            return Err(format!("Graph '{}' already exists", name));
        }

        let graph = NodeGraph {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            nodes: HashMap::new(),
            connections: HashMap::new(),
            metadata: HashMap::new(),
            viewport_position: [0.0, 0.0],
            viewport_zoom: 1.0,
        };

        graphs.insert(name.to_string(), graph);
        Ok(())
    }

    pub fn create_node(&self, graph_name: &str, node_type: NodeType, position: [f32; 2]) -> Result<NodeId, String> {
        let mut graphs = self.graphs.lock().unwrap();
        let templates = self.node_templates.lock().unwrap();
        
        let graph = graphs.get_mut(graph_name)
            .ok_or_else(|| format!("Graph '{}' not found", graph_name))?;
        
        let template = templates.get(&node_type)
            .ok_or_else(|| format!("Node template for {:?} not found", node_type))?;

        let node_id = NodeId::new();
        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();
        let mut properties = HashMap::new();

        // Create input ports from template
        for input_port in &template.inputs {
            inputs.insert(input_port.id.clone(), input_port.clone());
        }

        // Create output ports from template
        for output_port in &template.outputs {
            outputs.insert(output_port.id.clone(), output_port.clone());
        }

        // Create properties from template
        for prop_def in &template.properties {
            properties.insert(prop_def.name.clone(), prop_def.default_value.clone());
        }

        let node = Node {
            id: node_id.clone(),
            node_type: node_type.clone(),
            name: template.name.clone(),
            position,
            size: [200.0, 100.0],
            inputs,
            outputs,
            properties,
            enabled: true,
            collapsed: false,
            color: template.color,
            category: template.category.clone(),
        };

        graph.nodes.insert(node_id.clone(), node);
        Ok(node_id)
    }

    pub fn connect_ports(&self, graph_name: &str, source_node_id: &NodeId, source_port: &str, target_node_id: &NodeId, target_port: &str) -> Result<String, String> {
        let mut graphs = self.graphs.lock().unwrap();
        
        let graph = graphs.get_mut(graph_name)
            .ok_or_else(|| format!("Graph '{}' not found", graph_name))?;

        // Validate nodes exist
        let source_node = graph.nodes.get(source_node_id)
            .ok_or_else(|| format!("Source node '{}' not found", source_node_id))?;
        
        let target_node = graph.nodes.get(target_node_id)
            .ok_or_else(|| format!("Target node '{}' not found", target_node_id))?;

        // Validate ports exist and types match
        let source_output = source_node.outputs.get(source_port)
            .ok_or_else(|| format!("Output port '{}' not found in source node", source_port))?;
        
        let target_input = target_node.inputs.get(target_port)
            .ok_or_else(|| format!("Input port '{}' not found in target node", target_port))?;

        if source_output.port_type != target_input.port_type {
            return Err(format!("Port type mismatch: {} vs {}", 
                format!("{:?}", source_output.port_type), 
                format!("{:?}", target_input.port_type)));
        }

        // Create connection
        let connection_id = Uuid::new_v4().to_string();
        let connection = Connection {
            id: connection_id.clone(),
            source_node_id: source_node_id.clone(),
            source_port: source_port.to_string(),
            target_node_id: target_node_id.clone(),
            target_port: target_port.to_string(),
            connection_type: source_output.port_type.clone(),
        };

        graph.connections.insert(connection_id.clone(), connection);

        // Update port connection status
        if let Some(source_node) = graph.nodes.get_mut(source_node_id) {
            if let Some(output_port) = source_node.outputs.get_mut(source_port) {
                output_port.connected = true;
                output_port.connection_id = Some(connection_id.clone());
            }
        }

        if let Some(target_node) = graph.nodes.get_mut(target_node_id) {
            if let Some(input_port) = target_node.inputs.get_mut(target_port) {
                input_port.connected = true;
                input_port.connection_id = Some(connection_id.clone());
            }
        }

        // Invalidate execution cache
        let mut cache = self.execution_cache.lock().unwrap();
        cache.remove(graph_name);

        Ok(connection_id)
    }

    pub fn disconnect_ports(&self, graph_name: &str, connection_id: &str) -> Result<(), String> {
        let mut graphs = self.graphs.lock().unwrap();
        
        let graph = graphs.get_mut(graph_name)
            .ok_or_else(|| format!("Graph '{}' not found", graph_name))?;

        let connection = graph.connections.remove(connection_id)
            .ok_or_else(|| format!("Connection '{}' not found", connection_id))?;

        // Update port connection status
        if let Some(source_node) = graph.nodes.get_mut(&connection.source_node_id) {
            if let Some(output_port) = source_node.outputs.get_mut(&connection.source_port) {
                output_port.connected = false;
                output_port.connection_id = None;
            }
        }

        if let Some(target_node) = graph.nodes.get_mut(&connection.target_node_id) {
            if let Some(input_port) = target_node.inputs.get_mut(&connection.target_port) {
                input_port.connected = false;
                input_port.connection_id = None;
            }
        }

        // Invalidate execution cache
        let mut cache = self.execution_cache.lock().unwrap();
        cache.remove(graph_name);

        Ok(())
    }

    pub fn update_node_property(&self, graph_name: &str, node_id: &NodeId, property_name: &str, value: PropertyValue) -> Result<(), String> {
        let mut graphs = self.graphs.lock().unwrap();
        
        let graph = graphs.get_mut(graph_name)
            .ok_or_else(|| format!("Graph '{}' not found", graph_name))?;

        let node = graph.nodes.get_mut(node_id)
            .ok_or_else(|| format!("Node '{}' not found", node_id))?;

        node.properties.insert(property_name.to_string(), value);

        // Invalidate execution cache
        let mut cache = self.execution_cache.lock().unwrap();
        cache.remove(graph_name);

        Ok(())
    }

    pub fn execute_graph(&self, graph_name: &str, time: f32, audio_data: &[f32], midi_data: &[u8]) -> Result<ExecutionResult, String> {
        let mut graphs = self.graphs.lock().unwrap();
        let mut cache = self.execution_cache.lock().unwrap();
        
        let graph = graphs.get_mut(graph_name)
            .ok_or_else(|| format!("Graph '{}' not found", graph_name))?;

        // Check if we have a valid cache
        if let Some(cached_result) = cache.get(graph_name) {
            if cached_result.cache_valid && (time - cached_result.last_execution_time).abs() < 0.001 {
                return Ok(ExecutionResult {
                    node_results: cached_result.node_results.clone(),
                    output_values: self.collect_output_values(graph, &cached_result.node_results)?,
                    execution_time: time,
                });
            }
        }

        // Build execution order (topological sort)
        let execution_order = self.build_execution_order(graph)?;

        // Execute nodes in order
        let mut node_results = HashMap::new();
        
        for node_id in &execution_order {
            let node_result = self.execute_node(graph, node_id, time, audio_data, midi_data, &node_results)?;
            node_results.insert(node_id.clone(), node_result);
        }

        // Collect output values
        let output_values = self.collect_output_values(graph, &node_results)?;

        // Update cache
        let execution_cache = ExecutionCache {
            node_results: node_results.clone(),
            execution_order,
            last_execution_time: time,
            cache_valid: true,
        };
        
        cache.insert(graph_name.to_string(), execution_cache);

        Ok(ExecutionResult {
            node_results,
            output_values,
            execution_time: time,
        })
    }

    fn build_execution_order(&self, graph: &NodeGraph) -> Result<Vec<NodeId>, String> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut execution_order = Vec::new();

        for node_id in graph.nodes.keys() {
            if !visited.contains(node_id) {
                self.topological_sort(graph, node_id, &mut visited, &mut temp_visited, &mut execution_order)?;
            }
        }

        execution_order.reverse();
        Ok(execution_order)
    }

    fn topological_sort(&self, graph: &NodeGraph, node_id: &NodeId, visited: &mut HashSet<NodeId>, 
                       temp_visited: &mut HashSet<NodeId>, execution_order: &mut Vec<NodeId>) -> Result<(), String> {
        
        if temp_visited.contains(node_id) {
            return Err("Circular dependency detected in node graph".to_string());
        }

        if visited.contains(node_id) {
            return Ok(());
        }

        temp_visited.insert(node_id.clone());

        // Visit all dependent nodes first (nodes that this node connects to)
        for connection in graph.connections.values() {
            if connection.source_node_id == *node_id {
                self.topological_sort(graph, &connection.target_node_id, visited, temp_visited, execution_order)?;
            }
        }

        temp_visited.remove(node_id);
        visited.insert(node_id.clone());
        execution_order.push(node_id.clone());

        Ok(())
    }

    fn execute_node(&self, graph: &NodeGraph, node_id: &NodeId, time: f32, audio_data: &[f32], 
                   midi_data: &[u8], node_results: &HashMap<NodeId, PortValue>) -> Result<PortValue, String> {
        
        let node = graph.nodes.get(node_id)
            .ok_or_else(|| format!("Node '{}' not found", node_id))?;

        if !node.enabled {
            return Ok(PortValue::Float(0.0));
        }

        match &node.node_type {
            NodeType::TimeNode => Ok(PortValue::Float(time)),
            
            NodeType::FloatInputNode => {
                if let Some(PropertyValue::Float(value)) = node.properties.get("value") {
                    Ok(PortValue::Float(*value))
                } else {
                    Ok(PortValue::Float(0.0))
                }
            },
            
            NodeType::AddNode => {
                let a = self.get_input_value(graph, node, "a", node_results)?;
                let b = self.get_input_value(graph, node, "b", node_results)?;
                
                match (a, b) {
                    (PortValue::Float(a_val), PortValue::Float(b_val)) => {
                        Ok(PortValue::Float(a_val + b_val))
                    },
                    _ => Err("Add node requires float inputs".to_string()),
                }
            },
            
            NodeType::MultiplyNode => {
                let a = self.get_input_value(graph, node, "a", node_results)?;
                let b = self.get_input_value(graph, node, "b", node_results)?;
                
                match (a, b) {
                    (PortValue::Float(a_val), PortValue::Float(b_val)) => {
                        Ok(PortValue::Float(a_val * b_val))
                    },
                    _ => Err("Multiply node requires float inputs".to_string()),
                }
            },
            
            NodeType::SineNode => {
                let input = self.get_input_value(graph, node, "input", node_results)?;
                
                match input {
                    PortValue::Float(val) => Ok(PortValue::Float(val.sin())),
                    _ => Err("Sine node requires float input".to_string()),
                }
            },
            
            NodeType::Vec2ComposeNode => {
                let x = self.get_input_value(graph, node, "x", node_results)?;
                let y = self.get_input_value(graph, node, "y", node_results)?;
                
                match (x, y) {
                    (PortValue::Float(x_val), PortValue::Float(y_val)) => {
                        Ok(PortValue::Vec2([x_val, y_val]))
                    },
                    _ => Err("Vec2Compose node requires float inputs".to_string()),
                }
            },
            
            NodeType::Vec3ComposeNode => {
                let x = self.get_input_value(graph, node, "x", node_results)?;
                let y = self.get_input_value(graph, node, "y", node_results)?;
                let z = self.get_input_value(graph, node, "z", node_results)?;
                
                match (x, y, z) {
                    (PortValue::Float(x_val), PortValue::Float(y_val), PortValue::Float(z_val)) => {
                        Ok(PortValue::Vec3([x_val, y_val, z_val]))
                    },
                    _ => Err("Vec3Compose node requires float inputs".to_string()),
                }
            },
            
            NodeType::ColorComposeNode => {
                let r = self.get_input_value(graph, node, "r", node_results)?;
                let g = self.get_input_value(graph, node, "g", node_results)?;
                let b = self.get_input_value(graph, node, "b", node_results)?;
                let a = self.get_input_value(graph, node, "a", node_results)?;
                
                match (r, g, b, a) {
                    (PortValue::Float(r_val), PortValue::Float(g_val), PortValue::Float(b_val), PortValue::Float(a_val)) => {
                        Ok(PortValue::Color([r_val, g_val, b_val, a_val]))
                    },
                    _ => Err("ColorCompose node requires float inputs".to_string()),
                }
            },
            
            NodeType::NoiseNode => {
                let position = self.get_input_value(graph, node, "position", node_results)?;
                let scale = self.get_input_value(graph, node, "scale", node_results)?;
                
                match (position, scale) {
                    (PortValue::Vec2(pos), PortValue::Float(scale_val)) => {
                        let noise_value = self.generate_simple_noise(pos[0] * scale_val, pos[1] * scale_val);
                        Ok(PortValue::Float(noise_value))
                    },
                    _ => Err("Noise node requires vec2 position and float scale".to_string()),
                }
            },
            
            _ => Ok(PortValue::Float(0.0)), // Default for unimplemented nodes
        }
    }

    fn get_input_value(&self, graph: &NodeGraph, node: &Node, input_name: &str, 
                      node_results: &HashMap<NodeId, PortValue>) -> Result<PortValue, String> {
        
        // Check if input port is connected
        if let Some(input_port) = node.inputs.get(input_name) {
            if input_port.connected {
                // Find the connection that feeds into this port
                for connection in graph.connections.values() {
                    if connection.target_node_id == node.id && connection.target_port == input_name {
                        // Get the value from the source node
                        if let Some(source_result) = node_results.get(&connection.source_node_id) {
                            return Ok(source_result.clone());
                        }
                    }
                }
            }
            
            // Return the port's default value if not connected
            Ok(input_port.value.clone())
        } else {
            Err(format!("Input port '{}' not found", input_name))
        }
    }

    fn collect_output_values(&self, graph: &NodeGraph, node_results: &HashMap<NodeId, PortValue>) -> Result<HashMap<String, PortValue>, String> {
        let mut output_values = HashMap::new();

        for node in graph.nodes.values() {
            match &node.node_type {
                NodeType::ColorOutputNode | NodeType::FloatOutputNode | NodeType::Vec2OutputNode | 
                NodeType::Vec3OutputNode | NodeType::Vec4OutputNode => {
                    
                    // Find the input connection to this output node
                    for connection in graph.connections.values() {
                        if connection.target_node_id == node.id {
                            if let Some(input_value) = node_results.get(&connection.source_node_id) {
                                output_values.insert(node.name.clone(), input_value.clone());
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        Ok(output_values)
    }

    fn generate_simple_noise(&self, x: f32, y: f32) -> f32 {
        // Simple 2D noise function
        let n = x * 12.9898 + y * 78.233;
        let sin_n = (n * 43758.5453).sin();
        let fract_part = sin_n - sin_n.floor();
        fract_part * 2.0 - 1.0 // Convert to [-1, 1] range
    }

    pub fn generate_shader_code(&self, graph_name: &str) -> Result<String, String> {
        let graphs = self.graphs.lock().unwrap();
        let graph = graphs.get(graph_name)
            .ok_or_else(|| format!("Graph '{}' not found", graph_name))?;

        let mut shader_generator = self.shader_generator.lock().unwrap();
        shader_generator.clear();

        // Generate shader code from the node graph
        shader_generator.generate_from_graph(graph)?;
        
        Ok(shader_generator.get_complete_shader())
    }

    pub fn export_graph(&self, graph_name: &str) -> Result<String, String> {
        let graphs = self.graphs.lock().unwrap();
        let graph = graphs.get(graph_name)
            .ok_or_else(|| format!("Graph '{}' not found", graph_name))?;

        serde_json::to_string_pretty(graph)
            .map_err(|e| format!("Failed to serialize graph: {}", e))
    }

    pub fn import_graph(&self, json_data: &str) -> Result<String, String> {
        let graph: NodeGraph = serde_json::from_str(json_data)
            .map_err(|e| format!("Failed to deserialize graph: {}", e))?;
        
        let graph_name = graph.name.clone();
        self.graphs.lock().unwrap().insert(graph_name.clone(), graph);
        Ok(graph_name)
    }

    pub fn get_node_templates(&self) -> Vec<(String, String, Vec<String>, Vec<String>)> {
        let templates = self.node_templates.lock().unwrap();
        
        templates.iter().map(|(node_type, template)| {
            let input_names: Vec<String> = template.inputs.iter().map(|p| p.name.clone()).collect();
            let output_names: Vec<String> = template.outputs.iter().map(|p| p.name.clone()).collect();
            
            (template.name.clone(), template.category.clone(), input_names, output_names)
        }).collect()
    }
}

pub struct ExecutionResult {
    pub node_results: HashMap<NodeId, PortValue>,
    pub output_values: HashMap<String, PortValue>,
    pub execution_time: f32,
}

impl ShaderCodeGenerator {
    fn new() -> Self {
        Self {
            uniform_declarations: Vec::new(),
            function_definitions: Vec::new(),
            main_function_body: Vec::new(),
            output_assignments: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.uniform_declarations.clear();
        self.function_definitions.clear();
        self.main_function_body.clear();
        self.output_assignments.clear();
    }

    fn generate_from_graph(&mut self, graph: &NodeGraph) -> Result<(), String> {
        // Generate WGSL shader code from the node graph
        self.generate_uniforms(graph)?;
        self.generate_functions(graph)?;
        self.generate_main_function(graph)?;
        Ok(())
    }

    fn generate_uniforms(&mut self, graph: &NodeGraph) -> Result<(), String> {
        // Generate uniform declarations for time, resolution, etc.
        self.uniform_declarations.push("@group(0) @binding(0) var<uniform> time: f32;".to_string());
        self.uniform_declarations.push("@group(0) @binding(1) var<uniform> resolution: vec2<f32>;".to_string());
        self.uniform_declarations.push("@group(0) @binding(2) var<uniform> mouse: vec2<f32>;".to_string());
        
        // Generate uniforms for input nodes
        for node in graph.nodes.values() {
            match &node.node_type {
                NodeType::FloatInputNode => {
                    if let Some(PropertyValue::Float(value)) = node.properties.get("value") {
                        self.uniform_declarations.push(
                            format!("@group(1) @binding({}) var<uniform> {}: f32;", 
                                   self.uniform_declarations.len(), node.name.to_lowercase().replace(" ", "_"))
                        );
                    }
                },
                _ => {}
            }
        }
        
        Ok(())
    }

    fn generate_functions(&mut self, graph: &NodeGraph) -> Result<(), String> {
        // Generate helper functions
        self.function_definitions.push("""
fn noise_2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = dot(i, vec2<f32>(12.9898, 78.233));
    let b = dot(i + vec2<f32>(1.0, 0.0), vec2<f32>(12.9898, 78.233));
    let c = dot(i + vec2<f32>(0.0, 1.0), vec2<f32>(12.9898, 78.233));
    let d = dot(i + vec2<f32>(1.0, 1.0), vec2<f32>(12.9898, 78.233));
    
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(fract(sin(a) * 43758.5453), fract(sin(b) * 43758.5453), u.x),
               mix(fract(sin(c) * 43758.5453), fract(sin(d) * 43758.5453), u.x), u.y);
}
"".to_string());

        Ok(())
    }

    fn generate_main_function(&mut self, graph: &NodeGraph) -> Result<(), String> {
        self.main_function_body.push("var uv = (vec2<f32>(vertex_index % 2u, vertex_index / 2u) * 2.0 - 1.0);".to_string());
        self.main_function_body.push("var position = uv * vec2<f32>(1.0, -1.0);".to_string());
        
        // Generate node execution code
        for node in graph.nodes.values() {
            self.generate_node_execution(node)?;
        }
        
        // Generate output assignments
        for node in graph.nodes.values() {
            match &node.node_type {
                NodeType::ColorOutputNode => {
                    self.output_assignments.push("frag_color = vec4<f32>(color_output, 1.0);".to_string());
                },
                NodeType::FloatOutputNode => {
                    self.output_assignments.push("frag_color = vec4<f32>(float_output, float_output, float_output, 1.0);".to_string());
                },
                _ => {}
            }
        }
        
        Ok(())
    }

    fn generate_node_execution(&mut self, node: &Node) -> Result<(), String> {
        let node_var = node.name.to_lowercase().replace(' ', "_");
        
        match &node.node_type {
            NodeType::TimeNode => {
                self.main_function_body.push(format!("let {} = time;", node_var));
            },
            
            NodeType::FloatInputNode => {
                if let Some(PropertyValue::Float(_value)) = node.properties.get("value") {
                    self.main_function_body.push(format!("let {} = {};", node_var, node_var));
                }
            },
            
            NodeType::AddNode => {
                self.main_function_body.push(format!("let {} = a + b;", node_var));
            },
            
            NodeType::MultiplyNode => {
                self.main_function_body.push(format!("let {} = a * b;", node_var));
            },
            
            NodeType::SineNode => {
                self.main_function_body.push(format!("let {} = sin(input);", node_var));
            },
            
            NodeType::Vec2ComposeNode => {
                self.main_function_body.push(format!("let {} = vec2<f32>(x, y);", node_var));
            },
            
            NodeType::Vec3ComposeNode => {
                self.main_function_body.push(format!("let {} = vec3<f32>(x, y, z);", node_var));
            },
            
            NodeType::ColorComposeNode => {
                self.main_function_body.push(format!("let {} = vec4<f32>(r, g, b, a);", node_var));
            },
            
            NodeType::NoiseNode => {
                self.main_function_body.push(format!("let {} = noise_2d(position * scale);", node_var));
            },
            
            _ => {}
        }
        
        Ok(())
    }

    fn get_complete_shader(&self) -> String {
        format!(r#"
struct VertexOutput {{
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}}

struct FragmentOutput {{
    @location(0) color: vec4<f32>,
}}

{}

{}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {{
    var output: VertexOutput;
    {}
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = uv;
    return output;
}}

@fragment
fn fragment_main(input: VertexOutput) -> FragmentOutput {{
    var frag_color: vec4<f32>;
    {}
    return FragmentOutput {{ color: frag_color }};
}}
"#,
            self.uniform_declarations.join("\n"),
            self.function_definitions.join("\n"),
            self.main_function_body.join("\n    "),
            self.output_assignments.join("\n    ")
        )
    }
}

// Utility functions for creating common node graphs
pub fn create_simple_color_graph() -> NodeGraph {
    let mut graph = NodeGraph {
        id: Uuid::new_v4().to_string(),
        name: "Simple Color".to_string(),
        nodes: HashMap::new(),
        connections: HashMap::new(),
        metadata: HashMap::new(),
        viewport_position: [0.0, 0.0],
        viewport_zoom: 1.0,
    };

    graph
}

pub fn create_audio_reactive_graph() -> NodeGraph {
    let mut graph = NodeGraph {
        id: Uuid::new_v4().to_string(),
        name: "Audio Reactive".to_string(),
        nodes: HashMap::new(),
        connections: HashMap::new(),
        metadata: HashMap::new(),
        viewport_position: [0.0, 0.0],
        viewport_zoom: 1.0,
    };

    graph
}

pub fn create_fractal_noise_graph() -> NodeGraph {
    let mut graph = NodeGraph {
        id: Uuid::new_v4().to_string(),
        name: "Fractal Noise".to_string(),
        nodes: HashMap::new(),
        connections: HashMap::new(),
        metadata: HashMap::new(),
        viewport_position: [0.0, 0.0],
        viewport_zoom: 1.0,
    };

    graph
}