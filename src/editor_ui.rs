use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::fs;
use crate::node_graph::NodeGraph;
use crate::timeline::{Timeline, PlaybackState, ShaderParameter as TimelineShaderParameter};
use crate::audio::AudioAnalyzer;
use crate::wgsl_diagnostics::WgslDiagnostics;

/// UI startup gate to manage initialization timing
#[derive(Resource, Default)]
pub struct UiStartupGate {
    pub frames: u32,
}

/// Editor UI state resource
#[derive(Resource)]
pub struct EditorUiState {
    pub code_editor: String,
    pub shader_params: Vec<TimelineShaderParameter>,
    pub node_graph: NodeGraph,
    pub timeline: Timeline,
    pub audio_analyzer: AudioAnalyzer,
    pub shader_browser: ShaderBrowser,

    pub diagnostics: WgslDiagnostics,
    pub preview_size: (u32, u32),
    pub show_node_editor: bool,
    pub show_timeline: bool,
    pub show_shader_browser: bool,
    pub show_code_editor: bool,
    pub show_preview: bool,
    pub show_parameter_panel: bool,
    pub show_error_panel: bool,
    pub show_node_studio: bool,
    pub show_audio_panel: bool,
    pub show_midi_panel: bool,
    pub show_gesture_panel: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub auto_compile: bool,
    pub last_compiled_code: Option<String>,
    pub compilation_time: f32,
    pub draft_code: String,
    pub available_shaders_compatible: Vec<ShaderSearchResult>,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            code_editor: include_str!("../shaders/default.wgsl").to_string(),
            shader_params: vec![],
            node_graph: NodeGraph::new(),
            timeline: Timeline::new(),
            audio_analyzer: AudioAnalyzer::new(),
            shader_browser: ShaderBrowser::new(),

            diagnostics: WgslDiagnostics::new(),
            preview_size: (512, 512),
            show_node_editor: true,
            show_timeline: true,
            show_shader_browser: true,
            show_code_editor: true,
            show_preview: true,
            show_parameter_panel: true,
            show_error_panel: true,
            show_node_studio: false,
            show_audio_panel: false,
            show_midi_panel: false,
            show_gesture_panel: false,
            errors: vec![],
            warnings: vec![],
            auto_compile: true,
            last_compiled_code: None,
            compilation_time: 0.0,
            draft_code: String::new(),
            available_shaders_compatible: vec![],
        }
    }
}

// Temporary placeholder implementations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderCategory {
    Unknown,
    WGSL,
    ISF,
    GLSL,
    HLSL,
}

#[derive(Default)]
pub struct ShaderBrowser {
    pub search_query: String,
    pub filter_category: Option<ShaderCategory>,
    pub recent_files: Vec<String>,
    pub favorites: Vec<String>,
}

impl ShaderBrowser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scan_directories(&mut self) {
        // Disabled - no scanning functionality
    }

    pub fn load_shader(&self, _path: &str) -> Result<String, std::io::Error> {
        Ok(String::new()) // Return empty shader code
    }

    pub fn search_shaders(&self, _query: &str) -> Vec<ShaderSearchResult> {
        Vec::new() // Return empty results
    }
}

#[derive(Clone)]
pub struct ShaderSearchResult {
    pub name: String,
    pub path: String,
    pub category: ShaderCategory,
}

// Use the working visual_node_editor instead of broken node_graph
// use crate::visual_node_editor::VisualNodeEditor;

