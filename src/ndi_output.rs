//! NDI (Network Device Interface) output system for professional video streaming
//! Compatible with OBS Studio, vMix, Wirecast, and other NDI-enabled applications

use bevy::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

/// NDI output configuration
#[derive(Debug, Clone, Resource)]
pub struct NdiConfig {
    pub enabled: bool,
    pub source_name: String,
    pub group_name: String,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    pub bitrate: u32,
    pub multicast: bool,
    pub tcp_mode: bool,
    pub low_bandwidth: bool,
}

impl Default for NdiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            source_name: "WGSL Shader Studio".to_string(),
            group_name: "WGSL-Studio".to_string(),
            fps: 60,
            width: 1920,
            height: 1080,
            bitrate: 8000, // 8 Mbps
            multicast: false,
            tcp_mode: false,
            low_bandwidth: false,
        }
    }
}

/// NDI output state
#[derive(Debug, Resource)]
pub struct NdiOutput {
    config: NdiConfig,
    is_running: bool,
    frame_count: u64,
    start_time: Option<Instant>,
    last_frame_time: Option<Instant>,
    connection_status: String,
}

impl NdiOutput {
    pub fn new(config: NdiConfig) -> Self {
        Self {
            config,
            is_running: false,
            frame_count: 0,
            start_time: None,
            last_frame_time: None,
            connection_status: "Not initialized".to_string(),
        }
    }

    /// Initialize NDI output
    pub fn initialize(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        println!("Initializing NDI output: {}", self.config.source_name);
        println!("Resolution: {}x{} @ {} FPS", self.config.width, self.config.height, self.config.fps);
        
        // In a real implementation, this would initialize the NDI SDK
        // For now, we'll simulate the initialization
        self.connection_status = "Initialized (simulated)".to_string();
        Ok(())
    }

    /// Start NDI output
    pub fn start(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Err("NDI output is disabled".to_string());
        }

        if self.is_running {
            return Err("NDI output already running".to_string());
        }

        self.initialize()?;
        
        self.is_running = true;
        self.start_time = Some(Instant::now());
        self.last_frame_time = Some(Instant::now());
        self.frame_count = 0;
        
        self.connection_status = "Running (simulated)".to_string();
        println!("NDI output started: {}", self.config.source_name);
        
        Ok(())
    }

    /// Stop NDI output
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        self.is_running = false;
        self.connection_status = "Stopped".to_string();
        
        if let Some(start_time) = self.start_time {
            let duration = start_time.elapsed();
            println!("NDI output stopped after {} frames in {:?}", self.frame_count, duration);
        }
        
        Ok(())
    }

    /// Send video frame to NDI output
    pub fn send_frame(&mut self, pixel_data: &[u8], width: u32, height: u32) -> Result<(), String> {
        if !self.is_running {
            return Err("NDI output not running".to_string());
        }

        // Validate frame dimensions
        if width != self.config.width || height != self.config.height {
            return Err(format!("Frame dimensions mismatch: expected {}x{}, got {}x{}", 
                             self.config.width, self.config.height, width, height));
        }

        // Validate pixel data size (RGBA = 4 bytes per pixel)
        let expected_size = (width * height * 4) as usize;
        if pixel_data.len() != expected_size {
            return Err(format!("Pixel data size mismatch: expected {} bytes, got {}", 
                             expected_size, pixel_data.len()));
        }

        self.frame_count += 1;
        self.last_frame_time = Some(Instant::now());

        // In a real implementation, this would send the frame to the NDI SDK
        // For now, we'll simulate frame transmission
        if self.frame_count % 60 == 0 {
            println!("NDI: Sent {} frames to {}", self.frame_count, self.config.source_name);
        }

        Ok(())
    }

    /// Get current status
    pub fn get_status(&self) -> NdiStatus {
        NdiStatus {
            is_running: self.is_running,
            frame_count: self.frame_count,
            connection_status: self.connection_status.clone(),
            fps: if let (Some(start), Some(last)) = (self.start_time, self.last_frame_time) {
                if start.elapsed().as_secs() > 0 {
                    self.frame_count as f64 / start.elapsed().as_secs_f64()
                } else {
                    0.0
                }
            } else {
                0.0
            },
            uptime: self.start_time.map(|t| t.elapsed()),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: NdiConfig) -> Result<(), String> {
        let was_running = self.is_running;
        
        if was_running {
            self.stop()?;
        }

        self.config = new_config;

        if was_running && self.config.enabled {
            self.start()?;
        }

        Ok(())
    }
}

