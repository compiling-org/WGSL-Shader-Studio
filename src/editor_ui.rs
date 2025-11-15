use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::fs;
use std::path::Path;
use egui::text::LayoutJob;
use egui::TextBuffer;
use std::sync::Arc;
use crate::node_graph::{NodeGraph, NodeKind};
use crate::timeline::Timeline;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PipelineMode {
    Fragment,
    Compute,
}

impl Default for PipelineMode {
    fn default() -> Self { PipelineMode::Fragment }
}

#[derive(Resource, Default)]
pub struct EditorUiState {
    pub show_shader_browser: bool,
    pub show_parameter_panel: bool,
    pub show_preview: bool,
    pub show_code_editor: bool,
    // Top-level feature panels
    pub show_node_studio: bool,
    pub show_timeline: bool,
    pub show_audio_panel: bool,
    pub show_midi_panel: bool,
    pub show_gesture_panel: bool,
    pub fps: f32,
    // Preview pipeline mode
    pub pipeline_mode: PipelineMode,
    // Browser/state
    pub search_query: String,
    pub show_all_shaders: bool,
    pub available_shaders_all: Vec<String>,
    pub available_shaders_compatible: Vec<String>,
    pub selected_shader: Option<String>,
    pub selected_category: Option<String>,
    // Code editor buffer
    pub draft_code: String,
    pub apply_requested: bool,
    pub auto_apply: bool,
    // Node graph and project state
    pub node_graph: NodeGraph,
    pub last_project_path: Option<String>,
    pub timeline: Timeline,
    pub timeline_track_input: String,
    pub param_index_map: std::collections::HashMap<String, usize>,
    pub param_index_input: usize,
    // Quick parameter controls for preview panel
    pub quick_params_enabled: bool,
    pub quick_param_a: f32,
    pub quick_param_b: f32,
}

#[derive(Resource, Default)]
pub struct UiStartupGate {
    pub frames: u32,
}

// Helper that draws the menu using a provided egui context
pub fn draw_editor_menu(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🎨 WGSL Shader Studio").size(16.0));
            ui.separator();
            ui.checkbox(&mut ui_state.show_shader_browser, "Shader Browser");
            ui.checkbox(&mut ui_state.show_parameter_panel, "Parameters");
            ui.checkbox(&mut ui_state.show_preview, "Preview");
            ui.checkbox(&mut ui_state.show_code_editor, "Code Editor");
            ui.separator();
            ui.menu_button("Pipeline", |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut ui_state.pipeline_mode, PipelineMode::Fragment, "Fragment");
                    ui.radio_value(&mut ui_state.pipeline_mode, PipelineMode::Compute, "Compute");
                });
                ui.label("Switch between fragment and compute (shader must match)");
            });
            ui.menu_button("Studio", |ui| {
                ui.checkbox(&mut ui_state.show_node_studio, "Node Studio");
                ui.checkbox(&mut ui_state.show_timeline, "Timeline");
                ui.checkbox(&mut ui_state.show_audio_panel, "Audio");
                ui.checkbox(&mut ui_state.show_midi_panel, "MIDI");
                ui.checkbox(&mut ui_state.show_gesture_panel, "Gestures");
            });

            ui.separator();
            ui.menu_button("Import/Convert", |ui| {
                if ui.button("Import ISF (.fs) → WGSL into editor").clicked() {
                    import_isf_into_editor(ui_state);
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                if ui.button("Batch convert ISF directory → WGSL").clicked() {
                    batch_convert_isf_directory();
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Current buffer: GLSL → WGSL").clicked() {
                    convert_current_glsl_to_wgsl(ui_state);
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                if ui.button("Current buffer: HLSL → WGSL").clicked() {
                    convert_current_hlsl_to_wgsl(ui_state);
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Export current WGSL → GLSL").clicked() {
                    export_current_wgsl_to_glsl(&ui_state);
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                if ui.button("Export current WGSL → HLSL").clicked() {
                    export_current_wgsl_to_hlsl(&ui_state);
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
            });

            ui.separator();
            ui.menu_button("File", |ui| {
                if ui.button("New WGSL Buffer").clicked() {
                    println!("Clicked: New WGSL Buffer");
                    ui_state.draft_code = default_wgsl_template();
                    ctx.request_repaint();
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                if ui.button("Save Draft As…").clicked() {
                    println!("Clicked: Save Draft As…");
                    save_draft_wgsl_to_assets(&ui_state);
                    ctx.request_repaint();
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Save Project…").clicked() {
                    println!("Clicked: Save Project…");
                    let _ = export_project_json(&ui_state);
                    ctx.request_repaint();
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                if ui.button("Open Project…").clicked() {
                    println!("Clicked: Open Project…");
                    match import_project_json() {
                        Ok(proj) => {
                            ui_state.node_graph = proj.node_graph;
                            if let Some(code) = proj.draft_code { ui_state.draft_code = code; }
                            ui_state.timeline = proj.timeline;
                            ui_state.param_index_map = proj.param_index_map;
                        }
                        Err(e) => { println!("Import project failed: {}", e); }
                    }
                    ctx.request_repaint();
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
                ui.separator();
                if ui.button("Export recorded frames → MP4").clicked() {
                    println!("Clicked: Export recorded frames → MP4");
                    export_recorded_frames_to_mp4();
                    ctx.request_repaint();
                    ui.close_kind(bevy_egui::egui::UiKind::Menu);
                }
            });

            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("FPS: {:.0}", ui_state.fps));
                if ui.button("Apply to Preview").clicked() {
                    println!("Clicked: Apply to Preview");
                    ui_state.apply_requested = true;
                    ctx.request_repaint();
                }
            });
        });
    });
}

