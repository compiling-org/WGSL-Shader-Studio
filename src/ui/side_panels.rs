use super::state::{EditorUiState, OutputsMode, RightSidebarMode};
use crate::audio_system::AudioAnalyzer;
use crate::compute_pass_integration::ComputePassManager;
use crate::midi_system::MidiSystem;
use crate::ndi_output::{NdiConfig, NdiOutput, NdiUI};
use crate::osc_control::{OscConfig, OscControl, OscUI};
use crate::screenshot_video_export::ScreenshotVideoExporter;
use crate::spout_syphon_output::{SpoutSyphonConfig, SpoutSyphonOutput, SpoutSyphonUI};
use bevy::prelude::*;
use bevy_egui::egui;
use std::path::Path;

pub fn draw_editor_side_panels(
    ctx: &egui::Context,
    ui_state: &mut EditorUiState,
    audio_analyzer: &AudioAnalyzer,
    gesture_control: &mut crate::gesture_control::GestureControlSystem,
    compute_pass_manager: &mut ComputePassManager,
    video_exporter: Option<&ScreenshotVideoExporter>,
    midi_system: &mut MidiSystem,
    osc_config: &mut OscConfig,
    osc_control: &mut OscControl,
    spout_config: &mut SpoutSyphonConfig,
    spout_output: &mut SpoutSyphonOutput,
    ndi_config: &mut NdiConfig,
    ndi_output: &mut NdiOutput,
    scene_editor_state: Option<&mut crate::scene_editor_3d::SceneEditor3DState>,
    manipulable_query: Option<
        &Query<(Entity, &Name), With<crate::scene_editor_3d::EditorManipulable>>,
    >,
) {
    // Left panel: Shader Browser (via editor_ui logic for now)
    if ui_state.show_shader_browser {
        // This will call back to a function in editor_ui or we move it later
        crate::editor_ui::draw_editor_shader_browser_panel(ctx, ui_state);
    }

    egui::SidePanel::right("right_modes_panel").resizable(true).show(ctx, |ui| {
        ui.horizontal(|ui| {
            for (mode, label) in [
                (RightSidebarMode::Parameters, "Parameters"),
                (RightSidebarMode::Compute, "Compute"),
                (RightSidebarMode::Outputs, "Outputs"),
                (RightSidebarMode::OSC, "OSC"),
                (RightSidebarMode::Audio, "Audio"),
                (RightSidebarMode::MIDI, "MIDI"),
                (RightSidebarMode::Gestures, "Gestures"),
                (RightSidebarMode::Performance, "Performance"),
                (RightSidebarMode::Scene3D, "3D Scene"),
            ] {
                let sel = ui_state.right_sidebar_mode == mode;
                if ui.selectable_label(sel, label).clicked() {
                    ui_state.right_sidebar_mode = mode;
                }
            }
        });
        ui.separator();

        match ui_state.right_sidebar_mode {
            RightSidebarMode::MIDI => {
                ui.heading("MIDI");
                if ui.button("Scan Devices").clicked() {
                    let _ = midi_system.scan_devices();
                }
                ui.separator();
                let devices_snapshot = midi_system.devices.clone();
                for (i, dev) in devices_snapshot.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(&dev.name);
                        if dev.connected {
                            if ui.button("Disconnect").clicked() {
                                let _ = midi_system.disconnect_device(i);
                            }
                        } else {
                            if ui.button("Connect").clicked() {
                                let _ = midi_system.connect_device(i);
                            }
                        }
                    });
                }
                ui.separator();
                ui.heading("Parameter Mapping");
                let params = crate::editor_ui::parse_shader_parameters(&ui_state.draft_code);
                if params.is_empty() {
                    ui.label("No shader parameters available");
                } else {
                    egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        for p in params.iter() {
                            ui.horizontal(|ui| {
                                ui.label(&p.name);
                                if ui.button("Learn").clicked() {
                                    midi_system.start_midi_learn(&p.name);
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Channel");
                                ui.add(egui::DragValue::new(&mut ui_state.current_midi_channel).range(1..=16));
                                ui.label("CC");
                                ui.add(egui::DragValue::new(&mut ui_state.current_midi_number).range(0..=127));
                                if ui.button("Map CC").clicked() {
                                    let mapping = crate::midi_system::MidiMapping {
                                        parameter_name: p.name.clone(),
                                        midi_type: crate::midi_system::MidiMessageType::ControlChange,
                                        channel: ui_state.current_midi_channel,
                                        number: ui_state.current_midi_number,
                                        min_value: 0.0,
                                        max_value: 1.0,
                                        curve: crate::midi_system::MidiCurve::Linear,
                                        invert: false,
                                        smoothing: 0.0,
                                    };
                                    midi_system.add_mapping(mapping);
                                }
                            });
                            if let Some(existing) = midi_system.get_mapping(&p.name) {
                                ui.label(format!("Mapped: ch {} CC {}", existing.channel, existing.number));
                                if ui.button("Remove Mapping").clicked() {
                                    midi_system.remove_mapping(&p.name);
                                }
                            }
                            ui.separator();
                        }
                    });
                }
            }
            RightSidebarMode::Audio => {
                ui.heading("Audio");
                let data = audio_analyzer.get_audio_data();
                ui.horizontal(|ui| {
                    ui.label(format!("Volume: {:.2}", data.volume));
                    ui.label(format!("Bass: {:.2}", data.bass_level));
                    ui.label(format!("Mid: {:.2}", data.mid_level));
                    ui.label(format!("Treble: {:.2}", data.treble_level));
                });
                let graph_height = 80.0;
                let graph_width = ui.available_width();
                let (response, painter) = ui.allocate_painter(egui::Vec2::new(graph_width, graph_height), egui::Sense::hover());
                let rect = response.rect;
                let bg = egui::Color32::from_gray(30);
                painter.rect_filled(rect, egui::CornerRadius::same(0u8), bg);
                let bars = 32usize;
                let mut max_val = 1.0f32;
                if !data.frequencies.is_empty() {
                    max_val = data.frequencies.iter().cloned().fold(0.0f32, f32::max).max(1.0);
                }
                let bar_w = rect.width() / bars as f32;
                for i in 0..bars {
                    let v = if i < data.frequencies.len() { data.frequencies[i] } else { 0.0 };
                    let h = rect.height() * (v / max_val).clamp(0.0, 1.0);
                    let x0 = rect.min.x + i as f32 * bar_w;
                    let x1 = x0 + bar_w * 0.9;
                    let y0 = rect.max.y - h;
                    let y1 = rect.max.y;
                    let color = egui::Color32::from_rgb(80, (120 + (i as i32 % 80)) as u8, 220);
                    painter.rect_filled(egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1)), egui::CornerRadius::same(2u8), color);
                }
                ui.separator();
                
                // Audio Feature Bindings (Fosfora-style)
                ui.heading("Audio Feature Bindings");
                ui.label("Map audio features to shader parameters");
                ui.separator();
                
                let available_features = [
                    "sub_bass", "bass", "low_mid", "mid", "upper_mid", "presence", "brilliance",
                    "rms", "kick",
                    "centroid", "flux", "flatness", "rolloff", "bandwidth", "zcr",
                    "onset", "beat", "beat_phase", "bpm", "beat_strength",
                    "mfcc_0", "mfcc_1", "mfcc_2", "mfcc_3", "mfcc_4", "mfcc_5", "mfcc_6",
                    "mfcc_7", "mfcc_8", "mfcc_9", "mfcc_10", "mfcc_11", "mfcc_12",
                    "chroma_c0", "chroma_c1", "chroma_c2", "chroma_c3", "chroma_c4", "chroma_c5",
                    "chroma_c6", "chroma_c7", "chroma_c8", "chroma_c9", "chroma_c10", "chroma_c11",
                    "dominant_chroma",
                    "loudness_m", "loudness_s", "loudness_trend",
                    "key_class", "key_is_minor", "key_confidence",
                    "downbeat", "bar_phase", "beat_in_bar",
                    "pan", "stereo_width", "stereo_corr",
                    "section_novelty", "buildup", "drop",
                    "percussive_energy", "harmonic_energy", "harmonic_ratio",
                    "pitch", "pitch_confidence",
                    "contrast_0", "contrast_1", "contrast_2", "contrast_3", "contrast_4", "contrast_5",
                    "contrast_mean", "timbre_flux",
                    "band_pan_sub_bass", "band_pan_bass", "band_pan_low_mid", "band_pan_mid",
                    "band_pan_upper_mid", "band_pan_presence", "band_pan_brilliance",
                    "bar_index", "beat_index",
                ];
                
                let params = crate::editor_ui::parse_shader_parameters(&ui_state.draft_code);
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                
                if param_names.is_empty() {
                    ui.label("No shader parameters found - add parameters to your shader");
                } else {
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        if ui.button("Add Binding").clicked() {
                            ui_state.audio_bindings.push(crate::ui::state::AudioUniformBinding::default());
                        }
                        
                        for binding in ui_state.audio_bindings.iter_mut() {
                            ui.horizontal(|ui| {
                                ui.label(&binding.audio_feature);
                                ui.label("\u{2192}");
                                ui.label(&binding.uniform_target);
                                ui.label(format!("scale={:.2}", binding.scale));
                                if ui.button("Remove").clicked() {
                                    if let Some(pos) = ui_state.audio_bindings.iter().position(|b| b.audio_feature == binding.audio_feature && b.uniform_target == binding.uniform_target) {
                                        ui_state.audio_bindings.remove(pos);
                                    }
                                }
                            });
                        }
                    });
                }
                
                ui.separator();
                ui.heading("Apply Audio Features");
                if ui.button("Map Selected").clicked() {
                    let mut draft_code = ui_state.draft_code.clone();
                    crate::editor_ui::apply_audio_bindings(&mut draft_code, &mut ui_state);
                    ui_state.draft_code = draft_code;
                }
            }
            RightSidebarMode::Gestures => {
                ui.heading("Gestures");
                ui.checkbox(&mut ui_state.quick_params_enabled, "Enable quick params");
                if ui.button("Calibrate").clicked() {
                    ui_state.show_gesture_calibration = true;
                }
                ui.separator();
                ui.heading("Parameter Mapping");
                let params = crate::editor_ui::parse_shader_parameters(&ui_state.draft_code);
                if params.is_empty() {
                    ui.label("No shader parameters available");
                } else {
                    egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                        for p in params.iter() {
                            ui.horizontal(|ui| {
                                ui.label(&p.name);
                                egui::ComboBox::from_id_source(format!("gesture_combo_{}", &p.name))
                                    .selected_text(format!("{:?}", ui_state.current_gesture_type))
                                    .show_ui(ui, |ui| {
                                        for g in [
                                            crate::gesture_control::GestureType::HandOpen,
                                            crate::gesture_control::GestureType::HandClosed,
                                            crate::gesture_control::GestureType::Point,
                                            crate::gesture_control::GestureType::Pinch,
                                            crate::gesture_control::GestureType::SwipeLeft,
                                            crate::gesture_control::GestureType::SwipeRight,
                                            crate::gesture_control::GestureType::SwipeUp,
                                            crate::gesture_control::GestureType::SwipeDown,
                                            crate::gesture_control::GestureType::Circle,
                                            crate::gesture_control::GestureType::Grab,
                                            crate::gesture_control::GestureType::Release,
                                        ] {
                                            if ui.selectable_label(ui_state.current_gesture_type == g, format!("{:?}", g)).clicked() {
                                                ui_state.current_gesture_type = g;
                                            }
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label("Min");
                                ui.add(egui::DragValue::new(&mut ui_state.current_gesture_min).speed(0.1));
                                ui.label("Max");
                                ui.add(egui::DragValue::new(&mut ui_state.current_gesture_max).speed(0.1));
                                ui.checkbox(&mut ui_state.current_gesture_invert, "Invert");
                            });
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_source(format!("curve_combo_{}", &p.name))
                                    .selected_text(format!("{:?}", ui_state.current_gesture_curve))
                                    .show_ui(ui, |ui| {
                                        for c in [
                                            crate::gesture_control::CurveType::Linear,
                                            crate::gesture_control::CurveType::Quadratic,
                                            crate::gesture_control::CurveType::Cubic,
                                            crate::gesture_control::CurveType::Exponential,
                                            crate::gesture_control::CurveType::Logarithmic,
                                        ] {
                                            if ui.selectable_label(ui_state.current_gesture_curve == c, format!("{:?}", c)).clicked() {
                                                ui_state.current_gesture_curve = c;
                                            }
                                        }
                                    });
                                if ui.button("Map").clicked() {
                                    gesture_control.get_parameter_mappings_mut().insert(
                                        p.name.clone(),
                                        crate::gesture_control::GestureMapping {
                                            gesture: ui_state.current_gesture_type,
                                            parameter_name: p.name.clone(),
                                            min_value: ui_state.current_gesture_min,
                                            max_value: ui_state.current_gesture_max,
                                            curve_type: ui_state.current_gesture_curve,
                                            invert: ui_state.current_gesture_invert,
                                        }
                                    );
                                }
                                if ui.button("Remove").clicked() {
                                    gesture_control.get_parameter_mappings_mut().remove(&p.name);
                                }
                            });
                            if let Some(m) = gesture_control.get_parameter_mappings().get(&p.name) {
                                ui.label(format!("Mapped to {:?} [{:.2}..{:.2}] {:?}", m.gesture, m.min_value, m.max_value, m.curve_type));
                            }
                            ui.separator();
                        }
                    });
                }
            }
            RightSidebarMode::Parameters => {
                ui.heading("Parameters");
                if !ui_state.draft_code.is_empty() {
                    let params = crate::editor_ui::parse_shader_parameters(&ui_state.draft_code);
                    if params.is_empty() {
                        ui.label("No parameters found");
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for param in params.iter() {
                                ui.horizontal(|ui| {
                                    ui.label(&param.name);
                                    let mut v = ui_state.get_parameter_value(&param.name).unwrap_or(0.5);
                                    let min = param.min_value.unwrap_or(0.0);
                                    let max = param.max_value.unwrap_or(1.0);
                                    if ui.add(egui::Slider::new(&mut v, min..=max)).changed() {
                                        ui_state.set_parameter_value(&param.name, v);
                                    }
                                });
                                ui.separator();
                            }
                        });
                    }
                }
            }
            RightSidebarMode::Compute => {
                ui.heading("Compute Passes");
                ui.label("Compute Shader Dispatch");
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut ui_state.compute_pass_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Workgroup Size:");
                    ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_x).speed(1));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_y).speed(1));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut ui_state.compute_workgroup_z).speed(1));
                });
                if ui.button("Create Compute Pass").clicked() {
                    compute_pass_manager.create_ping_pong_texture(
                        &ui_state.compute_pass_name,
                        512,
                        512,
                        crate::compute_pass_integration::TextureFormat::Rgba8Unorm
                    );
                }
                ui.separator();
                ui.label("Create Ping-Pong Texture:");
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut ui_state.pingpong_texture_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.add(egui::DragValue::new(&mut ui_state.pingpong_width).speed(1));
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut ui_state.pingpong_height).speed(1));
                });
                if ui.button("Create Ping-Pong Texture").clicked() {
                    compute_pass_manager.create_ping_pong_texture(
                        &ui_state.pingpong_texture_name,
                        ui_state.pingpong_width,
                        ui_state.pingpong_height,
                        crate::compute_pass_integration::TextureFormat::Rgba8Unorm
                    );
                }
            }
            RightSidebarMode::Outputs => {
                ui.horizontal(|ui| {
                    for (mode, label) in [
                        (OutputsMode::ScreenshotsVideo, "Screenshots/Video"),
                        (OutputsMode::Ndi, "NDI"),
                        (OutputsMode::SpoutSyphon, "Spout/Syphon"),
                        (OutputsMode::Ffgl, "FFGL"),
                    ] {
                        let sel = ui_state.outputs_mode == mode;
                        if ui.selectable_label(sel, label).clicked() {
                            ui_state.outputs_mode = mode;
                        }
                    }
                });
                ui.separator();
                match ui_state.outputs_mode {
                    OutputsMode::ScreenshotsVideo => {
                        if let Some(exporter) = video_exporter {
                            crate::screenshot_video_export::ExportUI::render_export_controls(
                                ui, exporter, &mut ui_state.export_settings, &mut ui_state.video_export_settings
                            );
                        } else {
                            ui.label("Exporter not available");
                        }
                    }
                    OutputsMode::Ndi => {
                        NdiUI::render_ndi_controls(ui, ndi_config, ndi_output);
                    }
                    OutputsMode::SpoutSyphon => {
                        SpoutSyphonUI::render_spout_syphon_controls(ui, spout_config, spout_output);
                    }
                    OutputsMode::Ffgl => {
                        ui.heading("FFGL Plugin Output");
                        ui.separator();

                        ui.label("Plugin Info:");
                        ui.indent("ffgl_info", |ui| {
                            ui.label("Name: ISF Shaders");
                            ui.label("Unique ID: ISFS");
                            ui.label("Type: Effect");
                            ui.label("API Version: 1.5");
                        });

                        ui.separator();
                        ui.label("Status: Initialized (Host Mode)");
                        ui.label("The current shader is exposed as an FFGL source.");

                        ui.separator();
                        if ui.button("Export FFGL Plugin Bundle").clicked() {
                            match crate::ffgl_exporter::FfglExporter::export_bundle(&ui_state.draft_code, Path::new(".")) {
                                Ok(path) => println!("Exported FFGL bundle to: {:?}", path),
                                Err(e) => eprintln!("Failed to export FFGL bundle: {}", e),
                            }
                        }
                        ui.label("(Packages .dll and current shader into a .zip)");
                    }
                }
            }
            RightSidebarMode::OSC => {
                ui.heading("OSC Control");
                OscUI::render_osc_controls(ui, osc_config, osc_control);
            }
            RightSidebarMode::Scene3D => {
                if let (Some(editor_state), Some(query)) = (scene_editor_state, manipulable_query) {
                    crate::editor_ui::draw_3d_scene_panel(ui, editor_state, query);
                } else {
                    ui.heading("3D Scene Editor");
                    ui.label("3D editor not initialized");
                }
            }
            RightSidebarMode::Performance => {
                ui.heading("Performance Metrics");
                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    ui.label(format!("{:.1}", ui_state.fps));
                });
            }
            RightSidebarMode::PenumbraMaterials => {
                ui.heading("Penumbra Materials");
            }
            RightSidebarMode::FosforaEffects => {
                ui.heading("Fosfora Effects");
            }
        }
    });

    if ui_state.show_gesture_calibration {
        egui::SidePanel::right("gesture_calibration_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Gesture Calibration");
                if ui.button("Close").clicked() {
                    ui_state.show_gesture_calibration = false;
                }
            });
    }
}
