//! DMX Lighting Control System for professional VJ workflows
//! Enables real-time lighting control via DMX512 protocol for stage lighting integration

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::HashMap;

/// DMX universe configuration (512 channels per universe)
#[derive(Debug, Clone, Resource)]
pub struct DmxConfig {
    pub enabled: bool,
    pub universe_count: u8,
    pub start_universe: u16,
    pub frame_rate: u32, // DMX frames per second
    pub artnet_enabled: bool,
    pub sacn_enabled: bool,
    pub sacn_multicast: bool,
    pub artnet_broadcast: bool,
    pub listen_port: u16,
    pub target_ip: Option<String>,
    pub device_id: String,
}

impl Default for DmxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            universe_count: 4, // 4 universes = 2048 channels
            start_universe: 1,
            frame_rate: 44, // Standard DMX frame rate
            artnet_enabled: true,
            sacn_enabled: false,
            sacn_multicast: true,
            artnet_broadcast: true,
            listen_port: 6454, // Art-Net port
            target_ip: Some("255.255.255.255".to_string()), // Broadcast
            device_id: "WGSL-Shader-Studio".to_string(),
        }
    }
}

/// DMX channel mapping for shader parameters
#[derive(Debug, Clone)]
pub struct DmxChannelMapping {
    pub channel: u16, // 1-512 per universe
    pub universe: u16, // Universe number
    pub parameter_name: String,
    pub min_value: u8, // 0-255
    pub max_value: u8, // 0-255
    pub default_value: u8, // 0-255
    pub channel_type: DmxChannelType,
}

/// DMX channel types for different lighting functions
#[derive(Debug, Clone, PartialEq)]
pub enum DmxChannelType {
    Intensity,    // Master brightness
    Red,         // RGB color
    Green,
    Blue,
    Strobe,      // Strobe speed
    Pan,         // Moving head pan
    Tilt,        // Moving head tilt
    ColorWheel,  // Color wheel position
    GoboWheel,   // Gobo wheel position
    Focus,       // Focus adjustment
    Zoom,        // Zoom adjustment
    Iris,        // Iris adjustment
    Frost,       // Frost filter
    Prism,       // Prism rotation
    Speed,       // Movement speed
    Custom,      // Custom parameter
}

/// DMX channel value with metadata
#[derive(Debug, Clone)]
pub struct DmxChannelValue {
    pub channel: u16,
    pub universe: u16,
    pub value: u8,
    pub timestamp: std::time::Instant,
    pub source: String, // Source of the value (shader, manual, external)
}

/// DMX lighting control state
#[derive(Debug, Resource)]
pub struct DmxLightingControl {
    config: DmxConfig,
    is_running: bool,
    universes: HashMap<u16, [u8; 512]>, // Universe ID -> 512 channels
    mappings: Vec<DmxChannelMapping>,
    channel_values: HashMap<(u16, u16), DmxChannelValue>, // (universe, channel) -> value
    connection_status: String,
    frame_counter: u64,
    last_frame_time: std::time::Instant,
}

impl DmxLightingControl {
    pub fn new(config: DmxConfig) -> Self {
        let mut universes = HashMap::new();
        for i in 0..config.universe_count {
            universes.insert(config.start_universe + i as u16, [0u8; 512]);
        }

        Self {
            config,
            is_running: false,
            universes,
            mappings: Vec::new(),
            channel_values: HashMap::new(),
            connection_status: "Not initialized".to_string(),
            frame_counter: 0,
            last_frame_time: std::time::Instant::now(),
        }
    }

    /// Initialize DMX lighting control
    pub fn initialize(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        println!("Initializing DMX lighting control with {} universes", self.config.universe_count);
        
        // In a real implementation, this would initialize Art-Net/sACN libraries
        // For now, we'll simulate the initialization
        self.connection_status = "DMX initialized (simulated)".to_string();
        
        // Add default mappings for common lighting parameters
        self.add_default_mappings();
        
        Ok(())
    }

    /// Start DMX lighting control
    pub fn start(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Err("DMX lighting control is disabled".to_string());
        }

        if self.is_running {
            return Err("DMX lighting control already running".to_string());
        }

