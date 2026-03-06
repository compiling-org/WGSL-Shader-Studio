use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use crate::isf_loader::IsfLoader;
use crate::isf_converter::IsfConverter;

pub struct BatchConverter;

impl BatchConverter {
    pub fn new() -> Self {
        Self
    }

    /// Converts all ISF shaders in the source directory to WGSL in the target directory
    pub fn convert_all(&self, source_dir: &Path, target_dir: &Path) -> Result<Vec<(String, bool)>> {
        if !source_dir.exists() {
            return Err(anyhow::anyhow!("Source directory does not exist: {:?}", source_dir));
        }

        if !target_dir.exists() {
            fs::create_dir_all(target_dir).context("Failed to create target directory")?;
        }

        let mut results = Vec::new();
        let mut loader = IsfLoader::new();
        let mut converter = IsfConverter::new();

        for entry in fs::read_dir(source_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("fs") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                match self.convert_single_file(&path, target_dir, &mut loader, &mut converter) {
                    Ok(_) => results.push((name, true)),
                    Err(e) => {
                        eprintln!("Failed to convert {}: {}", name, e);
                        results.push((name, false));
                    }
                }
            }
        }

        Ok(results)
    }

    fn convert_single_file(
        &self,
        source_path: &Path,
        target_dir: &Path,
        loader: &mut IsfLoader,
        converter: &mut IsfConverter,
    ) -> Result<()> {
        let name = source_path.file_stem().unwrap().to_string_lossy();
        
        // Load the ISF shader
        let isf_shader = loader.load(source_path)
            .map_err(|e| anyhow::anyhow!("ISF Load Error: {}", e))?;
        
        // Convert to WGSL
        let wgsl_code = converter.convert_to_wgsl(&isf_shader)
            .map_err(|e| anyhow::anyhow!("WGSL Conversion Error: {}", e))?;
        
        // Save to target directory
        let target_path = target_dir.join(format!("{}.wgsl", name));
        fs::write(target_path, wgsl_code).context("Failed to write WGSL file")?;
        
        Ok(())
    }
}