pub fn editor_menu(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let ctx = match egui_ctx.ctx_mut() { Ok(c) => c, Err(_) => return };
    draw_editor_menu(&ctx, &mut *ui_state);
}

// Helper that draws the browser/parameters/timeline panels using a provided egui context
pub fn draw_editor_side_panels(ctx: &egui::Context, ui_state: &mut EditorUiState) {

    if ui_state.show_shader_browser {
        egui::SidePanel::left("shader_browser").resizable(true).show(ctx, |ui| {
            ui.heading("Shader Browser");
            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.show_all_shaders, "Show all shaders");
                if !ui_state.show_all_shaders {
                    ui.label("Showing compatible only (has @vertex and @fragment)");
                }
            });
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut ui_state.search_query);
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                let names = if ui_state.show_all_shaders {
                    ui_state.available_shaders_all.clone()
                } else {
                    ui_state.available_shaders_compatible.clone()
                };
                for name in names.iter() {
                    if !ui_state.search_query.is_empty() && !name.to_lowercase().contains(&ui_state.search_query.to_lowercase()) {
                        continue;
                    }
                    let selected = ui.selectable_label(ui_state.selected_shader.as_ref().map(|s| s == name).unwrap_or(false), name);
                    if selected.clicked() {
                        ui_state.selected_shader = Some(name.clone());
                    }
                }
            });
        });
    }

    if ui_state.show_parameter_panel {
        egui::SidePanel::right("parameters").resizable(true).show(ctx, |ui| {
            ui.heading("Parameters");
            ui.label("Stable mapping of named parameters → params[0..63]");
            ui.separator();
            // Editor inputs for adding/updating a mapping
            ui.horizontal(|ui| {
                ui.label("Track name:");
                ui.text_edit_singleline(&mut ui_state.timeline_track_input);
            });
            ui.horizontal(|ui| {
                ui.label("Index (0-63):");
                let mut idx_i32 = ui_state.param_index_input as i32;
                if ui.add(egui::Slider::new(&mut idx_i32, 0..=63)).changed() {
                    ui_state.param_index_input = idx_i32 as usize;
                }
                if ui.button("Add/Update Mapping").clicked() {
                    let name = ui_state.timeline_track_input.trim().to_string();
                    if !name.is_empty() {
                        let idx = ui_state.param_index_input;
                        ui_state.param_index_map.insert(name, idx);
                    }
                }
                if ui.button("Clear All").clicked() {
                    ui_state.param_index_map.clear();
                }
            });
            ui.separator();
            ui.label("Current mappings:");
            egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                let mut keys: Vec<_> = ui_state.param_index_map.keys().cloned().collect();
                keys.sort();
                for k in keys {
                    let idx = ui_state.param_index_map.get(&k).cloned().unwrap_or(0);
                    ui.horizontal(|ui| {
                        ui.label(format!("{} → params[{}]", k, idx));
                        if ui.button("Remove").clicked() {
                            ui_state.param_index_map.remove(&k);
                        }
                    });
                }
                if ui_state.param_index_map.is_empty() {
                    ui.label("(No mappings set; params filled alphabetically)");
                }
            });
        });
    }

    if ui_state.show_node_studio {
        let mut show = ui_state.show_node_studio;
        egui::Window::new("Node Studio").open(&mut show).show(ctx, |ui| {
            ui.heading("Node-based Shader Authoring");
            ui.label("Quick palette:");
            ui.horizontal(|ui| {
                if ui.button("Add UV").clicked() {
                    ui_state.node_graph.add_node(NodeKind::UV, "UV", (100.0, 100.0));
                }
                if ui.button("Add Time").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Time, "Time", (160.0, 100.0));
                }
                if ui.button("Add Param").clicked() {
                    let idx = ui_state.param_index_input.min(63);
                    ui_state.node_graph.add_node(NodeKind::Param(idx), &format!("Param[{}]", idx), (220.0, 100.0));
                }
                if ui.button("Add Sin").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Sine, "Sine", (220.0, 100.0));
                }
                if ui.button("Add Add").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Add, "Add", (280.0, 100.0));
                }
                if ui.button("Add Multiply").clicked() {
                    ui_state.node_graph.add_node(NodeKind::Multiply, "Multiply", (340.0, 100.0));
                }
                if ui.button("Add Const3").clicked() {
                    ui_state.node_graph.add_node(NodeKind::ConstantVec3([0.5, 0.3, 0.8]), "Const Vec3", (400.0, 100.0));
                }
                if ui.button("Add Output").clicked() {
                    ui_state.node_graph.add_node(NodeKind::OutputColor, "Output", (460.0, 100.0));
                }
            });
            ui.separator();
            ui.label("Note: Connections can be established via auto-wiring below.");
            if ui.button("Auto-wire: UV → TextureSample → Output").clicked() {
                // Create a minimal graph: uv -> sample -> output
                let uv = ui_state.node_graph.add_node(NodeKind::UV, "UV", (100.0, 160.0));
                let ts = ui_state.node_graph.add_node(NodeKind::TextureSample, "TextureSample", (220.0, 160.0));
                let out = ui_state.node_graph.add_node(NodeKind::OutputColor, "Output", (360.0, 160.0));
                // Find ports
                let uv_out = ui_state.node_graph.nodes.get(&uv).unwrap().outputs[0].id;
                let ts_in_uv = ui_state.node_graph.nodes.get(&ts).unwrap().inputs[1].id;
                let ts_out = ui_state.node_graph.nodes.get(&ts).unwrap().outputs[0].id;
                let out_in = ui_state.node_graph.nodes.get(&out).unwrap().inputs[0].id;
                ui_state.node_graph.connect(uv, uv_out, ts, ts_in_uv);
                ui_state.node_graph.connect(ts, ts_out, out, out_in);
            }
            if ui.button("Generate WGSL from Graph").clicked() {
                let wgsl = ui_state.node_graph.generate_wgsl(512, 512);
                ui_state.draft_code = wgsl;
                ui_state.apply_requested = true;
            }
            ui.separator();
            if ui.button("Export Project JSON").clicked() {
                if let Err(e) = export_project_json(&ui_state) {
                    ui.label(format!("Export error: {}", e));
                }
            }
            if ui.button("Import Project JSON").clicked() {
                match import_project_json() {
                    Ok(loaded) => {
                        ui_state.node_graph = loaded.node_graph;
                        if let Some(code) = loaded.draft_code {
                            ui_state.draft_code = code;
                        }
                        ui_state.timeline = loaded.timeline;
                        ui_state.param_index_map = loaded.param_index_map;
                    }
                    Err(e) => { ui.label(format!("Import error: {}", e)); }
                }
            }
        });
        ui_state.show_node_studio = show;
    }
    if ui_state.show_timeline {
        let mut show = ui_state.show_timeline;
        egui::Window::new("Timeline").open(&mut show).show(ctx, |ui| {
            ui.heading("Simple Timeline");
            ui.label("Create keyframes for parameter tracks and interpolate.");
            ui.separator();
            // Controls to add a keyframe to the 'time' track
            static mut KF_TIME: f32 = 0.0;
            static mut KF_VALUE: f32 = 0.0;
            let mut kf_time;
            let mut kf_value;
            unsafe { kf_time = KF_TIME; kf_value = KF_VALUE; }
            ui.horizontal(|ui| {
                ui.label("Track name:");
                ui.text_edit_singleline(&mut ui_state.timeline_track_input);
            });
            ui.horizontal(|ui| {
                ui.label("Keyframe time:");
                ui.add(egui::DragValue::new(&mut kf_time).range(0.0..=1000.0).speed(0.1));
                ui.label("Value:");
                ui.add(egui::DragValue::new(&mut kf_value).speed(0.1));
                let track = if ui_state.timeline_track_input.trim().is_empty() { "time".to_string() } else { ui_state.timeline_track_input.clone() };
                if ui.button(format!("Add keyframe to '{}'", track)).clicked() {
                    ui_state.timeline.add_keyframe(&track, kf_time, kf_value);
                }
            });
            unsafe { KF_TIME = kf_time; KF_VALUE = kf_value; }
            ui.separator();
            ui.label("Tracks:");
            egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                for (param, kfs) in ui_state.timeline.tracks.iter() {
                    ui.collapsing(format!("{} ({} kfs)", param, kfs.len()), |ui| {
                        for k in kfs.iter() {
                            ui.label(format!("t={:.2} → v={:.3}", k.time, k.value));
                        }
                    });
                }
            });
        });
        ui_state.show_timeline = show;
    }
    if ui_state.show_audio_panel {
        egui::Window::new("Audio").open(&mut ui_state.show_audio_panel).show(ctx, |ui| {
            ui.heading("Audio Analysis");
            ui.checkbox(&mut ui_state.quick_params_enabled, "Reactive");
            ui.horizontal(|ui| {
                ui.label("Gain");
                ui.add(egui::Slider::new(&mut ui_state.quick_param_b, 0.0..=2.0));
            });
        });
    }
    if ui_state.show_midi_panel {
        egui::Window::new("MIDI").open(&mut ui_state.show_midi_panel).show(ctx, |ui| {
            ui.heading("MIDI Mapping");
            ui.horizontal(|ui| {
                ui.label("CC #");
                ui.add(egui::DragValue::new(&mut ui_state.param_index_input).clamp_range(0..=127));
                ui.checkbox(&mut ui_state.quick_params_enabled, "Enable");
            });
        });
    }
    if ui_state.show_gesture_panel {
        egui::Window::new("Gestures").open(&mut ui_state.show_gesture_panel).show(ctx, |ui| {
            ui.heading("Gesture Controls");
            ui.checkbox(&mut ui_state.quick_params_enabled, "Map gestures to params");
            ui.horizontal(|ui| {
                ui.label("Sensitivity");
                ui.add(egui::Slider::new(&mut ui_state.quick_param_a, 0.0..=1.0));
            });
        });
    }
}

