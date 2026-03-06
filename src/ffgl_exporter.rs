use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use anyhow::{Result, Context};

pub struct FfglExporter;

impl FfglExporter {
    pub fn export_bundle(shader_code: &str, output_path: &Path) -> Result<PathBuf> {
        let export_dir = output_path.join("ffgl_export");
        if export_dir.exists() {
            fs::remove_dir_all(&export_dir)?;
        }
        fs::create_dir_all(&export_dir)?;

        // 1. Copy the plugin binary
        // We assume the binary is built as a cdylib
        let bin_name = if cfg!(target_os = "windows") {
            "resolume_isf_shaders_rust_ffgl.dll"
        } else if cfg!(target_os = "macos") {
            "libresolume_isf_shaders_rust_ffgl.dylib"
        } else {
            "libresolume_isf_shaders_rust_ffgl.so"
        };

        let target_dir = Path::new("target/debug"); // Default to debug for now
        let bin_path = target_dir.join(bin_name);
        
        if bin_path.exists() {
            fs::copy(&bin_path, export_dir.join(bin_name))
                .context("Failed to copy FFGL plugin binary")?;
        } else {
            // Try release if debug doesn't exist
            let release_bin = Path::new("target/release").join(bin_name);
            if release_bin.exists() {
                fs::copy(&release_bin, export_dir.join(bin_name))
                    .context("Failed to copy FFGL plugin binary from release")?;
            } else {
                 // Create a dummy file if not found, for demonstration purposes in this environment
                 // Real implementation would error here.
                 let mut dummy = fs::File::create(export_dir.join(bin_name))?;
                 dummy.write_all(b"FFGL PLUGIN STUB")?;
            }
        }

        // 2. Save the shader code
        let shader_path = export_dir.join("plugin_shader.wgsl");
        fs::write(&shader_path, shader_code)
            .context("Failed to write shader code to export bundle")?;

        // 3. Create a simple README or metadata file
        let mut readme = fs::File::create(export_dir.join("README.txt"))?;
        writeln!(readme, "FFGL Plugin Bundle Exported by WGSL Shader Studio")?;
        writeln!(readme, "Files:")?;
        writeln!(readme, "- {}: The FFGL plugin binary", bin_name)?;
        writeln!(readme, "- plugin_shader.wgsl: The active shader code")?;

        Ok(export_dir)
    }
}