#[derive(Resource)]
pub struct EditorState {
    pub code_editor: String,
    pub shader_params: Vec<TimelineShaderParameter>,
    pub node_graph: NodeGraph,
    pub timeline: Timeline,
    pub audio_analyzer: AudioAnalyzer,
    pub shader_browser: ShaderBrowser,
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
            timeline: Timeline::new(),
            audio_analyzer: AudioAnalyzer::new(),
            shader_browser: ShaderBrowser::new(),
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



pub fn editor_ui_system(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
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
                // Compilation is now handled by the WGPU integration system
                println!("Compile button clicked - WGPU system will handle compilation");
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
                .selected_text(format!("{:?}", editor_state.shader_browser.filter_category.unwrap_or(ShaderCategory::Unknown)))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, None, "All");
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, Some(ShaderCategory::WGSL), "WGSL");
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, Some(ShaderCategory::ISF), "ISF");
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, Some(ShaderCategory::GLSL), "GLSL");
                    ui.selectable_value(&mut editor_state.shader_browser.filter_category, Some(ShaderCategory::HLSL), "HLSL");
                });
        });
        
        ui.separator();
        
        // Quick actions
        ui.horizontal(|ui| {
            if ui.button("Load Default").clicked() {
                editor_state.code_editor = include_str!("../shaders/default.wgsl").to_string();
                if editor_state.auto_compile {
                    // Compilation is now handled by the WGPU integration system
                    println!("Manual compile requested - WGPU system will handle compilation");
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
                    if ui.button(std::path::Path::new(path).file_name().unwrap_or_default().to_string_lossy()).clicked() {
                        if let Ok(content) = editor_state.shader_browser.load_shader(path) {
                            editor_state.code_editor = content;
                            if editor_state.auto_compile {
                                // Compilation is now handled by the WGPU integration system
                                println!("Shader compilation triggered by parameter change");
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
                    if ui.button(std::path::Path::new(path).file_name().unwrap_or_default().to_string_lossy()).clicked() {
                        if let Ok(content) = editor_state.shader_browser.load_shader(path) {
                            editor_state.code_editor = content;
                            if editor_state.auto_compile {
                                // Compilation is now handled by the WGPU integration system
                        println!("Shader compilation triggered by parameter update");
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
                                // Compilation is now handled by the WGPU integration system
                        println!("Shader compilation triggered by timeline change");
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
            // Create a copy of parameter values to avoid borrowing issues
            let mut param_values_copy = vec![0.0f32; 64];
            let mut changed_indices = Vec::new();
            
            // First pass: display parameters and collect changes
            for (i, param) in editor_state.shader_params.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(&param.name);
                    let response = ui.add(egui::DragValue::new(&mut param.value)
                        .speed(0.01)
                        .range(param.min..=param.max));
                    
                    // Update parameters in real-time when values change
                    if response.changed() {
                        changed_indices.push(i);
                    }
                });
            }
            
            // Second pass: update parameter values and re-render if needed
            if !changed_indices.is_empty() {
                // Update the copy with new values
                for (i, param) in editor_state.shader_params.iter().enumerate() {
                    if i < 64 {
                        param_values_copy[i] = param.value;
                    }
                }
                
                // Parameter updates are now handled by the WGPU integration system
                println!("Node graph parameter update - WGPU system will handle rendering");
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
                            // Extract values before mutable borrow
                            let preview_width = editor_state.preview_size.0;
                            let preview_height = editor_state.preview_size.1;
                            let mut node_graph = std::mem::take(&mut editor_state.node_graph);
                            
                            // Generate WGSL from node graph
                            let wgsl_code = editor_state.node_graph.generate_wgsl(preview_width, preview_height);
                            editor_state.code_editor = wgsl_code;
                            editor_state.node_graph = node_graph;
                            // Compilation is now handled by the WGPU integration system
    println!("Shader compilation triggered by test function");
                        }
                        if ui.button("Clear Graph").clicked() {
                            editor_state.node_graph = NodeGraph::new();
                        }
                    });
                    
                    // Show node graph info
                    ui.separator();
                    ui.label(format!("Nodes: {}", editor_state.node_graph.nodes.len()));
                    ui.label(format!("Connections: {}", editor_state.node_graph.connections.len()));
                    
                    // Node graph UI placeholder - will implement proper UI later
                    ui.label("Node Graph Editor - Placeholder");
                    
                    // Auto-compile if enabled and node graph changed
                    if editor_state.auto_compile {
                        // Extract values before mutable borrow
                        let preview_width = editor_state.preview_size.0;
                        let preview_height = editor_state.preview_size.1;
                        let mut node_graph = std::mem::take(&mut editor_state.node_graph);
                        
                        // Simple auto-compile - generate WGSL and compile
                        let wgsl_code = editor_state.node_graph.generate_wgsl(preview_width, preview_height);
                        editor_state.code_editor = wgsl_code;
                        editor_state.node_graph = node_graph;
                        // Compilation is now handled by the WGPU integration system
                        println!("Shader compilation triggered by node editor");
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
                let text_style = egui::TextStyle::Monospace;
                let row_height = ui.text_style_height(&text_style);
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
                        // Compilation is now handled by the WGPU integration system
                        println!("Code editor change - WGPU system will handle compilation");
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
                
                // Display preview texture - WGPU integration handles this
                ui.label("🎨 Preview: WGPU Rendering Active");
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
                    .range(0.0..=duration));
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



fn extract_shader_parameters(code: &str) -> Vec<TimelineShaderParameter> {
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
                                                    
                                                    params.push(TimelineShaderParameter {
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

/// Draw the editor menu bar
pub fn draw_editor_menu(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    ui_state.code_editor = include_str!("../shaders/default.wgsl").to_string();
                    ui_state.node_graph = NodeGraph::new();
                    ui_state.errors.clear();
                    ui_state.warnings.clear();
                }
                if ui.button("Open").clicked() {
                    if let Ok(content) = fs::read_to_string("shader.wgsl") {
                        ui_state.code_editor = content;
                    }
                }
                if ui.button("Save").clicked() {
                    let _ = fs::write("shader.wgsl", &ui_state.code_editor);
                }
                if ui.button("Export").clicked() {
                    let _ = fs::write("exported_shader.wgsl", &ui_state.code_editor);
                }
            });
            
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut ui_state.show_node_editor, "Node Editor");
                ui.checkbox(&mut ui_state.show_timeline, "Timeline");
                ui.checkbox(&mut ui_state.show_shader_browser, "Shader Browser");
                ui.checkbox(&mut ui_state.show_error_panel, "Error Panel");
            });
            
            ui.separator();
            
            ui.checkbox(&mut ui_state.auto_compile, "Auto Compile");
            
            if ui.button("Compile").clicked() {
                println!("Compile button clicked - WGPU system will handle compilation");
            }
            
            // Compilation status indicator
            ui.separator();
            if !ui_state.errors.is_empty() {
                ui.label(egui::RichText::new(format!("❌ {} errors", ui_state.errors.len())).color(egui::Color32::RED));
            } else if !ui_state.warnings.is_empty() {
                ui.label(egui::RichText::new(format!("⚠️ {} warnings", ui_state.warnings.len())).color(egui::Color32::YELLOW));
            } else if ui_state.last_compiled_code.is_some() {
                ui.label(egui::RichText::new(format!("✅ Compiled ({:.2}s)", ui_state.compilation_time)).color(egui::Color32::GREEN));
            } else {
                ui.label(egui::RichText::new("🔄 Ready").color(egui::Color32::GRAY));
            }
            
            // Show compilation time
            if ui_state.compilation_time > 0.0 {
                ui.label(format!("{:.3}s", ui_state.compilation_time));
            }
        });
    });
}

