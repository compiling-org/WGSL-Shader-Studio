use resolume_isf_shaders_rust_ffgl::ui_analyzer::UIAnalyzer;

fn main() {
    println!("🔍 WGSL Shader Studio - Comprehensive UI Analyzer");
    println!("==================================================\n");

    let mut analyzer = UIAnalyzer::new();
    
    println!("🔄 Running comprehensive analysis...");
    let report = analyzer.run_comprehensive_analysis();
    
    println!("📊 Analysis complete!");
    println!("\n📝 Generating detailed report...\n");
    
    // Save report to file
    match std::fs::write("UI_AUDIT_REPORT.md", &report) {
        Ok(_) => println!("✅ Report saved to: UI_AUDIT_REPORT.md"),
        Err(e) => println!("❌ Failed to save report: {}", e),
    }
    
    // Print summary to console
    println!("\n📈 SUMMARY:");
    println!("-----------");
    
    let total_features = analyzer.get_total_features();
    let critical_missing = analyzer.get_missing_critical_features().len();
    let high_missing = analyzer.get_missing_high_priority_features().len();
    let critical_broken = analyzer.get_broken_critical_features().len();
    
    println!("Total Features Required: {}", total_features);
    println!("Critical Missing: {}", critical_missing);
    println!("High Priority Missing: {}", high_missing);
    println!("Critical Broken: {}", critical_broken);
    println!("Functional Features: {}", analyzer.get_functional_features_count());
    println!("Partial Features: {}", analyzer.get_partial_features_count());
    
    if critical_missing > 0 {
        println!("\n🚨 CRITICAL MISSING FEATURES:");
        for feature in analyzer.get_missing_critical_features() {
            println!("  - {} ({})", feature.name, feature.category);
        }
    }
    
    if critical_broken > 0 {
        println!("\n💥 CRITICAL BROKEN FEATURES:");
        for feature in analyzer.get_broken_critical_features() {
            println!("  - {} ({})", feature.name, feature.category);
        }
    }
    
    println!("\n📋 NEXT STEPS:");
    println!("1. Fix critical missing features first");
    println!("2. Repair broken critical features");
    println!("3. Implement high priority missing features");
    println!("4. Complete partial features");
    println!("5. Test all functionality");
    
    println!("\n📄 Full detailed report available in: UI_AUDIT_REPORT.md");
}