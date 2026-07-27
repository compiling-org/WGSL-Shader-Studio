use crate::audio_system::AudioAnalyzer;
use crate::compute_pass_integration::ComputePassManager;
use crate::midi_system::MidiSystem;
use crate::ndi_output::{NdiConfig, NdiOutput};
use crate::osc_control::{OscConfig, OscControl};
use crate::screenshot_video_export::ScreenshotVideoExporter;
use crate::spout_syphon_output::{SpoutSyphonConfig, SpoutSyphonOutput};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui::text::LayoutJob;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::performance_overlay::PerformanceMetrics;
pub use crate::ui::state::{
    CentralView, CodeEditorTab, DiagnosticMessage, DiagnosticSeverity, EditorUiState,
    GlobalShaderRenderer, OutputsMode, PipelineMode, PreviewScaleMode, RightSidebarMode,
    ShaderParameter, SourceSet, ThemePreference, UiStartupGate,
};

pub fn draw_performance_viewport_overlay(ui: &mut egui::Ui, metrics: &PerformanceMetrics) {
    let rect = ui.max_rect();
    let overlay_pos = egui::pos2(rect.right() - 10.0, rect.top() + 10.0);

    ui.painter().text(
        overlay_pos,
        egui::Align2::RIGHT_TOP,
        format!(
            "{:.1} FPS\n{}x{}",
            metrics.fps,
            rect.width() as u32,
            rect.height() as u32
        ),
        egui::FontId::proportional(14.0),
        egui::Color32::from_white_alpha(180),
    );
}

pub fn draw_status_bar(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    if !ui_state.show_status_bar {
        return;
    }

    egui::TopBottomPanel::bottom("status_bar")
        .default_height(24.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.small(format!("Status: {}", ui_state.status_message));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(format!(
                        "WGPU: {}",
                        if ui_state.wgpu_initialized {
                            "OK"
                        } else {
                            "Init..."
                        }
                    ));
                    if !ui_state.compilation_error.is_empty() {
                        ui.colored_label(egui::Color32::RED, "🔥 Shader Error");
                    }
                });
            });
        });
}

pub fn draw_preview_area(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    ui_state: &mut EditorUiState,
    audio_analyzer: &AudioAnalyzer,
    spout_output: &mut SpoutSyphonOutput,
    ndi_output: &mut NdiOutput,
    performance_metrics: &PerformanceMetrics,
) {
    let size = ui.available_size();

    // Call our compilation and rendering helper
    let code = ui_state.draft_code.clone();
    match compile_and_render_shader(
        &code,
        size,
        ctx,
        ui_state, // Pass full ui_state for caching
        Some(audio_analyzer),
        None, // video_exporter
    ) {
        Ok(texture) => {
            ui.image(&texture);

            // Draw performance overlay on top of preview
            if ui_state.show_performance_overlay {
                draw_performance_viewport_overlay(ui, performance_metrics);
            }

            // Handle output streaming
            if ui_state.is_recording_video {
                // Placeholder for streaming
            }
        }
        Err(e) => {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
        }
    }
}

pub fn generate_shader_with_wgsl_smith(prompt: &str) -> String {
    // Placeholder for WGSLSmith integration
    format!("// Generated from prompt: {}\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {{\n    return vec4<f32>(1.0, 0.0, 1.0, 1.0);\n}}", prompt)
}

/// Connect audio analysis to shader parameters
pub fn connect_audio_to_parameters(
    ui_state: &mut EditorUiState,
    audio_data: &crate::audio_system::AudioData,
) {
    // Map audio analysis to shader parameters
    let volume_param = audio_data.volume * 2.0; // Amplify for better effect
    let bass_param = audio_data.bass_level * 3.0;
    let mid_param = audio_data.mid_level * 2.0;
    let treble_param = audio_data.treble_level * 2.0;
    let beat_intensity = if audio_data.beat_detected { 1.0 } else { 0.0 };

    // Update parameter values with audio-reactive data
    ui_state.set_parameter_value("audio_volume", volume_param.min(1.0));
    ui_state.set_parameter_value("audio_bass", bass_param.min(1.0));
    ui_state.set_parameter_value("audio_mid", mid_param.min(1.0));
    ui_state.set_parameter_value("audio_treble", treble_param.min(1.0));
    ui_state.set_parameter_value("beat_intensity", beat_intensity);
    ui_state.set_parameter_value("audio_reactive", volume_param.min(1.0));
}

