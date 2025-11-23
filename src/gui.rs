#![cfg(feature = "eframe")] // DISABLED - EFAME ONLY - DO NOT USE
//! WGSL Shader Studio - Professional GUI Application (LEGACY EFAME - DO NOT USE)
//! Based on modular-fractal-shader UI architecture

#[cfg(feature = "gui")]
use eframe::egui;
#[cfg(feature = "gui")]
use std::collections::HashMap;
#[cfg(feature = "gui")]

#[cfg(feature = "gui")]
use std::sync::Arc;
#[cfg(feature = "gui")]
use std::path::PathBuf;
#[cfg(feature = "gui")]
use std::time::Instant;

#[cfg(feature = "gui")]
use rfd;
#[cfg(feature = "gui")]
use pollster;

#[cfg(feature = "gui")]
use crate::isf_loader::*;
#[cfg(feature = "gui")]
use crate::isf_auto_converter::IsfAutoConverter;
#[cfg(feature = "gui")]
use crate::audio::{AudioMidiSystem, AudioData};
use crate::audio::AudioData as ShaderAudioData;
#[cfg(feature = "gui")]
use crate::gesture_control::{GestureControlSystem, GestureType};
#[cfg(feature = "gui")]
use crate::shader_renderer::{ShaderRenderer, RenderParameters};
#[cfg(feature = "gui")]


#[cfg(feature = "gui")]


#[cfg(feature = "gui")]
pub struct ShaderGui {
    // Shader management
    shaders: Vec<IsfShader>,
    current_shader: Option<usize>,
    parameter_values: HashMap<String, f32>,
    shader_parameters: HashMap<String, f32>,
    current_wgsl_code: String,
    wgsl_code: String,
    shader_templates: Vec<ShaderTemplate>,
    expanded_template_library: Vec<ShaderTemplate>,
    available_shaders: Vec<String>,

    // UI state
    show_audio_panel: bool,
    show_midi_panel: bool,
    show_code_editor: bool,
    show_preview: bool,
    show_converter: bool,
    show_file_browser: bool,
    show_node_editor: bool,
    selected_template: Option<usize>,
    selected_template_category: String,
    selected_shader: Option<usize>,
    search_query: String,

    // Node-based editor state
    nodes: Vec<Node>,
    connections: Vec<NodeConnection>,
    selected_node: Option<NodeId>,
    dragged_node: Option<NodeId>,
    drag_offset: egui::Vec2,
    pending_connection: Option<PendingConnection>,
    grid_size: f32,
    pan_offset: egui::Vec2,
    zoom: f32,

    // File management
    current_file: Option<PathBuf>,
    recent_files: Vec<PathBuf>,

    // Audio/MIDI system
    audio_system: Option<Arc<std::sync::Mutex<AudioMidiSystem>>>,

    // Gesture control system
    gesture_control: Option<Arc<std::sync::Mutex<GestureControlSystem>>>,
    show_gesture_panel: bool,
    selected_gesture: Option<GestureType>,

    // Shader converter state
    from_format: String,
    to_format: String,

    // Live Preview System
    renderer: Option<std::sync::Arc<std::sync::Mutex<ShaderRenderer>>>,
    preview_texture: Option<egui::TextureHandle>,
    preview_size: (u32, u32),
    last_render_time: Instant,
    render_fps: f32,

    // WGSL Bindgen integration
    // bindgen_analyzer: Option<WgslBindgenAnalyzer>,
    
    // WGSL Diagnostics
    // diagnostics_analyzer: Option<WgslDiagnostics>,
    
    // ISF Auto-converter
    isf_auto_converter: Option<IsfAutoConverter>,

    // Performance
    fps_counter: f32,
    frame_count: u32,
    last_fps_update: Instant,

    // Error handling
    shader_errors: Vec<String>,
    compilation_status: ShaderCompilationStatus,

    // Theme and appearance
    current_theme: String,
    custom_theme_colors: HashMap<String, egui::Color32>,
    brightness: f32,
    contrast: f32,
    font_size: f32,
    auto_render: bool,
    time_slider: f32,

    // Info screens/dialogs
    show_about_dialog: bool,
    show_documentation_dialog: bool,
    show_shortcuts_dialog: bool,
    show_theme_editor: bool,

    // Initialization state
    initialized: bool,
    initialization_started: bool,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub struct ShaderTemplate {
    pub name: String,
    pub description: String,
    pub wgsl_code: String,
    pub category: String,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderCompilationStatus {
    NotCompiled,
    Compiling,
    Success,
    Error(String),
}

// Placeholder types for missing modules
#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub position: egui::Pos2,
    pub size: egui::Vec2,
    pub node_type: NodeType,
    pub title: String,
    pub inputs: Vec<NodePin>,
    pub outputs: Vec<NodePin>,
    pub value: f32,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub struct NodePin {
    pub id: PinId,
    pub name: String,
    pub pin_type: PinType,
    pub position: egui::Pos2,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PinId(pub usize);

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub enum NodeType {
    // Basic I/O
    Input,
    Output,
    Uniform,
    TextureInput,

    // Math Operations
    Math,
    Trigonometry,
    VectorMath,
    MatrixMath,

    // Color Operations
    Color,
    ColorAdjustment,
    ColorMix,
    ColorSpace,

    // Texture Operations
    Texture,
    TextureSample,
    TextureTransform,
    TextureBlend,

    // Geometry & 3D
    Transform,
    Geometry,
    Volumetric,
    PointCloud,

    // Advanced Rendering
    Lighting,
    Material,
    BRDF,
    RayMarching,

    // Neural & AI
    NeRF,
    MLInference,

    // Audio & Time
    AudioReactive,
    Time,
    Oscillator,

    // Post Processing
    Filter,
    Blur,
    Distortion,
    Effects,

    // Utility
    Constant,
    Variable,
    Switch,
    Loop,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub enum PinType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    Texture,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub from_node: NodeId,
    pub from_pin: PinId,
    pub to_node: NodeId,
    pub to_pin: PinId,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone)]
pub struct PendingConnection {
    pub from_node: NodeId,
    pub from_pin: PinId,
    pub from_pos: egui::Pos2,
}

#[cfg(feature = "gui")]
impl Default for ShaderGui {
    fn default() -> Self {
        Self {
            shaders: Vec::new(),
            current_shader: None,
            parameter_values: HashMap::new(),
            shader_parameters: HashMap::new(),
            current_wgsl_code: Self::default_wgsl_shader(),
            wgsl_code: String::new(),
            shader_templates: Self::create_shader_templates(),
            show_audio_panel: true,
            show_midi_panel: false,
            show_code_editor: true,
            show_preview: true,
            show_converter: false,
            show_file_browser: false,
            show_node_editor: false,
            selected_template: None,
            selected_template_category: "All".to_string(),
            selected_shader: None,
            search_query: String::new(),
            current_file: None,
            recent_files: Vec::new(),
            audio_system: None,
            gesture_control: None,
            show_gesture_panel: false,
            selected_gesture: None,
            renderer: None,
            preview_texture: None,
            preview_size: (512, 512),
            last_render_time: Instant::now(),
            render_fps: 0.0,
            fps_counter: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            shader_errors: Vec::new(),
            compilation_status: ShaderCompilationStatus::NotCompiled,
            nodes: Vec::new(),
            connections: Vec::new(),
            selected_node: None,
            dragged_node: None,
            drag_offset: egui::Vec2::ZERO,
            pending_connection: None,
            from_format: "WGSL".to_string(),
            to_format: "GLSL".to_string(),
            expanded_template_library: Self::create_expanded_template_library(),
            available_shaders: Vec::new(),
            grid_size: 20.0,
            pan_offset: egui::Vec2::ZERO,
            zoom: 1.0,
            current_theme: "professional_dark".to_string(),
            custom_theme_colors: HashMap::new(),
            brightness: 0.0,
            contrast: 1.0,
            font_size: 14.0,
            show_about_dialog: false,
            show_documentation_dialog: false,
            show_shortcuts_dialog: false,
            show_theme_editor: false,
            auto_render: false,
            time_slider: 0.0,
            initialized: false,
            initialization_started: false,
            // bindgen_analyzer: Some(WgslBindgenAnalyzer::new()),
            // diagnostics_analyzer: Some(WgslDiagnostics::new()),
            isf_auto_converter: Some(IsfAutoConverter::new()),
        }
    }
}

#[cfg(feature = "gui")]
impl ShaderGui {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Box<Self> {
        let mut gui = Self::default();
        gui.wgsl_code = gui.current_wgsl_code.clone();
        gui.selected_shader = gui.current_shader;
        gui.search_query = String::new();
        gui.shader_parameters = gui.parameter_values.clone();
        gui.available_shaders = gui.shaders.iter().map(|s| s.name.clone()).collect();
        
        // Apply professional theme
        gui.apply_professional_theme(_cc);
        
        Box::new(gui)
    }

    fn apply_professional_theme(&mut self, cc: &eframe::CreationContext<'_>) {
        // Modern dark theme inspired by Blender, Nuke, and Shadered
        let mut visuals = egui::Visuals::dark();

        // Deep charcoal/navy blue color scheme for professional look
        visuals.override_text_color = Some(egui::Color32::from_rgb(230, 230, 230));
        visuals.panel_fill = egui::Color32::from_rgb(35, 35, 38); // Deep charcoal panels
        visuals.window_fill = egui::Color32::from_rgb(28, 28, 31); // Navy blue windows
        visuals.faint_bg_color = egui::Color32::from_rgb(42, 42, 45);

        // High-contrast accent colors for better visibility
        visuals.hyperlink_color = egui::Color32::from_rgb(120, 180, 255); // Bright blue links
        visuals.warn_fg_color = egui::Color32::from_rgb(255, 200, 50); // Golden yellow warnings
        visuals.error_fg_color = egui::Color32::from_rgb(255, 80, 80); // Bright red errors

        // Professional selection with high contrast
        visuals.selection.bg_fill = egui::Color32::from_rgb(80, 140, 220);
        visuals.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(120, 180, 255));

        // Modern widget styling with subtle gradients
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 45, 48);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 55, 58);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(65, 65, 68);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(75, 75, 78);
        visuals.widgets.open.bg_fill = egui::Color32::from_rgb(50, 50, 53);

