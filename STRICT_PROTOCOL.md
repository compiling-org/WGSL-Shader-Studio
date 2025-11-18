# STRICT DEVELOPMENT PROTOCOL - WGSL SHADER STUDIO

## ABSOLUTE RULES - ZERO TOLERANCE

### 1. NO MORE TEST FILES OR DEMO CODE
- **NEVER** create files with names like `test_`, `demo_`, `simple_`, `example_`
- **IMMEDIATE FAILURE** if any test/demo file is created
- All code must be production-ready implementation

### 2. ALL FEATURES PERMANENTLY ENABLED
- **NO CONDITIONAL COMPILATION** - Remove all `#[cfg(feature = "...")]` 
- **NO TOGGLE FLAGS** - Remove all `if enabled` checks
- **NO COMMENTING OUT** - Fix errors, don't disable features
- **EVERYTHING ACTIVE** - All panels, all modules, all functionality

### 3. STRICT MODULE INTEGRATION PROTOCOL
```rust
// REQUIRED: All modules declared and active
pub mod audio;
pub mod gesture_control;
pub mod shader_converter;
pub mod shader_renderer;
pub mod isf_loader;
pub mod isf_converter;
pub mod ffgl_plugin;
pub mod ui;
pub mod wgsl_bindgen_integration;
pub mod wgsl_diagnostics;
pub mod isf_auto_converter;
pub mod isf_conversion_tester;
pub mod wgsl_reflect_integration;
pub mod wgslsmith_integration;
pub mod converter;
pub mod editor_ui;
pub mod simple_ui_auditor;
pub mod node_graph;
pub mod timeline;
pub mod ui_analyzer;
pub mod visual_node_editor;
pub mod bevy_shader_graph_integration;
pub mod wgpu_integration;
pub mod bevy_node_graph_integration;
pub mod shader_browser;

// REQUIRED: All modules re-exported
pub use audio::*;
pub use gesture_control::*;
pub use shader_converter::*;
pub use shader_renderer::*;
pub use isf_loader::*;
pub use ffgl_plugin::*;
pub use wgpu_integration::*;
pub use visual_node_editor::*;
pub use bevy_node_graph_integration::*;
pub use shader_browser::*;
```

### 4. UI IMPLEMENTATION PROTOCOL
```rust
// REQUIRED: All UI panels permanently enabled
ui_state.show_shader_browser = true;
ui_state.show_parameter_panel = true;
ui_state.show_preview = true;
ui_state.show_code_editor = true;
ui_state.show_node_studio = true;      // PERMANENTLY ENABLED
ui_state.show_timeline = true;         // PERMANENTLY ENABLED
ui_state.show_audio_panel = true;      // PERMANENTLY ENABLED
ui_state.show_midi_panel = true;       // PERMANENTLY ENABLED
ui_state.show_gesture_panel = true;    // PERMANENTLY ENABLED
ui_state.show_error_panel = true;      // PERMANENTLY ENABLED
```

### 5. ERROR FIXING PROTOCOL
1. **IDENTIFY ROOT CAUSE** - Don't disable, find the real problem
2. **FIX THE ERROR** - Implement proper solution
3. **TEST COMPILATION** - Ensure it builds
4. **VERIFY FUNCTIONALITY** - Confirm feature works
5. **NEVER COMMENT OUT** - This is failure

### 6. WGPU RENDERING PIPELINE - COMPLETE IMPLEMENTATION
```rust
// REQUIRED: Full WGPU integration with all features
impl Plugin for WgpuRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WgpuRenderPipeline>()
            .add_systems(Update, (
                shader_compilation_system,
                render_frame_system,
                update_uniforms_system,
                handle_shader_errors_system,
            ));
    }
}
```

### 7. SHADER COMPILATION PROTOCOL
```rust
// REQUIRED: Immediate compilation with error handling
fn compile_shader(code: &str) -> Result<ShaderModule, String> {
    // Real compilation - no placeholders
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("WGSL Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
    });
    
    // Real error checking
    match shader_module {
        Ok(module) => Ok(module),
        Err(e) => Err(format!("Shader compilation failed: {}", e)),
    }
}
```

