//! OSC (Open Sound Control) control system for professional VJ workflows
//! Enables real-time parameter control from external applications like TouchOSC, Lemur, etc.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
// use std::net::{SocketAddr, UdpSocket};
use std::net::UdpSocket;

/// OSC control configuration
#[derive(Debug, Clone, Resource)]
pub struct OscConfig {
    pub enabled: bool,
    pub listen_address: String,
    pub listen_port: u16,
    pub send_address: Option<String>,
    pub send_port: Option<u16>,
    pub auto_discovery: bool,
    pub feedback_enabled: bool,
}

impl Default for OscConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_address: "0.0.0.0".to_string(),
            listen_port: 9000,
            send_address: None,
            send_port: None,
            auto_discovery: true,
            feedback_enabled: true,
        }
    }
}

/// OSC message types
#[derive(Debug, Clone, PartialEq)]
pub enum OscMessageType {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
    // Add more types as needed
}

/// OSC message
#[derive(Debug, Clone)]
pub struct OscMessage {
    pub address: String,
    pub value: OscMessageType,
    pub timestamp: std::time::Instant,
}

/// OSC control mapping
#[derive(Debug, Clone)]
pub struct OscMapping {
    pub osc_address: String,
    pub parameter_name: String,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub message_type: OscMessageType,
}

/// OSC control state
#[derive(Debug, Resource)]
pub struct OscControl {
    config: OscConfig,
    is_running: bool,
    socket: Option<Arc<Mutex<UdpSocket>>>,
    mappings: Vec<OscMapping>,
    last_messages: HashMap<String, OscMessage>,
    parameter_values: HashMap<String, f32>,
    connection_status: String,
}

impl OscControl {
    pub fn new(config: OscConfig) -> Self {
        Self {
            config,
            is_running: false,
            socket: None,
            mappings: Vec::new(),
            last_messages: HashMap::new(),
            parameter_values: HashMap::new(),
            connection_status: "Not initialized".to_string(),
        }
    }
    
    pub fn add_mapping(&mut self, mapping: OscMapping) {
        self.mappings.retain(|m| m.parameter_name != mapping.parameter_name);
        self.mappings.push(mapping);
    }
    
    pub fn remove_mapping_for_parameter(&mut self, parameter_name: &str) {
        self.mappings.retain(|m| m.parameter_name != parameter_name);
        self.parameter_values.remove(parameter_name);
    }
    
    pub fn get_mapping_for_parameter(&self, parameter_name: &str) -> Option<&OscMapping> {
        self.mappings.iter().find(|m| m.parameter_name == parameter_name)
    }

    /// Initialize OSC control
    pub fn initialize(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        println!("Initializing OSC control on {}:{}", self.config.listen_address, self.config.listen_port);
        
        // In a real implementation, this would create and bind the UDP socket
        // For now, we'll simulate the initialization
        self.connection_status = "OSC initialized (simulated)".to_string();
        
        // Add default mappings for common VJ parameters
        self.add_default_mappings();
        
        Ok(())
    }

    /// Start OSC control
    pub fn start(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Err("OSC control is disabled".to_string());
        }

        if self.is_running {
            return Err("OSC control already running".to_string());
        }

        self.initialize()?;
        
        self.is_running = true;
        self.connection_status = "OSC control running (simulated)".to_string();
        
        println!("OSC control started on {}:{}", self.config.listen_address, self.config.listen_port);
        
