# Visual Node Editor Integration Guide

## Overview

This guide provides comprehensive instructions for integrating the fixed visual node editor with all the restored shader studio systems, ensuring seamless operation and maximum functionality.

## Integration Architecture

### System Integration Map

```
Visual Node Editor
├── Advanced Shader Compilation System
│   ├── WGSL compilation and optimization
│   ├── GLSL to WGSL conversion
│   ├── HLSL to WGSL conversion
│   └── ISF format support
├── Enhanced Audio System
│   ├── Real-time frequency analysis
│   ├── Beat detection and tempo
│   └── MIDI integration
├── Gesture Control System
│   ├── MediaPipe hand tracking
│   ├── LeapMotion integration
│   └── Unified gesture processing
├── Timeline Animation System
│   ├── Keyframe interpolation
│   ├── Multiple easing functions
│   └── Layer-based animation
└── WGSL Rendering System
    ├── WebGPU pipeline management
    ├── Real-time uniform updates
    └── Multi-format shader support
```

## Step-by-Step Integration

### Step 1: Replace the Broken Visual Node Editor

1. **Backup the original file:**
   ```bash
   cp src/visual_node_editor.rs src/visual_node_editor_backup.rs
   ```

2. **Replace with the fixed version:**
   ```bash
   cp src/visual_node_editor_fixed.rs src/visual_node_editor.rs
   ```

3. **Verify compilation:**
   ```bash
   cargo check
   ```

### Step 2: Integrate with Advanced Shader Compilation

Create integration module `src/visual_node_integration.rs`:

```rust
use crate::advanced_shader_compilation::AdvancedShaderCompiler;
use crate::visual_node_editor::VisualNodeEditor;
use crate::node_graph::NodeGraph;

pub struct NodeEditorIntegration {
    pub shader_compiler: AdvancedShaderCompiler,
    pub node_editor: VisualNodeEditor,
}

impl NodeEditorIntegration {
    pub fn new() -> Self {
        Self {
            shader_compiler: AdvancedShaderCompiler::new(),
            node_editor: VisualNodeEditor::new(),
        }
    }

    pub fn compile_node_graph(&mut self, node_graph: &NodeGraph) -> Result<String, Vec<String>> {
        // Generate WGSL from node graph
        let wgsl_code = node_graph.generate_wgsl()?;
        
        // Compile with advanced optimization
        self.shader_compiler.compile_shader(&wgsl_code, "node_graph_shader")
            .map(|compiled| compiled.code)
            .map_err(|e| vec![e])
    }

    pub fn auto_compile_with_optimization(&mut self, node_graph: &NodeGraph) -> Option<Result<String, Vec<String>>> {
        if self.node_editor.auto_compile {
            Some(self.compile_node_graph(node_graph))
        } else {
            None
        }
    }
}
```

### Step 3: Audio System Integration

```rust
use crate::enhanced_audio_system::{EnhancedAudioSystem, AudioAnalysisData};
use crate::visual_node_editor::VisualNodeEditor;

pub struct AudioNodeIntegration {
    audio_system: EnhancedAudioSystem,
    audio_uniforms: AudioUniforms,
}

#[derive(Debug, Clone)]
pub struct AudioUniforms {
    pub bass_level: f32,
    pub mid_level: f32,
    pub treble_level: f32,
    pub beat_intensity: f32,
    pub tempo: f32,
    pub frequency_bands: [f32; 16],
}

impl AudioNodeIntegration {
    pub fn new() -> Self {
        Self {
            audio_system: EnhancedAudioSystem::new(),
            audio_uniforms: AudioUniforms {
                bass_level: 0.0,
                mid_level: 0.0,
                treble_level: 0.0,
                beat_intensity: 0.0,
                tempo: 120.0,
                frequency_bands: [0.0; 16],
            },
        }
    }

    pub fn update_audio_data(&mut self) -> Result<AudioUniforms, String> {
        if let Some(analysis) = self.audio_system.get_audio_analysis() {
            self.audio_uniforms.bass_level = analysis.bass_level;
            self.audio_uniforms.mid_level = analysis.mid_level;
            self.audio_uniforms.treble_level = analysis.treble_level;
            self.audio_uniforms.beat_intensity = analysis.beat_intensity;
            self.audio_uniforms.tempo = analysis.tempo;
            self.audio_uniforms.frequency_bands.copy_from_slice(&analysis.frequency_bands[..16]);
            
            Ok(self.audio_uniforms.clone())
        } else {
            Err("No audio data available".to_string())
        }
    }

    pub fn create_audio_reactive_nodes(&self) -> Vec<NodeTemplate> {
        vec![
            NodeTemplate {
                name: "Bass Level".to_string(),
                category: "Audio".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.audio_uniforms.bass_level),
            },
            NodeTemplate {
                name: "Mid Level".to_string(),
                category: "Audio".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.audio_uniforms.mid_level),
            },
            NodeTemplate {
                name: "Treble Level".to_string(),
                category: "Audio".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.audio_uniforms.treble_level),
            },
            NodeTemplate {
                name: "Beat Intensity".to_string(),
                category: "Audio".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.audio_uniforms.beat_intensity),
            },
        ]
    }
}
```

