use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::editor_ui::EditorUiState;
use crate::audio_system::AudioAnalyzer;

pub fn draw_audio_panel(ctx: &egui::Context, ui_state: &mut EditorUiState, audio_analyzer: &AudioAnalyzer) {
    egui::Window::new("Audio Panel")
        .open(&mut ui_state.show_audio_panel)
        .show(ctx, |ui| {
            ui.heading("Audio Analysis");
            
            let audio_data = audio_analyzer.get_audio_data();
            
            ui.horizontal(|ui| {
                ui.label(format!("Volume: {:.3}", audio_data.volume));
                ui.add(egui::ProgressBar::new(audio_data.volume).show_percentage());
            });
            
            ui.horizontal(|ui| {
                ui.label(format!("Bass: {:.3}", audio_data.bass_level));
                ui.add(egui::ProgressBar::new(audio_data.bass_level).show_percentage());
            });
            
            ui.horizontal(|ui| {
                ui.label(format!("Mid: {:.3}", audio_data.mid_level));
                ui.add(egui::ProgressBar::new(audio_data.mid_level).show_percentage());
            });
            
            ui.horizontal(|ui| {
                ui.label(format!("Treble: {:.3}", audio_data.treble_level));
                ui.add(egui::ProgressBar::new(audio_data.treble_level).show_percentage());
            });
            
            ui.horizontal(|ui| {
                ui.label(format!("Tempo: {:.1} BPM", audio_data.tempo));
                ui.label(if audio_data.beat_detected { "BEAT DETECTED" } else { "no beat" });
            });
            
            // Frequency spectrum visualization
            ui.separator();
            ui.heading("Frequency Spectrum");
            
            let graph_height = 100.0;
            let graph_width = ui.available_width();
            let (response, painter) = ui.allocate_painter(egui::Vec2::new(graph_width, graph_height), egui::Sense::hover());
            let rect = response.rect;
            
            // Draw background
            painter.rect_filled(rect, egui::CornerRadius::same(0.0), egui::Color32::from_gray(30));
            
            // Draw frequency bars
            let bars = audio_data.frequencies.len().min(64); // Limit to 64 bars for performance
            if bars > 0 {
                let bar_width = rect.width() / bars as f32;
                let max_freq = audio_data.frequencies.iter().cloned().fold(0.0f32, f32::max).max(0.001);
                
                for (i, &freq) in audio_data.frequencies.iter().take(bars).enumerate() {
                    let bar_height = (freq / max_freq).min(1.0) * rect.height();
                    let x = rect.min.x + i as f32 * bar_width;
                    let bar_rect = egui::Rect::from_min_size(egui::Pos2::new(x, rect.max.y - bar_height), 
                                                             egui::Vec2::new(bar_width * 0.8, bar_height));
                    
                    let color = if freq > max_freq * 0.7 {
                        egui::Color32::from_rgb(255, 50, 50) // Red for high frequencies
                    } else if freq > max_freq * 0.3 {
                        egui::Color32::from_rgb(50, 255, 50) // Green for medium frequencies
                    } else {
                        egui::Color32::from_rgb(50, 50, 255) // Blue for low frequencies
                    };
                    
                    painter.rect_filled(bar_rect, egui::CornerRadius::same(2.0), color);
                }
            }
            
            // Waveform visualization
            ui.separator();
            ui.heading("Waveform");
            
            let waveform_height = 60.0;
            let waveform_width = ui.available_width();
            let (wf_response, wf_painter) = ui.allocate_painter(egui::Vec2::new(waveform_width, waveform_height), egui::Sense::hover());
            let wf_rect = wf_response.rect;
            
            // Draw background
            wf_painter.rect_filled(wf_rect, egui::CornerRadius::same(0.0), egui::Color32::from_gray(30));
            
            // Draw waveform
            if !audio_data.waveform.is_empty() {
                let points: Vec<egui::Pos2> = audio_data.waveform
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| {
                        let x = wf_rect.min.x + (i as f32 / audio_data.waveform.len() as f32) * wf_rect.width();
                        let y = wf_rect.center().y - (value * wf_rect.height() / 2.0);
                        egui::Pos2::new(x, y)
                    })
                    .collect();
                
                wf_painter.add(egui::Shape::line(points, egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 200, 255))));
            }
            
            // Audio reactive shader parameters
            ui.separator();
            ui.heading("Audio Reactive Parameters");
            
            ui.checkbox(&mut ui_state.quick_params_enabled, "Connect Audio to Parameters");
            if ui_state.quick_params_enabled {
                // Connect audio data to shader parameters
                crate::editor_ui::connect_audio_to_parameters(ui_state, &audio_data);
            }
        });
}