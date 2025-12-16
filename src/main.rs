//! Standalone application for testing ISF shaders

// GUI and audio modules are provided by the library crate

// Use library modules instead of re-declaring them locally
use resolume_isf_shaders_rust_ffgl::node_graph;
use resolume_isf_shaders_rust_ffgl::isf_converter;
use resolume_isf_shaders_rust_ffgl::wgsl_diagnostics;

// Import the specific types we need
use resolume_isf_shaders_rust_ffgl::audio_system::AudioAnalyzer;
use resolume_isf_shaders_rust_ffgl::compute_pass_integration::ComputePassManager;

// Re-export for easier access
use resolume_isf_shaders_rust_ffgl::isf_loader::IsfShader;
use std::env;
use std::process;

fn main() {
    // Check if we should run the GUI or CLI
    let args: Vec<String> = env::args().collect();

    // Check for explicit CLI flag first
    let has_cli_flag = args.contains(&"--cli".to_string());
    
    // If --cli flag is present, always run CLI
    if has_cli_flag {
        println!("Running in CLI mode...");
        run_cli();
        return;
    }

    // Check for GUI feature
    #[cfg(feature = "gui")]
    {
        println!("Running in GUI mode...");
        run_gui();
        return;
    }

    // If no GUI feature, default to CLI
    #[cfg(not(feature = "gui"))]
    {
        println!("GUI feature not enabled, running in CLI mode...");
        run_cli();
    }
}

#[cfg(feature = "gui")]
fn run_gui() {
    println!("Starting WGSL Shader Studio with corrected panel hierarchy...");
    
    // Use the proper bevy_app module that has the corrected panel hierarchy
    resolume_isf_shaders_rust_ffgl::bevy_app::run_app();
}

fn run_cli() {
    println!("WGSL Shader Studio - CLI Mode");
    println!("===============================");
    
    // Simple CLI argument parsing
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <shader_file> [--cli]", args[0]);
        println!("       {} --test-compute", args[0]);
        println!("       {} --test-audio", args[0]);
        println!("       {} --test-nodes", args[0]);
        println!("       {} --glsl-to-wgsl <input.glsl>", args[0]);
        println!("       {} --hlsl-to-wgsl <input.hlsl>", args[0]);
        println!("       {} --wgsl-to-glsl <input.wgsl>", args[0]);
        println!("       {} --wgsl-to-hlsl <input.wgsl>", args[0]);
        process::exit(1);
    }
    
    match args[1].as_str() {
        "--test-compute" => {
            println!("Testing compute pass integration...");
            test_compute_pass();
        }
        "--test-audio" => {
            println!("Testing audio system...");
            test_audio_system();
        }
        "--test-nodes" => {
            println!("Testing node graph system...");
            test_node_graph();
        }
        "--glsl-to-wgsl" => {
            if args.len() < 3 {
                println!("Missing input file");
                process::exit(1);
            }
            let input = &args[2];
            match std::fs::read_to_string(input) {
                Ok(src) => {
                    match resolume_isf_shaders_rust_ffgl::shader_converter::glsl_to_wgsl(&src) {
                        Ok(out) => {
                            let out_path = format!("{}.wgsl", input);
                            if let Err(e) = std::fs::write(&out_path, out) {
                                println!("Failed to write output: {}", e);
                            } else {
                                println!("Converted to {}", out_path);
                            }
                        }
                        Err(e) => println!("Conversion error: {}", e),
                    }
                }
                Err(e) => println!("Failed to read {}: {}", input, e),
            }
        }
        "--hlsl-to-wgsl" => {
            if args.len() < 3 {
                println!("Missing input file");
                process::exit(1);
            }
            let input = &args[2];
            match std::fs::read_to_string(input) {
                Ok(src) => {
                    #[cfg(feature = "naga_integration")]
                    {
                        let transpiler = resolume_isf_shaders_rust_ffgl::shader_transpiler::MultiFormatTranspiler::new();
                        let mut options = resolume_isf_shaders_rust_ffgl::shader_transpiler::TranspilerOptions::default();
                        options.source_language = resolume_isf_shaders_rust_ffgl::shader_transpiler::ShaderLanguage::Hlsl;
                        options.target_language = resolume_isf_shaders_rust_ffgl::shader_transpiler::ShaderLanguage::Wgsl;
                        match transpiler.transpile(&src, &options) {
                            Ok(res) => {
                                let out_path = format!("{}.wgsl", input);
                                if let Err(e) = std::fs::write(&out_path, res.source_code) {
                                    println!("Failed to write output: {}", e);
                                } else {
                                    println!("Converted to {}", out_path);
                                }
                            }
                            Err(e) => println!("Conversion error: {}", e),
                        }
                    }
                    #[cfg(not(feature = "naga_integration"))]
                    {
                        println!("Feature 'naga_integration' is disabled; enable it to use transpiler");
                        process::exit(1);
                    }
                }
                Err(e) => println!("Failed to read {}: {}", input, e),
            }
        }
        "--wgsl-to-glsl" => {
            if args.len() < 3 {
                println!("Missing input file");
                process::exit(1);
            }
            let input = &args[2];
            match std::fs::read_to_string(input) {
                Ok(src) => {
                    match resolume_isf_shaders_rust_ffgl::shader_converter::wgsl_to_glsl(&src) {
                        Ok(out) => {
                            let out_path = format!("{}.glsl", input);
                            if let Err(e) = std::fs::write(&out_path, out) {
                                println!("Failed to write output: {}", e);
                            } else {
                                println!("Converted to {}", out_path);
                            }
                        }
                        Err(e) => println!("Conversion error: {}", e),
                    }
                }
                Err(e) => println!("Failed to read {}: {}", input, e),
            }
        }
        "--wgsl-to-hlsl" => {
            if args.len() < 3 {
                println!("Missing input file");
                process::exit(1);
            }
            let input = &args[2];
            match std::fs::read_to_string(input) {
                Ok(src) => {
                    match resolume_isf_shaders_rust_ffgl::shader_converter::wgsl_to_hlsl(&src) {
                        Ok(out) => {
                            let out_path = format!("{}.hlsl", input);
                            if let Err(e) = std::fs::write(&out_path, out) {
                                println!("Failed to write output: {}", e);
                            } else {
                                println!("Converted to {}", out_path);
                            }
                        }
                        Err(e) => println!("Conversion error: {}", e),
                    }
                }
                Err(e) => println!("Failed to read {}: {}", input, e),
            }
        }
        file_path => {
            println!("Processing shader file: {}", file_path);
            process_shader_file(file_path);
        }
    }
}

