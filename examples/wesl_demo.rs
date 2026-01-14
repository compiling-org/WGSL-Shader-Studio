//! WESL Integration Demo
//! Demonstrates how to use the WESL integration in the shader studio

use wgsl_shader_studio::wesl_integration::WeslCompiler;
use wgsl_shader_studio::advanced_shader_compilation::{AdvancedShaderCompiler, ShaderFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 WESL Integration Demo");
    
    // Example 1: Using the WESL compiler directly
    println!("\n1. Direct WESL Compilation:");
    let mut wesl_compiler = WeslCompiler::new();
    let wesl_source = r#"
        // Simple WESL shader with conditional compilation
        #if DEBUG
            @group(0) @binding(0) var<uniform> debug_mode: f32;
        #else
            @group(0) @binding(0) var<uniform> time: f32;
        #endif
        
        @fragment
        fn main() -> @location(0) vec4<f32> {
            #if DEBUG
                return vec4<f32>(debug_mode, 0.0, 0.0, 1.0);
            #else
                return vec4<f32>(sin(time), cos(time), 0.5, 1.0);
            #endif
        }
    "#;
    
    match wesl_compiler.compile_wesl_to_wgsl(wesl_source, "demo.wesl") {
        Ok(wgsl_result) => {
            println!("✅ WESL compilation successful!");
            println!("Generated WGSL:\n{}", wgsl_result);
        }
        Err(e) => {
            eprintln!("❌ WESL compilation failed: {}", e);
        }
    }
    
    // Example 2: Using the advanced compiler with WESL format
    println!("\n2. Advanced Compiler with WESL:");
    let mut advanced_compiler = AdvancedShaderCompiler::new();
    
    match advanced_compiler.compile_shader(wesl_source, ShaderFormat::WESL, "wesl_demo").await {
        Ok(compiled_shader) => {
            println!("✅ Advanced WESL compilation successful!");
            println!("Shader name: {}", compiled_shader.metadata.name);
            println!("WGSL code length: {} chars", compiled_shader.wgsl_code.len());
            println!("Entry points: {:?}", compiled_shader.entry_points);
        }
        Err(e) => {
            eprintln!("❌ Advanced WESL compilation failed: {}", e);
        }
    }
    
    // Example 3: Show diagnostics
    println!("\n3. Diagnostics:");
    let diagnostics = wesl_compiler.get_diagnostics();
    if diagnostics.has_errors() {
        println!("❌ Found {} errors:", diagnostics.summary.error_count);
        for diagnostic in diagnostics.get_diagnostics() {
            println!("   - {}: {}", diagnostic.code, diagnostic.message);
        }
    } else {
        println!("✅ No errors found!");
    }
    
    if diagnostics.has_warnings() {
        println!("⚠️  Found {} warnings:", diagnostics.summary.warning_count);
        for diagnostic in diagnostics.get_diagnostics_by_severity(
            wgsl_shader_studio::converter::diagnostics::DiagnosticSeverity::Warning
        ) {
            println!("   - {}: {}", diagnostic.code, diagnostic.message);
        }
    }
    
    println!("\n🎉 WESL integration demo completed successfully!");
    Ok(())
}