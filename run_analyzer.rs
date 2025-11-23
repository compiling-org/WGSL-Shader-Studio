use std::process::Command;

fn main() {
    // Run the UI analyzer to get actual current state
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ui_analyzer"])
        .output()
        .expect("Failed to run UI analyzer");
    
    println!("{}", String::from_utf8_lossy(&output.stdout));
    
    if !output.stderr.is_empty() {
        println!("Errors: {}", String::from_utf8_lossy(&output.stderr));
    }
}