        self.initialize()?;
        
        self.is_running = true;
        self.connection_status = "DMX control running (simulated)".to_string();
        self.last_frame_time = std::time::Instant::now();
        
        println!("DMX lighting control started with {} universes at {} FPS", 
                 self.config.universe_count, self.config.frame_rate);
        
        Ok(())
    }

    /// Stop DMX lighting control
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        self.is_running = false;
        self.connection_status = "Stopped".to_string();
        
        // Blackout all channels
        self.blackout();
        
        println!("DMX lighting control stopped");
        
        Ok(())
    }

    /// Add default DMX channel mappings for shader parameters
    fn add_default_mappings(&mut self) {
        let default_mappings = vec![
            DmxChannelMapping {
                channel: 1,
                universe: self.config.start_universe,
                parameter_name: "master_intensity".to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 255,
                channel_type: DmxChannelType::Intensity,
            },
            DmxChannelMapping {
                channel: 2,
                universe: self.config.start_universe,
                parameter_name: "red".to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 255,
                channel_type: DmxChannelType::Red,
            },
            DmxChannelMapping {
                channel: 3,
                universe: self.config.start_universe,
                parameter_name: "green".to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 255,
                channel_type: DmxChannelType::Green,
            },
            DmxChannelMapping {
                channel: 4,
                universe: self.config.start_universe,
                parameter_name: "blue".to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 255,
                channel_type: DmxChannelType::Blue,
            },
            DmxChannelMapping {
                channel: 5,
                universe: self.config.start_universe,
                parameter_name: "strobe".to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 0,
                channel_type: DmxChannelType::Strobe,
            },
            DmxChannelMapping {
                channel: 6,
                universe: self.config.start_universe,
                parameter_name: "pan".to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 128,
                channel_type: DmxChannelType::Pan,
            },
            DmxChannelMapping {
                channel: 7,
                universe: self.config.start_universe,
                parameter_name: "tilt".to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 128,
                channel_type: DmxChannelType::Tilt,
            },
            DmxChannelMapping {
                channel: 8,
                universe: self.config.start_universe,
                parameter_name: "speed".to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 128,
                channel_type: DmxChannelType::Speed,
            },
        ];

        for mapping in default_mappings {
            self.mappings.push(mapping.clone());
            
            // Initialize channel values
            let channel_key = (mapping.universe, mapping.channel);
            self.channel_values.insert(channel_key, DmxChannelValue {
                channel: mapping.channel,
                universe: mapping.universe,
                value: mapping.default_value,
                timestamp: std::time::Instant::now(),
                source: "default".to_string(),
            });
        }
    }

    /// Set DMX channel value
    pub fn set_channel_value(&mut self, universe: u16, channel: u16, value: u8, source: &str) -> Result<(), String> {
        if !self.is_running {
            return Err("DMX lighting control not running".to_string());
        }

        if channel < 1 || channel > 512 {
            return Err(format!("Channel must be between 1-512, got: {}", channel));
        }

        if let Some(universe_data) = self.universes.get_mut(&universe) {
            let clamped_value = value.clamp(0, 255);
            universe_data[channel as usize - 1] = clamped_value;
            
            let channel_key = (universe, channel);
            self.channel_values.insert(channel_key, DmxChannelValue {
                channel,
                universe,
                value: clamped_value,
                timestamp: std::time::Instant::now(),
                source: source.to_string(),
            });
            
            println!("DMX: Universe {}, Channel {} = {} (source: {})", universe, channel, clamped_value, source);
            
            Ok(())
        } else {
            Err(format!("Universe {} not found", universe))
        }
    }

    /// Get DMX channel value
    pub fn get_channel_value(&self, universe: u16, channel: u16) -> Option<u8> {
        if let Some(universe_data) = self.universes.get(&universe) {
            if channel >= 1 && channel <= 512 {
                Some(universe_data[channel as usize - 1])
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Set all channels in a universe to a specific value
    pub fn set_universe_value(&mut self, universe: u16, value: u8) -> Result<(), String> {
        if let Some(universe_data) = self.universes.get_mut(&universe) {
            universe_data.fill(value);
            
            for channel in 1..=512 {
                let channel_key = (universe, channel);
                self.channel_values.insert(channel_key, DmxChannelValue {
                    channel,
                    universe,
                    value,
                    timestamp: std::time::Instant::now(),
                    source: "universe_set".to_string(),
                });
            }
            
            println!("DMX: Universe {} all channels set to {}", universe, value);
            Ok(())
        } else {
            Err(format!("Universe {} not found", universe))
        }
    }

    /// Blackout all channels (set to 0)
    pub fn blackout(&mut self) {
        for universe in self.config.start_universe..(self.config.start_universe + self.config.universe_count as u16) {
            let _ = self.set_universe_value(universe, 0);
        }
        println!("DMX: Blackout all channels");
    }

    /// Full brightness all channels (set to 255)
    pub fn full_brightness(&mut self) {
        for universe in self.config.start_universe..(self.config.start_universe + self.config.universe_count as u16) {
            let _ = self.set_universe_value(universe, 255);
        }
        println!("DMX: Full brightness all channels");
    }

    /// Map shader parameter to DMX channel
    pub fn map_parameter_to_channel(&mut self, parameter_name: &str, universe: u16, channel: u16, 
                                    min_param: f32, max_param: f32, param_value: f32) -> Result<(), String> {
        if let Some(mapping) = self.mappings.iter().find(|m| m.parameter_name == parameter_name) {
            // Convert parameter value to DMX range (0-255)
            let normalized_param = ((param_value - min_param) / (max_param - min_param)).clamp(0.0, 1.0);
            let dmx_value = (normalized_param * 255.0) as u8;
            
            self.set_channel_value(universe, channel, dmx_value, parameter_name)
        } else {
            // Create new mapping
            let new_mapping = DmxChannelMapping {
                channel,
                universe,
                parameter_name: parameter_name.to_string(),
                min_value: 0,
                max_value: 255,
                default_value: 128,
                channel_type: DmxChannelType::Custom,
            };
            
            self.mappings.push(new_mapping);
            
            // Convert parameter value to DMX range
            let normalized_param = ((param_value - min_param) / (max_param - min_param)).clamp(0.0, 1.0);
            let dmx_value = (normalized_param * 255.0) as u8;
            
            self.set_channel_value(universe, channel, dmx_value, parameter_name)
        }
    }

    /// Get current status
    pub fn get_status(&self) -> DmxStatus {
        DmxStatus {
            is_running: self.is_running,
            connection_status: self.connection_status.clone(),
            universe_count: self.config.universe_count,
            active_universes: self.universes.len(),
            num_mappings: self.mappings.len(),
            num_channels: self.channel_values.len(),
            frame_counter: self.frame_counter,
            frame_rate: self.config.frame_rate,
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: DmxConfig) -> Result<(), String> {
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

    /// Send DMX frame (simulated)
    pub fn send_frame(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Err("DMX lighting control not running".to_string());
        }

        self.frame_counter += 1;
        
        // In a real implementation, this would send the DMX data via Art-Net or sACN
        // For now, we'll simulate the frame sending
        
        Ok(())
    }
}

/// DMX status information
#[derive(Debug, Clone)]
pub struct DmxStatus {
    pub is_running: bool,
    pub connection_status: String,
    pub universe_count: u8,
    pub active_universes: usize,
    pub num_mappings: usize,
    pub num_channels: usize,
    pub frame_counter: u64,
    pub frame_rate: u32,
}

/// DMX lighting control plugin for Bevy
pub struct DmxLightingControlPlugin;

impl Plugin for DmxLightingControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DmxConfig>()
            .insert_resource(DmxLightingControl::new(DmxConfig::default()))
            .add_systems(Update, update_dmx_control)
            .add_systems(Update, dmx_ui_system);
    }
}

/// System to update DMX control
fn update_dmx_control(
    config: Res<DmxConfig>,
    mut dmx_control: ResMut<DmxLightingControl>,
) {
    // Update DMX control if configuration changed
    if config.is_changed() {
        let _ = dmx_control.update_config(config.clone());
    }

    // Send DMX frames at the configured frame rate
    if dmx_control.is_running {
        let frame_interval = std::time::Duration::from_millis(1000 / config.frame_rate as u64);
        if dmx_control.last_frame_time.elapsed() >= frame_interval {
            let _ = dmx_control.send_frame();
            dmx_control.last_frame_time = std::time::Instant::now();
        }
    }
}

/// UI system to render DMX controls within the main editor
fn dmx_ui_system(
    mut contexts: EguiContexts,
    mut config: ResMut<DmxConfig>,
    mut control: ResMut<DmxLightingControl>,
    mut ui_state: ResMut<crate::editor_ui::EditorUiState>,
) {
    if !ui_state.show_dmx_panel {
        return;
    }
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };
    egui::Window::new("DMX Lighting")
        .open(&mut ui_state.show_dmx_panel)
        .show(&ctx, |ui| {
            DmxUI::render_dmx_controls(ui, &mut *config, &mut *control);
        });
}

