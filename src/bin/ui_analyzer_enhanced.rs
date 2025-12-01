use std::fs;
use wgsl_shader_studio::ui_analyzer_enhanced::{UIAnalyzerEnhanced, FeatureStatus, Priority};

fn main() {
    println!("🔍 WGSL Shader Studio - ENHANCED COMPREHENSIVE UI ANALYZER");
    println!("==========================================================\n");

    let mut analyzer = UIAnalyzerEnhanced::new();
    
    println!("🔄 Running comprehensive analysis of all 27+ backend features...");
    analyzer.analyze_current_codebase();
    
    let report = analyzer.generate_detailed_report();
    
    println!("\n📊 Analysis complete!");
    println!("\n📝 GENERATING SURGICAL ANALYSIS REPORT...\n");
    
    // Save report to file
    match std::fs::write("UI_AUDIT_REPORT_ENHANCED.md", &report) {
        Ok(_) => println!("✅ Enhanced report saved to: UI_AUDIT_REPORT_ENHANCED.md"),
        Err(e) => println!("❌ Failed to save report: {}", e),
    }
    
    // Print surgical summary to console
    println!("\n📈 SURGICAL ANALYSIS SUMMARY:");
    println!("------------------------------");
    
    let summary = analyzer.get_summary();
    println!("Total Features Analyzed: {}", summary.total_features);
    println!("Functional Features: {}", summary.functional_count);
    println!("Broken Features: {}", summary.broken_count);
    println!("Missing Features: {}", summary.missing_count);
    println!("Completion Percentage: {:.1}%", summary.completion_percentage);
    
    let critical_missing = analyzer.get_features_by_status_and_priority(FeatureStatus::Missing, Priority::Critical);
    let high_missing = analyzer.get_features_by_status_and_priority(FeatureStatus::Missing, Priority::High);
    let broken_critical = analyzer.get_features_by_status_and_priority(FeatureStatus::Broken, Priority::Critical);
    
    if !critical_missing.is_empty() {
        println!("\n🚨 CRITICAL MISSING FEATURES (MUST FIX):");
        for feature in critical_missing {
            println!("  - {} ({}) - {}", feature.name, feature.category, feature.description);
        }
    }
    
    if !broken_critical.is_empty() {
        println!("\n💥 CRITICAL BROKEN FEATURES (URGENT REPAIR):");
        for feature in broken_critical {
            println!("  - {} ({}) - {}", feature.name, feature.category, feature.description);
        }
    }
    
    if !high_missing.is_empty() {
        println!("\n⚠️  HIGH PRIORITY MISSING FEATURES:");
        for feature in high_missing {
            println!("  - {} ({}) - {}", feature.name, feature.category, feature.description);
        }
    }
    
    let functional = analyzer.get_features_by_status(FeatureStatus::Functional);
    if !functional.is_empty() {
        println!("\n✅ FUNCTIONAL FEATURES (WORKING):");
        for feature in functional.iter().take(10) { // Show first 10
            println!("  - {} ({})", feature.name, feature.category);
        }
        if functional.len() > 10 {
            println!("  ... and {} more functional features", functional.len() - 10);
        }
    }
    
    println!("\n🔧 NEXT STEPS FOR SYSTEMATIC DEVELOPMENT:");
    println!("1. Fix all critical missing/broken features first");
    println!("2. Implement high priority missing features");
    println!("3. Test and validate each feature systematically");
    println!("4. Check detailed report in UI_AUDIT_REPORT_ENHANCED.md");
    
    println!("\n📋 DETAILED ANALYSIS COMPLETE");
    println!("   No more psychotic loops. Systematic precision only.");
}