### 8. VISUAL NODE EDITOR - COMPLETE IMPLEMENTATION
```rust
// REQUIRED: Full node graph with GitHub repository patterns
pub struct ShaderNodeGraph {
    pub nodes: HashMap<NodeId, ShaderNode>,
    pub connections: Vec<NodeConnection>,
}

impl ShaderNodeGraph {
    pub fn generate_wgsl(&self) -> Result<String, String> {
        // Real WGSL generation from node graph
        // Based on bevy_shader_graph and nodus patterns
        let mut wgsl = String::new();
        
        // Generate uniforms
        wgsl.push_str("struct Uniforms {\n");
        wgsl.push_str("    time: f32,\n");
        wgsl.push_str("    resolution: vec2<f32>,\n");
        wgsl.push_str("}\n\n");
        
        // Generate main function from nodes
        wgsl.push_str("@fragment\n");
        wgsl.push_str("fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {\n");
        
        // Process nodes in topological order
        for node_id in self.topological_sort()? {
            if let Some(node) = self.nodes.get(&node_id) {
                self.generate_node_code(node, &mut wgsl)?;
            }
        }
        
        wgsl.push_str("}\n");
        Ok(wgsl)
    }
}
```

### 9. AUDIO/MIDI INTEGRATION - COMPLETE
```rust
// REQUIRED: Real audio analysis with FFT and beat detection
impl AudioAnalyzer {
    pub fn analyze_audio(&mut self, audio_data: &[f32]) {
        // Real FFT analysis
        let fft_result = self.perform_fft(audio_data);
        
        // Real BPM detection
        self.bpm = self.detect_bpm(&fft_result);
        
        // Real volume analysis
        self.volume = self.calculate_volume(audio_data);
        
        // Real frequency bands
        self.frequency_bands = self.analyze_frequency_bands(&fft_result);
    }
}
```

### 10. ISF IMPORT/EXPORT - COMPLETE IMPLEMENTATION
```rust
// REQUIRED: Full ISF support with JSON parsing
impl IsfConverter {
    pub fn import_isf(&self, isf_json: &str) -> Result<String, String> {
        // Real ISF JSON parsing
        let isf_data: IsfShader = serde_json::from_str(isf_json)
            .map_err(|e| format!("ISF parse error: {}", e))?;
        
        // Real WGSL conversion
        let wgsl = self.convert_isf_to_wgsl(&isf_data)?;
        
        Ok(wgsl)
    }
    
    pub fn export_isf(&self, wgsl_code: &str) -> Result<String, String> {
        // Real WGSL to ISF conversion
        let isf_shader = self.parse_wgsl_to_isf(wgsl_code)?;
        
        // Real JSON generation
        let isf_json = serde_json::to_string_pretty(&isf_shader)
            .map_err(|e| format!("ISF export error: {}", e))?;
        
        Ok(isf_json)
    }
}
```

### 11. FFGL PLUGIN EXPORT - COMPLETE
```rust
// REQUIRED: Real FFGL plugin generation
impl FfglPlugin {
    pub fn export_ffgl(&self, shader_code: &str) -> Result<Vec<u8>, String> {
        // Real FFGL plugin compilation
        let plugin_code = self.generate_ffgl_code(shader_code)?;
        
        // Real binary generation
        let plugin_binary = self.compile_ffgl_plugin(&plugin_code)?;
        
        Ok(plugin_binary)
    }
}
```

### 12. COMPILATION VERIFICATION PROTOCOL
```bash
# REQUIRED: Build must succeed with all features
cargo check --all-features
cargo build --release

# REQUIRED: All modules accessible
cargo doc --no-deps

# REQUIRED: No warnings or errors
# ZERO tolerance for compilation failures
```

## VIOLATION CONSEQUENCES
- **ANY test file creation** = Immediate protocol violation
- **ANY feature disabling** = Immediate protocol violation  
- **ANY commenting out** = Immediate protocol violation
- **ANY placeholder code** = Immediate protocol violation

## SUCCESS CRITERIA
- ✅ All modules declared and active
- ✅ All UI panels permanently enabled
- ✅ All features fully implemented
- ✅ Zero compilation errors
- ✅ Working WGPU rendering
- ✅ Complete shader compilation
- ✅ Full node editor functionality
- ✅ Audio/MIDI integration working
- ✅ ISF import/export functional
- ✅ FFGL plugin export working

## IMPLEMENTATION ORDER
1. **Fix all module declarations** - Ensure all modules are declared
2. **Enable all UI panels** - Permanently enable everything
3. **Implement WGPU rendering** - Complete shader compilation
4. **Fix visual node editor** - Full GitHub integration
5. **Complete audio/MIDI** - Real analysis implementation
6. **Implement ISF support** - Full import/export
7. **Add FFGL export** - Complete plugin generation
8. **Final verification** - All features working

This protocol is **NON-NEGOTIABLE** and must be followed **EXACTLY**.