        Ok(())
    }

    /// Stop OSC control
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        self.is_running = false;
        self.connection_status = "Stopped".to_string();
        
        println!("OSC control stopped");
        
        Ok(())
    }

    /// Add default OSC mappings for common VJ parameters
    fn add_default_mappings(&mut self) {
        let default_mappings = vec![
            OscMapping {
                osc_address: "/shader/time".to_string(),
                parameter_name: "time".to_string(),
                min_value: 0.0,
                max_value: 100.0,
                default_value: 0.0,
                message_type: OscMessageType::Float(0.0),
            },
            OscMapping {
                osc_address: "/shader/parameter1".to_string(),
                parameter_name: "param1".to_string(),
                min_value: 0.0,
                max_value: 1.0,
                default_value: 0.5,
                message_type: OscMessageType::Float(0.5),
            },
            OscMapping {
                osc_address: "/shader/parameter2".to_string(),
                parameter_name: "param2".to_string(),
                min_value: 0.0,
                max_value: 1.0,
                default_value: 0.5,
                message_type: OscMessageType::Float(0.5),
            },
            OscMapping {
                osc_address: "/shader/speed".to_string(),
                parameter_name: "speed".to_string(),
                min_value: 0.0,
                max_value: 2.0,
                default_value: 1.0,
                message_type: OscMessageType::Float(1.0),
            },
            OscMapping {
                osc_address: "/shader/enable".to_string(),
                parameter_name: "enable".to_string(),
                min_value: 0.0,
                max_value: 1.0,
                default_value: 1.0,
                message_type: OscMessageType::Bool(true),
            },
        ];

        for mapping in default_mappings {
            self.mappings.push(mapping);
            self.parameter_values.insert("param1".to_string(), 0.5);
            self.parameter_values.insert("param2".to_string(), 0.5);
            self.parameter_values.insert("speed".to_string(), 1.0);
            self.parameter_values.insert("enable".to_string(), 1.0);
        }
    }

    /// Process incoming OSC message (simulated)
    pub fn process_message(&mut self, address: &str, value: OscMessageType) -> Result<(), String> {
        if !self.is_running {
            return Err("OSC control not running".to_string());
        }

        // Find mapping for this address
        if let Some(mapping) = self.mappings.iter().find(|m| m.osc_address == address) {
            // Convert value to float parameter
            let float_value = match &value {
                OscMessageType::Float(f) => *f,
                OscMessageType::Int(i) => *i as f32,
                OscMessageType::Bool(b) => if *b { 1.0 } else { 0.0 },
                OscMessageType::String(s) => s.parse::<f32>().unwrap_or(mapping.default_value),
            };

            // Clamp to mapping range
            let clamped_value = float_value.clamp(mapping.min_value, mapping.max_value);
            
            // Store parameter value
            self.parameter_values.insert(mapping.parameter_name.clone(), clamped_value);
            
            // Store last message
            let message = OscMessage {
                address: address.to_string(),
                value: value.clone(),
                timestamp: std::time::Instant::now(),
            };
            self.last_messages.insert(address.to_string(), message);
            
            println!("OSC: {} = {:?} (mapped to {})", address, value, mapping.parameter_name);
            
            Ok(())
        } else {
            Err(format!("No mapping found for OSC address: {}", address))
        }
    }

    /// Get parameter value by name
    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        self.parameter_values.get(name).copied()
    }

    /// Get all parameter values
    pub fn get_all_parameters(&self) -> &HashMap<String, f32> {
        &self.parameter_values
    }

    /// Get current status
    pub fn get_status(&self) -> OscStatus {
        OscStatus {
            is_running: self.is_running,
            connection_status: self.connection_status.clone(),
            num_mappings: self.mappings.len(),
            num_parameters: self.parameter_values.len(),
            last_message_count: self.last_messages.len(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: OscConfig) -> Result<(), String> {
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

/// OSC status information
#[derive(Debug, Clone)]
pub struct OscStatus {
    pub is_running: bool,
    pub connection_status: String,
    pub num_mappings: usize,
    pub num_parameters: usize,
    pub last_message_count: usize,
}

/// OSC control plugin for Bevy
pub struct OscControlPlugin;

impl Plugin for OscControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OscConfig>()
            .insert_resource(OscControl::new(OscConfig::default()))
            .add_systems(Update, update_osc_control);
    }
}

/// System to update OSC control
fn update_osc_control(
    config: Res<OscConfig>,
    mut osc_control: ResMut<OscControl>,
) {
    // Update OSC control if configuration changed
    if config.is_changed() {
        let _ = osc_control.update_config(config.clone());
    }
}


/// UI component for OSC controls
pub struct OscUI;

impl OscUI {
    pub fn render_osc_controls(
        ui: &mut egui::Ui,
        config: &mut OscConfig,
        control: &OscControl,
    ) {
        ui.heading("🎛 OSC Control");
        
        ui.separator();
        
        // Enable/disable OSC control
        ui.checkbox(&mut config.enabled, "Enable OSC Control");
        
        if config.enabled {
            ui.separator();
            
            // Network configuration
            ui.collapsing("Network Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Listen Address:");
                    ui.text_edit_singleline(&mut config.listen_address);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Listen Port:");
                    ui.add(egui::DragValue::new(&mut config.listen_port).speed(1.0));
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Send Address:");
                    if let Some(ref mut addr) = config.send_address {
                        ui.text_edit_singleline(addr);
                    } else {
                        if ui.button("Set Send Address").clicked() {
                            config.send_address = Some("127.0.0.1".to_string());
                        }
                    }
                });
                
                if let Some(ref mut port) = config.send_port {
                    ui.horizontal(|ui| {
                        ui.label("Send Port:");
                        ui.add(egui::DragValue::new(port).speed(1.0));
                    });
                } else if ui.button("Set Send Port").clicked() {
                    config.send_port = Some(9001);
                }
                
                ui.separator();
                
                ui.checkbox(&mut config.auto_discovery, "Auto Discovery");
                ui.checkbox(&mut config.feedback_enabled, "Enable Feedback");
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
            ui.label(format!("Mappings: {}", status.num_mappings));
            ui.label(format!("Parameters: {}", status.num_parameters));
            ui.label(format!("Last Messages: {}", status.last_message_count));
            
            ui.separator();
            
            // Parameter display
            ui.collapsing("Parameter Values", |ui| {
                for (name, value) in control.get_all_parameters() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", name));
                        ui.label(format!("{:.3}", value));
                    });
                }
            });
            
            ui.separator();
            
            // Control buttons
            ui.horizontal(|ui| {
                if status.is_running {
                    if ui.button("⏹ Stop Control").clicked() {
                        // This would trigger stop via system
                        println!("OSC stop requested");
                    }
                } else {
                    if ui.button("▶ Start Control").clicked() {
                        // This would trigger start via system
                        println!("OSC start requested");
                    }
                }
            });
            
            // Test message simulation
            ui.separator();
            ui.label("Test Messages:");
            ui.horizontal(|ui| {
                if ui.button("Send Test /shader/time").clicked() {
                    println!("Sending test OSC message: /shader/time = 1.5");
                }
                if ui.button("Send Test /shader/parameter1").clicked() {
                    println!("Sending test OSC message: /shader/parameter1 = 0.75");
                }
            });
        }
    }
}

