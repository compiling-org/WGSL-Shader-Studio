use std::fs;
use std::path::Path;
use anyhow::Result;

use crate::fosfora::loader::{EffectLoader, PfxEffect};

/// Test that .pfx files can be parsed correctly.
#[cfg(test)]
mod test_pfx_parser {
    use super::*;

    /// Test parsing of sample .pfx files with the new loader.
    #[test]
    fn test_pfx_parsing() -> Result<()> {
        // Create a temporary directory with test .pfx files
        let temp_dir = tempfile::tempdir()?;
        
        // Test with the lumen.pfx file from the reference repo
        let lumen_path = "reference_repos/fosfora/assets/effects/lumen.pfx";
        assert!(Path::new(lumen_path).exists(), "lumen.pfx not found");
        
        let content = fs::read_to_string(lumen_path)?;
        let effect: PfxEffect = serde_json::from_str(&content)?;
        
        assert_eq!(effect.name, "Lumen");
        assert!(!effect.description.is_empty());
        assert!(effect.passes.is_empty());
        
        // Check that audio_mappings exists and has mappings
        assert_eq!(effect.audio_mappings.len(), 8);
        for mapping in &effect.audio_mappings {
            assert!(!mapping.feature.is_empty());
            assert!(!mapping.target.is_empty());
        }
        
        // Check that passes exist
        assert!(!effect.inputs.is_empty());
        assert!(!effect.shader.is_empty());
        
        println!("Successfully parsed {} .pfx effect", effect.name);
        Ok(())
    }
}