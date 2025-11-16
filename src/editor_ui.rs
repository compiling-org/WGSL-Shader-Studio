use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::fs;
use std::sync::Arc;
use crate::node_graph::NodeGraph;
use crate::timeline::{Timeline, PlaybackState};
use crate::shader_renderer::ShaderRenderer;
use crate::audio::AudioAnalyzer;
// use crate::shader_browser::ShaderBrowser;
use crate::wgsl_diagnostics::WgslDiagnostics;
use std::time::{Instant, Duration};

// Temporarily define a placeholder until we fix the import
#[derive(Default)]
pub struct VisualNodeEditor {
    last_generation_time: Option<std::time::Instant>,
    needs_compilation: bool,
}

impl VisualNodeEditor {
    pub fn new() -> Self {
        Self {
            last_generation_time: None,
            needs_compilation: false,
        }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui, node_graph: &mut crate::node_graph::NodeGraph) {
        // Enhanced placeholder with basic node visualization
        ui.label("Visual Node Editor - Placeholder Implementation");
        ui.separator();
        
        // Show basic node graph info
        ui.group(|ui| {
            ui.label("Node Graph Status:");
            ui.label(format!("  Total Nodes: {}", node_graph.nodes.len()));
            ui.label(format!("  Connections: {}", node_graph.connections.len()));
            
            if let Some(last_gen) = self.last_generation_time {
                ui.label(format!("  Last Generated: {:.1}s ago", last_gen.elapsed().as_secs_f32()));
            } else {
                ui.label("  Last Generated: Never");
            }
            
            if self.needs_compilation {
                ui.label(egui::RichText::new("  Status: Needs Compilation").color(egui::Color32::YELLOW));
            }
        });
        
        ui.separator();
        
        // Simple node creation buttons
        ui.horizontal(|ui| {
            if ui.button("+ Input").clicked() {
                // Add input node placeholder
                self.needs_compilation = true;
            }
            if ui.button("+ Math").clicked() {
                // Add math node placeholder
                self.needs_compilation = true;
            }
            if ui.button("+ Output").clicked() {
                // Add output node placeholder
                self.needs_compilation = true;
            }
        });
        
        // Node list
        if !node_graph.nodes.is_empty() {
            ui.separator();
            ui.label("Nodes:");
            for (i, node) in node_graph.nodes.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: {}", i + 1, node.node_type));
                    if ui.small_button("×").clicked() {
                        // Remove node placeholder
                        self.needs_compilation = true;
                    }
                });
            }
        }
    }
    
    pub fn generate_and_compile(&mut self, node_graph: &crate::node_graph::NodeGraph, _width: u32, _height: u32) -> Result<String, Vec<String>> {
        // Generate WGSL from node graph
        let wgsl_code = node_graph.generate_wgsl(_width, _height);
        self.last_generation_time = Some(std::time::Instant::now());
        self.needs_compilation = false;
        
        match wgsl_code {
            Ok(code) => Ok(code),
            Err(e) => Err(vec![format!("Node graph generation failed: {}", e)]),
        }
    }
    
    pub fn auto_compile_if_needed(&mut self, node_graph: &crate::node_graph::NodeGraph, _width: u32, _height: u32) -> Option<Result<String, Vec<String>>> {
        // Auto-compile if needed (every 2 seconds or when flagged)
        let should_compile = self.needs_compilation || 
            self.last_generation_time.map(|t| t.elapsed().as_secs() > 2).unwrap_or(true);
            
        if should_compile {
            Some(self.generate_and_compile(node_graph, _width, _height))
        } else {
            None
        }
    }
}

use std::sync::Mutex;

#[derive(Resource)]
pub struct GlobalRenderer {
    pub renderer: Arc<Mutex<Option<ShaderRenderer>>>,
}

