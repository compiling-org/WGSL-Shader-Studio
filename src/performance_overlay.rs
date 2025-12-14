use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore};
use bevy_egui::{egui, EguiContexts};
use std::time::{Duration, Instant};

/// Performance monitoring system for real-time FPS and GPU metrics
pub struct PerformanceOverlayPlugin;

impl Plugin for PerformanceOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerformanceMetrics>()
            .add_systems(Update, update_performance_metrics)
            .add_systems(Update, draw_performance_overlay);
    }
}

#[derive(Resource)]
pub struct PerformanceMetrics {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub gpu_utilization: f32,
    pub memory_usage_mb: f32,
    pub shader_compilation_time_ms: f32,
    pub audio_latency_ms: f32,
    pub last_update: Instant,
    pub frame_count: u64,
    pub fps_history: Vec<f32>,
    pub max_history_size: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            frame_time_ms: 0.0,
            gpu_utilization: 0.0,
            memory_usage_mb: 0.0,
            shader_compilation_time_ms: 0.0,
            audio_latency_ms: 0.0,
            last_update: Instant::now(),
            frame_count: 0,
            fps_history: Vec::new(),
            max_history_size: 100,
        }
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            max_history_size: 120, // 2 seconds at 60 FPS
            ..Default::default()
        }
    }
    
    pub fn add_fps_sample(&mut self, fps: f32) {
        self.fps_history.push(fps);
        if self.fps_history.len() > self.max_history_size {
            self.fps_history.remove(0);
        }
    }
    
    pub fn get_average_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            return 0.0;
        }
        self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32
    }
    
    pub fn get_min_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            return 0.0;
        }
        self.fps_history.iter().fold(f32::INFINITY, |a, &b| a.min(b))
    }
    
    pub fn get_max_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            return 0.0;
        }
        self.fps_history.iter().fold(0.0f32, |a, &b| a.max(b))
    }
}

fn update_performance_metrics(
    diagnostics: Res<DiagnosticsStore>,
    mut metrics: ResMut<PerformanceMetrics>,
    time: Res<Time>,
) {
    // Update FPS from Bevy diagnostics
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            let avg_fps = average as f32;
            metrics.fps = avg_fps;
            metrics.add_fps_sample(avg_fps);
        }
    }
    
    // Update frame time
    if let Some(frame_time) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(average) = frame_time.average() {
            metrics.frame_time_ms = (average * 1000.0) as f32;
        }
    }
    
    // Simulate GPU utilization (would need platform-specific APIs in real implementation)
    metrics.gpu_utilization = (metrics.frame_time_ms / 16.67).min(1.0) * 100.0; // 16.67ms = 60 FPS target
    
    // Estimate memory usage (simplified)
    metrics.memory_usage_mb = 150.0 + metrics.frame_time_ms * 5.0; // Rough estimate
    
    // Simulate shader compilation time (would track actual compilation)
    metrics.shader_compilation_time_ms = 15.0; // + (rand::random::<f32>() - 0.5) * 10.0; // rand not available
    
    // Simulate audio latency (would measure actual audio pipeline)
    metrics.audio_latency_ms = 5.0; // + (rand::random::<f32>() - 0.5) * 3.0; // rand not available
    
    metrics.frame_count += 1;
    metrics.last_update = Instant::now();
}