/// CRITICAL: Actually compile and render WGSL shader using existing WGPU infrastructure
fn compile_and_render_shader(
    wgsl_code: &str,
    size: egui::Vec2,
    egui_ctx: &egui::Context,
    ui_state: &mut EditorUiState,
    audio_analyzer: Option<&crate::audio_system::AudioAnalyzer>,
    _video_exporter: Option<&ScreenshotVideoExporter>,
) -> Result<egui::TextureHandle, String> {
    // CRITICAL: Don't early return - keep egui responsive by reusing cached texture
    if wgsl_code.trim().is_empty() || size.x <= 0.0 || size.y <= 0.0 {
        if let Some(ref texture) = ui_state.preview_texture_handle {
            return Ok(texture.clone());
        }
        return Err("Empty shader code or zero preview size".to_string());
    }

    // CRITICAL: Cache reuse based on shader code + size + parameter changes
    let shader_hash = hash_shader_code(wgsl_code);
    let param_hash = hash_parameters(&ui_state.parameter_values);
    let cache_key = (shader_hash, size.x as u32, size.y as u32, param_hash);

    let needs_refresh = ui_state.apply_requested || ui_state.cache_key != cache_key;
    if !needs_refresh {
        if let Some(ref texture) = ui_state.preview_texture_handle {
            return Ok(texture.clone());
        }
    }

    // Attempt to lock renderer without unwrapping for safety
    let mut renderer_guard = match ui_state.global_renderer.renderer.lock() {
        Ok(guard) => guard,
        Err(_) => return Err("Renderer mutex poisoned".to_string()),
    };

    if let Some(ref mut renderer) = *renderer_guard {
        let audio_data = audio_analyzer.map(|analyzer| analyzer.get_audio_data());

        let params = crate::shader_renderer::RenderParameters {
            width: size.x as u32,
            height: size.y as u32,
            time: ui_state.time as f32,
            frame_rate: 60.0,
            audio_data,
        };

        // Robust parameter mapping using naga reflection
        let mut param_array = vec![0.0f32; 64];
        if let Ok(analyzer) = crate::wgsl_reflect_integration::analyze_shader_reflection(wgsl_code)
        {
            for uniform in analyzer.uniforms {
                if uniform.group == 0 && uniform.binding == 1 {
                    if let Some(&value) = ui_state.parameter_values.get(&uniform.name) {
                        let index = (uniform.offset / 4) as usize;
                        if index < 64 {
                            param_array[index] = value;
                        }
                    }
                }
            }
        } else {
            // Fallback to hashing if reflection fails (better than nothing, but should be rare)
            for (name, &value) in ui_state.parameter_values.iter() {
                let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
                let index = (hash as usize) % 64;
                param_array[index] = value;
            }
        }

        match renderer.render_frame(
            wgsl_code,
            &params,
            Some(&param_array),
            None::<crate::audio_system::AudioData>,
        ) {
            Ok(pixel_data) => {
                // CRITICAL: Deterministic upload after apply_requested
                let width = params.width as usize;
                let height = params.height as usize;

                let texture = egui_ctx.load_texture(
                    "shader_preview",
                    egui::ColorImage::from_rgba_unmultiplied([width, height], &pixel_data),
                    egui::TextureOptions::default(),
                );
                ui_state.preview_texture_handle = Some(texture);
                ui_state.cache_key = cache_key;
                ui_state.apply_requested = false;

                return Ok(ui_state.preview_texture_handle.as_ref().unwrap().clone());
            }
            Err(e) => {
                let error_msg = e.to_string();
                ui_state.compilation_error = error_msg.clone();
                drop(renderer_guard);
                sync_error_to_diagnostics(ui_state, &error_msg);
                return Err(format!("Render error: {}", error_msg));
            }
        }
    } else {
        Err("Renderer not initialized".to_string())
    }
}

fn hash_shader_code(code: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    hasher.finish()
}

fn hash_parameters(params: &std::collections::HashMap<String, f32>) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    for (k, v) in params.iter() {
        k.hash(&mut hasher);
        v.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

/// Parse WGPU error string and add to diagnostics
fn sync_error_to_diagnostics(ui_state: &mut EditorUiState, error: &str) {
    ui_state.diagnostics_messages.clear();

    // Look for line numbers in the error string (e.g., "line 42")
    let re = regex::Regex::new(r"line (\d+)").unwrap();
    if let Some(caps) = re.captures(error) {
        if let Ok(line_num) = caps[1].parse::<usize>() {
            ui_state.diagnostics_messages.push(DiagnosticMessage {
                message: error.to_string(),
                severity: DiagnosticSeverity::Error,
                line: Some(line_num),
                column: None,
            });
            return;
        }
    }

    // Fallback if no line number found
    ui_state.diagnostics_messages.push(DiagnosticMessage {
        message: error.to_string(),
        severity: DiagnosticSeverity::Error,
        line: None,
        column: None,
    });
}

/// Render shader to texture for preview
fn render_shader_to_texture(
    wgsl_code: &str,
    size: egui::Vec2,
    renderer: &mut crate::shader_renderer::ShaderRenderer,
    egui_ctx: &egui::Context,
) -> Result<egui::TextureHandle, String> {
    use crate::shader_renderer::RenderParameters;

    let params = RenderParameters {
        width: size.x as u32,
        height: size.y as u32,
        time: 0.0,
        frame_rate: 60.0,
        audio_data: None,
    };

    match renderer.render_frame(wgsl_code, &params, None, None) {
        Ok(pixel_data) => {
            let width = (params.width as usize).max(1);
            let height = (params.height as usize).max(1);

            let texture = egui_ctx.load_texture(
                "shader_preview",
                egui::ColorImage::from_rgba_unmultiplied([width, height], &pixel_data),
                egui::TextureOptions::default(),
            );
            Ok(texture)
        }
        Err(e) => Err(format!("Shader rendering failed: {}", e)),
    }
}

pub fn draw_editor_menu(
    ctx: &egui::Context,
    ui_state: &mut EditorUiState,
    auditor: &mut crate::simple_ui_auditor::SimpleUiAuditor,
) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Shader").clicked() {
                        ui_state.draft_code = String::from("@vertex\nfn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {\n  var positions = array<vec2<f32>, 3>(\n    vec2<f32>(-1.0, -1.0),\n    vec2<f32>( 3.0, -1.0),\n    vec2<f32>(-1.0,  3.0),\n  );\n  return vec4<f32>(positions[i], 0.0, 1.0);\n}\n\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n    return vec4<f32>(0.2, 0.2, 0.2, 1.0);\n}");
                        ui_state.apply_requested = true;
                        ui_state.show_status("New shader created", 2.0);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Open Project...").clicked() {
                        import_project_json(ui_state);
                        ui.close_menu();
                    }
                    if ui.button("Save Project As...").clicked() {
                        export_project_json(ui_state);
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Shader Browser").clicked() { ui_state.show_shader_browser = !ui_state.show_shader_browser; ui.close_menu(); }
                    if ui.button("Parameters").clicked() { ui_state.show_parameter_panel = !ui_state.show_parameter_panel; ui.close_menu(); }
                    if ui.button("Preview").clicked() { ui_state.show_preview = !ui_state.show_preview; ui.close_menu(); }
                    if ui.button("Code Editor").clicked() { ui_state.show_code_editor = !ui_state.show_code_editor; ui.close_menu(); }
                    if ui.button("Diagnostics").clicked() { ui_state.show_diagnostics_panel = !ui_state.show_diagnostics_panel; ui.close_menu(); }
                    if ui.button("Node Graph").clicked() { ui_state.central_view = CentralView::NodeGraph; ui.close_menu(); }
                });

                ui.menu_button("Conversion", |ui| {
                    if ui.button("Import ISF File...").clicked() {
                        import_isf_into_editor(ui_state);
                        ui.close_menu();
                    }
                    if ui.button("Batch Convert ISF Directory...").clicked() {
                        batch_convert_isf_directory();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Transpile GLSL to WGSL").clicked() {
                        convert_current_glsl_to_wgsl(ui_state);
                        ui.close_menu();
                    }
                    if ui.button("Transpile HLSL to WGSL").clicked() {
                        convert_current_hlsl_to_wgsl(ui_state);
                        ui.close_menu();
                    }
                });

                ui.menu_button("Export", |ui| {
                    if ui.button("Export WGSL to GLSL...").clicked() {
                        export_current_wgsl_to_glsl(ui_state);
                        ui.close_menu();
                    }
                    if ui.button("Export WGSL to HLSL...").clicked() {
                        export_current_wgsl_to_hlsl(ui_state);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export as FFGL Bundle...").clicked() {
                        export_current_wgsl_to_ffgl(ui_state);
                        ui.close_menu();
                    }
});
            });
        });
}
fn populate_shader_list_simple(ui_state: &mut EditorUiState) {
    // A simplified rescan helper for the UI button
    rescan_shaders_all(ui_state);
}

