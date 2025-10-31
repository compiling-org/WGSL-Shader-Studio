//! WGSL Shader Studio - Comprehensive GUI Application

#[cfg(feature = "gui")]
use eframe::egui;
#[cfg(feature = "gui")]
use std::collections::HashMap;
#[cfg(feature = "gui")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "gui")]
use std::path::PathBuf;
#[cfg(feature = "gui")]
use std::time::{Duration, Instant};

#[cfg(feature = "gui")]
use crate::isf_loader::*;
#[cfg(feature = "gui")]
use crate::shader_renderer::*;
#[cfg(feature = "gui")]
use crate::audio::*;

#[cfg(feature = "gui")]
pub struct ShaderGui {
    // Shader management
    shaders: Vec<IsfShader>,
    current_shader: Option<usize>,
    parameter_values: HashMap<String, f32>,
    current_wgsl_code: String,
    shader_templates: Vec<ShaderTemplate>,
    expanded_template_library: Self::create_expanded_template_library(),

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

    // File management
    current_file: Option<PathBuf>,
    recent_files: Vec<PathBuf>,

    // Audio/MIDI system
    audio_system: Arc<Mutex<AudioMidiSystem>>,

    // Shader converter state
    from_format: String,
    to_format: String,

    // Live Preview System
    renderer: Option<ShaderRenderer>,
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
    Input,
    Output,
    Math,
    Color,
    Texture,
    Transform,
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
            audio_system: Arc::new(Mutex::new(AudioMidiSystem::new())),
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
            from_format: "WGSL".to_string(),
            to_format: "GLSL".to_string(),
        }
    }
}

#[cfg(feature = "gui")]
impl ShaderGui {
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
            // Basic Templates
            ShaderTemplate {
                name: "Solid Color".to_string(),
                description: "Single solid color background".to_string(),
                category: "Basic".to_string(),
                wgsl_code: r#"// Solid Color
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.2, 0.8, 0.6, 1.0);
}"#.to_string(),
            },
            ShaderTemplate {
                name: "Gradient Background".to_string(),
                description: "Simple vertical gradient".to_string(),
                category: "Basic".to_string(),
                wgsl_code: r#"// Vertical Gradient
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    let col = mix(vec3<f32>(0.1, 0.2, 0.8), vec3<f32>(0.9, 0.7, 0.2), uv.y);
    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },

            // Animation Templates
            ShaderTemplate {
                name: "Pulsing Circle".to_string(),
                description: "Animated pulsing circle in center".to_string(),
                category: "Animation".to_string(),
                wgsl_code: r#"// Pulsing Circle
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy - 0.5 * resolution) / min(resolution.x, resolution.y);
    let dist = length(uv);
    let pulse = sin(time * 3.0) * 0.5 + 0.5;
    let circle = 1.0 - smoothstep(0.1 + pulse * 0.2, 0.15 + pulse * 0.2, dist);
    return vec4<f32>(vec3<f32>(circle), 1.0);
}"#.to_string(),
            },
            ShaderTemplate {
                name: "Wave Pattern".to_string(),
                description: "Sinusoidal wave animation".to_string(),
                category: "Animation".to_string(),
                wgsl_code: r#"// Wave Pattern
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    let wave = sin(uv.x * 10.0 + time * 2.0) * 0.5 + 0.5;
    let col = vec3<f32>(wave, 0.5, 1.0 - wave);
    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },

            // Fractal Templates
            ShaderTemplate {
                name: "Mandelbrot Set".to_string(),
                description: "Classic Mandelbrot fractal".to_string(),
                category: "Fractal".to_string(),
                wgsl_code: r#"// Mandelbrot Fractal
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

fn mandelbrot(c: vec2<f32>) -> f32 {
    var z = vec2<f32>(0.0, 0.0);
    for (var i = 0; i < 100; i = i + 1) {
        if (dot(z, z) > 4.0) {
            return f32(i) / 100.0;
        }
        z = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
    }
    return 1.0;
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

            // Effect Templates
            ShaderTemplate {
                name: "Vortex Effect".to_string(),
                description: "Spiral vortex pattern".to_string(),
                category: "Effects".to_string(),
                wgsl_code: r#"// Vortex Effect
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy - 0.5 * resolution) / min(resolution.x, resolution.y);
    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);
    let twist = angle + radius * 5.0 + time;
    let pattern = sin(twist * 3.0) * 0.5 + 0.5;
    let col = vec3<f32>(pattern, pattern * 0.7, pattern * 0.9);
    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },
            ShaderTemplate {
                name: "Particle System".to_string(),
                description: "Simple particle effect".to_string(),
                category: "Effects".to_string(),
                wgsl_code: r#"// Particle Effect
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

fn random(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    var col = vec3<f32>(0.0);

    for (var i = 0; i < 20; i = i + 1) {
        let fi = f32(i);
        let seed = vec2<f32>(fi, fi + 1.0);
        let pos = vec2<f32>(
            random(seed + time * 0.1),
            random(seed + vec2<f32>(1.0, 0.0) + time * 0.1)
        );
        let dist = distance(uv, pos);
        let particle = 1.0 - smoothstep(0.0, 0.02, dist);
        col += vec3<f32>(particle);
    }

    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },

            // Tutorial Templates
            ShaderTemplate {
                name: "Hello WGSL".to_string(),
                description: "Basic WGSL introduction".to_string(),
                category: "Tutorial".to_string(),
                wgsl_code: r#"// Hello WGSL - Basic Introduction
// This shader demonstrates basic WGSL concepts

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    // position contains the screen coordinates
    // We can use it to create simple patterns

    let x = position.x / 800.0;  // Normalize x coordinate
    let y = position.y / 600.0;  // Normalize y coordinate

    // Create a simple gradient
    let r = x;        // Red increases with x
    let g = y;        // Green increases with y
    let b = 0.5;      // Blue is constant

    return vec4<f32>(r, g, b, 1.0);
}"#.to_string(),
            },
            ShaderTemplate {
                name: "Uniforms Tutorial".to_string(),
                description: "Learn about uniform variables".to_string(),
                category: "Tutorial".to_string(),
                wgsl_code: r#"// Uniforms Tutorial
// Uniforms allow you to pass data from CPU to GPU

@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;
@group(0) @binding(2) var<uniform> mouse: vec2<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;

    // Use time for animation
    let t = sin(time) * 0.5 + 0.5;

    // Use mouse position for interaction
    let mouse_dist = distance(uv, mouse / resolution);

    // Combine time and mouse interaction
    let col = vec3<f32>(t, 1.0 - mouse_dist, mouse_dist);

    return vec4<f32>(col, 1.0);
}"#.to_string(),
            },
        ]
    }