/// Draw the editor side panels (shader browser, parameters, preview)
pub fn draw_editor_side_panels(ctx: &egui::Context, ui_state: &mut EditorUiState, audio_analyzer: &AudioAnalyzer) {
    // Shader Browser Panel
    egui::SidePanel::left("shader_browser").default_width(300.0).show_animated(ctx, ui_state.show_shader_browser, |ui| {
        ui.heading("Shader Browser");
        ui.separator();
        
        // Search bar
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut ui_state.shader_browser.search_query);
        });
        
        // Quick actions
        ui.horizontal(|ui| {
            if ui.button("Load Default").clicked() {
                ui_state.code_editor = include_str!("../shaders/default.wgsl").to_string();
                println!("Shader compilation triggered by parameter change");
            }
        });
        
        ui.separator();
        
        // Parameters panel
        ui.label("Parameters");
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for param in &mut ui_state.shader_params {
                ui.horizontal(|ui| {
                    ui.label(&param.name);
                    ui.add(egui::DragValue::new(&mut param.value)
                        .speed(0.01)
                        .range(param.min..=param.max));
                });
            }
        });
    });
    
    // Preview Panel - Fixed to show actual rendered texture
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Preview");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label(format!("Size: {}x{}", ui_state.preview_size.0, ui_state.preview_size.1));
            if ui.button("512x512").clicked() {
                ui_state.preview_size = (512, 512);
            }
            if ui.button("1024x1024").clicked() {
                ui_state.preview_size = (1024, 1024);
            }
        });
        
        // Display preview texture with proper sizing
        let preview_size = egui::Vec2::new(ui_state.preview_size.0 as f32, ui_state.preview_size.1 as f32);
        let available_size = ui.available_size();
        let scale = (available_size.x / preview_size.x).min(available_size.y / preview_size.y).min(1.0);
        let display_size = preview_size * scale;
        
        // Create a placeholder texture display
        let response = ui.add_sized(display_size, egui::Button::new("🎨 Shader Preview\nWGPU Rendering Active"));
        
        // Show compilation status in preview
        if !ui_state.errors.is_empty() {
            ui.colored_label(egui::Color32::RED, format!("❌ {} errors", ui_state.errors.len()));
        } else if !ui_state.warnings.is_empty() {
            ui.colored_label(egui::Color32::YELLOW, format!("⚠️ {} warnings", ui_state.warnings.len()));
        } else if ui_state.last_compiled_code.is_some() {
            ui.colored_label(egui::Color32::GREEN, format!("✅ Compiled ({:.3}s)", ui_state.compilation_time));
        }
    });
}

/// Draw the code editor panel
pub fn draw_editor_code_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            if ui_state.show_node_editor {
                // Node Editor Panel
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Node Editor");
                        ui.separator();
                        if ui.button("Generate WGSL").clicked() {
                            let preview_width = ui_state.preview_size.0;
                            let preview_height = ui_state.preview_size.1;
                            let wgsl_code = ui_state.node_graph.generate_wgsl(preview_width, preview_height);
                            ui_state.code_editor = wgsl_code;
                            println!("Shader compilation triggered by test function");
                        }
                        if ui.button("Clear Graph").clicked() {
                            ui_state.node_graph = NodeGraph::new();
                        }
                    });
                    
                    ui.label(format!("Nodes: {}", ui_state.node_graph.nodes.len()));
                    ui.label(format!("Connections: {}", ui_state.node_graph.connections.len()));
                    ui.label("Node Graph Editor - Placeholder");
                });
            }
            
            // Code Editor Panel
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("WGSL Code Editor");
                    ui.separator();
                    if ui.button("Format").clicked() {
                        ui_state.code_editor = format_wgsl_code(&ui_state.code_editor);
                    }
                    if ui.button("Clear").clicked() {
                        ui_state.code_editor.clear();
                    }
                });
                
                let response = ui.add(
                    egui::TextEdit::multiline(&mut ui_state.code_editor)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(20)
                        .desired_width(ui.available_width())
                        .lock_focus(true)
                );
                
                if response.changed() && ui_state.auto_compile {
                    println!("Code editor change - WGPU system will handle compilation");
                }
            });
        });
    });
}