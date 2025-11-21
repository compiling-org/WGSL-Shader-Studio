//! Standalone application for testing ISF shaders

#[cfg(feature = "gui")]
mod bevy_app;
#[cfg(feature = "gui")]
mod audio;

// Direct module imports for the binary
mod isf_loader;
mod shader_converter;
mod gesture_control;
mod shader_renderer;
mod editor_ui;
mod node_graph;
mod timeline;
mod isf_converter;
mod converter;
#[cfg(feature = "gui")]
mod visual_node_editor;

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
        run_cli(args);
    } else {
        // Default to GUI mode when no explicit CLI flag
        #[cfg(feature = "gui")]
        {
            println!("Starting WGSL Shader Studio GUI...");
            bevy_app::run_app();
        }
        #[cfg(not(feature = "gui"))]
        {
            println!("GUI not available. Enable with: cargo run --features gui");
            println!("Falling back to CLI mode...");
            run_cli(args);
        }
    }
}

fn run_cli(args: Vec<String>) {
    println!("WGSL Shader Studio - CLI Mode");
    println!("==============================");

    // Check for explicit CLI flag first
    let has_cli_flag = args.contains(&"--cli".to_string());
    let command_index = if has_cli_flag { 2 } else { 1 };
    if args.len() <= command_index {
        print_usage();
        process::exit(1);
    }
    let command = &args[command_index];

    match command.as_str() {
        "list" => {
            // List available ISF shaders
            match load_resolume_isf_shaders() {
                Ok(shaders) => {
                    println!("Available ISF Shaders:");
                    for shader in &shaders {
                        let metadata = get_shader_metadata(shader);
                        println!("  - {} ({})", shader.name, metadata.category.unwrap_or_else(|| "Uncategorized".to_string()));
                        if let Some(desc) = metadata.description {
                            println!("    {}", desc);
                        }
                        println!("    Inputs: {}", shader.inputs.len());
                        for input in &shader.inputs {
                            println!("      - {} ({:?})", input.name, input.input_type);
                        }
                        println!();
                    }
                    println!("Total shaders loaded: {}", shaders.len());
                }
                Err(e) => {
                    eprintln!("Error loading shaders: {}", e);
                    process::exit(1);
                }
            }
        }
        "validate" => {
            if args.len() < 3 {
                eprintln!("Error: Please provide a shader file path");
                print_usage();
                process::exit(1);
            }

            let shader_path = &args[2];
            match load_isf_shader(&std::path::Path::new(shader_path)) {
                Ok(shader) => {
                    match validate_isf_shader(&shader) {
                        Ok(_) => {
                            println!("✓ Shader '{}' is valid", shader.name);
                            let metadata = get_shader_metadata(&shader);
                            println!("  Description: {}", metadata.description.unwrap_or_else(|| "No description".to_string()));
                            println!("  Category: {}", metadata.category.unwrap_or_else(|| "Uncategorized".to_string()));
                            println!("  Author: {}", metadata.author.unwrap_or_else(|| "Unknown".to_string()));
                            println!("  Inputs: {}", shader.inputs.len());
                            println!("  Outputs: {}", shader.outputs.len());
                        }
                        Err(e) => {
                            eprintln!("✗ Shader validation failed: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error loading shader: {}", e);
                    process::exit(1);
                }
            }
        }
        "convert" => {
            if args.len() < 4 {
                eprintln!("Error: Please provide input and output file paths");
                print_usage();
                process::exit(1);
            }

            let input_path = &args[2];
            let output_path = &args[3];

            match load_isf_shader(&std::path::Path::new(input_path)) {
                Ok(shader) => {
                    // Convert isf_loader::IsfShader to shader_converter::IsfShader
                    let converter_shader = shader_converter::IsfShader {
                        name: shader.name.clone(),
                        source: shader.source.clone(),
                        inputs: shader.inputs.iter().map(|input| shader_converter::ShaderInput {
                            name: input.name.clone(),
                            input_type: match input.input_type {
                                isf_loader::InputType::Float => shader_converter::InputType::Float,
                                isf_loader::InputType::Bool => shader_converter::InputType::Bool,
                                isf_loader::InputType::Color => shader_converter::InputType::Color,
                                isf_loader::InputType::Point2D => shader_converter::InputType::Point2D,
                                isf_loader::InputType::Image => shader_converter::InputType::Image,
                            },
                            value: match &input.value {
                                isf_loader::ShaderValue::Float(f) => shader_converter::ShaderValue::Float(*f),
                                isf_loader::ShaderValue::Bool(b) => shader_converter::ShaderValue::Bool(*b),
                                isf_loader::ShaderValue::Color(c) => shader_converter::ShaderValue::Color(*c),
                                isf_loader::ShaderValue::Point2D(p) => shader_converter::ShaderValue::Point2D(*p),
                            },
                            min: input.min,
                            max: input.max,
                            default: input.default,
                        }).collect(),
                        outputs: shader.outputs.iter().map(|output| shader_converter::ShaderOutput {
                            name: output.name.clone(),
                            output_type: match output.output_type {
                                isf_loader::OutputType::Image => shader_converter::OutputType::Image,
                                isf_loader::OutputType::Float => shader_converter::OutputType::Float,
                            },
                        }).collect(),
                    };

                    match isf_to_wgsl(&converter_shader) {
                        Ok(wgsl_code) => {
                            match std::fs::write(output_path, wgsl_code) {
                                Ok(_) => println!("✓ Converted '{}' to WGSL: {}", shader.name, output_path),
                                Err(e) => {
                                    eprintln!("Error writing WGSL file: {}", e);
                                    process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error converting shader to WGSL: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error loading shader: {}", e);
                    process::exit(1);
                }
            }
        }
        "info" => {
            println!("WGSL Shader Studio");
            println!("==================");
            println!("Professional WGPU shader development environment with ISF support.");
            println!();
            println!("Features:");
            println!("  - ISF (Interactive Shader Format) support");
            println!("  - WGSL/GLSL/HLSL shader conversion");
            println!("  - Real-time audio analysis and MIDI control");
            println!("  - Node-based visual programming");
            println!("  - Live preview with WGPU rendering");
            println!("  - Standalone shader validation and conversion tools");
            println!();
            println!("ISF Directories:");
            println!("  - C:\\Program Files\\Magic\\Modules2\\ISF\\fractal");
            println!("  - C:\\Program Files\\Magic\\Modules2\\ISF\\fractal 2");
            println!("  - C:\\Program Files\\Magic\\Modules2\\ISF\\final");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    let exe_name = env::args().next().unwrap_or_else(|| "wgsl-shader-studio".to_string());
    println!("WGSL Shader Studio - GUI-first Development Environment");
    println!("=====================================================");
    println!("Usage:");
    println!("  {} [options]                Start GUI application (default)", exe_name);
    println!("  {} [options] --cli <command> [args...]", exe_name);
    println!();
    println!("Options:");
    println!("  --gui                       Start graphical interface (default)");
    println!("  --cli                        Enable CLI mode for developer commands");
    println!("  --help                       Show this help message");
    println!();
    println!("GUI Features:");
    println!("  - Professional WGPU shader development environment");
    println!("  - Live preview with real-time rendering");
    println!("  - Node-based visual programming (32 node types)");
    println!("  - WGSL syntax highlighting and error detection");
    println!("  - Audio/MIDI reactive shader parameters");
    println!("  - ISF shader import/export and conversion");
    println!("  - Template library with tutorials");
    println!();
    println!("CLI Commands (Developer Mode):");
    println!("  list                        List all available ISF shaders");
    println!("  validate <file>             Validate an ISF shader file");
    println!("  convert <input> <output>    Convert ISF shader to WGSL");
    println!("  info                        Show application information");
    println!();
    println!("Examples:");
    println!("  {}                           # Start GUI (default)", exe_name);
    println!("  {} --cli list", exe_name);
    println!("  {} --cli validate shader.fs", exe_name);
    println!("  {} --cli convert input.fs output.wgsl", exe_name);
    println!();
    println!("For more information, see the documentation at:");
    println!("  https://github.com/compiling-org/WGSL-Shader-Studio");
}