#[cfg(feature = "gui")]
impl ShaderGui {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Box<Self> {
        // Load ISF shaders
        let shaders = load_resolume_isf_shaders().unwrap_or_default();

        let mut gui = Self {
            shaders,
            ..Default::default()
        };

        // Load recent files
        gui.load_recent_files();

        Box::new(gui)
    }

    fn load_recent_files(&mut self) {
        // Load recent files from a config file or registry
        // For now, just initialize as empty
        self.recent_files = Vec::new();
    }

    fn save_recent_files(&self) {
        // Save recent files to config
        // Implementation would save to a config file
    }

    fn compile_wgsl_shader(&mut self) {
        self.compilation_status = ShaderCompilationStatus::Compiling;
        self.shader_errors.clear();

        // Basic syntax validation (renderer integration pending)
        if self.current_wgsl_code.contains("@fragment") && self.current_wgsl_code.contains("fs_main") {
            self.compilation_status = ShaderCompilationStatus::Success;
            println!("Shader compiled successfully (basic validation)");
        } else {
            self.compilation_status = ShaderCompilationStatus::Error("Missing @fragment or fs_main function".to_string());
            self.shader_errors.push("WGSL shader must contain @fragment and fs_main function".to_string());
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
            .add_filter("WGSL Shaders", &["wgsl"])
            .add_filter("GLSL Shaders", &["glsl", "frag", "vert"])
            .add_filter("ISF Shaders", &["fs"])
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

    fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Shader").clicked() {
                    self.new_shader();
                    ui.close_menu();
                }
                if ui.button("Open...").clicked() {
                    self.open_file();
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    if let Some(path) = &self.current_file {
                        self.save_file(path);
                    }
                    ui.close_menu();
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
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Recent Files", |ui| {
                    for recent_file in &self.recent_files {
                        if ui.button(recent_file.display().to_string()).clicked() {
                            // Load file
                            ui.close_menu();
                        }
                    }
                });
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    // Undo functionality
                }
                if ui.button("Redo").clicked() {
                    // Redo functionality
                }
                ui.separator();
                if ui.button("Find").clicked() {
                    // Find functionality
                }
                if ui.button("Replace").clicked() {
                    // Replace functionality
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.show_code_editor, "Code Editor");
                ui.checkbox(&mut self.show_preview, "Live Preview");
                ui.checkbox(&mut self.show_audio_panel, "Audio Panel");
                ui.checkbox(&mut self.show_midi_panel, "MIDI Panel");
                ui.checkbox(&mut self.show_converter, "Shader Converter");
                ui.checkbox(&mut self.show_file_browser, "File Browser");
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Compile Shader").clicked() {
                    self.compile_wgsl_shader();
                }
                ui.separator();
                if ui.button("Export to GLSL").clicked() {
                    // Export functionality
                }
                if ui.button("Export to HLSL").clicked() {
                    // Export functionality
                }
            });
        });
    }

    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("WGSL Code Editor", |ui| {
            ui.horizontal(|ui| {
                if ui.button("Compile").clicked() {
                    self.compile_wgsl_shader();
                }

                ui.label(match &self.compilation_status {
                    ShaderCompilationStatus::NotCompiled => "Not compiled",
                    ShaderCompilationStatus::Compiling => "Compiling...",
                    ShaderCompilationStatus::Success => "✓ Success",
                    ShaderCompilationStatus::Error(_) => "✗ Error",
                });

                // Auto-completion toggle
                ui.checkbox(&mut false, "Auto-complete");
            });

            // Show compilation errors with squiggles simulation
            if !self.shader_errors.is_empty() {
                ui.colored_label(egui::Color32::RED, "Compilation Errors:");
                for error in &self.shader_errors {
                    ui.colored_label(egui::Color32::RED, format!("• {}", error));
                }
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // WGSL syntax highlighting (corrected implementation)
                let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                    let mut job = egui::text::LayoutJob::default();
                    let font_id = egui::FontId::monospace(12.0);

                    for line in string.lines() {
                        let mut is_comment = false;
                        let mut is_string = false;

                        // Temporary string to build up a word/token
                        let mut current_token = String::new();

                        for ch in line.chars() {
                            // Check for single-line comment start
                            if !is_string && current_token.ends_with('/') && ch == '/' {
                                // Backtrack to remove the '/' added to the previous token
                                current_token.pop();

                                // Flush the token before the comment begins
                                if !current_token.is_empty() {
                                    job.append(&current_token, 0.0, egui::TextFormat { font_id: font_id.clone(), color: self.get_wgsl_syntax_color(&current_token), ..Default::default() });
                                    current_token.clear();
                                }

                                // Now, the rest of the line is a comment
                                is_comment = true;
                            }

                            if is_comment {
                                current_token.push(ch);
                            } else if ch.is_whitespace() || (!ch.is_alphanumeric() && ch != '_' && ch != '@' && ch != '.') {
                                // Flush alphanumeric token before adding separator
                                if !current_token.is_empty() {
                                    let color = if is_string {
                                        egui::Color32::from_rgb(214, 157, 133) // Custom color for strings (e.g., orange/brown)
                                    } else {
                                        self.get_wgsl_syntax_color(&current_token)
                                    };

                                    job.append(&current_token, 0.0, egui::TextFormat { font_id: font_id.clone(), color, ..Default::default() });
                                    current_token.clear();
                                }

                                // Handle the separator character (whitespace or symbol)
                                let separator_color = if is_string {
                                    egui::Color32::from_rgb(214, 157, 133)
                                } else if ch == '@' {
                                    // Attributes like @fragment, @location
                                    egui::Color32::from_rgb(155, 155, 0)
                                } else {
                                    egui::Color32::WHITE
                                };

                                job.append(&ch.to_string(), 0.0, egui::TextFormat { font_id: font_id.clone(), color: separator_color, ..Default::default() });

                                // Toggle string state for quotes
                                if ch == '"' {
                                    is_string = !is_string;
                                }

                            } else {
                                current_token.push(ch);
                            }
                        }

                        // Flush the last token/comment of the line
                        if !current_token.is_empty() {
                            let color = if is_comment {
                                egui::Color32::GREEN
                            } else if is_string {
                                egui::Color32::from_rgb(214, 157, 133)
                            } else {
                                self.get_wgsl_syntax_color(&current_token)
                            };
                            job.append(&current_token, 0.0, egui::TextFormat { font_id: font_id.clone(), color, ..Default::default() });
                        }

                        // Add newline
                        job.append("\n", 0.0, egui::TextFormat { font_id: font_id.clone(), color: egui::Color32::WHITE, ..Default::default() });
                    }

                    // This layouter is for the egui::TextEdit widget
                    job.wrap.max_width = wrap_width;

                    ui.fonts(|f| job.into_galley(f))
                };

                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.current_wgsl_code)
                        .font(egui::TextStyle::Monospace)
                        .layouter(&mut layouter)
                        .desired_rows(20)
                        .lock_focus(true)
                );

                // Context menu for code editor
                response.context_menu(|ui| {
                    if ui.button("Cut").clicked() {
                        // Cut functionality
                        ui.close_menu();
                    }
                    if ui.button("Copy").clicked() {
                        // Copy functionality
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        // Paste functionality
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Find").clicked() {
                        // Find functionality
                        ui.close_menu();
                    }
                    if ui.button("Replace").clicked() {
                        // Replace functionality
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn render_node_editor(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Node-based Shader Editor", |ui| {
            ui.label("Visual shader composition - drag nodes to create shaders");

            // Node palette
            ui.horizontal_wrapped(|ui| {
                if ui.button("➕ Input").clicked() {
                    self.add_node(NodeType::Input, egui::pos2(100.0, 100.0));
                }
                if ui.button("➖ Output").clicked() {
                    self.add_node(NodeType::Output, egui::pos2(400.0, 100.0));
                }
                if ui.button("🔢 Math").clicked() {
                    self.add_node(NodeType::Math, egui::pos2(250.0, 200.0));
                }
                if ui.button("🎨 Color").clicked() {
                    self.add_node(NodeType::Color, egui::pos2(250.0, 300.0));
                }
                if ui.button("🔄 Transform").clicked() {
                    self.add_node(NodeType::Transform, egui::pos2(250.0, 400.0));
                }
            });

            ui.separator();

            // Node graph area
            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::drag());

            // Handle mouse interactions
            if response.drag_started() {
                // Start dragging
            }

            // Draw connections
            for connection in &self.connections {
                if let (Some(from_node), Some(to_node)) = (
                    self.nodes.iter().find(|n| n.id == connection.from_node),
                    self.nodes.iter().find(|n| n.id == connection.to_node)
                ) {
                    let from_pos = from_node.position + egui::vec2(from_node.size.x, from_node.size.y / 2.0);
                    let to_pos = to_node.position + egui::vec2(0.0, to_node.size.y / 2.0);

                    painter.line_segment([from_pos, to_pos], egui::Stroke::new(2.0, egui::Color32::WHITE));
                }
            }

            // Draw nodes
            for node in &self.nodes {
                let node_rect = egui::Rect::from_min_size(node.position, node.size);

                // Node background
                painter.rect_filled(node_rect, 4.0, egui::Color32::from_rgb(60, 60, 70));

                // Node title
                painter.text(
                    node.position + egui::vec2(8.0, 8.0),
                    egui::Align2::LEFT_TOP,
                    &node.title,
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );

                // Node inputs (left side)
                for (i, input) in node.inputs.iter().enumerate() {
                    let pin_pos = node.position + egui::vec2(0.0, 20.0 + i as f32 * 20.0);
                    painter.circle_filled(pin_pos, 4.0, egui::Color32::BLUE);
                    painter.text(
                        pin_pos + egui::vec2(10.0, -4.0),
                        egui::Align2::LEFT_CENTER,
                        &input.name,
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE,
                    );
                }

                // Node outputs (right side)
                for (i, output) in node.outputs.iter().enumerate() {
                    let pin_pos = node.position + egui::vec2(node.size.x, 20.0 + i as f32 * 20.0);
                    painter.circle_filled(pin_pos, 4.0, egui::Color32::GREEN);
                    painter.text(
                        pin_pos + egui::vec2(-10.0, -4.0),
                        egui::Align2::RIGHT_CENTER,
                        &output.name,
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE,
                    );
                }

                // Selection highlight
                if Some(node.id) == self.selected_node {
                    painter.rect_stroke(node_rect, 4.0, egui::Stroke::new(2.0, egui::Color32::YELLOW));
                }
            }

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

    fn add_node(&mut self, node_type: NodeType, position: egui::Pos2) {
        let id = NodeId(self.nodes.len());
        let (title, inputs, outputs) = match node_type {
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
            NodeType::Math => (
                "Math".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "a".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "b".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "result".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO }]
            ),
            NodeType::Color => (
                "Color".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "r".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "g".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(2), name: "b".to_string(), pin_type: PinType::Float, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(3), name: "color".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::Texture => (
                "Texture".to_string(),
                vec![NodePin { id: PinId(0), name: "uv".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }],
                vec![NodePin { id: PinId(1), name: "color".to_string(), pin_type: PinType::Color, position: egui::Pos2::ZERO }]
            ),
            NodeType::Transform => (
                "Transform".to_string(),
                vec![
                    NodePin { id: PinId(0), name: "position".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                    NodePin { id: PinId(1), name: "scale".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO },
                ],
                vec![NodePin { id: PinId(2), name: "transformed".to_string(), pin_type: PinType::Vec2, position: egui::Pos2::ZERO }]
            ),
        };

        self.nodes.push(Node {
            id,
            position,
            size: egui::vec2(120.0, 80.0),
            node_type,
            title,
            inputs,
            outputs,
            value: 0.0,
        });
    }

    fn convert_nodes_to_code(&mut self) {
        // Basic node to WGSL conversion
        let mut wgsl_code = String::from("// Generated from node graph\n\n");

        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {\n");

        // Generate code based on nodes
        for node in &self.nodes {
            match node.node_type {
                NodeType::Input => {
                    wgsl_code.push_str(&format!("    let input_{} = 0.5;\n", node.id.0));
                }
                NodeType::Math => {
                    wgsl_code.push_str(&format!("    let math_{} = input_a + input_b;\n", node.id.0));
                }
                NodeType::Color => {
                    wgsl_code.push_str(&format!("    let color_{} = vec4<f32>(r, g, b, 1.0);\n", node.id.0));
                }
                _ => {}
            }
        }

        wgsl_code.push_str("    return vec4<f32>(1.0, 1.0, 1.0, 1.0);\n");
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
        ui.collapsing("Live Preview", |ui| {
            // Preview controls
            ui.horizontal(|ui| {
                ui.label("Resolution:");
                let mut width = self.preview_size.0 as i32;
                let mut height = self.preview_size.1 as i32;
                if ui.add(egui::DragValue::new(&mut width).prefix("W: ").range(256..=2048)).changed() {
                    self.preview_size.0 = width as u32;
                }
                if ui.add(egui::DragValue::new(&mut height).prefix("H: ").range(256..=2048)).changed() {
                    self.preview_size.1 = height as u32;
                }
            });

            ui.horizontal(|ui| {
                ui.label(format!("Render FPS: {:.1}", self.render_fps));
                ui.separator();
                if ui.button("▶ Play").clicked() {
                    // Start preview rendering
                    self.compile_wgsl_shader();
                }
                if ui.button("⏸ Pause").clicked() {
                    // Pause preview
                }
                if ui.button("⏹ Stop").clicked() {
                    // Stop preview
                }
            });

            // Preview area
            let preview_size = egui::vec2(self.preview_size.0 as f32 * 0.5, self.preview_size.1 as f32 * 0.5);
            let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

            // Show compilation status (renderer integration pending)
            let color = match self.compilation_status {
                ShaderCompilationStatus::NotCompiled => egui::Color32::GRAY,
                ShaderCompilationStatus::Compiling => egui::Color32::YELLOW,
                ShaderCompilationStatus::Success => egui::Color32::GREEN,
                ShaderCompilationStatus::Error(_) => egui::Color32::RED,
            };

            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(4),
                color,
            );

            let status_text = match &self.compilation_status {
                ShaderCompilationStatus::NotCompiled => "Not Compiled",
                ShaderCompilationStatus::Compiling => "Compiling...",
                ShaderCompilationStatus::Success => "Ready to Render\n(WGPU Integration Pending)",
                ShaderCompilationStatus::Error(_) => "Compilation Error",
            };

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                status_text,
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );

            // Update render timing (placeholder)
            let now = Instant::now();
            let delta_time = now.duration_since(self.last_render_time).as_secs_f32();
            if delta_time >= 1.0 {
                self.render_fps = 1.0 / delta_time;
                self.last_render_time = now;
            }

            // Performance monitoring
            ui.separator();
            ui.label("Performance Monitoring:");
            ui.label(format!("Preview Size: {}x{}", self.preview_size.0, self.preview_size.1));
            ui.label(format!("Render FPS: {:.1}", self.render_fps));
            ui.label(format!("UI FPS: {:.1}", self.fps_counter));
        });
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
                ui.selectable_value(&mut self.selected_template_category, "Basic", "Basic");
                ui.selectable_value(&mut self.selected_template_category, "Animation", "Animation");
                ui.selectable_value(&mut self.selected_template_category, "Fractal", "Fractal");
                ui.selectable_value(&mut self.selected_template_category, "Effects", "Effects");
                ui.selectable_value(&mut self.selected_template_category, "Tutorial", "Tutorial");
                ui.selectable_value(&mut self.selected_template_category, "All", "All");
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                let templates_to_show = if self.selected_template_category == "All" {
                    &self.expanded_template_library
                } else {
                    &self.shader_templates.iter()
                        .chain(self.expanded_template_library.iter())
                        .filter(|t| t.category == self.selected_template_category)
                        .collect::<Vec<_>>()
                };

                for (i, template) in templates_to_show.iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_width(260.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&template.name).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Load").clicked() {
                                    self.load_expanded_template(template.clone());
                                }
                            });
                        });

                        ui.label(&template.description);
                        ui.small(format!("Category: {}", template.category));
                    });

                    ui.add_space(2.0);
                }

                if templates_to_show.is_empty() {
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
                // Shader conversion logic with multiple format support
                match (self.from_format.as_str(), self.to_format.as_str()) {
                    ("WGSL", "GLSL") => {
                        match crate::shader_converter::wgsl_to_glsl(&self.current_wgsl_code) {
                            Ok(glsl_code) => {
                                println!("WGSL to GLSL conversion successful");
                                self.shader_errors.clear();
                                self.shader_errors.push("GLSL conversion completed successfully".to_string());
                                // Could display converted code in a new panel
                            }
                            Err(e) => {
                                self.shader_errors.push(format!("GLSL conversion failed: {}", e));
                            }
                        }
                    }
                    ("WGSL", "HLSL") => {
                        match crate::shader_converter::wgsl_to_hlsl(&self.current_wgsl_code) {
                            Ok(hlsl_code) => {
                                println!("WGSL to HLSL conversion successful");
                                self.shader_errors.clear();
                                self.shader_errors.push("HLSL conversion completed successfully".to_string());
                            }
                            Err(e) => {
                                self.shader_errors.push(format!("HLSL conversion failed: {}", e));
                            }
                        }
                    }
                    ("GLSL", "WGSL") => {
                        // Basic GLSL to WGSL conversion (placeholder)
                        self.shader_errors.clear();
                        self.shader_errors.push("GLSL to WGSL conversion not yet implemented".to_string());
                    }
                    ("HLSL", "WGSL") => {
                        // Basic HLSL to WGSL conversion (placeholder)
                        self.shader_errors.clear();
                        self.shader_errors.push("HLSL to WGSL conversion not yet implemented".to_string());
                    }
                    _ => {
                        self.shader_errors.push("Conversion between selected formats not yet implemented".to_string());
                    }
                }
            }

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
                                    // For now, just show a float slider for color
                                    ui.add(egui::Slider::new(&mut new_value, 0.0..=1.0).text(""));
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
                    let spectrum = data.get_spectrum();
                    for (i, &amp) in spectrum.iter().enumerate().step_by(10) {
                        let height = (amp * 50.0) as usize;
                        ui.add(egui::ProgressBar::new(amp).show_percentage().animate(false));
                    }
                } else {
                    ui.label("Audio system not available");
                }
            } else {
                ui.label("Audio system not available");
            }
        });
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
                        ui.add(egui::DragValue::new(&mut cc_value).range(0..127));

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
            ui.label(format!("FPS: {:.1}", self.fps_counter));

            // Update FPS counter
            self.frame_count += 1;
            let now = std::time::Instant::now();
            if (now - self.last_fps_update).as_secs_f32() >= 1.0 {
                self.fps_counter = self.frame_count as f32;
                self.frame_count = 0;
                self.last_fps_update = now;
            }
        });
    }
}
}