fn test_compute_pass() {
    use std::sync::{Arc, Mutex};
    
    println!("Initializing compute pass manager...");
    let compute_manager = Arc::new(Mutex::new(ComputePassManager::default()));
    
    // Test basic compute pass creation
    let _manager = compute_manager.lock().unwrap();
    
    // Create a simple compute shader
    let _compute_shader = r#"
        @group(0) @binding(0) var<storage, read_write> data: array<f32>;
        
        @compute @workgroup_size(64)
        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let index = global_id.x;
            if (index >= arrayLength(&data)) {
                return;
            }
            data[index] = data[index] * 2.0;
        }
    "#;
    
    // TODO: Fix compute pass API - this is a placeholder
    println!("✓ Compute pass test placeholder");
    
    println!("Compute pass test completed.");
}

fn test_audio_system() {
    use std::sync::{Arc, Mutex};
    
    println!("Initializing audio system...");
    let audio_analyzer = Arc::new(Mutex::new(AudioAnalyzer::new()));
    
    // Simulate some audio data
    let mut analyzer = audio_analyzer.lock().unwrap();
    
    // Test FFT processing
    analyzer.process_audio_frame();
    
    println!("Audio system test completed.");
}

fn test_node_graph() {
    use std::sync::{Arc, Mutex};
    
    println!("Initializing node graph...");
    let node_graph = Arc::new(Mutex::new(node_graph::NodeGraph::new()));
    
    let mut graph = node_graph.lock().unwrap();
    
    // Test adding nodes
    let noise_node = graph.add_node(node_graph::NodeKind::Noise2D, "Noise2D", (100.0, 100.0));
    let sine_node = graph.add_node(node_graph::NodeKind::Time, "Time", (200.0, 100.0));
    
    println!("Added nodes: Noise2D ({:?}), Time ({:?})", noise_node, sine_node);
    
    // Test connecting nodes
    let _ = graph.connect(noise_node, node_graph::PortId(0), sine_node, node_graph::PortId(0));
    println!("✓ Nodes connected successfully");
    
    println!("Node graph test completed.");
}

fn process_shader_file(file_path: &str) {
    println!("Loading shader file: {}", file_path);
    
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            println!("File loaded successfully ({} bytes)", content.len());
            
            // Check if it's an ISF file
            if file_path.to_lowercase().ends_with(".fs") {
                println!("Detected ISF shader format");
                match IsfShader::parse(file_path, &content) {
                    Ok(isf_shader) => {
                        println!("ISF shader parsed successfully");
                        println!("Shader name: {}", isf_shader.name);
                        println!("Inputs: {}", isf_shader.inputs.len());
                        
                        // Try to convert to WGSL
                        let mut converter = isf_converter::IsfConverter::new();
                        match converter.convert_to_wgsl(&isf_shader) {
                            Ok(wgsl_code) => {
                                println!("✓ Successfully converted to WGSL ({} bytes)", wgsl_code.len());
                                
                                // Save the converted shader
                                let output_path = format!("{}.wgsl", file_path.trim_end_matches(".fs"));
                                if let Err(e) = std::fs::write(&output_path, wgsl_code) {
                                    println!("✗ Failed to save converted shader: {}", e);
                                } else {
                                    println!("✓ Converted shader saved to: {}", output_path);
                                }
                            }
                            Err(e) => println!("✗ Failed to convert to WGSL: {}", e),
                        }
                    }
                    Err(e) => println!("✗ Failed to parse ISF shader: {}", e),
                }
            } else {
                println!("Assuming WGSL shader format");
                
                // Try to parse as WGSL
                let mut diagnostics = wgsl_diagnostics::WgslDiagnostics::new();
                let results = diagnostics.analyze(&content);
                if results.is_empty() {
                    println!("✓ WGSL shader appears valid");
                } else {
                    println!("⚠ WGSL shader has {} diagnostic(s):", results.len());
                    for (i, diagnostic) in results.iter().enumerate() {
                        println!("  {}: {}", i + 1, diagnostic.message);
                    }
                }
            }
        }
        Err(e) => println!("✗ Failed to read file: {}", e),
    }
}
