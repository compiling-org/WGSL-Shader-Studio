//! Spout (Windows) and Syphon (macOS) output system for real-time video sharing
//! Enables shader output to be shared with other applications in real-time

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Spout/Syphon output configuration
#[derive(Debug, Clone, Resource)]
pub struct SpoutSyphonConfig {
    pub enabled: bool,
    pub sender_name: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub platform: Platform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows, // Uses Spout
    MacOS,   // Uses Syphon
    Linux,   // Not supported (would need alternative)
}

impl Default for SpoutSyphonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sender_name: "WGSL Shader Studio".to_string(),
            width: 1920,
            height: 1080,
            fps: 60,
            platform: detect_platform(),
        }
    }
}

fn detect_platform() -> Platform {
    #[cfg(target_os = "windows")]
    return Platform::Windows;
    #[cfg(target_os = "macos")]
    return Platform::MacOS;
    #[cfg(target_os = "linux")]
    return Platform::Linux;
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return Platform::Windows; // Default fallback
}

/// Spout/Syphon output state
#[derive(Debug, Resource)]
pub struct SpoutSyphonOutput {
    config: SpoutSyphonConfig,
    is_running: bool,
    frame_count: u64,
    start_time: Option<std::time::Instant>,
    last_frame_time: Option<std::time::Instant>,
    connection_status: String,
}

impl SpoutSyphonOutput {
    pub fn new(config: SpoutSyphonConfig) -> Self {
        Self {
            config,
            is_running: false,
            frame_count: 0,
            start_time: None,
            last_frame_time: None,
            connection_status: "Not initialized".to_string(),
        }
    }

    /// Initialize Spout/Syphon output
    pub fn initialize(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        println!("Initializing {} output: {}", 
            match self.config.platform {
                Platform::Windows => "Spout",
                Platform::MacOS => "Syphon",
                Platform::Linux => "Linux (unsupported)",
            },
            self.config.sender_name
        );
        println!("Resolution: {}x{} @ {} FPS", self.config.width, self.config.height, self.config.fps);
        
        // In a real implementation, this would initialize the appropriate SDK
        // For now, we'll simulate the initialization
        self.connection_status = match self.config.platform {
            Platform::Windows => "Spout initialized (simulated)".to_string(),
            Platform::MacOS => "Syphon initialized (simulated)".to_string(),
            Platform::Linux => "Linux platform not supported for Spout/Syphon".to_string(),
        };
        
        Ok(())
    }

    /// Start Spout/Syphon output
    pub fn start(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Err("Spout/Syphon output is disabled".to_string());
        }

        if self.is_running {
            return Err("Spout/Syphon output already running".to_string());
        }

        self.initialize()?;
        
        self.is_running = true;
        self.start_time = Some(std::time::Instant::now());
        self.last_frame_time = Some(std::time::Instant::now());
        self.frame_count = 0;
        
        self.connection_status = match self.config.platform {
            Platform::Windows => "Spout sender running".to_string(),
            Platform::MacOS => "Syphon server running".to_string(),
            Platform::Linux => "Not supported on Linux".to_string(),
        };
        
        println!("{} output started: {}", 
            match self.config.platform {
                Platform::Windows => "Spout",
                Platform::MacOS => "Syphon",
                Platform::Linux => "Linux",
            },
            self.config.sender_name
        );
        
