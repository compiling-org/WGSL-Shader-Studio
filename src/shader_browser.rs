use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Add walkdir dependency that's used in the file
use walkdir;

// Shader browser module for managing shader files and projects

/// Shader file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderFile {
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
    pub size: u64,
    pub modified: std::time::SystemTime,
    pub category: ShaderCategory,
    pub tags: Vec<String>,
}

/// Shader categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShaderCategory {
    WGSL,
    ISF,
    GLSL,
    HLSL,
    Custom,
    Unknown,
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub shaders: Vec<ShaderFile>,
    pub created: std::time::SystemTime,
    pub modified: std::time::SystemTime,
    pub description: String,
}

/// Shader browser for managing projects and shader files
pub struct ShaderBrowser {
    pub projects: HashMap<String, Project>,
    pub current_project: Option<String>,
    pub scan_paths: Vec<PathBuf>,
    pub recent_files: Vec<PathBuf>,
    pub favorites: Vec<PathBuf>,
    pub search_query: String,
    pub filter_category: Option<ShaderCategory>,
    pub sort_by: SortBy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    Name,
    Date,
    Size,
    Category,
}

impl Default for ShaderBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderBrowser {
    pub fn new() -> Self {
        let mut browser = Self {
            projects: HashMap::new(),
            current_project: None,
            scan_paths: vec![
                PathBuf::from("./shaders"),
                PathBuf::from("./examples"),
                PathBuf::from("./projects"),
            ],
            recent_files: Vec::new(),
            favorites: Vec::new(),
            search_query: String::new(),
            filter_category: None,
            sort_by: SortBy::Name,
        };
        
        // Scan for existing projects and shaders
        browser.scan_directories();
        browser
    }
    
    /// Scan directories for shader files and projects
    pub fn scan_directories(&mut self) {
        for path in &self.scan_paths.clone() {
            self.scan_directory(path);
        }
    }
    
    /// Scan a single directory for shader files
    pub fn scan_directory(&mut self, path: &Path) {
        if !path.exists() {
            return;
        }
        
        let walker = match walkdir::WalkDir::new(path)
            .follow_links(true)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file()) {
            Ok(walker) => walker,
            Err(_) => return,
        };
        
