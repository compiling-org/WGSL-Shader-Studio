//! Visual Language Bridge
//! Provides round-trip conversion between visual node graphs and WGSL code

use crate::node_graph::NodeGraph;
use crate::visual_language_compiler::VisualLanguageCompiler;
use crate::visual_language_parser::VisualLanguageParser;

#[derive(Debug, Clone)]
pub struct VisualLanguageBridge {
    compiler: VisualLanguageCompiler,
    parser: VisualLanguageParser,
}

impl VisualLanguageBridge {
    pub fn new() -> Self {
        Self {
            compiler: VisualLanguageCompiler::new(),
            parser: VisualLanguageParser::new(),
        }
    }

    /// Compile a node graph to WGSL code
    pub fn compile_to_wgsl(&mut self, node_graph: &NodeGraph) -> Result<String, Vec<String>> {
        match self.compiler.compile_to_wgsl(node_graph) {
            Ok(code) => Ok(code),
            Err(errors) => {
                let error_strings: Vec<String> = errors
                    .iter()
                    .map(|e| format!(
                        "{}: {}", 
                        e.node_id.map_or("Graph".to_string(), |id| format!("Node({})", id.0)), 
                        e.message
                    ))
                    .collect();
                Err(error_strings)
            }
        }
    }

    /// Parse WGSL code back to a node graph
    pub fn parse_to_graph(&mut self, wgsl_code: &str) -> Result<NodeGraph, Vec<String>> {
        match self.parser.parse_wgsl_to_graph(wgsl_code) {
            Ok(graph) => Ok(graph),
            Err(errors) => {
                let error_messages: Vec<String> = errors
                    .iter()
                    .map(|e| format!("Line {}: {}", 
                        e.line.map_or("Unknown".to_string(), |l| l.to_string()), 
                        e.message))
                    .collect();
                Err(error_messages)
            }
        }
    }

    /// Perform round-trip conversion: graph -> wgsl -> graph
    pub fn round_trip_conversion(&mut self, original_graph: &NodeGraph) -> Result<NodeGraph, Vec<String>> {
        // First compile to WGSL
        let wgsl_code = self.compile_to_wgsl(original_graph)?;
        
        // Then parse back to graph
        let new_graph = self.parse_to_graph(&wgsl_code)?;
        
        Ok(new_graph)
    }

    /// Check if a round-trip conversion is equivalent
    pub fn is_equivalent_after_round_trip(&mut self, original_graph: &NodeGraph) -> Result<bool, Vec<String>> {
        let new_graph = self.round_trip_conversion(original_graph)?;
        
        // For now, we'll just check if the number of nodes and connections are the same
        // In a full implementation, we'd need more sophisticated comparison
        Ok(
            original_graph.nodes.len() == new_graph.nodes.len() &&
            original_graph.connections.len() == new_graph.connections.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_graph::{NodeKind, PortKind};

    #[test]
    fn test_round_trip_conversion() {
        let mut bridge = VisualLanguageBridge::new();
        let mut original_graph = NodeGraph::new();
        
        // Add a simple constant vec4 node
        let color_node = original_graph.add_node(
            NodeKind::ConstantVec4([1.0, 0.0, 0.0, 1.0]), 
            "Red Color", 
            (0.0, 0.0)
        );
        
        // Add an output node
        let output_node = original_graph.add_node(NodeKind::OutputColor, "Output", (200.0, 0.0));
        
        // Connect the color to output
        original_graph.connect(
            color_node, 
            original_graph.nodes[&color_node].outputs[0].id, 
            output_node, 
            original_graph.nodes[&output_node].inputs[0].id
        );
        
        // Perform round-trip conversion
        let result = bridge.round_trip_conversion(&original_graph);
        assert!(result.is_ok());
        
        let new_graph = result.unwrap();
        
        // Check that the new graph has the same number of nodes and connections
        assert_eq!(original_graph.nodes.len(), new_graph.nodes.len());
        assert_eq!(original_graph.connections.len(), new_graph.connections.len());
    }

    #[test]
    fn test_compile_and_parse_cycle() {
        let mut bridge = VisualLanguageBridge::new();
        
        // Create a simple graph
        let mut graph = NodeGraph::new();
        let color_node = graph.add_node(
            NodeKind::ConstantVec4([1.0, 0.5, 0.25, 1.0]), 
            "Color", 
            (0.0, 0.0)
        );
        let output_node = graph.add_node(NodeKind::OutputColor, "Output", (200.0, 0.0));
        graph.connect(
            color_node, 
            graph.nodes[&color_node].outputs[0].id, 
            output_node, 
            graph.nodes[&output_node].inputs[0].id
        );
        
        // Compile to WGSL
        let wgsl_result = bridge.compile_to_wgsl(&graph);
        assert!(wgsl_result.is_ok());
        let wgsl_code = wgsl_result.unwrap();
        
        // Parse back to graph
        let parsed_result = bridge.parse_to_graph(&wgsl_code);
        assert!(parsed_result.is_ok());
        
        let parsed_graph = parsed_result.unwrap();
        
        // Both graphs should have the same structure
        assert!(parsed_graph.nodes.len() >= 1); // At least an output node
    }
}