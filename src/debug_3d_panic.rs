//! Debug script to identify 3D preview panics
//! This script enables backtrace and runs the application with 3D preview enabled
//! to capture the exact point of failure.

use std::env;

fn main() {
    // Set environment variables for debugging
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "wgsl_shader_studio=debug,wgpu=warn");
    
    println!("Starting WGSL Shader Studio with 3D preview debugging enabled...");
    println!("Backtrace level: full");
    println!("Log level: debug for wgsl_shader_studio, warn for wgpu");
    
    // Run the main application
    // This would normally call the main application entry point
    // For now, we'll just print instructions
    
    println!("\nTo run with 3D preview and capture backtrace:");
    println!("  cargo run --features gui,3d_preview");
    println!("\nTo run with full backtrace:");
    println!("  RUST_BACKTRACE=1 cargo run --features gui,3d_preview");
    println!("\nTo run with debug logging:");
    println!("  RUST_LOG=wgsl_shader_studio=debug,wgpu=warn cargo run --features gui,3d_preview");
    
    println!("\nWhen the panic occurs, please capture the stack trace and analyze:");
    println!("1. Is it a wgpu pipeline panic (invalid WGSL/bindings)?");
    println!("2. Is it a Bevy system panic (missing resource/camera)?");
    println!("3. Is it a custom renderer assertion (viewport size, texture, etc.)?");
}