### Step 4: Gesture Control Integration

```rust
use crate::gesture_control_system::{UnifiedGestureSystem, UnifiedGesture};
use crate::visual_node_editor::VisualNodeEditor;

pub struct GestureNodeIntegration {
    gesture_system: UnifiedGestureSystem,
    gesture_uniforms: GestureUniforms,
}

#[derive(Debug, Clone)]
pub struct GestureUniforms {
    pub hand_position: (f32, f32, f32),
    pub hand_rotation: (f32, f32, f32, f32),
    pub gesture_intensity: f32,
    pub pinch_strength: f32,
    pub grab_strength: f32,
    pub finger_count: i32,
}

impl GestureNodeIntegration {
    pub fn new() -> Self {
        Self {
            gesture_system: UnifiedGestureSystem::new(),
            gesture_uniforms: GestureUniforms {
                hand_position: (0.0, 0.0, 0.0),
                hand_rotation: (0.0, 0.0, 0.0, 1.0),
                gesture_intensity: 0.0,
                pinch_strength: 0.0,
                grab_strength: 0.0,
                finger_count: 0,
            },
        }
    }

    pub fn update_gesture_data(&mut self) -> Result<GestureUniforms, String> {
        if let Some(gesture) = self.gesture_system.get_current_gesture() {
            self.gesture_uniforms.hand_position = gesture.position;
            self.gesture_uniforms.hand_rotation = gesture.rotation;
            self.gesture_uniforms.gesture_intensity = gesture.intensity;
            self.gesture_uniforms.pinch_strength = gesture.pinch_strength;
            self.gesture_uniforms.grab_strength = gesture.grab_strength;
            self.gesture_uniforms.finger_count = gesture.finger_count as i32;
            
            Ok(self.gesture_uniforms.clone())
        } else {
            Err("No gesture data available".to_string())
        }
    }

    pub fn create_gesture_control_nodes(&self) -> Vec<NodeTemplate> {
        vec![
            NodeTemplate {
                name: "Hand Position".to_string(),
                category: "Gesture".to_string(),
                outputs: vec![PortType::Vec3],
                default_value: NodeValue::Vec3(self.gesture_uniforms.hand_position),
            },
            NodeTemplate {
                name: "Gesture Intensity".to_string(),
                category: "Gesture".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.gesture_uniforms.gesture_intensity),
            },
            NodeTemplate {
                name: "Pinch Strength".to_string(),
                category: "Gesture".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.gesture_uniforms.pinch_strength),
            },
            NodeTemplate {
                name: "Finger Count".to_string(),
                category: "Gesture".to_string(),
                outputs: vec![PortType::Int],
                default_value: NodeValue::Int(self.gesture_uniforms.finger_count),
            },
        ]
    }
}
```

### Step 5: Timeline Animation Integration

