use bevy::prelude::*;
use bevy_egui::egui;
use crate::audio_system::AudioAnalyzer;
use crate::ndi_output::NdiOutput;
use crate::spout_syphon_output::SpoutSyphonOutput;
use crate::timeline::TimelineAnimation;
use super::state::{EditorUiState, CentralView, PreviewScaleMode, OutputsMode};

pub fn draw_editor_central_panel(
    ctx: &egui::Context,
    ui_state: &mut EditorUiState,
    audio_analyzer: &AudioAnalyzer,
    _video_exporter: Option<&crate::screenshot_video_export::ScreenshotVideoExporter>,
    mut node_graph_res: Option<&mut crate::bevy_node_graph_integration_enhanced::NodeGraphResource>,
    scene_state: &crate::scene_editor_3d::SceneEditor3DState,
    timeline_animation: &mut TimelineAnimation,
    spout_output: &mut SpoutSyphonOutput,
    ndi_output: &mut NdiOutput,
    performance_metrics: &crate::performance_overlay::PerformanceMetrics,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            let tabs = [
                (CentralView::Preview, "Preview"),
                (CentralView::NodeGraph, "Node Graph"),
                (CentralView::Scene3D, "3D Editor"),
                (CentralView::Timeline, "Timeline"),
            ];
            for (view, label) in tabs {
                let selected = ui_state.central_view == view;
                if ui.selectable_label(selected, label).clicked() {
                    ui_state.central_view = view;
                }
            }
        });
        ui.separator();

        match ui_state.central_view {
            CentralView::Preview => {
                ui.heading("Shader Preview");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut ui_state.quick_params_enabled, "Quick Params");
                    if ui_state.quick_params_enabled {
                        ui.label("A:");
                        ui.add(egui::Slider::new(&mut ui_state.quick_param_a, 0.0..=1.0));
                        ui.label("B:");
                        ui.add(egui::Slider::new(&mut ui_state.quick_param_b, 0.0..=1.0));
                    }
                });
                
                // Preview rendering logic (simplified for extraction, calls back to global_renderer)
                // This logic is complex and might need more refactoring later
                crate::editor_ui::draw_preview_area(ui, ctx, ui_state, audio_analyzer, spout_output, ndi_output, performance_metrics);
            }
            CentralView::NodeGraph => {
                ui.heading("Node Graph");
                if let Some(ref mut ngr) = node_graph_res {
                    crate::bevy_node_graph_integration_enhanced::draw_node_graph_embedded(ui, ngr);
                } else {
                    ui.label("Node graph not available");
                }
            }
            CentralView::Scene3D => {
                ui.heading("3D Editor");
                if let Some(tex_id) = ui_state.scene3d_texture_id {
                    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    ui.painter().image(tex_id, ui.available_rect_before_wrap(), uv, egui::Color32::WHITE);
                } else {
                    ui.label("3D viewport not ready");
                }
            }
            CentralView::Timeline => {
                ui.heading("Timeline");
                crate::timeline::draw_timeline_ui(ui, timeline_animation);
            }
        }
    });
}
