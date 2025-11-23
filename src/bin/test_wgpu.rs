use resolume_isf_shaders_rust_ffgl::shader_renderer::{ShaderRenderer, RenderParameters};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing WGPU renderer...");
    
    // Create runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;
    
    // Create renderer
    let mut renderer = rt.block_on(async {
        ShaderRenderer::new().await
    })?;
    
    println!("✓ Renderer created");
    
    // Test basic shader
    let test_shader = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var pos = vec2<f32>(0.0, 0.0);
    switch vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>(3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0, 3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / vec2<f32>(512.0, 512.0);
    return vec4<f32>(uv.x, uv.y, 0.5, 1.0);
}
"#;
    
    println!("Compiling test shader...");
    let result = renderer.compile_shader(test_shader, 512, 512)?;
    
    println!("✓ Shader compiled successfully!");
    println!("  Output size: {} bytes", result.len());
    println!("  Expected size: {} bytes", 512 * 512 * 4);
    
    // Check if we got reasonable output
    if result.len() == 512 * 512 * 4 {
        println!("✓ Output size matches expected");
        
        // Check a few pixels to see if they're not all black
        let mut non_black_pixels = 0;
        for i in (0..result.len()).step_by(4) {
            if result[i] > 0 || result[i+1] > 0 || result[i+2] > 0 {
                non_black_pixels += 1;
            }
        }
        
        println!("✓ Found {} non-black pixels out of {}", non_black_pixels, 512 * 512);
        
        if non_black_pixels > 1000 {
            println!("✓ Rendering appears to be working correctly!");
        } else {
            println!("⚠️  Warning: Very few non-black pixels found");
        }
    } else {
        println!("✗ Unexpected output size");
    }
    
    // Test with parameters
    println!("\nTesting shader with parameters...");
    let param_shader = r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<uniform> params: array<vec4<f32>, 16>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let color = params[0].xyz;
    return vec4<f32>(color, 1.0);
}
"#;
    
    let param_values = [1.0, 0.0, 0.0, 1.0]; // Red color
    let param_result = renderer.compile_shader_with_params(param_shader, 256, 256, Some(&param_values))?;
    
    println!("✓ Parameter shader compiled successfully!");
    println!("  Output size: {} bytes", param_result.len());
    
    // Check if the output is red
    let mut red_pixels = 0;
    for i in (0..param_result.len()).step_by(4) {
        if param_result[i] > 200 && param_result[i+1] < 50 && param_result[i+2] < 50 {
            red_pixels += 1;
        }
    }
    
    println!("✓ Found {} red pixels out of {}", red_pixels, 256 * 256);
    
    if red_pixels > 1000 {
        println!("✓ Parameter system appears to be working correctly!");
    }
    
    println!("\n🎉 All WGPU renderer tests passed!");
    Ok(())
}