/// UI component for DMX controls
pub struct DmxUI;

impl DmxUI {
    pub fn render_dmx_controls(
        ui: &mut egui::Ui,
        config: &mut DmxConfig,
        control: &mut DmxLightingControl,
    ) {
        ui.heading("🎭 DMX Lighting Control");
        
        ui.separator();
        
        // Enable/disable DMX control
        ui.checkbox(&mut config.enabled, "Enable DMX Control");
        
        if config.enabled {
            ui.separator();
            
            // Protocol configuration
            ui.collapsing("Protocol Settings", |ui| {
                ui.checkbox(&mut config.artnet_enabled, "Enable Art-Net");
                ui.checkbox(&mut config.sacn_enabled, "Enable sACN");
                
                if config.sacn_enabled {
                    ui.checkbox(&mut config.sacn_multicast, "sACN Multicast");
                }
                
                if config.artnet_enabled {
                    ui.checkbox(&mut config.artnet_broadcast, "Art-Net Broadcast");
                }
            });
            
            ui.separator();
            
            // Universe configuration
            ui.collapsing("Universe Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Universe Count:");
                    ui.add(egui::DragValue::new(&mut config.universe_count).speed(1.0).range(1..=16));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Start Universe:");
                    ui.add(egui::DragValue::new(&mut config.start_universe).speed(1.0).range(1..=32767));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Frame Rate:");
                    ui.add(egui::DragValue::new(&mut config.frame_rate).speed(1.0).range(1..=120));
                    ui.label("FPS");
                });
            });
            
            ui.separator();
            
            // Network configuration
            ui.collapsing("Network Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Listen Port:");
                    ui.add(egui::DragValue::new(&mut config.listen_port).speed(1.0).range(1024..=65535));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Target IP:");
                    if let Some(ref mut ip) = config.target_ip {
                        ui.text_edit_singleline(ip);
                    } else {
                        if ui.button("Set Target IP").clicked() {
                            config.target_ip = Some("192.168.1.255".to_string());
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Device ID:");
                    ui.text_edit_singleline(&mut config.device_id);
                });
            });
            
            ui.separator();
            
            // Status display
            let status = control.get_status();
            ui.horizontal(|ui| {
                ui.label("Status:");
                if status.is_running {
                    ui.label(egui::RichText::new("Running").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("Stopped").color(egui::Color32::RED));
                }
            });
            
            ui.label(format!("Connection: {}", status.connection_status));
            ui.label(format!("Universes: {}/{}", status.active_universes, status.universe_count));
            ui.label(format!("Mappings: {}", status.num_mappings));
            ui.label(format!("Channels: {}", status.num_channels));
            ui.label(format!("Frame: {} @ {} FPS", status.frame_counter, status.frame_rate));
            
            ui.separator();
            
            // Quick controls
            ui.collapsing("Quick Controls", |ui| {
                ui.horizontal(|ui| {
                    if ui.button("🌑 Blackout").clicked() {
                        control.blackout();
                    }
                    if ui.button("💡 Full Brightness").clicked() {
                        control.full_brightness();
                    }
                });
            });
            
            ui.separator();
            
            // Channel testing
            ui.collapsing("Channel Testing", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Universe:");
                    ui.add(egui::DragValue::new(&mut 1u16).speed(1.0).range(1..=16));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Channel:");
                    ui.add(egui::DragValue::new(&mut 1u16).speed(1.0).range(1..=512));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    let mut test_value = 128u8;
                    ui.add(egui::DragValue::new(&mut test_value).speed(1.0).range(0..=255));
                    if ui.button("Set").clicked() {
                        let _ = control.set_channel_value(1, 1, test_value, "manual_test");
                    }
                });
            });
            
            ui.separator();
            
            // Control buttons
            ui.horizontal(|ui| {
                if status.is_running {
                    if ui.button("⏹ Stop Control").clicked() {
                        let _ = control.stop();
                    }
                } else {
                    if ui.button("▶ Start Control").clicked() {
                        let _ = control.start();
                    }
                }
            });
            
            // Parameter mappings display
            ui.separator();
            ui.collapsing("Parameter Mappings", |ui| {
                for mapping in &control.mappings {
                    ui.horizontal(|ui| {
                        ui.label(format!("U{} Ch{}: {}", 
                                       mapping.universe, mapping.channel, mapping.parameter_name));
                        ui.label(format!("{}-{} ({})", 
                                       mapping.min_value, mapping.max_value, 
                                       match mapping.channel_type {
                                           DmxChannelType::Intensity => "Intensity",
                                           DmxChannelType::Red => "Red",
                                           DmxChannelType::Green => "Green",
                                           DmxChannelType::Blue => "Blue",
                                           DmxChannelType::Strobe => "Strobe",
                                           DmxChannelType::Pan => "Pan",
                                           DmxChannelType::Tilt => "Tilt",
                                           DmxChannelType::ColorWheel => "Color Wheel",
                                           DmxChannelType::GoboWheel => "Gobo Wheel",
                                           DmxChannelType::Focus => "Focus",
                                           DmxChannelType::Zoom => "Zoom",
                                           DmxChannelType::Iris => "Iris",
                                           DmxChannelType::Frost => "Frost",
                                           DmxChannelType::Prism => "Prism",
                                           DmxChannelType::Speed => "Speed",
                                           DmxChannelType::Custom => "Custom",
                                       }));
                    });
                }
            });
        }
    }
}