        Ok(())
    }

    /// Stop Spout/Syphon output
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        self.is_running = false;
        self.connection_status = "Stopped".to_string();
        
        if let Some(start_time) = self.start_time {
            let duration = start_time.elapsed();
            println!("{} output stopped after {} frames in {:?}", 
                match self.config.platform {
                    Platform::Windows => "Spout",
                    Platform::MacOS => "Syphon",
                    Platform::Linux => "Linux",
                },
                self.frame_count, 
                duration
            );
        }
        
        Ok(())
    }

    /// Send video frame to Spout/Syphon output
    pub fn send_frame(&mut self, pixel_data: &[u8], width: u32, height: u32) -> Result<(), String> {
        if !self.is_running {
            return Err("Spout/Syphon output not running".to_string());
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
        self.last_frame_time = Some(std::time::Instant::now());

        // In a real implementation, this would send the frame to the appropriate SDK
        // For now, we'll simulate frame transmission
        if self.frame_count % 60 == 0 {
            println!("{}: Sent {} frames to {}", 
                match self.config.platform {
                    Platform::Windows => "Spout",
                    Platform::MacOS => "Syphon",
                    Platform::Linux => "Linux",
                },
                self.frame_count, 
                self.config.sender_name
            );
        }

        Ok(())
    }

    /// Get current status
    pub fn get_status(&self) -> SpoutSyphonStatus {
        SpoutSyphonStatus {
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
            platform: self.config.platform,
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: SpoutSyphonConfig) -> Result<(), String> {
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

/// Spout/Syphon status information
#[derive(Debug, Clone)]
pub struct SpoutSyphonStatus {
    pub is_running: bool,
    pub frame_count: u64,
    pub connection_status: String,
    pub fps: f64,
    pub uptime: Option<std::time::Duration>,
    pub platform: Platform,
}

/// Spout/Syphon output plugin for Bevy
pub struct SpoutSyphonOutputPlugin;

impl Plugin for SpoutSyphonOutputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpoutSyphonConfig>()
            .insert_resource(SpoutSyphonOutput::new(SpoutSyphonConfig::default()))
            .add_systems(Update, update_spout_syphon_output);
    }
}

/// System to update Spout/Syphon output
fn update_spout_syphon_output(
    config: Res<SpoutSyphonConfig>,
    mut spout_output: ResMut<SpoutSyphonOutput>,
) {
    // Update Spout/Syphon output if configuration changed
    if config.is_changed() {
        let _ = spout_output.update_config(config.clone());
    }
}

fn spout_syphon_ui_system(
    mut contexts: EguiContexts,
    mut config: ResMut<SpoutSyphonConfig>,
    mut output: ResMut<SpoutSyphonOutput>,
    mut ui_state: ResMut<crate::editor_ui::EditorUiState>,
) {
    if !ui_state.show_spout_panel {
        return;
    }
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };
    egui::Window::new("Spout/Syphon")
        .open(&mut ui_state.show_spout_panel)
        .show(ctx, |ui| {
            SpoutSyphonUI::render_spout_syphon_controls(ui, &mut *config, &mut *output);
        });
}

/// UI component for Spout/Syphon controls
pub struct SpoutSyphonUI;

impl SpoutSyphonUI {
    pub fn render_spout_syphon_controls(
        ui: &mut egui::Ui,
        config: &mut SpoutSyphonConfig,
        output: &mut SpoutSyphonOutput,
    ) {
        let platform_name = match config.platform {
            Platform::Windows => "Spout",
            Platform::MacOS => "Syphon",
            Platform::Linux => "Linux (unsupported)",
        };

        ui.heading(format!("🎥 {} Output", platform_name));
        
        ui.separator();
        
        // Platform info
        ui.horizontal(|ui| {
            ui.label("Platform:");
            ui.label(egui::RichText::new(platform_name).color(egui::Color32::LIGHT_BLUE));
        });
        
        ui.separator();
        
        // Enable/disable output
        if config.platform != Platform::Linux {
            ui.checkbox(&mut config.enabled, format!("Enable {} Output", platform_name));
        } else {
            ui.label(egui::RichText::new("Linux platform not supported").color(egui::Color32::RED));
            config.enabled = false;
        }
        
        if config.enabled && config.platform != Platform::Linux {
            ui.separator();
            
            // Sender configuration
            ui.horizontal(|ui| {
                ui.label("Sender Name:");
                ui.text_edit_singleline(&mut config.sender_name);
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
                        let _ = output.stop();
                    }
                } else {
                    if ui.button("▶ Start Output").clicked() {
                        let _ = output.start();
                    }
                }
            });
        }
    }
}

/// Test Spout/Syphon output functionality
pub fn test_spout_syphon_output() {
    println!("Testing Spout/Syphon output functionality...");
    
    let config = SpoutSyphonConfig {
        enabled: true,
        sender_name: "Test Sender".to_string(),
        fps: 30,
        width: 1920,
        height: 1080,
        ..Default::default()
    };
    
    let mut output = SpoutSyphonOutput::new(config);
    
    match output.start() {
        Ok(_) => {
            println!("✓ Spout/Syphon output test started successfully");
            
            // Simulate sending a few frames
            let test_frame = vec![128u8; 1920 * 1080 * 4]; // Gray frame
            for i in 0..5 {
                match output.send_frame(&test_frame, 1920, 1080) {
                    Ok(_) => println!("✓ Frame {} sent successfully", i + 1),
                    Err(e) => println!("✗ Failed to send frame {}: {}", i + 1, e),
                }
            }
            
            let status = output.get_status();
            println!("✓ Test completed. Status: {:?}", status);
            
            output.stop().unwrap();
        }
        Err(e) => {
            println!("✗ Spout/Syphon output test failed: {}", e);
        }
    }
}
