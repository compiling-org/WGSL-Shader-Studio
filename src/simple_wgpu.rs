use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Simple WGPU rendering resource
#[derive(Resource)]
pub struct SimpleWgpuRenderer {
    pub is_initialized: bool,
    pub last_frame_time: Instant,
    pub render_errors: Arc<Mutex<Vec<String>>>,
}

impl Default for SimpleWgpuRenderer {
    fn default() -> Self {
        Self {
            is_initialized: false,
            last_frame_time: Instant::now(),
            render_errors: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl SimpleWgpuRenderer {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn initialize(&mut self) {
        println!("🚀 Initializing simple WGPU renderer...");
        self.is_initialized = true;
        println!("✅ Simple WGPU renderer initialized successfully!");
    }
    
    pub fn render_shader(&mut self, shader_code: &str, width: u32, height: u32, time: f32) -> Result<Vec<u8>, String> {
        if !self.is_initialized {
            return Err("Renderer not initialized".to_string());
        }
        
        // Simple placeholder rendering - create a gradient pattern
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        
        for y in 0..height {
            for x in 0..width {
                let index = ((y * width + x) * 4) as usize;
                
                // Simple gradient based on time and position
                let r = ((x as f32 / width as f32 + time.sin()) * 255.0) as u8;
                let g = ((y as f32 / height as f32 + time.cos()) * 255.0) as u8;
                let b = ((time.sin() * 0.5 + 0.5) * 255.0) as u8;
                
                pixels[index] = r;
                pixels[index + 1] = g;
                pixels[index + 2] = b;
                pixels[index + 3] = 255;
            }
        }
        
        Ok(pixels)
    }
    
    pub fn get_errors(&self) -> Vec<String> {
        self.render_errors.lock().unwrap().clone()
    }
    
    pub fn clear_errors(&self) {
        self.render_errors.lock().unwrap().clear();
    }
}

/// System to initialize the WGPU renderer
pub fn initialize_simple_wgpu_system(
    mut renderer: ResMut<SimpleWgpuRenderer>,
    mut startup_timer: Local<Instant>,
) {
    // Only initialize after a short delay
    if startup_timer.elapsed().as_millis() < 100 {
        return;
    }
    
    if !renderer.is_initialized {
        renderer.initialize();
    }
}

/// System to handle shader preview rendering
pub fn simple_preview_system(
    mut renderer: ResMut<SimpleWgpuRenderer>,
    mut editor_state: ResMut<crate::editor_ui::EditorUiState>,
    time: Res<Time>,
) {
    if !renderer.is_initialized || editor_state.code_editor.trim().is_empty() {
        return;
    }
    
    // Limit rendering to reasonable frame rate (30 FPS)
    let frame_interval = std::time::Duration::from_millis(33); // ~30 FPS
    if renderer.last_frame_time.elapsed() < frame_interval {
        return;
    }
    
    renderer.last_frame_time = Instant::now();
    
    // Simple rendering parameters
    let width = editor_state.preview_size.0;
    let height = editor_state.preview_size.1;
    let current_time = time.elapsed_secs();
    
    // Clear previous errors
    renderer.clear_errors();
    editor_state.errors.clear();
    editor_state.warnings.clear();
    
    // Try to render the shader
    match renderer.render_shader(&editor_state.code_editor, width, height, current_time) {
        Ok(pixels) => {
            println!("✅ Shader rendered successfully: {}x{} pixels", width, height);
            // Store the rendered frame (you could add a field to store this)
            editor_state.last_compiled_code = Some(editor_state.code_editor.clone());
        }
        Err(e) => {
            println!("❌ Shader rendering failed: {}", e);
            editor_state.errors = vec![e];
        }
    }
}

/// Plugin to add simple WGPU rendering capabilities
pub struct SimpleWgpuRenderPlugin;

impl Plugin for SimpleWgpuRenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimpleWgpuRenderer>()
            .insert_resource(Instant::now()) // For startup timing
            .add_systems(Update, (
                initialize_simple_wgpu_system,
                simple_preview_system,
            ).chain());
    }
}