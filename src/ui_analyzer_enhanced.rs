use crate::ui_analyzer::{UIAnalyzer, FeatureCheck, FeatureStatus, Priority};

pub struct AnalysisSummary {
    pub total_features: usize,
    pub functional_count: usize,
    pub broken_count: usize,
    pub missing_count: usize,
    pub completion_percentage: f32,
}

pub struct UIAnalyzerEnhanced {
    inner: UIAnalyzer,
}

impl UIAnalyzerEnhanced {
    pub fn new() -> Self {
        Self { inner: UIAnalyzer::new() }
    }

    pub fn analyze_current_codebase(&mut self) {
        self.inner.analyze_current_state();
    }

    pub fn get_features_by_status(&self, status: FeatureStatus) -> Vec<FeatureCheck> {
        self.inner.get_features_by_status(status)
    }

    pub fn get_features_by_status_and_priority(&self, status: FeatureStatus, priority: Priority) -> Vec<FeatureCheck> {
        self.inner.get_features_by_status_and_priority(status, priority)
    }

    pub fn get_summary(&self) -> AnalysisSummary {
        let total = self.inner.get_total_features();
        let functional = self.inner.get_functional_features_count();
        let broken = self.inner.get_broken_features_count();
        let missing = self.inner.get_missing_features_count();
        let completion = if total > 0 { (functional as f32 / total as f32) * 100.0 } else { 0.0 };
        AnalysisSummary {
            total_features: total,
            functional_count: functional,
            broken_count: broken,
            missing_count: missing,
            completion_percentage: completion,
        }
    }

    pub fn generate_detailed_report(&self) -> String {
        let mut out = String::new();
        out.push_str("WGSL Shader Studio - Enhanced UI Analyzer Report\n");
        out.push_str("================================================\n\n");
        let summary = self.get_summary();
        out.push_str(&format!("Total Features: {}\n", summary.total_features));
        out.push_str(&format!("Functional: {}\n", summary.functional_count));
        out.push_str(&format!("Broken: {}\n", summary.broken_count));
        out.push_str(&format!("Missing: {}\n", summary.missing_count));
        out.push_str(&format!("Completion: {:.1}%\n\n", summary.completion_percentage));
        out.push_str("Detailed Features:\n");
        for status in [FeatureStatus::Functional, FeatureStatus::Partial, FeatureStatus::Broken, FeatureStatus::Missing] {
            let list = self.get_features_by_status(status.clone());
            if list.is_empty() { continue; }
            out.push_str(&format!("\n{:?}:\n", status));
            for f in list {
                out.push_str(&format!("- {} ({})\n", f.name, f.category));
                for d in f.details {
                    out.push_str(&format!("  • {}\n", d));
                }
            }
        }
        out
    }
}
