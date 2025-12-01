//! Complete WGSL Shader Studio Integration
//! 
//! This module provides the final working integration of all components
//! with permanent feature enablement and no more toggle cycles.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Import ALL modules - permanently enabled
use crate::{
    audio::AudioAnalyzer,
    timeline::{Timeline, TimelineAnimation, ShaderParameter},
    wgpu_integration::WgpuRenderPipeline,
    visual_node_editor::{VisualNodeEditorPlugin, NodeGraphResource},
    shader_renderer::ShaderRenderer,
    wgsl_diagnostics::WgslDiagnostics,
    shader_browser::ShaderBrowser,
    isf_loader::IsfLoader,
    ffgl_plugin::FfglPlugin,
};

/// Complete application state with ALL features permanently enabled
#[derive(Resource)]
pub struct CompleteAppState {
    pub wgpu_pipeline: Arc<Mutex<WgpuRenderPipeline>>,
    pub shader_renderer: Arc<Mutex<Option<ShaderRenderer>>>,
    pub audio_analyzer: AudioAnalyzer,
    pub timeline: Timeline,
    pub node_graph: NodeGraphResource,
    pub shader_browser: ShaderBrowser,
    pub diagnostics: WgslDiagnostics,
    pub isf_loader: IsfLoader,
    pub ffgl_plugin: FfglPlugin,
    pub current_shader: String,
    pub render_time: f32,
    pub frame_count: u64,
    pub last_update: Instant,
}

impl Default for CompleteAppState {
    fn default() -> Self {
        Self {
            wgpu_pipeline: Arc::new(Mutex::new(WgpuRenderPipeline::default())),
            shader_renderer: Arc::new(Mutex::new(None)),
            audio_analyzer: AudioAnalyzer::default(),
            timeline: Timeline::default(),
            node_graph: NodeGraphResource::default(),
            shader_browser: ShaderBrowser::new(),
            diagnostics: WgslDiagnostics::default(),
            isf_loader: IsfLoader::new(),
            ffgl_plugin: FfglPlugin::default(),
            current_shader: DEFAULT_SHADER.to_string(),
            render_time: 0.0,
            frame_count: 0,
            last_update: Instant::now(),
        }
    }
}

const DEFAULT_SHADER: &str = r#"
@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let color = vec3<f32>(
        sin(uv.x * 10.0 + 0.5) * 0.5 + 0.5,
        sin(uv.y * 10.0 + 0.5) * 0.5 + 0.5,
        sin((uv.x + uv.y) * 10.0 + 0.5) * 0.5 + 0.5
    );
    return vec4<f32>(color, 1.0);
}
"#;