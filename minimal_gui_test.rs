use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_position([100.0, 100.0])
            .with_title("Minimal GUI Test")
            .with_visible(true)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Minimal GUI Test",
        options,
        Box::new(|_cc| Ok(Box::new(TestApp::default()))),
    ).unwrap();
}

struct TestApp {
    counter: usize,
}

impl Default for TestApp {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Minimal GUI Test");
            ui.label("If you can see this window, the GUI framework is working correctly.");
            ui.label(format!("Counter: {}", self.counter));
            if ui.button("Increment").clicked() {
                self.counter += 1;
            }
        });
        
        // Request repaint for animation
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}