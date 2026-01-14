use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::bevy_node_graph_integration_enhanced::{ShaderNodeGraph, NodeId, ShaderNodeType, PortType};
use crate::new_visual_node_editor::EnhancedVisualNodeEditor;

/// Plugin for enhanced visual node editor functionality
pub struct EnhancedVisualNodeEditorPlugin;

impl Plugin for EnhancedVisualNodeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnhancedVisualNodeEditorState>()
            .add_systems(Update, enhanced_visual_node_editor_ui);
    }
}

#[derive(Resource, Default)]
pub struct EnhancedVisualNodeEditorState {
    pub editor: EnhancedVisualNodeEditor,
    pub show_editor: bool,
    pub auto_compile: bool,
}

fn enhanced_visual_node_editor_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<EnhancedVisualNodeEditorState>,
    mut node_graph: ResMut<crate::bevy_node_graph_integration_enhanced::NodeGraphResource>, // Using NodeGraph from bevy_node_graph_integration_enhanced.rs
) {
    if !state.show_editor {
        return;
    }

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return, // Early return if context is not available
    };
    
    egui::Window::new("Enhanced Visual Node Editor")
        .default_size([800.0, 600.0])
        .resizable(true)
        .show(ctx, |ui| {
            // Control panel
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.editor.auto_compile(), "Auto Compile");
                ui.checkbox(&mut state.editor.show_grid(), "Show Grid");
                ui.checkbox(&mut state.editor.snap_to_grid(), "Snap to Grid");
                
                if ui.button("Compile").clicked() {
                    // The compile_node_graph method expects the old NodeGraph type
                    // We need to handle this differently since the types are incompatible
                    // For now, let's just generate WGSL directly from the enhanced graph
                    match node_graph.graph.generate_wgsl() {
                        Ok(wgsl) => {
                            // Update the shader in the main editor
                            // Note: Updating the main editor would require a custom event or shared resource
                // For now, we just print the generated WGSL
                println!("Generated WGSL:\n{}", wgsl);
                        }
                        Err(err) => {
                            println!("Compilation error: {}", err);
                        }
                    }
                }
                
                if ui.button("Validate").clicked() {
                    // Validation would need to be implemented differently for the new system
                    println!("Validation not implemented for new node graph system");
                }
                
                if ui.button("Add Time").clicked() {
                    // Add a time node using the new system
                    let _node_id = node_graph.graph.add_node(
                        ShaderNodeType::Time, 
                        "Time", 
                        vec![], 
                        vec!["time".to_string()]
                    );
                }
                
                if ui.button("Add UV").clicked() {
                    // Add a UV node using the new system
                    let _node_id = node_graph.graph.add_node(
                        ShaderNodeType::UV, 
                        "UV", 
                        vec![], 
                        vec!["uv".to_string()]
                    );
                }
                
                if ui.button("Add Output").clicked() {
                    // Add an output node using the new system
                    let _node_id = node_graph.graph.add_node(
                        ShaderNodeType::FragmentOutput, 
                        "Output", 
                        vec!["color".to_string()], 
                        vec![]
                    );
                }
            });
            
            ui.separator();
            
            // Main node editor canvas
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.set_min_height(500.0);
                // The UI method expects the old NodeGraph type, so we'll skip this for now
                // and provide a message to the user
                ui.label("Node graph editor interface needs to be updated to work with new system");
            });
            
            // Status panel
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Ready");
                
                ui.separator();
                ui.label(format!("Nodes: {}", node_graph.graph.nodes.len()));
                ui.label(format!("Connections: {}", node_graph.graph.connections.len()));
                ui.label("Zoom: 1.00x");
            });
        });
}