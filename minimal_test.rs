//! Minimal test to check if the GUI is working properly

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_position([100.0, 100.0])
            .with_title("Minimal Test")
            .with_visible(true)
            .with_active(true)
            .with_decorations(true)
            .with_transparent(false),
        vsync: false,
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        "Minimal Test",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp::default()))
        }),
    ).unwrap();
}

struct MyApp {
    counter: usize,
}

impl Default for MyApp {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Minimal Test");
            ui.label("If you can see this, the GUI framework is working");
            ui.label(format!("Counter: {}", self.counter));
            if ui.button("Increment").clicked() {
                self.counter += 1;
            }
        });
    }
}