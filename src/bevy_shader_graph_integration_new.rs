use bevy::prelude::*;

pub struct ShaderGraphIntegrationPlugin;

impl Plugin for ShaderGraphIntegrationPlugin {
    fn build(&self, app: &mut App) {
        info!("ShaderGraphIntegrationPlugin initialized - using custom node graph implementation");
        // Use our custom implementation instead of external dependency
        app.add_plugins(crate::bevy_node_graph_integration::NodeGraphPlugin);
    }
}