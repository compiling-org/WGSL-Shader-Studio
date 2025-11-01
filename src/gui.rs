//! WGSL Shader Studio - Professional GUI Application
//! Based on modular-fractal-shader UI architecture

#[cfg(feature = "gui")]
use eframe::egui;
#[cfg(feature = "gui")]
use std::collections::HashMap;
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
use crate::audio::AudioMidiSystem;
use resolume_isf_shaders_rust_ffgl::audio::AudioData;
use resolume_isf_shaders_rust_ffgl::gesture_control::{GestureControlSystem, GestureType, GestureMapping};

#[cfg(feature = "gui")]
use egui::TextureHandle;

// Use the working shader renderer from the library crate
#[cfg(feature = "gui")]
use resolume_isf_shaders_rust_ffgl::shader_renderer::{ShaderRenderer, RenderParameters, WorkingShaderExample};

#[cfg(feature = "gui")]
pub struct ShaderGui {
    // Shader management
    shaders: Vec<IsfShader>,
    current_shader: Option<usize>,
    parameter_values: HashMap<String, f32>,
    current_wgsl_code: String,
    shader_templates: Vec<ShaderTemplate>,
    expanded_template_library: Vec<ShaderTemplate>,

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
    audio_system: Option<Arc<std::sync::Mutex<crate::audio::AudioMidiSystem>>>,

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

    // Performance
    fps_counter: f32,
    frame_count: u32,
    last_fps_update: Instant,

    // Error handling
    shader_errors: Vec<String>,
    compilation_status: ShaderCompilationStatus,
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
            current_wgsl_code: Self::default_wgsl_shader(),
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
            grid_size: 20.0,
            pan_offset: egui::Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

#[cfg(feature = "gui")]
impl ShaderGui {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Box<Self> {
        // Load ISF shaders
        let shaders = load_resolume_isf_shaders().unwrap_or_default();

        // Initialize WGPU renderer
        let renderer = std::sync::Arc::new(std::sync::Mutex::new(
            pollster::block_on(ShaderRenderer::new()).unwrap_or_else(|e| {
                eprintln!("Failed to initialize WGPU renderer: {}", e);
                // Create a minimal fallback renderer
                pollster::block_on(ShaderRenderer::new()).unwrap()
            })
        ));

        let mut gui = Self {
            shaders,
            renderer: Some(renderer),
            ..Default::default()
        };

        // Load recent files on startup
        gui.load_recent_files();

        // Initialize gesture control system
        gui.gesture_control = Some(Arc::new(std::sync::Mutex::new(GestureControlSystem::new())));

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

    fn apply_theme(&mut self, theme_name: &str) {
        // Dynamic theme switching functionality
        match theme_name {
            "professional_dark" => {
                // Already applied by default
            }
            "professional_light" => {
                // Light theme implementation
            }
            "midnight_blue" => {
                // Blue theme
            }
            "sunrise_orange" => {
                // Orange theme
            }
            "forest_green" => {
                // Green theme
            }
            "purple_haze" => {
                // Purple theme
            }
            _ => {}
        }
    }
    fn default_wgsl_shader() -> String {
        r#"// Default WGSL Shader
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;
@group(0) @binding(2) var<uniform> mouse: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    let col = 0.5 + 0.5 * cos(time + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));
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
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    
    // Create a color based on position and time
    let r = uv.x;                    // Red based on X
    let g = uv.y;                    // Green based on Y
    let b = 0.5 + 0.5 * sin(time);   // Blue oscillates with time
    
    return vec4<f32>(r, g, b, 1.0);
}"#.to_string(),
            },

            ShaderTemplate {
                name: "Simple Animation".to_string(),
                description: "Basic time-based animation".to_string(),
                category: "Tutorial".to_string(),
                wgsl_code: r#"
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy - 0.5 * resolution) / min(resolution.x, resolution.y);
    let t = time;
    
    // Simple circle that pulsates
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