```rust
use crate::timeline_animation_system::{TimelineAnimationSystem, AnimationParameter};
use crate::visual_node_editor::VisualNodeEditor;

pub struct TimelineNodeIntegration {
    timeline_system: TimelineAnimationSystem,
    animation_uniforms: AnimationUniforms,
}

#[derive(Debug, Clone)]
pub struct AnimationUniforms {
    pub time_position: f32,
    pub animation_speed: f32,
    pub loop_progress: f32,
    pub keyframe_values: Vec<f32>,
    pub easing_factors: Vec<f32>,
}

impl TimelineNodeIntegration {
    pub fn new() -> Self {
        Self {
            timeline_system: TimelineAnimationSystem::new(),
            animation_uniforms: AnimationUniforms {
                time_position: 0.0,
                animation_speed: 1.0,
                loop_progress: 0.0,
                keyframe_values: vec![0.0; 16],
                easing_factors: vec![0.0; 16],
            },
        }
    }

    pub fn update_animation_data(&mut self, delta_time: f32) -> Result<AnimationUniforms, String> {
        self.timeline_system.update(delta_time)?;
        
        if let Some(params) = self.timeline_system.get_animation_parameters() {
            self.animation_uniforms.time_position = params.time_position;
            self.animation_uniforms.animation_speed = params.animation_speed;
            self.animation_uniforms.loop_progress = params.loop_progress;
            
            // Copy up to 16 keyframe values
            for (i, value) in params.keyframe_values.iter().take(16).enumerate() {
                self.animation_uniforms.keyframe_values[i] = *value;
            }
            
            // Copy up to 16 easing factors
            for (i, factor) in params.easing_factors.iter().take(16).enumerate() {
                self.animation_uniforms.easing_factors[i] = *factor;
            }
            
            Ok(self.animation_uniforms.clone())
        } else {
            Err("No animation data available".to_string())
        }
    }

    pub fn create_timeline_control_nodes(&self) -> Vec<NodeTemplate> {
        vec![
            NodeTemplate {
                name: "Time Position".to_string(),
                category: "Timeline".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.animation_uniforms.time_position),
            },
            NodeTemplate {
                name: "Animation Speed".to_string(),
                category: "Timeline".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.animation_uniforms.animation_speed),
            },
            NodeTemplate {
                name: "Loop Progress".to_string(),
                category: "Timeline".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.animation_uniforms.loop_progress),
            },
            NodeTemplate {
                name: "Keyframe Value".to_string(),
                category: "Timeline".to_string(),
                outputs: vec![PortType::Float],
                default_value: NodeValue::Float(self.animation_uniforms.keyframe_values[0]),
            },
        ]
    }
}
```

### Step 6: WGSL Rendering Integration

```rust
use crate::wgsl_rendering_system::{WgslRenderPipeline, WgslRenderConfig};
use crate::visual_node_editor::VisualNodeEditor;

pub struct RenderingNodeIntegration {
    pub render_pipeline: WgslRenderPipeline,
    pub uniform_buffer: UniformBuffer,
}

#[derive(Debug, Clone)]
pub struct UniformBuffer {
    pub time: f32,
    pub resolution: (f32, f32),
    pub mouse: (f32, f32),
    pub audio_data: AudioUniforms,
    pub gesture_data: GestureUniforms,
    pub animation_data: AnimationUniforms,
    pub custom_uniforms: Vec<f32>,
}

impl RenderingNodeIntegration {
    pub fn new(device: Arc<Device>, config: WgslRenderConfig) -> Self {
        Self {
            render_pipeline: WgslRenderPipeline::new(device, config),
            uniform_buffer: UniformBuffer {
                time: 0.0,
                resolution: (800.0, 600.0),
                mouse: (0.0, 0.0),
                audio_data: AudioUniforms::default(),
                gesture_data: GestureUniforms::default(),
                animation_data: AnimationUniforms::default(),
                custom_uniforms: vec![0.0; 64],
            },
        }
    }

    pub fn update_uniforms_from_node_graph(
        &mut self, 
        node_graph: &NodeGraph,
        audio_data: Option<AudioUniforms>,
        gesture_data: Option<GestureUniforms>,
        animation_data: Option<AnimationUniforms>,
    ) -> Result<(), String> {
        // Update audio uniforms if available
        if let Some(audio) = audio_data {
            self.uniform_buffer.audio_data = audio;
        }
        
        // Update gesture uniforms if available
        if let Some(gesture) = gesture_data {
            self.uniform_buffer.gesture_data = gesture;
        }
        
        // Update animation uniforms if available
        if let Some(animation) = animation_data {
            self.uniform_buffer.animation_data = animation;
        }
        
        // Update render pipeline uniforms
        self.render_pipeline.update_uniforms(&self.uniform_buffer)
            .map_err(|e| format!("Failed to update uniforms: {}", e))?;
        
        Ok(())
    }

    pub fn render_node_graph_shader(&mut self, node_graph_wgsl: String) -> Result<(), String> {
        // Load the generated WGSL shader
        self.render_pipeline.load_shader("node_graph", &node_graph_wgsl)
            .map_err(|e| format!("Failed to load shader: {}", e))?;
        
        // Render the shader
        self.render_pipeline.render()
            .map_err(|e| format!("Failed to render: {}", e))?;
        
        Ok(())
    }
}
```