pub fn editor_side_panels(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let ctx = match egui_ctx.ctx_mut() { Ok(c) => c, Err(_) => return };
    draw_editor_side_panels(&ctx, &mut *ui_state);
}

/// Populate UI state's shader list by scanning common directories.
/// This runs at Startup from the Bevy app.
pub fn populate_shader_list(mut ui_state: ResMut<EditorUiState>) {
    let mut found_all = Vec::new();
    let dirs = ["examples", "assets/shaders", "assets", "shaders"];
    for d in dirs.iter() {
        let path = Path::new(d);
        if !path.exists() { continue; }
        collect_wgsl_files(path, &mut found_all);
    }
    found_all.sort();
    found_all.dedup();
    // Compute compatible set once using validator
    let mut compatible = Vec::new();
    for p in found_all.iter() {
        if let Ok(src) = fs::read_to_string(p) {
            if is_wgsl_shader_compatible(&src) { compatible.push(p.clone()); }
        }
    }
    ui_state.available_shaders_all = found_all;
    ui_state.available_shaders_compatible = compatible;
}

fn collect_wgsl_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_wgsl_files(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("wgsl") {
                    if let Some(s) = p.to_str() {
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
}

/// Bottom code editor panel bound to `EditorUiState::draft_code`.
// Helper that draws the code editor panel using a provided egui context
pub fn draw_editor_code_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    if !ui_state.show_code_editor { return; }
    egui::TopBottomPanel::bottom("code_editor")
        .resizable(false)
        .default_height(240.0)
        .min_height(160.0)
        .max_height(280.0)
        .show(ctx, |ui| {
        ui.heading("WGSL Code Editor");
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut edit = egui::TextEdit::multiline(&mut ui_state.draft_code)
                .code_editor()
                .desired_rows(12)
                .lock_focus(true)
                .hint_text("Paste or write WGSL here...");
            let mut layouter = |ui: &egui::Ui, text: &dyn TextBuffer, wrap_width: f32| -> Arc<egui::Galley> {
                highlight_wgsl(ui, text, wrap_width)
            };
            edit = edit.layouter(&mut layouter);
            // Fix editor height so it doesn't balloon and block the viewport
            let fixed_height = 180.0;
            ui.add_sized(egui::vec2(ui.available_width(), fixed_height), edit);
        });
        ui.horizontal(|ui| {
            if ui.button("Apply to Preview").clicked() {
                ui_state.apply_requested = true;
            }
            ui.checkbox(&mut ui_state.auto_apply, "Auto Apply");
            ui.label("Tip: Load a shader from the browser, edit, then apply.");
        });
    });
}

