use bevy::prelude::*;
use bevy_egui::egui;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;

/// Responsive backend system that manages complex shader operations
#[derive(Resource, Clone)]
pub struct ResponsiveBackend {
    /// Thread-safe rendering state
    pub render_state: Arc<Mutex<RenderState>>,
    /// Performance monitoring
    pub performance_monitor: Arc<PerformanceMonitor>,
    /// Responsive UI scaling
    pub ui_scaling: Arc<AtomicU32>,
    /// Backend health status
    pub is_healthy: Arc<AtomicBool>,
}

/// Current rendering state with thread safety
#[derive(Debug, Clone)]
pub struct RenderState {
    pub is_rendering: bool,
    pub current_shader: Option<String>,
    pub render_target_size: (u32, u32),
    pub frame_count: u64,
    pub last_frame_time: f32,
    pub shader_compilation_errors: Vec<String>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            is_rendering: false,
            current_shader: None,
            render_target_size: (512, 512),
            frame_count: 0,
            last_frame_time: 0.0,
            shader_compilation_errors: Vec::new(),
        }
    }
}

/// Performance monitoring with atomic operations
#[derive(Debug)]
pub struct PerformanceMonitor {
    pub fps: AtomicU32,
    pub frame_time_ms: AtomicU32,
    pub memory_usage_mb: AtomicU32,
    pub gpu_utilization: AtomicU32,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            fps: AtomicU32::new(0),
            frame_time_ms: AtomicU32::new(0),
            memory_usage_mb: AtomicU32::new(0),
            gpu_utilization: AtomicU32::new(0),
        }
    }
}

impl ResponsiveBackend {
    /// Create a new responsive backend system
    pub fn new() -> Self {
        Self {
            render_state: Arc::new(Mutex::new(RenderState::default())),
            performance_monitor: Arc::new(PerformanceMonitor::default()),
            ui_scaling: Arc::new(AtomicU32::new(100)), // 100% scaling
            is_healthy: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Update performance metrics
    pub fn update_performance(&self, delta_time: f32) {
        let fps = (1.0 / delta_time.max(0.001)) as u32;
        let frame_time_ms = (delta_time * 1000.0) as u32;
        
        self.performance_monitor.fps.store(fps, Ordering::Relaxed);
        self.performance_monitor.frame_time_ms.store(frame_time_ms, Ordering::Relaxed);
        
        // Update render state
        if let Ok(mut state) = self.render_state.lock() {
            state.frame_count += 1;
            state.last_frame_time = delta_time;
        }
    }

    /// Get current performance data for UI display
    pub fn get_performance_data(&self) -> PerformanceData {
        PerformanceData {
            fps: self.performance_monitor.fps.load(Ordering::Relaxed),
            frame_time_ms: self.performance_monitor.frame_time_ms.load(Ordering::Relaxed),
            memory_usage_mb: self.performance_monitor.memory_usage_mb.load(Ordering::Relaxed),
            gpu_utilization: self.performance_monitor.gpu_utilization.load(Ordering::Relaxed),
        }
    }

    /// Check if backend is responsive and healthy
    pub fn is_responsive(&self) -> bool {
        self.is_healthy.load(Ordering::Relaxed)
    }

    /// Update UI scaling based on window size
    pub fn update_ui_scaling(&self, window_size: egui::Vec2) {
        let base_size = egui::Vec2::new(1920.0, 1080.0);
        let scale_x = window_size.x / base_size.x;
        let scale_y = window_size.y / base_size.y;
        let scale = (scale_x.min(scale_y) * 100.0) as u32;
        
        // Clamp between 50% and 200% scaling
        let clamped_scale = scale.clamp(50, 200);
        self.ui_scaling.store(clamped_scale, Ordering::Relaxed);
    }

    /// Get current UI scaling factor
    pub fn get_ui_scale(&self) -> f32 {
        self.ui_scaling.load(Ordering::Relaxed) as f32 / 100.0
    }

    /// Start shader compilation
    pub fn start_shader_compilation(&self, shader_code: String) -> Result<(), String> {
        if let Ok(mut state) = self.render_state.lock() {
            state.is_rendering = false;
            state.current_shader = Some(shader_code.clone());
            state.shader_compilation_errors.clear();
            Ok(())
        } else {
            Err("Failed to lock render state".to_string())
        }
    }

    /// Complete shader compilation
    pub fn complete_shader_compilation(&self, success: bool, errors: Vec<String>) {
        if let Ok(mut state) = self.render_state.lock() {
            state.is_rendering = success;
            state.shader_compilation_errors = errors;
        }
    }

    /// Get current render state for UI
    pub fn get_render_state(&self) -> RenderState {
        self.render_state.lock().unwrap().clone()
    }
}

/// Performance data for UI display
#[derive(Debug, Clone)]
pub struct PerformanceData {
    pub fps: u32,
    pub frame_time_ms: u32,
    pub memory_usage_mb: u32,
    pub gpu_utilization: u32,
}

/// Plugin for responsive backend systems
pub struct ResponsiveBackendPlugin;

impl Plugin for ResponsiveBackendPlugin {
    fn build(&self, app: &mut App) {
        // Initialize responsive backend
        let backend = ResponsiveBackend::new();
        app.insert_resource(backend.clone());
        
        // Add systems
        app.add_systems(Update, (
            update_responsive_backend,
            monitor_backend_health,
        ));
    }
}

/// System to update responsive backend
fn update_responsive_backend(
    time: Res<Time>,
    backend: Res<ResponsiveBackend>,
    windows: Query<&Window>,
) {
    let delta_time = time.delta_secs();
    backend.update_performance(delta_time);
    
    // Update UI scaling based on window size
    if let Ok(window) = windows.single() {
        let window_size = egui::Vec2::new(window.width(), window.height());
        backend.update_ui_scaling(window_size);
    }
}

/// System to monitor backend health
fn monitor_backend_health(backend: Res<ResponsiveBackend>) {
    // Simple health check - in a real implementation, this would check
    // for GPU timeouts, memory leaks, etc.
    let is_healthy = backend.is_responsive();
    
    if !is_healthy {
        eprintln!("Backend health check failed - system may be unresponsive");
    }
}

/// Extension trait for responsive UI elements
pub trait ResponsiveUi {
    /// Apply responsive scaling to UI elements
    fn apply_responsive_scaling(self, scale: f32) -> Self;
}

impl ResponsiveUi for egui::Ui {
    fn apply_responsive_scaling(self, scale: f32) -> Self {
        // This would apply scaling to the UI context
        // In practice, you'd modify the egui context scaling
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_responsive_backend_creation() {
        let backend = ResponsiveBackend::new();
        assert!(backend.is_responsive());
        assert_eq!(backend.get_ui_scale(), 1.0);
    }

    #[test]
    fn test_performance_monitoring() {
        let backend = ResponsiveBackend::new();
        backend.update_performance(0.016); // 60 FPS
        
        let data = backend.get_performance_data();
        assert_eq!(data.fps, 62); // ~62 FPS
        assert_eq!(data.frame_time_ms, 16); // ~16ms
    }

    #[test]
    fn test_shader_compilation_lifecycle() {
        let backend = ResponsiveBackend::new();
        
        // Start compilation
        let result = backend.start_shader_compilation("test shader".to_string());
        assert!(result.is_ok());
        
        // Complete compilation
        backend.complete_shader_compilation(true, vec![]);
        
        let state = backend.get_render_state();
        assert!(state.is_rendering);
        assert_eq!(state.shader_compilation_errors.len(), 0);
    }
}