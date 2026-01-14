use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::editor_ui::EditorUiState;
use crate::screenshot_video_export::{ExportUI, ExportSettings, VideoExportSettings};

pub fn draw_export_panel(ctx: &egui::Context, ui_state: &mut EditorUiState, export_ui: &mut ExportUI) {
    egui::Window::new("Export Panel")
        .open(&mut ui_state.show_export_panel)
        .show(ctx, |ui| {
            ui.heading("Export Options");
            
            ui.separator();
            ui.heading("Screenshot Settings");
            
            ui.horizontal(|ui| {
                ui.label("Format:");
                ui.radio_value(&mut ui_state.export_settings.image_format, crate::screenshot_video_export::ImageFormat::PNG, "PNG");
                ui.radio_value(&mut ui_state.export_settings.image_format, crate::screenshot_video_export::ImageFormat::JPEG, "JPEG");
                ui.radio_value(&mut ui_state.export_settings.image_format, crate::screenshot_video_export::ImageFormat::BMP, "BMP");
            });
            
            ui.add(egui::Slider::new(&mut ui_state.export_settings.jpeg_quality, 1..=100).text("JPEG Quality"));
            
            ui.separator();
            ui.heading("Video Settings");
            
            ui.horizontal(|ui| {
                ui.label("Format:");
                ui.radio_value(&mut ui_state.video_export_settings.video_format, crate::screenshot_video_export::VideoFormat::MP4, "MP4");
                ui.radio_value(&mut ui_state.video_export_settings.video_format, crate::screenshot_video_export::VideoFormat::AVI, "AVI");
                ui.radio_value(&mut ui_state.video_export_settings.video_format, crate::screenshot_video_export::VideoFormat::MOV, "MOV");
            });
            
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut ui_state.video_export_settings.fps).clamp_range(1..=120).suffix(" FPS"));
                ui.separator();
                ui.add(egui::DragValue::new(&mut ui_state.video_export_settings.width).clamp_range(64..=4096).suffix(" W"));
                ui.add(egui::DragValue::new(&mut ui_state.video_export_settings.height).clamp_range(64..=4096).suffix(" H"));
            });
            
            ui.add(egui::Slider::new(&mut ui_state.video_export_settings.bitrate, 100_000..=100_000_000).text("Bitrate"));
            ui.add(egui::Slider::new(&mut ui_state.video_export_settings.quality, 1..=100).text("Quality"));
            
            ui.separator();
            ui.heading("Export Actions");
            
            ui.horizontal(|ui| {
                if ui.button("Capture Screenshot").clicked() {
                    export_ui.capture_screenshot(&ui_state.export_settings);
                }
                
                if ui.button("Start Recording").clicked() {
                    ui_state.is_recording_video = true;
                    export_ui.start_video_recording(&ui_state.video_export_settings);
                }
                
                if ui.button("Stop Recording").clicked() {
                    ui_state.is_recording_video = false;
                    export_ui.stop_video_recording();
                }
                
                ui.label(if ui_state.is_recording_video { "🔴 RECORDING" } else { "● STOPPED" });
            });
            
            ui.separator();
            ui.heading("Export Queue");
            
            if ui_state.is_recording_video {
                ui.colored_label(egui::Color32::GREEN, "Recording in progress...");
                
                // Show recording progress
                let elapsed = export_ui.get_recording_time();
                ui.label(format!("Recording time: {:.2}s", elapsed));
            } else {
                ui.label("Ready to record");
            }
            
            // Show export history
            if !export_ui.get_export_history().is_empty() {
                ui.separator();
                ui.heading("Recent Exports");
                
                for export in export_ui.get_export_history().iter().rev().take(5) {
                    ui.label(&export);
                }
            }
            
            ui.separator();
            ui.heading("Export Path");
            
            ui.horizontal(|ui| {
                ui.label("Directory:");
                ui.text_edit_singleline(&mut ui_state.export_settings.export_directory);
                
                if ui.button("Browse").clicked() {
                    // In a real implementation, this would open a directory browser
                    println!("Opening directory browser...");
                }
            });
            
            ui.separator();
            ui.heading("Batch Export");
            
            ui.horizontal(|ui| {
                if ui.button("Export Current Frame").clicked() {
                    export_ui.export_current_frame(&ui_state.export_settings);
                }
                
                if ui.button("Batch Export Frames").clicked() {
                    export_ui.batch_export_frames(&ui_state.export_settings);
                }
                
                if ui.button("Export Animation").clicked() {
                    export_ui.export_animation(&ui_state.video_export_settings);
                }
            });
        });
}

#[derive(Debug, Clone)]
pub enum ImageFormat {
    PNG,
    JPEG,
    BMP,
}

#[derive(Debug, Clone)]
pub enum VideoFormat {
    MP4,
    AVI,
    MOV,
}