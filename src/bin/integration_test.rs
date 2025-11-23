use resolume_isf_shaders_rust_ffgl::editor_ui::{EditorUiState, check_wgsl_diagnostics, DiagnosticSeverity};
use resolume_isf_shaders_rust_ffgl::audio_system::AudioAnalyzer;
use resolume_isf_shaders_rust_ffgl::timeline::{TimelineAnimation, InterpolationType};

fn main() {
    println!("🧪 Testing WGSL Shader Studio Integrations");
    println!("==========================================");
    
    // Test 1: WGSL Diagnostics Integration
    println!("\n1. Testing WGSL Diagnostics...");
    test_wgsl_diagnostics();
    
    // Test 2: Audio System Integration
    println!("\n2. Testing Audio System Integration...");
    test_audio_integration();
    
    // Test 3: Timeline Animation Integration
    println!("\n3. Testing Timeline Animation Integration...");
    test_timeline_integration();
    
    // Test 4: Editor UI State Management
    println!("\n4. Testing Editor UI State Management...");
    test_editor_ui_state();
    
    // Test 5: Parameter System Integration
    println!("\n5. Testing Parameter System Integration...");
    test_parameter_system();
    
    println!("\n✅ All integration tests completed!");
}

fn test_wgsl_diagnostics() {
    // Test valid WGSL shader
    let valid_shader = r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 3.0,  1.0),
    );
    let pos = positions[vertex_index];
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / uniforms.resolution;
    return vec4<f32>(uv.x, uv.y, 0.5, 1.0);
}
"#;

    let diagnostics = check_wgsl_diagnostics(valid_shader);
    println!("   Valid shader diagnostics: {} messages", diagnostics.len());
    
    // Test invalid WGSL shader
    let invalid_shader = r#"
struct Uniforms {
    time: float,  // Wrong: should be f32
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms
// Missing semicolon

@vertex
fn vs_main(vertex_index: u32) -> vec4<f32> {
    // Missing @builtin(position)
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

// Missing @fragment entry point
"#;

    let diagnostics = check_wgsl_diagnostics(invalid_shader);
    println!("   Invalid shader diagnostics: {} messages", diagnostics.len());
    
    for (i, diagnostic) in diagnostics.iter().enumerate() {
        match diagnostic.severity {
            DiagnosticSeverity::Error => println!("   ❌ Error {}: {}", i + 1, diagnostic.message),
            DiagnosticSeverity::Warning => println!("   ⚠️  Warning {}: {}", i + 1, diagnostic.message),
            DiagnosticSeverity::Info => println!("   ℹ️  Info {}: {}", i + 1, diagnostic.message),
        }
        if let Some(line) = diagnostic.line {
            println!("      Line: {}", line);
        }
    }
}

fn test_audio_integration() {
    // Create audio analyzer
    let audio_analyzer = AudioAnalyzer::default();
    
    // Test audio data access
    let audio_data = audio_analyzer.get_audio_data();
    println!("   Audio data accessed successfully");
    println!("   Volume: {}", audio_data.volume);
    println!("   Bass level: {}", audio_data.bass_level);
    println!("   Beat detected: {}", audio_data.beat_detected);
    println!("   Waveform length: {}", audio_data.waveform.len());
    println!("   Frequencies length: {}", audio_data.frequencies.len());
}

fn test_timeline_integration() {
    // Create timeline animation
    let mut timeline = TimelineAnimation::default();
    
    // Add some keyframes
    timeline.timeline.add_keyframe("time", 0.0, 0.0, InterpolationType::Linear);
    timeline.timeline.add_keyframe("time", 1.0, 1.0, InterpolationType::Linear);
    timeline.timeline.add_keyframe("audio_volume", 0.0, 0.5, InterpolationType::Linear);
    timeline.timeline.add_keyframe("audio_volume", 2.0, 1.0, InterpolationType::EaseInOut);
    
    println!("   Timeline created with {} tracks", timeline.timeline.tracks.len());
    
    // Test animation at different times
    for time in [0.0, 0.5, 1.0, 1.5, 2.0].iter() {
        let time_value = timeline.timeline.evaluate("time", *time, 0.0);
        let volume_value = timeline.timeline.evaluate("audio_volume", *time, 0.0);
        println!("   Time {}: time={:.2}, volume={:.2}", time, time_value, volume_value);
    }
}

fn test_editor_ui_state() {
    // Create editor UI state
    let mut ui_state = EditorUiState::default();
    
    // Test parameter setting
    ui_state.set_parameter_value("test_param", 0.75);
    println!("   Parameter 'test_param' set to: {:?}", ui_state.get_parameter_value("test_param"));
    
    // Test WGSLSmith integration
    ui_state.wgsl_smith_prompt = "Create a colorful plasma shader".to_string();
    println!("   WGSLSmith prompt set: '{}'", ui_state.wgsl_smith_prompt);
    
    // Test diagnostics messages
    ui_state.diagnostics_messages.push(resolume_isf_shaders_rust_ffgl::editor_ui::DiagnosticMessage {
        severity: DiagnosticSeverity::Warning,
        message: "Test diagnostic message".to_string(),
        line: Some(42),
        column: Some(10),
    });
    println!("   Diagnostics messages count: {}", ui_state.diagnostics_messages.len());
}

fn test_parameter_system() {
    // Test parameter parsing
    let test_shader = r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> params: array<vec4<f32>, 16>;
"#;
    
    let parameters = resolume_isf_shaders_rust_ffgl::editor_ui::parse_shader_parameters(test_shader);
    println!("   Parsed {} parameters from shader", parameters.len());
    
    for param in parameters.iter() {
        println!("   Parameter: {} (type: {})", param.name, param.wgsl_type);
    }
}