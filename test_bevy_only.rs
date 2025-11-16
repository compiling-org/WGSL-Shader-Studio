// Test if bevy_app.rs compiles by itself
#[path = "src/bevy_app.rs"]
mod bevy_app;

fn main() {
    println!("Testing bevy_app compilation...");
    // bevy_app::run_app(); // Don't actually run, just test compilation