        // Enhanced button styling with modern contrast
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.2, egui::Color32::from_rgb(190, 190, 200));
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.2, egui::Color32::from_rgb(210, 210, 220));
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.2, egui::Color32::from_rgb(230, 230, 240));

        // Professional window styling with enhanced shadows
        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 2,
            color: egui::Color32::from_black_alpha(150),
        };

        // Modern scrollbar styling (removed deprecated fields)
        // Enhanced spacing and interaction feedback
        visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

        cc.egui_ctx.set_visuals(visuals);
    }

    fn analyze_shader_uniforms(&mut self, shader_name: &str) {
        // TODO: Re-enable when WgslBindgenAnalyzer is properly integrated
        // if let Some(ref mut analyzer) = self.bindgen_analyzer {
        //     match analyzer.analyze_shader(&self.current_wgsl_code, shader_name) {
        //         Ok(layouts) => {
        //             println!("Analyzed {} uniform layouts for shader {}", layouts.len(), shader_name);
        //             for layout in &layouts {
        //                 println!("  - Uniform '{}' at binding {}:{}", layout.name, layout.group, layout.binding);
        //             }
        //         }
        //         Err(e) => {
        //             eprintln!("Failed to analyze shader uniforms: {}", e);
        //         }
        //     }
        // }
    }

    fn apply_theme(&mut self, theme_name: &str, ctx: &egui::Context) {
        self.current_theme = theme_name.to_string();

        let mut visuals = match theme_name {
            "professional_dark" => {
                let mut v = egui::Visuals::dark();
                // Deep charcoal/navy blue color scheme for professional look
                v.override_text_color = Some(egui::Color32::from_rgb(230, 230, 230));
                v.panel_fill = egui::Color32::from_rgb(35, 35, 38); // Deep charcoal panels
                v.window_fill = egui::Color32::from_rgb(28, 28, 31); // Navy blue windows
                v.faint_bg_color = egui::Color32::from_rgb(42, 42, 45);
                v.hyperlink_color = egui::Color32::from_rgb(120, 180, 255); // Bright blue links
                v.warn_fg_color = egui::Color32::from_rgb(255, 200, 50); // Golden yellow warnings
                v.error_fg_color = egui::Color32::from_rgb(255, 80, 80); // Bright red errors
                v.selection.bg_fill = egui::Color32::from_rgb(80, 140, 220);
                v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(120, 180, 255));
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 45, 48);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 55, 58);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(65, 65, 68);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(75, 75, 78);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(50, 50, 53);
                v.widgets.inactive.fg_stroke = egui::Stroke::new(1.2, egui::Color32::from_rgb(190, 190, 200));
                v.widgets.hovered.fg_stroke = egui::Stroke::new(1.2, egui::Color32::from_rgb(210, 210, 220));
                v.widgets.active.fg_stroke = egui::Stroke::new(1.2, egui::Color32::from_rgb(230, 230, 240));
                v.window_shadow = egui::epaint::Shadow {
                    offset: [0, 12],
                    blur: 32,
                    spread: 2,
                    color: egui::Color32::from_black_alpha(150),
                };
                v
            }
            "professional_light" => {
                let mut v = egui::Visuals::light();
                v.override_text_color = Some(egui::Color32::from_rgb(30, 30, 30));
                v.panel_fill = egui::Color32::from_rgb(250, 250, 252);
                v.window_fill = egui::Color32::from_rgb(255, 255, 255);
                v.faint_bg_color = egui::Color32::from_rgb(245, 245, 247);
                v.hyperlink_color = egui::Color32::from_rgb(0, 100, 200);
                v.warn_fg_color = egui::Color32::from_rgb(200, 150, 0);
                v.error_fg_color = egui::Color32::from_rgb(200, 50, 50);
                v.selection.bg_fill = egui::Color32::from_rgb(200, 220, 255);
                v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255));
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(240, 240, 242);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(230, 230, 232);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(220, 220, 222);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(210, 210, 212);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(235, 235, 237);
                v
            }
            "midnight_blue" => {
                let mut v = egui::Visuals::dark();
                v.override_text_color = Some(egui::Color32::from_rgb(220, 220, 240));
                v.panel_fill = egui::Color32::from_rgb(15, 15, 35);
                v.window_fill = egui::Color32::from_rgb(10, 10, 25);
                v.faint_bg_color = egui::Color32::from_rgb(20, 20, 45);
                v.hyperlink_color = egui::Color32::from_rgb(100, 150, 255);
                v.warn_fg_color = egui::Color32::from_rgb(255, 200, 100);
                v.error_fg_color = egui::Color32::from_rgb(255, 100, 100);
                v.selection.bg_fill = egui::Color32::from_rgb(50, 100, 200);
                v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255));
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 25, 50);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 35, 60);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 45, 70);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(55, 55, 80);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(30, 30, 55);
                v
            }
            "sunrise_orange" => {
                let mut v = egui::Visuals::dark();
                v.override_text_color = Some(egui::Color32::from_rgb(250, 240, 220));
                v.panel_fill = egui::Color32::from_rgb(45, 25, 15);
                v.window_fill = egui::Color32::from_rgb(35, 20, 10);
                v.faint_bg_color = egui::Color32::from_rgb(55, 35, 25);
                v.hyperlink_color = egui::Color32::from_rgb(255, 150, 50);
                v.warn_fg_color = egui::Color32::from_rgb(255, 220, 100);
                v.error_fg_color = egui::Color32::from_rgb(255, 100, 100);
                v.selection.bg_fill = egui::Color32::from_rgb(200, 120, 50);
                v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 50));
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(60, 35, 20);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(70, 45, 30);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(80, 55, 40);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(90, 65, 50);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(65, 40, 25);
                v
            }
            "forest_green" => {
                let mut v = egui::Visuals::dark();
                v.override_text_color = Some(egui::Color32::from_rgb(220, 240, 220));
                v.panel_fill = egui::Color32::from_rgb(15, 35, 15);
                v.window_fill = egui::Color32::from_rgb(10, 25, 10);
                v.faint_bg_color = egui::Color32::from_rgb(20, 45, 20);
                v.hyperlink_color = egui::Color32::from_rgb(100, 200, 100);
                v.warn_fg_color = egui::Color32::from_rgb(255, 220, 100);
                v.error_fg_color = egui::Color32::from_rgb(255, 100, 100);
                v.selection.bg_fill = egui::Color32::from_rgb(50, 150, 50);
                v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100));
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 50, 25);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 60, 35);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 70, 45);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(55, 80, 55);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(30, 55, 30);
                v
            }
            "purple_haze" => {
                let mut v = egui::Visuals::dark();
                v.override_text_color = Some(egui::Color32::from_rgb(240, 220, 250));
                v.panel_fill = egui::Color32::from_rgb(35, 15, 45);
                v.window_fill = egui::Color32::from_rgb(25, 10, 35);
                v.faint_bg_color = egui::Color32::from_rgb(45, 25, 55);
                v.hyperlink_color = egui::Color32::from_rgb(200, 100, 255);
                v.warn_fg_color = egui::Color32::from_rgb(255, 220, 100);
                v.error_fg_color = egui::Color32::from_rgb(255, 100, 100);
                v.selection.bg_fill = egui::Color32::from_rgb(150, 50, 200);
                v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 100, 255));
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(50, 25, 60);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 35, 70);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 45, 80);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(80, 55, 90);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(55, 30, 65);
                v
            }
            "custom" => {
                // Use custom colors from the hashmap
                let mut v = egui::Visuals::dark();
                if let Some(bg_color) = self.custom_theme_colors.get("background") {
                    v.panel_fill = *bg_color;
                }
                if let Some(window_color) = self.custom_theme_colors.get("window") {
                    v.window_fill = *window_color;
                }
                if let Some(text_color) = self.custom_theme_colors.get("text") {
                    v.override_text_color = Some(*text_color);
                }
                v
            }
            _ => egui::Visuals::dark(),
        };

        // Apply brightness/contrast adjustments
        let brightness_factor = self.brightness;
        let contrast_factor = self.contrast;

        fn adjust_color(color: egui::Color32, brightness: f32, contrast: f32) -> egui::Color32 {
            let [r, g, b, a] = color.to_srgba_unmultiplied();
            let rf = r as f32 / 255.0;
            let gf = g as f32 / 255.0;
            let bf = b as f32 / 255.0;

            // Apply contrast first, then brightness
            let cr = ((rf - 0.5) * contrast + 0.5 + brightness).clamp(0.0, 1.0);
            let cg = ((gf - 0.5) * contrast + 0.5 + brightness).clamp(0.0, 1.0);
            let cb = ((bf - 0.5) * contrast + 0.5 + brightness).clamp(0.0, 1.0);

            egui::Color32::from_rgba_premultiplied(
                (cr * 255.0) as u8,
                (cg * 255.0) as u8,
                (cb * 255.0) as u8,
                a,
            )
        }

        // Adjust all color fields
        visuals.panel_fill = adjust_color(visuals.panel_fill, brightness_factor, contrast_factor);
        visuals.window_fill = adjust_color(visuals.window_fill, brightness_factor, contrast_factor);
        visuals.faint_bg_color = adjust_color(visuals.faint_bg_color, brightness_factor, contrast_factor);
        visuals.widgets.noninteractive.bg_fill = adjust_color(visuals.widgets.noninteractive.bg_fill, brightness_factor, contrast_factor);
        visuals.widgets.inactive.bg_fill = adjust_color(visuals.widgets.inactive.bg_fill, brightness_factor, contrast_factor);
        visuals.widgets.hovered.bg_fill = adjust_color(visuals.widgets.hovered.bg_fill, brightness_factor, contrast_factor);
        visuals.widgets.active.bg_fill = adjust_color(visuals.widgets.active.bg_fill, brightness_factor, contrast_factor);
        visuals.widgets.open.bg_fill = adjust_color(visuals.widgets.open.bg_fill, brightness_factor, contrast_factor);

        ctx.set_visuals(visuals);
        
        // Save theme settings whenever theme is applied
        self.save_theme_settings();
    }
    fn default_wgsl_shader() -> String {
        r#"// Minimal Working WGSL Shader
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    audio_volume: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_treble: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var pos = vec2<f32>(0.0, 0.0);
    switch vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>(3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0, 3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    // Convert from clip space (-1 to 1) to UV coordinates (0 to 1)
    let uv = (position.xy + vec2<f32>(1.0, 1.0)) * 0.5;
    let col = 0.5 + 0.5 * cos(uniforms.time + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));
    return vec4<f32>(col, 1.0);
}"#.to_string()
    }

    fn create_shader_templates() -> Vec<ShaderTemplate> {
        vec![
            ShaderTemplate {
                name: "Basic Color".to_string(),
                description: "Simple colored background".to_string(),
                category: "Basic".to_string(),
                wgsl_code: r#"// Basic Color Shader
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.5, 0.7, 1.0, 1.0);
}"#.to_string(),
            },
            ShaderTemplate {
                name: "Time-based Animation".to_string(),
                description: "Animated shader using time uniform".to_string(),
                category: "Animation".to_string(),
                wgsl_code: r#"// Time-based Animation
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    let col = 0.5 + 0.5 * cos(time + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));
    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },
            ShaderTemplate {
                name: "Fractal Pattern".to_string(),
                description: "Simple fractal pattern".to_string(),
                category: "Fractal".to_string(),
                wgsl_code: r#"// Fractal Pattern
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

fn fractal(uv: vec2<f32>) -> f32 {
    var p = uv;
    var d = 0.0;
    for (var i = 0; i < 8; i = i + 1) {
        p = abs(p) / dot(p, p) - 0.5;
        d = max(d, dot(p, p));
    }
    return d;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy - 0.5 * resolution) / min(resolution.x, resolution.y);
    let d = fractal(uv + time * 0.1);
    let col = vec3<f32>(1.0 - d);
    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },
        ]
    }

    fn create_expanded_template_library() -> Vec<ShaderTemplate> {
        vec![
            // Working Animation Examples
            ShaderTemplate {
                name: "Animated Gradient".to_string(),
                description: "Beautiful animated color gradient using time".to_string(),
                category: "Basic".to_string(),
                wgsl_code: r#"
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    let t = time;
    
    let r = 0.5 + 0.5 * sin(t + uv.x * 6.28318);
    let g = 0.5 + 0.5 * sin(t * 0.8 + uv.x * 6.28318);
    let b = 0.5 + 0.5 * sin(t * 1.2 + uv.x * 6.28318);
    
    return vec4<f32>(r, g, b, 1.0);
}"#.to_string(),
            },

            ShaderTemplate {
                name: "Kaleidoscope".to_string(),
                description: "Beautiful kaleidoscope pattern".to_string(),
                category: "Effects".to_string(),
                wgsl_code: r#"
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy / resolution - 0.5) * 2.0;
    let t = time;
    
    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);
    
    let kaleido = angle + t * 0.5;
    let pattern = sin(kaleido * 6.0 + radius * 10.0 + t);
    
    let col = vec3<f32>(
        0.5 + 0.5 * sin(pattern),
        0.5 + 0.5 * sin(pattern * 1.3 + 2.0944),
        0.5 + 0.5 * sin(pattern * 1.7 + 4.18879)
    );
    
    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },

            ShaderTemplate {
                name: "Fire Effect".to_string(),
                description: "Realistic fire/flame effect".to_string(),
                category: "Effects".to_string(),
                wgsl_code: r#"
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

fn noise(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn fractal_noise(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var freq = 1.0;
    loop {
        if (freq > 32.0) {
            break;
        }
        value += amplitude * noise(p * freq);
        amplitude *= 0.5;
        freq *= 2.0;
    }
    return value;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    let t = time;
    
    let fire = uv.y + fractal_noise(uv * vec2<f32>(10.0, 5.0) + t * 2.0);
    
    let flame = vec3<f32>(
        smoothstep(0.0, 0.3, fire) * 0.8,
        smoothstep(0.3, 0.6, fire) * 0.6,
        smoothstep(0.6, 1.0, fire) * 0.4
    );
    
    return vec4<f32>(flame, 1.0);
}"#.to_string(),
            },

            // Audio Reactive Templates
            ShaderTemplate {
                name: "Audio Reactive Wave".to_string(),
                description: "Wave pattern that responds to audio levels".to_string(),
                category: "Audio".to_string(),
                wgsl_code: r#"
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;
@group(0) @binding(2) var<uniform> audio_volume: f32;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    let t = time;
    
    let wave = sin(uv.x * 10.0 + t * 2.0 + audio_volume * 5.0) * 0.5 + 0.5;
    let boost = audio_volume * 0.3;
    
    let r = wave + boost;
    let g = 0.5 + 0.5 * sin(t + uv.y * 6.28318);
    let b = 0.5 + 0.5 * cos(t * 0.8);
    
    return vec4<f32>(r, g, b, 1.0);
}"#.to_string(),
            },

            // Fractal Templates
            ShaderTemplate {
                name: "Mandelbrot Fractal".to_string(),
                description: "Classic Mandelbrot fractal with beautiful coloring".to_string(),
                category: "Fractal".to_string(),
                wgsl_code: r#"
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

fn mandelbrot(c: vec2<f32>) -> f32 {
    var z = vec2<f32>(0.0, 0.0);
    let max_iter = 100.0;
    var iterations: f32 = 0.0;
    loop {
        if (dot(z, z) > 4.0 || iterations >= max_iter) {
            break;
        }
        z = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        iterations = iterations + 1.0;
    }
    return iterations / max_iter;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy - 0.5 * resolution) / min(resolution.x, resolution.y);
    let zoom = 2.0;
    let pan = vec2<f32>(-0.5, 0.0);
    let c = uv * zoom + pan;
    
    let m = mandelbrot(c);
    let col = vec3<f32>(m, m * 0.5, m * 0.8);
    
    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },

            ShaderTemplate {
                name: "Julia Set".to_string(),
                description: "Beautiful Julia set fractal".to_string(),
                category: "Fractal".to_string(),
                wgsl_code: r#"
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

fn julia(z: vec2<f32>, c: vec2<f32>) -> f32 {
    var zz = z;
    let max_iter = 100.0;
    var iterations: f32 = 0.0;
    loop {
        if (dot(zz, zz) > 4.0 || iterations >= max_iter) {
            break;
        }
        zz = vec2<f32>(zz.x * zz.x - zz.y * zz.y, 2.0 * zz.x * zz.y) + c;
        iterations = iterations + 1.0;
    }
    return iterations / max_iter;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy - 0.5 * resolution) / min(resolution.x, resolution.y);
    let zoom = 2.0;
    let c = vec2<f32>(0.285 + 0.01 * sin(time * 0.3), 0.01 + 0.01 * cos(time * 0.5));
    
    let j = julia(uv * zoom, c);
    let col = vec3<f32>(
        0.5 + 0.5 * sin(6.28318 * (j + 0.33)),
        0.5 + 0.5 * sin(6.28318 * (j + 0.67)),
        0.5 + 0.5 * sin(6.28318 * (j + 1.0))
    );
    
    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },

            // Tutorial Templates
            ShaderTemplate {
                name: "WGSL Basics".to_string(),
                description: "Learn WGSL fundamentals".to_string(),
                category: "Tutorial".to_string(),
                wgsl_code: r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    
    let r = uv.x;
    let g = uv.y;
    let b = 0.5 + 0.5 * sin(uniforms.time);
    
    return vec4<f32>(r, g, b, 1.0);
}"#.to_string(),
            },

            ShaderTemplate {
                name: "Simple Animation".to_string(),
                description: "Basic time-based animation".to_string(),
                category: "Tutorial".to_string(),
                wgsl_code: r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let res = uniforms.resolution;
    let uv = (position.xy - 0.5 * res) / min(res.x, res.y);
    let t = uniforms.time;
    
    let dist = length(uv);
    let pulse = 0.3 + 0.2 * sin(t * 3.0);
    let circle = 1.0 - smoothstep(pulse, pulse + 0.05, dist);
    
    let color = mix(vec3<f32>(0.1, 0.1, 0.8), vec3<f32>(0.8, 0.1, 0.1), circle);
    
    return vec4<f32>(color, 1.0);
}"#.to_string(),
            },
        ]
    }

    fn load_recent_files(&mut self) {
        // Load recent files from a config file
        if let Some(config_dir) = directories::ProjectDirs::from("com", "WGSLShaderStudio", "ShaderStudio") {
            let recent_files_path = config_dir.config_dir().join("recent_files.json");
            if recent_files_path.exists() {
                match std::fs::read_to_string(&recent_files_path) {
                    Ok(content) => {
                        if let Ok(files) = serde_json::from_str::<Vec<String>>(&content) {
                            self.recent_files = files.into_iter()
                                .filter(|p| std::path::Path::new(p).exists())
                                .map(|p| std::path::PathBuf::from(p))
                                .collect();
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load recent files: {}", e);
                    }
                }
            }
        }
    }

    fn save_recent_files(&self) {
        // Save recent files to config
        if let Some(config_dir) = directories::ProjectDirs::from("com", "WGSLShaderStudio", "ShaderStudio") {
            let recent_files_path = config_dir.config_dir().join("recent_files.json");
            if let Some(parent) = recent_files_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Failed to create config directory: {}", e);
                    return;
                }
            }

            let files_to_save: Vec<String> = self.recent_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();

            if let Err(e) = std::fs::write(&recent_files_path, serde_json::to_string_pretty(&files_to_save).unwrap_or_default()) {
                eprintln!("Failed to save recent files: {}", e);
            }
        }
    }

    fn load_theme_settings(&mut self) {
        // Load theme settings from config
        if let Some(config_dir) = directories::ProjectDirs::from("com", "WGSLShaderStudio", "ShaderStudio") {
            let theme_settings_path = config_dir.config_dir().join("theme_settings.json");
            if theme_settings_path.exists() {
                match std::fs::read_to_string(&theme_settings_path) {
                    Ok(content) => {
                        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                            // Load current theme
                            if let Some(theme) = settings.get("current_theme").and_then(|v| v.as_str()) {
                                self.current_theme = theme.to_string();
                            }
                            // Load brightness/contrast/font_size
                            if let Some(brightness) = settings.get("brightness").and_then(|v| v.as_f64()) {
                                self.brightness = brightness as f32;
                            }
    
                            if let Some(contrast) = settings.get("contrast").and_then(|v| v.as_f64()) {
                                self.contrast = contrast as f32;
                            }
    
                            if let Some(font_size) = settings.get("font_size").and_then(|v| v.as_f64()) {
                                self.font_size = font_size as f32;
                            }

                            // Load custom theme colors
                            if let Some(colors) = settings.get("custom_theme_colors").and_then(|v| v.as_object()) {
                                for (key, value) in colors {
                                    if let Some(arr) = value.as_array() {
                                        if arr.len() == 4 {
                                            let r = arr[0].as_u64().unwrap_or(0) as u8;
                                            let g = arr[1].as_u64().unwrap_or(0) as u8;
                                            let b = arr[2].as_u64().unwrap_or(0) as u8;
                                            let a = arr[3].as_u64().unwrap_or(255) as u8;
                                            self.custom_theme_colors.insert(key.clone(), egui::Color32::from_rgba_unmultiplied(r, g, b, a));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load theme settings: {}", e);
                    }
                }
            }
        }
    }

    fn save_theme_settings(&self) {
        // Save theme settings to config
        if let Some(config_dir) = directories::ProjectDirs::from("com", "WGSLShaderStudio", "ShaderStudio") {
            let theme_settings_path = config_dir.config_dir().join("theme_settings.json");
            if let Some(parent) = theme_settings_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Failed to create config directory: {}", e);
                    return;
                }
            }

            // Create settings object
            let mut settings = serde_json::Map::new();
            settings.insert("current_theme".to_string(), serde_json::Value::String(self.current_theme.clone()));
            settings.insert("brightness".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(self.brightness as f64).unwrap()));
            settings.insert("contrast".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(self.contrast as f64).unwrap()));
            settings.insert("font_size".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(self.font_size as f64).unwrap()));

            // Save custom theme colors
            let mut colors = serde_json::Map::new();
            for (key, color) in &self.custom_theme_colors {
                let [r, g, b, a] = color.to_srgba_unmultiplied();
                let color_array = serde_json::Value::Array(vec![
                    serde_json::Value::Number(serde_json::Number::from(r)),
                    serde_json::Value::Number(serde_json::Number::from(g)),
                    serde_json::Value::Number(serde_json::Number::from(b)),
                    serde_json::Value::Number(serde_json::Number::from(a)),
                ]);
                colors.insert(key.clone(), color_array);
            }
            settings.insert("custom_theme_colors".to_string(), serde_json::Value::Object(colors));

            if let Err(e) = std::fs::write(&theme_settings_path, serde_json::to_string_pretty(&serde_json::Value::Object(settings)).unwrap_or_default()) {
                eprintln!("Failed to save theme settings: {}", e);
            }
        }
    }

    fn compile_wgsl_shader(&mut self) {
        self.compilation_status = ShaderCompilationStatus::Compiling;
        self.shader_errors.clear();

        // Minimal validation: ensure required WGSL entry point structure exists
        let has_fragment = self.current_wgsl_code.contains("@fragment");
        let has_fs_main = self.current_wgsl_code.contains("fn fs_main");
        let has_position = self.current_wgsl_code.contains("@builtin(position)");
        let has_location0 = self.current_wgsl_code.contains("@location(0)");

        if has_fragment && has_fs_main && has_position && has_location0 {
            self.compilation_status = ShaderCompilationStatus::Success;
            println!("Shader compiled successfully");
        } else {
            self.compilation_status = ShaderCompilationStatus::Error("Compilation failed".to_string());
            if !has_fragment { self.shader_errors.push("Missing @fragment attribute".to_string()); }
            if !has_fs_main { self.shader_errors.push("Missing fs_main function".to_string()); }
            if !has_position { self.shader_errors.push("Missing @builtin(position) parameter".to_string()); }
            if !has_location0 { self.shader_errors.push("Missing @location(0) output".to_string()); }
        }

        self.run_reflection_diagnostics();
    }

    fn run_reflection_diagnostics(&mut self) {
        match naga::front::wgsl::parse_str(&self.current_wgsl_code) {
            Ok(module) => {
                for (_, var) in module.global_variables.iter() {
                    if let Some(binding) = var.binding {
                        let grp = binding.group;
                        let bnd = binding.binding;
                        self.shader_errors.push(format!("binding group {} binding {}", grp, bnd));
                    }
                }
            }
            Err(e) => {
                self.shader_errors.push(format!("naga parse error: {}", e));
            }
        }
    }

    fn load_template(&mut self, template_index: usize) {
        if let Some(template) = self.shader_templates.get(template_index) {
            self.current_wgsl_code = template.wgsl_code.clone();
            self.current_file = None;
            self.compile_wgsl_shader();
        }
    }

    fn load_expanded_template(&mut self, template: ShaderTemplate) {
        self.current_wgsl_code = template.wgsl_code.clone();
        self.current_file = None;
        self.compile_wgsl_shader();
    }

    fn new_shader(&mut self) {
        self.current_wgsl_code = Self::default_wgsl_shader();
        self.current_file = None;
        self.parameter_values.clear();
        self.compile_wgsl_shader();
    }

    fn open_file(&mut self) {
        let task = rfd::AsyncFileDialog::new()
            .add_filter("WGSL Files", &["wgsl"])
            .add_filter("GLSL Files", &["glsl", "frag", "vert"])
            .add_filter("ISF Files", &["fs"])
            .add_filter("All Files", &["*"])
            .pick_file();

        pollster::block_on(async {
            if let Some(file) = task.await {
                match std::fs::read_to_string(file.path()) {
                    Ok(content) => {
                        self.current_wgsl_code = content;
                        self.current_file = Some(file.path().to_path_buf());
                        self.add_to_recent_files(file.path().to_path_buf());
                        self.compile_wgsl_shader();
                        println!("Opened shader from: {:?}", file.path());
                    }
                    Err(e) => {
                        self.shader_errors.push(format!("Failed to read file: {}", e));
                    }
                }
            }
        });
    }

    fn save_file(&mut self, path: &PathBuf) {
        match std::fs::write(path, &self.current_wgsl_code) {
            Ok(_) => {
                self.current_file = Some(path.clone());
                self.add_to_recent_files(path.clone());
                self.shader_errors.clear();
                self.shader_errors.push(format!("Shader saved successfully to: {}", path.display()));
                println!("Shader saved to: {:?}", path);
            }
            Err(e) => {
                self.shader_errors.push(format!("Failed to save file: {}", e));
            }
        }
    }

    fn add_to_recent_files(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);
        // Add to front
        self.recent_files.insert(0, path);
        // Keep only last 10
        self.recent_files.truncate(10);
        self.save_recent_files();
    }

    fn load_shaders(&mut self) {
        match load_resolume_isf_shaders() {
            Ok(shaders) => {
                self.shaders = shaders;
                println!("Loaded {} shaders", self.shaders.len());
            }
            Err(e) => {
                eprintln!("Failed to load shaders: {}", e);
            }
        }
    }

    fn select_shader(&mut self, index: usize) {
        if index < self.shaders.len() {
            self.current_shader = Some(index);
            let shader = &self.shaders[index];

            // Initialize parameter values
            self.parameter_values.clear();
            for input in &shader.inputs {
                let default_value = input.default.unwrap_or(match input.input_type {
                    InputType::Float => 0.5,
                    InputType::Bool => 0.0,
                    _ => 0.0,
                });
                self.parameter_values.insert(input.name.clone(), default_value);
            }

            let shader_name = shader.name.clone();
            let src = shader.source.clone();
            let is_wgsl = src.contains("@fragment") || src.contains("var<uniform>") || src.contains("@vertex");
            
            if is_wgsl {
                self.current_wgsl_code = src;
            } else {
                // Use advanced ISF auto-converter for seamless conversion
                if let Some(ref mut converter) = self.isf_auto_converter {
                    // Create a temporary ISF file content for conversion
                    let isf_content = format!(r#"
                    /*{{
                        "NAME": "{}",
                        "DESCRIPTION": "Auto-converted ISF shader",
                        "INPUTS": []
                    }}*/
                    {}
                    "#, shader_name, src);
                    
                    // Write to temporary file for conversion
                    let temp_path = std::env::temp_dir().join(format!("temp_isf_{}.fs", shader_name.replace(" ", "_")));
                    if let Ok(_) = std::fs::write(&temp_path, isf_content) {
                        match converter.load_and_convert(&temp_path) {
                            Ok(advanced_isf) => {
                                if let Some(conversion_result) = &advanced_isf.converted_wgsl {
                                    self.current_wgsl_code = format!(
                                        "// ISF Auto-converted: {}\n// {}\n\n{}\n\n{}",
                                        shader_name,
                                        conversion_result.conversion_notes.join("\n// "),
                                        conversion_result.vertex_shader,
                                        conversion_result.fragment_shader
                                    );
                                    
                                    // Show performance hints if any
                                    if !conversion_result.performance_hints.is_empty() {
                                        println!("🚀 Performance hints for {}:", shader_name);
                                        for hint in &conversion_result.performance_hints {
                                            println!("  💡 {}", hint);
                                        }
                                    }
                                } else {
                                    // Fallback to basic conversion
                                    let converted = self.convert_isf_to_wgsl(&src);
                                    self.current_wgsl_code = converted;
                                }
                                
                                // Clean up temp file
                                let _ = std::fs::remove_file(&temp_path);
                            }
                            Err(e) => {
                                println!("⚠️  Advanced ISF conversion failed: {}, falling back to basic conversion", e);
                                let converted = self.convert_isf_to_wgsl(&src);
                                self.current_wgsl_code = converted;
                            }
                        }
                    } else {
                        // Fallback to basic conversion
                        let converted = self.convert_isf_to_wgsl(&src);
                        self.current_wgsl_code = converted;
                    }
                } else {
                    // Fallback to basic conversion
                    let converted = self.convert_isf_to_wgsl(&src);
                    self.current_wgsl_code = converted;
                }
            }
            self.compile_wgsl_shader();
            self.start_preview_rendering();
            // Analyze shader uniforms using wgsl_bindgen
            self.analyze_shader_uniforms(&shader_name);
            
            println!("Selected shader: {}", shader_name);
        }
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // File menu - VS Code style
                ui.menu_button("File", |ui| {
                    if ui.button("📄 New Shader").on_hover_text("Create new shader (Ctrl+N)").clicked() || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::N)) {
                        self.new_shader();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("📂 Open...").on_hover_text("Open shader file (Ctrl+O)").clicked() || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
                        self.open_file();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button("💾 Save").on_hover_text("Save shader (Ctrl+S)").clicked() || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
                        if let Some(path) = self.current_file.clone() {
                            self.save_file(&path);
                        }
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("💾 Save As...").on_hover_text("Save shader as... (Ctrl+Shift+S)").clicked() || ui.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::S)) {
                        let task = rfd::AsyncFileDialog::new()
                            .add_filter("WGSL Shaders", &["wgsl"])
                            .add_filter("GLSL Shaders", &["glsl", "frag", "vert"])
                            .add_filter("All Files", &["*"])
                            .save_file();

                        pollster::block_on(async {
                            if let Some(file) = task.await {
                                self.save_file(&file.path().to_path_buf());
                            }
                        });
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    ui.menu_button("🕒 Recent Files", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label("(No recent files)");
                        } else {
                            for recent_file in &self.recent_files {
                                if ui.button(recent_file.display().to_string()).clicked() {
                                    // Load file
                                    ui.close_kind(egui::UiKind::Menu);
                                }
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("🚪 Exit").on_hover_text("Exit application").clicked() {
                        std::process::exit(0);
                    }
                });

                // Edit menu - VS Code style
                ui.menu_button("Edit", |ui| {
                    if ui.button(".undo Undo").on_hover_text("Undo last action (Ctrl+Z)").clicked() {
                        // Undo functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button(".redo Redo").on_hover_text("Redo last action (Ctrl+Y)").clicked() {
                        // Redo functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button("✂️ Cut").on_hover_text("Cut selected text (Ctrl+X)").clicked() {
                        // Cut functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("📋 Copy").on_hover_text("Copy selected text (Ctrl+C)").clicked() {
                        // Copy functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("📌 Paste").on_hover_text("Paste from clipboard (Ctrl+V)").clicked() {
                        // Paste functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button("🔍 Find").on_hover_text("Find in code (Ctrl+F)").clicked() {
                        // Find functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("🔄 Replace").on_hover_text("Replace in code (Ctrl+H)").clicked() {
                        // Replace functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });

                // View menu - VS Code style panels
                ui.menu_button("View", |ui| {
                    ui.label("⚪ Code Editor (Always On)");
                    ui.label("⚪ Live Preview (Always On)");
                    if ui.checkbox(&mut self.show_node_editor, "🎨 Node Editor").on_hover_text("Toggle node editor").clicked() {
                        // State change handled by checkbox
                    }
                    if ui.checkbox(&mut self.show_file_browser, "📂 File Browser").on_hover_text("Toggle file browser").clicked() {
                        // State change handled by checkbox
                    }
                    if ui.checkbox(&mut self.show_converter, "🔄 Shader Converter").on_hover_text("Toggle shader converter").clicked() {
                        // State change handled by checkbox
                    }
                    if ui.checkbox(&mut self.show_audio_panel, "🎵 Audio Panel").on_hover_text("Toggle audio panel").clicked() {
                        // State change handled by checkbox
                    }
                    if ui.checkbox(&mut self.show_midi_panel, "🎹 MIDI Panel").on_hover_text("Toggle MIDI panel").clicked() {
                        // State change handled by checkbox
                    }
                    if ui.checkbox(&mut self.show_gesture_panel, "✋ Gesture Control").on_hover_text("Toggle gesture control").clicked() {
                        // State change handled by checkbox
                    }
                    ui.separator();
                    ui.menu_button("🎨 Themes", |ui| {
                        if ui.button("🎨 Professional Dark").on_hover_text("Apply professional dark theme").clicked() {
                            self.apply_theme("professional_dark", ctx);
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                        if ui.button("☀️ Professional Light").on_hover_text("Apply professional light theme").clicked() {
                            self.apply_theme("professional_light", ctx);
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                        if ui.button("🌙 Midnight Blue").on_hover_text("Apply midnight blue theme").clicked() {
                            self.apply_theme("midnight_blue", ctx);
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                        if ui.button("🌅 Sunrise Orange").on_hover_text("Apply sunrise orange theme").clicked() {
                            self.apply_theme("sunrise_orange", ctx);
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                        if ui.button("🌲 Forest Green").on_hover_text("Apply forest green theme").clicked() {
                            self.apply_theme("forest_green", ctx);
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                        if ui.button("💜 Purple Haze").on_hover_text("Apply purple haze theme").clicked() {
                            self.apply_theme("purple_haze", ctx);
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                        ui.separator();
                        if ui.button("🎨 Custom Theme Editor").on_hover_text("Open custom theme editor").clicked() {
                            self.show_theme_editor = true;
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    });
                    ui.separator();
                    ui.menu_button("⚙️ Appearance", |ui| {
                        ui.label("Brightness:");
                        ui.add(egui::Slider::new(&mut self.brightness, -0.5..=0.5).text(""));
                        ui.label("Contrast:");
                        ui.add(egui::Slider::new(&mut self.contrast, 0.5..=2.0).text(""));
                        ui.label("Font Size:");
                        ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0).text(""));
                        ui.separator();
                        if ui.button("Apply Changes").on_hover_text("Apply appearance changes").clicked() {
                            self.apply_theme(&self.current_theme.clone(), ctx);
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                        if ui.button("Reset to Defaults").on_hover_text("Reset appearance to defaults").clicked() {
                            self.brightness = 0.0;
                            self.contrast = 1.0;
                            self.font_size = 14.0;
                            self.apply_theme(&self.current_theme.clone(), ctx);
                            self.save_theme_settings();
                            ui.close_kind(egui::UiKind::Menu);
                        }
                    });
                    ui.separator();
                    if ui.button("🔄 Reset Layout").on_hover_text("Reset window layout (Ctrl+R)").clicked() || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::R)) {
                        self.reset_layout();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });

                // Tools menu - Shader-specific tools
                ui.menu_button("Tools", |ui| {
                    if ui.button("▶️ Compile Shader").on_hover_text("Compile current shader (F5)").clicked() {
                        self.compile_wgsl_shader();
                    }
                    ui.separator();
                    if ui.button("📤 Export to GLSL").on_hover_text("Export shader to GLSL").clicked() {
                        self.export_to_glsl();
                    }
                    if ui.button("📤 Export to HLSL").on_hover_text("Export shader to HLSL").clicked() {
                        self.export_to_hlsl();
                    }
                    ui.separator();
                    if ui.button("📥 Import ISF Shader").on_hover_text("Import ISF shader").clicked() {
                        self.import_isf_shader();
                    }
                    ui.separator();
                    if ui.button("✋ Initialize Leap Motion").on_hover_text("Initialize Leap Motion controller").clicked() {
                        self.initialize_leap_motion();
                    }
                    if ui.button("✋ Initialize MediaPipe").on_hover_text("Initialize MediaPipe gesture recognition").clicked() {
                        self.initialize_mediapipe();
                    }
                });

                // Help menu - VS Code style
                ui.menu_button("Help", |ui| {
                    if ui.button("📚 Documentation").on_hover_text("Open documentation").clicked() {
                        self.show_documentation_dialog = true;
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("⌨️ Keyboard Shortcuts").on_hover_text("Show keyboard shortcuts").clicked() {
                        self.show_shortcuts_dialog = true;
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button("ℹ️ About WGSL Shader Studio").on_hover_text("Show application information").clicked() {
                        self.show_about_dialog = true;
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });
            });
        });
    }

    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        // Professional code editor similar to VS Code with enhanced features - VS Code style
        egui::Frame::NONE
            .fill(egui::Color32::from_rgb(25, 25, 30))
            .inner_margin(0.0)
            .show(ui, |ui| {
                // Enhanced toolbar with more professional features - VS Code style
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(35, 35, 40))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // File operations - VS Code style toolbar
                            if ui.button("📄 New").on_hover_text("Create new shader (Ctrl+N)").clicked() || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::N)) {
                                self.current_wgsl_code = String::from(
"// WGSL Shader Studio - New Shader
// Press F5 or click Compile to compile your shader

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) % 3 - 1);
    let y = f32(i32(in_vertex_index) / 3 * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}");
                                self.current_file = None;
                                self.shader_errors.clear();
                                self.compilation_status = ShaderCompilationStatus::NotCompiled;
                            }
                            if ui.button("📂 Open").on_hover_text("Open shader file (Ctrl+O)").clicked() || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
                                let task = rfd::AsyncFileDialog::new()
                                    .add_filter("WGSL Shaders", &["wgsl"])
                                    .add_filter("GLSL Shaders", &["glsl", "frag", "vert"])
                                    .add_filter("All Files", &["*"])
                                    .pick_file();

                                pollster::block_on(async {
                                    if let Some(file) = task.await {
                                        // Load file content
                                        if let Ok(content) = std::fs::read_to_string(&file.path()) {
                                            self.current_wgsl_code = content;
                                            self.current_file = Some(file.path().to_path_buf());
                                            self.add_to_recent_files(file.path().to_path_buf());
                                            self.compile_wgsl_shader();
                                        }
                                    }
                                });
                            }
                            if ui.button("💾 Save").on_hover_text("Save shader (Ctrl+S)").clicked() || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
                                if let Some(path) = &self.current_file {
                                    self.save_file(&path.clone());
                                } else {
                                    // Show save dialog
                                    let task = rfd::AsyncFileDialog::new()
                                        .add_filter("WGSL Shaders", &["wgsl"])
                                        .add_filter("All Files", &["*"])
                                        .save_file();

                                    pollster::block_on(async {
                                        if let Some(file) = task.await {
                                            self.save_file(&file.path().to_path_buf());
                                        }
                                    });
                                }
                            }
                            if ui.button("💾 Save As").on_hover_text("Save shader as...").clicked() {
                                let task = rfd::AsyncFileDialog::new()
                                    .add_filter("WGSL Shaders", &["wgsl"])
                                    .add_filter("All Files", &["*"])
                                    .save_file();

                                pollster::block_on(async {
                                    if let Some(file) = task.await {
                                        self.save_file(&file.path().to_path_buf());
                                    }
                                });
                            }

                            ui.separator();

                            // Edit operations
                            if ui.button("✂️ Cut").on_hover_text("Cut selected text (Ctrl+X)").clicked() {
                                // Cut functionality would be implemented
                            }
                            if ui.button("📋 Copy").on_hover_text("Copy selected text (Ctrl+C)").clicked() {
                                // Copy functionality would be implemented
                            }
                            if ui.button("📌 Paste").on_hover_text("Paste from clipboard (Ctrl+V)").clicked() {
                                // Paste functionality would be implemented
                            }
                            if ui.button(".undo").on_hover_text("Undo last action (Ctrl+Z)").clicked() {
                                // Undo functionality would be implemented
                            }
                            if ui.button(".redo").on_hover_text("Redo last action (Ctrl+Y)").clicked() {
                                // Redo functionality would be implemented
                            }

                            ui.separator();

                            // Compilation operations
                            if ui.button("▶ Compile").on_hover_text("Compile shader (F5)").clicked() || ui.input(|i| i.key_pressed(egui::Key::F5)) {
                                self.compile_wgsl_shader();
                            }
                            if ui.button("🔧 Format").on_hover_text("Format code (Ctrl+Shift+F)").clicked() || ui.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::F)) {
                                self.format_wgsl_code();
                            }

                            ui.separator();

                            // Compilation status with clear indicators
                            let (status_icon, status_color) = match &self.compilation_status {
                                ShaderCompilationStatus::NotCompiled => ("○", egui::Color32::GRAY),
                                ShaderCompilationStatus::Compiling => ("●", egui::Color32::YELLOW),
                                ShaderCompilationStatus::Success => ("●", egui::Color32::GREEN),
                                ShaderCompilationStatus::Error(_) => ("●", egui::Color32::RED),
                            };

                            ui.colored_label(status_color, format!("{} {}",
                                status_icon,
                                match &self.compilation_status {
                                    ShaderCompilationStatus::NotCompiled => "Ready",
                                    ShaderCompilationStatus::Compiling => "Compiling...",
                                    ShaderCompilationStatus::Success => "Compiled",
                                    ShaderCompilationStatus::Error(_) => "Error",
                                }
                            ));

                            ui.separator();

                            // Find functionality
                            if ui.button("🔍 Find").on_hover_text("Find in code (Ctrl+F)").clicked() {
                                // Find functionality would be implemented
                            }
                            if ui.button("🔄 Replace").on_hover_text("Replace in code (Ctrl+H)").clicked() {
                                // Replace functionality would be implemented
                            }

                            ui.separator();

                            // Template functionality
                            if ui.button("📋 Templates").on_hover_text("Insert shader template").clicked() {
                                // Template functionality would be implemented
                            }
                        });
                    });

                // Show errors only when compilation failed
                if matches!(self.compilation_status, ShaderCompilationStatus::Error(_)) && !self.shader_errors.is_empty() {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(50, 20, 20))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::RED, "❌ Errors:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("✕ Clear").clicked() {
                                        self.shader_errors.clear();
                                    }
                                });
                            });
                            ui.separator();
                            egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                                for (i, error) in self.shader_errors.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(egui::Color32::RED, format!("{}.", i + 1));
                                        ui.label(egui::RichText::new(error).color(egui::Color32::LIGHT_RED));
                                    });
                                    ui.add_space(4.0);
                                }
                            });
                        });
                }

                // Enhanced success message
                if let ShaderCompilationStatus::Success = self.compilation_status {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(20, 50, 20))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::GREEN, "✓ Shader compiled successfully");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("✕").clicked() {
                                        // Reset status after user acknowledges
                                    }
                                });
                            });
                        });
                }

                // Main code editor with enhanced syntax highlighting and features
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(30, 30, 35))
                    .inner_margin(0.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Enhanced line numbers column with better styling
                                egui::Frame::NONE
                                    .fill(egui::Color32::from_rgb(25, 25, 30))
                                    .inner_margin(4.0)
                                    .show(ui, |ui| {
                                        ui.set_width(40.0);
                                        let line_count = self.current_wgsl_code.lines().count().max(1);
                                        for i in 1..=line_count {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                                ui.add_space(2.0);
                                                ui.colored_label(egui::Color32::from_rgb(100, 100, 120), format!("{:2}", i));
                                            });
                                            ui.add_space(2.0); // Add spacing between line numbers
                                        }
                                    });

                                ui.separator();

                                // Enhanced code editing area with better styling
                                ui.vertical(|ui| {
                                    let response = ui.add(
                                        egui::TextEdit::multiline(&mut self.current_wgsl_code)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_rows(25)
                                            .desired_width(f32::INFINITY)
                                            .code_editor() // Use code editor style
                                            .hint_text("// Write your WGSL shader code here...")
                                            .margin(egui::Vec2::new(10.0, 10.0)) // Add some margin
                                    );

                                    // Enhanced context menu with more options
                                    response.context_menu(|ui| {
                                        if ui.button("✂️ Cut").clicked() {
                                            // Cut functionality
                                            ui.close();
                                        }
                                        if ui.button("📋 Copy").clicked() {
                                            // Copy functionality
                                            ui.close();
                                        }
                                        if ui.button("📌 Paste").clicked() {
                                            // Paste functionality
                                            ui.close();
                                        }
                                        ui.separator();
                                        if ui.button("🔧 Format Code").clicked() {
                                            self.format_code();
                                            ui.close();
                                        }
                                        if ui.button("✅ Select All").clicked() {
                                            // Select all functionality
                                            ui.close();
                                        }
                                        ui.separator();
                                        if ui.button("📖 Documentation").clicked() {
                                            // Show documentation
                                            ui.close();
                                        }
                                    });
                                });
                            });
                        });
                    });

                // Enhanced status bar with more information - VS Code style
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(35, 35, 40))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let lines = self.current_wgsl_code.lines().count();
                            let chars = self.current_wgsl_code.chars().count();
                            let words = self.current_wgsl_code.split_whitespace().count();

                            ui.label(egui::RichText::new(format!("Lines: {}", lines)).color(egui::Color32::LIGHT_GRAY));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("Words: {}", words)).color(egui::Color32::LIGHT_GRAY));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("Chars: {}", chars)).color(egui::Color32::LIGHT_GRAY));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("File: {}", self.current_file.as_ref().map_or("Untitled".to_string(), |p| p.file_name().unwrap_or_default().to_string_lossy().to_string()))).color(egui::Color32::LIGHT_GRAY));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new("WGSL").color(egui::Color32::GOLD));
                            });
                        });
                    });
            });
    }

    fn render_node_editor(&mut self, ui: &mut egui::Ui) {
        // Professional node-based shader editor - VS Code style
        egui::Frame::NONE
            .fill(egui::Color32::from_rgb(25, 25, 30))
            .inner_margin(0.0)
            .show(ui, |ui| {
                // Enhanced toolbar with more professional features
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(35, 35, 40))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🎨 Node Palette").color(egui::Color32::LIGHT_BLUE));
                            ui.separator();
                            
                            // Node creation buttons with hover text
                            if ui.button("📥 Input").on_hover_text("Add input node").clicked() {
                                self.add_node(NodeType::Input, egui::pos2(100.0, 100.0));
                            }
                            if ui.button("📤 Output").on_hover_text("Add output node").clicked() {
                                self.add_node(NodeType::Output, egui::pos2(400.0, 100.0));
                            }
                            if ui.button("➕ Math").on_hover_text("Add math operation node").clicked() {
                                self.add_node(NodeType::Math, egui::pos2(250.0, 200.0));
                            }
                            if ui.button("🎨 Color").on_hover_text("Add color operation node").clicked() {
                                self.add_node(NodeType::Color, egui::pos2(250.0, 300.0));
                            }
                            if ui.button("🔄 Transform").on_hover_text("Add transform node").clicked() {
                                self.add_node(NodeType::Transform, egui::pos2(250.0, 400.0));
                            }
                            if ui.button("🖼️ Texture").on_hover_text("Add texture node").clicked() {
                                self.add_node(NodeType::Texture, egui::pos2(250.0, 500.0));
                            }
                            
                            ui.separator();
                            
                            // Graph controls
                            if ui.button("🗑️ Clear").on_hover_text("Clear all nodes").clicked() {
                                self.nodes.clear();
                                self.connections.clear();
                            }
                            if ui.button("💾 Save").on_hover_text("Save node graph").clicked() {
                                // Save node graph
                            }
                            if ui.button("📂 Load").on_hover_text("Load node graph").clicked() {
                                // Load node graph
                            }
                            
                            ui.separator();
                            
                            // Graph statistics
                            ui.label(egui::RichText::new(format!("📊 Nodes: {}", self.nodes.len())).color(egui::Color32::LIGHT_GRAY));
                            ui.label(egui::RichText::new(format!("🔗 Connections: {}", self.connections.len())).color(egui::Color32::LIGHT_GRAY));
                            
                            ui.separator();
                            
                            // Convert to code
                            if ui.button("🔄 To Code").on_hover_text("Convert node graph to WGSL code").clicked() {
                                self.convert_nodes_to_code();
                            }
                        });
                    });

                ui.separator();

                // Node graph area with zoom and pan
                let available_size = ui.available_size();
                let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

                // Handle dragging with improved logic
                if let Some(dragged_node) = self.dragged_node {
                    if let Some(node) = self.nodes.iter_mut().find(|n| n.id == dragged_node) {
                        node.position += response.drag_delta();
                    }
                }

                // Handle connection creation with improved pin detection
                if response.drag_started() {
                    // Check if we started dragging from a pin
                    if let Some(pointer_pos) = response.interact_pointer_pos() {
                        for node in &self.nodes {
                            let node_rect = egui::Rect::from_min_size(node.position, node.size);

                            // Check input pins (left side)
                            for (i, input) in node.inputs.iter().enumerate() {
                                let pin_pos = node.position + egui::vec2(0.0, 20.0 + i as f32 * 20.0);
                                let pin_rect = egui::Rect::from_center_size(pin_pos, egui::vec2(8.0, 8.0));
                                if pin_rect.contains(pointer_pos) {
                                    // Start connection from input pin
                                    self.pending_connection = Some(PendingConnection {
                                        from_node: node.id,
                                        from_pin: input.id,
                                        from_pos: pin_pos,
                                    });
                                    println!("Started connection from input pin");
                                    break;
                                }
                            }

                            // Check output pins (right side)
                            for (i, output) in node.outputs.iter().enumerate() {
                                let pin_pos = node.position + egui::vec2(node.size.x, 20.0 + i as f32 * 20.0);
                                let pin_rect = egui::Rect::from_center_size(pin_pos, egui::vec2(8.0, 8.0));
                                if pin_rect.contains(pointer_pos) {
                                    // Start connection from output pin
                                    self.pending_connection = Some(PendingConnection {
                                        from_node: node.id,
                                        from_pin: output.id,
                                        from_pos: pin_pos,
                                    });
                                    println!("Started connection from output pin");
                                    break;
                                }
                            }
                        }
                    }
                }

                // Handle connection completion with validation
                if response.drag_delta().length() > 0.0 && !response.dragged() {
                    if let Some(pending) = self.pending_connection.take() {
                        if let Some(pointer_pos) = response.interact_pointer_pos() {
                            // Check if we released on a compatible pin
                            for node in &self.nodes {
                                // Skip if trying to connect to same node
                                if node.id == pending.from_node {
                                    continue;
                                }

                                let node_rect = egui::Rect::from_min_size(node.position, node.size);

                                // Check input pins for connection completion
                                for (i, input) in node.inputs.iter().enumerate() {
                                    let pin_pos = node.position + egui::vec2(0.0, 20.0 + i as f32 * 20.0);
                                    let pin_rect = egui::Rect::from_center_size(pin_pos, egui::vec2(8.0, 8.0));
                                    if pin_rect.contains(pointer_pos) {
                                        // Validate connection types
                                        if self.can_connect_pins(pending.from_pin, input.id) {
                                            // Create the connection
                                            self.connections.push(NodeConnection {
                                                from_node: pending.from_node,
                                                from_pin: pending.from_pin,
                                                to_node: node.id,
                                                to_pin: input.id,
                                            });
                                            println!("Connection created!");
                                        }
                                        break;
                                    }
                                }

                                // Check output pins for connection completion
                                for (i, output) in node.outputs.iter().enumerate() {
                                    let pin_pos = node.position + egui::vec2(node.size.x, 20.0 + i as f32 * 20.0);
                                    let pin_rect = egui::Rect::from_center_size(pin_pos, egui::vec2(8.0, 8.0));
                                    if pin_rect.contains(pointer_pos) {
                                        // Validate connection types
                                        if self.can_connect_pins(pending.from_pin, output.id) {
                                            // Create the connection
                                            self.connections.push(NodeConnection {
                                                from_node: pending.from_node,
                                                from_pin: pending.from_pin,
                                                to_node: node.id,
                                                to_pin: output.id,
                                            });
                                            println!("Connection created!");
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                // Draw pending connection line with bezier curve
                if let Some(pending) = &self.pending_connection {
                    if let Some(pointer_pos) = response.interact_pointer_pos() {
                        // Draw bezier curve for better visual feedback
                        let control_point1 = pending.from_pos + egui::vec2(50.0, 0.0);
                        let control_point2 = pointer_pos - egui::vec2(50.0, 0.0);
                        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape::from_points_stroke(
                            [pending.from_pos, control_point1, control_point2, pointer_pos],
                            false,
                            egui::Color32::BLACK,
                            egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        )));
                    }
                }

                // Handle mouse interactions
                if response.drag_started() {
                // Check if we started dragging on a node (not on a pin)
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let mut found_pin = false;
                    for node in &self.nodes {
                        // Check pins first
                        for (i, _) in node.inputs.iter().enumerate() {
                            let pin_pos = node.position + egui::vec2(0.0, 20.0 + i as f32 * 20.0);
                            let pin_rect = egui::Rect::from_center_size(pin_pos, egui::vec2(8.0, 8.0));
                            if pin_rect.contains(pointer_pos) {
                                found_pin = true;
                                break;
                            }
                        }
                        if found_pin { break; }
                        for (i, _) in node.outputs.iter().enumerate() {
                            let pin_pos = node.position + egui::vec2(node.size.x, 20.0 + i as f32 * 20.0);
                            let pin_rect = egui::Rect::from_center_size(pin_pos, egui::vec2(8.0, 8.0));
                            if pin_rect.contains(pointer_pos) {
                                found_pin = true;
                                break;
                            }
                        }
                        if found_pin { break; }
                    }

                    if !found_pin {
                        for node in &self.nodes {
                            let node_rect = egui::Rect::from_min_size(node.position, node.size);
                            if node_rect.contains(pointer_pos) {
                                self.dragged_node = Some(node.id);
                                break;
                            }
                        }
                    }
                }
            }

            if !response.dragged() {
                self.dragged_node = None;
            }

            // Draw connections with bezier curves
            for connection in &self.connections {
                if let (Some(from_node), Some(to_node)) = (
                    self.nodes.iter().find(|n| n.id == connection.from_node),
                    self.nodes.iter().find(|n| n.id == connection.to_node)
                ) {
                    // Find pin positions
                    let from_pin_pos = self.get_pin_position(from_node, connection.from_pin, true);
                    let to_pin_pos = self.get_pin_position(to_node, connection.to_pin, false);

                    // Draw bezier curve
                    let control_point1 = from_pin_pos + egui::vec2(50.0, 0.0);
                    let control_point2 = to_pin_pos - egui::vec2(50.0, 0.0);
                    painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape::from_points_stroke(
                        [from_pin_pos, control_point1, control_point2, to_pin_pos],
                        false,
                        egui::Color32::BLACK,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    )));
                }
            }

            // Draw nodes with professional visuals and enhanced styling
            for node in &self.nodes {
                let node_rect = egui::Rect::from_min_size(node.position, node.size);

                // Enhanced hover detection
                let is_hovered = response.hovered() && node_rect.contains(response.hover_pos().unwrap_or(egui::pos2(0.0, 0.0)));
                let is_selected = Some(node.id) == self.selected_node;

                // Professional node styling with gradients and shadows
                let base_color = match node.node_type {
                    // Basic I/O
                    NodeType::Input => egui::Color32::from_rgb(76, 175, 80),    // Green
                    NodeType::Output => egui::Color32::from_rgb(244, 67, 54),   // Red
                    NodeType::Uniform => egui::Color32::from_rgb(96, 125, 139), // Blue Grey
                    NodeType::TextureInput => egui::Color32::from_rgb(255, 87, 34), // Deep Orange

                    // Math Operations
                    NodeType::Math => egui::Color32::from_rgb(33, 150, 243),    // Blue
                    NodeType::Trigonometry => egui::Color32::from_rgb(0, 150, 136), // Teal
                    NodeType::VectorMath => egui::Color32::from_rgb(63, 81, 181), // Indigo
                    NodeType::MatrixMath => egui::Color32::from_rgb(103, 58, 183), // Deep Purple

                    // Color Operations
                    NodeType::Color => egui::Color32::from_rgb(156, 39, 176),   // Purple
                    NodeType::ColorAdjustment => egui::Color32::from_rgb(233, 30, 99), // Pink
                    NodeType::ColorMix => egui::Color32::from_rgb(121, 85, 72), // Brown
                    NodeType::ColorSpace => egui::Color32::from_rgb(158, 158, 158), // Grey

                    // Texture Operations
                    NodeType::Texture => egui::Color32::from_rgb(255, 152, 0),  // Orange
                    NodeType::TextureSample => egui::Color32::from_rgb(255, 193, 7), // Amber
                    NodeType::TextureTransform => egui::Color32::from_rgb(255, 235, 59), // Yellow
                    NodeType::TextureBlend => egui::Color32::from_rgb(205, 220, 57), // Lime

                    // Geometry & 3D
                    NodeType::Transform => egui::Color32::from_rgb(0, 188, 212), // Cyan
                    NodeType::Geometry => egui::Color32::from_rgb(0, 150, 136), // Teal
                    NodeType::Volumetric => egui::Color32::from_rgb(3, 169, 244), // Light Blue
                    NodeType::PointCloud => egui::Color32::from_rgb(33, 150, 243), // Blue

                    // Advanced Rendering
                    NodeType::Lighting => egui::Color32::from_rgb(255, 235, 59), // Yellow
                    NodeType::Material => egui::Color32::from_rgb(255, 152, 0), // Orange
                    NodeType::BRDF => egui::Color32::from_rgb(255, 87, 34), // Deep Orange
                    NodeType::RayMarching => egui::Color32::from_rgb(121, 85, 72), // Brown

                    // Neural & AI
                    NodeType::NeRF => egui::Color32::from_rgb(156, 39, 176), // Purple
                    NodeType::MLInference => egui::Color32::from_rgb(103, 58, 183), // Deep Purple

                    // Audio & Time
                    NodeType::AudioReactive => egui::Color32::from_rgb(76, 175, 80), // Green
                    NodeType::Time => egui::Color32::from_rgb(0, 150, 136), // Teal
                    NodeType::Oscillator => egui::Color32::from_rgb(0, 188, 212), // Cyan

                    // Post Processing
                    NodeType::Filter => egui::Color32::from_rgb(255, 235, 59), // Yellow
                    NodeType::Blur => egui::Color32::from_rgb(158, 158, 158), // Grey
                    NodeType::Distortion => egui::Color32::from_rgb(244, 67, 54), // Red
                    NodeType::Effects => egui::Color32::from_rgb(233, 30, 99), // Pink

                    // Utility
                    NodeType::Constant => egui::Color32::from_rgb(96, 125, 139), // Blue Grey
                    NodeType::Variable => egui::Color32::from_rgb(121, 85, 72), // Brown
                    NodeType::Switch => egui::Color32::from_rgb(63, 81, 181), // Indigo
                    NodeType::Loop => egui::Color32::from_rgb(3, 169, 244), // Light Blue
                };

                // Dynamic color based on state
                let bg_color = if is_selected {
                    egui::Color32::from_rgb(
                        (base_color.r() as f32 * 1.3).min(255.0) as u8,
                        (base_color.g() as f32 * 1.3).min(255.0) as u8,
                        (base_color.b() as f32 * 1.3).min(255.0) as u8,
                    )
                } else if is_hovered {
                    egui::Color32::from_rgb(
                        (base_color.r() as f32 * 1.1).min(255.0) as u8,
                        (base_color.g() as f32 * 1.1).min(255.0) as u8,
                        (base_color.b() as f32 * 1.1).min(255.0) as u8,
                    )
                } else {
                    base_color
                };

                // Draw shadow effect
                let shadow_rect = node_rect.translate(egui::vec2(2.0, 2.0));
                painter.rect_filled(shadow_rect, 6.0, egui::Color32::from_black_alpha(50));

                // Main node background
                painter.rect_filled(node_rect, 6.0, bg_color);

                // Node border
                let border_color = if is_selected {
                    egui::Color32::YELLOW
                } else if is_hovered {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_rgb(100, 100, 100)
                };
                painter.rect_stroke(node_rect, 6.0, egui::Stroke::new(2.0, border_color), egui::StrokeKind::Middle);

                // Professional node header with gradient
                let header_rect = egui::Rect::from_min_size(
                    node.position,
                    egui::vec2(node.size.x, 24.0)
                );
                let header_color = egui::Color32::from_rgb(
                    (bg_color.r() as f32 * 0.8) as u8,
                    (bg_color.g() as f32 * 0.8) as u8,
                    (bg_color.b() as f32 * 0.8) as u8,
                );
                painter.rect_filled(header_rect, 6.0, header_color);

                // Node icon and title
                let icon = match node.node_type {
                    // Basic I/O
                    NodeType::Input => "📥",
                    NodeType::Output => "📤",
                    NodeType::Uniform => "🎛️",
                    NodeType::TextureInput => "🖼️",
                    
                    // Math Operations
                    NodeType::Math => "🔢",
                    NodeType::Trigonometry => "📐",
                    NodeType::VectorMath => "↗️",
                    NodeType::MatrixMath => "🔲",
                    
                    // Color Operations
                    NodeType::Color => "🎨",
                    NodeType::ColorAdjustment => "⚙️",
                    NodeType::ColorMix => "🔄",
                    NodeType::ColorSpace => "🌈",
                    
                    // Texture Operations
                    NodeType::Texture => "🖼️",
                    NodeType::TextureSample => "📋",
                    NodeType::TextureTransform => "🔧",
                    NodeType::TextureBlend => "🥤",
                    
                    // Geometry & 3D
                    NodeType::Transform => "🔄",
                    NodeType::Geometry => "📐",
                    NodeType::Volumetric => "🌫️",
                    NodeType::PointCloud => "⚫",
                    
                    // Advanced Rendering
                    NodeType::Lighting => "💡",
                    NodeType::Material => "🧱",
                    NodeType::BRDF => "🌟",
                    NodeType::RayMarching => "🚀",
                    
                    // Neural & AI
                    NodeType::NeRF => "🧠",
                    NodeType::MLInference => "🤖",
                    
                    // Audio & Time
                    NodeType::AudioReactive => "🎵",
                    NodeType::Time => "⏰",
                    NodeType::Oscillator => "📈",
                    
                    // Post Processing
                    NodeType::Filter => "🔍",
                    NodeType::Blur => "🌫️",
                    NodeType::Distortion => "🌊",
                    NodeType::Effects => "✨",
                    
                    // Utility
                    NodeType::Constant => "🔢",
                    NodeType::Variable => "📝",
                    NodeType::Switch => "🔀",
                    NodeType::Loop => "🔁",
                };

                painter.text(
                    node.position + egui::vec2(8.0, 6.0),
                    egui::Align2::LEFT_TOP,
                    format!("{} {}", icon, node.title),
                    egui::FontId::proportional(14.0).clone(),
                    egui::Color32::WHITE,
                );

                // Node inputs (left side) with enhanced styling
                for (i, input) in node.inputs.iter().enumerate() {
                    let pin_pos = node.position + egui::vec2(0.0, 32.0 + i as f32 * 20.0);

                    // Input pin with glow effect
                    painter.circle_filled(pin_pos, 5.0, egui::Color32::from_rgb(100, 149, 237)); // Cornflower blue
                    painter.circle_stroke(pin_pos, 5.0, egui::Stroke::new(2.0, egui::Color32::WHITE));

                    // Input label
                    painter.text(
                        pin_pos + egui::vec2(12.0, -4.0),
                        egui::Align2::LEFT_CENTER,
                        &input.name,
                        egui::FontId::proportional(11.0),
                        egui::Color32::WHITE,
                    );
                }

                // Node outputs (right side) with enhanced styling
                for (i, output) in node.outputs.iter().enumerate() {
                    let pin_pos = node.position + egui::vec2(node.size.x, 32.0 + i as f32 * 20.0);

                    // Output pin with glow effect
                    painter.circle_filled(pin_pos, 5.0, egui::Color32::from_rgb(255, 105, 180)); // Hot pink
                    painter.circle_stroke(pin_pos, 5.0, egui::Stroke::new(2.0, egui::Color32::WHITE));

                    // Output label
                    painter.text(
                        pin_pos + egui::vec2(-12.0, -4.0),
                        egui::Align2::RIGHT_CENTER,
                        &output.name,
                        egui::FontId::proportional(11.0),
                        egui::Color32::WHITE,
                    );
                }

                // Node value display for certain node types
                if matches!(node.node_type, NodeType::Input | NodeType::Math) {
                    let value_rect = egui::Rect::from_min_size(
                        node.position + egui::vec2(8.0, node.size.y - 20.0),
                        egui::vec2(node.size.x - 16.0, 16.0)
                    );
                    painter.rect_filled(value_rect, 2.0, egui::Color32::from_black_alpha(100));
                    painter.text(
                        value_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{:.2}", node.value),
                        egui::FontId::proportional(10.0),
                        egui::Color32::WHITE,
                    );
                }
            }

            // Context menu for node editor
            response.context_menu(|ui| {
                if ui.button("Add Input Node").clicked() {
                    let pos = response.interact_pointer_pos().unwrap_or(egui::pos2(100.0, 100.0));
                    self.add_node(NodeType::Input, pos);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Add Output Node").clicked() {
                    let pos = response.interact_pointer_pos().unwrap_or(egui::pos2(400.0, 100.0));
                    self.add_node(NodeType::Output, pos);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Add Math Node").clicked() {
                    let pos = response.interact_pointer_pos().unwrap_or(egui::pos2(250.0, 200.0));
                    self.add_node(NodeType::Math, pos);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Add Color Node").clicked() {
                    let pos = response.interact_pointer_pos().unwrap_or(egui::pos2(250.0, 300.0));
                    self.add_node(NodeType::Color, pos);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Add Texture Node").clicked() {
                    let pos = response.interact_pointer_pos().unwrap_or(egui::pos2(250.0, 400.0));
                    self.add_node(NodeType::Texture, pos);
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Clear All").clicked() {
                    self.nodes.clear();
                    self.connections.clear();
                    ui.close_kind(egui::UiKind::Menu);
                }
            });

            // Handle node interactions
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                if response.clicked() {
                    // Select node or clear selection
                    self.selected_node = None;
                    for node in &self.nodes {
                        let node_rect = egui::Rect::from_min_size(node.position, node.size);
                        if node_rect.contains(pointer_pos) {
                            self.selected_node = Some(node.id);
                            break;
                        }
                    }
                }
            }

            // Convert to code button
            ui.separator();
            if ui.button("🔄 Convert to WGSL Code").clicked() {
                self.convert_nodes_to_code();
            }
        }); // Close the frame for node editor
    }

    fn can_connect_pins(&self, from_pin: PinId, to_pin: PinId) -> bool {
        // Find pin types
        let from_type = self.get_pin_type(from_pin);
        let to_type = self.get_pin_type(to_pin);

        // Basic type compatibility check
        match (from_type, to_type) {
            (Some(PinType::Float), Some(PinType::Float)) => true,
            (Some(PinType::Vec2), Some(PinType::Vec2)) => true,
            (Some(PinType::Vec3), Some(PinType::Vec3)) => true,
            (Some(PinType::Vec4), Some(PinType::Vec4)) => true,
            (Some(PinType::Color), Some(PinType::Color)) => true,
            (Some(PinType::Texture), Some(PinType::Texture)) => true,
            _ => false,
        }
    }

    fn get_pin_type(&self, pin_id: PinId) -> Option<PinType> {
        for node in &self.nodes {
            for pin in &node.inputs {
                if pin.id == pin_id {
                    return Some(pin.pin_type.clone());
                }
            }
            for pin in &node.outputs {
                if pin.id == pin_id {
                    return Some(pin.pin_type.clone());
                }
            }
        }
        None
    }

    fn get_pin_position(&self, node: &Node, pin_id: PinId, is_output: bool) -> egui::Pos2 {
        if is_output {
            for (i, output) in node.outputs.iter().enumerate() {
                if output.id == pin_id {
                    return node.position + egui::vec2(node.size.x, 20.0 + i as f32 * 20.0);
                }
            }
        } else {
            for (i, input) in node.inputs.iter().enumerate() {
                if input.id == pin_id {
                    return node.position + egui::vec2(0.0, 20.0 + i as f32 * 20.0);
                }
            }
        }
        node.position // fallback
    }

    fn add_node(&mut self, node_type: NodeType, position: egui::Pos2) {
        let id = NodeId(self.nodes.len());
        let (title, inputs, outputs) = match node_type {
            // Basic I/O nodes
            NodeType::Input => (
                "Input".to_string(),
                vec![],
                vec![NodePin {
                    id: PinId(0),
                    name: "value".to_string(),
                    pin_type: PinType::Float,
                    position: egui::Pos2::ZERO,
                }]
            ),
            NodeType::Output => (
                "Output".to_string(),
                vec![NodePin {
                    id: PinId(0),
                    name: "color".to_string(),
                    pin_type: PinType::Color,
                    position: egui::Pos2::ZERO,
                }],
                vec![]
            ),
            NodeType::Uniform => (
                "Uniform".to_string(),
                vec![NodePin { id: PinId(0), name: "value".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "output".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::TextureInput => (
                "Texture Input".to_string(),
                vec![],
                vec![NodePin { id: PinId(0), name: "texture".to_string(), pin_type: PinType::Texture, position: egui::Pos2::ZERO }]
            ),

            // Math Operations
            NodeType::Math => (
                "Math".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "a".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "b".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "result".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Trigonometry => (
                "Trigonometry".to_string(),
                vec![NodePin { id: PinId(0), name: "angle".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "sin".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::VectorMath => (
                "Vector Math".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "v1".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "v2".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "result".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }]
            ),
            NodeType::MatrixMath => (
                "Matrix Math".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "matrix".to_string(), pin_type: PinType::Vec4, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "vector".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "output".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }]
            ),

            // Color Operations
            NodeType::Color => (
                "Color".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "r".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "g".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(2), name: "b".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(3), name: "color".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::ColorAdjustment => (
                "Color Adjustment".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "input".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "brightness".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "output".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::ColorMix => (
                "Color Mix".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "color1".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "color2".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(2), name: "mix_factor".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(3), name: "mixed".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::ColorSpace => (
                "Color Space".to_string(),
                vec![NodePin { id: PinId(0), name: "input".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "output".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),

            // Texture Operations
            NodeType::Texture => (
                "Texture".to_string(),
                vec![NodePin { id: PinId(0), name: "uv".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "color".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::TextureSample => (
                "Texture Sample".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "texture".to_string(), pin_type: PinType::Texture, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "uv".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "color".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::TextureTransform => (
                "Texture Transform".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "uv".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "offset".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(2), name: "scale".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(3), name: "transformed".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }]
            ),
            NodeType::TextureBlend => (
                "Texture Blend".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "texture1".to_string(), pin_type: PinType::Texture, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "texture2".to_string(), pin_type: PinType::Texture, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(2), name: "blend".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(3), name: "blended".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),

            // Geometry & 3D
            NodeType::Transform => (
                "Transform".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "position".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "scale".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "transformed".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }]
            ),
            NodeType::Geometry => (
                "Geometry".to_string(),
                vec![NodePin { id: PinId(0), name: "position".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "distance".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Volumetric => (
                "Volumetric".to_string(),
                vec![NodePin { id: PinId(0), name: "position".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "density".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::PointCloud => (
                "Point Cloud".to_string(),
                vec![NodePin { id: PinId(0), name: "position".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "intensity".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),

            // Advanced Rendering
            NodeType::Lighting => (
                "Lighting".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "normal".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "light".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "intensity".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Material => (
                "Material".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "albedo".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "roughness".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "material".to_string(), pin_type: PinType::Vec4, position: egui::Pos2::ZERO }]
            ),
            NodeType::BRDF => (
                "BRDF".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "normal".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "light".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(2), name: "view".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(3), name: "brdf".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::RayMarching => (
                "Ray Marching".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "origin".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "direction".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "distance".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),

            // Neural & AI
            NodeType::NeRF => (
                "NeRF".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "position".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "direction".to_string(), pin_type: PinType::Vec3, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "color".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::MLInference => (
                "ML Inference".to_string(),
                vec![NodePin { id: PinId(0), name: "input".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "output".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }]
            ),

            // Audio & Time
            NodeType::AudioReactive => (
                "Audio Reactive".to_string(),
                vec![NodePin { id: PinId(0), name: "audio".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "value".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Time => (
                "Time".to_string(),
                vec![],
                vec![NodePin { id: PinId(0), name: "time".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Oscillator => (
                "Oscillator".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "frequency".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "amplitude".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "output".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),

            // Post Processing
            NodeType::Filter => (
                "Filter".to_string(),
                vec![NodePin { id: PinId(0), name: "input".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "filtered".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::Blur => (
                "Blur".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "input".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "radius".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "blurred".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::Distortion => (
                "Distortion".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "input".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "amount".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "distorted".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::Effects => (
                "Effects".to_string(),
                vec![NodePin { id: PinId(0), name: "input".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "effect".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),

            // Utility
            NodeType::Constant => (
                "Constant".to_string(),
                vec![],
                vec![NodePin { id: PinId(0), name: "value".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Variable => (
                "Variable".to_string(),
                vec![NodePin { id: PinId(0), name: "input".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "output".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Switch => (
                "Switch".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "condition".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "true_value".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(2), name: "false_value".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(3), name: "output".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Loop => (
                "Loop".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "count".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "body".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "result".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
        };

        self.nodes.push(Node {
            id,
            position,
            size: egui::vec2(140.0, 100.0), // Slightly larger for more complex nodes
            node_type,
            title,
            inputs,
            outputs,
            value: 0.0,
        });
    }

    fn convert_nodes_to_code(&mut self) {
        // Basic topological sort for node graph traversal
        let mut visited = std::collections::HashSet::new();
        let mut order = Vec::new();

        fn visit_node(node_id: NodeId, nodes: &Vec<Node>, connections: &Vec<NodeConnection>, visited: &mut std::collections::HashSet<NodeId>, order: &mut Vec<NodeId>) {
            if visited.contains(&node_id) {
                return;
            }
            visited.insert(node_id);

            // Visit dependencies first (nodes that connect TO this node)
            for connection in connections {
                if connection.to_node == node_id {
                    visit_node(connection.from_node, nodes, connections, visited, order);
                }
            }

            order.push(node_id);
        }

        // Start from output nodes
        for node in &self.nodes {
            if matches!(node.node_type, NodeType::Output) {
                visit_node(node.id, &self.nodes, &self.connections, &mut visited, &mut order);
            }
        }

        // Generate WGSL code with consistent uniform block
        let mut wgsl_code = String::from("// Generated from node graph\n\n");
        wgsl_code.push_str("struct Uniforms {\n");
        wgsl_code.push_str("    time: f32,\n");
        wgsl_code.push_str("    resolution: vec2<f32>,\n");
        wgsl_code.push_str("    mouse: vec2<f32>,\n");
        wgsl_code.push_str("};\n\n");
        wgsl_code.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");

        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {\n");

        // Generate variable declarations in topological order
        for &node_id in &order {
            if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
                match node.node_type {
                    NodeType::Input => {
                        wgsl_code.push_str(&format!("    let input_{} = {};\n", node.id.0, node.value));
                    }
                    NodeType::Math => {
                        // Find input connections
                        let a_conn = self.connections.iter().find(|c| c.to_node == node.id && c.to_pin.0 == 0);
                        let b_conn = self.connections.iter().find(|c| c.to_node == node.id && c.to_pin.0 == 1);

                        let a_val = if let Some(conn) = a_conn {
                            match self.nodes.iter().find(|n| n.id == conn.from_node).map(|n| n.node_type.clone()) {
                                Some(NodeType::Input) => format!("input_{}", conn.from_node.0),
                                Some(NodeType::Math) => format!("math_{}", conn.from_node.0),
                                _ => "0.0".to_string(),
                            }
                        } else {
                            "0.0".to_string()
                        };

                        let b_val = if let Some(conn) = b_conn {
                            match self.nodes.iter().find(|n| n.id == conn.from_node).map(|n| n.node_type.clone()) {
                                Some(NodeType::Input) => format!("input_{}", conn.from_node.0),
                                Some(NodeType::Math) => format!("math_{}", conn.from_node.0),
                                _ => "0.0".to_string(),
                            }
                        } else {
                            "0.0".to_string()
                        };

                        wgsl_code.push_str(&format!("    let math_{} = {} + {};\n", node.id.0, a_val, b_val));
                    }
                    NodeType::Color => {
                        // Find input connections for r, g, b
                        let r_conn = self.connections.iter().find(|c| c.to_node == node.id && c.to_pin.0 == 0);
                        let g_conn = self.connections.iter().find(|c| c.to_node == node.id && c.to_pin.0 == 1);
                        let b_conn = self.connections.iter().find(|c| c.to_node == node.id && c.to_pin.0 == 2);

                        let r_val = if let Some(conn) = r_conn {
                            match self.nodes.iter().find(|n| n.id == conn.from_node).map(|n| n.node_type.clone()) {
                                Some(NodeType::Input) => format!("input_{}", conn.from_node.0),
                                Some(NodeType::Math) => format!("math_{}", conn.from_node.0),
                                _ => "1.0".to_string(),
                            }
                        } else {
                            "1.0".to_string()
                        };

                        let g_val = if let Some(conn) = g_conn {
                            match self.nodes.iter().find(|n| n.id == conn.from_node).map(|n| n.node_type.clone()) {
                                Some(NodeType::Input) => format!("input_{}", conn.from_node.0),
                                Some(NodeType::Math) => format!("math_{}", conn.from_node.0),
                                _ => "1.0".to_string(),
                            }
                        } else {
                            "1.0".to_string()
                        };

                        let b_val = if let Some(conn) = b_conn {
                            match self.nodes.iter().find(|n| n.id == conn.from_node).map(|n| n.node_type.clone()) {
                                Some(NodeType::Input) => format!("input_{}", conn.from_node.0),
                                Some(NodeType::Math) => format!("math_{}", conn.from_node.0),
                                _ => "1.0".to_string(),
                            }
                        } else {
                            "1.0".to_string()
                        };

                        wgsl_code.push_str(&format!("    let color_{} = vec4<f32>({}, {}, {}, 1.0);\n", node.id.0, r_val, g_val, b_val));
                    }
                    NodeType::Output => {
                        // Find color input connection
                        let color_conn = self.connections.iter().find(|c| c.to_node == node.id && c.to_pin.0 == 0);
                        let color_val = if let Some(conn) = color_conn {
                            match self.nodes.iter().find(|n| n.id == conn.from_node).map(|n| n.node_type.clone()) {
                                Some(NodeType::Color) => format!("color_{}", conn.from_node.0),
                                Some(NodeType::Math) => format!("vec4<f32>(math_{}, 0.0, 0.0, 1.0)", conn.from_node.0),
                                Some(NodeType::Input) => format!("vec4<f32>(input_{}, 0.0, 0.0, 1.0)", conn.from_node.0),
                                _ => "vec4<f32>(1.0, 1.0, 1.0, 1.0)".to_string(),
                            }
                        } else {
                            "vec4<f32>(1.0, 1.0, 1.0, 1.0)".to_string()
                        };

                        wgsl_code.push_str(&format!("    let output_{} = {};\n", node.id.0, color_val));
                    }
                    _ => {}
                }
            }
        }

        // Find the output node and return its value
        let output_value = if let Some(output_node) = self.nodes.iter().find(|n| matches!(n.node_type, NodeType::Output)) {
            format!("output_{}", output_node.id.0)
        } else {
            "vec4<f32>(1.0, 1.0, 1.0, 1.0)".to_string()
        };

        wgsl_code.push_str(&format!("    return {};\n", output_value));
        wgsl_code.push_str("}\n");

        self.current_wgsl_code = wgsl_code;
        self.compile_wgsl_shader();
    }

    fn get_wgsl_syntax_color(&self, word: &str) -> egui::Color32 {
        // WGSL keywords
        let keywords = [
            "fn", "let", "var", "struct", "const", "if", "else", "for", "while", "return",
            "true", "false", "vec2", "vec3", "vec4", "mat2", "mat3", "mat4", "f32", "i32", "u32",
            "texture_2d", "sampler", "textureSample", "group", "binding", "uniform", "storage",
            "vertex", "fragment", "compute", "workgroup_size", "builtin", "location",
            "abs", "sin", "cos", "tan", "pow", "sqrt", "clamp", "mix", "floor", "ceil", "fract"
        ];

        if keywords.contains(&word) {
            egui::Color32::from_rgb(86, 156, 214) // Blue for keywords
        } else if word.starts_with('@') {
            egui::Color32::from_rgb(155, 155, 0) // Yellow for attributes
        } else if word.chars().all(|c| c.is_numeric() || c == '.') {
            egui::Color32::from_rgb(181, 206, 168) // Green for numbers
        } else {
            egui::Color32::WHITE // Default color
        }
    }

    fn render_live_preview(&mut self, ui: &mut egui::Ui) {
        // Professional preview controls in a toolbar with enhanced styling - VS Code style
        egui::Frame::NONE
            .fill(egui::Color32::from_rgb(35, 35, 40))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📐 Resolution:").color(egui::Color32::LIGHT_BLUE));
                    let mut width = self.preview_size.0 as i32;
                    let mut height = self.preview_size.1 as i32;

                    ui.add(egui::DragValue::new(&mut width).prefix("W:").range(256..=4096).speed(10));
                    ui.add(egui::DragValue::new(&mut height).prefix("H:").range(256..=4096).speed(10));

                    // Update preview size
                    if width != self.preview_size.0 as i32 || height != self.preview_size.1 as i32 {
                        self.preview_size = (width as u32, height as u32);
                        if let Some(renderer) = &self.renderer {
                            if let Ok(mut renderer) = renderer.lock() {
                                let _ = renderer.resize(width as u32, height as u32);
                            }
                        }
                    }

                    ui.separator();

                    // Enhanced render controls with better visual feedback
                    if ui.button("▶️ Play").on_hover_text("Start rendering (Space)").clicked() || ui.input(|i| i.key_pressed(egui::Key::Space)) {
                        self.start_preview_rendering();
                        self.compile_wgsl_shader();
                    }
                    if ui.button("⏸️ Pause").on_hover_text("Pause rendering").clicked() {
                        self.pause_preview_rendering();
                    }
                    if ui.button("⏹️ Stop").on_hover_text("Stop rendering").clicked() {
                        self.stop_preview_rendering();
                    }
                    if ui.button("📸 Screenshot").on_hover_text("Take screenshot (F12)").clicked() || ui.input(|i| i.key_pressed(egui::Key::F12)) {
                        self.take_screenshot();
                    }

                    ui.separator();

                    // Auto-render toggle for continuous rendering
                    let auto_render_text = if self.auto_render { "🔄 Auto Render: ON" } else { "🔄 Auto Render: OFF" };
                    if ui.button(auto_render_text).on_hover_text("Toggle continuous rendering").clicked() {
                        self.auto_render = !self.auto_render;
                        if self.auto_render {
                            self.start_preview_rendering();
                            self.compile_wgsl_shader();
                        }
                    }

                    ui.separator();

                    // Template selection for real shader examples
                    ui.label(egui::RichText::new("🎨 Examples:").color(egui::Color32::LIGHT_BLUE));
                    egui::ComboBox::from_label("")
                        .selected_text("Select Shader")
                        .show_ui(ui, |ui| {
                            let mut selected_example: Option<String> = None;
                            if let Some(renderer) = &self.renderer {
                                if let Ok(renderer) = renderer.lock() {
                                    for example in renderer.get_working_examples() {
                                        if ui.selectable_label(false, &example.name).clicked() {
                                            selected_example = Some(example.wgsl_code.clone());
                                            break;
                                        }
                                    }
                                }
                            }

                            if let Some(wgsl_code) = selected_example {
                                self.current_wgsl_code = wgsl_code;
                                self.compile_wgsl_shader();
                            }
                        });

                    ui.separator();

                    // Real-time parameter controls
                    ui.label(egui::RichText::new("🎛️ Time:").color(egui::Color32::LIGHT_BLUE));
                    ui.add(egui::Slider::new(&mut self.time_slider, 0.0..=100.0).text(""));
                    
                    // Reset time button
                    if ui.button("↺ Reset").on_hover_text("Reset time parameter").clicked() {
                        self.time_slider = 0.0;
                    }
                });
            });

        ui.separator();

        // Render preview directly in the panel (no separate window to avoid duplication)
        let preview_size = egui::vec2(self.preview_size.0 as f32, self.preview_size.1 as f32);
        let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

        // Subtle background
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(4),
            egui::Color32::from_rgb(20, 20, 25),
        );

        println!("🔥 GUI render loop: Attempting to render shader...");
        self.render_actual_shader_preview(ui, rect, preview_size);
        println!("🔥 GUI render loop: Render attempt completed");

        // Update render timing for smooth FPS calculation
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_render_time).as_secs_f32();
        if delta_time > 0.0 {
            let instant_fps = 1.0 / delta_time;
            self.render_fps = self.render_fps * 0.9 + instant_fps * 0.1;
            self.last_render_time = now;
        }
    }

    fn render_actual_shader_preview(&mut self, ui: &mut egui::Ui, rect: egui::Rect, preview_size: egui::Vec2) {
        // Only render if we have a valid renderer and shader
        if self.renderer.is_none() {
            self.render_fallback_animation(ui, rect, preview_size);
            return;
        }

        // Only render every few frames to avoid hanging
        static mut LAST_RENDER_TIME: f32 = 0.0;
        let current_time = self.last_render_time.elapsed().as_secs_f32();
        unsafe {
            if current_time - LAST_RENDER_TIME < 0.016 { // ~60 FPS limit
                // Display cached texture if available
                if let Some(texture) = &self.preview_texture {
                    let image = egui::Image::new(texture)
                        .fit_to_exact_size(preview_size);
                    ui.add(image);
                } else {
                    ui.painter().rect_filled(rect, egui::CornerRadius::same(4), egui::Color32::from_rgb(30, 30, 40));
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Initializing preview...",
                        egui::FontId::proportional(14.0),
                        egui::Color32::from_rgb(180, 180, 200),
                    );
                }
                return;
            }
            LAST_RENDER_TIME = current_time;
        }

        // Get audio data if available
        let shader_audio_data = if let Some(audio_sys) = &self.audio_system {
            if let Ok(audio) = audio_sys.lock() {
                let local = audio.audio_analyzer.get_audio_data();
                Some(ShaderAudioData {
                    enabled: local.enabled,
                    bass_level: local.bass_level,
                    mid_level: local.mid_level,
                    treble_level: local.treble_level,
                    overall_level: local.overall_level,
                    beat_detected: local.beat_detected,
                    beat_intensity: local.beat_intensity,
                    volume: local.volume,
                })
            } else {
                None
            }
        } else {
            None
        };

        let params = RenderParameters {
            width: self.preview_size.0,
            height: self.preview_size.1,
            time: current_time,
            frame_rate: self.render_fps,
            audio_data: shader_audio_data.clone(),
        };

        let wgsl_code = self.current_wgsl_code.clone();

        // Try to render the user's current shader first
        println!("Attempting to render shader...");
        let render_result = if let Ok(mut renderer) = self.renderer.as_ref().unwrap().lock() {
            println!("✓ Renderer locked, calling render_frame...");
            let result = renderer.render_frame(&wgsl_code, &params, shader_audio_data.clone());
            println!("✓ render_frame completed with result: {:?}", result.is_ok());
            if let Err(ref e) = result {
                println!("✗ render_frame error: {}", e);
            }
            result
        } else {
            println!("✗ Failed to lock renderer");
            Err("Failed to lock renderer".into())
        };
        
        println!("Render result: {:?}", render_result.is_ok());

        match render_result {
            Ok(pixel_data) => {
                println!("✓ Shader rendered successfully, pixel data size: {}", pixel_data.len());
                // Successfully rendered shader - create egui texture from pixel data
                let texture_id = format!("shader_output_{}_{}", self.preview_size.0, self.preview_size.1);

                // Ensure pixel data is the correct size
                let expected_size = (self.preview_size.0 * self.preview_size.1 * 4) as usize;
                if pixel_data.len() == expected_size {
                    println!("✓ Pixel data size matches expected: {} bytes", expected_size);
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [self.preview_size.0 as usize, self.preview_size.1 as usize],
                        &pixel_data
                    );

                    let texture_handle = ui.ctx().load_texture(
                        texture_id,
                        color_image,
                        egui::TextureOptions::default()
                    );

                    // Store texture handle for reuse
                    self.preview_texture = Some(texture_handle.clone());

                    // Display the rendered texture
                    let image = egui::Image::new(&texture_handle)
                        .fit_to_exact_size(preview_size);
                    ui.add(image);
                } else {
                    // Pixel data size mismatch - show error
                    println!("✗ Pixel data size mismatch! Expected: {}, Got: {}", expected_size, pixel_data.len());
                    ui.painter().rect_filled(
                        rect,
                        egui::CornerRadius::same(4),
                        egui::Color32::from_rgb(200, 50, 50),
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("Pixel data size mismatch!\nExpected: {}, Got: {}", expected_size, pixel_data.len()),
                        egui::FontId::proportional(14.0),
                        egui::Color32::WHITE,
                    );
                }
            }
            Err(e) => {
                println!("✗ User shader rendering error: {}", e);
                // Show error message
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(4),
                    egui::Color32::from_rgb(150, 50, 50),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("Shader error: {}\nClick 'Render Now' to retry", e),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );
            }
        }

        if let Ok(renderer) = self.renderer.as_ref().unwrap().lock() {
            let errs = renderer.get_last_errors();
            if !errs.is_empty() {
                self.shader_errors = errs.to_vec();
            }
        }
    }

    fn render_shader_animation(&mut self, ui: &mut egui::Ui, rect: egui::Rect, preview_size: egui::Vec2, category: &str) {
        // High-quality shader animation based on category
        let time = self.last_render_time.elapsed().as_secs_f32();
        let pixel_size = 2.0; // Smaller pixels for higher quality

        for y in 0..(preview_size.y / pixel_size) as u32 {
            for x in 0..(preview_size.x / pixel_size) as u32 {
                let uv_x = x as f32 / (preview_size.x / pixel_size);
                let uv_y = y as f32 / (preview_size.y / pixel_size);

                let (r, g, b) = match category {
                    "Fractal" => {
                        // Mandelbrot-style fractal coloring
                        let zoom = 2.0;
                        let pan = egui::vec2(-0.5, 0.0);
                        let c = egui::vec2(
                            (uv_x - 0.5) * zoom + pan.x,
                            (uv_y - 0.5) * zoom + pan.y
                        );
                        
                        let mut z = egui::vec2(0.0, 0.0);
                        let mut iterations = 0.0;
                        let max_iter = 100.0;
                        
                        loop {
                            if z.length() > 2.0 || iterations >= max_iter {
                                break;
                            }
                            let new_z = egui::vec2(
                                z.x * z.x - z.y * z.y,
                                2.0 * z.x * z.y
                            ) + c;
                            z = new_z;
                            iterations += 1.0;
                        }
                        
                        let t = iterations / max_iter;
                        (
                            (t * 255.0) as u8,
                            ((1.0 - t) * 255.0 * 0.5) as u8,
                            ((1.0 - t) * 255.0 * 0.8) as u8
                        )
                    }
                    "Audio" => {
                        // Audio-reactive pattern
                        let wave = (uv_x * 10.0 + time * 2.0).sin() * 0.5 + 0.5;
                        let bass = (time * 1.3 + uv_x * 2.0 - uv_y).cos();
                        let mid = (time * 0.7 + (uv_x * uv_y) * 4.0).sin();
                        
                        (
                            ((wave + 0.5) * 127.0) as u8,
                            ((bass + 0.5) * 127.0) as u8,
                            ((mid + 0.5) * 127.0) as u8
                        )
                    }
                    "Effects" => {
                        // Kaleidoscope/plasma effect
                        let angle = (uv_y - 0.5).atan2(uv_x - 0.5);
                        let radius = (egui::vec2(uv_x - 0.5, uv_y - 0.5)).length();
                        let kaleido = angle + time * 0.5;
                        let pattern = (kaleido * 6.0 + radius * 10.0 + time).sin();
                        
                        (
                            ((0.5 + 0.5 * pattern) * 255.0) as u8,
                            ((0.5 + 0.5 * (pattern * 1.3 + 2.0944).sin()) * 255.0) as u8,
                            ((0.5 + 0.5 * (pattern * 1.7 + 4.18879).sin()) * 255.0) as u8
                        )
                    }
                    _ => {
                        // Default gradient animation
                        let r = (127.0 + 127.0 * (time + uv_x * 3.0 + uv_y * 2.0).sin()) as u8;
                        let g = (127.0 + 127.0 * (time * 1.3 + uv_x * 2.0 - uv_y).cos()) as u8;
                        let b = (127.0 + 127.0 * (time * 0.7 + (uv_x * uv_y) * 4.0).sin()) as u8;
                        (r, g, b)
                    }
                };

                let pixel_pos = rect.min + egui::vec2(x as f32 * pixel_size, y as f32 * pixel_size);
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(pixel_pos, egui::vec2(pixel_size, pixel_size)),
                    0.0,
                    egui::Color32::from_rgb(r, g, b),
                );
            }
        }
    }

    fn render_fallback_animation(&mut self, ui: &mut egui::Ui, rect: egui::Rect, preview_size: egui::Vec2) {
        // Simple fallback animation when shader rendering fails
        let time = self.last_render_time.elapsed().as_secs_f32();
        let pixel_size = 4.0;

        for y in 0..(preview_size.y / pixel_size) as u32 {
            for x in 0..(preview_size.x / pixel_size) as u32 {
                let uv_x = x as f32 / (preview_size.x / pixel_size);
                let uv_y = y as f32 / (preview_size.y / pixel_size);

                let r = (127.0 + 127.0 * (time + uv_x * 2.0).sin()) as u8;
                let g = (127.0 + 127.0 * (time + uv_y * 2.0).cos()) as u8;
                let b = (127.0 + 127.0 * (time * 0.5).sin()) as u8;

                let pixel_pos = rect.min + egui::vec2(x as f32 * pixel_size, y as f32 * pixel_size);
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(pixel_pos, egui::vec2(pixel_size, pixel_size)),
                    0.0,
                    egui::Color32::from_rgb(r, g, b),
                );
            }
        }
    }

    fn start_preview_rendering(&mut self) {
        // Start the preview rendering by triggering a re-render
        println!("🎬 Starting preview rendering...");
        self.last_render_time = std::time::Instant::now();
        self.render_fps = 60.0; // Default FPS
        println!("✓ Preview rendering initialized");
    }


    fn take_screenshot(&mut self) {
        // Placeholder for screenshot functionality
        println!("Screenshot functionality not yet implemented");
    }


    fn parse_wgsl_with_errors(&self, code: &str) -> (String, Vec<(usize, usize)>) {
        (code.to_string(), Vec::new())
    }

    fn get_completion_suggestions(&self, prefix: &str) -> Vec<String> {
        let wgsl_keywords = vec![
            "fn", "let", "var", "struct", "const", "if", "else", "for", "while", "return",
            "true", "false", "vec2", "vec3", "vec4", "mat2", "mat3", "mat4", "f32", "i32", "u32",
            "texture_2d", "sampler", "textureSample", "group", "binding", "uniform", "storage",
            "vertex", "fragment", "compute", "workgroup_size", "builtin", "location",
            "abs", "sin", "cos", "tan", "pow", "sqrt", "clamp", "mix", "floor", "ceil", "fract",
            "dot", "cross", "normalize", "length", "distance", "smoothstep", "step"
        ];

        let wgsl_attributes = vec![
            "@group", "@binding", "@builtin", "@location", "@vertex", "@fragment", "@compute"
        ];

        let mut suggestions: Vec<String> = Vec::new();

        // Add matching keywords
        for keyword in &wgsl_keywords {
            if keyword.starts_with(prefix) {
                suggestions.push(keyword.to_string());
            }
        }

        // Add matching attributes
        for attr in &wgsl_attributes {
            if attr.starts_with(prefix) {
                suggestions.push(attr.to_string());
            }
        }

        // Add variable names from current code (simplified)
        for line in self.current_wgsl_code.lines() {
            if line.contains("let ") || line.contains("var ") {
                if let Some(var_name) = line.split_whitespace().find(|s| s.chars().all(|c| c.is_alphanumeric() || c == '_')) {
                    if var_name.starts_with(prefix) && !suggestions.contains(&var_name.to_string()) {
                        suggestions.push(var_name.to_string());
                    }
                }
            }
        }

        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    fn get_global_position(&self, line_idx: usize, char_idx: usize) -> usize {
        let mut pos = 0;
        for (i, line) in self.current_wgsl_code.lines().enumerate() {
            if i < line_idx {
                pos += line.len() + 1; // +1 for newline
            } else if i == line_idx {
                pos += char_idx;
                break;
            }
        }
        pos
    }

    fn pause_preview_rendering(&mut self) {
        // Pause rendering (implementation depends on renderer)
        if let Some(renderer) = &mut self.renderer {
            // renderer.pause(); // Would need to implement this method
            println!("Preview rendering paused");
        }
    }

    fn stop_preview_rendering(&mut self) {
        // Stop rendering and cleanup
        if let Some(renderer) = &mut self.renderer {
            // renderer.stop(); // Would need to implement this method
            println!("Preview rendering stopped");
        }
    }

    fn perform_shader_conversion(&mut self) {
        self.shader_errors.clear();
        
        let (converted_code, success) = match (self.from_format.as_str(), self.to_format.as_str()) {
            ("WGSL", "GLSL") => self.convert_wgsl_to_glsl(&self.current_wgsl_code),
            ("WGSL", "HLSL") => self.convert_wgsl_to_hlsl(&self.current_wgsl_code),
            ("GLSL", "WGSL") => self.convert_glsl_to_wgsl(&self.current_wgsl_code),
            ("HLSL", "WGSL") => self.convert_hlsl_to_wgsl(&self.current_wgsl_code),
            ("GLSL", "HLSL") => self.convert_glsl_to_hlsl(&self.current_wgsl_code),
            ("HLSL", "GLSL") => self.convert_hlsl_to_glsl(&self.current_wgsl_code),
            ("WGSL", "ISF") => self.convert_wgsl_to_isf(&self.current_wgsl_code),
            ("ISF", "WGSL") => (self.convert_isf_to_wgsl(&self.current_wgsl_code), true),
            _ => ("Conversion between selected formats not supported".to_string(), false),
        };

        if success {
            self.current_wgsl_code = converted_code;
            self.shader_errors.push("✅ Conversion completed successfully!".to_string());
            self.shader_errors.push(format!("{} → {}", self.from_format, self.to_format));
            self.compile_wgsl_shader();
        } else {
            self.shader_errors.push("❌ Conversion failed: ".to_string() + &converted_code);
        }
    }

    fn convert_wgsl_to_glsl(&self, wgsl_code: &str) -> (String, bool) {
        match crate::shader_converter::wgsl_to_glsl(wgsl_code) {
            Ok(code) => (code, true),
            Err(e) => (format!("Conversion error: {}", e), false),
        }
    }

    fn convert_wgsl_to_hlsl(&self, wgsl_code: &str) -> (String, bool) {
        match crate::shader_converter::wgsl_to_hlsl(wgsl_code) {
            Ok(code) => (code, true),
            Err(e) => (format!("Conversion error: {}", e), false),
        }
    }

    fn convert_glsl_to_wgsl(&self, glsl_code: &str) -> (String, bool) {
        match crate::shader_converter::glsl_to_wgsl(glsl_code) {
            Ok(code) => (code, true),
            Err(e) => (format!("Conversion error: {}", e), false),
        }
    }

    fn convert_hlsl_to_wgsl(&self, hlsl_code: &str) -> (String, bool) {
        let mut wgsl_code = hlsl_code.to_string();

        // Replace main function
        wgsl_code = wgsl_code.replace("float4 main(", "@fragment\nfn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {");

        // Insert uniform block if missing
        if !wgsl_code.contains("var<uniform> uniforms: Uniforms") {
            let header = r#"struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

"#;
            wgsl_code.insert_str(0, header);
        }

        // Replace output
        wgsl_code = wgsl_code.replace("return float4(", "return vec4<f32>(");
        wgsl_code = wgsl_code.replace(") : SV_Target;", "}");
        wgsl_code = wgsl_code.replace(") : SV_Target0;", "}");

        // Type conversions
        wgsl_code = wgsl_code.replace("float2", "vec2<f32>");
        wgsl_code = wgsl_code.replace("float3", "vec3<f32>");
        wgsl_code = wgsl_code.replace("float4", "vec4<f32>");
        wgsl_code = wgsl_code.replace("float", "f32");

        (wgsl_code, true)
    }

    fn convert_glsl_to_hlsl(&self, glsl_code: &str) -> (String, bool) {
        let mut hlsl_code = glsl_code.to_string();

        // Replace main function
        hlsl_code = hlsl_code.replace("void main()", "float4 main(");
        hlsl_code = hlsl_code.replace("gl_FragColor = vec4(", "return float4(");
        hlsl_code = hlsl_code.replace("vec2", "float2");
        hlsl_code = hlsl_code.replace("vec3", "float3");
        hlsl_code = hlsl_code.replace("vec4", "float4");
        hlsl_code = hlsl_code.replace("float", "float");
        
        // Add semicolons
        hlsl_code = hlsl_code.replace("return float4(", "return float4(");
        hlsl_code.push_str(" : SV_Target;");
        
        (hlsl_code, true)
    }

    fn convert_hlsl_to_glsl(&self, hlsl_code: &str) -> (String, bool) {
        let mut glsl_code = hlsl_code.to_string();

        // Replace main function
        glsl_code = glsl_code.replace("float4 main(", "void main()");
        glsl_code = glsl_code.replace("return float4(", "gl_FragColor = vec4(");
        glsl_code = glsl_code.replace("float2", "vec2");
        glsl_code = glsl_code.replace("float3", "vec3");
        glsl_code = glsl_code.replace("float4", "vec4");
        glsl_code = glsl_code.replace(" : SV_Target;", "");
        glsl_code = glsl_code.replace(" : SV_Target0;", "");
        
        (glsl_code, true)
    }

    fn convert_wgsl_to_isf(&self, wgsl_code: &str) -> (String, bool) {
        let isf_code = "ISF conversion is complex and requires JSON structure. This is a simplified conversion. Use the GLSL output as a starting point for ISF development.".to_string();
        (isf_code, false)
    }

    fn convert_isf_to_wgsl(&self, isf_code: &str) -> String {
        let mut header = String::from("// Converted from ISF\n\n");
        header.push_str("struct Uniforms {\n");
        header.push_str("    time: f32,\n");
        header.push_str("    resolution: vec2<f32>,\n");
        header.push_str("    mouse: vec2<f32>,\n");
        header.push_str("};\n\n");
        header.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");

        let mut extra = String::new();
        if isf_code.contains("sampler2D") || isf_code.contains("texture2D") || isf_code.contains("IMG") {
            extra.push_str("@group(1) @binding(0) var input_texture: texture_2d<f32>;\n");
            extra.push_str("@group(1) @binding(1) var input_sampler: sampler;\n\n");
        }

        let mut body = isf_code.to_string();
        body = body.replace("void main()", "@fragment\nfn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32>");
        body = body.replace("gl_FragCoord", "position");
        body = body.replace("gl_FragColor", "return vec4<f32>");
        body = body.replace("vec2(", "vec2<f32>(");
        body = body.replace("vec3(", "vec3<f32>(");
        body = body.replace("vec4(", "vec4<f32>(");

        let mut wgsl = String::new();
        wgsl.push_str(&header);
        wgsl.push_str(&extra);
        wgsl.push_str(&body);
        if !wgsl.trim_end().ends_with("}") {
            wgsl.push_str("\n}\n");
        }
        wgsl
    }

    fn export_to_glsl(&mut self) {
        self.shader_errors.clear();
        let (glsl_code, success) = self.convert_wgsl_to_glsl(&self.current_wgsl_code);
        if success {
            self.shader_errors.push("✅ GLSL export completed successfully!".to_string());
            println!("Exported to GLSL:\n{}", glsl_code);
            // Show the converted code in console or copy to clipboard
        } else {
            self.shader_errors.push("❌ GLSL export failed".to_string());
        }
    }

    fn export_to_hlsl(&mut self) {
        self.shader_errors.clear();
        let (hlsl_code, success) = self.convert_wgsl_to_hlsl(&self.current_wgsl_code);
        if success {
            self.shader_errors.push("✅ HLSL export completed successfully!".to_string());
            println!("Exported to HLSL:\n{}", hlsl_code);
        } else {
            self.shader_errors.push("❌ HLSL export failed".to_string());
        }
    }

    fn import_isf_shader(&mut self) {
        // ISF shader import functionality
        self.shader_errors.clear();
        self.shader_errors.push("ISF import started...".to_string());

        let task = rfd::AsyncFileDialog::new()
            .add_filter("ISF Files", &["fs"])
            .add_filter("All Files", &["*"])
            .pick_file();

        pollster::block_on(async {
            if let Some(file) = task.await {
                match std::fs::read_to_string(file.path()) {
                    Ok(content) => {
                        // Basic ISF to WGSL conversion
                        let wgsl_code = self.convert_isf_to_wgsl(&content);
                        self.current_wgsl_code = wgsl_code;
                        self.current_file = Some(file.path().to_path_buf());
                        self.add_to_recent_files(file.path().to_path_buf());
                        self.compile_wgsl_shader();
                        self.shader_errors.push("✅ ISF shader imported successfully!".to_string());
                        println!("Imported ISF shader from: {:?}", file.path());
                    }
                    Err(e) => {
                        self.shader_errors.push(format!("❌ Failed to read ISF file: {}", e));
                    }
                }
            } else {
                self.shader_errors.push("ISF import cancelled".to_string());
            }
        });
    }

    fn reset_layout(&mut self) {
        // Reset all panel visibility to defaults
        // Code Editor and Live Preview are ALWAYS ON
        self.show_code_editor = true;
        self.show_preview = true;
        self.show_audio_panel = true;
        self.show_midi_panel = false;
        self.show_converter = false;
        self.show_file_browser = false;
        self.show_node_editor = false;
    }

    fn format_wgsl_code(&mut self) {
        // Basic WGSL code formatting - clean up whitespace and basic structure
        let lines: Vec<&str> = self.current_wgsl_code.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut indent_level: i32 = 0;
        let indent_size = 4;

        for &line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                formatted_lines.push("".to_string());
                continue;
            }

            // Handle closing braces - decrease indent
            if trimmed.starts_with('}') {
                indent_level = indent_level.saturating_sub(1);
            }

            // Add formatted line with proper indentation
            let indent = " ".repeat((indent_level * indent_size) as usize);
            formatted_lines.push(format!("{}{}", indent, trimmed));

            // Handle opening braces - increase indent
            if trimmed.ends_with('{') {
                indent_level += 1;
            }
        }

        self.current_wgsl_code = formatted_lines.join("\n");
        println!("WGSL code formatted successfully");
    }

    fn format_code(&mut self) {
        self.format_wgsl_code();
    }

    fn create_syntax_highlighting_layouter(&self) -> egui::text::LayoutJob {
        use egui::text::{LayoutJob, TextFormat};
        use egui::{Color32, FontId};
        
        let mut job = LayoutJob::default();
        let text = &self.current_wgsl_code;
        
        // Split text into tokens for syntax highlighting
        let mut pos = 0;
        for line in text.lines() {
            let line_start = pos;
            
            // Handle line comments
            if let Some(comment_pos) = line.find("//") {
                // Add code before comment
                let code_part = &line[..comment_pos];
                self.add_highlighted_tokens(&mut job, code_part, line_start);
                
                // Add comment part
                let comment_start = line_start + comment_pos;
                job.append(
                    &line[comment_pos..],
                    0.0,
                    TextFormat::simple(FontId::monospace(14.0), Color32::from_rgb(106, 153, 85)) // Green for comments
                );
            } else {
                // No comment in this line, highlight normally
                self.add_highlighted_tokens(&mut job, line, line_start);
            }
            
            // Add newline
            job.append("\n", 0.0, TextFormat::simple(FontId::monospace(14.0), Color32::WHITE));
            pos += line.len() + 1; // +1 for newline
        }
        
        job
    }
    
    fn add_highlighted_tokens(&self, job: &mut egui::text::LayoutJob, text: &str, start_offset: usize) {
        use egui::text::TextFormat;
        use egui::{Color32, FontId};
        
        let font = FontId::monospace(14.0);
        let mut pos = 0;
        
        // Simple tokenization - split by whitespace and common delimiters
        let mut chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let ch = chars[i];
            
            // Skip whitespace
            if ch.is_whitespace() {
                let start = i;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                let whitespace: String = chars[start..i].iter().collect();
                job.append(&whitespace, 0.0, TextFormat::simple(font.clone(), Color32::WHITE));
                continue;
            }
            
            // Handle identifiers and keywords
            if ch.is_alphabetic() || ch == '_' || ch == '@' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '@') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                
                // Get color based on word type
                let color = self.get_wgsl_syntax_color(&word);
                job.append(&word, 0.0, TextFormat::simple(font.clone(), color));
                continue;
            }
            
            // Handle numbers
            if ch.is_numeric() || ch == '.' {
                let start = i;
                while i < chars.len() && (chars[i].is_numeric() || chars[i] == '.' || chars[i] == 'e' || chars[i] == 'E' || chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                let number: String = chars[start..i].iter().collect();
                job.append(&number, 0.0, TextFormat::simple(font.clone(), Color32::from_rgb(181, 206, 168))); // Green for numbers
                continue;
            }
            
            // Handle strings
            if ch == '"' {
                let start = i;
                i += 1; // Skip opening quote
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    i += 1; // Skip closing quote
                }
                let string: String = chars[start..i].iter().collect();
                job.append(&string, 0.0, TextFormat::simple(font.clone(), Color32::from_rgb(206, 145, 120))); // Brown for strings
                continue;
            }
            
            // Handle single characters
            let single_char = ch.to_string();
            let color = match ch {
                '{' | '}' | '(' | ')' | '[' | ']' => Color32::from_rgb(120, 120, 120), // Gray for brackets
                ';' | ',' => Color32::from_rgb(120, 120, 120), // Gray for separators
                '+' | '-' | '*' | '/' | '=' | '!' | '<' | '>' => Color32::from_rgb(200, 120, 120), // Red for operators
                _ => Color32::WHITE,
            };
            job.append(&single_char, 0.0, TextFormat::simple(font.clone(), color));
            i += 1;
        }
    }

    fn render_shader_browser(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("ISF Shader Browser", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut selected_index = None;
                for (i, shader) in self.shaders.iter().enumerate() {
                    let selected = self.current_shader == Some(i);
                    let response = ui.selectable_label(selected, &shader.name);

                    if response.clicked() {
                        selected_index = Some(i);
                    }

                    // Show shader info on hover
                    response.on_hover_ui(|ui| {
                        ui.label(&shader.name);
                        if let Some(desc) = get_shader_metadata(shader).description {
                            ui.label(desc);
                        }
                        ui.label(format!("Inputs: {}", shader.inputs.len()));
                        ui.label(format!("Outputs: {}", shader.outputs.len()));
                    });
                }

                if let Some(index) = selected_index {
                    self.select_shader(index);
                }
            });
        });
    }

    fn render_templates(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Shader Templates", |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_template_category, "Basic".to_string(), "Basic");
                ui.selectable_value(&mut self.selected_template_category, "Animation".to_string(), "Animation");
                ui.selectable_value(&mut self.selected_template_category, "Fractal".to_string(), "Fractal");
                ui.selectable_value(&mut self.selected_template_category, "Effects".to_string(), "Effects");
                ui.selectable_value(&mut self.selected_template_category, "Tutorial".to_string(), "Tutorial");
                ui.selectable_value(&mut self.selected_template_category, "All".to_string(), "All");
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Collect template data first to avoid borrow conflicts
                let template_data: Vec<(String, String, String)> = {
                    let all_templates: Vec<_> = self.shader_templates.iter().chain(self.expanded_template_library.iter()).collect();
                    
                    let templates_to_show: Vec<_> = if self.selected_template_category == "All" {
                        all_templates
                    } else {
                        all_templates.into_iter().filter(|t| t.category == self.selected_template_category).collect()
                    };
                    
                    templates_to_show.iter().map(|template| {
                        (template.name.clone(), template.description.clone(), template.category.clone())
                    }).collect()
                };

                for (name, description, category) in &template_data {
                    ui.group(|ui| {
                        ui.set_width(260.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(name).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Load").clicked() {
                                    // Find the actual template by name and load it
                                    if let Some(template) = self.shader_templates.iter().find(|t| &t.name == name) {
                                        self.load_expanded_template(template.clone());
                                    } else if let Some(template) = self.expanded_template_library.iter().find(|t| &t.name == name) {
                                        self.load_expanded_template(template.clone());
                                    }
                                }
                            });
                        });

                        ui.label(description);
                        ui.small(format!("Category: {}", category));
                    });

                    ui.add_space(2.0);
                }

                if template_data.is_empty() {
                    ui.label("No templates in this category");
                }
            });
        });
    }

    fn render_converter(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Shader Converter", |ui| {
            ui.label("Convert between shader formats:");

            ui.horizontal(|ui| {
                ui.label("From:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.from_format)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.from_format, "WGSL".to_string(), "WGSL");
                        ui.selectable_value(&mut self.from_format, "GLSL".to_string(), "GLSL");
                        ui.selectable_value(&mut self.from_format, "HLSL".to_string(), "HLSL");
                        ui.selectable_value(&mut self.from_format, "ISF".to_string(), "ISF");
                        ui.selectable_value(&mut self.from_format, "SPIRV".to_string(), "SPIRV");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("To:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.to_format)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.to_format, "WGSL".to_string(), "WGSL");
                        ui.selectable_value(&mut self.to_format, "GLSL".to_string(), "GLSL");
                        ui.selectable_value(&mut self.to_format, "HLSL".to_string(), "HLSL");
                        ui.selectable_value(&mut self.to_format, "ISF".to_string(), "ISF");
                    });
            });

            if ui.button("Convert").clicked() {
                self.perform_shader_conversion();
            }

            // Conversion examples and help
            ui.separator();
            ui.label("💡 Conversion Examples:");
            egui::CollapsingHeader::new("WGSL → GLSL").default_open(false).show(ui, |ui| {
                ui.small("• @fragment → void main()");
                ui.small("• @builtin(position) → gl_FragCoord");
                ui.small("• @location(0) → gl_FragData[0]");
                ui.small("• vec2<f32> → vec2");
                ui.small("• f32 → float");
            });

            egui::CollapsingHeader::new("WGSL → HLSL").default_open(false).show(ui, |ui| {
                ui.small("• @group(0) @binding(n) → register(bn)");
                ui.small("• @location(0) → SV_Target0");
                ui.small("• @builtin(position) → SV_POSITION");
                ui.small("• vec2<f32> → float2");
                ui.small("• f32 → float");
            });

            // Show conversion status
            if !self.shader_errors.is_empty() {
                ui.separator();
                ui.label("Conversion Status:");
                for error in &self.shader_errors {
                    if error.contains("successfully") {
                        ui.colored_label(egui::Color32::GREEN, error);
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, error);
                    }
                }
            }
        });
    }

    fn render_file_browser(&mut self, ui: &mut egui::Ui) {
        let mut sources: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(root) = std::env::current_dir() {
            sources.push(root.join("assets").join("shaders"));
            sources.push(root.join("assets").join("isf"));
        }
        sources.push(std::path::PathBuf::from(r"C:\\Program Files\\Magic\\Modules2\\ISF\\fractal"));
        sources.push(std::path::PathBuf::from(r"C:\\Program Files\\Magic\\Modules2\\ISF\\fractal 2"));
        sources.push(std::path::PathBuf::from(r"C:\\Program Files\\Magic\\Modules2\\ISF\\final"));

        egui::ScrollArea::vertical().show(ui, |ui| {
            for dir in sources {
                if dir.exists() {
                    ui.collapsing(dir.display().to_string(), |ui| {
                        if let Ok(entries) = std::fs::read_dir(&dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if let Some(ext) = path.extension() {
                                    let ext_str = ext.to_string_lossy().to_string();
                                    if ext_str == "wgsl" || ext_str == "fs" {
                                        ui.horizontal(|ui| {
                                            ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                                            if ui.button("Load").clicked() {
                                                if let Ok(content) = std::fs::read_to_string(&path) {
                                                    if ext_str == "wgsl" {
                                                        self.current_wgsl_code = content;
                                                    } else {
                                                        let conv = self.convert_isf_to_wgsl(&content);
                                                        self.current_wgsl_code = conv;
                                                    }
                                                    self.current_file = Some(path.clone());
                                                    self.add_to_recent_files(path.clone());
                                                    self.compile_wgsl_shader();
                                                    self.start_preview_rendering();
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    });
                }
            }
        });
    }

    fn render_parameter_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(shader_index) = self.current_shader {
            let shader = &self.shaders[shader_index];

            // Add uniform layout analysis section
            ui.collapsing("Uniform Layout", |ui| {
                // TODO: Re-enable when WgslBindgenAnalyzer is properly integrated
                // if let Some(ref mut analyzer) = self.bindgen_analyzer {
                //     if let Ok(layouts) = analyzer.analyze_shader(&self.current_wgsl_code, &shader.name) {
                //         ui.label(format!("Found {} uniform buffers", layouts.len()));
                //         for layout in &layouts {
                //             ui.horizontal(|ui| {
                //                 ui.label(format!("@group({}) @binding({})", layout.group, layout.binding));
                //                 ui.label(&layout.name);
                //             });
                //             ui.label(format!("Size: {} bytes", layout.size));
                //             for field in &layout.fields {
                //                 ui.horizontal(|ui| {
                //                     ui.label(&format!("  {}: {}", field.name, field.ty));
                //                     ui.label(format!("@{} bytes", field.offset));
                //                 });
                //             }
                //     }
                // }
                ui.label("Uniform layout analysis temporarily disabled");
            });
            
            ui.add_space(10.0);

            ui.collapsing("Parameters", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for input in &shader.inputs {
                        let current_value = self.parameter_values.get(&input.name).copied().unwrap_or(0.0);

                        ui.horizontal(|ui| {
                            ui.label(&input.name);

                            let mut new_value = current_value;
                            let range = match (input.min, input.max) {
                                (Some(min), Some(max)) => min..=max,
                                _ => 0.0..=1.0,
                            };

                            match input.input_type {
                                InputType::Float => {
                                    ui.add(egui::Slider::new(&mut new_value, range).text(""));
                                }
                                InputType::Bool => {
                                    let mut bool_value = new_value > 0.0;
                                    if ui.checkbox(&mut bool_value, "").changed() {
                                        new_value = if bool_value { 1.0 } else { 0.0 };
                                    }
                                }
                                InputType::Color => {
                                    // Use proper color picker for color inputs
                                    let mut color = egui::Color32::from_rgb((new_value * 255.0) as u8, (new_value * 255.0) as u8, (new_value * 255.0) as u8);
                                    if egui::color_picker::color_edit_button_srgba(ui, &mut color, egui::color_picker::Alpha::Opaque).changed() {
                                        let [r, g, b, _] = color.to_srgba_unmultiplied();
                                        new_value = ((r as f32 + g as f32 + b as f32) / 3.0) / 255.0; // Use luminance as representative value
                                    }
                                }
                                _ => {
                                    ui.add(egui::Slider::new(&mut new_value, range).text(""));
                                }
                            }

                            if new_value != current_value {
                                self.parameter_values.insert(input.name.clone(), new_value);
                            }
                        });
                    }
                });
            });

            ui.collapsing("Errors", |ui| {
                // Renderer validation errors
                if let Some(renderer) = &self.renderer {
                    if let Ok(r) = renderer.lock() {
                        let errs = r.get_last_errors();
                        if !errs.is_empty() {
                            ui.label("Renderer validation errors:");
                            for e in errs {
                                ui.label(egui::RichText::new(e).color(egui::Color32::LIGHT_RED));
                            }
                            ui.separator();
                        }
                    }
                }
                // Converter/compile errors
                if !self.shader_errors.is_empty() {
                    ui.label("Conversion/compile messages:");
                    for e in &self.shader_errors {
                        ui.label(egui::RichText::new(e).color(egui::Color32::LIGHT_RED));
                    }
                    if ui.button("Clear").clicked() { self.shader_errors.clear(); }
                } else {
                    ui.label("No errors");
                }
            });
        }
    }
    fn render_audio_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Audio Analysis", |ui| {
            if let Some(audio_sys) = &self.audio_system {
                if let Ok(audio_sys) = audio_sys.lock() {
                    let data = audio_sys.audio_analyzer.get_audio_data();

                    ui.horizontal(|ui| {
                        ui.label("Enabled:");
                        ui.label(if data.enabled { "Yes" } else { "No" });
                    });
                    ui.separator();

                    ui.label("Levels:");
                    ui.add(egui::ProgressBar::new(data.bass_level.clamp(0.0, 1.0)).text("Bass"));
                    ui.add(egui::ProgressBar::new(data.mid_level.clamp(0.0, 1.0)).text("Mid"));
                    ui.add(egui::ProgressBar::new(data.treble_level.clamp(0.0, 1.0)).text("Treble"));
                    ui.add(egui::ProgressBar::new(data.overall_level.clamp(0.0, 1.0)).text("Overall"));

                    ui.separator();
                    ui.label(format!("Volume: {:.3}", data.volume));
                    ui.label(format!("Beat detected: {} (intensity {:.2})", data.beat_detected, data.beat_intensity));
                } else {
                    ui.label("Audio system not initialized");
                }
            } else {
                ui.label("Audio system not available");
            }
        });

        // Always show Initialize Audio button if system not available
        if self.audio_system.is_none() {
            if ui.button("Initialize Audio").clicked() {
                // Initialize audio system
                self.audio_system = Some(Arc::new(std::sync::Mutex::new(crate::audio::AudioMidiSystem::new())));
            }
        }
    }


    fn render_midi_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("MIDI Control", |ui| {
            ui.label("MIDI system not available");
        });
    }

    fn render_performance_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Performance", |ui| {
            ui.label(format!("UI FPS: {:.1}", self.fps_counter));

            // FPS counter is now updated centrally in the update method
            // No need to duplicate the logic here
        });
    }

    fn render_gesture_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Gesture Control", |ui| {
            ui.label("Gesture control system status:");
            ui.label("• Leap Motion: Not initialized");
            ui.label("• MediaPipe: Not initialized");
            ui.label("• Real-time parameter mapping: Ready");
            ui.label("• Multi-touch gesture support: Ready");

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Initialize Leap Motion").clicked() {
                    self.initialize_leap_motion();
                }
                if ui.button("Initialize MediaPipe").clicked() {
                    self.initialize_mediapipe();
                }
            });

            ui.separator();

            ui.label("Gesture mappings:");
            ui.label("• Hand position → Mouse position");
            ui.label("• Finger gestures → Shader parameters");
            ui.label("• Face tracking → Camera controls");
        });
    }

    fn initialize_leap_motion(&mut self) {
        // Initialize Leap Motion gesture control
        self.shader_errors.clear();
        self.shader_errors.push("Leap Motion initialization started...".to_string());

        // TODO: Implement actual Leap Motion initialization
        // For now, just show a message
        self.shader_errors.push("Leap Motion SDK not available in this build".to_string());
        self.shader_errors.push("Install Leap Motion SDK and rebuild to enable".to_string());
    }

    fn initialize_mediapipe(&mut self) {
        // Initialize MediaPipe gesture recognition
        self.shader_errors.clear();
        self.shader_errors.push("MediaPipe initialization started...".to_string());

        // TODO: Implement actual MediaPipe initialization
        // For now, just show a message
        self.shader_errors.push("MediaPipe not available in this build".to_string());
        self.shader_errors.push("Install MediaPipe and rebuild to enable".to_string());
    }

    fn show_info_dialogs(&mut self, ctx: &egui::Context) {
        // About dialog
        if self.show_about_dialog {
            let mut open = true;
            egui::Window::new("About WGSL Shader Studio")
                .open(&mut open)
                .resizable(false)
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.heading("🎨 WGSL Shader Studio");
                        ui.add_space(5.0);
                        ui.label("Version 1.0.0");
                        ui.add_space(15.0);

                        ui.label("A professional WebGPU shader development environment");
                        ui.add_space(10.0);

                        ui.separator();
                        ui.add_space(10.0);

                        ui.label("🚀 Features:");
                        ui.add_space(5.0);
                        ui.label("• Real-time WGSL shader compilation and rendering");
                        ui.label("• Live preview with GPU acceleration");
                        ui.label("• Node-based visual shader editor");
                        ui.label("• Audio-reactive shader support");
                        ui.label("• MIDI parameter control");
                        ui.label("• Gesture control integration");
                        ui.label("• Shader format conversion (WGSL ↔ GLSL ↔ HLSL)");
                        ui.label("• ISF shader compatibility");
                        ui.label("• Professional UI with multiple themes");

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label("💻 Built with:");
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("• Rust");
                            ui.label("• WebGPU (wgpu)");
                            ui.label("• egui");
                        });

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label("📄 License: MIT");
                        ui.add_space(5.0);
                        ui.label("© 2024 WGSL Shader Studio Team");

                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Close").clicked() {
                                    self.show_about_dialog = false;
                                }
                            });
                        });
                    });
                });
            if !open {
                self.show_about_dialog = false;
            }
        }

        // Documentation dialog
        if self.show_documentation_dialog {
            let mut open = true;
            egui::Window::new("Documentation")
                .open(&mut open)
                .default_size([600.0, 400.0])
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("📚 WGSL Shader Studio Documentation");

                        ui.add_space(10.0);
                        ui.label("Welcome to WGSL Shader Studio! This guide will help you get started with creating stunning GPU-accelerated shaders.");

                        ui.add_space(20.0);
                        ui.heading("🎯 Getting Started");
                        ui.add_space(5.0);
                        ui.label("1. **Code Editor**: Write your WGSL shaders in the code editor panel");
                        ui.label("2. **Live Preview**: See your shaders render in real-time in the preview panel");
                        ui.label("3. **Templates**: Use pre-built shader templates to get started quickly");
                        ui.label("4. **Node Editor**: Create shaders visually using the node-based editor");

                        ui.add_space(20.0);
                        ui.heading("🔧 WGSL Basics");
                        ui.add_space(5.0);
                        ui.label("WGSL (WebGPU Shading Language) is the shader language for WebGPU. Key concepts:");
                        ui.add_space(5.0);
                        ui.label("• **@vertex**: Defines vertex shader entry point");
                        ui.label("• **@fragment**: Defines fragment shader entry point");
                        ui.label("• **@group/@binding**: Specifies uniform buffer bindings");
                        ui.label("• **@builtin**: Accesses built-in variables like position");
                        ui.label("• **var<uniform>**: Declares uniform variables");

                        ui.add_space(20.0);
                        ui.heading("🎨 Built-in Uniforms");
                        ui.add_space(5.0);
                        ui.label("The following uniforms are automatically provided:");
                        ui.add_space(5.0);
                        ui.code("time: f32          // Current time in seconds");
                        ui.code("resolution: vec2<f32>  // Viewport resolution");
                        ui.code("mouse: vec2<f32>   // Mouse position (0-1)");
                        ui.code("audio_volume: f32  // Current audio volume");
                        ui.code("audio_bass: f32    // Bass frequency level");
                        ui.code("audio_mid: f32     // Mid frequency level");
                        ui.code("audio_treble: f32  // Treble frequency level");

                        ui.add_space(20.0);
                        ui.heading("🎵 Audio Integration");
                        ui.add_space(5.0);
                        ui.label("WGSL Shader Studio supports audio-reactive shaders:");
                        ui.add_space(5.0);
                        ui.label("• Enable audio analysis in the Audio panel");
                        ui.label("• Use audio uniforms in your shaders");
                        ui.label("• Audio data is analyzed in real-time");

                        ui.add_space(20.0);
                        ui.heading("🎛️ MIDI Control");
                        ui.add_space(5.0);
                        ui.label("Control shader parameters with MIDI:");
                        ui.add_space(5.0);
                        ui.label("• Map MIDI CC messages to shader parameters");
                        ui.label("• Real-time parameter modulation");
                        ui.label("• Save/load MIDI mappings");

                        ui.add_space(20.0);
                        ui.heading("🔄 Shader Conversion");
                        ui.add_space(5.0);
                        ui.label("Convert between shader formats:");
                        ui.add_space(5.0);
                        ui.label("• WGSL ↔ GLSL");
                        ui.label("• WGSL ↔ HLSL");
                        ui.label("• WGSL ↔ ISF");
                        ui.label("• Automatic syntax conversion");

                        ui.add_space(20.0);
                        ui.heading("⌨️ Keyboard Shortcuts");
                        ui.add_space(5.0);
                        ui.label("• **Ctrl+N**: New shader");
                        ui.label("• **Ctrl+O**: Open shader");
                        ui.label("• **Ctrl+S**: Save shader");
                        ui.label("• **Ctrl+Shift+S**: Save shader as");
                        ui.label("• **F5**: Compile shader");
                        ui.label("• **F11**: Toggle fullscreen");

                        ui.add_space(30.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Close").clicked() {
                                    self.show_documentation_dialog = false;
                                }
                            });
                        });
                    });
                });
            if !open {
                self.show_documentation_dialog = false;
            }
        }

        // Keyboard shortcuts dialog
        if self.show_shortcuts_dialog {
            let mut open = true;
            egui::Window::new("Keyboard Shortcuts")
                .open(&mut open)
                .resizable(false)
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.heading("⌨️ Keyboard Shortcuts");
                        ui.add_space(15.0);

                        ui.label("📝 File Operations:");
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("• New Shader:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("Ctrl+N");
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label("• Open Shader:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("Ctrl+O");
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label("• Save Shader:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("Ctrl+S");
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label("• Save As:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("Ctrl+Shift+S");
                            });
                        });

                        ui.add_space(15.0);
                        ui.label("🔧 Development:");
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("• Compile Shader:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("F5");
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label("• Format Code:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("Ctrl+Shift+F");
                            });
                        });

                        ui.add_space(15.0);
                        ui.label("🎬 Preview:");
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("• Play/Pause:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("Space");
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label("• Screenshot:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("F12");
                            });
                        });

                        ui.add_space(15.0);
                        ui.label("🎨 Interface:");
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("• Toggle Fullscreen:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("F11");
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label("• Reset Layout:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.code("Ctrl+R");
                            });
                        });

                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Close").clicked() {
                                    self.show_shortcuts_dialog = false;
                                }
                            });
                        });
                    });
                });
            if !open {
                self.show_shortcuts_dialog = false;
            }
        }

        // Theme editor dialog
        if self.show_theme_editor {
            let mut open = true;
            egui::Window::new("🎨 Custom Theme Editor")
                .open(&mut open)
                .default_size([400.0, 500.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.heading("Custom Theme Editor");
                        ui.add_space(10.0);

                        ui.label("Customize your own color scheme:");

                        ui.add_space(15.0);
                        ui.label("🎨 Primary Colors:");

                        // Background color picker
                        ui.horizontal(|ui| {
                            ui.label("Background:");
                            let mut bg_color = self.custom_theme_colors.get("background").copied().unwrap_or(egui::Color32::from_rgb(35, 35, 38));
                            if egui::color_picker::color_edit_button_srgba(ui, &mut bg_color, egui::color_picker::Alpha::Opaque).changed() {
                                self.custom_theme_colors.insert("background".to_string(), bg_color);
                            }
                        });

                        // Window color picker
                        ui.horizontal(|ui| {
                            ui.label("Window:");
                            let mut window_color = self.custom_theme_colors.get("window").copied().unwrap_or(egui::Color32::from_rgb(28, 28, 31));
                            if egui::color_picker::color_edit_button_srgba(ui, &mut window_color, egui::color_picker::Alpha::Opaque).changed() {
                                self.custom_theme_colors.insert("window".to_string(), window_color);
                            }
                        });

                        // Text color picker
                        ui.horizontal(|ui| {
                            ui.label("Text:");
                            let mut text_color = self.custom_theme_colors.get("text").copied().unwrap_or(egui::Color32::from_rgb(230, 230, 230));
                            if egui::color_picker::color_edit_button_srgba(ui, &mut text_color, egui::color_picker::Alpha::Opaque).changed() {
                                self.custom_theme_colors.insert("text".to_string(), text_color);
                            }
                        });

                        ui.add_space(15.0);
                        ui.label("🎯 Accent Colors:");

                        // Selection color picker
                        ui.horizontal(|ui| {
                            ui.label("Selection:");
                            let mut selection_color = self.custom_theme_colors.get("selection").copied().unwrap_or(egui::Color32::from_rgb(80, 140, 220));
                            if egui::color_picker::color_edit_button_srgba(ui, &mut selection_color, egui::color_picker::Alpha::Opaque).changed() {
                                self.custom_theme_colors.insert("selection".to_string(), selection_color);
                            }
                        });

                        // Hyperlink color picker
                        ui.horizontal(|ui| {
                            ui.label("Links:");
                            let mut link_color = self.custom_theme_colors.get("link").copied().unwrap_or(egui::Color32::from_rgb(120, 180, 255));
                            if egui::color_picker::color_edit_button_srgba(ui, &mut link_color, egui::color_picker::Alpha::Opaque).changed() {
                                self.custom_theme_colors.insert("link".to_string(), link_color);
                            }
                        });

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("🎨 Apply Custom Theme").clicked() {
                                self.apply_theme("custom", ctx);
                                self.save_theme_settings();
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Reset").clicked() {
                                    self.custom_theme_colors.clear();
                                    self.save_theme_settings();
                                }
                                if ui.button("Close").clicked() {
                                    self.show_theme_editor = false;
                                }
                            });
                        });
                    });
                });
            if !open {
                self.show_theme_editor = false;
            }
        }
    }

    // Note: save_theme_settings and load_theme_settings methods already exist in the impl block

}