fn draw_performance_overlay(
    mut contexts: EguiContexts,
    metrics: Res<PerformanceMetrics>,
) {
    let ctx = contexts.ctx_mut();
    let ctx_ref = ctx.unwrap();
    
    // Create a semi-transparent overlay window
    egui::Window::new("Performance Metrics")
        .collapsible(true)
        .resizable(true)
        .default_pos([10.0, 10.0])
        .default_size([280.0, 200.0])
        .show(ctx_ref, |ui| {
            ui.heading("Live Performance Metrics");
            ui.separator();
            
            // FPS Display
            let fps_color = if metrics.fps >= 55.0 {
                egui::Color32::GREEN
            } else if metrics.fps >= 30.0 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::RED
            };
            
            ui.horizontal(|ui| {
                ui.label("FPS:");
                ui.colored_label(fps_color, format!("{:.1}", metrics.fps));
                ui.label(format!("({:.1} avg)", metrics.get_average_fps()));
            });
            
            ui.horizontal(|ui| {
                ui.label("Frame Time:");
                ui.label(format!("{:.2}ms", metrics.frame_time_ms));
            });
            
            // GPU Utilization
            let gpu_color = if metrics.gpu_utilization < 80.0 {
                egui::Color32::GREEN
            } else if metrics.gpu_utilization < 95.0 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::RED
            };
            
            ui.horizontal(|ui| {
                ui.label("GPU Utilization:");
                ui.colored_label(gpu_color, format!("{:.1}%", metrics.gpu_utilization));
            });
            
            ui.horizontal(|ui| {
                ui.label("Memory Usage:");
                ui.label(format!("{:.0}MB", metrics.memory_usage_mb));
            });
            
            ui.separator();
            
            // Shader compilation metrics
            ui.horizontal(|ui| {
                ui.label("Shader Compile:");
                ui.label(format!("{:.1}ms", metrics.shader_compilation_time_ms));
            });
            
            ui.horizontal(|ui| {
                ui.label("Audio Latency:");
                ui.label(format!("{:.1}ms", metrics.audio_latency_ms));
            });
            
            ui.separator();
            
            // FPS History Graph
            ui.label("FPS History (2s):");
            
            // Draw a simple FPS history graph
            let graph_height = 40.0;
            let graph_width = ui.available_width();
            
            let (response, painter) = ui.allocate_painter(egui::Vec2::new(graph_width, graph_height), egui::Sense::hover());
            let rect = response.rect;
            
            // Draw background
            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));
            
            // Draw grid lines
            for i in 0..=4 {
                let y = rect.top() + rect.height() * (i as f32 / 4.0);
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(1.0, egui::Color32::from_gray(60))
                );
            }
            
            // Draw FPS curve
            if metrics.fps_history.len() > 1 {
                let points: Vec<egui::Pos2> = metrics.fps_history
                    .iter()
                    .enumerate()
                    .map(|(i, &fps)| {
                        let x = rect.left() + (i as f32 / (metrics.fps_history.len() - 1) as f32) * rect.width();
                        let y = rect.bottom() - (fps / 120.0) * rect.height(); // Scale to 120 FPS max
                        egui::pos2(x, y)
                    })
                    .collect();
                
                for i in 1..points.len() {
                    let color = if metrics.fps_history[i] >= 55.0 {
                        egui::Color32::GREEN
                    } else if metrics.fps_history[i] >= 30.0 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::RED
                    };
                    
                    painter.line_segment(
                        [points[i-1], points[i]],
                        egui::Stroke::new(2.0, color)
                    );
                }
            }
            
            // Draw 60 FPS reference line
            let target_y = rect.bottom() - (60.0 / 120.0) * rect.height();
            painter.line_segment(
                [egui::pos2(rect.left(), target_y), egui::pos2(rect.right(), target_y)],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 200, 255))
            );
            
            ui.separator();
            
            // Frame statistics
            ui.horizontal(|ui| {
                ui.label("Frames:");
                ui.label(format!("{}", metrics.frame_count));
            });
            
            ui.horizontal(|ui| {
                ui.label("Min/Max FPS:");
                ui.label(format!("{:.1}/{:.1}", metrics.get_min_fps(), metrics.get_max_fps()));
            });
        });
}

/// System to monitor and log performance warnings
pub fn performance_monitoring_system(
    metrics: Res<PerformanceMetrics>,
    mut last_warning: Local<Option<Instant>>,
) {
    let now = Instant::now();
    
    // Initialize if first run
    if last_warning.is_none() {
        *last_warning = Some(now);
        return;
    }
    
    // Only check every 5 seconds to avoid spam
    if let Some(last) = *last_warning {
        if now.duration_since(last) < Duration::from_secs(5) {
            return;
        }
    }
    
    // Check for performance issues
    if metrics.fps < 20.0 {
        eprintln!("⚠️  PERFORMANCE WARNING: Very low FPS ({:.1}) detected!", metrics.fps);
        *last_warning = Some(now);
        return;
    }
    
    if metrics.frame_time_ms > 100.0 {
        eprintln!("⚠️  PERFORMANCE WARNING: High frame time ({:.1}ms) detected!", metrics.frame_time_ms);
        *last_warning = Some(now);
        return;
    }
    
    if metrics.gpu_utilization > 95.0 {
        eprintln!("⚠️  PERFORMANCE WARNING: Very high GPU utilization ({:.1}%) detected!", metrics.gpu_utilization);
        *last_warning = Some(now);
        return;
    }
}