impl Default for GlobalRenderer {
    fn default() -> Self {
        Self {
            renderer: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Resource)]
pub struct EditorState {
    pub code_editor: String,
    pub shader_params: Vec<ShaderParameter>,
    pub node_graph: NodeGraph,
    pub visual_node_editor: VisualNodeEditor,
    pub timeline: Timeline,
    pub audio_analyzer: AudioAnalyzer,
    // pub shader_browser: ShaderBrowser,
    pub diagnostics: WgslDiagnostics,
    pub preview_size: (u32, u32),
    pub show_node_editor: bool,
    pub show_timeline: bool,
    pub show_shader_browser: bool,
    pub show_error_panel: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub auto_compile: bool,
    pub last_compiled_code: Option<String>,
    pub compilation_time: f32,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            code_editor: include_str!("../shaders/default.wgsl").to_string(),
            shader_params: vec![],
            node_graph: NodeGraph::new(),
            visual_node_editor: VisualNodeEditor::new(),
            timeline: Timeline::new(),
            audio_analyzer: AudioAnalyzer::new(),
            // shader_browser: ShaderBrowser::new(),
            diagnostics: WgslDiagnostics::new(),
            preview_size: (512, 512),
            show_node_editor: true,
            show_timeline: true,
            show_shader_browser: true,
            show_error_panel: true,
            errors: vec![],
            warnings: vec![],
            auto_compile: true,
            last_compiled_code: None,
            compilation_time: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShaderParameter {
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub binding: u32,
    pub group: u32,
}

pub fn editor_ui_system(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    global_renderer: Res<GlobalRenderer>,
) {
    let ctx = contexts.ctx_mut().expect("Failed to get egui context");
    
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    editor_state.code_editor = include_str!("../shaders/default.wgsl").to_string();
                    editor_state.node_graph = NodeGraph::new();
                    editor_state.errors.clear();
                    editor_state.warnings.clear();
                }
                if ui.button("Open").clicked() {
                    if let Ok(content) = fs::read_to_string("shader.wgsl") {
                        editor_state.code_editor = content;
                    }
                }
                if ui.button("Save").clicked() {
                    let _ = fs::write("shader.wgsl", &editor_state.code_editor);
                }
                if ui.button("Export").clicked() {
                    let _ = fs::write("exported_shader.wgsl", &editor_state.code_editor);
                }
            });
            
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut editor_state.show_node_editor, "Node Editor");
                ui.checkbox(&mut editor_state.show_timeline, "Timeline");
                ui.checkbox(&mut editor_state.show_shader_browser, "Shader Browser");
                ui.checkbox(&mut editor_state.show_error_panel, "Error Panel");
            });
            
            ui.separator();
            
            ui.checkbox(&mut editor_state.auto_compile, "Auto Compile");
            
            if ui.button("Compile").clicked() {
                compile_and_render_shader(&mut editor_state, &global_renderer);
            }
            
            // Compilation status indicator
            ui.separator();
            if !editor_state.errors.is_empty() {
                ui.label(egui::RichText::new(format!("❌ {} errors", editor_state.errors.len())).color(egui::Color32::RED));
            } else if !editor_state.warnings.is_empty() {
                ui.label(egui::RichText::new(format!("⚠️ {} warnings", editor_state.warnings.len())).color(egui::Color32::YELLOW));
            } else if editor_state.last_compiled_code.is_some() {
                ui.label(egui::RichText::new(format!("✅ Compiled ({:.2}s)", editor_state.compilation_time)).color(egui::Color32::GREEN));
            } else {
                ui.label(egui::RichText::new("🔄 Ready").color(egui::Color32::GRAY));
            }
            
            // Show compilation time
            if editor_state.compilation_time > 0.0 {
                ui.label(format!("{:.3}s", editor_state.compilation_time));
            }
        });
    });

    egui::SidePanel::left("shader_browser").default_width(300.0).show_animated(ctx, editor_state.show_shader_browser, |ui| {
        ui.heading("Shader Browser");
        ui.separator();
        
        // Search bar
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut editor_state.shader_browser.search_query);
        });
        
        // Category filter
        ui.horizontal(|ui| {
            ui.label("Filter:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", editor_state.shader_browser.filter_category.unwrap_or(crate::shader_browser::ShaderCategory::Unknown)))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, None, "All");
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, Some(crate::shader_browser::ShaderCategory::WGSL), "WGSL");
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, Some(crate::shader_browser::ShaderCategory::ISF), "ISF");
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, Some(crate::shader_browser::ShaderCategory::GLSL), "GLSL");
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, Some(crate::shader_browser::ShaderCategory::HLSL), "HLSL");
                });
        });
        
        ui.separator();
        
        // Quick actions
        ui.horizontal(|ui| {
            if ui.button("Load Default").clicked() {
                editor_state.code_editor = include_str!("../shaders/default.wgsl").to_string();
                if editor_state.auto_compile {
                    compile_and_render_shader(&mut editor_state, &global_renderer);
                }
            }
            if ui.button("Scan Directory").clicked() {
                editor_state.shader_browser.scan_directories();
            }
        });
        
        ui.separator();
        
        // Recent files
        ui.collapsing("Recent Files", |ui| {
            for path in &editor_state.shader_browser.recent_files.clone() {
                ui.horizontal(|ui| {
                    if ui.button(path.file_name().unwrap_or_default().to_string_lossy()).clicked() {
                        if let Ok(content) = editor_state.shader_browser.load_shader(path) {
                            editor_state.code_editor = content;
                            if editor_state.auto_compile {
                                compile_and_render_shader(&mut editor_state, &global_renderer);
                            }
                        }
                    }
                    if ui.small_button("★").clicked() {
                        if !editor_state.shader_browser.favorites.contains(path) {
                            editor_state.shader_browser.favorites.push(path.clone());
                        }
                    }
                });
            }
        });
        
        // Favorites
        ui.collapsing("Favorites", |ui| {
            for path in &editor_state.shader_browser.favorites.clone() {
                ui.horizontal(|ui| {
                    if ui.small_button("☆").clicked() {
                        editor_state.shader_browser.favorites.retain(|p| p != path);
                    }
                    if ui.button(path.file_name().unwrap_or_default().to_string_lossy()).clicked() {
                        if let Ok(content) = editor_state.shader_browser.load_shader(path) {
                            editor_state.code_editor = content;
                            if editor_state.auto_compile {
                                compile_and_render_shader(&mut editor_state, &global_renderer);
                            }
                        }
                    }
                });
            }
        });
        
        // Search results
        if !editor_state.shader_browser.search_query.is_empty() {
            ui.separator();
            ui.label("Search Results:");
            
            let results = editor_state.shader_browser.search_shaders(&editor_state.shader_browser.search_query.clone());
            for shader in results {
                ui.horizontal(|ui| {
                    ui.label(&shader.name);
                    ui.label(format!("{:?}", shader.category));
                    if ui.button("Load").clicked() {
                        if let Ok(content) = editor_state.shader_browser.load_shader(&shader.path) {
                            editor_state.code_editor = content;
                            if editor_state.auto_compile {
                                compile_and_render_shader(&mut editor_state, &global_renderer);
                            }
                        }
                    }
                    if ui.small_button("★").clicked() {
                        if !editor_state.shader_browser.favorites.contains(&shader.path) {
                            editor_state.shader_browser.favorites.push(shader.path.clone());
                        }
                    }
                });
            }
        }
        
        ui.separator();
        
        // Parameters panel (moved from original location)
        ui.label("Parameters");
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for param in &mut editor_state.shader_params {
                ui.horizontal(|ui| {
                    ui.label(&param.name);
                    let response = ui.add(egui::DragValue::new(&mut param.value)
                        .speed(0.01)
                        .clamp_range(param.min..=param.max));
                    
                    // Update parameters in real-time when values change
                    if response.changed() {
                        // Prepare parameter values array
                        let mut param_values = vec![0.0f32; 64]; // Max 64 parameters
                        for (i, p) in editor_state.shader_params.iter().enumerate() {
                            if i < 64 {
                                param_values[i] = p.value;
                            }
                        }
                        
                        // Update renderer parameters
                        if let Ok(mut renderer) = global_renderer.renderer.lock() {
                            if let Some(renderer) = renderer.as_mut() {
                                let _ = renderer.update_parameters(&param_values);
                                
                                // Re-render with new parameters
                                if editor_state.auto_compile {
                                    compile_and_render_shader(editor_state, global_renderer);
                                }
                            }
                        }
                    }
                });
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            if editor_state.show_node_editor {
                // Node Editor Panel
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Node Editor");
                        ui.separator();
                        if ui.button("Generate WGSL").clicked() {
                            // Generate WGSL from node graph
                            match editor_state.visual_node_editor.generate_and_compile(
                                &mut editor_state.node_graph, 
                                editor_state.preview_size.0, 
                                editor_state.preview_size.1
                            ) {
                                Ok(wgsl_code) => {
                                    editor_state.code_editor = wgsl_code;
                                    compile_and_render_shader(&mut editor_state, &global_renderer);
                                }
                                Err(errors) => {
                                    editor_state.errors = errors;
                                }
                            }
                        }
                        if ui.button("Clear Graph").clicked() {
                            editor_state.node_graph = NodeGraph::new();
                        }
                    });
                    
                    // Show node graph info
                    ui.separator();
                    ui.label(format!("Nodes: {}", editor_state.node_graph.nodes.len()));
                    ui.label(format!("Connections: {}", editor_state.node_graph.connections.len()));
                    
                    // Call the visual node editor UI
                    editor_state.visual_node_editor.ui(ui, &mut editor_state.node_graph);
                    
                    // Auto-compile if enabled and node graph changed
                    if editor_state.auto_compile {
                        if let Some(result) = editor_state.visual_node_editor.auto_compile_if_needed(
                            &mut editor_state.node_graph,
                            editor_state.preview_size.0,
                            editor_state.preview_size.1
                        ) {
                            match result {
                                Ok(wgsl_code) => {
                                    if editor_state.code_editor != wgsl_code {
                                        editor_state.code_editor = wgsl_code;
                                        compile_and_render_shader(&mut editor_state, &global_renderer);
                                    }
                                }
                                Err(errors) => {
                                    editor_state.errors = errors;
                                }
                            }
                        }
                    }
                });
            }
            
            // Code Editor Panel
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("WGSL Code Editor");
                    ui.separator();
                    if ui.button("Format").clicked() {
                        editor_state.code_editor = format_wgsl_code(&editor_state.code_editor);
                    }
                    if ui.button("Clear").clicked() {
                        editor_state.code_editor.clear();
                    }
                });
                
                // Create a custom text editor with line numbers and error highlighting
                let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                let row_height = ui.text_style_height(&font_id);
                let desired_width = ui.available_width();
                
                // Calculate line numbers width
                let line_count = editor_state.code_editor.lines().count().max(1);
                let line_number_width = format!("{}", line_count).len() as f32 * 8.0;
                
                ui.horizontal(|ui| {
                    // Line numbers column
                    ui.vertical(|ui| {
                        ui.set_width(line_number_width);
                        for i in 1..=line_count {
                            // Check if this line has errors
                            let has_error = editor_state.errors.iter().any(|error| {
                                error.contains(&format!("line {}", i)) || 
                                error.contains(&format!("Line {}", i))
                            });
                            
                            let has_warning = editor_state.warnings.iter().any(|warning| {
                                warning.contains(&format!("line {}", i)) || 
                                warning.contains(&format!("Line {}", i))
                            });
                            
                            let line_text = format!("{:>3} ", i);
                            let text_color = if has_error {
                                egui::Color32::RED
                            } else if has_warning {
                                egui::Color32::YELLOW
                            } else {
                                ui.style().visuals.text_color()
                            };
                            
                            ui.label(egui::RichText::new(line_text).color(text_color).monospace());
                        }
                    });
                    
                    // Code editor
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut editor_state.code_editor)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(20)
                            .desired_width(desired_width - line_number_width)
                            .lock_focus(true)
                    );
                    
                    if response.changed() && editor_state.auto_compile {
                        compile_and_render_shader(&mut editor_state, &global_renderer);
                    }
                });
            });
            
            // Preview Panel
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Preview");
                    ui.separator();
                    ui.label(format!("Size: {}x{}", editor_state.preview_size.0, editor_state.preview_size.1));
                    if ui.button("512x512").clicked() {
                        editor_state.preview_size = (512, 512);
                    }
                    if ui.button("1024x1024").clicked() {
                        editor_state.preview_size = (1024, 1024);
                    }
                });
                
                // Display preview texture
                if let Ok(renderer) = global_renderer.renderer.lock() {
                    if let Some(renderer) = renderer.as_ref() {
                        if let Some(texture) = renderer.get_preview_texture() {
                            // For now, just show a placeholder
                            ui.label("Preview: Ready");
                        } else {
                            ui.label("No preview available");
                        }
                    }
                }
            });
        });
    });

    if editor_state.show_timeline {
        egui::TopBottomPanel::bottom("timeline").default_height(200.0).show(ctx, |ui| {
            ui.heading("Timeline");
            ui.separator();
            
            ui.horizontal(|ui| {
                if ui.button("▶").clicked() {
                    editor_state.timeline.playback_state = PlaybackState::Playing;
                }
                if ui.button("⏸").clicked() {
                    editor_state.timeline.playback_state = PlaybackState::Paused;
                }
                if ui.button("⏹").clicked() {
                    editor_state.timeline.playback_state = PlaybackState::Stopped;
                    editor_state.timeline.current_time = 0.0;
                }
                
                ui.separator();
                ui.label(format!("Time: {:.2}s", editor_state.timeline.current_time));
                ui.label(format!("Duration: {:.2}s", editor_state.timeline.duration));
            });
            
            // Timeline scrubber
            ui.horizontal(|ui| {
                ui.label("Time:");
                let duration = editor_state.timeline.duration;
                ui.add(egui::DragValue::new(&mut editor_state.timeline.current_time)
                    .speed(0.1)
                    .clamp_range(0.0..=duration));
            });
            
            // Keyframe editor
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (param_name, keyframes) in &editor_state.timeline.get_all_keyframes() {
                        ui.group(|ui| {
                            ui.label(param_name);
                            for keyframe in keyframes.iter() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:.2}s", keyframe.time));
                                    ui.label(format!("{:.3}", keyframe.value));
                                    ui.label(format!("{:?}", keyframe.interpolation));
                                });
                            }
                        });
                    }
                });
            });
        });
    }

    if editor_state.show_error_panel {
        egui::TopBottomPanel::bottom("errors").default_height(150.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Errors & Warnings");
                ui.separator();
                if ui.button("Clear").clicked() {
                    editor_state.errors.clear();
                    editor_state.warnings.clear();
                }
            });
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Group errors by severity
                if !editor_state.errors.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("ERRORS").color(egui::Color32::RED).strong());
                        ui.label(format!("({})", editor_state.errors.len()));
                    });
                    
                    for (i, error) in editor_state.errors.iter().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("❌").color(egui::Color32::RED));
                                ui.vertical(|ui| {
                                    // Parse line number if present
                                    if let Some(line_pos) = error.find("line ") {
                                        let line_start = line_pos + 5;
                                        if let Some(line_end) = error[line_start..].find(|c: char| !c.is_numeric()) {
                                            let line_num = &error[line_start..line_start + line_end];
                                            ui.label(egui::RichText::new(format!("Line {}: ", line_num)).color(egui::Color32::RED).strong());
                                        }
                                    }
                                    ui.label(error);
                                });
                            });
                        });
                        ui.add_space(2.0);
                    }
                }
                
                if !editor_state.warnings.is_empty() {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("WARNINGS").color(egui::Color32::YELLOW).strong());
                        ui.label(format!("({})", editor_state.warnings.len()));
                    });
                    
                    for warning in &editor_state.warnings {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚠️").color(egui::Color32::YELLOW));
                                ui.label(warning);
                            });
                        });
                        ui.add_space(2.0);
                    }
                }
                
                if editor_state.errors.is_empty() && editor_state.warnings.is_empty() {
                    ui.label(egui::RichText::new("✅ No errors or warnings").color(egui::Color32::GREEN));
                }
            });
        });
    }
}

