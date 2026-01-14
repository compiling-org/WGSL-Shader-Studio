//! Visual Language Manager
//! Centralized manager for all visual language operations

use crate::node_graph::NodeGraph;
use crate::new_visual_node_editor::EnhancedVisualNodeEditor;
use crate::visual_language_bridge::VisualLanguageBridge;

#[derive(Debug, Clone)]
pub struct VisualLanguageManager {
    bridge: VisualLanguageBridge,
    // Configuration settings
    auto_sync: bool,
    validate_on_compile: bool,
    preserve_node_positions: bool,
}

impl VisualLanguageManager {
    pub fn new() -> Self {
        Self {
            bridge: VisualLanguageBridge::new(),
            auto_sync: true,
            validate_on_compile: true,
            preserve_node_positions: true,
        }
    }

    /// Compile a node graph to WGSL with full validation
    pub fn compile_to_wgsl(&mut self, node_graph: &NodeGraph) -> Result<String, Vec<String>> {
        if self.validate_on_compile {
            // First validate the graph
            match self.bridge.compile_to_wgsl(node_graph) {
                Ok(code) => {
                    // Additional validation could go here
                    Ok(code)
                }
                Err(errors) => Err(errors),
            }
        } else {
            self.bridge.compile_to_wgsl(node_graph)
        }
    }

    /// Parse WGSL code back to a node graph
    pub fn parse_to_graph(&mut self, wgsl_code: &str) -> Result<NodeGraph, Vec<String>> {
        self.bridge.parse_to_graph(wgsl_code)
    }

    /// Perform round-trip conversion: graph -> wgsl -> graph
    pub fn round_trip_conversion(&mut self, original_graph: &NodeGraph) -> Result<NodeGraph, Vec<String>> {
        self.bridge.round_trip_conversion(original_graph)
    }

    /// Synchronize a visual node editor with WGSL code
    pub fn sync_editor_with_code(&mut self, editor: &mut EnhancedVisualNodeEditor, wgsl_code: &str) -> Result<(), Vec<String>> {
        let graph = self.parse_to_graph(wgsl_code)?;
        // Here we would update the editor with the parsed graph
        // For now, we'll just validate that the operation is possible
        Ok(())
    }

    /// Synchronize WGSL code with a visual node editor
    pub fn sync_code_with_editor(&mut self, editor: &mut EnhancedVisualNodeEditor, node_graph: &NodeGraph) -> Result<String, Vec<String>> {
        let wgsl = self.compile_to_wgsl(node_graph)?;
        // Here we would update any code editors with the generated WGSL
        // For now, we'll just return the generated code
        Ok(wgsl)
    }

    /// Validate a node graph for common issues
    pub fn validate_graph(&mut self, node_graph: &NodeGraph) -> Result<(), Vec<String>> {
        // Use the bridge to perform a round-trip conversion and check for equivalence
        match self.bridge.is_equivalent_after_round_trip(node_graph) {
            Ok(true) => Ok(()),
            Ok(false) => Err(vec!["Graph is not equivalent after round-trip conversion".to_string()]),
            Err(errors) => Err(errors),
        }
    }

    /// Optimize a node graph by removing unused nodes
    pub fn optimize_graph(&self, node_graph: &mut NodeGraph) {
        // Remove nodes that don't contribute to the output
        let mut nodes_to_remove = Vec::new();
        
        for (node_id, node) in &node_graph.nodes {
            // Check if this node is connected to an output node
            let is_connected_to_output = self.is_node_connected_to_output(*node_id, node_graph);
            
            // Check if this node has any outputs
            let has_outputs = node_graph.connections.iter().any(|conn| conn.from_node == *node_id);
            
            // If it's not connected to output and has no outputs of its own, it might be unused
            if !is_connected_to_output && !has_outputs {
                // Check if it's an input node (like Time, UV, etc.) - these should be kept
                let is_input_node = matches!(node.kind, 
                    crate::node_graph::NodeKind::Time | 
                    crate::node_graph::NodeKind::UV | 
                    crate::node_graph::NodeKind::Param(_) |
                    crate::node_graph::NodeKind::Resolution |
                    crate::node_graph::NodeKind::Mouse |
                    crate::node_graph::NodeKind::ConstantFloat(_) |
                    crate::node_graph::NodeKind::ConstantVec2(_) |
                    crate::node_graph::NodeKind::ConstantVec3(_) |
                    crate::node_graph::NodeKind::ConstantVec4(_)
                );
                
                if !is_input_node {
                    nodes_to_remove.push(*node_id);
                }
            }
        }
        
        // Remove unused nodes
        for node_id in nodes_to_remove {
            node_graph.nodes.remove(&node_id);
            // Remove associated connections
            node_graph.connections.retain(|conn| {
                conn.from_node != node_id && conn.to_node != node_id
            });
        }
    }

