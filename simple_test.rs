use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn main() {
    println!("Starting simple test...");
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simple Test".to_string(),
                resolution: bevy::window::WindowResolution::new(300, 200),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Update, test_ui_system)
        .run();
    
    println!("Window closed successfully");
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn test_ui_system(mut egui_ctx: EguiContexts) {
    let ctx = egui_ctx.ctx_mut().expect("Failed to get egui context");
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Simple Test");
        ui.label("If you can see this, the Bevy + bevy_egui GUI is working!");
    });
}