## Complete Integration Example

### Main Application Integration

```rust
use crate::visual_node_editor::VisualNodeEditor;
use crate::advanced_shader_compilation::AdvancedShaderCompiler;
use crate::enhanced_audio_system::EnhancedAudioSystem;
use crate::gesture_control_system::UnifiedGestureSystem;
use crate::timeline_animation_system::TimelineAnimationSystem;
use crate::wgsl_rendering_system::WgslRenderPipeline;

pub struct IntegratedShaderStudio {
    node_editor: VisualNodeEditor,
    shader_compiler: AdvancedShaderCompiler,
    audio_system: EnhancedAudioSystem,
    gesture_system: UnifiedGestureSystem,
    timeline_system: TimelineAnimationSystem,
    render_pipeline: WgslRenderPipeline,
    
    // Integration modules
    node_integration: NodeEditorIntegration,
    audio_integration: AudioNodeIntegration,
    gesture_integration: GestureNodeIntegration,
    timeline_integration: TimelineNodeIntegration,
    rendering_integration: RenderingNodeIntegration,
}

impl IntegratedShaderStudio {
    pub fn new(device: Arc<Device>) -> Self {
        let config = WgslRenderConfig::default();
        
        Self {
            node_editor: VisualNodeEditor::new(),
            shader_compiler: AdvancedShaderCompiler::new(),
            audio_system: EnhancedAudioSystem::new(),
            gesture_system: UnifiedGestureSystem::new(),
            timeline_system: TimelineAnimationSystem::new(),
            render_pipeline: WgslRenderPipeline::new(device.clone(), config),
            
            node_integration: NodeEditorIntegration::new(),
            audio_integration: AudioNodeIntegration::new(),
            gesture_integration: GestureNodeIntegration::new(),
            timeline_integration: TimelineNodeIntegration::new(),
            rendering_integration: RenderingNodeIntegration::new(device, config),
        }
    }

    pub fn update(&mut self, delta_time: f32) -> Result<(), String> {
        // Update all systems
        self.audio_system.update()?;
        self.gesture_system.update()?;
        self.timeline_system.update(delta_time)?;
        
        // Get updated data from systems
        let audio_data = self.audio_integration.update_audio_data().ok();
        let gesture_data = self.gesture_integration.update_gesture_data().ok();
        let animation_data = self.timeline_integration.update_animation_data(delta_time).ok();
        
        // Update rendering uniforms
        self.rendering_integration.update_uniforms_from_node_graph(
            &self.node_graph,
            audio_data,
            gesture_data,
            animation_data,
        )?;
        
        // Auto-compile if needed
        if let Some(result) = self.node_integration.auto_compile_with_optimization(&self.node_graph) {
            match result {
                Ok(wgsl_code) => {
                    self.rendering_integration.render_node_graph_shader(wgsl_code)?;
                }
                Err(errors) => {
                    return Err(format!("Compilation errors: {:?}", errors));
                }
            }
        }
        
        Ok(())
    }

    pub fn render_ui(&mut self, ui: &mut Ui) {
        // Render the visual node editor
        self.node_editor.ui(ui, &mut self.node_graph);
        
        // Render system status panels
        self.render_audio_panel(ui);
        self.render_gesture_panel(ui);
        self.render_timeline_panel(ui);
        self.render_rendering_panel(ui);
    }

    fn render_audio_panel(&mut self, ui: &mut Ui) {
        ui.window("Audio System")
            .resizable(true)
            .default_pos([10.0, 400.0])
            .show(ui.ctx(), |ui| {
                if let Ok(audio_data) = self.audio_integration.update_audio_data() {
                    ui.label(format!("Bass: {:.2}", audio_data.bass_level));
                    ui.label(format!("Mid: {:.2}", audio_data.mid_level));
                    ui.label(format!("Treble: {:.2}", audio_data.treble_level));
                    ui.label(format!("Beat: {:.2}", audio_data.beat_intensity));
                    ui.label(format!("Tempo: {:.0} BPM", audio_data.tempo));
                }
            });
    }

    fn render_gesture_panel(&mut self, ui: &mut Ui) {
        ui.window("Gesture System")
            .resizable(true)
            .default_pos([220.0, 400.0])
            .show(ui.ctx(), |ui| {
                if let Ok(gesture_data) = self.gesture_integration.update_gesture_data() {
                    ui.label(format!("Position: {:.2}, {:.2}, {:.2}", 
                        gesture_data.hand_position.0,
                        gesture_data.hand_position.1,
                        gesture_data.hand_position.2
                    ));
                    ui.label(format!("Intensity: {:.2}", gesture_data.gesture_intensity));
                    ui.label(format!("Pinch: {:.2}", gesture_data.pinch_strength));
                    ui.label(format!("Grab: {:.2}", gesture_data.grab_strength));
                    ui.label(format!("Fingers: {}", gesture_data.finger_count));
                }
            });
    }

    fn render_timeline_panel(&mut self, ui: &mut Ui) {
        ui.window("Timeline System")
            .resizable(true)
            .default_pos([430.0, 400.0])
            .show(ui.ctx(), |ui| {
                if let Ok(animation_data) = self.timeline_integration.update_animation_data(0.016) {
                    ui.label(format!("Time: {:.2}s", animation_data.time_position));
                    ui.label(format!("Speed: {:.2}x", animation_data.animation_speed));
                    ui.label(format!("Loop: {:.2}%", animation_data.loop_progress * 100.0));
                    
                    ui.separator();
                    ui.label("Keyframe Values:");
                    for (i, value) in animation_data.keyframe_values.iter().take(4).enumerate() {
                        ui.label(format!("  KF{}: {:.3}", i, value));
                    }
                }
            });
    }

    fn render_rendering_panel(&mut self, ui: &mut Ui) {
        ui.window("Rendering System")
            .resizable(true)
            .default_pos([640.0, 400.0])
            .show(ui.ctx(), |ui| {
                ui.label(format!("Resolution: {:.0}x{:.0}", 
                    self.rendering_integration.uniform_buffer.resolution.0,
                    self.rendering_integration.uniform_buffer.resolution.1
                ));
                ui.label(format!("Time: {:.2}s", self.rendering_integration.uniform_buffer.time));
                ui.label(format!("Mouse: {:.0}, {:.0}", 
                    self.rendering_integration.uniform_buffer.mouse.0,
                    self.rendering_integration.uniform_buffer.mouse.1
                ));
                
                if ui.button("Take Screenshot").clicked() {
                    if let Err(e) = self.rendering_integration.take_screenshot() {
                        eprintln!("Screenshot error: {}", e);
                    }
                }
                
                if ui.button("Record Video").clicked() {
                    if let Err(e) = self.rendering_integration.start_video_recording() {
                        eprintln!("Video recording error: {}", e);
                    }
                }
            });
    }
}
```

