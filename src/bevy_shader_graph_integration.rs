use bevy::prelude::*;
use bevy_shader_graph::prelude::*;
use crate::shader_renderer::{ShaderRenderer, RenderParameters};

/// Plugin that integrates bevy_shader_graph for shader preview rendering
pub struct ShaderGraphIntegrationPlugin;

impl Plugin for ShaderGraphIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderGraphPlugin)
            .init_resource::<ShaderGraphState>()
            .add_systems(Update, update_shader_preview);
    }
}

/// State for managing shader graph integration
#[derive(Resource, Default)]
pub struct ShaderGraphState {
    pub current_shader: Option<String>,
    pub render_target: Option<Handle<ShaderGraph>>,
    pub parameters: RenderParameters,
}

/// System to update shader preview using bevy_shader_graph
fn update_shader_preview(
    mut shader_graph_state: ResMut<ShaderGraphState>,
    mut shader_graph_assets: ResMut<Assets<ShaderGraph>>,
) {
    if let Some(wgsl_code) = &shader_graph_state.current_shader {
        // Create a shader graph from WGSL code
        let mut shader_graph = ShaderGraph::default();
        
        // Add a fragment shader node
        let fragment_node = shader_graph.add_node(ShaderNode::Fragment {
            code: wgsl_code.clone(),
            inputs: vec![],
            outputs: vec!["color".to_string()],
        });
        
        // Set the fragment output
        shader_graph.set_output("color", fragment_node, "color");
        
        // Store the shader graph
        let handle = shader_graph_assets.add(shader_graph);
        shader_graph_state.render_target = Some(handle);
    }
}

/// Function to compile WGSL shader to shader graph
pub fn compile_wgsl_to_shader_graph(wgsl_code: &str) -> Result<ShaderGraph, String> {
    let mut shader_graph = ShaderGraph::default();
    
    // Parse WGSL code and create appropriate nodes
    if wgsl_code.contains("@fragment") {
        // Extract fragment shader code
        let fragment_code = extract_fragment_shader(wgsl_code);
        
        let fragment_node = shader_graph.add_node(ShaderNode::Fragment {
            code: fragment_code,
            inputs: vec!["uv".to_string(), "time".to_string(), "resolution".to_string()],
            outputs: vec!["color".to_string()],
        });
        
        // Add input nodes for common uniforms
        let time_node = shader_graph.add_node(ShaderNode::Uniform {
            name: "time".to_string(),
            default_value: ShaderValue::Float(0.0),
        });
        
        let resolution_node = shader_graph.add_node(ShaderNode::Uniform {
            name: "resolution".to_string(),
            default_value: ShaderValue::Vec2([512.0, 512.0]),
        });
        
        let uv_node = shader_graph.add_node(ShaderNode::Uv {
            coordinate_space: CoordinateSpace::Screen,
        });
        
        // Connect nodes
        shader_graph.add_edge(uv_node, "uv", fragment_node, "uv");
        shader_graph.add_edge(time_node, "value", fragment_node, "time");
        shader_graph.add_edge(resolution_node, "value", fragment_node, "resolution");
        
        // Set output
        shader_graph.set_output("color", fragment_node, "color");
    }
    
    Ok(shader_graph)
}

fn extract_fragment_shader(wgsl_code: &str) -> String {
    // Simple extraction - in production, use proper WGSL parsing
    if wgsl_code.contains("@fragment") {
        wgsl_code.to_string()
    } else {
        format!("@fragment\nfn main() -> @location(0) vec4<f32> {{\n    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n}}")
    }
}

/// System to render shader graph to texture
pub fn render_shader_graph_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    shader_graph_state: Res<ShaderGraphState>,
    shader_graph_assets: Res<Assets<ShaderGraph>>,
) {
    if let Some(handle) = &shader_graph_state.render_target {
        if let Some(shader_graph) = shader_graph_assets.get(handle) {
            // Create a quad to render the shader
            let mesh = meshes.add(Mesh::from(Rectangle::new(2.0, 2.0)));
            
            // Create material with shader graph
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..Default::default()
            });
            
            // Spawn quad with shader graph material
            commands.spawn((
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::default(),
            ));
        }
    }
}