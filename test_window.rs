// Minimal test to check if eframe works on Windows
use eframe::egui;

fn main() {
    println!("Testing basic eframe window...");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("Basic Test Window"),
    };

    eframe::run_native(
        "Basic Test",
        options,
        Box::new(|_cc| {
            Box::new(TestApp::default())
        }),
    ).unwrap();
}

#[derive(Default)]
struct TestApp {}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Basic eframe test");
            ui.label("If you can see this, eframe is working!");
            
            if ui.button("Click me").clicked() {
                println!("Button clicked!");
            }
        });
    }
}