fn compile_and_render_shader(editor_state: &mut EditorState, global_renderer: &GlobalRenderer) {
    let start_time = std::time::Instant::now();
    editor_state.errors.clear();
    editor_state.warnings.clear();
    
    // Run WGSL diagnostics first
    let diagnostics = editor_state.diagnostics.check_shader(&editor_state.code_editor);
    editor_state.errors.extend(diagnostics.errors.clone());
    editor_state.warnings.extend(diagnostics.warnings.clone());
    
    // If there are critical errors, don't attempt compilation
    if !diagnostics.errors.is_empty() {
        editor_state.compilation_time = start_time.elapsed().as_secs_f32();
        return;
    }
    
    // Extract parameters from code
    editor_state.shader_params = extract_shader_parameters(&editor_state.code_editor);
    
    // Apply timeline animation to parameters
    editor_state.timeline.apply_to_parameters(&mut editor_state.shader_params);
    
    // Prepare parameter values array
    let mut param_values = vec![0.0f32; 64]; // Max 64 parameters
    for (i, param) in editor_state.shader_params.iter().enumerate() {
        if i < 64 {
            param_values[i] = param.value;
        }
    }
    
    // Compile and render with parameters
    if let Ok(mut renderer) = global_renderer.renderer.lock() {
        if let Some(renderer) = renderer.as_mut() {
            match renderer.compile_shader_with_params(&editor_state.code_editor, editor_state.preview_size.0, editor_state.preview_size.1, Some(&param_values)) {
                Ok(_) => {
                    editor_state.last_compiled_code = Some(editor_state.code_editor.clone());
                    editor_state.compilation_time = start_time.elapsed().as_secs_f32();
                }
                Err(errors) => {
                    editor_state.errors = vec![format!("{}", errors)];
                }
            }
        }
    }
}

