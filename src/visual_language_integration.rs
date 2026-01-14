//! Visual Language Integration
//! Bridges the visual node editor with the visual language compiler for enhanced functionality

use crate::node_graph::NodeGraph;
use crate::visual_language_compiler::VisualLanguageCompiler;
use crate::new_visual_node_editor::EnhancedVisualNodeEditor;

#[derive(Debug, Clone)]
pub struct VisualLanguageIntegration {
    pub compiler: VisualLanguageCompiler,
}

impl VisualLanguageIntegration {
    pub fn new() -> Self {
        Self {
            compiler: VisualLanguageCompiler::new(),
        }
    }

    /// Compile a node graph to WGSL with full validation
    pub fn compile_node_graph(&mut self, node_graph: &NodeGraph) -> Result<String, Vec<String>> {
        match self.compiler.compile_to_wgsl(node_graph) {
            Ok(wgsl) => Ok(wgsl),
            Err(errors) => {
                let error_messages: Vec<String> = errors
                    .iter()
                    .map(|e| format!("{}: {}", 
                        e.node_id.map_or("Graph".to_string(), |id| format!("Node({})", id.0)), 
                        e.message))
                    .collect();
                Err(error_messages)
            }
        }
    }

    /// Validate a node graph without compiling
    pub fn validate_node_graph(&mut self, node_graph: &NodeGraph) -> Result<(), Vec<String>> {
        // We'll use the compiler's validation but not generate code
        let result = self.compiler.compile_to_wgsl(node_graph);
        
        match result {
            Ok(_) => Ok(()),
            Err(errors) => {
                let error_messages: Vec<String> = errors
                    .iter()
                    .map(|e| format!("{}: {}", 
                        e.node_id.map_or("Graph".to_string(), |id| format!("Node({})", id.0)), 
                        e.message))
                    .collect();
                Err(error_messages)
            }
        }
    }

    /// Get the internal compiler for direct access
    pub fn compiler(&mut self) -> &mut VisualLanguageCompiler {
        &mut self.compiler
    }
}

/// Extension trait for enhanced visual node editor
pub trait VisualNodeEditorExt {
    /// Compile the node graph in the editor with validation
    fn compile_with_validation(&mut self, node_graph: &NodeGraph) -> Result<String, String>;
    
    /// Validate the node graph in the editor
    fn validate(&mut self, node_graph: &NodeGraph) -> Result<(), String>;
}

impl VisualNodeEditorExt for EnhancedVisualNodeEditor {
    fn compile_with_validation(&mut self, node_graph: &NodeGraph) -> Result<String, String> {
        match self.visual_language_integration.compile_node_graph(node_graph) {
            Ok(wgsl) => Ok(wgsl),
            Err(errors) => Err(errors.join("\n")),
        }
    }
    
    fn validate(&mut self, node_graph: &NodeGraph) -> Result<(), String> {
        match self.visual_language_integration.validate_node_graph(node_graph) {
            Ok(()) => Ok(()),
            Err(errors) => Err(errors.join("\n")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_graph::{NodeKind, PortKind};

    #[test]
    fn test_visual_language_integration() {
        let mut integration = VisualLanguageIntegration::new();
        let mut node_graph = NodeGraph::new();
        
        // Add a simple time node
        let time_node = node_graph.add_node(NodeKind::Time, "Time", (0.0, 0.0));
        
        // Add an output node
        let output_node = node_graph.add_node(NodeKind::OutputColor, "Output", (200.0, 0.0));
        
        // Connect them (this would require an intermediate node in a real scenario)
        // For now, just validate that the graph compiles without errors
        let result = integration.compile_node_graph(&node_graph);
        
        // Should have an error because there's no connection between time (f32) and output (vec4)
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_graph_compilation() {
        let mut integration = VisualLanguageIntegration::new();
        let mut node_graph = NodeGraph::new();
        
        // Add a constant vec4 node (color)
        let color_node = node_graph.add_node(
            NodeKind::ConstantVec4([1.0, 0.0, 0.0, 1.0]), 
            "Red Color", 
            (0.0, 0.0)
        );
        
        // Add an output node
        let output_node = node_graph.add_node(NodeKind::OutputColor, "Output", (200.0, 0.0));
        
        // Connect the color to output
        node_graph.connect(color_node, node_graph.nodes[&color_node].outputs[0].id, output_node, node_graph.nodes[&output_node].inputs[0].id);
        
        let result = integration.compile_node_graph(&node_graph);
        
        // Should compile successfully
        assert!(result.is_ok());
        let wgsl = result.unwrap();
        assert!(wgsl.contains("vec4<f32>(1.0,0.0,0.0,1.0)"));
        assert!(wgsl.contains("return"));
    }
}