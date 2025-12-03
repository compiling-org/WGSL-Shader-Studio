use bevy_egui::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::audio_system::AudioAnalyzer;
use crate::shader_renderer::{ShaderRenderer, RenderParameters};

pub struct UiStartupGate {
    pub frames: u64,
}

impl Default for UiStartupGate {
    fn default() -> Self { Self { frames: 0 } }
}

pub struct GlobalShaderRenderer {
    pub renderer: Arc<Mutex<Option<ShaderRenderer>>>,
}

impl Default for GlobalShaderRenderer {
    fn default() -> Self { Self { renderer: Arc::new(Mutex::new(None)) } }
}

#[derive(Clone, Debug)]
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

pub struct EditorUiState {
    pub dark_mode: bool,
    pub show_shader_browser: bool,
    pub show_parameter_panel: bool,
    pub show_preview: bool,
    pub show_code_editor: bool,
    pub show_node_studio: bool,
    pub show_timeline: bool,
    pub show_audio_panel: bool,
    pub show_midi_panel: bool,
    pub show_gesture_panel: bool,
    pub wgpu_initialized: bool,
    pub compilation_error: String,
    pub time: f64,
    pub draft_code: String,
    pub parameters: HashMap<String, f32>,
    pub global_renderer: GlobalShaderRenderer,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            dark_mode: true,
            show_shader_browser: true,
            show_parameter_panel: true,
            show_preview: true,
            show_code_editor: true,
            show_node_studio: false,
            show_timeline: false,
            show_audio_panel: false,
            show_midi_panel: false,
            show_gesture_panel: false,
            wgpu_initialized: false,
            compilation_error: String::new(),
            time: 0.0,
            draft_code: String::from("// WGSL
@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / vec2<f32>(800.0, 600.0);
    return vec4<f32>(uv.x, uv.y, 0.0, 1.0);
}"),
            parameters: HashMap::new(),
            global_renderer: GlobalShaderRenderer::default(),
        }
    }
}

impl EditorUiState {
    pub fn set_parameter_value(&mut self, name: &str, value: f32) {
        self.parameters.insert(name.to_string(), value);
    }
}

pub fn draw_editor_menu(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Shader Browser").clicked() { ui_state.show_shader_browser = !ui_state.show_shader_browser; }
            if ui.button("Parameters").clicked() { ui_state.show_parameter_panel = !ui_state.show_parameter_panel; }
            if ui.button("Preview").clicked() { ui_state.show_preview = !ui_state.show_preview; }
            if ui.button("Code Editor").clicked() { ui_state.show_code_editor = !ui_state.show_code_editor; }
            ui.toggle_value(&mut ui_state.dark_mode, "Dark Mode");
        });
    });
}

pub fn draw_editor_shader_browser_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::SidePanel::left("shader_browser_panel").show(ctx, |ui| {
        ui.heading("Available shaders:");
        let lock = ui_state.global_renderer.renderer.lock().unwrap();
        if let Some(ref renderer) = *lock {
            for ex in renderer.get_working_examples() {
                if ui.button(&ex.name).clicked() {
                    ui_state.draft_code = ex.wgsl_code.clone();
                }
            }
        } else {
            ui.label("Renderer not initialized yet");
        }
    });
}

pub fn draw_editor_parameter_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::SidePanel::right("parameter_panel").show(ctx, |ui| {
        ui.heading("Interactive shader parameters");
        let params = parse_shader_parameters(&ui_state.draft_code);
        for p in params {
            let mut val = *ui_state.parameters.get(&p.name).unwrap_or(&p.value);
            let min = p.min_value.unwrap_or(0.0);
            let max = p.max_value.unwrap_or(1.0);
            ui.label(format!("{}", p.name));
            ui.add(egui::Slider::new(&mut val, min..=max));
            ui_state.set_parameter_value(&p.name, val);
        }
    });
}

pub fn draw_editor_code_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::TopBottomPanel::bottom("code_editor_panel").default_height(200.0).show(ctx, |ui| {
        ui.heading("Code Editor");
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.text_edit_multiline(&mut ui_state.draft_code);
        });
    });
}

pub fn draw_editor_central_panel(
    ctx: &egui::Context,
    ui_state: &mut EditorUiState,
    audio_analyzer: &AudioAnalyzer,
    _video_exporter: Option<&crate::screenshot_video_export::ScreenshotVideoExporter>,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Live shader preview");
        let size = ui.available_size();

        let params_map = ui_state.parameters.clone();
        let guard = ui_state.global_renderer.renderer.lock().unwrap();
        if let Some(ref renderer) = *guard {
            let mut renderer = renderer.clone();
            let render_params = RenderParameters {
                width: size.x as u32,
                height: size.y as u32,
                time: ui_state.time as f32,
                frame_rate: 60.0,
                audio_data: Some(audio_analyzer.get_audio_data()),
            };

            match renderer.render_frame(&ui_state.draft_code, &render_params, render_params.audio_data.clone()) {
                Ok(pixels) => {
                    let tex = ctx.load_texture(
                        "shader_preview",
                        egui::ColorImage {
                            size: [render_params.width as usize, render_params.height as usize],
                            pixels: pixels.chunks(4).map(|c| egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])).collect(),
                            source_size: size,
                        },
                        egui::TextureOptions::default(),
                    );
                    ui.image(egui::load::SizedTexture::from_handle(tex, size));
                }
                Err(e) => {
                    ui_state.compilation_error = e.to_string();
                    ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", ui_state.compilation_error));
                }
            }
        } else {
            ui.label("WGPU renderer not initialized yet");
        }
    });
}

pub fn parse_shader_parameters(shader_code: &str) -> Vec<ShaderParameter> {
    let mut parameters = Vec::new();

    // Uniforms: @group(X) @binding(Y) var<uniform> name : Type;
    let uniform_pattern = r"@group\(\s*(\d+)\s*\)\s*@binding\(\s*(\d+)\s*\)\s*var<uniform>\s*(\w+)\s*:\s*([^;]+);";
    let uniform_regex = regex::Regex::new(uniform_pattern).unwrap_or_else(|_| regex::Regex::new("^$").unwrap());
    for cap in uniform_regex.captures_iter(shader_code) {
        let group = cap[1].parse::<u32>().unwrap_or(0);
        let binding = cap[2].parse::<u32>().unwrap_or(0);
        let name = cap[3].to_string();
        let wgsl_type = cap[4].trim().to_string();
        parameters.push(ShaderParameter { name, wgsl_type, group, binding, value: 0.5, default_value: None, min_value: None, max_value: None });
    }

    // Textures: @group(X) @binding(Y) var name : texture_*;
    let texture_pattern = r"@group\(\s*(\d+)\s*\)\s*@binding\(\s*(\d+)\s*\)\s*var\s+(\w+)\s*:\s*texture_(\w+);";
    let texture_regex = regex::Regex::new(texture_pattern).unwrap_or_else(|_| regex::Regex::new("^$").unwrap());
    for cap in texture_regex.captures_iter(shader_code) {
        let group = cap[1].parse::<u32>().unwrap_or(0);
        let binding = cap[2].parse::<u32>().unwrap_or(0);
        let name = cap[3].to_string();
        parameters.push(ShaderParameter { name, wgsl_type: "texture".to_string(), group, binding, value: 0.5, default_value: None, min_value: None, max_value: None });
    }

    parameters
}

