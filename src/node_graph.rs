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
    ConstantFloat(f32),
    ConstantVec3([f32; 3]),
    Time,
    UV,
    Param(usize),
    Add,
    Multiply,
    Sine,
    TextureSample,
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
            NodeKind::ConstantFloat(_) => {
                node.outputs.push(self.new_port("value", PortKind::Float));
            }
            NodeKind::ConstantVec3(_) => {
                node.outputs.push(self.new_port("value", PortKind::Vec3));
            }
            NodeKind::Time => {
                node.outputs.push(self.new_port("time", PortKind::Float));
            }
            NodeKind::UV => {
                node.outputs.push(self.new_port("uv", PortKind::Vec2));
            }
            NodeKind::Param(_) => {
                node.outputs.push(self.new_port("value", PortKind::Float));
            }
            NodeKind::Add | NodeKind::Multiply => {
                node.inputs.push(self.new_port("a", PortKind::Float));
                node.inputs.push(self.new_port("b", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            NodeKind::Sine => {
                node.inputs.push(self.new_port("x", PortKind::Float));
                node.outputs.push(self.new_port("out", PortKind::Float));
            }
            NodeKind::TextureSample => {
                node.inputs.push(self.new_port("tex", PortKind::Texture));
                node.inputs.push(self.new_port("uv", PortKind::Vec2));
                node.outputs.push(self.new_port("color", PortKind::Color));
            }
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
                NodeKind::ConstantVec3(v) => {
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec3<f32> = vec3<f32>({},{},{});\n", v[0], v[1], v[2]));
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
                NodeKind::Add | NodeKind::Multiply => {
                    let a = &node.inputs[0];
                    let b = &node.inputs[1];
                    let a_src = self.find_source_var(*id, a.id, &port_vars);
                    let b_src = self.find_source_var(*id, b.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    if matches!(node.kind, NodeKind::Add) {
                        code.push_str(&format!("  let {var}: f32 = {a_src} + {b_src};\n"));
                    } else {
                        code.push_str(&format!("  let {var}: f32 = {a_src} * {b_src};\n"));
                    }
                }
                NodeKind::Sine => {
                    let x = &node.inputs[0];
                    let x_src = self.find_source_var(*id, x.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: f32 = sin({x_src});\n"));
                }
                NodeKind::TextureSample => {
                    let _tex = &node.inputs[0];
                    let uv_in = &node.inputs[1];
                    let uv_src = self.find_source_var(*id, uv_in.id, &port_vars);
                    let out = node.outputs[0].id;
                    let var = self.add_port_var(&mut port_vars, &mut var_counter, *id, out);
                    code.push_str(&format!("  let {var}: vec4<f32> = textureSample(tex0, samp, {uv_src});\n"));
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