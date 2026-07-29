use crate::node_graph::NodeGraph;
use crate::screenshot_video_export::{ExportSettings, VideoExportSettings};
use crate::shader_renderer::ShaderRenderer;
use crate::timeline::TimelineAnimation;
use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ShaderParameter {
    pub name: String,
    pub wgsl_type: String,
    pub group: u32,
    pub binding: u32,
    pub value: f32,
    pub default_value: Option<f32>,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CentralView {
    Preview,
    NodeGraph,
    Scene3D,
    Timeline,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PipelineMode {
    Fragment,
    Compute,
}

impl Default for PipelineMode {
    fn default() -> Self {
        PipelineMode::Fragment
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

impl Default for ThemePreference {
    fn default() -> Self {
        ThemePreference::Dark
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RightSidebarMode {
    Parameters,
    Compute,
    Outputs,
    OSC,
    Audio,
    MIDI,
    Gestures,
    Performance,
    Scene3D,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceSet {
    All,
    Assets,
    ISF,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputsMode {
    Ndi,
    SpoutSyphon,
    ScreenshotsVideo,
    Ffgl,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CodeEditorTab {
    Editor,
    AI,
    Diagnostics,
    Analyzer,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreviewScaleMode {
    Fit,
    Fill,
    OneToOne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct DiagnosticMessage {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: DiagnosticSeverity,
}

/// Global shader renderer for preview functionality
#[derive(Resource)]
pub struct GlobalShaderRenderer {
    pub renderer: Mutex<Option<ShaderRenderer>>,
}

impl Default for GlobalShaderRenderer {
    fn default() -> Self {
        Self {
            renderer: Mutex::new(None),
        }
    }
}

#[derive(Resource)]
pub struct EditorUiState {
    pub show_shader_browser: bool,
    pub show_parameter_panel: bool,
    pub show_preview: bool,
    pub show_code_editor: bool,
    pub show_diagnostics_panel: bool,
    pub current_gesture_curve: crate::gesture_control::CurveType,
    pub gesture_control_active: bool,
    // Top-level feature panels (some still used for specific overlays or logic)
    pub show_gesture_calibration: bool,
    pub show_wgslsmith_panel: bool,
    pub show_performance_overlay: bool,
    pub show_color_grading_panel: bool,
    pub central_view: CentralView,
    pub fps: f32,
    pub time: f64,
    // Preview pipeline mode
    pub pipeline_mode: PipelineMode,
    // Right sidebar mode
    pub right_sidebar_mode: RightSidebarMode,
    pub outputs_mode: OutputsMode,
    pub code_editor_tab: CodeEditorTab,
    pub selected_source: SourceSet,
    // Theme settings
    pub dark_mode: bool,
    pub theme_preference: ThemePreference,
    // Browser/state
    pub search_query: String,
    pub show_all_shaders: bool,
    pub available_shaders_all: Vec<String>,
    pub available_shaders_compatible: Vec<String>,
    pub selected_shader: Option<String>,
    pub selected_category: Option<String>,
    // Code editor buffer
    pub draft_code: String,
    pub current_file: String,
    pub code: String,
    pub code_changed: bool,
    
    pub auto_apply: bool,
    // Node graph and project state
    pub node_graph: NodeGraph,
    // pub visual_node_editor: NodeEditorAdapter,
    pub last_project_path: Option<String>,
    pub timeline: TimelineAnimation,
    pub timeline_track_input: String,
    pub param_index_map: HashMap<String, usize>,
    pub param_index_input: usize,
    // Quick parameter controls for preview panel
    pub quick_params_enabled: bool,
    pub quick_param_a: f32,
    pub quick_param_b: f32,
    // Global shader renderer
    pub global_renderer: GlobalShaderRenderer,
    pub wgpu_initialized: bool,
    pub compilation_error: String,
    // Parameter values storage for shader rendering
    pub parameter_values: HashMap<String, f32>,
    pub parameter_values_hash: u64,

    // WGSLSmith AI fields
    pub wgsl_smith_prompt: String,
    pub wgsl_smith_generated: String,
    pub wgsl_smith_status: String,
    // WGSL diagnostics
    pub diagnostics_messages: Vec<DiagnosticMessage>,
    pub analyzer_status: Vec<String>,
    pub analyzer_run_requested: bool,
    // Compute pass UI state
    pub compute_pass_name: String,
    pub compute_workgroup_x: u32,
    pub compute_workgroup_y: u32,
    pub compute_workgroup_z: u32,
    pub pingpong_texture_name: String,
    pub pingpong_width: u32,
    pub pingpong_height: u32,
    pub dispatch_size_x: u32,
    pub dispatch_size_y: u32,
    pub dispatch_size_z: u32,
    // Video recording state
    pub is_recording_video: bool,
    pub video_fps: u32,
    pub video_duration: f32,
    pub video_format: String,
    pub video_quality: u8,
    // 3D Scene parameters (Space Editor inspired)
    pub camera_position: [f32; 3],
    pub camera_rotation: [f32; 3],
    pub camera_fov: f32,
    pub camera_near: f32,
    pub camera_far: f32,
    pub light_position: [f32; 3],
    pub light_color: [f32; 3],
    pub light_intensity: f32,
    pub ambient_light_color: [f32; 3],
    pub ambient_light_intensity: f32,
    pub export_settings: ExportSettings,
    pub video_export_settings: VideoExportSettings,
    pub use_legacy_windows: bool,
    pub ast_ok: bool,
    pub ast_error: String,
    pub validator_ok: bool,
    pub validator_error: String,
    pub transpiled_glsl: String,
    pub transpiler_error: String,
    pub scene3d_texture_id: Option<egui::TextureId>,
    pub scene3d_texture_handle: Option<bevy::prelude::Handle<bevy::prelude::Image>>,
    pub preview_scale_mode: PreviewScaleMode,
    pub preview_resolution: (u32, u32),
    // Performance optimization: cache the egui texture handle to avoid per-frame uploads
    pub preview_texture_handle: Option<egui::TextureHandle>,
    pub last_render_id: u64,
    pub cache_key: (u64, u32, u32, u64),
    // Background task tracking
    pub is_scanning_shaders: bool,
    pub shader_scan_status: String,
    // Persistent UI mapping state
    pub current_midi_channel: u8,
    pub current_midi_number: u8,
    pub current_gesture_type: crate::gesture_control::GestureType,
    pub current_gesture_min: f32,
    pub current_gesture_max: f32,
    pub current_gesture_invert: bool,
    // ... other state fields
    // Status Bar
    pub status_message: String,
    pub status_message_timer: f32,
    pub show_status_bar: bool,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            show_shader_browser: true,
            show_parameter_panel: true,
            show_preview: true,
            show_code_editor: true,
            show_diagnostics_panel: false,
            current_gesture_curve: crate::gesture_control::CurveType::Linear,
            gesture_control_active: true,
            // Top-level feature panels (some still used for specific overlays or logic)
            show_gesture_calibration: true,
            show_wgslsmith_panel: true,
            show_performance_overlay: false,
            show_color_grading_panel: false,
            central_view: CentralView::Preview,
            fps: 0.0,
            time: 0.0,
            pipeline_mode: PipelineMode::default(),
            right_sidebar_mode: RightSidebarMode::Parameters,
            outputs_mode: OutputsMode::ScreenshotsVideo,
            code_editor_tab: CodeEditorTab::Editor,
            selected_source: SourceSet::All,
            dark_mode: true,
            theme_preference: ThemePreference::default(),
            search_query: String::new(),
            show_all_shaders: true,
            available_shaders_all: Vec::new(),
            available_shaders_compatible: Vec::new(),
            selected_shader: None,
            selected_category: None,
            draft_code: String::from("@vertex\nfn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {\n    var pos = vec2<f32>(0.0, 0.0);\n    switch vertex_index {\n        case 0u: { pos = vec2<f32>(-1.0, -1.0); }\n        case 1u: { pos = vec2<f32>( 3.0, -1.0); }\n        case 2u: { pos = vec2<f32>(-1.0,  3.0); }\n        default: { pos = vec2<f32>(0.0, 0.0); }\n    }\n    return vec4<f32>(pos, 0.0, 1.0);\n}\n\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n    return vec4<f32>(0.2, 0.2, 0.2, 1.0);\n}"),
            current_file: String::new(),
            code: String::from("@vertex\nfn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {\n    var pos = vec2<f32>(0.0, 0.0);\n    switch vertex_index {\n        case 0u: { pos = vec2<f32>(-1.0, -1.0); }\n        case 1u: { pos = vec2<f32>( 3.0, -1.0); }\n        case 2u: { pos = vec2<f32>(-1.0,  3.0); }\n        default: { pos = vec2<f32>(0.0, 0.0); }\n    }\n    return vec4<f32>(pos, 0.0, 1.0);\n}\n\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n    return vec4<f32>(0.2, 0.2, 0.2, 1.0);\n}"),
            code_changed: false,
            auto_apply: false,
            node_graph: NodeGraph::default(),
            // visual_node_editor: NodeEditorAdapter::new(),
            last_project_path: None,
            timeline: TimelineAnimation::default(),
            timeline_track_input: String::new(),
            param_index_map: HashMap::new(),
            param_index_input: 0,
            quick_params_enabled: false,
            quick_param_a: 0.5,
            quick_param_b: 0.5,
            global_renderer: GlobalShaderRenderer::default(),
            wgpu_initialized: false,
            compilation_error: String::new(),
            parameter_values: HashMap::new(),
             parameter_values_hash: 0,
            wgsl_smith_prompt: String::new(),
            wgsl_smith_generated: String::new(),
            wgsl_smith_status: String::new(),
            diagnostics_messages: Vec::new(),
            analyzer_status: Vec::new(),
            analyzer_run_requested: false,
            // Compute pass UI defaults
            compute_pass_name: "compute_pass_1".to_string(),
            compute_workgroup_x: 8,
            compute_workgroup_y: 8,
            compute_workgroup_z: 1,
            pingpong_texture_name: "pingpong_tex".to_string(),
            pingpong_width: 512,
            pingpong_height: 512,
            dispatch_size_x: 8,
            dispatch_size_y: 8,
            dispatch_size_z: 1,
            // Video recording defaults
            is_recording_video: false,
            video_fps: 30,
            video_duration: 10.0,
            video_format: "mp4".to_string(),
            video_quality: 90,
            // 3D Scene parameters defaults
            camera_position: [0.0, 0.0, 5.0],
            camera_rotation: [0.0, 0.0, 0.0],
            camera_fov: 60.0,
            camera_near: 0.1,
            camera_far: 100.0,
            light_position: [2.0, 2.0, 2.0],
            light_color: [1.0, 1.0, 1.0],
            light_intensity: 1.0,
            ambient_light_color: [0.2, 0.2, 0.2],
            ambient_light_intensity: 0.3,
            export_settings: ExportSettings::default(),
            video_export_settings: VideoExportSettings::default(),
            use_legacy_windows: false,
            ast_ok: false,
            ast_error: String::new(),
            validator_ok: false,
            validator_error: String::new(),
            transpiled_glsl: String::new(),
            transpiler_error: String::new(),
            scene3d_texture_id: None,
            scene3d_texture_handle: None,
            preview_scale_mode: PreviewScaleMode::Fit,
            preview_resolution: (1280, 720),
            preview_texture_handle: None,
            last_render_id: 0,
            cache_key: (0, 0, 0, 0),
            is_scanning_shaders: false,
            shader_scan_status: String::new(),
            current_midi_channel: 1,
            current_midi_number: 0,
            current_gesture_type: crate::gesture_control::GestureType::Pinch,
            current_gesture_min: 0.0,
            current_gesture_max: 1.0,
            current_gesture_invert: false,
            status_message: "Ready".to_string(),
            status_message_timer: 0.0,
            show_status_bar: true,
        }
    }
}

impl EditorUiState {
    /// Set a parameter value for shader rendering
    pub fn set_parameter_value(&mut self, name: &str, value: f32) {
        self.parameter_values.insert(name.to_string(), value);
    }

    /// Get a parameter value
    pub fn get_parameter_value(&self, name: &str) -> Option<f32> {
        self.parameter_values.get(name).copied()
    }

    /// Get all parameter values as a reference
    pub fn get_parameter_values(&self) -> &HashMap<String, f32> {
        &self.parameter_values
    }

    /// Show a message in the status bar for a specified duration in seconds
    pub fn show_status(&mut self, message: impl Into<String>, duration: f32) {
        self.status_message = message.into();
        self.status_message_timer = duration;
    }
}

#[derive(Resource, Default)]
pub struct UiStartupGate {
    pub frames: u32,
}