pub fn editor_code_panel(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let ctx = match egui_ctx.ctx_mut() { Ok(c) => c, Err(_) => return };
    draw_editor_code_panel(&ctx, &mut *ui_state);
}

/// System to load selected shader file contents into draft buffer.
pub fn apply_shader_selection(mut ui_state: ResMut<EditorUiState>) {
    if let Some(sel) = ui_state.selected_shader.clone() {
        if let Ok(src) = fs::read_to_string(&sel) {
            // Only update draft; preview is updated when Apply is pressed.
            ui_state.draft_code = src;
            // Auto-apply if enabled
            if ui_state.auto_apply {
                ui_state.apply_requested = true;
            }
        }
        // Clear selection so we don't re-load every frame
        ui_state.selected_shader = None;
    }
}

/// Validator: requires both @vertex and @fragment entry points for compatibility.
pub fn is_wgsl_shader_compatible(src: &str) -> bool {
    let has_vertex = src.contains("@vertex");
    let has_fragment = src.contains("@fragment");
    has_vertex && has_fragment
}

/// If incompatible, return a clear message; otherwise, Ok(())
pub fn validate_wgsl_entry_points(src: &str) -> Result<(), String> {
    let has_vertex = src.contains("@vertex");
    let has_fragment = src.contains("@fragment");
    match (has_vertex, has_fragment) {
        (true, true) => Ok(()),
        (false, true) => Err("Missing @vertex entry point".to_string()),
        (true, false) => Err("Missing @fragment entry point".to_string()),
        (false, false) => Err("Missing both @vertex and @fragment entry points".to_string()),
    }
}

