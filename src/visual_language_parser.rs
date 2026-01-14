//! Visual Language Parser
//! Converts WGSL code back to visual node graphs for round-trip editing

use crate::node_graph::{NodeGraph, NodeId, NodeKind, PortId, PortKind};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VisualLanguageParser {
    // Parsing settings
    parse_uniforms: bool,
    parse_functions: bool,
    parse_structs: bool,
    // Error tracking
    errors: Vec<ParsingError>,
    warnings: Vec<ParsingWarning>,
}

#[derive(Debug, Clone)]
pub struct ParsingError {
    pub line: Option<usize>,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone)]
pub struct ParsingWarning {
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

impl Default for VisualLanguageParser {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualLanguageParser {
    pub fn new() -> Self {
        Self {
            parse_uniforms: true,
            parse_functions: true,
            parse_structs: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Parse WGSL code and convert it to a node graph
    pub fn parse_wgsl_to_graph(&mut self, wgsl_code: &str) -> Result<NodeGraph, Vec<ParsingError>> {
        // Reset errors
        self.errors.clear();
        self.warnings.clear();

        let mut node_graph = NodeGraph::new();
        
        // Parse the code line by line
        let lines: Vec<&str> = wgsl_code.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim();
            
            // Skip empty lines and comments
            if trimmed_line.is_empty() || trimmed_line.starts_with("//") {
                continue;
            }
            
            // Parse different types of statements
            if let Err(e) = self.parse_line(&mut node_graph, line_num, trimmed_line) {
                self.errors.push(e);
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(node_graph)
    }

    /// Parse a single line of WGSL code
    fn parse_line(&mut self, node_graph: &mut NodeGraph, line_num: usize, line: &str) -> Result<(), ParsingError> {
        // Look for different patterns in the line
        if line.starts_with("let ") {
            self.parse_let_statement(node_graph, line_num, line)
        } else if line.starts_with("@vertex") || line.starts_with("@fragment") {
            // Entry point annotations - skip for now
            Ok(())
        } else if line.starts_with("fn ") {
            // Function definitions - skip for now
            Ok(())
        } else if line.contains("uniforms.") {
            // Uniform access
            self.parse_uniform_access(node_graph, line_num, line)
        } else if line.contains("textureSample") {
            // Texture sampling operations
            self.parse_texture_sample(node_graph, line_num, line)
        } else if line.contains("return ") {
            // Return statements
            self.parse_return_statement(node_graph, line_num, line)
        } else if line.contains("vec") && line.contains("=") {
            // Vector operations
            self.parse_vector_operation(node_graph, line_num, line)
        } else if line.contains('(') && line.contains(')') {
            // Function calls
            self.parse_function_call(node_graph, line_num, line)
        } else {
            // Other statements - for now just skip
            Ok(())
        }
    }

    /// Parse a let statement (variable assignment)
    fn parse_let_statement(&mut self, node_graph: &mut NodeGraph, line_num: usize, line: &str) -> Result<(), ParsingError> {
        // Example: "let v1: f32 = uniforms.time;"
        // Extract variable name, type, and expression
        if let Some(eq_pos) = line.find('=') {
            let left_part = &line[4..eq_pos].trim(); // Skip "let "
            let right_part = &line[eq_pos + 1..line.len() - 1].trim(); // Skip "=" and ";"

            // Extract variable name and type
            let var_parts: Vec<&str> = left_part.split(':').collect();
            if var_parts.len() < 2 {
                return Ok(()); // Skip if format is unexpected
            }
            
            let var_name = var_parts[0].trim();
            let var_type = var_parts[1].trim();

            // Create appropriate node based on the expression
            if right_part == &"uniforms.time" {
                let node_id = node_graph.add_node(NodeKind::Time, &format!("Time_{}", var_name), (0.0, 0.0));
                // Store mapping from variable name to node ID for later connections
                // For now, we'll just create the node
            } else if right_part == &"uv" {
                let node_id = node_graph.add_node(NodeKind::UV, &format!("UV_{}", var_name), (0.0, 0.0));
            } else if right_part.starts_with("vec2<f32>") {
                // Parse vector values
                if let Some(values) = self.parse_vector_values(right_part) {
                    let node_id = node_graph.add_node(
                        NodeKind::ConstantVec2([values[0], values[1]]),
                        &format!("Vec2_{}", var_name),
                        (0.0, 0.0)
                    );
                }
            } else if right_part.starts_with("vec3<f32>") {
                if let Some(values) = self.parse_vector_values(right_part) {
                    let node_id = node_graph.add_node(
                        NodeKind::ConstantVec3([values[0], values[1], values[2]]),
                        &format!("Vec3_{}", var_name),
                        (0.0, 0.0)
                    );
                }
            } else if right_part.starts_with("vec4<f32>") {
                if let Some(values) = self.parse_vector_values(right_part) {
                    let node_id = node_graph.add_node(
                        NodeKind::ConstantVec4(values),
                        &format!("Vec4_{}", var_name),
                        (0.0, 0.0)
                    );
                }
            } else if let Ok(num) = right_part.parse::<f32>() {
                let node_id = node_graph.add_node(
                    NodeKind::ConstantFloat(num),
                    &format!("Float_{}", var_name),
                    (0.0, 0.0)
                );
            }
        }

        Ok(())
    }

    /// Parse a uniform access (like uniforms.time or uniforms.resolution)
    fn parse_uniform_access(&mut self, node_graph: &mut NodeGraph, _line_num: usize, line: &str) -> Result<(), ParsingError> {
        if line.contains("uniforms.time") {
            let node_id = node_graph.add_node(NodeKind::Time, "Time", (0.0, 0.0));
        } else if line.contains("uniforms.resolution") {
            let node_id = node_graph.add_node(NodeKind::Resolution, "Resolution", (0.0, 0.0));
        }
        
        Ok(())
    }

    /// Parse a texture sample operation
    fn parse_texture_sample(&mut self, node_graph: &mut NodeGraph, _line_num: usize, line: &str) -> Result<(), ParsingError> {
        // Example: textureSample(tex0, samp, uv)
        let node_id = node_graph.add_node(NodeKind::TextureSample, "TextureSample", (0.0, 0.0));
        Ok(())
    }

    /// Parse a return statement
    fn parse_return_statement(&mut self, node_graph: &mut NodeGraph, _line_num: usize, line: &str) -> Result<(), ParsingError> {
        // Example: return vec4<f32>(color, 1.0);
        let return_expr = &line[7..line.len()-1].trim(); // Skip "return " and ";"
        
        // Create an output node
        let node_id = node_graph.add_node(NodeKind::OutputColor, "Output", (200.0, 0.0));
        Ok(())
    }

    /// Parse vector operations
    fn parse_vector_operation(&mut self, _node_graph: &mut NodeGraph, _line_num: usize, _line: &str) -> Result<(), ParsingError> {
        // For now, just return Ok. Vector operations would need more complex parsing
        // to determine what operations are being performed
        Ok(())
    }

    /// Parse function calls (like sin, cos, mix, etc.)
    fn parse_function_call(&mut self, node_graph: &mut NodeGraph, _line_num: usize, line: &str) -> Result<(), ParsingError> {
        if line.contains("sin(") {
            let node_id = node_graph.add_node(NodeKind::Sine, "Sine", (0.0, 0.0));
        } else if line.contains("cos(") {
            let node_id = node_graph.add_node(NodeKind::Cosine, "Cosine", (0.0, 0.0));
        } else if line.contains("mix(") {
            let node_id = node_graph.add_node(NodeKind::Mix, "Mix", (0.0, 0.0));
        } else if line.contains("abs(") {
            let node_id = node_graph.add_node(NodeKind::Abs, "Abs", (0.0, 0.0));
        } else if line.contains("length(") {
            let node_id = node_graph.add_node(NodeKind::Length, "Length", (0.0, 0.0));
        } else if line.contains("normalize(") {
            let node_id = node_graph.add_node(NodeKind::Normalize, "Normalize", (0.0, 0.0));
        } else if line.contains("dot(") {
            let node_id = node_graph.add_node(NodeKind::Dot, "Dot", (0.0, 0.0));
        } else if line.contains("cross(") {
            let node_id = node_graph.add_node(NodeKind::Cross, "Cross", (0.0, 0.0));
        } else if line.contains("min(") {
            let node_id = node_graph.add_node(NodeKind::Min, "Min", (0.0, 0.0));
        } else if line.contains("max(") {
            let node_id = node_graph.add_node(NodeKind::Max, "Max", (0.0, 0.0));
        } else if line.contains("pow(") {
            let node_id = node_graph.add_node(NodeKind::Pow, "Pow", (0.0, 0.0));
        } else if line.contains("sqrt(") {
            let node_id = node_graph.add_node(NodeKind::Sqrt, "Sqrt", (0.0, 0.0));
        } else if line.contains("floor(") {
            let node_id = node_graph.add_node(NodeKind::Floor, "Floor", (0.0, 0.0));
        } else if line.contains("ceil(") {
            let node_id = node_graph.add_node(NodeKind::Ceil, "Ceil", (0.0, 0.0));
        } else if line.contains("fract(") {
            let node_id = node_graph.add_node(NodeKind::Fract, "Fract", (0.0, 0.0));
        } else if line.contains("sign(") {
            let node_id = node_graph.add_node(NodeKind::Sign, "Sign", (0.0, 0.0));
        } else if line.contains("step(") {
            let node_id = node_graph.add_node(NodeKind::Step, "Step", (0.0, 0.0));
        } else if line.contains("smoothstep(") {
            let node_id = node_graph.add_node(NodeKind::Smoothstep, "Smoothstep", (0.0, 0.0));
        } else if line.contains("clamp(") {
            let node_id = node_graph.add_node(NodeKind::Clamp, "Clamp", (0.0, 0.0));
        }

        Ok(())
    }

    /// Parse vector values from a string like "vec2<f32>(1.0, 2.0)"
    fn parse_vector_values(&self, input: &str) -> Option<[f32; 4]> {
        // Extract values between parentheses
        if let (Some(start), Some(end)) = (input.find('('), input.find(')')) {
            let values_str = &input[start+1..end];
            let values: Vec<&str> = values_str.split(',').map(|s| s.trim()).collect();
            
            // Convert to array of f32
            let mut result = [0.0f32; 4];
            for (i, val_str) in values.iter().enumerate() {
                if i >= 4 { break; } // Max 4 values
                if let Ok(val) = val_str.parse::<f32>() {
                    result[i] = val;
                } else {
                    // Try to handle expressions like "1.0" or "uv.x"
                    // For now, just skip if can't parse
                    return None;
                }
            }
            
            // Return based on vector size
            match values.len() {
                2 => Some([result[0], result[1], 0.0, 0.0]),
                3 => Some([result[0], result[1], result[2], 0.0]),
                4 => Some([result[0], result[1], result[2], result[3]]),
                _ => Some(result), // Default to 4-element vector
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_wgsl() {
        let wgsl_code = r#"
struct Uniforms {
  time: f32,
  resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 3.0,  1.0),
  );
  let pos = positions[i];
  return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let uv = pos.xy / uniforms.resolution;
  let v1: f32 = uniforms.time;
  let v2: vec2<f32> = vec2<f32>(1.0, 2.0);
  let v3: vec3<f32> = vec3<f32>(1.0, 2.0, 3.0);
  let v4: vec4<f32> = vec4<f32>(1.0, 2.0, 3.0, 4.0);
  let v5: f32 = sin(v1);
  return vec4<f32>(v4.x, v4.y, v4.z, 1.0);
}
"#;

        let mut parser = VisualLanguageParser::new();
        let result = parser.parse_wgsl_to_graph(wgsl_code);
        
        assert!(result.is_ok());
        let graph = result.unwrap();
        
        // Should have nodes for time, UV, constants, sine, and output
        assert!(graph.nodes.values().any(|n| matches!(n.kind, NodeKind::Time)));
        assert!(graph.nodes.values().any(|n| matches!(n.kind, NodeKind::OutputColor)));
        assert!(graph.nodes.values().any(|n| matches!(n.kind, NodeKind::Sine)));
    }

    #[test]
    fn test_parse_vector_values() {
        let parser = VisualLanguageParser::new();
        
        // Test vec2
        let result = parser.parse_vector_values("vec2<f32>(1.0, 2.0)");
        assert_eq!(result, Some([1.0, 2.0, 0.0, 0.0]));
        
        // Test vec3
        let result = parser.parse_vector_values("vec3<f32>(1.0, 2.0, 3.0)");
        assert_eq!(result, Some([1.0, 2.0, 3.0, 0.0]));
        
        // Test vec4
        let result = parser.parse_vector_values("vec4<f32>(1.0, 2.0, 3.0, 4.0)");
        assert_eq!(result, Some([1.0, 2.0, 3.0, 4.0]));
    }
}