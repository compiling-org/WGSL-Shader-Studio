use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::editor_ui::EditorUiState;
use crate::shader_renderer::ShaderRenderer;
use std::sync::Arc;
use std::sync::Mutex;

pub fn draw_preview_window(
    ctx: &egui::Context, 
    ui_state: &mut EditorUiState, 
    global_renderer: &crate::editor_ui::GlobalShaderRenderer,
    audio_analyzer: Option<&crate::audio_system::AudioAnalyzer>,
    video_exporter: Option<&crate::screenshot_video_export::ScreenshotVideoExporter>,
) {
    // Use CentralPanel for the main preview area
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Shader Preview");
        
        // Add controls for preview settings
        ui.horizontal(|ui| {
            ui.label("Scale Mode:");
            ui.radio_value(&mut ui_state.preview_scale_mode, crate::editor_ui::PreviewScaleMode::Fit, "Fit");
            ui.radio_value(&mut ui_state.preview_scale_mode, crate::editor_ui::PreviewScaleMode::Fill, "Fill");
            ui.radio_value(&mut ui_state.preview_scale_mode, crate::editor_ui::PreviewScaleMode::OneToOne, "1:1");
            
            ui.separator();
            
            ui.label("Resolution:");
            ui.add(egui::DragValue::new(&mut ui_state.preview_resolution.0).clamp_range(64..=4096).suffix("w"));
            ui.label("x");
            ui.add(egui::DragValue::new(&mut ui_state.preview_resolution.1).clamp_range(64..=4096).suffix("h"));
        });
        
        ui.separator();
        
        // Calculate available size for the preview
        let available_size = ui.available_size();
        let preview_size = egui::Vec2::new(
            ui_state.preview_resolution.0 as f32,
            ui_state.preview_resolution.1 as f32,
        );
        
        // Adjust size based on scale mode
        let display_size = match ui_state.preview_scale_mode {
            crate::editor_ui::PreviewScaleMode::Fit => {
                let scale = (available_size.x / preview_size.x).min(available_size.y / preview_size.y).min(1.0);
                egui::Vec2::new(preview_size.x * scale, preview_size.y * scale)
            },
            crate::editor_ui::PreviewScaleMode::Fill => {
                let scale = (available_size.x / preview_size.x).max(available_size.y / preview_size.y);
                egui::Vec2::new(preview_size.x * scale, preview_size.y * scale)
            },
            crate::editor_ui::PreviewScaleMode::OneToOne => preview_size.min(available_size),
        };
        
        // Render the shader
        if let Ok(texture_handle) = crate::editor_ui::compile_and_render_shader(
            &ui_state.draft_code,
            display_size,
            ctx,
            global_renderer,
            &ui_state.parameter_values,
            audio_analyzer,
            video_exporter.map(|_| &())
        ) {
            ui.image((texture_handle.id(), display_size));
        } else {
            // Display error message if shader fails to compile/render
            ui.colored_label(egui::Color32::RED, "Shader compilation failed. Check your code.");
            ui.label("Preview area - shader will render here when valid");
        }
        
        // Add controls below the preview
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Apply Changes").clicked() {
                ui_state.apply_requested = true;
            }
            
            if ui.button("Reset Time").clicked() {
                // Reset time for animation
                ui_state.time = 0.0;
            }
            
            ui.checkbox(&mut ui_state.auto_apply, "Auto Apply");
            
            if ui.button("Capture Frame").clicked() {
                // Trigger frame capture
                println!("Frame capture requested");
            }
            
            if ui.button("Start Recording").clicked() {
                ui_state.is_recording_video = true;
                println!("Started video recording");
            }
            
            if ui.button("Stop Recording").clicked() {
                ui_state.is_recording_video = false;
                println!("Stopped video recording");
            }
            
            ui.label(if ui_state.is_recording_video { "🔴 RECORDING" } else { "● STOPPED" });
        });
    });
}