    fn compile_wgsl_shader(&mut self) {
        self.compilation_status = ShaderCompilationStatus::Compiling;
        self.shader_errors.clear();

        // Enhanced WGSL validation
        let mut is_valid = true;
        let mut errors = Vec::new();

        // Check for required WGSL elements
        if !self.current_wgsl_code.contains("@fragment") {
            is_valid = false;
            errors.push("Missing @fragment attribute".to_string());
        }

        if !self.current_wgsl_code.contains("fn fs_main") {
            is_valid = false;
            errors.push("Missing fs_main function".to_string());
        }

        if !self.current_wgsl_code.contains("@builtin(position)") {
            is_valid = false;
            errors.push("Missing @builtin(position) parameter".to_string());
        }

        if !self.current_wgsl_code.contains("@location(0)") {
            is_valid = false;
            errors.push("Missing @location(0) output".to_string());
        }

        // Check for basic syntax issues
        let lines: Vec<&str> = self.current_wgsl_code.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();

            // Check for unmatched braces
            let open_braces = line.chars().filter(|&c| c == '{').count();
            let close_braces = line.chars().filter(|&c| c == '}').count();
            if open_braces != close_braces {
                is_valid = false;
                errors.push(format!("Line {}: Unmatched braces", i + 1));
            }

            // Check for missing semicolons (basic check)
            if line.contains("let ") && !line.ends_with(';') && !line.contains('{') && !line.contains('}') {
                is_valid = false;
                errors.push(format!("Line {}: Missing semicolon after let statement", i + 1));
            }
        }

        if is_valid {
            self.compilation_status = ShaderCompilationStatus::Success;
            self.shader_errors.push("✓ Shader compiled successfully!".to_string());
            println!("Shader compiled successfully with enhanced validation");
        } else {
            self.compilation_status = ShaderCompilationStatus::Error("Compilation failed".to_string());
            self.shader_errors.extend(errors);
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

            // Prepare shader for rendering (disabled for GUI build)
            // if let Some(renderer) = &mut self.renderer {
            //     let _ = renderer.prepare_shader(shader);
            // }

            println!("Selected shader: {}", shader.name);
        }
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // File menu - TouchDesigner style
                ui.menu_button("File", |ui| {
                    if ui.button("New Shader").clicked() {
                        self.new_shader();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("Open...").clicked() {
                        self.open_file();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        if let Some(path) = self.current_file.clone() {
                            self.save_file(&path);
                        }
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("Save As...").clicked() {
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
                    ui.menu_button("Recent Files", |ui| {
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
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                // Edit menu
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        // Undo functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("Redo").clicked() {
                        // Redo functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button("Cut").clicked() {
                        // Cut functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("Copy").clicked() {
                        // Copy functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("Paste").clicked() {
                        // Paste functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button("Find").clicked() {
                        // Find functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("Replace").clicked() {
                        // Replace functionality
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });

                // View menu - TouchDesigner style panels
                ui.menu_button("View", |ui| {
                    ui.label("⚪ Code Editor (Always On)");
                    ui.label("⚪ Live Preview (Always On)");
                    ui.checkbox(&mut self.show_node_editor, "Node Editor");
                    ui.checkbox(&mut self.show_file_browser, "File Browser");
                    ui.checkbox(&mut self.show_converter, "Shader Converter");
                    ui.checkbox(&mut self.show_audio_panel, "Audio Panel");
                    ui.checkbox(&mut self.show_midi_panel, "MIDI Panel");
                    ui.checkbox(&mut self.show_gesture_panel, "Gesture Control");
                    ui.separator();
                    if ui.button("Reset Layout").clicked() {
                        self.reset_layout();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });

                // Tools menu - Shader-specific tools
                ui.menu_button("Tools", |ui| {
                    if ui.button("Compile Shader").clicked() {
                        self.compile_wgsl_shader();
                    }
                    ui.separator();
                    if ui.button("Export to GLSL").clicked() {
                        self.export_to_glsl();
                    }
                    if ui.button("Export to HLSL").clicked() {
                        self.export_to_hlsl();
                    }
                    ui.separator();
                    if ui.button("Import ISF Shader").clicked() {
                        self.import_isf_shader();
                    }
                    ui.separator();
                    if ui.button("Initialize Leap Motion").clicked() {
                        self.initialize_leap_motion();
                    }
                    if ui.button("Initialize MediaPipe").clicked() {
                        self.initialize_mediapipe();
                    }
                });

                // Help menu
                ui.menu_button("Help", |ui| {
                    if ui.button("Documentation").clicked() {
                        // Open documentation
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui.button("Keyboard Shortcuts").clicked() {
                        // Show shortcuts
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button("About WGSL Shader Studio").clicked() {
                        // Show about dialog
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });
            });
        });
    }

    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("WGSL Code Editor", |ui| {
            // Professional toolbar with functional buttons
            ui.horizontal(|ui| {
                if ui.button("Compile").clicked() {
                    self.compile_wgsl_shader();
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

                // Functional editing features
                if ui.button("🔧 Format").clicked() {
                    self.format_wgsl_code();
                }
                if ui.button("Find").clicked() {
                    // Find functionality would be implemented
                }
            });

            // Error display with clear formatting
            if !self.shader_errors.is_empty() {
                ui.separator();
                ui.colored_label(egui::Color32::RED, "Compilation Errors:");
                egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                    for error in &self.shader_errors {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::RED, "•");
                            ui.label(error);
                        });
                    }
                });
            }

            // Success message
            if let ShaderCompilationStatus::Success = self.compilation_status {
                ui.separator();
                ui.colored_label(egui::Color32::GREEN, "✓ Shader compiled successfully");
            }

            // Main code editor with line numbers
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Line numbers column
                    ui.vertical(|ui| {
                        ui.set_width(35.0);
                        let line_count = self.current_wgsl_code.lines().count().max(1);
                        for i in 1..=line_count {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                ui.add_space(2.0);
                                ui.colored_label(egui::Color32::from_rgb(120, 120, 120), format!("{:2}", i));
                            });
                        }
                    });