/// Test DMX lighting control functionality
pub fn test_dmx_lighting_control() {
    println!("Testing DMX lighting control functionality...");
    
    let config = DmxConfig {
        enabled: true,
        universe_count: 2,
        start_universe: 1,
        frame_rate: 44,
        artnet_enabled: true,
        ..Default::default()
    };
    
    let mut control = DmxLightingControl::new(config);
    
    match control.start() {
        Ok(_) => {
            println!("✓ DMX lighting control test started successfully");
            
            // Test setting some channel values
            let test_channels = vec![
                (1, 1, 255u8, "master_intensity"),
                (1, 2, 128u8, "red"),
                (1, 3, 64u8, "green"),
                (1, 4, 192u8, "blue"),
                (1, 5, 32u8, "strobe"),
                (1, 6, 64u8, "pan"),
                (1, 7, 192u8, "tilt"),
                (1, 8, 128u8, "speed"),
            ];
            
            for (universe, channel, value, source) in test_channels {
                match control.set_channel_value(universe, channel, value, source) {
                    Ok(_) => println!("✓ Set DMX channel U{} Ch{} = {} ({})", universe, channel, value, source),
                    Err(e) => println!("✗ Failed to set DMX channel U{} Ch{}: {}", universe, channel, e),
                }
            }
            
            // Test parameter mapping
            match control.map_parameter_to_channel("shader_brightness", 1, 10, 0.0, 1.0, 0.75) {
                Ok(_) => println!("✓ Mapped shader parameter to DMX channel"),
                Err(e) => println!("✗ Failed to map parameter: {}", e),
            }
            
            let status = control.get_status();
            println!("✓ DMX test completed. Status: {:?}", status);
            
            control.stop().unwrap();
        }
        Err(e) => {
            println!("✗ DMX lighting control test failed: {}", e);
        }
    }
}
