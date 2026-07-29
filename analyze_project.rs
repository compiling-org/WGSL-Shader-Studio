use std::fs;

// Include the UI analyzer code directly
include!("src/ui_analyzer.rs");

fn main() {
    let mut analyzer = UIAnalyzer::new();
    let report = analyzer.run_comprehensive_analysis();
    println!("{}", report);
}