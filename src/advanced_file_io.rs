//! Advanced file I/O system for shader studio
//! 
//! This module provides comprehensive file operations including shader loading/saving,
//! screenshot/video export, project management, and asset handling with proper
//! error handling and performance optimization.

use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use image::{DynamicImage, ImageBuffer, Rgba, ImageFormat};
use serde::{Serialize, Deserialize};

/// Supported shader file formats
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShaderFileFormat {
    WGSL,
    GLSL,
    HLSL,
    ISF,
    JSON,
}

/// Supported image export formats
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ImageExportFormat {
    Png,
    Jpeg,
    Bmp,
    Tiff,
    WebP,
    Tga,
}

/// Supported video export formats
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VideoExportFormat {
    Mp4,
    WebM,
    Gif,
    Apng,
    Mov,
    Avi,
}

/// Project file structure for saving/loading complete shader projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderProject {
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: u64,
    pub modified_at: u64,
    pub shaders: HashMap<String, ShaderFile>,
    pub assets: Vec<AssetReference>,
    pub settings: ProjectSettings,
    pub timeline: Option<crate::timeline::Timeline>,
    pub node_graph: Option<crate::node_graph::NodeGraph>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderFile {
    pub name: String,
    pub format: ShaderFileFormat,
    pub content: String,
    pub metadata: ShaderMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderMetadata {
    pub author: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub version: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReference {
    pub id: String,
    pub name: String,
    pub path: String,
    pub asset_type: AssetType,
    pub file_size: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Audio,
    Video,
    Font,
    Model,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub render_width: u32,
    pub render_height: u32,
    pub frame_rate: f32,
    pub audio_enabled: bool,
    pub gesture_control_enabled: bool,
    pub auto_save_interval: u64, // seconds
    pub backup_count: u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            render_width: 1920,
            render_height: 1080,
            frame_rate: 60.0,
            audio_enabled: true,
            gesture_control_enabled: false,
            auto_save_interval: 300, // 5 minutes
            backup_count: 5,
        }
    }
}

/// Advanced file I/O manager
pub struct FileIOManager {
    project_dir: PathBuf,
    cache_dir: PathBuf,
    export_dir: PathBuf,
    backup_dir: PathBuf,
    project: Option<ShaderProject>,
    recent_files: Vec<PathBuf>,
    max_recent_files: usize,
}

impl FileIOManager {
    pub fn new(base_dir: &Path) -> Result<Self> {
        let project_dir = base_dir.join("projects");
        let cache_dir = base_dir.join("cache");
        let export_dir = base_dir.join("exports");
        let backup_dir = base_dir.join("backups");
        
        // Create directories if they don't exist
        fs::create_dir_all(&project_dir)?;
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(&export_dir)?;
        fs::create_dir_all(&backup_dir)?;
        
        Ok(Self {
            project_dir,
            cache_dir,
            export_dir,
            backup_dir,
            project: None,
            recent_files: Vec::new(),
            max_recent_files: 10,
        })
    }

    /// Load a shader file from disk
    pub fn load_shader_file(&self, file_path: &Path) -> Result<ShaderFile> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read shader file: {:?}", file_path))?;
        
        let format = self.detect_shader_format(file_path, &content)?;
        let name = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let metadata = self.extract_shader_metadata(&content, format)?;
        