/// Mode-aware validator supporting fragment or compute pipelines.
pub fn validate_wgsl_for_mode(src: &str, mode: PipelineMode) -> Result<(), String> {
    match mode {
        PipelineMode::Fragment => {
            // Require vertex + fragment entries
            validate_wgsl_entry_points(src).and_then(|_| {
                // Heuristic binding checks for group(0)
                let has_uniforms = src.contains("@group(0)") && src.contains("@binding(0)");
                let has_params = src.contains("@group(0)") && src.contains("@binding(1)");
                if !has_uniforms {
                    return Err("Fragment mode: expected @group(0) @binding(0) uniforms".to_string());
                }
                if !has_params {
                    return Err("Fragment mode: expected @group(0) @binding(1) params".to_string());
                }
                // Ensure fragment outputs a color
                let has_color_out = src.contains("@fragment") && src.contains("@location(0)");
                if !has_color_out {
                    return Err("Fragment mode: expected @location(0) color output".to_string());
                }
                Ok(())
            })
        }
        PipelineMode::Compute => {
            let has_compute = src.contains("@compute");
            if !has_compute { return Err("Missing @compute entry point".to_string()); }
            // Heuristic binding checks
            let has_uniforms = src.contains("@group(0)") && src.contains("@binding(0)");
            let has_params = src.contains("@group(0)") && src.contains("@binding(1)");
            let has_storage = src.contains("texture_storage_2d") && src.contains("@binding(2)");
            if !has_uniforms {
                return Err("Compute mode: expected @group(0) @binding(0) uniforms".to_string());
            }
            if !has_params {
                return Err("Compute mode: expected @group(0) @binding(1) params".to_string());
            }
            if !has_storage {
                return Err("Compute mode: expected @group(0) @binding(2) storage texture".to_string());
            }
            Ok(())
        }
    }
}