#[cfg(feature = "gui")]
impl eframe::App for ShaderGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Initialize heavy components on first update to avoid blocking window creation
        if !self.initialized && !self.initialization_started {
            self.initialization_started = true;
            println!("Initializing heavy components...");
            
            // Load ISF shaders
            match load_resolume_isf_shaders() {
                Ok(shaders) => {
                    self.shaders = shaders;
                    println!("✓ Loaded {} ISF shaders", self.shaders.len());
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to load ISF shaders: {}", e);
                    self.shaders = Vec::new();
                }
            }
            
            // Load recent files
            self.load_recent_files();
            
            // Load theme settings
            self.load_theme_settings();
            
            // Initialize audio system
            let audio_system = AudioMidiSystem::new();
            self.audio_system = Some(Arc::new(std::sync::Mutex::new(audio_system)));
            println!("✓ Audio/MIDI system initialized successfully");
            
            // Initialize gesture control system
            self.gesture_control = Some(Arc::new(std::sync::Mutex::new(GestureControlSystem::new())));
            
            self.initialized = true;
            println!("✓ Initialization completed");
        }
        
        // Request window focus to ensure visibility
        if self.initialized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        
        // Initialize renderer after window is created but only once
        if self.initialized && self.renderer.is_none() {
            println!("Initializing shader renderer in GUI...");
            let renderer_result = pollster::block_on(ShaderRenderer::new());
            match renderer_result {
                Ok(renderer) => {
                    println!("✓ Shader renderer initialized successfully");
                    self.renderer = Some(std::sync::Arc::new(std::sync::Mutex::new(renderer)));
                    
                    // Auto-compile and start rendering the default shader
                    println!("🚀 Auto-starting shader compilation and rendering...");
                    self.compile_wgsl_shader();
                    self.start_preview_rendering();
                },
                Err(e) => {
                    eprintln!("Failed to initialize WGPU renderer: {}", e);
                    // Try fallback renderer
                    println!("Attempting fallback renderer creation...");
                    match pollster::block_on(ShaderRenderer::new()) {
                        Ok(renderer) => {
                            println!("✓ Fallback shader renderer initialized successfully");
                            self.renderer = Some(std::sync::Arc::new(std::sync::Mutex::new(renderer)));
                            
                            // Auto-compile and start rendering the default shader
                            println!("🚀 Auto-starting shader compilation and rendering...");
                            self.compile_wgsl_shader();
                            self.start_preview_rendering();
                        },
                        Err(e) => {
                            eprintln!("Failed to create fallback renderer: {}", e);
                            // If we can't create a renderer, we'll continue without it
                            // This will cause rendering to fail but the GUI should still work
                            eprintln!("⚠️  Continuing without renderer - preview will not work");
                        }
                    }
                }
            }
        } else if self.initialized && self.renderer.is_some() {
            println!("Renderer already initialized");
        }
        
        // Show dialogs/windows that overlay the UI
        self.show_info_dialogs(ctx);

        // Menu bar
        self.render_menu_bar(ctx);

        // Render preview in a controlled panel to avoid duplication
        if self.show_preview {
            egui::TopBottomPanel::top("preview_panel")
                .resizable(true)
                .default_height(300.0)
                .min_height(200.0)
                .max_height(600.0)
                .show(ctx, |ui| {
                    self.render_live_preview(ui);
                });
        }

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.1}", self.fps_counter));

                ui.label(match &self.compilation_status {
                    ShaderCompilationStatus::NotCompiled => "Not compiled",
                    ShaderCompilationStatus::Compiling => "Compiling...",
                    ShaderCompilationStatus::Success => "✓ Shader compiled successfully",
                    ShaderCompilationStatus::Error(_) => "✗ Compilation error",
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(file_path) = &self.current_file {
                        ui.label(format!("File: {}", file_path.display()));
                    } else {
                        ui.label("Unsaved shader");
                    }
                });
            });
        });

        // Main content - Professional shader editor layout
        let left_panel_width = 250.0;
        let right_panel_width = 250.0;
        let bottom_panel_height = 250.0;

        // Left Panel - Shader templates and browser
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(left_panel_width)
            .width_range(200.0..=350.0)
            .show(ctx, |ui| {
                ui.set_min_height(400.0);
                ui.vertical(|ui| {
                    ui.heading("Templates");
                    ui.separator();
                    self.render_templates(ui);
                });
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.heading("Shaders");
                    ui.separator();
                    self.render_shader_browser(ui);
                });
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.heading("Converter");
                    ui.separator();
                    self.render_converter(ui);
                });
                if self.show_file_browser {
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.heading("Files");
                        ui.separator();
                        self.render_file_browser(ui);
                    });
                }
            });

        // Right Panel - Parameters and controls
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(right_panel_width)
            .width_range(200.0..=400.0)
            .show(ctx, |ui| {
                ui.set_min_height(400.0);
                ui.vertical(|ui| {
                    ui.heading("Parameters");
                    ui.separator();
                    self.render_parameter_panel(ui);
                });
                ui.add_space(10.0);
                if self.show_audio_panel {
                    ui.vertical(|ui| {
                        ui.heading("Audio");
                        ui.separator();
                        self.render_audio_panel(ui);
                    });
                }
                ui.add_space(10.0);
                if self.show_midi_panel {
                    ui.vertical(|ui| {
                        ui.heading("MIDI");
                        ui.separator();
                        self.render_midi_panel(ui);
                    });
                }
                ui.add_space(10.0);
                if self.show_gesture_panel {
                    ui.vertical(|ui| {
                        ui.heading("Gestures");
                        ui.separator();
                        self.render_gesture_panel(ui);
                    });
                }
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.heading("Performance");
                    ui.separator();
                    self.render_performance_panel(ui);
                });
            });

        // Bottom Panel - Code editor
        egui::TopBottomPanel::bottom("editor_bottom")
            .resizable(true)
            .default_height(bottom_panel_height)
            .height_range(150.0..=500.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Code Editor");
                    ui.separator();
                    self.render_code_editor(ui);
                });
            });

        // Timeline Panel
        egui::TopBottomPanel::bottom("timeline_bottom")
            .resizable(false)
            .default_height(64.0)
            .height_range(48.0..=96.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let mut playing = self.auto_render;
                    if ui.toggle_value(&mut playing, "Play").changed() {
                        self.auto_render = playing;
                    }
                    ui.label("Time");
                    ui.add(egui::Slider::new(&mut self.time_slider, 0.0..=600.0));
                    ui.label("Speed");
                    let mut speed = self.render_fps;
                    ui.add(egui::Slider::new(&mut speed, 1.0..=120.0));
                    self.render_fps = speed;
                });
            });

        // Central Panel - Main viewport (no direct preview draw to avoid duplication)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_min_height(400.0);
        });

        // Update FPS counter centrally with moving average smoothing
        self.frame_count += 1;
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_fps_update).as_secs_f32();
        if delta_time >= 1.0 {
            let instant_fps = self.frame_count as f32 / delta_time;
            self.fps_counter = self.fps_counter * 0.9 + instant_fps * 0.1; // Exponential moving average
            self.frame_count = 0;
            self.last_fps_update = now;
        }
        
        // Request repaint with conditional logic to avoid hanging
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

}