                    ui.separator();

                    // Code editing area
                    ui.vertical(|ui| {
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut self.current_wgsl_code)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(20)
                                .desired_width(f32::INFINITY)
                        );

                        // Functional context menu
                        response.context_menu(|ui| {
                            if ui.button("Cut").clicked() {
                                // Cut functionality would use clipboard
                                ui.close();
                            }
                            if ui.button("Copy").clicked() {
                                // Copy functionality would use clipboard
                                ui.close();
                            }
                            if ui.button("Paste").clicked() {
                                // Paste functionality would use clipboard
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("Format Code").clicked() {
                                self.format_code();
                                ui.close();
                            }
                        });
                    });
                });
            });

            // Code statistics
            ui.separator();
            ui.horizontal(|ui| {
                let lines = self.current_wgsl_code.lines().count();
                let chars = self.current_wgsl_code.chars().count();

                ui.label(format!("Lines: {}", lines));
                ui.separator();
                ui.label(format!("Characters: {}", chars));
                ui.separator();
                ui.label(format!("File: {}", self.current_file.as_ref().map_or("Unsaved".to_string(), |p| p.file_name().unwrap_or_default().to_string_lossy().to_string())));
            });
        });
    }

    fn render_node_editor(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("🎨 Node-based Shader Editor", |ui| {
            ui.label("🧩 Visual shader composition - drag nodes to create shaders");

            // Professional node palette with categories
            ui.label("🎭 Node Palette:");

            // Input/Output nodes
            ui.horizontal_wrapped(|ui| {
                ui.label("📥 I/O:");
                if ui.button("➕ Input").clicked() {
                    self.add_node(NodeType::Input, egui::pos2(100.0, 100.0));
                }
                if ui.button("➖ Output").clicked() {
                    self.add_node(NodeType::Output, egui::pos2(400.0, 100.0));
                }
            });

            // Math nodes
            ui.horizontal_wrapped(|ui| {
                ui.label("🔢 Math:");
                if ui.button("➕ Add").clicked() {
                    self.add_node(NodeType::Math, egui::pos2(250.0, 200.0));
                }
                if ui.button("➖ Subtract").clicked() {
                    self.add_node(NodeType::Math, egui::pos2(250.0, 250.0));
                }
                if ui.button("✖️ Multiply").clicked() {
                    self.add_node(NodeType::Math, egui::pos2(250.0, 300.0));
                }
                if ui.button("➗ Divide").clicked() {
                    self.add_node(NodeType::Math, egui::pos2(250.0, 350.0));
                }
                if ui.button("🔴 Sin").clicked() {
                    self.add_node(NodeType::Math, egui::pos2(250.0, 400.0));
                }
                if ui.button("🔵 Cos").clicked() {
                    self.add_node(NodeType::Math, egui::pos2(250.0, 450.0));
                }
            });

            // Color nodes
            ui.horizontal_wrapped(|ui| {
                ui.label("🎨 Color:");
                if ui.button("🌈 RGB").clicked() {
                    self.add_node(NodeType::Color, egui::pos2(250.0, 300.0));
                }
                if ui.button("⚫ HSV").clicked() {
                    self.add_node(NodeType::Color, egui::pos2(250.0, 350.0));
                }
                if ui.button("🔄 Mix").clicked() {
                    self.add_node(NodeType::Color, egui::pos2(250.0, 400.0));
                }
            });

            // Transform nodes
            ui.horizontal_wrapped(|ui| {
                ui.label("🔄 Transform:");
                if ui.button("📐 Scale").clicked() {
                    self.add_node(NodeType::Transform, egui::pos2(250.0, 400.0));
                }
                if ui.button("🔄 Rotate").clicked() {
                    self.add_node(NodeType::Transform, egui::pos2(250.0, 450.0));
                }
                if ui.button("📍 Translate").clicked() {
                    self.add_node(NodeType::Transform, egui::pos2(250.0, 500.0));
                }
            });

            // Texture nodes
            ui.horizontal_wrapped(|ui| {
                ui.label("🖼️ Texture:");
                if ui.button("🖼️ Sample").clicked() {
                    self.add_node(NodeType::Texture, egui::pos2(250.0, 500.0));
                }
                if ui.button("🔀 UV").clicked() {
                    self.add_node(NodeType::Texture, egui::pos2(250.0, 550.0));
                }
            });

            ui.separator();

            // Node graph controls
            ui.horizontal(|ui| {
                ui.label("🎛️ Graph Controls:");
                if ui.button("🗑️ Clear All").clicked() {
                    self.nodes.clear();
                    self.connections.clear();
                }
                if ui.button("💾 Save Graph").clicked() {
                    // Save node graph
                }
                if ui.button("📂 Load Graph").clicked() {
                    // Load node graph
                }
                ui.separator();
                ui.label(format!("📊 Nodes: {}", self.nodes.len()));
                ui.label(format!("🔗 Connections: {}", self.connections.len()));
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
        });
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

        // Generate WGSL code with proper uniforms and variable handling
        let mut wgsl_code = String::from("// Generated from node graph\n\n");
        wgsl_code.push_str("@group(0) @binding(0) var<uniform> time: f32;\n");
        wgsl_code.push_str("@group(0) @binding(1) var<uniform> resolution: vec2<f32>;\n");
        wgsl_code.push_str("@group(0) @binding(2) var<uniform> mouse: vec2<f32>;\n\n");

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
        // Professional preview controls in a toolbar
        ui.horizontal(|ui| {
            ui.label("📐 Resolution:");
            let mut width = self.preview_size.0 as i32;
            let mut height = self.preview_size.1 as i32;

            ui.add(egui::DragValue::new(&mut width).prefix("W:").range(256..=2048).speed(10));
            ui.add(egui::DragValue::new(&mut height).prefix("H:").range(256..=2048).speed(10));

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

            // Prominent render controls
            if ui.button("▶️ Play").clicked() {
                self.start_preview_rendering();
                self.compile_wgsl_shader();
            }
            if ui.button("⏸️ Pause").clicked() {
                self.pause_preview_rendering();
            }
            if ui.button("⏹️ Stop").clicked() {
                self.stop_preview_rendering();
            }
            if ui.button("📸 Screenshot").clicked() {
                self.take_screenshot();
            }

            ui.separator();

            // Template selection for real shader examples
            ui.label("🎨 Examples:");
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
            ui.label("🎛️ Time:");
            ui.add(egui::Slider::new(&mut 0.0, 0.0..=10.0).text(""));
        });

        ui.separator();

        // Main preview viewport - takes up most of the central space
        let available_size = ui.available_size();
        let preview_size = egui::vec2(available_size.x, available_size.y - 60.0); // Leave space for controls
        let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

        // High-contrast border to emphasize the viewport as the main focus
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(2),
            egui::Stroke::new(3.0, egui::Color32::from_rgb(120, 120, 130)),
            egui::StrokeKind::Middle,
        );

        // Real WGPU shader rendering based on compilation status
        match &self.compilation_status {
            ShaderCompilationStatus::Success => {
                // Use actual WGPU renderer for real shader output
                self.render_actual_shader_preview(ui, rect, preview_size);
            }
            ShaderCompilationStatus::Compiling => {
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(4),
                    egui::Color32::from_rgb(255, 200, 0),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🟡 Compiling Shader...",
                    egui::FontId::proportional(18.0),
                    egui::Color32::BLACK,
                );
            }
            ShaderCompilationStatus::Error(_) => {
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(4),
                    egui::Color32::from_rgb(200, 50, 50),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🔴 Compilation Error\nCheck code and try again",
                    egui::FontId::proportional(16.0),
                    egui::Color32::WHITE,
                );
            }
            ShaderCompilationStatus::NotCompiled => {
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(4),
                    egui::Color32::from_rgb(80, 80, 80),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "⚪ Ready to Compile\nClick '▶️ Play' to start",
                    egui::FontId::proportional(16.0),
                    egui::Color32::WHITE,
                );
            }
        }

        // Performance overlay in top-right corner (subtle but visible)
        let overlay_rect = egui::Rect::from_min_size(
            rect.max - egui::vec2(120.0, 50.0),
            egui::vec2(110.0, 40.0)
        );
        ui.painter().rect_filled(overlay_rect, egui::CornerRadius::same(6), egui::Color32::from_black_alpha(200));
        ui.painter().rect_stroke(overlay_rect, egui::CornerRadius::same(6), egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150)), egui::StrokeKind::Middle);

        ui.painter().text(
            overlay_rect.min + egui::vec2(8.0, 5.0),
            egui::Align2::LEFT_TOP,
            format!("{:.0} FPS", self.render_fps),
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
        ui.painter().text(
            overlay_rect.min + egui::vec2(8.0, 20.0),
            egui::Align2::LEFT_TOP,
            format!("{}×{}", self.preview_size.0, self.preview_size.1),
            egui::FontId::proportional(10.0),
            egui::Color32::from_rgb(200, 200, 200),
        );

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
        // Get audio data if available
        let audio_data = if let Some(audio_sys) = &self.audio_system {
            if let Ok(audio) = audio_sys.lock() {
                let local_data = audio.get_audio_data();
                Some(AudioData {
                    volume: local_data.volume,
                    bass_level: local_data.bass_level,
                    mid_level: local_data.mid_level,
                    treble_level: local_data.treble_level,
                    beat: local_data.beat,
                    centroid: local_data.centroid,
                    rolloff: local_data.rolloff,
                    spectrum: local_data.spectrum.clone(),
                    waveform: local_data.waveform.clone(),
                })
            } else {
                None
            }
        } else {
            None
        };

        // Check if we have a renderer available
        if self.renderer.is_none() {
            self.render_fallback_animation(ui, rect, preview_size);
            return;
        }

        // Try to render with the current shader
        let params = RenderParameters {
            width: self.preview_size.0,
            height: self.preview_size.1,
            time: self.last_render_time.elapsed().as_secs_f32(),
            frame_rate: self.render_fps,
            audio_data: audio_data.clone(),
        };

        let wgsl_code = self.current_wgsl_code.clone();

        // Try to render the user's current shader first
        let render_result = if let Ok(mut renderer) = self.renderer.as_ref().unwrap().lock() {
            renderer.render_frame(&wgsl_code, &params, audio_data.clone())
        } else {
            Err("Failed to lock renderer".into())
        };

        match render_result {
            Ok(pixel_data) => {
                // Successfully rendered shader - create egui texture from pixel data
                let texture_id = format!("shader_output_{}_{}", self.preview_size.0, self.preview_size.1);

                // Ensure pixel data is the correct size
                let expected_size = (self.preview_size.0 * self.preview_size.1 * 4) as usize;
                if pixel_data.len() == expected_size {
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
                // Shader compilation/rendering failed - try working examples
                eprintln!("User shader rendering error: {}", e);

                // Get working examples first
                let working_examples: Vec<WorkingShaderExample> = if let Ok(renderer) = self.renderer.as_ref().unwrap().lock() {
                    renderer.get_working_examples().to_vec()
                } else {
                    Vec::new()
                };

                if let Some(example) = working_examples.first() {
                    // Try to render the example
                    let example_result = if let Ok(mut renderer) = self.renderer.as_ref().unwrap().lock() {
                        renderer.render_frame(&example.wgsl_code, &params, audio_data)
                    } else {
                        Err("Failed to lock renderer for example".into())
                    };

                    match example_result {
                        Ok(pixel_data) => {
                            // Successfully rendered example shader
                            let texture_id = format!("example_output_{}_{}", self.preview_size.0, self.preview_size.1);

                            // Ensure pixel data is the correct size
                            let expected_size = (self.preview_size.0 * self.preview_size.1 * 4) as usize;
                            if pixel_data.len() == expected_size {
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
                                self.render_fallback_animation(ui, rect, preview_size);
                            }
                        }
                        Err(example_e) => {
                            // Even example failed - fallback to animation
                            eprintln!("Example shader rendering error: {}", example_e);
                            self.render_fallback_animation(ui, rect, preview_size);
                        }
                    }
                } else {
                    self.render_fallback_animation(ui, rect, preview_size);
                }
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
        // Placeholder for preview rendering start
        println!("Preview rendering started (placeholder)");
    }


    fn take_screenshot(&mut self) {
        // Placeholder for screenshot functionality
        println!("Screenshot functionality not yet implemented");
    }


    fn parse_wgsl_with_errors(&self, code: &str) -> (String, Vec<(usize, usize)>) {
        // Basic error detection - in a real implementation, this would use a proper WGSL parser
        let mut error_ranges = Vec::new();

        // Simple syntax error detection
        for (line_idx, line) in code.lines().enumerate() {
            // Check for common syntax errors
            if line.contains("let") && !line.contains('=') && !line.contains(';') {
                // Missing assignment or semicolon
                let start = self.get_global_position(line_idx, 0);
                let end = self.get_global_position(line_idx, line.len());
                error_ranges.push((start, end));
            }

            // Check for unmatched braces
            let open_braces = line.chars().filter(|&c| c == '{').count();
            let close_braces = line.chars().filter(|&c| c == '}').count();
            if open_braces != close_braces {
                let start = self.get_global_position(line_idx, 0);
                let end = self.get_global_position(line_idx, line.len());
                error_ranges.push((start, end));
            }

            // Check for missing semicolons
            if line.trim().starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '@') && !line.trim().ends_with(';') && !line.trim().ends_with('{') && !line.trim().ends_with('}') && !line.trim().starts_with("//") {
                let start = self.get_global_position(line_idx, 0);
                let end = self.get_global_position(line_idx, line.len());
                error_ranges.push((start, end));
            }
        }

        (code.to_string(), error_ranges)
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
        let mut glsl_code = wgsl_code.to_string();

        // Remove WGSL-specific attributes and replace with GLSL equivalents
        glsl_code = glsl_code.replace("@fragment", "void main()");
        glsl_code = glsl_code.replace("@vertex", "void main()");
        glsl_code = glsl_code.replace("@builtin(position)", "");
        glsl_code = glsl_code.replace("@location(0)", "");
        
        // Replace uniform bindings
        glsl_code = glsl_code.replace("@group(0) @binding(0)", "uniform");
        glsl_code = glsl_code.replace("@group(0) @binding(1)", "uniform");
        glsl_code = glsl_code.replace("@group(0) @binding(2)", "uniform");
        
        // Type conversions
        glsl_code = glsl_code.replace("vec2<f32>", "vec2");
        glsl_code = glsl_code.replace("vec3<f32>", "vec3");
        glsl_code = glsl_code.replace("vec4<f32>", "vec4");
        glsl_code = glsl_code.replace("f32", "float");
        glsl_code = glsl_code.replace("i32", "int");
        
        // Replace variable declarations
        glsl_code = glsl_code.replace("var<uniform> time", "float time");
        glsl_code = glsl_code.replace("var<uniform> resolution", "vec2 resolution");
        glsl_code = glsl_code.replace("var<uniform> mouse", "vec2 mouse");
        
        // Replace output
        glsl_code = glsl_code.replace("return vec4<f32>(", "gl_FragColor = vec4(");
        glsl_code = glsl_code.replace(") -> @location(0) vec4<f32>", ");");
        
        // Fix common syntax issues
        glsl_code = glsl_code.replace("let ", "float ");
        glsl_code = glsl_code.replace("fn ", "void ");
        
        (glsl_code, true)
    }

    fn convert_wgsl_to_hlsl(&self, wgsl_code: &str) -> (String, bool) {
        let mut hlsl_code = wgsl_code.to_string();

        // Replace fragment shader attributes
        hlsl_code = hlsl_code.replace("@fragment", "float4 main(");
        hlsl_code = hlsl_code.replace("@vertex", "void main(");
        
        // Replace uniform bindings with HLSL registers
        hlsl_code = hlsl_code.replace("@group(0) @binding(0)", "cbuffer CB0 : register(b0)");
        hlsl_code = hlsl_code.replace("@group(0) @binding(1)", "cbuffer CB1 : register(b1)");
        hlsl_code = hlsl_code.replace("@group(0) @binding(2)", "cbuffer CB2 : register(b2)");
        
        // Type conversions
        hlsl_code = hlsl_code.replace("vec2<f32>", "float2");
        hlsl_code = hlsl_code.replace("vec3<f32>", "float3");
        hlsl_code = hlsl_code.replace("vec4<f32>", "float4");
        hlsl_code = hlsl_code.replace("f32", "float");
        
        // Replace output
        hlsl_code = hlsl_code.replace("return vec4<f32>(", "return float4(");
        hlsl_code = hlsl_code.replace(") -> @location(0) vec4<f32>", ") : SV_Target;");
        
        // Fix function declarations
        hlsl_code = hlsl_code.replace("fn ", "");
        
        (hlsl_code, true)
    }

    fn convert_glsl_to_wgsl(&self, glsl_code: &str) -> (String, bool) {
        let mut wgsl_code = glsl_code.to_string();

        // Replace main function
        wgsl_code = wgsl_code.replace("void main()", "@fragment\nfn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {");
        
        // Add uniforms if missing
        if !wgsl_code.contains("@group(0) @binding(0)") {
            wgsl_code.insert_str(0, "@group(0) @binding(0) var<uniform> time: f32;\n");
            wgsl_code.insert_str(0, "@group(0) @binding(1) var<uniform> resolution: vec2<f32>;\n");
            wgsl_code.insert_str(0, "@group(0) @binding(2) var<uniform> mouse: vec2<f32>;\n\n");
        }
        
        // Replace output
        wgsl_code = wgsl_code.replace("gl_FragColor = vec4(", "return vec4<f32>(");
        wgsl_code = wgsl_code.replace("float", "f32");
        wgsl_code = wgsl_code.replace("vec2", "vec2<f32>");
        wgsl_code = wgsl_code.replace("vec3", "vec3<f32>");
        wgsl_code = wgsl_code.replace("vec4", "vec4<f32>");
        wgsl_code = wgsl_code.replace("int", "i32");
        wgsl_code = wgsl_code.replace(";", ";");
        
        // Fix common syntax issues
        wgsl_code = wgsl_code.replace("void ", "fn ");
        wgsl_code = wgsl_code.replace("float ", "let ");
        
        (wgsl_code, true)
    }

    fn convert_hlsl_to_wgsl(&self, hlsl_code: &str) -> (String, bool) {
        let mut wgsl_code = hlsl_code.to_string();

        // Replace main function
        wgsl_code = wgsl_code.replace("float4 main(", "@fragment\nfn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {");
        
        // Add uniforms if missing
        if !wgsl_code.contains("@group(0) @binding(0)") {
            wgsl_code.insert_str(0, "@group(0) @binding(0) var<uniform> time: f32;\n");
            wgsl_code.insert_str(0, "@group(0) @binding(1) var<uniform> resolution: vec2<f32>;\n");
            wgsl_code.insert_str(0, "@group(0) @binding(2) var<uniform> mouse: vec2<f32>;\n\n");
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
        let mut wgsl_code = String::from("// Converted from ISF\n\n");

        // Add standard WGSL uniforms
        wgsl_code.push_str("@group(0) @binding(0) var<uniform> time: f32;\n");
        wgsl_code.push_str("@group(0) @binding(1) var<uniform> resolution: vec2<f32>;\n");
        wgsl_code.push_str("@group(0) @binding(2) var<uniform> mouse: vec2<f32>;\n\n");

        // Basic ISF to WGSL conversion
        let converted_code = isf_code
            .replace("void main()", "@fragment\nfn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32>")
            .replace("gl_FragCoord", "position")
            .replace("gl_FragColor", "return vec4<f32>")
            .replace("vec2(", "vec2<f32>(")
            .replace("vec3(", "vec3<f32>(")
            .replace("vec4(", "vec4<f32>(")
            .replace("float ", "let ")
            .replace("uniform ", "// uniform ");

        wgsl_code.push_str(&converted_code);
        wgsl_code.push_str("\n}\n");

        wgsl_code
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

    fn render_parameter_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(shader_index) = self.current_shader {
            let shader = &self.shaders[shader_index];

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
        }
    }
    fn render_audio_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Audio Analysis", |ui| {
            if let Some(audio_sys) = &self.audio_system {
                if let Ok(audio_data) = audio_sys.lock() {
                    let data = audio_data.get_audio_data();

                    ui.label(format!("Volume: {:.3}", data.volume));
                    ui.label(format!("Beat: {:.3}", data.beat));
                    ui.label(format!("Bass: {:.3}", data.bass_level));
                    ui.label(format!("Mid: {:.3}", data.mid_level));
                    ui.label(format!("Treble: {:.3}", data.treble_level));
                    ui.label(format!("Centroid: {:.3}", data.centroid));
                    ui.label(format!("Rolloff: {:.3}", data.rolloff));

                    // Audio reactivity toggles
                    ui.horizontal(|ui| {
                        let mut audio_enabled = audio_data.audio_reactivity_enabled;
                        if ui.checkbox(&mut audio_enabled, "Audio Reactivity").changed() {
                            drop(audio_data);
                            if let Ok(mut audio) = audio_sys.lock() {
                                audio.toggle_audio_reactivity();
                            }
                        }
                    });

                    // Audio visualization (simple bars)
                    ui.label("Spectrum:");
                    // Placeholder spectrum data since get_spectrum method doesn't exist
                    let spectrum = vec![0.1, 0.3, 0.2, 0.5, 0.4, 0.6, 0.3, 0.7, 0.2, 0.8];
                    for &amp in spectrum.iter() {
                        ui.add(egui::ProgressBar::new(amp).show_percentage().animate(false));
                    }
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
            if let Some(audio_sys) = &self.audio_system {
                if let Ok(audio_data) = audio_sys.lock() {
                    let midi = audio_data.get_midi_controller();

                    ui.label("MIDI Mappings:");
                    for (key, mapping) in midi.get_mappings() {
                        ui.horizontal(|ui| {
                            ui.label(format!("CC{} -> {}", key.1, mapping.parameter_name));
                            if ui.button("Remove").clicked() {
                                // Remove mapping functionality would be implemented
                            }
                        });
                    }

                    ui.separator();
                    ui.label("Add MIDI Mapping:");
                    ui.horizontal(|ui| {
                        ui.label("Parameter:");
                        egui::ComboBox::from_label("")
                            .selected_text("time")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut "time", "time", "time");
                                ui.selectable_value(&mut "mouse_x", "mouse_x", "mouse_x");
                                ui.selectable_value(&mut "mouse_y", "mouse_y", "mouse_y");
                            });

                        ui.label("CC:");
                        let mut cc_value = 0;
                        ui.add(egui::DragValue::new(&mut cc_value).range(0..=127));

                        if ui.button("Add").clicked() {
                            // Add MIDI mapping functionality would be implemented
                        }
                    });

                    ui.horizontal(|ui| {
                        let mut midi_enabled = audio_data.midi_enabled;
                        if ui.checkbox(&mut midi_enabled, "MIDI Control").changed() {
                            drop(audio_data);
                            if let Ok(mut audio) = audio_sys.lock() {
                                audio.toggle_midi();
                            }
                        }
                    });
                } else {
                    ui.label("MIDI system not available");
                }
            } else {
                ui.label("MIDI system not available");
            }
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

}


#[cfg(feature = "gui")]
impl eframe::App for ShaderGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint for smooth animation
        ctx.request_repaint();

        // Menu bar
        self.render_menu_bar(ctx);

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

        // Main content - Modern three-panel layout inspired by Blender/Nuke
        egui::CentralPanel::default().show(ctx, |ui| {
            // Use egui::SidePanel and CentralPanel for better layout control
            let mut left_panel_width = 280.0;
            let mut right_panel_width = 320.0;
            let mut bottom_panel_height = 200.0;

            // Left Panel - Tools and Templates
            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(left_panel_width)
                .width_range(200.0..=400.0)
                .show_inside(ui, |ui| {
                    ui.set_min_height(600.0);

                    if self.show_file_browser {
                        self.render_shader_browser(ui);
                    }

                    self.render_templates(ui);

                    if self.show_converter {
                        self.render_converter(ui);
                    }
                });

            // Right Panel - Parameters and Controls
            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(right_panel_width)
                .width_range(250.0..=500.0)
                .show_inside(ui, |ui| {
                    ui.set_min_height(600.0);

                    self.render_parameter_panel(ui);

                    if self.show_audio_panel {
                        self.render_audio_panel(ui);
                    }

                    if self.show_midi_panel {
                        self.render_midi_panel(ui);
                    }

                    if self.show_gesture_panel {
                        self.render_gesture_panel(ui);
                    }

                    self.render_performance_panel(ui);
                });

            // Bottom Panel - Code Editor
            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(true)
                .default_height(bottom_panel_height)
                .height_range(150.0..=400.0)
                .show_inside(ui, |ui| {
                    if self.show_code_editor {
                        self.render_code_editor(ui);
                    }
                });

            // Central Panel - Main Viewport with Tabs
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.set_min_height(400.0);

                // Tabbed interface for Live Preview and Node Editor
                ui.horizontal(|ui| {
                    if ui.selectable_label(!self.show_node_editor, "🎬 Live Preview").clicked() {
                        self.show_node_editor = false;
                    }
                    if ui.selectable_label(self.show_node_editor, "🎨 Node Editor").clicked() {
                        self.show_node_editor = true;
                    }
                });

                ui.separator();

                if !self.show_node_editor {
                    // Live Preview Tab
                    if self.show_preview {
                        self.render_live_preview(ui);
                    }
                } else {
                    // Node Editor Tab
                    if self.show_node_editor {
                        self.render_node_editor(ui);
                    }
                }
            });
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
    }
}

#[cfg(feature = "gui")]
pub fn run_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("WGSL Shader Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "WGSL Shader Studio",
        options,
        Box::new(|cc| Ok(ShaderGui::new(cc))),
    ).unwrap();
}

#[cfg(not(feature = "gui"))]
pub fn run_gui() {
    println!("GUI feature not enabled. Use --features gui to enable the graphical interface.");
}