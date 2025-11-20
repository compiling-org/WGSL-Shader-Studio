use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::shader_renderer::{ShaderRenderer, RenderParameters, WorkingShaderExample};
use crate::audio::AudioData;

/// Complete shader playground system with live preview and parameter controls
pub struct ShaderPlaygroundPlugin;

impl Plugin for ShaderPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ShaderPlaygroundState>()
            .add_systems(Update, (
                shader_playground_ui,
                update_shader_preview,
                handle_shader_compilation,
            ));
    }
}

/// State management for the shader playground
#[derive(Resource, Default)]
pub struct ShaderPlaygroundState {
    pub current_shader_code: String,
    pub current_shader_name: String,
    pub is_compiling: bool,
    pub last_error: Option<String>,
    pub preview_texture_id: Option<u64>,
    pub preview_pixels: Vec<u8>,
    pub preview_size: (u32, u32),
    pub parameter_values: Vec<f32>,
    pub parameter_names: Vec<String>,
    pub show_parameter_panel: bool,
    pub show_shader_browser: bool,
    pub show_code_editor: bool,
    pub auto_compile: bool,
    pub frame_count: u64,
    pub time: f32,
    pub selected_example: Option<usize>,
    pub custom_shaders: Vec<WorkingShaderExample>,
    pub renderer: Option<Arc<Mutex<ShaderRenderer>>>,
}

impl ShaderPlaygroundState {
    pub fn new() -> Self {
        let mut state = Self::default();
        state.preview_size = (512, 512);
        state.parameter_values = vec![0.0; 64];
        state.parameter_names = (0..64).map(|i| format!("param_{}", i)).collect();
        state.auto_compile = true;
        state.show_parameter_panel = true;
        state.show_shader_browser = true;
        state.show_code_editor = true;
        state.current_shader_name = "Animated Gradient".to_string();
        state
    }

    pub async fn initialize_renderer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let renderer = ShaderRenderer::new_with_size(self.preview_size).await?;
        self.renderer = Some(Arc::new(Mutex::new(renderer)));
        
        // Load initial example shader
        if let Some(renderer_ref) = &self.renderer {
            let renderer = renderer_ref.lock().await;
            let examples = renderer.get_working_examples();
            if !examples.is_empty() {
                self.current_shader_code = examples[0].wgsl_code.clone();
                self.current_shader_name = examples[0].name.clone();
                self.selected_example = Some(0);
            }
        }
        
        Ok(())
    }

    pub async fn compile_current_shader(&mut self) -> Result<Vec<u8>, String> {
        if self.current_shader_code.trim().is_empty() {
            return Err("Shader code is empty".to_string());
        }

        if let Some(renderer_ref) = &self.renderer {
            let mut renderer = renderer_ref.lock().await;
            
            let params = RenderParameters {
                width: self.preview_size.0,
                height: self.preview_size.1,
                time: self.time,
                frame_rate: 60.0,
                audio_data: None, // Will be integrated later
            };

            match renderer.render_frame_with_params(&self.current_shader_code, &params, Some(&self.parameter_values)).await {
                Ok(pixels) => {
                    self.last_error = None;
                    self.preview_pixels = pixels.clone();
                    Ok(pixels)
                }
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    self.last_error = Some(error_msg.clone());
                    Err(error_msg)
                }
            }
        } else {
            Err("Renderer not initialized".to_string())
        }
    }

    pub fn load_example_shader(&mut self, example: &WorkingShaderExample) {
        self.current_shader_code = example.wgsl_code.clone();
        self.current_shader_name = example.name.clone();
        self.last_error = None;
    }

    pub fn add_custom_shader(&mut self, name: String, description: String, code: String) {
        let example = WorkingShaderExample {
            name: name.clone(),
            description,
            wgsl_code: code,
            category: "Custom".to_string(),
        };
        self.custom_shaders.push(example);
    }
}

