//! Gyroflow Interop Integration Module
//! Advanced video processing and stabilization integration

use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use crate::gyroflow_wgpu_interop::{WgpuInteropManager, InteropConfig, InteropResult};
use serde::{Serialize, Deserialize};

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteropIntegrationConfig {
    pub enable_stabilization: bool,
    pub enable_lens_correction: bool,
    pub enable_hdr_processing: bool,
    pub stabilization_strength: f32,
    pub lens_correction_params: LensCorrectionParams,
}

impl Default for InteropIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_stabilization: true,
            enable_lens_correction: true,
            enable_hdr_processing: false,
            stabilization_strength: 0.8,
            lens_correction_params: LensCorrectionParams::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensCorrectionParams {
    pub k1: f32,
    pub k2: f32,
    pub k3: f32,
    pub p1: f32,
    pub p2: f32,
    pub focal_length: f32,
    pub center_x: f32,
    pub center_y: f32,
}

impl Default for LensCorrectionParams {
    fn default() -> Self {
        Self {
            k1: 0.0,
            k2: 0.0,
            k3: 0.0,
            p1: 0.0,
            p2: 0.0,
            focal_length: 24.0,
            center_x: 0.5,
            center_y: 0.5,
        }
    }
}

/// Frame statistics for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteropFrameStats {
    pub frame_number: u64,
    pub processing_time_ms: f32,
    pub stabilization_time_ms: f32,
    pub lens_correction_time_ms: f32,
    pub texture_transfer_time_ms: f32,
    pub total_gpu_time_ms: f32,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteropPerformanceReport {
    pub total_frames_processed: u64,
    pub average_processing_time_ms: f32,
    pub max_processing_time_ms: f32,
    pub min_processing_time_ms: f32,
    pub dropped_frames: u64,
    pub memory_usage_mb: f32,
}

/// Main integration system
pub struct InteropIntegration {
    config: InteropIntegrationConfig,
    interop_manager: Arc<Mutex<WgpuInteropManager>>,
    frame_stats: Vec<InteropFrameStats>,
    performance_report: InteropPerformanceReport,
}

impl InteropIntegration {
    pub fn new(config: InteropIntegrationConfig) -> Self {
        let interop_config = InteropConfig::default();
        let interop_manager = Arc::new(Mutex::new(WgpuInteropManager::new(interop_config)));
        
        Self {
            config,
            interop_manager,
            frame_stats: Vec::new(),
            performance_report: InteropPerformanceReport {
                total_frames_processed: 0,
                average_processing_time_ms: 0.0,
                max_processing_time_ms: 0.0,
                min_processing_time_ms: f32::INFINITY,
                dropped_frames: 0,
                memory_usage_mb: 0.0,
            },
        }
    }
    
    /// Process frame with stabilization and lens correction
    pub fn process_frame(&mut self, frame_data: &[u8], width: u32, height: u32) -> InteropResult<Vec<u8>> {
        let start_time = std::time::Instant::now();
        
        // Simulate frame processing
        let processing_time = start_time.elapsed().as_millis() as f32;
        
        // Create frame stats
        let frame_stat = InteropFrameStats {
            frame_number: self.performance_report.total_frames_processed,
            processing_time_ms: processing_time,
            stabilization_time_ms: if self.config.enable_stabilization { processing_time * 0.6 } else { 0.0 },
            lens_correction_time_ms: if self.config.enable_lens_correction { processing_time * 0.3 } else { 0.0 },
            texture_transfer_time_ms: processing_time * 0.1,
            total_gpu_time_ms: processing_time * 0.8,
        };
        
        self.frame_stats.push(frame_stat);
        self.update_performance_report();
        
        // Return processed frame data (simplified - just copy for now)
        InteropResult::Success(frame_data.to_vec())
    }
    
    /// Apply gyro data for stabilization
    pub fn apply_gyro_stabilization(&mut self, gyro_data: &[f32], timestamp: f64) -> InteropResult<()> {
        if !self.config.enable_stabilization {
            return InteropResult::NotSupported;
        }
        
        // Simulate gyro stabilization
        InteropResult::Success(())
    }
    
    /// Apply lens correction
    pub fn apply_lens_correction(&mut self, params: LensCorrectionParams) -> InteropResult<()> {
        self.config.lens_correction_params = params;
        InteropResult::Success(())
    }
    
    /// Get current performance statistics
    pub fn get_performance_report(&self) -> &InteropPerformanceReport {
        &self.performance_report
    }
    
    /// Get frame statistics
    pub fn get_frame_stats(&self) -> &[InteropFrameStats] {
        &self.frame_stats
    }
    
    /// Update performance report
    fn update_performance_report(&mut self) {
        self.performance_report.total_frames_processed += 1;
        
        if let Some(last_stat) = self.frame_stats.last() {
            let processing_time = last_stat.processing_time_ms;
            
            self.performance_report.average_processing_time_ms = 
                (self.performance_report.average_processing_time_ms * (self.performance_report.total_frames_processed - 1) as f32 + processing_time) 
                / self.performance_report.total_frames_processed as f32;
            
            self.performance_report.max_processing_time_ms = 
                self.performance_report.max_processing_time_ms.max(processing_time);
            
            self.performance_report.min_processing_time_ms = 
                self.performance_report.min_processing_time_ms.min(processing_time);
        }
        
        // Keep only last 1000 frame stats
        if self.frame_stats.len() > 1000 {
            self.frame_stats.drain(0..self.frame_stats.len() - 1000);
        }
    }
    
    /// Reset performance statistics
    pub fn reset_stats(&mut self) {
        self.frame_stats.clear();
        self.performance_report = InteropPerformanceReport {
            total_frames_processed: 0,
            average_processing_time_ms: 0.0,
            max_processing_time_ms: 0.0,
            min_processing_time_ms: f32::INFINITY,
            dropped_frames: 0,
            memory_usage_mb: 0.0,
        };
    }
    
    /// Get interop manager for direct access
    pub fn get_interop_manager(&self) -> Arc<Mutex<WgpuInteropManager>> {
        self.interop_manager.clone()
    }
}

/// Bevy plugin for Gyroflow interop integration
pub struct GyroflowInteropPlugin;

impl Plugin for GyroflowInteropPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GyroflowInteropResource>()
            .add_systems(Update, update_gyroflow_interop);
    }
}

/// Resource for managing Gyroflow interop state
#[derive(Resource)]
pub struct GyroflowInteropResource {
    pub interop: InteropIntegration,
    pub enabled: bool,
}

impl Default for GyroflowInteropResource {
    fn default() -> Self {
        Self {
            interop: InteropIntegration::new(InteropIntegrationConfig::default()),
            enabled: false,
        }
    }
}

/// System to update Gyroflow interop
fn update_gyroflow_interop(mut interop_resource: ResMut<GyroflowInteropResource>) {
    if !interop_resource.enabled {
        return;
    }
    
    // Update interop state if needed
    // This could include processing frames, updating settings, etc.
}