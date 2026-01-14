use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::editor_ui::EditorUiState;

pub fn draw_3d_scene_panel(ctx: &egui::Context, ui_state: &mut EditorUiState) {
    egui::Window::new("3D Scene Panel")
        .open(&mut ui_state.show_3d_scene_panel)
        .show(ctx, |ui| {
            ui.heading("3D Scene Configuration");
            
            ui.separator();
            ui.heading("Camera Settings");
            
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.add(egui::DragValue::new(&mut ui_state.camera_position[0]).speed(0.1).prefix("X: "));
                ui.add(egui::DragValue::new(&mut ui_state.camera_position[1]).speed(0.1).prefix("Y: "));
                ui.add(egui::DragValue::new(&mut ui_state.camera_position[2]).speed(0.1).prefix("Z: "));
            });
            
            ui.horizontal(|ui| {
                ui.label("Rotation:");
                ui.add(egui::DragValue::new(&mut ui_state.camera_rotation[0]).speed(0.1).prefix("X: "));
                ui.add(egui::DragValue::new(&mut ui_state.camera_rotation[1]).speed(0.1).prefix("Y: "));
                ui.add(egui::DragValue::new(&mut ui_state.camera_rotation[2]).speed(0.1).prefix("Z: "));
            });
            
            ui.add(egui::Slider::new(&mut ui_state.camera_fov, 30.0..=120.0).text("Field of View"));
            ui.add(egui::Slider::new(&mut ui_state.camera_near, 0.01..=1.0).text("Near Plane"));
            ui.add(egui::Slider::new(&mut ui_state.camera_far, 10.0..=1000.0).text("Far Plane"));
            
            ui.separator();
            ui.heading("Lighting");
            
            ui.horizontal(|ui| {
                ui.label("Light Position:");
                ui.add(egui::DragValue::new(&mut ui_state.light_position[0]).speed(0.1).prefix("X: "));
                ui.add(egui::DragValue::new(&mut ui_state.light_position[1]).speed(0.1).prefix("Y: "));
                ui.add(egui::DragValue::new(&mut ui_state.light_position[2]).speed(0.1).prefix("Z: "));
            });
            
            ui.horizontal(|ui| {
                ui.label("Light Color:");
                ui.add(egui::DragValue::new(&mut ui_state.light_color[0]).speed(0.01).clamp_range(0.0..=1.0).prefix("R: "));
                ui.add(egui::DragValue::new(&mut ui_state.light_color[1]).speed(0.01).clamp_range(0.0..=1.0).prefix("G: "));
                ui.add(egui::DragValue::new(&mut ui_state.light_color[2]).speed(0.01).clamp_range(0.0..=1.0).prefix("B: "));
            });
            
            ui.add(egui::Slider::new(&mut ui_state.light_intensity, 0.0..=5.0).text("Light Intensity"));
            
            ui.horizontal(|ui| {
                ui.label("Ambient Color:");
                ui.add(egui::DragValue::new(&mut ui_state.ambient_light_color[0]).speed(0.01).clamp_range(0.0..=1.0).prefix("R: "));
                ui.add(egui::DragValue::new(&mut ui_state.ambient_light_color[1]).speed(0.01).clamp_range(0.0..=1.0).prefix("G: "));
                ui.add(egui::DragValue::new(&mut ui_state.ambient_light_color[2]).speed(0.01).clamp_range(0.0..=1.0).prefix("B: "));
            });
            
            ui.add(egui::Slider::new(&mut ui_state.ambient_light_intensity, 0.0..=1.0).text("Ambient Intensity"));
            
            ui.separator();
            ui.heading("Scene Objects");
            
            if ui.button("Add Sphere").clicked() {
                println!("Adding sphere to scene");
            }
            
            if ui.button("Add Cube").clicked() {
                println!("Adding cube to scene");
            }
            
            if ui.button("Add Plane").clicked() {
                println!("Adding plane to scene");
            }
            
            if ui.button("Add Light").clicked() {
                println!("Adding light to scene");
            }
            
            ui.separator();
            ui.heading("Scene Actions");
            
            if ui.button("Reset Scene").clicked() {
                // Reset scene to default state
                ui_state.camera_position = [0.0, 0.0, 5.0];
                ui_state.camera_rotation = [0.0, 0.0, 0.0];
                ui_state.light_position = [2.0, 2.0, 2.0];
                println!("Scene reset to default");
            }
            
            if ui.button("Load Scene").clicked() {
                println!("Loading scene from file");
            }
            
            if ui.button("Save Scene").clicked() {
                println!("Saving scene to file");
            }
        });
}

// Additional functions for 3D scene management
pub fn update_3d_scene_from_ui(ui_state: &EditorUiState) -> SceneUpdate {
    SceneUpdate {
        camera_position: Vec3::from_array(ui_state.camera_position),
        camera_rotation: Vec3::from_array(ui_state.camera_rotation),
        camera_fov: ui_state.camera_fov,
        light_position: Vec3::from_array(ui_state.light_position),
        light_color: Color::rgb(ui_state.light_color[0], ui_state.light_color[1], ui_state.light_color[2]),
        light_intensity: ui_state.light_intensity,
        ambient_light_color: Color::rgb(ui_state.ambient_light_color[0], ui_state.ambient_light_color[1], ui_state.ambient_light_color[2]),
        ambient_light_intensity: ui_state.ambient_light_intensity,
    }
}

pub struct SceneUpdate {
    pub camera_position: Vec3,
    pub camera_rotation: Vec3,
    pub camera_fov: f32,
    pub light_position: Vec3,
    pub light_color: Color,
    pub light_intensity: f32,
    pub ambient_light_color: Color,
    pub ambient_light_intensity: f32,
}