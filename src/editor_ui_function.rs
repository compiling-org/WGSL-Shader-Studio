// Helper that draws the browser/parameters/timeline panels using a provided egui context
pub fn draw_editor_side_panels(ctx: &egui::Context, ui_state: &mut EditorUiState, _audio_analyzer: &AudioAnalyzer) {
    // FIX: Use proper panel hierarchy to avoid CentralPanel conflicts
    
    // Left panel - Shader Browser
    if ui_state.show_shader_browser {
        egui::SidePanel::left("shader_browser").resizable(true).show(ctx, |ui| {
            ui.heading("Shader Browser");
            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.show_all_shaders, "Show all shaders");
                if !ui_state.show_all_shaders {
                    ui.label("Showing compatible only (has @vertex and @fragment)");
                }
            });
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut ui_state.search_query);
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                let names = if ui_state.show_all_shaders {
                    ui_state.available_shaders_all.clone()
                } else {
                    ui_state.available_shaders_compatible.clone()
                };
                for name in names.iter() {
                    if !ui_state.search_query.is_empty() && !name.to_lowercase().contains(&ui_state.search_query.to_lowercase()) {
                        continue;
                    }
                    let selected = ui.selectable_label(ui_state.selected_shader.as_ref().map(|s| s == name).unwrap_or(false), name);
                    if selected.clicked() {
                        ui_state.selected_shader = Some(name.clone());
                        // Load the shader immediately
                        if let Ok(content) = std::fs::read_to_string(name) {
                            // Check if this is an ISF file (.fs extension)
                            if name.to_lowercase().ends_with(".fs") {
                                // Parse as ISF and convert to WGSL
                                match crate::isf_loader::IsfShader::parse(&name, &content) {
                                    Ok(isf_shader) => {
                                        // Convert ISF to WGSL using the ISF converter
                                        let mut converter = super::isf_converter::IsfConverter::new();
                                        match converter.convert_to_wgsl(&isf_shader) {
                                            Ok(wgsl_code) => ui_state.draft_code = wgsl_code,
                                            Err(e) => {
                                                println!("Failed to convert ISF to WGSL: {}", e);
                                                ui_state.draft_code = content; // Fallback to raw content
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("Failed to parse ISF: {}", e);
                                        ui_state.draft_code = content; // Fallback to raw content
                                    }
                                }
                            } else {
                                // Regular WGSL file
                                ui_state.draft_code = content;
                            }
                            // Clear any previous errors
                            ui_state.last_error = None;
                        } else {
                            ui_state.last_error = Some(format!("Failed to read file: {}", name));
                        }
                    }
                }
            });
        });
    }

    // Right panel - Parameters
    if ui_state.show_parameters {
        egui::SidePanel::right("parameters").resizable(true).show(ctx, |ui| {
            ui.heading("Parameters");
            ui.separator();
            
            // Parse parameters from current shader code
            let params = crate::editor_ui::parse_shader_parameters(&ui_state.draft_code);
            if params.is_empty() {
                ui.label("No parameters found in shader");
            } else {
                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    for param in params.iter() {
                        ui.horizontal(|ui| {
                            ui.label(&param.name);
                            match param.param_type {
                                crate::editor_ui::ParameterType::Float => {
                                    let mut value = ui_state.get_parameter_value(&param.name).unwrap_or(param.default_value);
                                    let response = ui.add(egui::DragValue::new(&mut value).speed(0.01).clamp_range(param.min_value..=param.max_value));
                                    if response.changed() {
                                        ui_state.set_parameter_value(&param.name, value);
                                    }
                                }
                                crate::editor_ui::ParameterType::Int => {
                                    let mut value = ui_state.get_parameter_value(&param.name).unwrap_or(param.default_value) as i32;
                                    let response = ui.add(egui::DragValue::new(&mut value).speed(1).clamp_range((param.min_value as i32)..=(param.max_value as i32)));
                                    if response.changed() {
                                        ui_state.set_parameter_value(&param.name, value as f32);
                                    }
                                }
                                crate::editor_ui::ParameterType::Bool => {
                                    let mut value = ui_state.get_parameter_value(&param.name).unwrap_or(param.default_value) != 0.0;
                                    let response = ui.checkbox(&mut value, "");
                                    if response.changed() {
                                        ui_state.set_parameter_value(&param.name, if value { 1.0 } else { 0.0 });
                                    }
                                }
                                crate::editor_ui::ParameterType::Vec2 => {
                                    let mut values = [0.0f32; 2];
                                    let base_value = ui_state.get_parameter_value(&param.name).unwrap_or(param.default_value);
                                    values[0] = base_value;
                                    values[1] = ui_state.get_parameter_value(&format!("{}_y", param.name)).unwrap_or(param.default_value);
                                    let response = ui.horizontal(|ui| {
                                        ui.drag_value(&mut values[0]).speed(0.01).clamp_range(param.min_value..=param.max_value);
                                        ui.drag_value(&mut values[1]).speed(0.01).clamp_range(param.min_value..=param.max_value);
                                    });
                                    if response.response.changed() {
                                        ui_state.set_parameter_value(&param.name, values[0]);
                                        ui_state.set_parameter_value(&format!("{}_y", param.name), values[1]);
                                    }
                                }
                            }
                        });
                        ui.separator();
                    }
                });
            }
        });
    }

    // Bottom panel - Timeline
    if ui_state.show_timeline {
        egui::TopBottomPanel::bottom("timeline").resizable(true).min_height(100.0).show(ctx, |ui| {
            ui.heading("Timeline");
            ui.separator();
            
            // Timeline controls
            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() {
                    ui_state.timeline.current_time = 0.0;
                }
                if ui.button("⏯").clicked() {
                    ui_state.timeline.playing = !ui_state.timeline.playing;
                }
                if ui.button("⏹").clicked() {
                    ui_state.timeline.playing = false;
                    ui_state.timeline.current_time = 0.0;
                }
                
                ui.label(format!("Time: {:.2}s", ui_state.timeline.current_time));
                ui.label(format!("Duration: {:.2}s", ui_state.timeline.duration));
            });
            
            // Timeline scrubber
            ui.horizontal(|ui| {
                ui.label("0.0");
                let mut time = ui_state.timeline.current_time;
                let response = ui.add(egui::Slider::new(&mut time, 0.0..=ui_state.timeline.duration).text("Time"));
                if response.changed() {
                    ui_state.timeline.current_time = time;
                }
                ui.label(format!("{:.1}", ui_state.timeline.duration));
            });
            
            // Keyframe list
            ui.separator();
            ui.label("Keyframes:");
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                for (i, keyframe) in ui_state.timeline.keyframes.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: {:.2}s - {}", i + 1, keyframe.time, keyframe.parameter));
                        if ui.small_button("X").clicked() {
                            // Remove keyframe
                            // This would need to be implemented properly
                        }
                    });
                }
            });
        });
    }
}