        for entry in walker {
            let path = entry.path();
            if let Some(shader_file) = self.analyze_shader_file(path) {
                // Add to recent files if not already present
                if !self.recent_files.contains(&path.to_path_buf()) {
                    self.recent_files.push(path.to_path_buf());
                    if self.recent_files.len() > 20 {
                        self.recent_files.remove(0);
                    }
                }
            }
        }
    }
    
    /// Analyze a shader file and extract metadata
    fn analyze_shader_file(&self, path: &Path) -> Option<ShaderFile> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        let category = match extension.as_str() {
            "wgsl" => ShaderCategory::WGSL,
            "isf" => ShaderCategory::ISF,
            "glsl" => ShaderCategory::GLSL,
            "hlsl" => ShaderCategory::HLSL,
            _ => return None,
        };
        
        let metadata = fs::metadata(path).ok()?;
        let name = path.file_stem()?.to_str()?.to_string();
        
        // Extract tags from filename or content
        let mut tags = vec![];
        if name.contains("gradient") {
            tags.push("gradient".to_string());
        }
        if name.contains("noise") {
            tags.push("noise".to_string());
        }
        if name.contains("fractal") {
            tags.push("fractal".to_string());
        }
        
        Some(ShaderFile {
            path: path.to_path_buf(),
            name,
            extension,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(std::time::SystemTime::now()),
            category,
            tags,
        })
    }
    
    /// Create a new project
    pub fn create_project(&mut self, name: String, description: String) -> Result<(), String> {
        if self.projects.contains_key(&name) {
            return Err("Project name already exists".to_string());
        }
        
        let project_path = PathBuf::from("./projects").join(&name);
        fs::create_dir_all(&project_path).map_err(|e| e.to_string())?;
        
        let project = Project {
            name: name.clone(),
            path: project_path,
            shaders: Vec::new(),
            created: std::time::SystemTime::now(),
            modified: std::time::SystemTime::now(),
            description,
        };
        
        self.projects.insert(name.clone(), project);
        self.current_project = Some(name);
        Ok(())
    }
    
    /// Load a shader file
    pub fn load_shader(&self, path: &Path) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| e.to_string())
    }
    
    /// Save a shader file
    pub fn save_shader(&self, path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(path, content).map_err(|e| e.to_string())
    }
    
    /// Import ISF shader and convert to WGSL
    pub fn import_isf_shader(&self, path: &Path) -> Result<String, String> {
        let content = self.load_shader(path)?;
        // Basic ISF to WGSL conversion (simplified)
        self.convert_isf_to_wgsl(&content)
    }
    
    /// Basic ISF to WGSL conversion
    fn convert_isf_to_wgsl(&self, isf_content: &str) -> Result<String, String> {
        // This is a simplified conversion - in a real implementation,
        // you would use a proper ISF parser and converter
        
        let mut wgsl_code = String::new();
        wgsl_code.push_str("struct Uniforms {\n");
        wgsl_code.push_str("    time: f32,\n");
        wgsl_code.push_str("    resolution: vec2<f32>,\n");
        wgsl_code.push_str("    mouse: vec2<f32>,\n");
        wgsl_code.push_str("};\n\n");
        
        wgsl_code.push_str("@group(0) @binding(0)\n");
        wgsl_code.push_str("var<uniform> uniforms: Uniforms;\n\n");
        
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {\n");
        wgsl_code.push_str("    let uv = position.xy / uniforms.resolution;\n");
        wgsl_code.push_str("    let time = uniforms.time;\n\n");
        
        // Simple conversion of ISF main function body
        if isf_content.contains("gl_FragColor") {
            // Extract main function body (simplified)
            if let Some(start) = isf_content.find("void main") {
                if let Some(body_start) = isf_content[start..].find('{') {
                    if let Some(body_end) = isf_content[start + body_start..].rfind('}') {
                        let body = &isf_content[start + body_start + 1..start + body_start + body_end];
                        let converted_body = body
                            .replace("gl_FragColor", "color")
                            .replace("gl_FragCoord", "position")
                            .replace("isf_FragNormCoord", "uv");
                        wgsl_code.push_str(&converted_body);
                    }
                }
            }
        } else {
            // Default shader if no main function found
            wgsl_code.push_str("    let color = vec4<f32>(uv.x, uv.y, 0.5, 1.0);\n");
        }
        
        wgsl_code.push_str("\n    return color;\n");
        wgsl_code.push_str("}\n");
        
        Ok(wgsl_code)
    }
    
    /// Search for shaders by name, category, or tags
    pub fn search_shaders(&self, query: &str) -> Vec<ShaderFile> {
        let query = query.to_lowercase();
        let mut results = Vec::new();
        
        // Search in recent files
        for path in &self.recent_files {
            if let Some(shader) = self.analyze_shader_file(path) {
                if self.matches_search(&shader, &query) {
                    results.push(shader);
                }
            }
        }
        
        results
    }
    
    /// Check if a shader matches the search query
    fn matches_search(&self, shader: &ShaderFile, query: &str) -> bool {
        shader.name.to_lowercase().contains(query) ||
        shader.tags.iter().any(|tag| tag.to_lowercase().contains(query)) ||
        format!("{:?}", shader.category).to_lowercase().contains(query)
    }
    
    /// Get filtered and sorted shader list
    pub fn get_filtered_shaders(&self) -> Vec<ShaderFile> {
        let mut shaders = if self.search_query.is_empty() {
            // Get all shaders from recent files
            self.recent_files.iter()
                .filter_map(|path| self.analyze_shader_file(path))
                .collect()
        } else {
            self.search_shaders(&self.search_query)
        };
        
        // Filter by category
        if let Some(category) = &self.filter_category {
            shaders.retain(|s| s.category == *category);
        }
        
        // Sort
        match self.sort_by {
            SortBy::Name => shaders.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::Date => shaders.sort_by(|a, b| b.modified.cmp(&a.modified)),
            SortBy::Size => shaders.sort_by(|a, b| b.size.cmp(&a.size)),
            SortBy::Category => shaders.sort_by(|a, b| format!("{:?}", a.category).cmp(&format!("{:?}", b.category))),
        }
        
        shaders
    }
    
    /// Export project to file
    pub fn export_project(&self, project_name: &str, path: &Path) -> Result<(), String> {
        if let Some(project) = self.projects.get(project_name) {
            let json = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;
            fs::write(path, json).map_err(|e| e.to_string())
        } else {
            Err("Project not found".to_string())
        }
    }
    
    /// Import project from file
    pub fn import_project(&mut self, path: &Path) -> Result<String, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let project: Project = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let name = project.name.clone();
        self.projects.insert(name.clone(), project);
        Ok(name)
    }
}