fn highlight_wgsl(ui: &egui::Ui, text: &dyn TextBuffer, wrap_width: f32) -> Arc<egui::Galley> {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let s = text.as_str();
    let mut _line_start = 0;
    for (i, line) in s.lines().enumerate() {
        let mut _idx = 0;
        let mut _in_comment = false;
        while _idx < line.len() {
            // Detect comments
            if !_in_comment {
                if let Some(pos) = line[_idx..].find("//") {
                    // append up to comment normally
                    let before = &line[_idx.._idx+pos];
                    append_tokens(&mut job, before);
                    // append comment
                    let comment = &line[_idx+pos..];
                    job.append(
                        comment,
                        0.0,
                        egui::TextFormat {
                            color: egui::Color32::from_rgb(120, 130, 140),
                            ..Default::default()
                        },
                    );
                    _in_comment = true;
                    _idx = line.len();
                    break;
                }
            }
            if !_in_comment {
                let rest = &line[_idx..];
                append_tokens(&mut job, rest);
                _idx = line.len();
            }
        }
        // newline at end of each line except maybe last
        if i < s.lines().count() {
            job.append("\n", 0.0, Default::default());
        }
        _line_start += line.len() + 1;
    }
    ui.fonts_mut(|f| f.layout_job(job))
}

fn append_tokens(job: &mut LayoutJob, s: &str) {
    // Tokenize by whitespace and punctuation (very naive)
    let mut token = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            token.push(ch);
        } else {
            if !token.is_empty() { append_token(job, &token); token.clear(); }
            job.append(
                &ch.to_string(),
                0.0,
                egui::TextFormat { ..Default::default() },
            );
        }
    }
    if !token.is_empty() { append_token(job, &token); }
}

fn append_token(job: &mut LayoutJob, tok: &str) {
    let (color, _italic) = match tok {
        // WGSL attributes and builtins
        "@fragment" | "@vertex" | "@compute" | "@group" | "@binding" | "@location" | "@builtin" => (egui::Color32::from_rgb(180, 120, 255), false),
        // Types
        "f32" | "u32" | "i32" | "vec2" | "vec3" | "vec4" | "mat2x2" | "mat3x3" | "mat4x4" => (egui::Color32::from_rgb(110, 180, 255), false),
        // Keywords
        "struct" | "var" | "let" | "fn" | "return" | "if" | "else" | "for" | "while" | "break" | "continue" | "true" | "false" => (egui::Color32::from_rgb(255, 200, 100), false),
        // Common identifiers
        "uniforms" | "time" | "resolution" | "mouse" => (egui::Color32::LIGHT_GRAY, false),
        _ => (egui::Color32::WHITE, false),
    };
    job.append(
        tok,
        0.0,
        egui::TextFormat { color, ..Default::default() },
    );
}