#[cfg(feature = "gui")]
pub fn run_gui() {
    println!("Starting WGSL Shader Studio GUI...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0]) // Larger window size
            .with_position([100.0, 100.0]) // Position closer to top-left corner
            .with_title("WGSL Shader Studio")
            .with_visible(true)
            .with_active(true)
            .with_decorations(true)
            .with_transparent(false) // Ensure window is not transparent
            .with_resizable(true), // Make window resizable
        vsync: true,
        // Disable persistence to prevent window position issues
        persist_window: false,
        ..Default::default()
    };

    match eframe::run_native(
        "WGSL Shader Studio",
        options,
        Box::new(|cc| {
            println!("Creating GUI application...");
            let app = ShaderGui::new(cc);
            println!("GUI application created successfully");
            Ok(app)
        }),
    ) {
        Ok(_) => {
            println!("GUI closed successfully");
        }
        Err(e) => {
            eprintln!("Failed to start GUI: {}", e);
            eprintln!("This could be due to missing graphics drivers, incompatible GPU, or display issues.");
            eprintln!("Troubleshooting steps:");
            eprintln!("  1. Ensure your graphics drivers are up to date");
            eprintln!("  2. Check if your GPU supports Vulkan or DirectX 12");
            eprintln!("  3. Try running with integrated graphics if available");
            eprintln!("  4. Run with --cli flag for command-line mode");
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "gui"))]
pub fn run_gui() {
    println!("GUI feature not enabled. Use --features gui to enable the graphical interface.");
}
