use std::fs;

fn main() {
    println!("Minimal WGSL Shader Studio Test");
    
    // Try to read a simple shader file
    match fs::read_to_string("examples/default.wgsl") {
        Ok(content) => {
            println!("Successfully read shader file:");
            println!("Content length: {} characters", content.len());
            println!("First 100 characters: {}", &content[..100.min(content.len())]);
        }
        Err(e) => {
            println!("Error reading shader file: {}", e);
        }
    }
    
    println!("Test completed.");
}