// ==== Converter actions ====
fn import_isf_into_editor(ui_state: &mut EditorUiState) {
    // Select an ISF file and convert to WGSL into draft buffer
    let file = rfd::FileDialog::new()
        .add_filter("ISF Files", &["fs"])
        .pick_file();
    if let Some(p) = file {
        if let Ok(content) = std::fs::read_to_string(&p) {
            match crate::IsfShader::parse(p.file_stem().and_then(|s| s.to_str()).unwrap_or("ISF"), &content) {
                Ok(parsed) => {
                    // Map to converter type
                    let converter_shader = crate::shader_converter::IsfShader {
                        name: parsed.name.clone(),
                        source: content.clone(),
                        inputs: parsed.inputs.iter().map(|input| crate::shader_converter::ShaderInput {
                            name: input.name.clone(),
                            input_type: match input.input_type {
                                crate::InputType::Float => crate::shader_converter::InputType::Float,
                                crate::InputType::Bool => crate::shader_converter::InputType::Bool,
                                crate::InputType::Color => crate::shader_converter::InputType::Color,
                                crate::InputType::Point2D => crate::shader_converter::InputType::Point2D,
                                crate::InputType::Image => crate::shader_converter::InputType::Image,
                            },
                            value: match input.value {
                                crate::ShaderValue::Float(f) => crate::shader_converter::ShaderValue::Float(f),
                                crate::ShaderValue::Bool(b) => crate::shader_converter::ShaderValue::Bool(b),
                                crate::ShaderValue::Color(c) => crate::shader_converter::ShaderValue::Color(c),
                                crate::ShaderValue::Point2D(p) => crate::shader_converter::ShaderValue::Point2D(p),
                            },
                            min: input.min,
                            max: input.max,
                            default: input.default,
                        }).collect(),
                        outputs: parsed.outputs.iter().map(|output| crate::shader_converter::ShaderOutput {
                            name: output.name.clone(),
                            output_type: match output.output_type {
                                crate::OutputType::Image => crate::shader_converter::OutputType::Image,
                                crate::OutputType::Float => crate::shader_converter::OutputType::Float,
                            },
                        }).collect(),
                    };
                    match crate::shader_converter::isf_to_wgsl(&converter_shader) {
                        Ok(wgsl) => {
                            ui_state.draft_code = wgsl;
                        }
                        Err(e) => {
                            println!("ISF→WGSL conversion failed: {}", e);
                        }
                    }
                }
                Err(e) => println!("ISF parse failed: {}", e),
            }
        }
    }
}

fn batch_convert_isf_directory() {
    let src = rfd::FileDialog::new().pick_folder();
    if src.is_none() { return; }
    let out = rfd::FileDialog::new().pick_folder();
    if out.is_none() { return; }
    match crate::shader_converter::convert_isf_directory_to_wgsl(&src.unwrap(), &out.unwrap()) {
        Ok(report) => {
            println!("Converted {} ISF files to WGSL", report.len());
        }
        Err(e) => println!("Batch ISF conversion failed: {}", e),
    }
}

fn convert_current_glsl_to_wgsl(ui_state: &mut EditorUiState) {
    match crate::shader_converter::glsl_to_wgsl_full(&ui_state.draft_code) {
        Ok(wgsl) => ui_state.draft_code = wgsl,
        Err(e) => println!("GLSL→WGSL conversion failed: {}", e),
    }
}

fn convert_current_hlsl_to_wgsl(ui_state: &mut EditorUiState) {
    match crate::shader_converter::hlsl_to_wgsl_full(&ui_state.draft_code) {
        Ok(wgsl) => ui_state.draft_code = wgsl,
        Err(e) => println!("HLSL→WGSL conversion failed: {}", e),
    }
}

fn export_current_wgsl_to_glsl(ui_state: &EditorUiState) {
    match crate::shader_converter::wgsl_to_glsl(&ui_state.draft_code) {
        Ok(glsl) => {
            if let Some(save_path) = rfd::FileDialog::new().save_file() {
                let _ = std::fs::write(save_path, glsl);
            }
        }
        Err(e) => println!("WGSL→GLSL export failed: {}", e),
    }
}