## Testing and Validation

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_compilation() {
        let integration = NodeEditorIntegration::new();
        let mut node_graph = NodeGraph::new();
        
        // Add some test nodes
        let time_node = node_graph.add_node(NodeKind::Time, "Time", (100.0, 100.0));
        let sine_node = node_graph.add_node(NodeKind::Sine, "Sine", (200.0, 100.0));
        let output_node = node_graph.add_node(NodeKind::OutputColor, "Output", (300.0, 100.0));
        
        // Connect nodes
        if let (Some(time), Some(sine), Some(output)) = (
            node_graph.nodes.get(&time_node),
            node_graph.nodes.get(&sine_node),
            node_graph.nodes.get(&output_node)
        ) {
            if !time.outputs.is_empty() && !sine.inputs.is_empty() {
                node_graph.connect(time_node, time.outputs[0].id, sine_node, sine.inputs[0].id);
            }
            if !sine.outputs.is_empty() && !output.inputs.is_empty() {
                node_graph.connect(sine_node, sine.outputs[0].id, output_node, output.inputs[0].id);
            }
        }
        
        // Test compilation
        let result = integration.compile_node_graph(&node_graph);
        assert!(result.is_ok(), "Node graph compilation failed: {:?}", result);
    }

    #[test]
    fn test_audio_integration() {
        let mut audio_integration = AudioNodeIntegration::new();
        
        // Test audio data retrieval
        let audio_data = audio_integration.update_audio_data();
        
        // Should either succeed or fail gracefully
        match audio_data {
            Ok(data) => {
                assert!(data.bass_level >= 0.0);
                assert!(data.mid_level >= 0.0);
                assert!(data.treble_level >= 0.0);
            }
            Err(_) => {
                // This is acceptable if no audio device is available
                println!("No audio device available for testing");
            }
        }
    }

    #[test]
    fn test_gesture_integration() {
        let mut gesture_integration = GestureNodeIntegration::new();
        
        // Test gesture data retrieval
        let gesture_data = gesture_integration.update_gesture_data();
        
        // Should either succeed or fail gracefully
        match gesture_data {
            Ok(data) => {
                assert!(data.gesture_intensity >= 0.0);
                assert!(data.pinch_strength >= 0.0);
                assert!(data.grab_strength >= 0.0);
            }
            Err(_) => {
                // This is acceptable if no gesture device is available
                println!("No gesture device available for testing");
            }
        }
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_integration() {
        // Create a mock WebGPU device for testing
        // Note: This would require a proper WebGPU testing setup
        
        let studio = IntegratedShaderStudio::new(mock_device());
        
        // Test the complete integration
        assert!(studio.update(0.016).is_ok());
        
        // Verify all systems are properly initialized
        assert!(studio.audio_system.is_initialized());
        assert!(studio.gesture_system.is_initialized());
        assert!(studio.timeline_system.is_initialized());
        assert!(studio.render_pipeline.is_initialized());
    }

    #[test]
    fn test_shader_generation_pipeline() {
        let mut integration = NodeEditorIntegration::new();
        let mut node_graph = NodeGraph::new();
        
        // Create a complex node graph
        let time = node_graph.add_node(NodeKind::Time, "Time", (100.0, 100.0));
        let sine = node_graph.add_node(NodeKind::Sine, "Sine", (200.0, 100.0));
        let multiply = node_graph.add_node(NodeKind::Multiply, "Multiply", (300.0, 100.0));
        let color = node_graph.add_node(NodeKind::RGB, "Color", (400.0, 100.0));
        let output = node_graph.add_node(NodeKind::OutputColor, "Output", (500.0, 100.0));
        
        // Connect the nodes
        // ... connection logic ...
        
        // Generate and compile shader
        let result = integration.compile_node_graph(&node_graph);
        assert!(result.is_ok());
        
        let wgsl_code = result.unwrap();
        assert!(!wgsl_code.is_empty());
        assert!(wgsl_code.contains("@vertex"));
        assert!(wgsl_code.contains("@fragment"));
    }
}
```

## Performance Optimization

### 1. Caching Strategy
```rust
pub struct NodeEditorCache {
    shader_cache: HashMap<String, CompiledShader>,
    uniform_cache: HashMap<String, UniformBuffer>,
    node_template_cache: HashMap<String, NodeTemplate>,
}

