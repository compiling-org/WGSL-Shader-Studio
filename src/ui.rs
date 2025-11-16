#![cfg(feature = "eframe")] // DISABLED - EFAME ONLY - DO NOT USE
//! ISF Shader Editor UI using eframe + egui (LEGACY - DO NOT USE)
//!
//! This provides a standalone application for developing and testing ISF shaders
//! before deploying them as FFGL plugins.

use eframe::egui;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Main ISF Shader Editor Application
pub struct IsfShaderEditor {
    // ISF shader management
    pub isf_shaders: HashMap<String, crate::IsfShader>,
    pub selected_shader: Option<String>,
    pub shader_parameters: HashMap<String, f32>,

    // UI state
    pub show_shader_browser: bool,
    pub show_parameter_panel: bool,
    pub show_preview: bool,
    pub show_code_editor: bool,

    // Preview and rendering
    pub preview_texture: Option<egui::TextureHandle>,
    pub renderer: Option<crate::ShaderRenderer>,

    // Performance metrics
    pub fps: f32,
    pub frame_time: f64,

    // Async runtime for loading
    pub runtime: Arc<Runtime>,
}

impl IsfShaderEditor {
    pub fn new() -> Self {
        let runtime = Arc::new(Runtime::new().expect("Failed to create tokio runtime"));

        let mut app = Self {
            isf_shaders: HashMap::new(),
            selected_shader: None,
            shader_parameters: HashMap::new(),
            show_shader_browser: true,
            show_parameter_panel: true,
            show_preview: true,
            show_code_editor: false,
            preview_texture: None,
            renderer: None,
            fps: 0.0,
            frame_time: 0.0,
            runtime,
        };

        // Load ISF shaders in background
        app.load_isf_shaders();

        app
    }

    /// Load ISF shaders from the configured directories
    fn load_isf_shaders(&mut self) {
        let runtime = self.runtime.clone();
        let shaders = runtime.block_on(async {
            crate::isf_loader::load_resolume_isf_shaders()
        });

        match shaders {
            Ok(shader_list) => {
                for shader in shader_list {
                    // self.isf_shaders.insert(shader.name.clone(), shader);
                }
                println!("Loaded {} ISF shaders", self.isf_shaders.len());
            }
            Err(e) => {
                eprintln!("Failed to load ISF shaders: {}", e);
            }
        }
    }

    /// Initialize the WGPU renderer
    fn init_renderer(&mut self) {
        if self.renderer.is_none() {
            match Runtime::new() {
                Ok(rt) => {
                    self.renderer = rt.block_on(crate::ShaderRenderer::new()).ok();
                }
                Err(e) => eprintln!("Failed to create runtime for renderer: {}", e),
            }
        }
    }
}

impl eframe::App for IsfShaderEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update performance metrics
        self.frame_time = ctx.input(|i| i.time);

        // Set up dark theme for VJ tool
        self.setup_theme(ctx);

        // Main menu bar
        self.render_menu_bar(ctx);

        // Main layout
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_layout(ui);
        });

        // Request continuous repaint for smooth preview
        ctx.request_repaint();
    }
}