pub fn draw_editor_side_panels(
    ctx: &egui::Context,
    ui_state: &mut EditorUiState,
    midi_system: &mut MidiSystem,
    audio_analyzer: &AudioAnalyzer,
    compute_pass_manager: &mut ComputePassManager,
    spout_config: &mut SpoutSyphonConfig,
    spout_output: &mut SpoutSyphonOutput,
    ndi_config: &mut NdiConfig,
    ndi_output: &mut NdiOutput,
    osc_config: &mut OscConfig,
    osc_control: &mut OscControl,
    scene_editor_state: Option<&mut crate::scene_editor_3d::SceneEditor3DState>,
    manipulable_query: Option<
        &Query<(Entity, &Name), With<crate::scene_editor_3d::EditorManipulable>>,
    >,
    gesture_control: &mut crate::gesture_control::GestureControlSystem,
    video_exporter: Option<&ScreenshotVideoExporter>,
) {
    crate::ui::side_panels::draw_editor_side_panels(
        ctx,
        ui_state,
        audio_analyzer,
        gesture_control,
        compute_pass_manager,
        video_exporter,
        midi_system,
        osc_config,
        osc_control,
        spout_config,
        spout_output,
        ndi_config,
        ndi_output,
        scene_editor_state,
        manipulable_query,
    );
}

pub fn populate_shader_list(mut ui_state: ResMut<EditorUiState>) {
    if ui_state.is_scanning_shaders {
        return;
    }

    ui_state.is_scanning_shaders = true;
    ui_state.shader_scan_status = "Scanning for shaders...".to_string();

    // Create a simple channel for communicating the results back
    // In a real Bevy app, we might use a Task directly, but for now,
    // we'll just use a thread and a mechanism to poll for completion.
    // However, since we can't easily capture 'ui_state' across threads,
    // we'll just mark it as "to be scanned" and let a system handle it.
    println!("Shader scan initiated in background...");
}

pub fn background_shader_scan_system(mut ui_state: ResMut<EditorUiState>) {
    if !ui_state.is_scanning_shaders {
        return;
    }

    use once_cell::sync::Lazy;
    use std::sync::mpsc;
    use std::sync::Mutex;

    type ScanResult = (Vec<String>, Vec<String>);
    static SCAN_CHANNEL: Lazy<(mpsc::Sender<ScanResult>, Mutex<mpsc::Receiver<ScanResult>>)> =
        Lazy::new(|| {
            let (tx, rx) = mpsc::channel();
            (tx, Mutex::new(rx))
        });

    // Check if we have results
    if let Ok(receiver) = SCAN_CHANNEL.1.lock() {
        if let Ok(results) = receiver.try_recv() {
            ui_state.available_shaders_all = results.0;
            ui_state.available_shaders_compatible = results.1;
            ui_state.is_scanning_shaders = false;
            ui_state.shader_scan_status =
                format!("Found {} shaders", ui_state.available_shaders_all.len());
            println!(
                "Background shader scan complete: {} total",
                ui_state.available_shaders_all.len()
            );
            return;
        }
    }

    // If not started yet, start the thread
    if ui_state.shader_scan_status == "Scanning for shaders..." {
        ui_state.shader_scan_status = "Scanning in progress...".to_string();
        println!("🚀 Spawning background scan thread...");
        let tx = SCAN_CHANNEL.0.clone();
        std::thread::spawn(move || {
            let mut found_all = Vec::new();

            // Search in assets and shaders (assuming standard project structure)
            let paths = [Path::new("./assets"), Path::new("./shaders")];
            for path in paths.iter() {
                if path.exists() {
                    collect_wgsl_files(path, &mut found_all);
                    collect_isf_files(path, &mut found_all);
                }
            }

            // Also search common ISF locations
            let isf_dirs = ["./assets/isf", "./assets/ISF", "./isf-shaders"];

            for dir_str in isf_dirs.iter() {
                let p = Path::new(dir_str);
                if p.exists() {
                    collect_isf_files(p, &mut found_all);
                }
            }

            found_all.sort();
            found_all.dedup();

            let mut compatible = Vec::new();
            for p_str in found_all.iter() {
                if let Ok(src) = std::fs::read_to_string(p_str) {
                    if is_wgsl_shader_compatible(&src) {
                        compatible.push(p_str.clone());
                    }
                }
            }

            let _ = tx.send((found_all, compatible));
        });
    }
}

pub fn collect_wgsl_files(dir: &Path, out: &mut Vec<String>) {
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

pub fn collect_isf_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                // Recursively search subdirectories
                collect_isf_files(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                // Collect both .fs (ISF fragment shaders) and .vs (ISF vertex shaders)
                if ext.eq_ignore_ascii_case("fs")
                    || ext.eq_ignore_ascii_case("vs")
                    || ext.eq_ignore_ascii_case("isf")
                {
                    if let Some(s) = p.to_str() {
                        out.push(s.to_string());
                        println!("Found ISF shader: {}", s);
                    }
                }
            }
        }
    }
}

/// Bottom code editor panel bound to `EditorUiState::draft_code`.
// Helper that draws the code editor panel using a provided egui context
pub fn draw_editor_code_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    crate::ui::code_panel::draw_editor_code_panel(ctx, ui_state);
}

pub fn editor_code_panel(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let ctx = egui_ctx.ctx_mut().expect("Failed to get egui context");
    draw_editor_code_panel(ctx, &mut *ui_state);
}

