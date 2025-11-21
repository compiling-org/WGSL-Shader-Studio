use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

#[derive(Resource, Default)]
struct TestState {
    counter: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Minimal GUI Test".to_string(),
                resolution: bevy::window::WindowResolution::new(400, 300),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_resource::<TestState>()
        .add_systems(Startup, setup_camera)
        .add_systems(Update, test_ui_system)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn test_ui_system(mut egui_ctx: EguiContexts, mut state: ResMut<TestState>) {
    let ctx = match egui_ctx.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return, // Context not ready yet, skip this frame
    };
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Minimal GUI Test");
        ui.label("If you can see this window, the Bevy + bevy_egui framework is working correctly.");
        ui.label(format!("Counter: {}", state.counter));
        if ui.button("Increment").clicked() {
            state.counter += 1;
        }
    });
    
    // Request repaint for animation
    ctx.request_repaint_after(std::time::Duration::from_millis(100));
}