use std::fs;
use std::path::Path;
use std::collections::HashMap;
use anyhow::{Result, anyhow};

/// Simple ISF integration for loading ISF files
pub struct IsfIntegration {
    isf_directory: String,
}

impl IsfIntegration {
    pub fn new() -> Self {
        Self {
            isf_directory: "C:\\Program Files\\Magic\\Modules2\\ISF".to_string(),
        }
    }

    /// Load ISF files from Magic directory
    pub fn load_magic_isfs(&self) -> Result<Vec<String>> {
        let paths = vec![
            "C:\\Program Files\\Magic\\Modules2\\ISF",
            "C:\\Program Files (x86)\\Magic\\Modules2\\ISF",
            "C:\\Magic\\Modules2\\ISF",
            "C:\\Program Files\\Magic\\ISF",
        ];

        for path in &paths {
            if Path::new(path).exists() {
                return self.load_isfs_from_directory(path);
            }
        }

        Err(anyhow!("Magic ISF directory not found. Please specify the correct path."))
    }

    /// Load ISF files from directory
    pub fn load_isfs_from_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>> {
        let mut isf_files = Vec::new();
        
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("fs") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        isf_files.push(name.to_string());
                    }
                }
            }
        }
        
        Ok(isf_files)
    }

    /// Load a single ISF file
    pub fn load_isf_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to load ISF file: {}", e))
    }

    /// Convert ISF to WGSL (basic conversion)
    pub fn convert_isf_to_wgsl(&self, isf_content: &str) -> String {
        let mut wgsl_code = String::new();
        
        // Basic WGSL template with ISF uniforms
        wgsl_code.push_str("struct Uniforms {\n");
        wgsl_code.push_str("    time: f32,\n");
        wgsl_code.push_str("    timeDelta: f32,\n");
        wgsl_code.push_str("    frame: u32,\n");
        wgsl_code.push_str("    fps: f32,\n");
        wgsl_code.push_str("    progress: f32,\n");
        wgsl_code.push_str("    renderSize: vec2<f32>,\n");
        wgsl_code.push_str("    aspectRatio: f32,\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("@group(0) @binding(0) var<uniform> uniforms: Uniforms;\n\n");
        
        // Vertex shader
        wgsl_code.push_str("@vertex\n");
        wgsl_code.push_str("fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {\n");
        wgsl_code.push_str("    return vec4<f32>(position, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n\n");
        
        // Fragment shader
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {\n");
        wgsl_code.push_str("    let uv = coord.xy / uniforms.renderSize;\n");
        wgsl_code.push_str("    let TIME = uniforms.time;\n");
        wgsl_code.push_str("    let RENDERSIZE = uniforms.renderSize;\n");
        wgsl_code.push_str("    let ASPECT = uniforms.aspectRatio;\n");
        
        // Convert ISF code
        let converted_code = self.convert_isf_functions(isf_content);
        wgsl_code.push_str(&converted_code);
        wgsl_code.push_str("\n    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n");
        
        wgsl_code
    }

    /// Convert ISF functions to WGSL
    fn convert_isf_functions(&self, code: &str) -> String {
        let mut converted = code.to_string();
        
        // Basic ISF to WGSL conversions
        converted = converted.replace("TIME", "uniforms.time");
        converted = converted.replace("TIMEDELTA", "uniforms.timeDelta");
        converted = converted.replace("FRAME", "f32(uniforms.frame)");
        converted = converted.replace("FPS", "uniforms.fps");
        converted = converted.replace("PROGRESS", "uniforms.progress");
        converted = converted.replace("RENDERSIZE", "uniforms.renderSize");
        converted = converted.replace("ASPECT", "uniforms.aspectRatio");
        
        converted = converted.replace("vec2(", "vec2<f32>(");
        converted = converted.replace("vec3(", "vec3<f32>(");
        converted = converted.replace("vec4(", "vec4<f32>(");
        
        converted
    }
}

impl Default for IsfIntegration {
    fn default() -> Self {
        Self::new()
    }
}