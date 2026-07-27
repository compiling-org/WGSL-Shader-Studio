use wgsl_shader_studio::bevy_node_graph_integration_enhanced::{ShaderNodeGraph, ShaderNodeType};

// A simple smoke test to verify our visual node editor's WGSL codegen
#[test]
fn test_node_graph_codegen() {
    let mut graph = ShaderNodeGraph::default();

    // Create a time node
    let time_id = graph.add_node(
        ShaderNodeType::Time,
        "Time",
        vec![],
        vec!["time".to_string()],
    );

    // Create a math node (sin)
    let sin_id = graph.add_node(
        ShaderNodeType::Sin,
        "Sin",
        vec!["input".to_string()],
        vec!["result".to_string()],
    );

    // Connect time to sin
    assert!(graph.connect(time_id, 0, sin_id, 0).is_ok());

    // Generate WGSL
    let wgsl = graph.generate_wgsl().expect("Failed to generate WGSL");

    // Verify it contains the time uniform and the sin function call
    assert!(wgsl.contains("uniforms.time"));
    assert!(wgsl.contains("sin("));
}