#[cfg(feature = "gui")]
impl ShaderGui {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Box<Self> {
        // Load ISF shaders
        let shaders = load_resolume_isf_shaders().unwrap_or_default();

        let mut gui = Self {
            shaders,
            ..Default::default()
        };

        // Load recent files
        // gui.load_recent_files();

        Box::new(gui)
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                if ui.button("New Shader").clicked() {
                    self.new_shader();
                }
                if ui.button("Open...").clicked() {
                    self.open_file();
                }
                if ui.button("Save").clicked() {
                    if let Some(path) = &self.current_file {
                        self.save_file(path);
                    }
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
                }
                ui.separator();
                ui.menu_button("Recent Files", |ui| {
                    for recent_file in &self.recent_files {
                        if ui.button(recent_file.display().to_string()).clicked() {
                            // Load file functionality
                            match std::fs::read_to_string(recent_file) {
                                Ok(content) => {
                                    self.current_wgsl_code = content;
                                    self.current_file = Some(recent_file.clone());
                                    self.compile_wgsl_shader();
                                }
                                Err(e) => {
                                    self.shader_errors.push(format!("Failed to load file: {}", e));
                                }
                            }
                        }
                    }
                });
                });

                ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    // Undo functionality placeholder
                }
                if ui.button("Redo").clicked() {
                    // Redo functionality placeholder
                }
                ui.separator();
                if ui.button("Find").clicked() {
                    // Find functionality placeholder
                }
                if ui.button("Replace").clicked() {
                    // Replace functionality placeholder
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.show_code_editor, "Code Editor");
                ui.checkbox(&mut self.show_preview, "Live Preview");
                ui.checkbox(&mut self.show_node_editor, "Node Editor");
                ui.checkbox(&mut self.show_audio_panel, "Audio Panel");
                ui.checkbox(&mut self.show_midi_panel, "MIDI Panel");
                ui.checkbox(&mut self.show_converter, "Shader Converter");
                ui.checkbox(&mut self.show_file_browser, "File Browser");
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Compile Shader").clicked() {
                    self.compile_wgsl_shader();
                }
                ui.separator();
                if ui.button("Export to GLSL").clicked() {
                    // Export to GLSL
                    match crate::shader_converter::wgsl_to_glsl(&self.current_wgsl_code) {
                        Ok(glsl_code) => {
                            let task = rfd::AsyncFileDialog::new()
                                .add_filter("GLSL Shaders", &["glsl", "frag"])
                                .add_filter("All Files", &["*"])
                                .save_file();

                            pollster::block_on(async {
                                if let Some(file) = task.await {
                                    match std::fs::write(file.path(), glsl_code) {
                                        Ok(_) => {
                                            self.shader_errors.clear();
                                            self.shader_errors.push("Exported to GLSL successfully".to_string());
                                        }
                                        Err(e) => {
                                            self.shader_errors.push(format!("Failed to save GLSL file: {}", e));
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            self.shader_errors.push(format!("GLSL conversion failed: {}", e));
                        }
                    }
                }
                if ui.button("Export to HLSL").clicked() {
                    // Export to HLSL
                    match crate::shader_converter::wgsl_to_hlsl(&self.current_wgsl_code) {
                        Ok(hlsl_code) => {
                            let task = rfd::AsyncFileDialog::new()
                                .add_filter("HLSL Shaders", &["hlsl"])
                                .add_filter("All Files", &["*"])
                                .save_file();

                            pollster::block_on(async {
                                if let Some(file) = task.await {
                                    match std::fs::write(file.path(), hlsl_code) {
                                        Ok(_) => {
                                            self.shader_errors.clear();
                                            self.shader_errors.push("Exported to HLSL successfully".to_string());
                                        }
                                        Err(e) => {
                                            self.shader_errors.push(format!("Failed to save HLSL file: {}", e));
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            self.shader_errors.push(format!("HLSL conversion failed: {}", e));
                        }
                    }
                }
                ui.separator();
                ui.menu_button("Advanced", |ui| {
                    if ui.button("Shader Visualizer").clicked() {
                        // Shader visualizer functionality placeholder
                    }
                    if ui.button("AST Viewer").clicked() {
                        // AST visualization placeholder
                    }
                    if ui.button("Performance Analyzer").clicked() {
                        // Performance analysis placeholder
                    }
                });
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    // About dialog placeholder
                }
                if ui.button("Documentation").clicked() {
                    // Documentation placeholder
                }
                if ui.button("Keyboard Shortcuts").clicked() {
                    // Keyboard shortcuts help placeholder
                }
            });
        });
    }

    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("WGSL Code Editor", |ui| {
            ui.horizontal(|ui| {
                if ui.button("Compile").clicked() {
                    self.compile_wgsl_shader();
                }

                ui.label(match &self.compilation_status {
                    ShaderCompilationStatus::NotCompiled => "Not compiled",
                    ShaderCompilationStatus::Compiling => "Compiling...",
                    ShaderCompilationStatus::Success => "✓ Success",
                    ShaderCompilationStatus::Error(_) => "✗ Error",
                });
            });

            // Show compilation errors
            if !self.shader_errors.is_empty() {
                ui.colored_label(egui::Color32::RED, "Compilation Errors:");
                for error in &self.shader_errors {
                    ui.colored_label(egui::Color32::RED, format!("• {}", error));
                }
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // WGSL syntax highlighting (basic implementation)
                let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                    let mut job = egui::text::LayoutJob::default();
                    let mut is_comment = false;
                    let mut is_string = false;

                    for line in string.lines() {
                        let mut chars = line.chars().peekable();
                        let mut word_start = 0;

                        while let Some(ch) = chars.next() {
                            let pos = chars.clone().count();

                            // Handle comments
                            if ch == '/' && chars.peek() == Some(&'/') {
                                is_comment = true;
                            }

                            // Handle strings
                            if ch == '"' && !is_comment {
                                is_string = !is_string;
                            }

                            // Check for word boundaries
                            if ch.is_whitespace() || pos == 0 {
                                if word_start < pos {
                                    let word = &line[word_start..pos];
                                    let color = if is_comment {
                                        egui::Color32::GREEN
                                    } else if is_string {
                                        egui::Color32::YELLOW
                                    } else {
                                        self.get_wgsl_syntax_color(word)
                                    };
                                    job.append(word, 0.0, egui::TextFormat {
                                        font_id: egui::FontId::monospace(12.0),
                                        color,
                                        ..Default::default()
                                    });
                                }
                                word_start = pos;
                            }
                        }

                        // Handle end of line
                        if word_start < line.len() {
                            let word = &line[word_start..];
                            let color = if is_comment {
                                egui::Color32::GREEN
                            } else if is_string {
                                egui::Color32::YELLOW
                            } else {
                                self.get_wgsl_syntax_color(word)
                            };
                            job.append(word, 0.0, egui::TextFormat {
                                font_id: egui::FontId::monospace(12.0),
                                color,
                                ..Default::default()
                            });
                        }

                        job.append("\n", 0.0, egui::TextFormat {
                            font_id: egui::FontId::monospace(12.0),
                            color: egui::Color32::WHITE,
                            ..Default::default()
                        });

                        // Reset for next line
                        is_comment = false;
                        is_string = false;
                    }

                    ui.fonts(|f| f.layout_job(job))
                };

                ui.add(
                    egui::TextEdit::multiline(&mut self.current_wgsl_code)
                        .font(egui::TextStyle::Monospace)
                        .layouter(&mut layouter)
                        .desired_rows(20)
                        .lock_focus(true)
                );
            });
        });
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
        ui.collapsing("Live Preview", |ui| {
            // Preview controls
            ui.horizontal(|ui| {
                ui.label("Resolution:");
                let mut width = self.preview_size.0 as i32;
                let mut height = self.preview_size.1 as i32;
                if ui.add(egui::DragValue::new(&mut width).prefix("W: ").range(256..=2048)).changed() {
                    self.preview_size.0 = width as u32;
                }
                if ui.add(egui::DragValue::new(&mut height).prefix("H: ").range(256..=2048)).changed() {
                    self.preview_size.1 = height as u32;
                }
            });

            ui.horizontal(|ui| {
                ui.label(format!("Render FPS: {:.1}", self.render_fps));
                ui.separator();
                if ui.button("▶ Play").clicked() {
                    // Start preview rendering
                    self.compile_wgsl_shader();
                }
                if ui.button("⏸ Pause").clicked() {
                    // Pause preview
                }
                if ui.button("⏹ Stop").clicked() {
                    // Stop preview
                }
            });

            // Preview area
            let preview_size = egui::vec2(self.preview_size.0 as f32 * 0.5, self.preview_size.1 as f32 * 0.5);
            let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

            // Show compilation status (renderer integration pending)
            let color = match self.compilation_status {
                ShaderCompilationStatus::NotCompiled => egui::Color32::GRAY,
                ShaderCompilationStatus::Compiling => egui::Color32::YELLOW,
                ShaderCompilationStatus::Success => egui::Color32::GREEN,
                ShaderCompilationStatus::Error(_) => egui::Color32::RED,
            };

            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(4),
                color,
            );

            let status_text = match &self.compilation_status {
                ShaderCompilationStatus::NotCompiled => "Not Compiled",
                ShaderCompilationStatus::Compiling => "Compiling...",
                ShaderCompilationStatus::Success => "Ready to Render\n(WGPU Integration Pending)",
                ShaderCompilationStatus::Error(_) => "Compilation Error",
            };

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                status_text,
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );

            // Update render timing (placeholder)
            let now = Instant::now();
            let delta_time = now.duration_since(self.last_render_time).as_secs_f32();
            if delta_time >= 1.0 {
                self.render_fps = 1.0 / delta_time;
                self.last_render_time = now;
            }

            // Performance monitoring
            ui.separator();
            ui.label("Performance Monitoring:");
            ui.label(format!("Preview Size: {}x{}", self.preview_size.0, self.preview_size.1));
            ui.label(format!("Render FPS: {:.1}", self.render_fps));
            ui.label(format!("UI FPS: {:.1}", self.fps_counter));
        });
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
                ui.selectable_value(&mut self.selected_template_category, "Basic", "Basic");
                ui.selectable_value(&mut self.selected_template_category, "Animation", "Animation");
                ui.selectable_value(&mut self.selected_template_category, "Fractal", "Fractal");
                ui.selectable_value(&mut self.selected_template_category, "Effects", "Effects");
                ui.selectable_value(&mut self.selected_template_category, "Tutorial", "Tutorial");
                ui.selectable_value(&mut self.selected_template_category, "All", "All");
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                let templates_to_show = if self.selected_template_category == "All" {
                    &self.expanded_template_library
                } else {
                    &self.shader_templates.iter()
                        .chain(self.expanded_template_library.iter())
                        .filter(|t| t.category == self.selected_template_category)
                        .collect::<Vec<_>>()
                };

                for (i, template) in templates_to_show.iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_width(260.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&template.name).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Load").clicked() {
                                    self.load_expanded_template(template.clone());
                                }
                            });
                        });

                        ui.label(&template.description);
                        ui.small(format!("Category: {}", template.category));
                    });

                    ui.add_space(2.0);
                }

                if templates_to_show.is_empty() {
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
                // Shader conversion logic with multiple format support
                match (self.from_format.as_str(), self.to_format.as_str()) {
                    ("WGSL", "GLSL") => {
                        match crate::shader_converter::wgsl_to_glsl(&self.current_wgsl_code) {
                            Ok(glsl_code) => {
                                println!("WGSL to GLSL conversion successful");
                                self.shader_errors.clear();
                                self.shader_errors.push("GLSL conversion completed successfully".to_string());
                                // Could display converted code in a new panel
                            }
                            Err(e) => {
                                self.shader_errors.push(format!("GLSL conversion failed: {}", e));
                            }
                        }
                    }
                    ("WGSL", "HLSL") => {
                        match crate::shader_converter::wgsl_to_hlsl(&self.current_wgsl_code) {
                            Ok(hlsl_code) => {
                                println!("WGSL to HLSL conversion successful");
                                self.shader_errors.clear();
                                self.shader_errors.push("HLSL conversion completed successfully".to_string());
                            }
                            Err(e) => {
                                self.shader_errors.push(format!("HLSL conversion failed: {}", e));
                            }
                        }
                    }
                    ("GLSL", "WGSL") => {
                        // Basic GLSL to WGSL conversion (placeholder)
                        self.shader_errors.clear();
                        self.shader_errors.push("GLSL to WGSL conversion not yet implemented".to_string());
                    }
                    ("HLSL", "WGSL") => {
                        // Basic HLSL to WGSL conversion (placeholder)
                        self.shader_errors.clear();
                        self.shader_errors.push("HLSL to WGSL conversion not yet implemented".to_string());
                    }
                    _ => {
                        self.shader_errors.push("Conversion between selected formats not yet implemented".to_string());
                    }
                }
            }

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
                                    // For now, just show a float slider for color
                                    ui.add(egui::Slider::new(&mut new_value, 0.0..=1.0).text(""));
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
                    let spectrum = data.get_spectrum();
                    for (i, &amp) in spectrum.iter().enumerate().step_by(10) {
                        let height = (amp * 50.0) as usize;
                        ui.add(egui::ProgressBar::new(amp).show_percentage().animate(false));
                    }
                } else {
                    ui.label("Audio system not available");
                }
            } else {
                ui.label("Audio system not initialized");
                if ui.button("Initialize Audio").clicked() {
                    // Initialize audio system
                    self.audio_system = Some(Arc::new(Mutex::new(AudioMidiSystem::new())));
                }
            }
        });
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
                        ui.add(egui::DragValue::new(&mut cc_value).range(0..127));

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
                ui.label("Audio/MIDI system not initialized");
                if ui.button("Initialize Audio/MIDI").clicked() {
                    // Initialize audio system
                    self.audio_system = Some(Arc::new(Mutex::new(AudioMidiSystem::new())));
                }
            }
        });
    }

    fn render_performance_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Performance", |ui| {
            ui.label(format!("FPS: {:.1}", self.fps_counter));

            // Update FPS counter
            self.frame_count += 1;
            let now = std::time::Instant::now();
            if (now - self.last_fps_update).as_secs_f32() >= 1.0 {
                self.fps_counter = self.frame_count as f32;
                self.frame_count = 0;
                self.last_fps_update = now;
            }
        });
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

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(4, |columns| {
                // Left column - Tools and templates
                columns[0].vertical(|ui| {
                    ui.set_min_width(200.0);

                    if self.show_file_browser {
                        self.render_shader_browser(ui);
                    }

                    self.render_templates(ui);

                    if self.show_converter {
                        self.render_converter(ui);
                    }
                });

                // Middle-left column - Code editor and node editor
                columns[1].vertical(|ui| {
                    ui.set_min_width(400.0);

                    if self.show_code_editor {
                        self.render_code_editor(ui);
                    }

                    if self.show_node_editor {
                        ui.add_space(4.0);
                        self.render_node_editor(ui);
                    }
                });

                // Middle-right column - Live preview and parameters
                columns[2].vertical(|ui| {
                    ui.set_min_width(300.0);

                    if self.show_preview {
                        self.render_live_preview(ui);
                    }

                    self.render_parameter_panel(ui);
                });

                // Right column - Audio/MIDI and performance
                columns[3].vertical(|ui| {
                    ui.set_min_width(250.0);

                    if self.show_audio_panel {
                        self.render_audio_panel(ui);
                    }

                    if self.show_midi_panel {
                        self.render_midi_panel(ui);
                    }

                    self.render_performance_panel(ui);
                });
            });
        });

        // Update FPS counter
        self.frame_count += 1;
        let now = std::time::Instant::now();
        if (now - self.last_fps_update).as_secs_f32() >= 1.0 {
            self.fps_counter = self.frame_count as f32;
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