/// NDI status information
#[derive(Debug, Clone)]
pub struct NdiStatus {
    pub is_running: bool,
    pub frame_count: u64,
    pub connection_status: String,
    pub fps: f64,
    pub uptime: Option<Duration>,
}

/// NDI output plugin for Bevy
pub struct NdiOutputPlugin;

impl Plugin for NdiOutputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NdiConfig>()
            .insert_resource(NdiOutput::new(NdiConfig::default()))
            .add_systems(Update, update_ndi_output);
    }
}

/// System to update NDI output
fn update_ndi_output(
    config: Res<NdiConfig>,
    mut ndi_output: ResMut<NdiOutput>,
) {
    // Update NDI output if configuration changed
    if config.is_changed() {
        let _ = ndi_output.update_config(config.clone());
    }
}

/// UI component for NDI controls
pub struct NdiUI;

impl NdiUI {
    pub fn render_ndi_controls(
        ui: &mut egui::Ui,
        config: &mut NdiConfig,
        output: &NdiOutput,
    ) {
        ui.heading("🌐 NDI Output");
        
        ui.separator();
        
        // Enable/disable NDI output
        ui.checkbox(&mut config.enabled, "Enable NDI Output");
        
        if config.enabled {
            ui.separator();
            
            // Source configuration
            ui.horizontal(|ui| {
                ui.label("Source Name:");
                ui.text_edit_singleline(&mut config.source_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("Group Name:");
                ui.text_edit_singleline(&mut config.group_name);
            });
            
            ui.separator();
            
            // Video settings
            ui.horizontal(|ui| {
                ui.label("Resolution:");
                ui.add(egui::DragValue::new(&mut config.width).speed(1.0));
                ui.label("x");
                ui.add(egui::DragValue::new(&mut config.height).speed(1.0));
            });
            
            ui.horizontal(|ui| {
                ui.label("FPS:");
                ui.add(egui::DragValue::new(&mut config.fps).speed(1.0));
                ui.label("Bitrate (kbps):");
                ui.add(egui::DragValue::new(&mut config.bitrate).speed(100.0));
            });
            
            ui.separator();
            
            // Network settings
            ui.collapsing("Network Settings", |ui| {
                ui.checkbox(&mut config.multicast, "Use Multicast");
                ui.checkbox(&mut config.tcp_mode, "TCP Mode");
                ui.checkbox(&mut config.low_bandwidth, "Low Bandwidth Mode");
            });
            
            ui.separator();
            
            // Status display
            let status = output.get_status();
            ui.horizontal(|ui| {
                ui.label("Status:");
                if status.is_running {
                    ui.label(egui::RichText::new("Running").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("Stopped").color(egui::Color32::RED));
                }
            });
            
            ui.label(format!("Connection: {}", status.connection_status));
            ui.label(format!("Frames Sent: {}", status.frame_count));
            ui.label(format!("FPS: {:.1}", status.fps));
            
            if let Some(uptime) = status.uptime {
                ui.label(format!("Uptime: {:?}", uptime));
            }
            
            ui.separator();
            
            // Control buttons
            ui.horizontal(|ui| {
                if status.is_running {
                    if ui.button("⏹ Stop Output").clicked() {
                        // This would trigger stop via system
                        println!("NDI stop requested");
                    }
                } else {
                    if ui.button("▶ Start Output").clicked() {
                        // This would trigger start via system
                        println!("NDI start requested");
                    }
                }
            });
        }
    }
}

/// Test NDI output functionality
pub fn test_ndi_output() {
    println!("Testing NDI output functionality...");
    
    let config = NdiConfig {
        enabled: true,
        source_name: "Test Source".to_string(),
        fps: 30,
        width: 1920,
        height: 1080,
        ..Default::default()
    };
    
    let mut ndi_output = NdiOutput::new(config);
    
    match ndi_output.start() {
        Ok(_) => {
            println!("NDI output test started successfully");
            
            // Simulate sending a few frames
            let test_frame = vec![128u8; 1920 * 1080 * 4]; // Gray frame
            for i in 0..5 {
                match ndi_output.send_frame(&test_frame, 1920, 1080) {
                    Ok(_) => println!("Frame {} sent successfully", i + 1),
                    Err(e) => println!("Failed to send frame {}: {}", i + 1, e),
                }
            }
            
            let status = ndi_output.get_status();
            println!("Test completed. Status: {:?}", status);
            
            ndi_output.stop().unwrap();
        }
        Err(e) => println!("Failed to start NDI output: {}", e),
    }
}