        Ok(ShaderFile {
            name,
            format,
            content,
            metadata,
        })
    }

    /// Save a shader file to disk
    pub fn save_shader_file(&self, shader: &ShaderFile, file_path: &Path) -> Result<()> {
        let mut file = fs::File::create(file_path)
            .with_context(|| format!("Failed to create shader file: {:?}", file_path))?;
        
        // Add metadata header if it's a supported format
        let content_with_metadata = self.add_metadata_header(&shader.content, &shader.metadata, shader.format)?;
        
        file.write_all(content_with_metadata.as_bytes())
            .with_context(|| format!("Failed to write shader file: {:?}", file_path))?;
        
        Ok(())
    }

    /// Load a complete project from disk
    pub fn load_project(&mut self, project_path: &Path) -> Result<ShaderProject> {
        let content = fs::read_to_string(project_path)
            .with_context(|| format!("Failed to read project file: {:?}", project_path))?;
        
        let project: ShaderProject = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse project file: {:?}", project_path))?;
        
        // Verify all referenced assets exist
        self.verify_assets(&project)?;
        
        self.project = Some(project.clone());
        self.add_to_recent_files(project_path);
        
        Ok(project)
    }

    /// Save a complete project to disk
    pub fn save_project(&self, project: &ShaderProject, project_path: &Path) -> Result<()> {
        // Create backup if file exists
        if project_path.exists() {
            self.create_backup(project_path)?;
        }
        
        let content = serde_json::to_string_pretty(project)
            .with_context(|| "Failed to serialize project")?;
        
        let mut file = fs::File::create(project_path)
            .with_context(|| format!("Failed to create project file: {:?}", project_path))?;
        
        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write project file: {:?}", project_path))?;
        
        Ok(())
    }

    /// Export screenshot/image
    pub fn export_image(
        &self,
        pixel_data: &[u8],
        width: u32,
        height: u32,
        format: ImageExportFormat,
        file_path: &Path,
    ) -> Result<()> {
        // Convert pixel data to image
        let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, pixel_data.to_vec())
            .ok_or_else(|| anyhow::anyhow!("Invalid pixel data dimensions"))?;
        
        let dynamic_image = DynamicImage::ImageRgba8(image);
        
        // Save in requested format
        let image_format = match format {
            ImageExportFormat::Png => ImageFormat::Png,
            ImageExportFormat::Jpeg => ImageFormat::Jpeg,
            ImageExportFormat::Bmp => ImageFormat::Bmp,
            ImageExportFormat::Tiff => ImageFormat::Tiff,
            ImageExportFormat::WebP => ImageFormat::WebP,
            ImageExportFormat::Tga => ImageFormat::Tga,
        };
        
        dynamic_image.save_with_format(file_path, image_format)
            .with_context(|| format!("Failed to export image: {:?}", file_path))?;
        
        Ok(())
    }

    /// Export video sequence
    pub fn export_video_sequence(
        &self,
        frames: &[Vec<u8>],
        width: u32,
        height: u32,
        fps: f32,
        format: VideoExportFormat,
        file_path: &Path,
    ) -> Result<()> {
        // For now, we'll save as a sequence of images
        // Full video encoding would require additional dependencies
        let base_name = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("sequence");
        
        let extension = match format {
            VideoExportFormat::Gif => "gif",
            VideoExportFormat::Apng => "png",
            _ => "png", // Default to PNG sequence
        };
        
        for (i, frame) in frames.iter().enumerate() {
            let frame_path = file_path.parent()
                .unwrap_or(&self.export_dir)
                .join(format!("{}_{:04}.{}", base_name, i, extension));
            
            self.export_image(
                frame,
                width,
                height,
                ImageExportFormat::Png,
                &frame_path,
            )?;
        }
        
        Ok(())
    }

    /// Load shader files from a directory
    pub fn load_shader_directory(&self, dir_path: &Path) -> Result<Vec<ShaderFile>> {
        if !dir_path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {:?}", dir_path));
        }
        
        let mut shaders = Vec::new();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && self.is_shader_file(&path) {
                match self.load_shader_file(&path) {
                    Ok(shader) => shaders.push(shader),
                    Err(e) => eprintln!("Failed to load shader {:?}: {}", path, e),
                }
            }
        }
        
        Ok(shaders)
    }

    /// Create a new project with default settings
    pub fn create_new_project(&mut self, name: &str, description: Option<String>) -> ShaderProject {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let project = ShaderProject {
            version: "1.0.0".to_string(),
            name: name.to_string(),
            description,
            created_at: now,
            modified_at: now,
            shaders: HashMap::new(),
            assets: Vec::new(),
            settings: ProjectSettings::default(),
            timeline: None,
            node_graph: None,
        };
        
        self.project = Some(project.clone());
        project
    }

    /// Get current project
    pub fn get_current_project(&self) -> Option<&ShaderProject> {
        self.project.as_ref()
    }

    /// Get recent files list
    pub fn get_recent_files(&self) -> &[PathBuf] {
        &self.recent_files
    }

    /// Clear recent files list
    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
    }

    /// Detect shader format from file extension and content
    fn detect_shader_format(&self, file_path: &Path, content: &str) -> Result<ShaderFileFormat> {
        if let Some(ext) = file_path.extension() {
            match ext.to_str().unwrap_or("") {
                "wgsl" => return Ok(ShaderFileFormat::WGSL),
                "glsl" | "vert" | "frag" => return Ok(ShaderFileFormat::GLSL),
                "hlsl" => return Ok(ShaderFileFormat::HLSL),
                "fs" => return Ok(ShaderFileFormat::ISF),
                "json" => return Ok(ShaderFileFormat::JSON),
                _ => {}
            }
        }
        
        // Fallback to content detection
        if content.contains("@group") || content.contains("@binding") {
            Ok(ShaderFileFormat::WGSL)
        } else if content.contains("#version") {
            Ok(ShaderFileFormat::GLSL)
        } else if content.contains("cbuffer") || content.contains("Texture2D") {
            Ok(ShaderFileFormat::HLSL)
        } else if content.contains("\"NAME\"") && content.contains("\"INPUTS\"") {
            Ok(ShaderFileFormat::ISF)
        } else {
            Err(anyhow::anyhow!("Cannot detect shader format for file: {:?}", file_path))
        }
    }

    /// Extract shader metadata from content
    fn extract_shader_metadata(&self, content: &str, format: ShaderFileFormat) -> Result<ShaderMetadata> {
        let mut metadata = ShaderMetadata {
            author: None,
            description: None,
            tags: Vec::new(),
            category: None,
            version: None,
            license: None,
        };
        
        // Extract metadata from comments
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                let comment = trimmed.trim_start_matches("//").trim_start_matches("/*").trim_end_matches("*/").trim();
                
                if comment.starts_with("Author:") {
                    metadata.author = Some(comment.trim_start_matches("Author:").trim().to_string());
                } else if comment.starts_with("Description:") {
                    metadata.description = Some(comment.trim_start_matches("Description:").trim().to_string());
                } else if comment.starts_with("Tags:") {
                    let tags_str = comment.trim_start_matches("Tags:").trim();
                    metadata.tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
                } else if comment.starts_with("Category:") {
                    metadata.category = Some(comment.trim_start_matches("Category:").trim().to_string());
                } else if comment.starts_with("Version:") {
                    metadata.version = Some(comment.trim_start_matches("Version:").trim().to_string());
                } else if comment.starts_with("License:") {
                    metadata.license = Some(comment.trim_start_matches("License:").trim().to_string());
                }
            }
        }
        
        Ok(metadata)
    }

    /// Add metadata header to shader content
    fn add_metadata_header(&self, content: &str, metadata: &ShaderMetadata, format: ShaderFileFormat) -> Result<String> {
        let mut header = String::new();
        
        match format {
            ShaderFileFormat::WGSL | ShaderFileFormat::GLSL | ShaderFileFormat::HLSL => {
                if let Some(author) = &metadata.author {
                    header.push_str(&format!("// Author: {}\n", author));
                }
                if let Some(description) = &metadata.description {
                    header.push_str(&format!("// Description: {}\n", description));
                }
                if !metadata.tags.is_empty() {
                    header.push_str(&format!("// Tags: {}\n", metadata.tags.join(", ")));
                }
                if let Some(category) = &metadata.category {
                    header.push_str(&format!("// Category: {}\n", category));
                }
                if let Some(version) = &metadata.version {
                    header.push_str(&format!("// Version: {}\n", version));
                }
                if let Some(license) = &metadata.license {
                    header.push_str(&format!("// License: {}\n", license));
                }
                header.push('\n');
            }
            ShaderFileFormat::ISF => {
                // ISF format has JSON metadata
                let mut json_metadata = serde_json::Map::new();
                json_metadata.insert("NAME".to_string(), serde_json::Value::String(metadata.name.clone()));
                
                if let Some(description) = &metadata.description {
                    json_metadata.insert("DESCRIPTION".to_string(), serde_json::Value::String(description.clone()));
                }
                if let Some(credit) = &metadata.author {
                    json_metadata.insert("CREDIT".to_string(), serde_json::Value::String(credit.clone()));
                }
                if !metadata.tags.is_empty() {
                    json_metadata.insert("KEYWORDS".to_string(), serde_json::Value::Array(
                        metadata.tags.iter().map(|s| serde_json::Value::String(s.clone())).collect()
                    ));
                }
                
                header.push_str(&format!("/*{}*/\n", serde_json::to_string_pretty(&json_metadata)?));
            }
            ShaderFileFormat::JSON => {
                // JSON format doesn't need a header
            }
        }
        
        Ok(format!("{}{}", header, content))
    }

    /// Check if file is a shader file
    fn is_shader_file(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension() {
            match ext.to_str().unwrap_or("") {
                "wgsl" | "glsl" | "vert" | "frag" | "hlsl" | "fs" | "json" => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Verify all assets referenced in a project
    fn verify_assets(&self, project: &ShaderProject) -> Result<()> {
        for asset in &project.assets {
            let asset_path = Path::new(&asset.path);
            if !asset_path.exists() {
                return Err(anyhow::anyhow!("Asset not found: {:?}", asset_path));
            }
        }
        Ok(())
    }

    /// Create backup of a file
    fn create_backup(&self, file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            return Ok(());
        }
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        let file_name = file_path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("backup");
        
        let backup_path = self.backup_dir
            .join(format!("{}_{}", timestamp, file_name));
        
        fs::copy(file_path, backup_path)
            .with_context(|| format!("Failed to create backup for: {:?}", file_path))?;
        
        Ok(())
    }

    /// Add file to recent files list
    fn add_to_recent_files(&mut self, file_path: &Path) {
        // Remove if already exists
        self.recent_files.retain(|p| p != file_path);
        
        // Add to front
        self.recent_files.insert(0, file_path.to_path_buf());
        
        // Trim to max size
        if self.recent_files.len() > self.max_recent_files {
            self.recent_files.truncate(self.max_recent_files);
        }
    }
}

impl Default for FileIOManager {
    fn default() -> Self {
        Self::new(Path::new(".")).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_io_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileIOManager::new(temp_dir.path());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_shader_format_detection() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileIOManager::new(temp_dir.path()).unwrap();
        
        // Test WGSL detection
        let wgsl_content = "@group(0) @binding(0) var<uniform> time: f32;";
        let format = manager.detect_shader_format(Path::new("test.wgsl"), wgsl_content);
        assert_eq!(format.unwrap(), ShaderFileFormat::WGSL);
        
        // Test GLSL detection
        let glsl_content = "#version 450 core";
        let format = manager.detect_shader_format(Path::new("test.glsl"), glsl_content);
        assert_eq!(format.unwrap(), ShaderFileFormat::GLSL);
        
        // Test ISF detection
        let isf_content = r#"{"NAME": "Test", "INPUTS": []}"#;
        let format = manager.detect_shader_format(Path::new("test.fs"), isf_content);
        assert_eq!(format.unwrap(), ShaderFileFormat::ISF);
    }

    #[test]
    fn test_project_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = FileIOManager::new(temp_dir.path()).unwrap();
        
        let project = manager.create_new_project("Test Project", Some("A test project".to_string()));
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, Some("A test project".to_string()));
        assert!(project.shaders.is_empty());
    }

    #[test]
    fn test_recent_files() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = FileIOManager::new(temp_dir.path()).unwrap();
        
        let test_path = Path::new("/test/path.wgsl");
        manager.add_to_recent_files(test_path);
        
        assert_eq!(manager.get_recent_files().len(), 1);
        assert_eq!(manager.get_recent_files()[0], test_path);
    }
}