impl IsfShaderEditor {
    fn setup_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Professional VJ tool theme
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(25);
        style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_gray(200);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 120, 200);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 80, 140);

        // Dark theme
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = egui::Color32::from_gray(20);
        style.visuals.window_fill = egui::Color32::from_gray(30);
        style.visuals.faint_bg_color = egui::Color32::from_gray(15);

        ctx.set_style(style);
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🎨 ISF Shader Editor").size(16.0));

                ui.separator();

                ui.checkbox(&mut self.show_shader_browser, "Shader Browser");
                ui.checkbox(&mut self.show_parameter_panel, "Parameters");
                ui.checkbox(&mut self.show_preview, "Preview");
                ui.checkbox(&mut self.show_code_editor, "Code Editor");

                ui.separator();

                if ui.button("🔄 Reload Shaders").clicked() {
                    self.load_isf_shaders();
                }

                if ui.button("📁 Load ISF File").clicked() {
                    // TODO: File dialog for loading individual ISF files
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("FPS: {:.0}", self.fps));
                });
            });
        });
    }

    fn render_main_layout(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("main_layout")
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                // Left panel - Shader Browser
                if self.show_shader_browser {
                    ui.vertical(|ui| {
                        self.render_shader_browser(ui);
                    });
                    ui.add_space(4.0);
                }

                // Center panel - Preview
                if self.show_preview {
                    ui.vertical(|ui| {
                        self.render_preview(ui);
                    });
                    ui.add_space(4.0);
                }

                // Right panel - Parameters and Code
                ui.vertical(|ui| {
                    if self.show_parameter_panel {
                        self.render_parameters(ui);
                        ui.add_space(4.0);
                    }

                    if self.show_code_editor {
                        self.render_code_editor(ui);
                    }
                });
            });
    }

    fn render_shader_browser(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_min_width(280.0);

                ui.label(egui::RichText::new("🎨 ISF Shader Library").size(14.0));
                ui.separator();

                // Search/filter
                let mut filter = String::new();
                ui.text_edit_singleline(&mut filter);

                ui.add_space(4.0);

                // Shader list
                egui::ScrollArea::vertical()
                    .max_height(600.0)
                    .show(ui, |ui| {
                        for (name, shader) in &self.isf_shaders {
                            if !filter.is_empty() && !name.to_lowercase().contains(&filter.to_lowercase()) {
                                continue;
                            }

                            // let metadata = crate::isf_loader::get_shader_metadata(shader);

                            ui.group(|ui| {
                                ui.set_width(260.0);

                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(name).strong());
                                    if ui.button("👁").on_hover_text("Preview").clicked() {
                                        self.selected_shader = Some(name.clone());
                                        // self.init_renderer();
                                    }
                                });

                                let desc = "ISF shader description".to_string();
                                if !desc.is_empty() {
                                    ui.small(egui::RichText::new(desc).italics());
                                }

                                ui.small(format!("{} inputs, {} outputs",
                                    shader.inputs.len(),
                                    shader.outputs.len()
                                ));
                            });

                            ui.add_space(2.0);
                        }
                    });
            });
    }

    fn render_preview(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(400.0, 300.0));

                ui.label(egui::RichText::new("👁 Preview").size(14.0));
                ui.separator();

                if let Some(ref shader_name) = self.selected_shader {
                    if let Some(shader) = self.isf_shaders.get(shader_name) {
                        ui.label(format!("Playing: {}", shader_name));

                        // Preview area
                        let preview_size = egui::vec2(380.0, 250.0);
                        let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

                        // Placeholder for shader preview
                        ui.painter().rect_filled(
                            rect,
                            egui::Rounding::same(4),
                            egui::Color32::from_gray(50),
                        );

                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Shader Preview\n(Coming Soon)",
                            egui::FontId::proportional(16.0),
                            egui::Color32::WHITE,
                        );

                        // TODO: Implement actual shader rendering in preview
                    }
                } else {
                    ui.label("No shader selected");
                    ui.label("Choose a shader from the browser to preview");
                }
            });
    }

    fn render_parameters(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_min_width(250.0);

                ui.label(egui::RichText::new("🎛️ Parameters").size(14.0));
                ui.separator();

                if let Some(ref shader_name) = self.selected_shader {
                    if let Some(shader) = self.isf_shaders.get(shader_name) {
                        ui.label(format!("⚙️ {}", shader_name));
                        ui.separator();

                        for input in &shader.inputs {
                            let current_value = self.shader_parameters
                                .get(&input.name)
                                .copied()
                                .unwrap_or(input.default.unwrap_or(0.0));

                            let mut new_value = current_value;

                            match input.input_type {
                                crate::InputType::Float => {
                                    let min = input.min.unwrap_or(0.0);
                                    let max = input.max.unwrap_or(1.0);
                                    ui.add(egui::Slider::new(&mut new_value, min..=max)
                                        .text(&input.name));
                                }
                                crate::InputType::Bool => {
                                    let mut bool_value = current_value > 0.0;
                                    if ui.checkbox(&mut bool_value, &input.name).changed() {
                                        new_value = if bool_value { 1.0 } else { 0.0 };
                                    }
                                }
                                crate::InputType::Color => {
                                    ui.label(format!("{} (Color - Coming Soon)", input.name));
                                }
                                crate::InputType::Point2D => {
                                    ui.label(format!("{} (Point2D - Coming Soon)", input.name));
                                }
                                crate::InputType::Image => {
                                    ui.label(format!("{} (Image Input)", input.name));
                                }
                            }

                            if new_value != current_value {
                                self.shader_parameters.insert(input.name.clone(), new_value);
                            }
                        }

                        if shader.inputs.is_empty() {
                            ui.label("(No parameters)");
                        }
                    }
                } else {
                    ui.label("No shader selected");
                }

                ui.separator();

                // Performance info
                ui.label("📊 Performance");
                ui.label(format!("FPS: {:.0}", self.fps));
                ui.label(format!("Shaders: {}", self.isf_shaders.len()));
            });
    }

    fn render_code_editor(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_min_height(200.0);

                ui.label(egui::RichText::new("📝 Shader Code").size(14.0));
                ui.separator();

                if let Some(ref shader_name) = self.selected_shader {
                    if let Some(shader) = self.isf_shaders.get(shader_name) {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut shader.source.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .desired_rows(20)
                                        .interactive(false) // Read-only for now
                                );
                            });
                    }
                } else {
                    ui.label("Select a shader to view its code");
                }
            });
    }
}

/// Main entry point for the standalone application
#[cfg(feature = "eframe")]
pub fn main() -> Result<(), eframe::Error> {
    // Setup logging
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logging");

    // Application options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ISF Shader Editor - FFGL Development Tool")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_decorations(true)
            .with_resizable(true),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    // Create and run the application
    eframe::run_native(
        "ISF Shader Editor",
        options,
        Box::new(|_cc| {
            Box::new(IsfShaderEditor::new())
        }),
    )
}

#[cfg(not(feature = "eframe"))]
pub fn main() {
    println!("The 'eframe' feature is not enabled.");
    println!("Run with: cargo run --features eframe");
    println!("Or: cargo run --bin isf-shaders --features eframe");
}