fn extract_shader_parameters(code: &str) -> Vec<ShaderParameter> {
    let mut params = vec![];
    
    // Simple regex-like parsing for @group(X) @binding(Y) uniforms
    let lines: Vec<&str> = code.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Look for uniform declarations with group/binding
        if trimmed.contains("@group(") && trimmed.contains("@binding(") && trimmed.contains("var<") {
            if let Some(group_start) = trimmed.find("@group(") {
                if let Some(group_end) = trimmed[group_start..].find(")") {
                    let group_str = &trimmed[group_start + 7..group_start + group_end];
                    if let Ok(group) = group_str.parse::<u32>() {
                        if let Some(binding_start) = trimmed.find("@binding(") {
                            if let Some(binding_end) = trimmed[binding_start..].find(")") {
                                let binding_str = &trimmed[binding_start + 9..binding_start + binding_end];
                                if let Ok(binding) = binding_str.parse::<u32>() {
                                    // Extract parameter name and type
                                    if let Some(var_start) = trimmed.find("var<") {
                                        if let Some(var_end) = trimmed[var_start..].find(">") {
                                            let var_content = &trimmed[var_start + 4..var_start + var_end];
                                            if let Some(name_start) = trimmed[var_start + var_end + 1..].find(|c: char| c.is_alphabetic()) {
                                                let name_part = &trimmed[var_start + var_end + 1 + name_start..];
                                                if let Some(name_end) = name_part.find(|c: char| !c.is_alphanumeric() && c != '_') {
                                                    let name = &name_part[..name_end];
                                                    
                                                    // Extract type and default values
                                                    let param_type = if var_content.contains("f32") {
                                                        "f32"
                                                    } else if var_content.contains("i32") {
                                                        "i32"
                                                    } else {
                                                        "unknown"
                                                    };
                                                    
                                                    params.push(ShaderParameter {
                                                        name: name.to_string(),
                                                        value: 0.5,
                                                        min: 0.0,
                                                        max: 1.0,
                                                        default: 0.5,
                                                        binding,
                                                        group,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    params
}

fn format_wgsl_code(code: &str) -> String {
    // Advanced WGSL formatting with proper attribute and statement handling
    let mut formatted = String::new();
    let mut indent_level: i32 = 0;
    let mut in_struct = false;
    let mut in_function = false;
    
    for line in code.lines() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            formatted.push('\n');
            continue;
        }
        
        // Handle WGSL attributes (start with @)
        if trimmed.starts_with('@') {
            // Attributes stay at current indent level
            formatted.push_str(&"    ".repeat(indent_level.max(0) as usize));
            formatted.push_str(trimmed);
            formatted.push('\n');
            continue;
        }
        
        // Decrease indent for closing braces and certain keywords
        if trimmed.starts_with('}') || 
           (trimmed.starts_with("else") && !trimmed.ends_with('{')) {
            indent_level = indent_level.saturating_sub(1);
        }
        
        // Add indentation
        for _ in 0..indent_level.max(0) {
            formatted.push_str("    ");
        }
        
        // Track context
        if trimmed.starts_with("struct ") {
            in_struct = true;
        } else if trimmed.starts_with("fn ") {
            in_function = true;
        } else if trimmed == "}" {
            if in_struct {
                in_struct = false;
            } else if in_function {
                in_function = false;
            }
        }
        
        // Format struct members with proper alignment
        if in_struct && trimmed.contains(':') && !trimmed.starts_with("struct") {
            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
            if parts.len() == 2 {
                let member_name = parts[0].trim();
                let member_type = parts[1].trim().trim_end_matches(',');
                formatted.push_str(&format!("{}: {},", member_name, member_type));
            } else {
                formatted.push_str(trimmed);
            }
        } else {
            formatted.push_str(trimmed);
        }
        
        formatted.push('\n');
        
        // Increase indent for opening braces and certain statements
        if trimmed.ends_with('{') || 
           (trimmed.starts_with("if ") && !trimmed.ends_with(';')) ||
           (trimmed.starts_with("else") && trimmed.ends_with('{')) ||
           (trimmed.starts_with("for ") && !trimmed.ends_with(';')) ||
           (trimmed.starts_with("while ") && !trimmed.ends_with(';')) {
            indent_level += 1;
        }
    }
    
    formatted
}

/// Update timeline animation and trigger re-rendering
pub fn update_timeline_animation(
    mut editor_state: ResMut<EditorState>,
    global_renderer: Res<GlobalRenderer>,
    time: Res<Time>,
) {
    // Only update if playing
    if editor_state.timeline.playback_state != PlaybackState::Playing {
        return;
    }
    
    // Update timeline with delta time
    let delta_time = Duration::from_secs_f32(time.delta_secs());
    editor_state.timeline.update(delta_time);
    
    // Re-render with updated parameters
    compile_and_render_shader(&mut editor_state, &global_renderer);
}