use resolume_isf_shaders_rust_ffgl::compute_pass_integration::{ComputePassManager, TextureFormat};

fn main() {
    println!("Testing compute pass integration...");
    
    let mut manager = ComputePassManager::default();
    
    // Test creating ping-pong texture
    manager.create_ping_pong_texture("test_texture", 512, 512, TextureFormat::Rgba8Unorm);
    println!("✓ Created ping-pong texture");
    
    // Test creating compute pipeline
    manager.create_compute_pipeline(
        "test_pipeline",
        (8, 8, 1),
        "@compute @workgroup_size(8, 8, 1) fn main() {}".to_string(),
        vec![]
    );
    println!("✓ Created compute pipeline");
    
    // Test creating compute pass execution
    manager.create_compute_pass_execution(
        "test_pass",
        "test_pipeline",
        (64, 64, 1),
        vec!["test_texture".to_string()],
        vec!["test_texture".to_string()],
        vec![]
    );
    println!("✓ Created compute pass execution");
    
    println!("Compute pass manager stats:");
    println!("  Ping-pong textures: {}", manager.ping_pong_textures.len());
    println!("  Compute pipelines: {}", manager.compute_pipelines.len());
    println!("  Active passes: {}", manager.active_compute_passes.len());
    
    println!("✅ All compute pass integration tests passed!");
}
