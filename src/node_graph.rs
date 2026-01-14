use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortKind {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Texture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    // Constants
    ConstantFloat(f32),
    ConstantVec2([f32; 2]),
    ConstantVec3([f32; 3]),
    ConstantVec4([f32; 4]),
    
    // Input/Time
    Time,
    UV,
    Param(usize),
    Resolution,
    Mouse,
    
    // Math Operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Sine,
    Cosine,
    Tangent,
    Length,
    Normalize,
    Distance,
    
    // Vector Operations
    Dot,
    Cross,
    Reflect,
    Refract,
    
    // Interpolation & Utility
    Mix,
    Clamp,
    Step,
    Smoothstep,
    Fract,
    Floor,
    Ceil,
    Abs,
    Min,
    Max,
    Pow,
    Sqrt,
    Sign,
    
    // Color Operations
    RGB,
    HSV,
    ColorMix,
    ColorAdjust,
    
    // Noise & Procedural
    Noise2D,
    Noise3D,
    Voronoi,
    
    // Texture Operations
    TextureSample,
    TextureSampleLod,
    TextureSize,
    
    // Output
    OutputColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub kind: PortKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub title: String,
    pub kind: NodeKind,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub pos: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_node: NodeId,
    pub from_port: PortId,
    pub to_node: NodeId,
    pub to_port: PortId,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub connections: Vec<Connection>,
    next_node_id: u32,
    next_port_id: u32,
}

impl NodeGraph {
    pub fn new() -> Self { Self::default() }

    pub fn add_node(&mut self, kind: NodeKind, title: &str, pos: (f32, f32)) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        let mut node = Node {
            id,
            title: title.to_string(),
            kind: kind.clone(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            pos,
        };
        self.populate_ports(&mut node);
        self.nodes.insert(id, node);
        id
    }

    pub fn connect(&mut self, from_node: NodeId, from_port: PortId, to_node: NodeId, to_port: PortId) {
        self.connections.push(Connection { from_node, from_port, to_node, to_port });
    }

    fn new_port(&mut self, name: &str, kind: PortKind) -> Port {
        let id = PortId(self.next_port_id);
        self.next_port_id += 1;
        Port { id, name: name.to_string(), kind }
    }

