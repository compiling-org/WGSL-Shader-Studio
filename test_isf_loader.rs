use std::path::Path;

// Test the ISF loader functionality
fn main() {
    println!("Testing ISF loader functionality...");
    
    // Test loading ISF shaders from the Magic directory
    match resolume_isf_shaders_rust_ffgl::isf_loader::load_resolume_isf_shaders() {
        Ok(shaders) => {
            println!("Successfully loaded {} ISF shaders", shaders.len());
            
            // Show first few shaders as examples
            for (i, shader) in shaders.iter().take(5).enumerate() {
                println!("Shader {}: {} (inputs: {})", 
                    i + 1, 
                    shader.name, 
                    shader.inputs.len()
                );
                
                // Show input details
                for input in &shader.inputs {
                    println!("  - Input: {} (type: {:?})", input.name, input.input_type);
                }
            }
            
            if shaders.len() > 5 {
                println!("... and {} more shaders", shaders.len() - 5);
            }
        }
        Err(e) => {
            println!("Failed to load ISF shaders: {}", e);
        }
    }
    
    // Test loading from specific directory
    let magic_dir = r"C:\Program Files\Magic\Modules2\ISF";
    if Path::new(magic_dir).exists() {
        println!("\nTesting direct load from Magic directory...");
        match resolume_isf_shaders_rust_ffgl::isf_loader::load_isf_shaders_from_directory(magic_dir) {
            Ok(shaders) => {
                println!("Loaded {} shaders directly from Magic directory", shaders.len());
            }
            Err(e) => {
                println!("Failed to load from Magic directory: {}", e);
            }
        }
    } else {
        println!("\nMagic directory not found at: {}", magic_dir);
    }
    
    // Test local ISF directory
    let local_dir = "./isf-shaders";
    if Path::new(local_dir).exists() {
        println!("\nTesting load from local ISF directory...");
        match resolume_isf_shaders_rust_ffgl::isf_loader::load_isf_shaders_from_directory(local_dir) {
            Ok(shaders) => {
                println!("Loaded {} shaders from local directory", shaders.len());
            }
            Err(e) => {
                println!("Failed to load from local directory: {}", e);
            }
        }
    } else {
        println!("\nLocal ISF directory not found at: {}", local_dir);
    }
}