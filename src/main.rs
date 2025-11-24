//! Standalone application for testing ISF shaders

#[cfg(feature = "gui")]
mod bevy_app;
#[cfg(feature = "gui")]
mod audio_system;

// Direct module imports for the binary
mod isf_loader;
mod shader_converter;
mod gesture_control;
mod shader_renderer;
mod editor_ui;
mod visual_node_editor;
mod visual_node_editor_adapter;
mod node_graph;
mod timeline;
mod isf_converter;
mod converter;
mod compute_pass_integration;
mod screenshot_video_export;
mod scene_editor_3d;

// Re-export for easier access
use isf_loader::*;
use shader_converter::*;
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
    use bevy::prelude::*;
    
    println!("Starting Bevy app with egui integration and space_editor 3D scene management...");
    
    // Create the Bevy app with all necessary plugins and systems
    let mut app = App::new();
    
    // Add default plugins with window settings
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "WGSL Shader Studio".to_string(),
            resolution: (1280.0, 720.0).into(),
            ..default()
        }),
        ..default()
    }));
    
    // Add egui plugin
    app.add_plugins(bevy_egui::EguiPlugin);
    
    // Add our custom systems and resources
    app.init_resource::<super::editor_ui::EditorUiState>()
        .init_resource::<super::audio_system::AudioAnalyzer>()
        .init_resource::<super::compute_pass_integration::ComputePassManager>()
        .init_resource::<super::gesture_control::GestureControlSystem>()
        .init_resource::<super::scene_editor_3d::SceneEditor3DState>()
        .add_plugins(super::scene_editor_3d::SceneEditor3DPlugin)
        .add_systems(Startup, super::bevy_app::setup_camera)
        .add_systems(Update, (
            super::bevy_app::editor_ui_system,
            super::bevy_app::audio_system,
            super::bevy_app::timeline_system,
            super::bevy_app::compute_pass_system,
            super::bevy_app::gesture_control_system,
            super::scene_editor_3d::scene_editor_3d_ui,
            super::scene_editor_3d::scene_3d_viewport_ui,
        ));
    
    println!("Running Bevy app with space_editor 3D scene management...");
    app.run();
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
        file_path => {
            println!("Processing shader file: {}", file_path);
            process_shader_file(file_path);
        }
    }
}

fn test_compute_pass() {
    use std::sync::{Arc, Mutex};
    
    println!("Initializing compute pass manager...");
    let compute_manager = Arc::new(Mutex::new(super::compute_pass_integration::ComputePassManager::new()));
    
    // Test basic compute pass creation
    let mut manager = compute_manager.lock().unwrap();
    
    // Create a simple compute shader
    let compute_shader = r#"
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
    
    match manager.create_compute_pass("test_pass", compute_shader) {
        Ok(_) => println!("✓ Compute pass created successfully"),
        Err(e) => println!("✗ Failed to create compute pass: {}", e),
    }
    
    println!("Compute pass test completed.");
}

fn test_audio_system() {
    use std::sync::{Arc, Mutex};
    
    println!("Initializing audio system...");
    let audio_analyzer = Arc::new(Mutex::new(super::audio_system::AudioAnalyzer::new()));
    
    // Simulate some audio data
    let mut analyzer = audio_analyzer.lock().unwrap();
    
    // Test FFT processing
    let test_audio = vec![0.0f32; 1024];
    analyzer.process_audio_data(&test_audio);
    
    println!("Audio system test completed.");
}

fn test_node_graph() {
    use std::sync::{Arc, Mutex};
    
    println!("Initializing node graph...");
    let node_graph = Arc::new(Mutex::new(super::node_graph::NodeGraph::new()));
    
    let mut graph = node_graph.lock().unwrap();
    
    // Test adding nodes
    let noise_node = graph.add_node(super::node_graph::NodeKind::Noise2D);
    let sine_node = graph.add_node(super::node_graph::NodeKind::SineWave);
    
    println!("Added nodes: Noise2D ({}), SineWave ({})", noise_node, sine_node);
    
    // Test connecting nodes
    match graph.connect_nodes(noise_node, 0, sine_node, 0) {
        Ok(_) => println!("✓ Nodes connected successfully"),
        Err(e) => println!("✗ Failed to connect nodes: {}", e),
    }
    
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
                match super::isf_loader::IsfShader::parse(file_path, &content) {
                    Ok(isf_shader) => {
                        println!("ISF shader parsed successfully");
                        println!("Shader name: {}", isf_shader.name);
                        println!("Inputs: {}", isf_shader.inputs.len());
                        
                        // Try to convert to WGSL
                        let mut converter = super::isf_converter::IsfConverter::new();
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
                match super::wgsl_diagnostics::check_wgsl_diagnostics(&content) {
                    diagnostics => {
                        if diagnostics.is_empty() {
                            println!("✓ WGSL shader appears valid");
                        } else {
                            println!("⚠ WGSL shader has {} diagnostic(s):", diagnostics.len());
                            for (i, diagnostic) in diagnostics.iter().enumerate() {
                                println!("  {}: {}", i + 1, diagnostic.message);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => println!("✗ Failed to read file: {}", e),
    }
}