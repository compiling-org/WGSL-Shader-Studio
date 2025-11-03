use eframe;

fn main() {
    println!("Starting simple test...");
    
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([300.0, 200.0])
            .with_position([100.0, 100.0])
            .with_title("Simple Test"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "Simple Test",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    );
    
    match result {
        Ok(_) => println!("Window closed successfully"),
        Err(e) => eprintln!("Failed to create window: {}", e),
    }
}

struct MyApp {}

impl Default for MyApp {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simple Test");
            ui.label("If you can see this, the GUI is working!");
        });
    }
}