/// Main UI system for the shader playground
fn shader_playground_ui(
    mut contexts: EguiContexts,
    mut playground_state: ResMut<ShaderPlaygroundState>,
) {
    let ctx = contexts.ctx_mut();
    
    egui::Window::new("Shader Playground")
        .default_size([1200.0, 800.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Shader").clicked() {
                        playground_state.current_shader_code = "@fragment\nfn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n}".to_string();
                        playground_state.current_shader_name = "New Shader".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Save Shader").clicked() {
                        playground_state.add_custom_shader(
                            playground_state.current_shader_name.clone(),
                            "Custom shader".to_string(),
                            playground_state.current_shader_code.clone(),
                        );
                        ui.close_menu();
                    }
                });
                
                ui.separator();
                
                if ui.button("Compile").clicked() {
                    playground_state.is_compiling = true;
                }
                
                ui.checkbox(&mut playground_state.auto_compile, "Auto Compile");
                
                ui.separator();
                
                ui.label(format!("Frame: {}", playground_state.frame_count));
                ui.label(format!("Time: {:.2}s", playground_state.time));
            });

            ui.separator();

            // Main layout
            ui.horizontal(|ui| {
                // Left panel: Shader browser and examples
                ui.vertical(|ui| {
                    ui.collapsing("Shader Browser", |ui| {
                        if let Some(renderer_ref) = &playground_state.renderer {
                            if let Ok(renderer) = renderer_ref.try_lock() {
                                let examples = renderer.get_working_examples();
                                
                                // Group examples by category
                                let mut categories: std::collections::HashMap<String, Vec<&WorkingShaderExample>> = std::collections::HashMap::new();
                                for example in examples {
                                    categories.entry(example.category.clone()).or_default().push(example);
                                }
                                
                                // Display examples by category
                                for (category, category_examples) in categories {
                                    ui.collapsing(&category, |ui| {
                                        for example in category_examples {
                                            ui.horizontal(|ui| {
                                                if ui.button(&example.name).clicked() {
                                                    playground_state.load_example_shader(example);
                                                }
                                                ui.label(&example.description);
                                            });
                                        }
                                    });
                                }
                                
                                // Show custom shaders
                                if !playground_state.custom_shaders.is_empty() {
                                    ui.separator();
                                    ui.collapsing("Custom Shaders", |ui| {
                                        for example in &playground_state.custom_shaders {
                                            ui.horizontal(|ui| {
                                                if ui.button(&example.name).clicked() {
                                                    playground_state.load_example_shader(example);
                                                }
                                                ui.label(&example.description);
                                            });
                                        }
                                    });
                                }
                            }
                        }
                    });
                });

                ui.separator();

                // Center panel: Code editor and preview
                ui.vertical(|ui| {
                    // Code editor
                    ui.collapsing("Code Editor", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Shader Name:");
                            ui.text_edit_singleline(&mut playground_state.current_shader_name);
                        });
                        
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut playground_state.current_shader_code)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(600.0)
                                .desired_rows(20)
                        );
                        
                        if playground_state.auto_compile && response.changed() {
                            playground_state.is_compiling = true;
                        }
                    });

                    ui.separator();

                    // Preview
                    ui.collapsing("Preview", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Preview Size:");
                            if ui.button("256x256").clicked() {
                                playground_state.preview_size = (256, 256);
                            }
                            if ui.button("512x512").clicked() {
                                playground_state.preview_size = (512, 512);
                            }
                            if ui.button("1024x1024").clicked() {
                                playground_state.preview_size = (1024, 1024);
                            }
                        });

                        // Display preview texture
                        let preview_size = egui::vec2(400.0, 400.0);
                        let (response, painter) = ui.allocate_painter(preview_size, egui::Sense::hover());
                        
                        if !playground_state.preview_pixels.is_empty() {
                            // Convert pixels to texture
                            let texture = contexts.add_image(egui::ColorImage::from_rgba_unmultiplied(
                                [playground_state.preview_size.0 as usize, playground_state.preview_size.1 as usize],
                                &playground_state.preview_pixels
                            ));
                            
                            let rect = response.rect;
                            painter.image(
                                texture.id(),
                                rect,
                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                egui::Color32::WHITE,
                            );
                        } else {
                            painter.rect_filled(
                                response.rect,
                                0.0,
                                egui::Color32::from_gray(50),
                            );
                            painter.text(
                                response.rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "No Preview Available",
                                egui::FontId::proportional(16.0),
                                egui::Color32::WHITE,
                            );
                        }
                    });
                });

                ui.separator();

                // Right panel: Parameters and controls
                ui.vertical(|ui| {
                    ui.collapsing("Parameters", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Parameter Count:");
                            ui.label(format!("{}", playground_state.parameter_values.len()));
                        });
                        
                        // Show first 8 parameters as sliders
                        for i in 0..8.min(playground_state.parameter_values.len()) {
                            ui.horizontal(|ui| {
                                ui.label(&playground_state.parameter_names[i]);
                                ui.add(egui::Slider::new(&mut playground_state.parameter_values[i], 0.0..=1.0));
                            });
                        }
                        
                        if playground_state.parameter_values.len() > 8 {
                            ui.label(format!("... and {} more parameters", playground_state.parameter_values.len() - 8));
                        }
                    });

                    ui.separator();

                    // Error display
                    if let Some(error) = &playground_state.last_error {
                        ui.collapsing("Errors", |ui| {
                            ui.colored_label(egui::Color32::RED, error);
                        });
                    }

                    ui.separator();

                    // Controls
                    ui.collapsing("Controls", |ui| {
                        if ui.button("Reset Time").clicked() {
                            playground_state.time = 0.0;
                            playground_state.frame_count = 0;
                        }
                        
                        if ui.button("Export Image").clicked() {
                            // TODO: Implement image export
                        }
                        
                        if ui.button("Export Video").clicked() {
                            // TODO: Implement video export
                        }
                    });
                });
            });
        });
}

