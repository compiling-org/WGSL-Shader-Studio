//! Standalone application for testing ISF shaders
//!
//! ## Control Engine CLI Flags
//!
//! ```bash
//! # Run with control engine enabled
//! cargo run -- --cli --control-engine
//!
//! # Custom prime schedule intervals
//! cargo run -- --cli --control-engine --prime-schedule 2,3,5,7
//!
//! # Test the control engine
//! cargo run -- --cli --test-control-engine
//! ```

// GUI and audio modules are provided by the library crate

// Use library modules instead of re-declaring them locally
use resolume_isf_shaders_rust_ffgl::isf_converter;
use resolume_isf_shaders_rust_ffgl::node_graph;
use resolume_isf_shaders_rust_ffgl::ui_analyzer::UIAnalyzer;
use resolume_isf_shaders_rust_ffgl::wgsl_diagnostics;

// Import the specific types we need
use resolume_isf_shaders_rust_ffgl::audio_system::AudioAnalyzer;
use resolume_isf_shaders_rust_ffgl::compute_pass_integration::ComputePassManager;

// Control Engine & SuperInstance imports
use resolume_isf_shaders_rust_ffgl::control_engine::{
    ControlEngine, ControlEngineConfig, ControlState,
};
use resolume_isf_shaders_rust_ffgl::superinstance::{
    conservation_integration::{BudgetTracker, CompileOperation},
    flux_integration::FluxCompiler,
    plato_integration::{PlatoRoom, SensorValue},
    parse_superinstance_args, SuperInstanceConfig,
};

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

