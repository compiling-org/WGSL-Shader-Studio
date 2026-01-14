use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::editor_ui::EditorUiState;

pub fn draw_parameter_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::Window::new("Parameter Panel")
        .open(&mut ui_state.show_parameter_panel)
        .show(ctx, |ui| {
            ui.heading("Shader Parameters");
            
            // Quick parameter controls
            ui.checkbox(&mut ui_state.quick_params_enabled, "Enable Quick Params");
            if ui_state.quick_params_enabled {
                ui.add(egui::Slider::new(&mut ui_state.quick_param_a, 0.0..=1.0).text("Quick Param A"));
                ui.add(egui::Slider::new(&mut ui_state.quick_param_b, 0.0..=1.0).text("Quick Param B"));
            }
            
            // Parse and display shader parameters
            let params = parse_shader_parameters(&ui_state.draft_code);
            if params.is_empty() {
                ui.label("No parameters found in current shader");
            } else {
                ui.separator();
                ui.heading("Shader Parameters");
                
                for param in params {
                    match param.param_type {
                        ParamType::Float => {
                            let mut value = ui_state.get_parameter_value(&param.name).unwrap_or(0.5);
                            ui.add(egui::Slider::new(&mut value, 0.0..=1.0).text(&param.name));
                            ui_state.set_parameter_value(&param.name, value);
                        },
                        ParamType::Vec2 => {
                            let mut value = [
                                ui_state.get_parameter_value(&format!("{}_x", param.name)).unwrap_or(0.5),
                                ui_state.get_parameter_value(&format!("{}_y", param.name)).unwrap_or(0.5)
                            ];
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: ", param.name));
                                ui.add(egui::DragValue::new(&mut value[0]).speed(0.01).clamp_range(0.0..=1.0));
                                ui.add(egui::DragValue::new(&mut value[1]).speed(0.01).clamp_range(0.0..=1.0));
                            });
                            ui_state.set_parameter_value(&format!("{}_x", param.name), value[0]);
                            ui_state.set_parameter_value(&format!("{}_y", param.name), value[1]);
                        },
                        ParamType::Vec3 => {
                            let mut value = [
                                ui_state.get_parameter_value(&format!("{}_x", param.name)).unwrap_or(0.5),
                                ui_state.get_parameter_value(&format!("{}_y", param.name)).unwrap_or(0.5),
                                ui_state.get_parameter_value(&format!("{}_z", param.name)).unwrap_or(0.5)
                            ];
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: ", param.name));
                                ui.add(egui::DragValue::new(&mut value[0]).speed(0.01).clamp_range(0.0..=1.0));
                                ui.add(egui::DragValue::new(&mut value[1]).speed(0.01).clamp_range(0.0..=1.0));
                                ui.add(egui::DragValue::new(&mut value[2]).speed(0.01).clamp_range(0.0..=1.0));
                            });
                            ui_state.set_parameter_value(&format!("{}_x", param.name), value[0]);
                            ui_state.set_parameter_value(&format!("{}_y", param.name), value[1]);
                            ui_state.set_parameter_value(&format!("{}_z", param.name), value[2]);
                        },
                        ParamType::Vec4 => {
                            let mut value = [
                                ui_state.get_parameter_value(&format!("{}_x", param.name)).unwrap_or(0.5),
                                ui_state.get_parameter_value(&format!("{}_y", param.name)).unwrap_or(0.5),
                                ui_state.get_parameter_value(&format!("{}_z", param.name)).unwrap_or(0.5),
                                ui_state.get_parameter_value(&format!("{}_w", param.name)).unwrap_or(0.5)
                            ];
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: ", param.name));
                                ui.add(egui::DragValue::new(&mut value[0]).speed(0.01).clamp_range(0.0..=1.0));
                                ui.add(egui::DragValue::new(&mut value[1]).speed(0.01).clamp_range(0.0..=1.0));
                                ui.add(egui::DragValue::new(&mut value[2]).speed(0.01).clamp_range(0.0..=1.0));
                                ui.add(egui::DragValue::new(&mut value[3]).speed(0.01).clamp_range(0.0..=1.0));
                            });
                            ui_state.set_parameter_value(&format!("{}_x", param.name), value[0]);
                            ui_state.set_parameter_value(&format!("{}_y", param.name), value[1]);
                            ui_state.set_parameter_value(&format!("{}_z", param.name), value[2]);
                            ui_state.set_parameter_value(&format!("{}_w", param.name), value[3]);
                        },
                    }
                }
            }
        });
}

#[derive(Debug, Clone)]
pub struct ShaderParameter {
    pub name: String,
    pub param_type: ParamType,
}

#[derive(Debug, Clone)]
pub enum ParamType {
    Float,
    Vec2,
    Vec3,
    Vec4,
}

pub fn parse_shader_parameters(shader_code: &str) -> Vec<ShaderParameter> {
    let mut params = Vec::new();
    
    // Simple regex-based parsing for uniform parameters
    // This is a basic implementation - a full implementation would use proper WGSL parsing
    for line in shader_code.lines() {
        if line.contains("var<uniform>") {
            // Look for variable declarations like: var<uniform> my_param: f32;
            if let Some(start) = line.find("var<uniform>") {
                let remaining = &line[start + 13..]; // "var<uniform>".len() = 13
                if let Some(name_start) = remaining.find(|c: char| c.is_alphanumeric() || c == '_') {
                    let name_part = &remaining[name_start..];
                    if let Some(name_end) = name_part.find(|c: char| !c.is_alphanumeric() && c != '_') {
                        let param_name = name_part[..name_end].to_string();
                        
                        // Determine type based on the line
                        let param_type = if line.contains(": f32") {
                            ParamType::Float
                        } else if line.contains(": vec2") {
                            ParamType::Vec2
                        } else if line.contains(": vec3") {
                            ParamType::Vec3
                        } else if line.contains(": vec4") {
                            ParamType::Vec4
                        } else {
                            ParamType::Float // default
                        };
                        
                        params.push(ShaderParameter {
                            name: param_name,
                            param_type,
                        });
                    }
                }
            }
        }
    }
    
    params
}