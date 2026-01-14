use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::editor_ui::EditorUiState;
use crate::midi_system::{MidiSystem, MidiMapping, MidiMessageType, MidiCurve};

pub fn draw_midi_panel(ctx: &egui::Context, ui_state: &mut EditorUiState, midi_system: &mut MidiSystem) {
    egui::Window::new("MIDI Panel")
        .open(&mut ui_state.show_midi_panel)
        .show(ctx, |ui| {
            ui.heading("MIDI Control");
            
            // Device scanning and connection
            ui.horizontal(|ui| {
                if ui.button("Scan Devices").clicked() {
                    if let Err(e) = midi_system.scan_devices() {
                        println!("MIDI scan error: {}", e);
                    }
                }
                
                if ui.button("Refresh").clicked() {
                    // Refresh device list
                    let _ = midi_system.scan_devices();
                }
            });
            
            ui.separator();
            ui.heading("Available MIDI Devices");
            
            let devices_snapshot = midi_system.devices.clone();
            for (i, dev) in devices_snapshot.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&dev.name);
                        ui.label(format!("({})", dev.port_name));
                        
                        if dev.connected {
                            ui.colored_label(egui::Color32::GREEN, "CONNECTED");
                            if ui.button("Disconnect").clicked() {
                                if let Err(e) = midi_system.disconnect_device(i) {
                                    println!("MIDI disconnect error: {}", e);
                                }
                            }
                        } else {
                            if ui.button("Connect").clicked() {
                                if let Err(e) = midi_system.connect_device(i) {
                                    println!("MIDI connect error: {}", e);
                                }
                            }
                        }
                    });
                    
                    // Show active messages for this device
                    if dev.connected {
                        if let Some(ref rx) = dev.message_receiver {
                            // Display recent messages (in a real implementation)
                            ui.label("Listening for MIDI messages...");
                        }
                    }
                });
            }
            
            ui.separator();
            ui.heading("Parameter Mapping");
            
            // Parse shader parameters to map to MIDI
            let params = crate::parameter_panel::parse_shader_parameters(&ui_state.draft_code);
            
            if params.is_empty() {
                ui.label("No shader parameters available for mapping");
            } else {
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for param in params.iter() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(&param.name);
                                
                                if ui.button("Learn MIDI").clicked() {
                                    midi_system.start_midi_learn(&param.name);
                                }
                                
                                if let Some(existing_mapping) = midi_system.get_mapping(&param.name) {
                                    ui.label(format!("Mapped to: Ch{} {}{}", 
                                        existing_mapping.channel,
                                        match existing_mapping.midi_type {
                                            MidiMessageType::NoteOn => "Note",
                                            MidiMessageType::NoteOff => "NoteOff",
                                            MidiMessageType::ControlChange => "CC",
                                            MidiMessageType::ProgramChange => "PC",
                                            MidiMessageType::PitchBend => "PitchBend",
                                        },
                                        existing_mapping.number
                                    ));
                                    
                                    if ui.button("Remove").clicked() {
                                        midi_system.remove_mapping(&param.name);
                                    }
                                }
                            });
                            
                            // Mapping configuration
                            if let Some(mut mapping) = midi_system.get_mapping(&param.name).cloned() {
                                ui.horizontal(|ui| {
                                    ui.label("Channel:");
                                    ui.add(egui::DragValue::new(&mut mapping.channel).range(1..=16));
                                    
                                    ui.label("Type:");
                                    ui.radio_value(&mut mapping.midi_type, MidiMessageType::ControlChange, "CC");
                                    ui.radio_value(&mut mapping.midi_type, MidiMessageType::NoteOn, "Note");
                                    ui.radio_value(&mut mapping.midi_type, MidiMessageType::PitchBend, "Pitch");
                                    
                                    match mapping.midi_type {
                                        MidiMessageType::ControlChange | MidiMessageType::NoteOn => {
                                            ui.label("Number:");
                                            ui.add(egui::DragValue::new(&mut mapping.number).range(0..=127));
                                        },
                                        MidiMessageType::PitchBend => {
                                            ui.label("Min:");
                                            ui.add(egui::DragValue::new(&mut mapping.min_value).speed(0.01));
                                            ui.label("Max:");
                                            ui.add(egui::DragValue::new(&mut mapping.max_value).speed(0.01));
                                        },
                                        _ => {}
                                    }
                                    
                                    ui.label("Curve:");
                                    ui.radio_value(&mut mapping.curve, MidiCurve::Linear, "Linear");
                                    ui.radio_value(&mut mapping.curve, MidiCurve::Logarithmic, "Log");
                                    ui.radio_value(&mut mapping.curve, MidiCurve::Exponential, "Exp");
                                    
                                    ui.checkbox(&mut mapping.invert, "Invert");
                                    ui.add(egui::DragValue::new(&mut mapping.smoothing).speed(0.01).clamp_range(0.0..=1.0));
                                    
                                    if ui.button("Update").clicked() {
                                        midi_system.update_mapping(mapping);
                                    }
                                });
                            }
                        });
                    }
                });
            }
            
            ui.separator();
            ui.heading("MIDI Learn Mode");
            
            ui.horizontal(|ui| {
                ui.checkbox(&mut midi_system.learn_mode_active, "MIDI Learn Active");
                
                if midi_system.learn_mode_active {
                    ui.label("Move a MIDI controller to map it...");
                    
                    // Check for incoming MIDI messages in learn mode
                    if let Some((param_name, message)) = midi_system.check_learn_message() {
                        ui.label(format!("Learned: {} -> {}", param_name, message));
                    }
                }
            });
            
            ui.separator();
            ui.heading("MIDI Activity");
            
            // Show last received MIDI messages
            let recent_messages = midi_system.get_recent_messages(10);
            if !recent_messages.is_empty() {
                ui.collapsing("Recent Messages", |ui| {
                    for msg in recent_messages.iter().rev() {
                        ui.label(format!("{}", msg));
                    }
                });
            }
        });
}