pub fn draw_editor_shader_browser_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::SidePanel::left("shader_browser")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Shader Browser");

            if ui_state.is_scanning_shaders {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(&ui_state.shader_scan_status);
                });
                ui.separator();
            }

            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.show_all_shaders, "Show all shaders");
                if !ui_state.show_all_shaders {
                    ui.label("Showing compatible only (has @fragment or @compute)");
                }
            });
            ui.horizontal(|ui| {
                for (src, label) in [
                    (SourceSet::All, "All Sources"),
                    (SourceSet::Assets, "Assets"),
                    (SourceSet::ISF, "ISF"),
                ] {
                    let sel = ui_state.selected_source == src;
                    if ui.selectable_label(sel, label).clicked() {
                        ui_state.selected_source = src;
                        match src {
                            SourceSet::All => rescan_shaders_all(ui_state),
                            SourceSet::Assets => rescan_shaders_assets_only(ui_state),
                            SourceSet::ISF => rescan_shaders_isf_only(ui_state),
                        }
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut ui_state.search_query);
            });
            ui.horizontal(|ui| {
                if ui.button("Rescan (All)").clicked() {
                    rescan_shaders_all(ui_state);
                }
                if ui.button("Rescan (ISF only)").clicked() {
                    rescan_shaders_isf_only(ui_state);
                }
            });
            ui.horizontal(|ui| {
                let mut current_cat = ui_state
                    .selected_category
                    .clone()
                    .unwrap_or_else(|| "All".to_string());
                for cat in ["All", "ISF", "WGSL", "GLSL", "HLSL"] {
                    let selected = current_cat == cat;
                    if ui.selectable_label(selected, cat).clicked() {
                        current_cat = cat.to_string();
                    }
                }
                ui_state.selected_category = Some(current_cat);
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let mut names = if ui_state.show_all_shaders {
                        ui_state.available_shaders_all.clone()
                    } else {
                        ui_state.available_shaders_compatible.clone()
                    };
                    if let Some(cat) = &ui_state.selected_category {
                        names = match cat.as_str() {
                            "ISF" => names
                                .into_iter()
                                .filter(|n: &String| n.to_lowercase().ends_with(".fs"))
                                .collect(),
                            "WGSL" => names
                                .into_iter()
                                .filter(|n: &String| n.to_lowercase().ends_with(".wgsl"))
                                .collect(),
                            "GLSL" => names
                                .into_iter()
                                .filter(|n: &String| n.to_lowercase().ends_with(".glsl"))
                                .collect(),
                            "HLSL" => names
                                .into_iter()
                                .filter(|n: &String| n.to_lowercase().ends_with(".hlsl"))
                                .collect(),
                            _ => names,
                        };
                    }
                    for name in names.iter() {
                        if !ui_state.search_query.is_empty()
                            && !name
                                .to_lowercase()
                                .contains(&ui_state.search_query.to_lowercase())
                        {
                            continue;
                        }
                        let selected = ui.selectable_label(
                            ui_state
                                .selected_shader
                                .as_ref()
                                .map(|s| s == name)
                                .unwrap_or(false),
                            name,
                        );
                        if selected.clicked() {
                            ui_state.selected_shader = Some(name.clone());
                            if let Ok(content) = std::fs::read_to_string(name) {
                                let name_lower = name.to_lowercase();
                                if name_lower.ends_with(".fs") {
                                    ui_state.show_status("Transpiling ISF...", 1.0);
                                    match crate::isf_loader::IsfShader::parse(name, &content) {
                                        Ok(isf_shader) => {
                                            let mut converter =
                                                super::isf_converter::IsfConverter::new();
                                            match converter.convert_to_wgsl(&isf_shader) {
                                                Ok(wgsl_code) => {
                                                    ui_state.draft_code = wgsl_code;
                                                    ui_state.show_status(
                                                        "ISF loaded and transpiled",
                                                        2.0,
                                                    );
                                                }
                                                Err(e) => {
                                                    ui_state.draft_code = content;
                                                    ui_state.show_status(
                                                        format!("ISF conversion failed: {}", e),
                                                        5.0,
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            ui_state.draft_code = content;
                                            ui_state.show_status(
                                                format!("ISF parse failed: {}", e),
                                                5.0,
                                            );
                                        }
                                    }
                                } else if name_lower.ends_with(".glsl") {
                                    ui_state.show_status("Transpiling GLSL...", 1.0);
                                    match super::converter::GLSLConverter::new() {
                                        Ok(mut converter) => {
                                            match converter.convert(&content, name) {
                                                Ok(wgsl) => {
                                                    ui_state.draft_code = wgsl;
                                                    ui_state.show_status(
                                                        "GLSL loaded and transpiled",
                                                        2.0,
                                                    );
                                                }
                                                Err(e) => {
                                                    ui_state.draft_code = content;
                                                    ui_state.show_status(
                                                        format!("GLSL conversion failed: {}", e),
                                                        5.0,
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => ui_state.show_status(
                                            format!("GLSL converter error: {}", e),
                                            5.0,
                                        ),
                                    }
                                } else if name_lower.ends_with(".hlsl") {
                                    ui_state.show_status("Transpiling HLSL...", 1.0);
                                    match super::converter::HLSLConverter::new() {
                                        Ok(mut converter) => {
                                            match converter.convert(&content, name) {
                                                Ok(wgsl) => {
                                                    ui_state.draft_code = wgsl;
                                                    ui_state.show_status(
                                                        "HLSL loaded and transpiled",
                                                        2.0,
                                                    );
                                                }
                                                Err(e) => {
                                                    ui_state.draft_code = content;
                                                    ui_state.show_status(
                                                        format!("HLSL conversion failed: {}", e),
                                                        5.0,
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => ui_state.show_status(
                                            format!("HLSL converter error: {}", e),
                                            5.0,
                                        ),
                                    }
                                } else {
                                    ui_state.draft_code = content;
                                    ui_state.show_status("Shader loaded", 1.0);
                                }
                                ui_state.apply_requested = true;
                            }
                        }
                    }
                });
        });
}

fn rescan_shaders_all(ui_state: &mut EditorUiState) {
    let mut found_all = Vec::new();
    let standard_dirs = ["examples", "assets/shaders", "assets", "shaders"];
    for d in standard_dirs.iter() {
        let path = Path::new(d);
        if path.exists() {
            collect_wgsl_files(path, &mut found_all);
        }
    }
    let isf_dirs = [
        "C:/Program Files/Magic/Modules2/ISF",
        "C:/Program Files/Magic/ISF",
        "C:/Magic/ISF",
        "~/Magic/ISF",
        "~/Documents/Magic/ISF",
        "./isf-shaders",
        "./ISF",
        "./assets/isf",
        "./assets/ISF",
    ];
    for dir_str in isf_dirs.iter() {
        let expanded_path = if dir_str.starts_with("~/") {
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            Path::new(&home_dir).join(&dir_str[2..])
        } else {
            Path::new(dir_str).to_path_buf()
        };
        if expanded_path.exists() {
            collect_isf_files(&expanded_path, &mut found_all);
        }
    }
    found_all.sort();
    found_all.dedup();
    let mut compatible = Vec::new();
    for p in found_all.iter() {
        if let Ok(src) = fs::read_to_string(p) {
            if is_wgsl_shader_compatible(&src) {
                compatible.push(p.clone());
            }
        }
    }
    ui_state.available_shaders_all = found_all;
    ui_state.available_shaders_compatible = compatible;
}

fn rescan_shaders_isf_only(ui_state: &mut EditorUiState) {
    let mut found_all = Vec::new();
    let isf_dirs = [
        "C:/Program Files/Magic/Modules2/ISF",
        "C:/Program Files/Magic/ISF",
        "C:/Magic/ISF",
        "~/Magic/ISF",
        "~/Documents/Magic/ISF",
        "./isf-shaders",
        "./ISF",
        "./assets/isf",
        "./assets/ISF",
    ];
    for dir_str in isf_dirs.iter() {
        let expanded_path = if dir_str.starts_with("~/") {
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            Path::new(&home_dir).join(&dir_str[2..])
        } else {
            Path::new(dir_str).to_path_buf()
        };
        if expanded_path.exists() {
            collect_isf_files(&expanded_path, &mut found_all);
        }
    }
    found_all.sort();
    found_all.dedup();
    ui_state.available_shaders_all = found_all.clone();
    let mut compatible = Vec::new();
    for p in found_all.iter() {
        if let Ok(src) = fs::read_to_string(p) {
            if is_wgsl_shader_compatible(&src) {
                compatible.push(p.clone());
            }
        }
    }
    ui_state.available_shaders_compatible = compatible;
}

fn rescan_shaders_assets_only(ui_state: &mut EditorUiState) {
    let mut found_all = Vec::new();
    let standard_dirs = ["examples", "assets/shaders", "assets", "shaders"];
    for d in standard_dirs.iter() {
        let path = Path::new(d);
        if path.exists() {
            collect_wgsl_files(path, &mut found_all);
        }
    }
    found_all.sort();
    found_all.dedup();
    let mut compatible = Vec::new();
    for p in found_all.iter() {
        if let Ok(src) = fs::read_to_string(p) {
            if is_wgsl_shader_compatible(&src) {
                compatible.push(p.clone());
            }
        }
    }
    ui_state.available_shaders_all = found_all;
    ui_state.available_shaders_compatible = compatible;
}

pub fn draw_editor_parameter_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::SidePanel::right("parameters")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Parameters");
            ui.label("Interactive shader parameters");
            ui.separator();
            if !ui_state.draft_code.is_empty() {
                let params = parse_shader_parameters(&ui_state.draft_code);
                if params.is_empty() {
                    ui.label("No parameters found in shader");
                } else {
                    ui.label(format!("Found {} parameters:", params.len()));
                    ui.separator();
                    for param in params.iter() {
                        ui.horizontal(|ui| {
                            ui.label(&param.name);
                            if let (Some(min), Some(max)) = (param.min_value, param.max_value) {
                                let mut current_val =
                                    param.default_value.unwrap_or((min + max) / 2.0);
                                if ui
                                    .add(egui::Slider::new(&mut current_val, min..=max))
                                    .changed()
                                {
                                    ui_state.set_parameter_value(&param.name, current_val);
                                }
                            } else {
                                let mut current_val = param.default_value.unwrap_or(0.5);
                                if ui
                                    .add(egui::Slider::new(&mut current_val, 0.0..=1.0))
                                    .changed()
                                {
                                    ui_state.set_parameter_value(&param.name, current_val);
                                }
                            }
                        });
                        ui.separator();
                    }
                }
            } else {
                ui.label("Load a shader to see parameters");
            }
        });
}

// 3D Scene Editor panel integrated into right sidebar
pub fn draw_3d_scene_panel(
    ui: &mut egui::Ui,
    editor_state: &mut crate::scene_editor_3d::SceneEditor3DState,
    manipulable_query: &Query<(Entity, &Name), With<crate::scene_editor_3d::EditorManipulable>>,
) {
    ui.heading("3D Scene Controls");
    ui.separator();

    // Manipulation mode buttons
    ui.horizontal(|ui| {
        ui.label("Mode:");
        for mode in [
            crate::scene_editor_3d::ManipulationMode::Translate,
            crate::scene_editor_3d::ManipulationMode::Rotate,
            crate::scene_editor_3d::ManipulationMode::Scale,
        ] {
            if ui
                .selectable_label(
                    editor_state.manipulation_mode == mode,
                    format!("{:?}", mode),
                )
                .clicked()
            {
                editor_state.manipulation_mode = mode;
            }
        }
    });

    ui.separator();

    // Primitive creation
    ui.horizontal(|ui| {
        ui.label("Create:");
        egui::ComboBox::from_id_source("primitive_type_combo")
            .selected_text(format!("{:?}", editor_state.create_primitive_type))
            .show_ui(ui, |ui| {
                for p_type in [
                    crate::scene_editor_3d::PrimitiveType::Cube,
                    crate::scene_editor_3d::PrimitiveType::Sphere,
                    crate::scene_editor_3d::PrimitiveType::Cylinder,
                    crate::scene_editor_3d::PrimitiveType::Plane,
                ] {
                    if ui
                        .selectable_label(
                            editor_state.create_primitive_type == p_type,
                            format!("{:?}", p_type),
                        )
                        .clicked()
                    {
                        editor_state.create_primitive_type = p_type;
                    }
                }
            });
        ui.label("(Ctrl+N)");
    });

    ui.separator();

    // Scene hierarchy
    ui.heading("Scene Hierarchy");
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for (entity, name) in manipulable_query.iter() {
                let is_selected = editor_state.selected_entity == Some(entity);
                let response = ui.selectable_label(
                    is_selected,
                    format!("{} (Entity {:?})", name.as_str(), entity),
                );

                if response.clicked() {
                    editor_state.selected_entity = Some(entity);
                }
            }
        });

    ui.separator();

    // Editor options
    ui.checkbox(&mut editor_state.show_gizmos, "Show Gizmos");
    ui.checkbox(&mut editor_state.enabled, "Editor Enabled");
    ui.checkbox(&mut editor_state.snap_to_grid, "Snap to Grid");

    if editor_state.snap_to_grid {
        ui.horizontal(|ui| {
            ui.label("Grid Size:");
            ui.add(egui::Slider::new(&mut editor_state.grid_size, 0.1..=10.0));
        });
        ui.label("Press G to snap selected entities");
    }

    ui.separator();

    // Instructions
    ui.label("Controls:");
    ui.label("• Left Click: Select entity");
    ui.label("• Right Drag: Orbit camera");
    ui.label("• Middle Drag: Pan camera");
    ui.label("• Mouse Wheel: Zoom in/out");
    ui.label("• Q/Z: Zoom out/in");
    ui.label("• W/E/R: Switch manipulation mode");
    ui.label("• Ctrl+N: Create new primitive");
    ui.label("• G: Snap to grid (when enabled)");
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
    let has_fragment = src.contains("@fragment");
    let has_compute = src.contains("@compute");
    has_fragment || has_compute
}

/// If incompatible, return a clear message; otherwise, Ok(())
pub fn validate_wgsl_entry_points(src: &str) -> Result<(), String> {
    let has_fragment = src.contains("@fragment");
    let has_compute = src.contains("@compute");
    if has_fragment || has_compute {
        Ok(())
    } else {
        Err("Shader must contain @fragment or @compute entry point".to_string())
    }
}

/// Mode-aware validator supporting fragment or compute pipelines.
pub fn validate_wgsl_for_mode(src: &str, mode: PipelineMode) -> Result<(), String> {
    match mode {
        PipelineMode::Fragment => validate_wgsl_entry_points(src),
        PipelineMode::Compute => {
            let has_compute = src.contains("@compute");
            if !has_compute {
                return Err("Missing @compute entry point".to_string());
            }
            Ok(())
        }
    }
}

pub fn highlight_wgsl(ui: &egui::Ui, text: &str, wrap_width: f32) -> Arc<egui::Galley> {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let s = text;
    let mut _line_start = 0;
    for (i, line) in s.lines().enumerate() {
        let mut _idx = 0;
        let mut _in_comment = false;
        while _idx < line.len() {
            // Detect comments
            if !_in_comment {
                if let Some(pos) = line[_idx..].find("//") {
                    // append up to comment normally
                    let before = &line[_idx.._idx + pos];
                    append_tokens(&mut job, before);
                    // append comment
                    let comment = &line[_idx + pos..];
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
    ui.painter().layout_job(job)
}

fn append_tokens(job: &mut LayoutJob, s: &str) {
    // Tokenize by whitespace and punctuation (very naive)
    let mut token = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            token.push(ch);
        } else {
            if !token.is_empty() {
                append_token(job, &token);
                token.clear();
            }
            job.append(
                &ch.to_string(),
                0.0,
                egui::TextFormat {
                    ..Default::default()
                },
            );
        }
    }
    if !token.is_empty() {
        append_token(job, &token);
    }
}

fn append_token(job: &mut LayoutJob, tok: &str) {
    let (color, _italic) = match tok {
        // WGSL attributes and builtins
        "@fragment" | "@vertex" | "@compute" | "@group" | "@binding" | "@location" | "@builtin" => {
            (egui::Color32::from_rgb(180, 120, 255), false)
        }
        // Types
        "f32" | "u32" | "i32" | "vec2" | "vec3" | "vec4" | "mat2x2" | "mat3x3" | "mat4x4" => {
            (egui::Color32::from_rgb(110, 180, 255), false)
        }
        // Keywords
        "struct" | "var" | "let" | "fn" | "return" | "if" | "else" | "for" | "while" | "break"
        | "continue" | "true" | "false" => (egui::Color32::from_rgb(255, 200, 100), false),
        // Common identifiers
        "uniforms" | "time" | "resolution" | "mouse" => (egui::Color32::LIGHT_GRAY, false),
        _ => (egui::Color32::WHITE, false),
    };
    job.append(
        tok,
        0.0,
        egui::TextFormat {
            color,
            ..Default::default()
        },
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
            // Use the advanced ISF converter
            let mut converter = super::converter::ISFParser::new();
            match converter.parse_isf(&content, p.to_str().unwrap_or("unknown")) {
                Ok(isf_shader) => match converter.convert_to_wgsl(&isf_shader) {
                    Ok(wgsl) => {
                        ui_state.draft_code = wgsl;
                        println!("Successfully converted ISF to WGSL");
                    }
                    Err(e) => println!("ISF→WGSL conversion failed: {}", e),
                },
                Err(e) => println!("ISF parse failed: {}", e),
            }
        }
    }
}

fn batch_convert_isf_directory() {
    let src = rfd::FileDialog::new()
        .set_title("Select Source ISF Directory")
        .pick_folder();
    if src.is_none() {
        return;
    }
    let out = rfd::FileDialog::new()
        .set_title("Select Output WGSL Directory")
        .pick_folder();
    if out.is_none() {
        return;
    }

    let src_path = src.unwrap();
    let out_path = out.unwrap();

    println!(
        "Starting batch ISF conversion from {:?} to {:?}",
        src_path, out_path
    );

    let batch_converter = crate::utils::batch_converter::BatchConverter::new();
    match batch_converter.convert_all(&src_path, &out_path) {
        Ok(results) => {
            let total = results.len();
            let success = results.iter().filter(|(_, s)| *s).count();
            println!("Batch conversion finished: {}/{} succeeded", success, total);
        }
        Err(e) => {
            eprintln!("Batch conversion failed: {}", e);
        }
    }
}

fn convert_current_glsl_to_wgsl(ui_state: &mut EditorUiState) {
    match super::converter::GLSLConverter::new() {
        Ok(mut converter) => match converter.convert(&ui_state.draft_code, "input.glsl") {
            Ok(wgsl) => ui_state.draft_code = wgsl,
            Err(e) => println!("GLSL→WGSL conversion failed: {}", e),
        },
        Err(e) => println!("Failed to create GLSL converter: {}", e),
    }
}

fn convert_current_hlsl_to_wgsl(ui_state: &mut EditorUiState) {
    match super::converter::HLSLConverter::new() {
        Ok(mut converter) => match converter.convert(&ui_state.draft_code, "input.hlsl") {
            Ok(wgsl) => ui_state.draft_code = wgsl,
            Err(e) => println!("HLSL→WGSL conversion failed: {}", e),
        },
        Err(e) => println!("Failed to create HLSL converter: {}", e),
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

fn export_current_wgsl_to_ffgl(ui_state: &EditorUiState) {
    if let Some(save_dir) = rfd::FileDialog::new()
        .set_title("Select FFGL Export Directory")
        .pick_folder()
    {
        match crate::ffgl_exporter::FfglExporter::export_bundle(&ui_state.draft_code, &save_dir) {
            Ok(path) => println!("FFGL bundle exported successfully to {:?}", path),
            Err(e) => println!("FFGL export failed: {}", e),
        }
    }
}

fn show_transpiler_panel(ui_state: &mut EditorUiState) {
    // Create a comprehensive transpiler panel with multiple language support
    println!("Opening multi-language transpiler panel...");

    // This function will be called to show a dedicated transpiler window
    // For now, we'll create a simple implementation that can be expanded
    ui_state.show_wgslsmith_panel = true; // Use the existing WGSLSmith panel for transpiler features

    // Add transpiler-specific test cases
    let test_cases = vec![
        ("GLSL Basic", "// Basic GLSL shader\nvoid main() {\n    gl_FragColor = vec4(1.0);\n}"),
        ("HLSL Basic", "// Basic HLSL shader\nfloat4 main() : SV_TARGET {\n    return float4(1.0, 1.0, 1.0, 1.0);\n}"),
        ("WGSL Basic", "// Basic WGSL shader\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n    return vec4<f32>(1.0, 1.0, 1.0, 1.0);\n}"),
    ];

    for (name, code) in test_cases {
        println!("Transpiler test case available: {}", name);
        // In a full implementation, these would be loaded into the transpiler panel
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectData {
    pub draft_code: String,
    pub parameter_values: std::collections::HashMap<String, f32>,
    pub node_graph: crate::node_graph::NodeGraph,
    pub timeline: crate::timeline::TimelineAnimation,
}

pub fn export_project_json(ui_state: &mut EditorUiState) {
    let proj = ProjectData {
        draft_code: ui_state.draft_code.clone(),
        parameter_values: ui_state.parameter_values.clone(),
        node_graph: ui_state.node_graph.clone(),
        timeline: ui_state.timeline.clone(),
    };

    match serde_json::to_string_pretty(&proj) {
        Ok(json) => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Project", &["json"])
                .set_title("Save Project")
                .save_file()
            {
                if let Err(e) = std::fs::write(&path, json) {
                    ui_state.show_status(format!("Error saving: {}", e), 5.0);
                } else {
                    ui_state.show_status("Project saved successfully", 3.0);
                }
            }
        }
        Err(e) => ui_state.show_status(format!("Serialization error: {}", e), 5.0),
    }
}

pub fn import_project_json(ui_state: &mut EditorUiState) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Project", &["json"])
        .set_title("Open Project")
        .pick_file()
    {
        match std::fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<ProjectData>(&json) {
                Ok(proj) => {
                    ui_state.draft_code = proj.draft_code;
                    ui_state.parameter_values = proj.parameter_values;
                    ui_state.node_graph = proj.node_graph;
                    ui_state.timeline = proj.timeline;
                    ui_state.apply_requested = true;
                    ui_state.show_status("Project loaded successfully", 3.0);
                }
                Err(e) => {
                    ui_state.show_status(format!("Load error: {}", e), 5.0);
                }
            },
            Err(e) => {
                ui_state.show_status(format!("Error reading file: {}", e), 5.0);
            }
        }
    }
}

/// Helper function to extract pixel data from an egui texture handle
fn get_texture_pixels(
    texture_handle: &egui::TextureHandle,
    ctx: &egui::Context,
) -> Result<Vec<u8>, String> {
    // This is a simplified implementation - in a real implementation you'd need to
    // access the underlying GPU texture data, which requires more complex WGPU integration
    // For now, we'll return a placeholder
    Ok(vec![0u8; 4 * 800 * 600]) // RGBA placeholder
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
"#
    .to_string()
}

fn load_particle_physics_example(ui_state: &mut EditorUiState) {
    if let Ok(content) = std::fs::read_to_string("examples/particle_physics.wgsl") {
        ui_state.draft_code = content;
        println!("Loaded particle physics example");
    } else {
        println!("Failed to load particle physics example - file not found");
        // Fallback to default template
        ui_state.draft_code = default_wgsl_template();
    }
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
                "-hide_banner",
                "-loglevel",
                "error",
                "-framerate",
                "60",
                "-i",
                &input_str,
                "-pix_fmt",
                "yuv420p",
                "-y",
                &out_str,
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

/// Parse shader code for parameters (uniforms, textures, etc.)
pub fn parse_shader_parameters(shader_code: &str) -> Vec<ShaderParameter> {
    let mut parameters = Vec::new();

    // First, try to parse ISF metadata if this is an ISF shader
    if let Some(isf_params) = parse_isf_parameters(shader_code) {
        return isf_params;
    }

    // Fall back to robust WGSL reflection using naga
    if let Ok(analyzer) = crate::wgsl_reflect_integration::analyze_shader_reflection(shader_code) {
        for uniform in analyzer.uniforms {
            parameters.push(ShaderParameter {
                name: uniform.name,
                wgsl_type: format!(
                    "{}<{}>",
                    uniform.type_info.base_type, uniform.type_info.components
                ),
                group: uniform.group,
                binding: uniform.binding,
                value: 0.5, // Default value
                default_value: None,
                min_value: None,
                max_value: None,
            });
        }

        // Also capture textures as parameters if needed for UI mapping
        for texture in analyzer.textures {
            parameters.push(ShaderParameter {
                name: texture.name,
                wgsl_type: texture.texture_type,
                group: texture.group,
                binding: texture.binding,
                value: 0.0,
                default_value: None,
                min_value: None,
                max_value: None,
            });
        }

        if !parameters.is_empty() {
            return parameters;
        }
    }

    // Final fallback if reflection fails for any reason
    parameters
}

/// Parse ISF parameters from shader code containing ISF metadata
fn parse_isf_parameters(shader_code: &str) -> Option<Vec<ShaderParameter>> {
    // Look for ISF JSON metadata in comments
    if let Some(json_start) = shader_code.find("/*{") {
        if let Some(json_end) = shader_code[json_start..].find("}*/") {
            let json_str = &shader_code[json_start + 2..json_start + json_end + 1];
            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(json_str) {
                let mut parameters = Vec::new();

                // Parse ISF inputs
                if let Some(inputs_json) = metadata.get("INPUTS") {
                    if let Some(inputs_array) = inputs_json.as_array() {
                        for (index, input_json) in inputs_array.iter().enumerate() {
                            if let Some(name) = input_json.get("NAME").and_then(|n| n.as_str()) {
                                let input_type = input_json
                                    .get("TYPE")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("float");

                                let default = input_json
                                    .get("DEFAULT")
                                    .and_then(|d| d.as_f64())
                                    .map(|d| d as f32);

                                let min = input_json
                                    .get("MIN")
                                    .and_then(|m| m.as_f64())
                                    .map(|m| m as f32);

                                let max = input_json
                                    .get("MAX")
                                    .and_then(|m| m.as_f64())
                                    .map(|m| m as f32);

                                parameters.push(ShaderParameter {
                                    name: name.to_string(),
                                    wgsl_type: map_isf_type_to_wgsl(input_type),
                                    group: 0, // ISF inputs typically use group 0
                                    binding: index as u32,
                                    value: default.unwrap_or(0.5), // Use default value or 0.5
                                    default_value: default,
                                    min_value: min,
                                    max_value: max,
                                });
                            }
                        }
                    }
                }

                return Some(parameters);
            }
        }
    }

    None
}

/// Map ISF input types to WGSL types
fn map_isf_type_to_wgsl(isf_type: &str) -> String {
    match isf_type.to_lowercase().as_str() {
        "float" => "f32".to_string(),
        "bool" => "bool".to_string(),
        "color" => "vec4<f32>".to_string(),
        "point2d" => "vec2<f32>".to_string(),
        "image" => "texture_2d<f32>".to_string(),
        _ => "f32".to_string(), // Default to float
    }
}

/// Check WGSL code for common issues and return diagnostic messages
pub fn check_wgsl_diagnostics(wgsl_code: &str) -> Vec<DiagnosticMessage> {
    let mut diagnostics = Vec::new();

    // Check for basic syntax issues
    if wgsl_code.trim().is_empty() {
        diagnostics.push(DiagnosticMessage {
            severity: DiagnosticSeverity::Error,
            message: "Shader code is empty".to_string(),
            line: None,
            column: None,
        });
        return diagnostics;
    }

    // Check for required entry points
    let has_vertex = wgsl_code.contains("@vertex");
    let has_fragment = wgsl_code.contains("@fragment");
    let has_compute = wgsl_code.contains("@compute");

    if !has_vertex && !has_fragment && !has_compute {
        diagnostics.push(DiagnosticMessage {
            severity: DiagnosticSeverity::Error,
            message: "No entry point found (@vertex, @fragment, or @compute)".to_string(),
            line: None,
            column: None,
        });
    }

    // Check for uniform bindings
    if !wgsl_code.contains("@group") || !wgsl_code.contains("@binding") {
        diagnostics.push(DiagnosticMessage {
            severity: DiagnosticSeverity::Warning,
            message: "No uniform bindings found (@group, @binding)".to_string(),
            line: None,
            column: None,
        });
    }

    // Check for common WGSL syntax issues
    let lines: Vec<&str> = wgsl_code.lines().collect();
    for (line_num, line) in lines.iter().enumerate() {
        let line_number = line_num as usize + 1;

        // Check for missing semicolons (basic check)
        if line.trim().starts_with("var") || line.trim().starts_with("let") {
            if !line.trim().ends_with(';') && !line.trim().is_empty() {
                diagnostics.push(DiagnosticMessage {
                    severity: DiagnosticSeverity::Warning,
                    message: "Possible missing semicolon".to_string(),
                    line: Some(line_number),
                    column: None,
                });
            }
        }

        // Check for invalid type declarations
        if line.contains("float") && !line.contains("f32") {
            diagnostics.push(DiagnosticMessage {
                severity: DiagnosticSeverity::Warning,
                message: "Use 'f32' instead of 'float' in WGSL".to_string(),
                line: Some(line_number),
                column: None,
            });
        }

        // Check for vec3/float mixing issues
        if line.contains("vec3") && line.contains("float") {
            diagnostics.push(DiagnosticMessage {
                severity: DiagnosticSeverity::Warning,
                message: "Possible type mismatch: vec3 with float".to_string(),
                line: Some(line_number),
                column: None,
            });
        }
    }

    // Check for texture sampling issues
    if wgsl_code.contains("textureSample") && !wgsl_code.contains("texture_2d") {
        diagnostics.push(DiagnosticMessage {
            severity: DiagnosticSeverity::Warning,
            message: "textureSample used without texture_2d declaration".to_string(),
            line: None,
            column: None,
        });
    }

    // Check for uniform struct issues
    if wgsl_code.contains("struct") && wgsl_code.contains("uniform") {
        if !wgsl_code.contains("var<uniform>") {
            diagnostics.push(DiagnosticMessage {
                severity: DiagnosticSeverity::Warning,
                message: "Uniform struct should use var<uniform>".to_string(),
                line: None,
                column: None,
            });
        }
    }

    diagnostics
}

/// Run WGSL diagnostics and update UI state
pub fn run_wgsl_diagnostics(ui_state: &mut EditorUiState) {
    ui_state.diagnostics_messages = check_wgsl_diagnostics(&ui_state.draft_code);
}