fn export_current_wgsl_to_hlsl(ui_state: &EditorUiState) {
    match crate::shader_converter::wgsl_to_hlsl(&ui_state.draft_code) {
        Ok(hlsl) => {
            if let Some(save_path) = rfd::FileDialog::new().save_file() {
                let _ = std::fs::write(save_path, hlsl);
            }
        }
        Err(e) => println!("WGSL→HLSL export failed: {}", e),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectData {
    node_graph: crate::node_graph::NodeGraph,
    draft_code: Option<String>,
    timeline: crate::timeline::Timeline,
    param_index_map: std::collections::HashMap<String, usize>,
}

fn export_project_json(ui_state: &EditorUiState) -> Result<(), String> {
    let proj = ProjectData {
        node_graph: ui_state.node_graph.clone(),
        draft_code: Some(ui_state.draft_code.clone()),
        timeline: ui_state.timeline.clone(),
        param_index_map: ui_state.param_index_map.clone(),
    };
    let json = serde_json::to_string_pretty(&proj).map_err(|e| e.to_string())?;
    let path = rfd::FileDialog::new().add_filter("Project", &["json"]).set_directory(".").set_title("Save Project").save_file();
    if let Some(p) = path { std::fs::write(&p, json).map_err(|e| e.to_string())?; }
    Ok(())
}

fn import_project_json() -> Result<ProjectData, String> {
    let path = rfd::FileDialog::new().add_filter("Project", &["json"]).set_directory(".").set_title("Open Project").pick_file();
    if let Some(p) = path {
        let s = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let proj: ProjectData = serde_json::from_str(&s).map_err(|e| e.to_string())?;
        Ok(proj)
    } else {
        Err("No file selected".to_string())
    }
}

fn default_wgsl_template() -> String {
    r#"
struct Uniforms {
  time: f32,
  resolution: vec2<f32>,
  mouse: vec2<f32>,
  audio_volume: f32,
  audio_bass: f32,
  audio_mid: f32,
  audio_treble: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> params: array<vec4<f32>, 16>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
  var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 3.0,  1.0),
  );
  let pos = positions[vertex_index];
  return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let uv = pos.xy / uniforms.resolution;
  let p0 = params[0].x;
  let t = uniforms.time;
  let base = 0.5 + 0.5 * sin(t);
  return vec4<f32>(uv.x * (1.0 + 0.2 * p0), uv.y, base, 1.0);
}
"#.to_string()
}

fn save_draft_wgsl_to_assets(ui_state: &EditorUiState) {
    let dialog = rfd::FileDialog::new()
        .add_filter("WGSL", &["wgsl"])
        .set_directory("assets/shaders")
        .set_title("Save WGSL Draft As");
    if let Some(path) = dialog.save_file() {
        match std::fs::write(&path, &ui_state.draft_code) {
            Ok(_) => println!("Saved WGSL draft to {:?}", path),
            Err(e) => println!("Failed to save WGSL: {}", e),
        }
    } else {
        println!("Save cancelled");
    }
}

fn export_recorded_frames_to_mp4() {
    use std::process::Command;
    let input_pattern = std::path::Path::new("assets/output/frame_%05d.png");
    let first_frame = std::path::Path::new("assets/output/frame_00000.png");
    if !first_frame.exists() {
        println!("No recorded frames found in assets/output/ (start recording in Preview panel)");
        return;
    }
    let dialog = rfd::FileDialog::new()
        .add_filter("MP4", &["mp4"])
        .set_directory("assets/output")
        .set_title("Export MP4");
    if let Some(out_path) = dialog.save_file() {
        let out_str = out_path.to_string_lossy().to_string();
        let input_str = input_pattern.to_string_lossy().to_string();
        println!("Running ffmpeg to export MP4: {}", out_str);
        let status = Command::new("ffmpeg")
            .args([
                "-hide_banner", "-loglevel", "error",
                "-framerate", "60",
                "-i", &input_str,
                "-pix_fmt", "yuv420p",
                "-y", &out_str,
            ])
            .status();
        match status {
            Ok(s) if s.success() => println!("Exported MP4 to {}", out_str),
            Ok(s) => println!("ffmpeg exited with code {:?}", s.code()),
            Err(e) => println!("Failed to run ffmpeg: {} (ensure ffmpeg is on PATH)", e),
        }
    } else {
        println!("Export cancelled");
    }
}
// removed deprecated attribute; updated calls to modern egui API