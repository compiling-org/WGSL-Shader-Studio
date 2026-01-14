//! Visual Language Compiler
//! Converts between visual node graphs and WGSL code with validation and error checking

use crate::node_graph::{NodeGraph, NodeId, NodeKind, PortId, PortKind};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VisualLanguageCompiler {
    // Compilation settings
    validate_connections: bool,
    generate_debug_info: bool,
    optimize_code: bool,
    // Error tracking
    errors: Vec<CompilationError>,
    warnings: Vec<CompilationWarning>,
}

#[derive(Debug, Clone)]
pub struct CompilationError {
    pub node_id: Option<NodeId>,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone)]
pub struct CompilationWarning {
    pub node_id: Option<NodeId>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

impl Default for VisualLanguageCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualLanguageCompiler {
    pub fn new() -> Self {
        Self {
            validate_connections: true,
            generate_debug_info: false,
            optimize_code: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Compile a node graph to WGSL with validation
    pub fn compile_to_wgsl(&mut self, node_graph: &NodeGraph) -> Result<String, Vec<CompilationError>> {
        // Reset errors
        self.errors.clear();
        self.warnings.clear();

        // Validate the graph first
        if self.validate_connections {
            self.validate_node_graph(node_graph)?;
        }

        // Generate WGSL code
        let wgsl_code = self.generate_wgsl_from_graph(node_graph);

        if !self.errors.is_empty() {
            Err(self.errors.clone())
        } else {
            Ok(wgsl_code)
        }
    }

    /// Validate the node graph for compilation readiness
    fn validate_node_graph(&mut self, node_graph: &NodeGraph) -> Result<(), Vec<CompilationError>> {
        // Check for unconnected required inputs
        for (node_id, node) in &node_graph.nodes {
            for input in &node.inputs {
                let is_connected = node_graph.connections.iter().any(|conn| {
                    conn.to_node == *node_id && conn.to_port == input.id
                });

                if !is_connected && self.is_required_input(node, &input.id) {
                    self.errors.push(CompilationError {
                        node_id: Some(*node_id),
                        message: format!("Required input '{}' is not connected", input.name),
                        severity: ErrorSeverity::Error,
                    });
                }
            }
        }

        // Check for cycles in the graph
        if self.has_cycles(node_graph) {
            self.errors.push(CompilationError {
                node_id: None,
                message: "Graph contains cycles which would cause infinite loops".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        // Check for output nodes
        let has_output = node_graph.nodes.values().any(|node| {
            matches!(node.kind, NodeKind::OutputColor)
        });

        if !has_output {
            self.errors.push(CompilationError {
                node_id: None,
                message: "Graph must contain at least one output node".to_string(),
                severity: ErrorSeverity::Error,
            });
        }

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(())
    }

    /// Check if an input is required for the node to function
    fn is_required_input(&self, node: &crate::node_graph::Node, port_id: &PortId) -> bool {
        // For now, consider all inputs as required except for optional parameters
        true
    }

    /// Check if the graph has cycles that would cause infinite loops
    fn has_cycles(&self, node_graph: &NodeGraph) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for node_id in node_graph.nodes.keys() {
            if !visited.contains(node_id) && self.has_cycle_util(node_graph, *node_id, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    /// Utility function for cycle detection
    fn has_cycle_util(
        &self,
        node_graph: &NodeGraph,
        node_id: NodeId,
        visited: &mut std::collections::HashSet<NodeId>,
        rec_stack: &mut std::collections::HashSet<NodeId>,
    ) -> bool {
        if !visited.contains(&node_id) {
            visited.insert(node_id);
            rec_stack.insert(node_id);

            // Find all nodes that depend on this node
            for conn in &node_graph.connections {
                if conn.from_node == node_id {
                    let to_node = conn.to_node;
                    if !visited.contains(&to_node) && self.has_cycle_util(node_graph, to_node, visited, rec_stack) {
                        return true;
                    } else if rec_stack.contains(&to_node) {
                        return true;
                    }
                }
            }
        }

        rec_stack.remove(&node_id);
        false
    }

    /// Generate WGSL code from the node graph
    fn generate_wgsl_from_graph(&self, node_graph: &NodeGraph) -> String {
        let mut code = String::new();
        
        // Add standard uniforms
        code.push_str("struct Uniforms {\n  time: f32,\n  resolution: vec2<f32>,\n};\n\n");
        code.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");
        // Secondary uniform buffer for named parameters (64 floats packed into 16 vec4s for std140)
        code.push_str("@group(0) @binding(1) var<uniform> params: array<vec4<f32>, 16>;\n\n");
        // Vertex shader: full-screen triangle
        code.push_str("@vertex\nfn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {\n  var positions = array<vec2<f32>, 3>(\n    vec2<f32>(-1.0, -3.0),\n    vec2<f32>(-1.0,  1.0),\n    vec2<f32>( 3.0,  1.0),\n  );\n  let pos = positions[i];\n  return vec4<f32>(pos, 0.0, 1.0);\n}\n\n");

        // Fragment prelude: declare any textures if present
        let mut uses_texture = false;
        for n in node_graph.nodes.values() {
            if matches!(n.kind, NodeKind::TextureSample | NodeKind::TextureSampleLod | NodeKind::TextureSize) {
                uses_texture = true;
                break;
            }
        }
        if uses_texture {
            // Use non-conflicting bindings: params occupy binding(1)
            code.push_str("@group(0) @binding(2) var samp: sampler;\n@group(0) @binding(3) var tex0: texture_2d<f32>;\n\n");
        }

        // Build evaluation order (topological sort by repeatedly selecting nodes whose inputs are satisfied)
        let mut order: Vec<NodeId> = Vec::new();
        let mut satisfied: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
        
        // Source nodes have no inputs that need to be satisfied by other nodes
        for (id, node) in &node_graph.nodes {
            let mut has_unsatisfied_inputs = false;
            for input in &node.inputs {
                let has_input = node_graph.connections.iter().any(|c| c.to_node == *id && c.to_port == input.id);
                if has_input {
                    has_unsatisfied_inputs = true;
                    break;
                }
            }
            if !has_unsatisfied_inputs {
                satisfied.insert(*id);
                order.push(*id);
            }
        }

        // Iterate to include remaining nodes
        let mut remaining: std::collections::HashSet<NodeId> = node_graph.nodes.keys().copied().collect();
        for id in order.iter() { 
            remaining.remove(id); 
        }
        let mut guard = 0;
        while !remaining.is_empty() && guard < 1024 {
            guard += 1;
            let mut progressed = false;
            for id in remaining.clone() {
                let node = &node_graph.nodes[&id];
                let mut all_inputs_satisfied = true;
                for inp in &node.inputs {
                    let has_input = node_graph.connections.iter().any(|c| c.to_node == id && c.to_port == inp.id && satisfied.contains(&c.from_node));
                    if !has_input { 
                        all_inputs_satisfied = false; 
                        break; 
                    }
                }
                if all_inputs_satisfied {
                    satisfied.insert(id);
                    order.push(id);
                    remaining.remove(&id);
                    progressed = true;
                }
            }
            if !progressed { 
                // If we can't progress, there might be a cycle or unconnected nodes
                break; 
            }
        }

        // Temporary mapping: for each port, a WGSL variable name
        let mut port_vars: HashMap<(NodeId, PortId), String> = HashMap::new();
        let mut var_counter = 0u32;

        code.push_str("@fragment\nfn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {\n");
        code.push_str("  let uv = pos.xy / uniforms.resolution;\n");

        for id in order.iter() {
            let node = &node_graph.nodes[id];
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
        for c in &crate::node_graph::NodeGraph::default().connections {
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