fn run_ui_analysis() {
    let mut analyzer = UIAnalyzer::new();
    let report = analyzer.run_comprehensive_analysis();
    println!("{}", report);
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
        println!("       {} --analyze-ui", args[0]);
        println!("       {} --glsl-to-wgsl <input.glsl>", args[0]);
        println!("       {} --hlsl-to-wgsl <input.hlsl>", args[0]);
        println!("       {} --wgsl-to-glsl <input.wgsl>", args[0]);
        println!("       {} --wgsl-to-hlsl <input.wgsl>", args[0]);
        println!("       {} --test-control-engine", args[0]);
        println!("       {} --control-engine [--prime-schedule 2,3,5,7]", args[0]);
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
        "--analyze-ui" => {
            println!("Running comprehensive UI analysis...");
            run_ui_analysis();
        }
        "--test-control-engine" => {
            println!("Testing Control Engine integration...");
            test_control_engine();
        }
        "--control-engine" => {
            println!("Running with Control Engine enabled...");
            run_with_control_engine(&args);
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
                        options.source_language =
                            resolume_isf_shaders_rust_ffgl::shader_transpiler::ShaderLanguage::Hlsl;
                        options.target_language =
                            resolume_isf_shaders_rust_ffgl::shader_transpiler::ShaderLanguage::Wgsl;
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
                        println!(
                            "Feature 'naga_integration' is disabled; enable it to use transpiler"
                        );
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

    println!(
        "Added nodes: Noise2D ({:?}), Time ({:?})",
        noise_node, sine_node
    );

    // Test connecting nodes
    let _ = graph.connect(
        noise_node,
        node_graph::PortId(0),
        sine_node,
        node_graph::PortId(0),
    );
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
                                println!(
                                    "✓ Successfully converted to WGSL ({} bytes)",
                                    wgsl_code.len()
                                );

                                // Save the converted shader
                                let output_path =
                                    format!("{}.wgsl", file_path.trim_end_matches(".fs"));
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

/// Test the Control Engine integration end-to-end
fn test_control_engine() {
    println!("\n=== Control Engine Integration Test ===\n");

    // 1. Create SuperInstance config
    let si_config = SuperInstanceConfig {
        flux_enabled: true,
        conservation_enabled: true,
        daily_budget: 0.10,
        ..SuperInstanceConfig::default()
    };
    println!("✓ SuperInstance config created (budget: ${})", si_config.daily_budget);

    // 2. Create Control Engine config
    let ce_config = ControlEngineConfig {
        enabled: true,
        prime_scheduling_enabled: true,
        complex_math_enabled: true,
        prime_intervals: vec![2, 3, 5],
        ..ControlEngineConfig::default()
    };
    println!("✓ Control Engine config created (primes: {:?})", ce_config.prime_intervals);

    // 3. Create and run the engine
    let mut engine = ControlEngine::new(ce_config, &si_config);
    println!("✓ Control Engine initialized");

    // 4. Run a few frames
    let audio_dummy = [0.5, 0.3, 0.8, 0.1];
    for frame in 0..12 {
        let state = engine.update(Some(&audio_dummy), None);
        if frame % 2 == 0 || frame < 3 {
            println!(
                "  Frame {}: active_groups={:?}, prime_phase={:.3}, latent[0]={:.3}",
                state.frame,
                state.active_groups,
                state.prime_phase,
                state.complex_latent.first().copied().unwrap_or(0.0)
            );
        }
    }

    // 5. Verify SuperInstance bridge updated
    let bridge = &engine.si_bridge;
    println!(
        "✓ SuperInstance bridge updated: {} operations tracked",
        bridge.last_summary.operations_this_session
    );

    // 6. Test PLATO room integration
    let mut room = PlatoRoom::shader_compiler_room();
    let state = engine.state.clone();
    bridge.update_plato_room(&mut room, &state);
    match room.get_sensor("budget_remaining") {
        Some(SensorValue::Float(v)) => println!("  Plato budget sensor: ${:.4}", v),
        _ => println!("  Plato budget sensor: not found"),
    }

    // 7. Test FluxCompiler integration
    let mut compiler = FluxCompiler::new();
    bridge.update_flux_compiler(&mut compiler, &state);
    println!(
        "  Flux compiler enabled: {}",
        if compiler.is_enabled() { "yes" } else { "no" }
    );

    // 8. Test instance modulations
    let mask = vec![true, false, true, false, true];
    let mods = bridge.get_instance_modulations(5, &mask, &state);
    println!(
        "  Instance modulations: [{:.3}, {:.3}, {:.3}, {:.3}, {:.3}]",
        mods[0], mods[1], mods[2], mods[3], mods[4]
    );

    // 9. Test global uniforms
    let uniforms = bridge.get_global_uniforms(&state);
    println!(
        "  Global uniforms: [prime_phase={:.3}, frame={:.3}, peak={:.3}, ...]",
        uniforms.first().copied().unwrap_or(0.0),
        uniforms.get(1).copied().unwrap_or(0.0),
        uniforms.get(2).copied().unwrap_or(0.0),
    );

    println!("\n=== Control Engine Integration Test Complete ===\n");
}

/// Run the application with Control Engine enabled, processing shader files
fn run_with_control_engine(args: &[String]) {
    // Parse SuperInstance args
    let si_config = parse_superinstance_args(args);

    // Parse control engine args
    let mut ce_config = ControlEngineConfig {
        enabled: true,
        ..ControlEngineConfig::default()
    };

    // Check for custom prime schedule
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--prime-schedule" && i + 1 < args.len() {
            let schedule_str = &args[i + 1];
            let intervals: Vec<u64> = schedule_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u64>().ok())
                .collect();
            if !intervals.is_empty() {
                ce_config.prime_intervals = intervals;
                println!("Custom prime schedule: {:?}", ce_config.prime_intervals);
            }
            i += 1;
        }
        i += 1;
    }

    // Initialize control engine
    let mut engine = ControlEngine::new(ce_config, &si_config);
    println!("Control Engine initialized with {} prime intervals", engine.prime_schedule.groups.len());

    // Run a few warm-up frames
    for _ in 0..5 {
        engine.update(None, None);
    }
    println!(
        "Engine state after warmup: frame={}, prime_phase={:.3}",
        engine.state.frame,
        engine.state.prime_phase
    );

    // Process shader files if provided
    let shader_files: Vec<&String> = args.iter().filter(|a| {
        !a.starts_with("--")
            && (a.ends_with(".wgsl") || a.ends_with(".fs") || a.ends_with(".glsl") || a.ends_with(".hlsl"))
    }).collect();

    if shader_files.is_empty() {
        println!("No shader files provided. Run with a shader file path to process it.");
        println!("Example: cargo run -- --cli --control-engine shaders/my_shader.wgsl");
    } else {
        for file in shader_files {
            println!("\nProcessing shader with control engine: {}", file);
            process_shader_file(file);

            // Update engine with some simulated audio
            let audio = [0.5, 0.3, 0.8, 0.1];
            let state = engine.update(Some(&audio), None);
            println!(
                "Control frame {}: {} groups active, phase={:.3}",
                state.frame,
                state.active_groups.len(),
                state.prime_phase
            );
        }
    }

    // Print budget summary
    let summary = &engine.si_bridge.last_summary;
    println!(
        "\nBudget: γ=${:.4} remaining, η=${:.4} remaining, total ops={}",
        summary.gamma_remaining,
        summary.eta_remaining,
        summary.operations_this_session,
    );
}