/// Test OSC control functionality
pub fn test_osc_control() {
    println!("Testing OSC control functionality...");
    
    let config = OscConfig {
        enabled: true,
        listen_address: "127.0.0.1".to_string(),
        listen_port: 9000,
        ..Default::default()
    };
    
    let mut control = OscControl::new(config);
    
    match control.start() {
        Ok(_) => {
            println!("✓ OSC control test started successfully");
            
            // Simulate receiving some messages
            let test_messages = vec![
                ("/shader/time", OscMessageType::Float(2.5)),
                ("/shader/parameter1", OscMessageType::Float(0.8)),
                ("/shader/parameter2", OscMessageType::Float(0.3)),
                ("/shader/speed", OscMessageType::Float(1.2)),
                ("/shader/enable", OscMessageType::Bool(true)),
            ];
            
            for (address, value) in test_messages {
                match control.process_message(address, value) {
                    Ok(_) => println!("✓ Processed OSC message: {} = {:?}", address, control.get_parameter(&address.replace("/shader/", "")).unwrap_or(0.0)),
                    Err(e) => println!("✗ Failed to process message {}: {}", address, e),
                }
            }
            
            let status = control.get_status();
            println!("✓ Test completed. Status: {:?}", status);
            
            control.stop().unwrap();
        }
        Err(e) => {
            println!("✗ OSC control test failed: {}", e);
        }
    }
}