    fn populate_ports(&mut self, node: &mut Node) {
        match node.kind {
            // Constants
            NodeKind::ConstantFloat(_) => {
                node.outputs.push(self.new_port("value", PortKind::Float));
            }
            NodeKind::ConstantVec2(_) => {
                node.outputs.push(self.new_port("value", PortKind::Vec2));
            }
            NodeKind::ConstantVec3(_) => {
                node.outputs.push(self.new_port("value", PortKind::Vec3));
            }
            NodeKind::ConstantVec4(_) => {
                node.outputs.push(self.new_port("value", PortKind::Vec4));
            }
            
            // Input/Time
            NodeKind::Time => {
                node.outputs.push(self.new_port("time", PortKind::Float));
            }
            NodeKind::UV => {
                node.outputs.push(self.new_port("uv", PortKind::Vec2));
            }
            NodeKind::Param(_) => {
                node.outputs.push(self.new_port("value", PortKind::Float));
            }
            NodeKind::Resolution => {
                node.outputs.push(self.new_port("resolution", PortKind::Vec2));
            }
            NodeKind::Mouse => {
                node.outputs.push(self.new_port("mouse", PortKind::Vec2));
            }
            
            // Math Operations
            NodeKind::Add | NodeKind::Subtract | NodeKind::Multiply | NodeKind::Divide | NodeKind::Min | NodeKind::Max | NodeKind::Pow | NodeKind::Distance => {
                node.inputs.push(self.new_port("a", PortKind::Float));
                node.inputs.push(self.new_port("b", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            
            // Unary math operations
            NodeKind::Sine | NodeKind::Cosine | NodeKind::Tangent | NodeKind::Length | NodeKind::Fract | NodeKind::Floor | NodeKind::Ceil | NodeKind::Abs | NodeKind::Sqrt | NodeKind::Sign => {
                node.inputs.push(self.new_port("x", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            
            // Vector operations
            NodeKind::Normalize => {
                node.inputs.push(self.new_port("vector", PortKind::Vec3));
                node.outputs.push(self.new_port("out", PortKind::Vec3));
            }
            NodeKind::Dot => {
                node.inputs.push(self.new_port("a", PortKind::Vec3));
                node.inputs.push(self.new_port("b", PortKind::Vec3));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            NodeKind::Cross => {
                node.inputs.push(self.new_port("a", PortKind::Vec3));
                node.inputs.push(self.new_port("b", PortKind::Vec3));
                node.outputs.push(self.new_port("out", PortKind::Vec3));
            }
            NodeKind::Reflect => {
                node.inputs.push(self.new_port("incident", PortKind::Vec3));
                node.inputs.push(self.new_port("normal", PortKind::Vec3));
                node.outputs.push(self.new_port("out", PortKind::Vec3));
            }
            NodeKind::Refract => {
                node.inputs.push(self.new_port("incident", PortKind::Vec3));
                node.inputs.push(self.new_port("normal", PortKind::Vec3));
                node.inputs.push(self.new_port("eta", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Vec3));
            }
            
            // Interpolation
            NodeKind::Mix => {
                node.inputs.push(self.new_port("a", PortKind::Float));
                node.inputs.push(self.new_port("b", PortKind::Float));
                node.inputs.push(self.new_port("t", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            NodeKind::Step => {
                node.inputs.push(self.new_port("edge", PortKind::Float));
                node.inputs.push(self.new_port("x", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            NodeKind::Smoothstep => {
                node.inputs.push(self.new_port("edge0", PortKind::Float));
                node.inputs.push(self.new_port("edge1", PortKind::Float));
                node.inputs.push(self.new_port("x", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            NodeKind::Clamp => {
                node.inputs.push(self.new_port("x", PortKind::Float));
                node.inputs.push(self.new_port("min", PortKind::Float));
                node.inputs.push(self.new_port("max", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            
            // Color Operations
            NodeKind::RGB => {
                node.inputs.push(self.new_port("r", PortKind::Float));
                node.inputs.push(self.new_port("g", PortKind::Float));
                node.inputs.push(self.new_port("b", PortKind::Float));
                node.outputs.push(self.new_port("color", PortKind::Color));
            }
            NodeKind::HSV => {
                node.inputs.push(self.new_port("h", PortKind::Float));
                node.inputs.push(self.new_port("s", PortKind::Float));
                node.inputs.push(self.new_port("v", PortKind::Float));
                node.outputs.push(self.new_port("color", PortKind::Color));
            }
            NodeKind::ColorMix => {
                node.inputs.push(self.new_port("color1", PortKind::Color));
                node.inputs.push(self.new_port("color2", PortKind::Color));
                node.inputs.push(self.new_port("t", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Color));
            }
            NodeKind::ColorAdjust => {
                node.inputs.push(self.new_port("color", PortKind::Color));
                node.inputs.push(self.new_port("brightness", PortKind::Float));
                node.inputs.push(self.new_port("contrast", PortKind::Float));
                node.inputs.push(self.new_port("saturation", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Color));
            }
            
            // Noise & Procedural
            NodeKind::Noise2D => {
                node.inputs.push(self.new_port("position", PortKind::Vec2));
                node.outputs.push(self.new_port("value", PortKind::Float));
            }
            NodeKind::Noise3D => {
                node.inputs.push(self.new_port("position", PortKind::Vec3));
                node.outputs.push(self.new_port("value", PortKind::Float));
            }
            NodeKind::Voronoi => {
                node.inputs.push(self.new_port("position", PortKind::Vec2));
                node.outputs.push(self.new_port("value", PortKind::Float));
                node.outputs.push(self.new_port("cell_id", PortKind::Float));
            }
            
            // Texture operations
            NodeKind::TextureSample => {
                node.inputs.push(self.new_port("tex", PortKind::Texture));
                node.inputs.push(self.new_port("uv", PortKind::Vec2));
                node.outputs.push(self.new_port("color", PortKind::Color));
            }
            NodeKind::TextureSampleLod => {
                node.inputs.push(self.new_port("tex", PortKind::Texture));
                node.inputs.push(self.new_port("uv", PortKind::Vec2));
                node.inputs.push(self.new_port("lod", PortKind::Float));
                node.outputs.push(self.new_port("color", PortKind::Color));
            }
            NodeKind::TextureSize => {
                node.inputs.push(self.new_port("tex", PortKind::Texture));
                node.outputs.push(self.new_port("size", PortKind::Vec2));
            }
            
            // Output
            NodeKind::OutputColor => {
                node.inputs.push(self.new_port("color", PortKind::Color));
            }
        }
    }

    /// Generate WGSL code from the current node graph. Produces a minimal shader
    /// with `@vertex` and `@fragment` entry points and a uniform block including time/resolution.
pub fn generate_wgsl(&self, _width: u32, _height: u32) -> String {
        let mut code = String::new();
        code.push_str("struct Uniforms {\n  time: f32,\n  resolution: vec2<f32>,\n};\n\n");
        code.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");
        // Secondary uniform buffer for named parameters (64 floats packed into 16 vec4s for std140)
        code.push_str("@group(0) @binding(1) var<uniform> params: array<vec4<f32>, 16>;\n\n");
        // Vertex shader: full-screen triangle
        code.push_str("@vertex\nfn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {\n  var positions = array<vec2<f32>, 3>(\n    vec2<f32>(-1.0, -3.0),\n    vec2<f32>(-1.0,  1.0),\n    vec2<f32>( 3.0,  1.0),\n  );\n  let pos = positions[i];\n  return vec4<f32>(pos, 0.0, 1.0);\n}\n\n");

        // Fragment prelude: declare any textures if present
        let mut uses_texture = false;
        for n in self.nodes.values() {
            if matches!(n.kind, NodeKind::TextureSample) {
                uses_texture = true;
                break;
            }
        }
        if uses_texture {
            // Use non-conflicting bindings: params occupy binding(1)
            code.push_str("@group(0) @binding(2) var samp: sampler;\n@group(0) @binding(3) var tex0: texture_2d<f32>;\n\n");
        }

        // Build evaluation order (simple topological by repeatedly selecting nodes whose inputs are satisfied)
        let mut order: Vec<NodeId> = Vec::new();
        let mut satisfied: HashSet<NodeId> = HashSet::new();
        // Source nodes have no inputs
        for (id, node) in &self.nodes {
            if node.inputs.is_empty() { satisfied.insert(*id); order.push(*id); }
        }
        // Iterate to include remaining nodes
        let mut remaining: HashSet<NodeId> = self.nodes.keys().copied().collect();
        for id in order.iter() { remaining.remove(id); }
        let mut guard = 0;
        while !remaining.is_empty() && guard < 1024 {
            guard += 1;
            let mut progressed = false;
            for id in remaining.clone() {
                let node = &self.nodes[&id];
                let mut all_inputs_satisfied = true;
                for inp in &node.inputs {
                    let has_input = self.connections.iter().any(|c| c.to_node == id && c.to_port == inp.id && satisfied.contains(&c.from_node));
                    if !has_input { all_inputs_satisfied = false; break; }
                }
                if all_inputs_satisfied {
                    satisfied.insert(id);
                    order.push(id);
                    remaining.remove(&id);
                    progressed = true;
                }
            }
            if !progressed { break; }
        }

        // Temporary mapping: for each port, a WGSL variable name
        let mut port_vars: HashMap<(NodeId, PortId), String> = HashMap::new();
        let mut var_counter = 0u32;

        code.push_str("@fragment\nfn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {\n");
        code.push_str("  let uv = pos.xy / uniforms.resolution;\n");

        for id in order.iter() {
            let node = &self.nodes[id];
            match node.kind {
                NodeKind::ConstantFloat(f) => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = {f};\n"));
                }
                NodeKind::ConstantVec2(v) => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec2<f32> = vec2<f32>({},{});\n", v[0], v[1]));
                }
                NodeKind::ConstantVec3(v) => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = vec3<f32>({},{},{});\n", v[0], v[1], v[2]));
                }
                NodeKind::ConstantVec4(v) => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec4<f32> = vec4<f32>({},{},{},{});\n", v[0], v[1], v[2], v[3]));
                }
                NodeKind::Time => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = uniforms.time;\n"));
                }
                NodeKind::UV => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec2<f32> = uv;\n"));
                }
                NodeKind::Param(idx) => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    let vec_index = idx / 4;
                    let comp = match idx % 4 { 0 => "x", 1 => "y", 2 => "z", _ => "w" };
                    code.push_str(&format!("  let {var}: f32 = params[{vec_index}].{comp};\n"));
                }
                // Binary math operations
                NodeKind::Add | NodeKind::Subtract | NodeKind::Multiply | NodeKind::Divide | NodeKind::Min | NodeKind::Max | NodeKind::Pow | NodeKind::Distance => {
                    let a = &node.inputs[0];
                    let b = &node.inputs[1];
                    let a_src = self.find_source_var(*id, a.id, &port_vars);
                    let b_src = self.find_source_var(*id, b.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    let op = match node.kind {
                        NodeKind::Add => "+",
                        NodeKind::Subtract => "-",
                        NodeKind::Multiply => "*",
                        NodeKind::Divide => "/",
                        NodeKind::Min => "min",
                        NodeKind::Max => "max",
                        NodeKind::Pow => "pow",
                        NodeKind::Distance => "distance",
                        _ => "+",
                    };
                    if matches!(node.kind, NodeKind::Min | NodeKind::Max | NodeKind::Pow | NodeKind::Distance) {
                        code.push_str(&format!("  let {var}: f32 = {op}({a_src}, {b_src});\n"));
                    } else {
                        code.push_str(&format!("  let {var}: f32 = {a_src} {op} {b_src};\n"));
                    }
                }
                // Unary math operations
                NodeKind::Sine | NodeKind::Cosine | NodeKind::Tangent | NodeKind::Length | NodeKind::Fract | NodeKind::Floor | NodeKind::Ceil | NodeKind::Abs | NodeKind::Sqrt => {
                    let x = &node.inputs[0];
                    let x_src = self.find_source_var(*id, x.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    let func = match node.kind {
                        NodeKind::Sine => "sin",
                        NodeKind::Cosine => "cos",
                        NodeKind::Tangent => "tan",
                        NodeKind::Length => "length",
                        NodeKind::Fract => "fract",
                        NodeKind::Floor => "floor",
                        NodeKind::Ceil => "ceil",
                        NodeKind::Abs => "abs",
                        NodeKind::Sqrt => "sqrt",
                        _ => "sin",
                    };
                    code.push_str(&format!("  let {var}: f32 = {func}({x_src});\n"));
                }
                // Vector operations
                NodeKind::Normalize => {
                    let vec = &node.inputs[0];
                    let vec_src = self.find_source_var(*id, vec.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = normalize({vec_src});\n"));
                }
                NodeKind::Dot => {
                    let a = &node.inputs[0];
                    let b = &node.inputs[1];
                    let a_src = self.find_source_var(*id, a.id, &port_vars);
                    let b_src = self.find_source_var(*id, b.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = dot({a_src}, {b_src});\n"));
                }
                NodeKind::Cross => {
                    let a = &node.inputs[0];
                    let b = &node.inputs[1];
                    let a_src = self.find_source_var(*id, a.id, &port_vars);
                    let b_src = self.find_source_var(*id, b.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = cross({a_src}, {b_src});\n"));
                }
                // Interpolation
                NodeKind::Mix => {
                    let a = &node.inputs[0];
                    let b = &node.inputs[1];
                    let t = &node.inputs[2];
                    let a_src = self.find_source_var(*id, a.id, &port_vars);
                    let b_src = self.find_source_var(*id, b.id, &port_vars);
                    let t_src = self.find_source_var(*id, t.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = mix({a_src}, {b_src}, {t_src});\n"));
                }
                NodeKind::Step => {
                    let edge = &node.inputs[0];
                    let x = &node.inputs[1];
                    let edge_src = self.find_source_var(*id, edge.id, &port_vars);
                    let x_src = self.find_source_var(*id, x.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = step({edge_src}, {x_src});\n"));
                }
                NodeKind::Smoothstep => {
                    let edge0 = &node.inputs[0];
                    let edge1 = &node.inputs[1];
                    let x = &node.inputs[2];
                    let edge0_src = self.find_source_var(*id, edge0.id, &port_vars);
                    let edge1_src = self.find_source_var(*id, edge1.id, &port_vars);
                    let x_src = self.find_source_var(*id, x.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = smoothstep({edge0_src}, {edge1_src}, {x_src});\n"));
                }
                NodeKind::Clamp => {
                    let x = &node.inputs[0];
                    let min = &node.inputs[1];
                    let max = &node.inputs[2];
                    let x_src = self.find_source_var(*id, x.id, &port_vars);
                    let min_src = self.find_source_var(*id, min.id, &port_vars);
                    let max_src = self.find_source_var(*id, max.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = clamp({x_src}, {min_src}, {max_src});\n"));
                }
                NodeKind::TextureSample => {
                    let _tex = &node.inputs[0];
                    let uv_in = &node.inputs[1];
                    let uv_src = self.find_source_var(*id, uv_in.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec4<f32> = textureSample(tex0, samp, {uv_src});\n"));
                }
                // Missing node types - add basic implementations
                NodeKind::Resolution => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec2<f32> = uniforms.resolution;\n"));
                }
                NodeKind::Mouse => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    // Use center of screen as mouse position for now
                    code.push_str(&format!("  let {var}: vec2<f32> = uniforms.resolution * 0.5;\n"));
                }
                NodeKind::Reflect => {
                    let i = &node.inputs[0];
                    let n = &node.inputs[1];
                    let i_src = self.find_source_var(*id, i.id, &port_vars);
                    let n_src = self.find_source_var(*id, n.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = reflect({i_src}, {n_src});\n"));
                }
                NodeKind::Refract => {
                    let i = &node.inputs[0];
                    let n = &node.inputs[1];
                    let eta = &node.inputs[2];
                    let i_src = self.find_source_var(*id, i.id, &port_vars);
                    let n_src = self.find_source_var(*id, n.id, &port_vars);
                    let eta_src = self.find_source_var(*id, eta.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = refract({i_src}, {n_src}, {eta_src});\n"));
                }
                NodeKind::Sign => {
                    let x = &node.inputs[0];
                    let x_src = self.find_source_var(*id, x.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = sign({x_src});\n"));
                }
                NodeKind::RGB => {
                    let r = &node.inputs[0];
                    let g = &node.inputs[1];
                    let b = &node.inputs[2];
                    let r_src = self.find_source_var(*id, r.id, &port_vars);
                    let g_src = self.find_source_var(*id, g.id, &port_vars);
                    let b_src = self.find_source_var(*id, b.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = vec3<f32>({r_src}, {g_src}, {b_src});\n"));
                }
                NodeKind::HSV => {
                    let h = &node.inputs[0];
                    let s = &node.inputs[1];
                    let v = &node.inputs[2];
                    let h_src = self.find_source_var(*id, h.id, &port_vars);
                    let s_src = self.find_source_var(*id, s.id, &port_vars);
                    let v_src = self.find_source_var(*id, v.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    // Simple HSV to RGB conversion
                    code.push_str(&format!("  let c = {v_src} * {s_src};\n"));
                    code.push_str(&format!("  let x = c * (1.0 - abs(mod({h_src} * 6.0, 2.0) - 1.0));\n"));
                    code.push_str(&format!("  let m = {v_src} - c;\n"));
                    code.push_str(&format!("  let {var}: vec3<f32> = vec3<f32>(c, x, 0.0) + m;\n"));
                }
                NodeKind::ColorMix => {
                    let color1 = &node.inputs[0];
                    let color2 = &node.inputs[1];
                    let t = &node.inputs[2];
                    let color1_src = self.find_source_var(*id, color1.id, &port_vars);
                    let color2_src = self.find_source_var(*id, color2.id, &port_vars);
                    let t_src = self.find_source_var(*id, t.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = mix({color1_src}, {color2_src}, {t_src});\n"));
                }
                NodeKind::ColorAdjust => {
                    let color = &node.inputs[0];
                    let brightness = &node.inputs[1];
                    let contrast = &node.inputs[2];
                    let saturation = &node.inputs[3];
                    let color_src = self.find_source_var(*id, color.id, &port_vars);
                    let brightness_src = self.find_source_var(*id, brightness.id, &port_vars);
                    let contrast_src = self.find_source_var(*id, contrast.id, &port_vars);
                    let saturation_src = self.find_source_var(*id, saturation.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = ({color_src} * {contrast_src} + {brightness_src}) * {saturation_src};\n"));
                }
                NodeKind::Noise2D => {
                    let position = &node.inputs[0];
                    let position_src = self.find_source_var(*id, position.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    // Simple pseudo-noise function
                    code.push_str(&format!("  let {var}: f32 = fract(sin(dot({position_src}, vec2<f32>(12.9898, 78.233))) * 43758.5453);\n"));
                }
                NodeKind::Noise3D => {
                    let position = &node.inputs[0];
                    let position_src = self.find_source_var(*id, position.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    // Simple pseudo-noise function for 3D
                    code.push_str(&format!("  let {var}: f32 = fract(sin(dot({position_src}, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);\n"));
                }
                NodeKind::Voronoi => {
                    let position = &node.inputs[0];
                    let position_src = self.find_source_var(*id, position.id, &port_vars);
                    let out = node.outputs[0].id;
                    let cell_out = node.outputs[1].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    let cell_var = self.add_port_var(&mut port_vars, &mut var_counter, *id, cell_out);
                    // Simple voronoi approximation
                    code.push_str(&format!("  let {var}: f32 = length(fract({position_src}) - 0.5);\n"));
                    code.push_str(&format!("  let {cell_var}: f32 = floor({position_src}.x) + floor({position_src}.y) * 100.0;\n"));
                }
                NodeKind::TextureSampleLod => {
                    let _tex = &node.inputs[0];
                    let uv_in = &node.inputs[1];
                    let lod_in = &node.inputs[2];
                    let uv_src = self.find_source_var(*id, uv_in.id, &port_vars);
                    let lod_src = self.find_source_var(*id, lod_in.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec4<f32> = textureSampleLevel(tex0, samp, {uv_src}, {lod_src});\n"));
                }
                NodeKind::TextureSize => {
                    let _tex = &node.inputs[0];
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec2<f32> = vec2<f32>(textureDimensions(tex0));\n"));
                }
                NodeKind::OutputColor => {
                    let color_in = &node.inputs[0];
                    let src = self.find_source_var(*id, color_in.id, &port_vars);
                    code.push_str(&format!("  return {src};\n"));
                }
            }
        }

        // Fallback if no output color produced
        code.push_str("  return vec4<f32>(uv.x, uv.y, 0.5, 1.0);\n}\n");

        code
    }

    fn add_port_var(
        &self,
        port_vars: &mut HashMap<(NodeId, PortId), String>,
        var_counter: &mut u32,
        nid: NodeId,
        pid: PortId,
    ) -> String {
        *var_counter += 1;
        let name = format!("v{}", *var_counter);
        port_vars.insert((nid, pid), name.clone());
        name
    }

    fn find_source_var(&self, to_node: NodeId, to_port: PortId, port_vars: &HashMap<(NodeId, PortId), String>) -> String {
        for c in &self.connections {
            if c.to_node == to_node && c.to_port == to_port {
                if let Some(name) = port_vars.get(&(c.from_node, c.from_port)) {
                    return name.clone();
                }
            }
        }
        // Default zero values per kind are handled upstream; return a neutral
        "0.0".to_string()
    }
}

// Resource for use with Bevy
use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct NodeGraphResource {
    pub graph: NodeGraph,
    pub selected_node: Option<NodeId>,
    pub selected_nodes: std::collections::HashSet<NodeId>,
}