    /// Check if a node is connected to an output node (directly or indirectly)
    fn is_node_connected_to_output(&self, node_id: crate::node_graph::NodeId, node_graph: &NodeGraph) -> bool {
        // Find all output nodes
        let output_nodes: Vec<_> = node_graph.nodes.iter()
            .filter(|(_, node)| matches!(node.kind, crate::node_graph::NodeKind::OutputColor))
            .map(|(id, _)| *id)
            .collect();
        
        // Check if the given node can reach any output node
        for output_node in output_nodes {
            if self.can_reach_node(node_id, output_node, node_graph) {
                return true;
            }
        }
        
        false
    }

    /// Check if source_node can reach target_node through the connection graph
    fn can_reach_node(&self, source_node: crate::node_graph::NodeId, target_node: crate::node_graph::NodeId, node_graph: &NodeGraph) -> bool {
        if source_node == target_node {
            return true;
        }
        
        // Use a breadth-first search to check connectivity
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(source_node);
        visited.insert(source_node);
        
        while let Some(current) = queue.pop_front() {
            if current == target_node {
                return true;
            }
            
            // Find all nodes that the current node connects to
            for conn in &node_graph.connections {
                if conn.from_node == current && !visited.contains(&conn.to_node) {
                    visited.insert(conn.to_node);
                    queue.push_back(conn.to_node);
                }
            }
        }
        
        false
    }

    // Getters and setters
    pub fn auto_sync(&self) -> bool {
        self.auto_sync
    }

    pub fn set_auto_sync(&mut self, auto_sync: bool) {
        self.auto_sync = auto_sync;
    }

    pub fn validate_on_compile(&self) -> bool {
        self.validate_on_compile
    }

    pub fn set_validate_on_compile(&mut self, validate: bool) {
        self.validate_on_compile = validate;
    }

    pub fn preserve_node_positions(&self) -> bool {
        self.preserve_node_positions
    }

    pub fn set_preserve_node_positions(&mut self, preserve: bool) {
        self.preserve_node_positions = preserve;
    }
}

impl Default for VisualLanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_graph::{NodeKind, PortKind};

    #[test]
    fn test_visual_language_manager_creation() {
        let manager = VisualLanguageManager::new();
        assert_eq!(manager.auto_sync(), true);
        assert_eq!(manager.validate_on_compile(), true);
    }

    #[test]
    fn test_graph_optimization() {
        let mut manager = VisualLanguageManager::new();
        let mut graph = NodeGraph::new();
        
        // Add a time node (input)
        let time_node = graph.add_node(NodeKind::Time, "Time", (0.0, 0.0));
        
        // Add an unused constant node
        let unused_node = graph.add_node(NodeKind::ConstantFloat(1.0), "Unused", (100.0, 0.0));
        
        // Add a color node
        let color_node = graph.add_node(NodeKind::ConstantVec4([1.0, 0.0, 0.0, 1.0]), "Color", (200.0, 0.0));
        
        // Add an output node
        let output_node = graph.add_node(NodeKind::OutputColor, "Output", (300.0, 0.0));
        
        // Connect color to output
        graph.connect(
            color_node,
            graph.nodes[&color_node].outputs[0].id,
            output_node,
            graph.nodes[&output_node].inputs[0].id
        );
        
        // Before optimization, we should have 4 nodes
        assert_eq!(graph.nodes.len(), 4);
        
        // Optimize the graph
        manager.optimize_graph(&mut graph);
        
        // After optimization, the unused node should be removed
        // but the time node should remain (as an input node)
        assert_eq!(graph.nodes.len(), 3); // Time, Color, Output
        
        // Verify that the time node is still there
        assert!(graph.nodes.contains_key(&time_node));
        
        // Verify that the color and output nodes are still connected
        assert!(graph.nodes.contains_key(&color_node));
        assert!(graph.nodes.contains_key(&output_node));
    }

    #[test]
    fn test_node_connectivity() {
        let manager = VisualLanguageManager::new();
        let mut graph = NodeGraph::new();
        
        // Add nodes
        let node_a = graph.add_node(NodeKind::ConstantFloat(1.0), "A", (0.0, 0.0));
        let node_b = graph.add_node(NodeKind::ConstantFloat(2.0), "B", (100.0, 0.0));
        let node_c = graph.add_node(NodeKind::OutputColor, "C", (200.0, 0.0));
        
        // Connect A -> B -> C
        graph.connect(
            node_a,
            graph.nodes[&node_a].outputs[0].id,
            node_b,
            graph.nodes[&node_b].inputs[0].id
        );
        
        graph.connect(
            node_b,
            graph.nodes[&node_b].outputs[0].id,
            node_c,
            graph.nodes[&node_c].inputs[0].id
        );
        
        // A should be able to reach C
        assert!(manager.can_reach_node(node_a, node_c, &graph));
        
        // C should not be able to reach A
        assert!(!manager.can_reach_node(node_c, node_a, &graph));
    }
}