/// System to update shader preview with animation
fn update_shader_preview(
    time: Res<Time>,
    mut playground_state: ResMut<ShaderPlaygroundState>,
) {
    playground_state.time += time.delta_seconds();
    playground_state.frame_count += 1;
    
    // Auto-compile if enabled and not currently compiling
    if playground_state.auto_compile && !playground_state.is_compiling {
        // Check if enough time has passed since last compile
        // This prevents too frequent compilation
        if playground_state.frame_count % 30 == 0 { // Compile every 30 frames
            playground_state.is_compiling = true;
        }
    }
}

/// System to handle shader compilation asynchronously
fn handle_shader_compilation(
    mut playground_state: ResMut<ShaderPlaygroundState>,
) {
    if playground_state.is_compiling {
        // In a real implementation, this would use async/await or a task system
        // For now, we'll do a simple synchronous compilation
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        match runtime.block_on(playground_state.compile_current_shader()) {
            Ok(_pixels) => {
                // Preview was updated in compile_current_shader
            }
            Err(error) => {
                // Error was already set in compile_current_shader
                println!("Shader compilation error: {}", error);
            }
        }
        
        playground_state.is_compiling = false;
    }
}

/// Extension methods for ShaderPlaygroundState
impl ShaderPlaygroundState {
    pub fn get_current_shader_info(&self) -> (String, String) {
        (self.current_shader_name.clone(), self.current_shader_code.clone())
    }
    
    pub fn set_shader_code(&mut self, name: String, code: String) {
        self.current_shader_name = name;
        self.current_shader_code = code;
        if self.auto_compile {
            self.is_compiling = true;
        }
    }
    
    pub fn get_preview_data(&self) -> Option<(&[u8], (u32, u32))> {
        if self.preview_pixels.is_empty() {
            None
        } else {
            Some((&self.preview_pixels, self.preview_size))
        }
    }
    
    pub fn is_ready(&self) -> bool {
        self.renderer.is_some()
    }
}