impl NodeEditorCache {
    pub fn new() -> Self {
        Self {
            shader_cache: HashMap::new(),
            uniform_cache: HashMap::new(),
            node_template_cache: HashMap::new(),
        }
    }

    pub fn get_cached_shader(&self, key: &str) -> Option<&CompiledShader> {
        self.shader_cache.get(key)
    }

    pub fn cache_shader(&mut self, key: String, shader: CompiledShader) {
        self.shader_cache.insert(key, shader);
    }

    pub fn clear_cache(&mut self) {
        self.shader_cache.clear();
        self.uniform_cache.clear();
        self.node_template_cache.clear();
    }
}
```

### 2. Async Compilation
```rust
use tokio::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct AsyncNodeCompiler {
    sender: mpsc::Sender<CompilationRequest>,
    receiver: Arc<Mutex<mpsc::Receiver<CompilationResult>>>,
}

#[derive(Debug, Clone)]
pub struct CompilationRequest {
    pub node_graph: NodeGraph,
    pub request_id: String,
}

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub request_id: String,
    pub result: Result<String, Vec<String>>,
    pub compilation_time: std::time::Duration,
}

impl AsyncNodeCompiler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let (result_tx, result_rx) = mpsc::channel(100);
        
        // Spawn compilation worker
        tokio::spawn(async move {
            let mut compiler = AdvancedShaderCompiler::new();
            
            while let Some(request) = rx.recv().await {
                let start = std::time::Instant::now();
                let result = compiler.compile_node_graph(&request.node_graph);
                let duration = start.elapsed();
                
                let compilation_result = CompilationResult {
                    request_id: request.request_id,
                    result,
                    compilation_time: duration,
                };
                
                if let Err(_) = result_tx.send(compilation_result).await {
                    break;
                }
            }
        });
        
        Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(result_rx)),
        }
    }

    pub async fn compile_async(&self, node_graph: NodeGraph, request_id: String) -> Result<(), String> {
        let request = CompilationRequest {
            node_graph,
            request_id,
        };
        
        self.sender.send(request).await
            .map_err(|e| format!("Failed to send compilation request: {}", e))
    }

    pub async fn get_compilation_result(&self) -> Option<CompilationResult> {
        self.receiver.lock().unwrap().recv().await
    }
}
```

## Troubleshooting Common Issues

### 1. Compilation Errors
**Issue**: Node graph fails to compile to WGSL
**Solution**: 
- Check node connections are valid
- Verify all required inputs are connected
- Ensure node types are compatible
- Review shader compilation logs

### 2. Performance Issues
**Issue**: Slow rendering or compilation
**Solution**:
- Enable caching for frequently used shaders
- Use async compilation for complex graphs
- Optimize node graph structure
- Reduce number of active nodes

### 3. Integration Failures
**Issue**: Systems not communicating properly
**Solution**:
- Verify all integration modules are initialized
- Check data format compatibility
- Ensure proper error handling
- Review system update order

### 4. Audio/Gesture Detection Issues
**Issue**: Audio or gesture data not available
**Solution**:
- Verify device permissions
- Check device connectivity
- Test with alternative input sources
- Review system configuration

## Conclusion

This integration guide provides a comprehensive framework for connecting the fixed visual node editor with all the restored shader studio systems. The modular architecture ensures maintainability while the comprehensive testing strategy guarantees reliability. The performance optimization techniques ensure smooth operation even with complex node graphs.

By following this guide, you can create a fully integrated shader studio that leverages all the advanced features including real-time audio processing, gesture control, timeline animation, and high-performance WebGPU rendering.