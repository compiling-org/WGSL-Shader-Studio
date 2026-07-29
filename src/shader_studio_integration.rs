use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::advanced_shader_compilation::{AdvancedShaderCompiler, CompiledShader};
use crate::isf_integration_advanced::{IsfToWgslConverter, IsfShaderLoader};
use crate::advanced_file_io::{FileIOManager, ShaderProject};
use crate::enhanced_audio_system::{AudioMidiIntegration, AudioConfig};
use crate::gesture_control_system::{UnifiedGestureSystem, GestureShaderUniforms};
use crate::timeline_animation_system::{TimelineAnimationSystem, Timeline, TimelineLayer, TimelineTrack, AnimationCurve};

pub struct ShaderStudioConfig {
    pub enable_audio_processing: bool,
    pub enable_gesture_control: bool,
    pub enable_timeline_animation: bool,
    pub enable_isf_support: bool,
    pub enable_file_io: bool,
    pub audio_config: AudioConfig,
    pub leapmotion_host: String,
    pub leapmotion_port: u16,
    pub default_frame_rate: f32,
    pub max_shader_cache_size: usize,
    pub auto_save_interval: Duration,
}

impl Default for ShaderStudioConfig {
    fn default() -> Self {
        Self {
            enable_audio_processing: true,
            enable_gesture_control: true,
            enable_timeline_animation: true,
            enable_isf_support: true,
            enable_file_io: true,
            audio_config: AudioConfig::default(),
            leapmotion_host: "localhost".to_string(),
            leapmotion_port: 6437,
            default_frame_rate: 60.0,
            max_shader_cache_size: 1000,
            auto_save_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

pub struct ShaderStudioIntegration {
    config: ShaderStudioConfig,
    shader_compiler: Arc<Mutex<AdvancedShaderCompiler>>,
    isf_converter: Arc<Mutex<IsfToWgslConverter>>,
    file_io_manager: Arc<Mutex<FileIOManager>>,
    audio_midi_system: Arc<Mutex<AudioMidiIntegration>>,
    gesture_system: Arc<Mutex<UnifiedGestureSystem>>,
    timeline_system: Arc<Mutex<TimelineAnimationSystem>>,
    is_initialized: Arc<Mutex<bool>>,
    last_update: Arc<Mutex<Instant>>,
    frame_count: Arc<Mutex<u64>>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub average_frame_time: f32,
    pub current_fps: f32,
    pub shader_compilation_time: f32,
    pub audio_processing_time: f32,
    pub gesture_processing_time: f32,
    pub timeline_update_time: f32,
    pub total_update_time: f32,
    pub frame_times: Vec<f32>,
    pub max_frame_history: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            average_frame_time: 0.0,
            current_fps: 0.0,
            shader_compilation_time: 0.0,
            audio_processing_time: 0.0,
            gesture_processing_time: 0.0,
            timeline_update_time: 0.0,
            total_update_time: 0.0,
            frame_times: Vec::new(),
            max_frame_history: 100,
        }
    }
}

impl ShaderStudioIntegration {
    pub fn new(config: ShaderStudioConfig) -> Self {
        Self {
            config,
            shader_compiler: Arc::new(Mutex::new(AdvancedShaderCompiler::new())),
            isf_converter: Arc::new(Mutex::new(IsfToWgslConverter::new())),
            file_io_manager: Arc::new(Mutex::new(FileIOManager::new("./projects"))),
            audio_midi_system: Arc::new(Mutex::new(AudioMidiIntegration::new())),
            gesture_system: Arc::new(Mutex::new(UnifiedGestureSystem::new())),
            timeline_system: Arc::new(Mutex::new(TimelineAnimationSystem::new())),
            is_initialized: Arc::new(Mutex::new(false)),
            last_update: Arc::new(Mutex::new(Instant::now())),
            frame_count: Arc::new(Mutex::new(0)),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), String> {
        let mut is_initialized = self.is_initialized.lock().unwrap();
        if *is_initialized {
            return Ok(());
        }

        let start_time = Instant::now();

        // Initialize audio system
        if self.config.enable_audio_processing {
            if let Err(e) = self.audio_midi_system.lock().unwrap().initialize().await {
                return Err(format!("Failed to initialize audio system: {:?}", e));
            }
        }

        // Initialize gesture system
        if self.config.enable_gesture_control {
            let mut gesture_system = self.gesture_system.lock().unwrap();
            
            // Initialize MediaPipe
            if let Err(e) = gesture_system.initialize_mediapipe().await {
                web_sys::console::warn_1(&format!("MediaPipe initialization failed: {:?}", e).into());
            }

            // Connect to LeapMotion (optional)
            if let Err(e) = gesture_system.connect_leapmotion(&self.config.leapmotion_host, self.config.leapmotion_port).await {
                web_sys::console::warn_1(&format!("LeapMotion connection failed: {:?}", e).into());
            }

            gesture_system.start_gesture_processing();
        }

        // Initialize timeline system with default timeline
        if self.config.enable_timeline_animation {
            let timeline_system = self.timeline_system.lock().unwrap();
            timeline_system.create_timeline("default", 60.0).unwrap();
            
            // Create default animation layers
            let main_layer = TimelineLayer {
                name: "Main".to_string(),
                tracks: Vec::new(),
                enabled: true,
                opacity: 1.0,
                blend_mode: crate::timeline_animation_system::BlendMode::Normal,
            };
            
            timeline_system.add_layer_to_timeline("default", main_layer).unwrap();
        }

        // Initialize file I/O system
        if self.config.enable_file_io {
            let mut file_io_manager = self.file_io_manager.lock().unwrap();
            file_io_manager.initialize().map_err(|e| format!("Failed to initialize file I/O: {}", e))?;
        }

        *is_initialized = true;
        
        let mut metrics = self.performance_metrics.lock().unwrap();
        metrics.total_update_time = start_time.elapsed().as_secs_f32() * 1000.0;

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) {
        let start_time = Instant::now();
        let mut metrics = self.performance_metrics.lock().unwrap();

        // Update timeline animation system
        if self.config.enable_timeline_animation {
            let timeline_start = Instant::now();
            self.timeline_system.lock().unwrap().update(delta_time);
            metrics.timeline_update_time = timeline_start.elapsed().as_secs_f32() * 1000.0;
        }

        // Update gesture system (runs continuously)
        if self.config.enable_gesture_control {
            let gesture_start = Instant::now();
            // Gesture system updates automatically via its internal processing loop
            metrics.gesture_processing_time = gesture_start.elapsed().as_secs_f32() * 1000.0;
        }

        // Update audio system (runs continuously)
        if self.config.enable_audio_processing {
            let audio_start = Instant::now();
            // Audio system updates automatically via its internal processing loop
            metrics.audio_processing_time = audio_start.elapsed().as_secs_f32() * 1000.0;
        }

        // Update frame metrics
        let frame_time = start_time.elapsed().as_secs_f32() * 1000.0;
        metrics.total_update_time = frame_time;
        
        if metrics.frame_times.len() >= metrics.max_frame_history {
            metrics.frame_times.remove(0);
        }
        metrics.frame_times.push(frame_time);
        
        metrics.average_frame_time = metrics.frame_times.iter().sum::<f32>() / metrics.frame_times.len() as f32;
        metrics.current_fps = if metrics.average_frame_time > 0.0 {
            1000.0 / metrics.average_frame_time
        } else {
            0.0
        };

        *self.frame_count.lock().unwrap() += 1;
    }

    pub fn compile_shader(&self, shader_source: &str, shader_type: &str) -> Result<CompiledShader, String> {
        let compilation_start = Instant::now();
        let mut compiler = self.shader_compiler.lock().unwrap();
        
        let result = match shader_type.to_lowercase().as_str() {
            "wgsl" => compiler.compile_wgsl(shader_source),
            "glsl" => compiler.convert_glsl_to_wgsl(shader_source),
            "hlsl" => compiler.convert_hlsl_to_wgsl(shader_source),
            "isf" => {
                // First convert ISF to WGSL
                let isf_converter = self.isf_converter.lock().unwrap();
                let wgsl_source = isf_converter.convert_isf_to_wgsl(shader_source)
                    .map_err(|e| format!("ISF conversion failed: {}", e))?;
                compiler.compile_wgsl(&wgsl_source)
            },
            _ => Err(format!("Unsupported shader type: {}", shader_type)),
        };

        let mut metrics = self.performance_metrics.lock().unwrap();
        metrics.shader_compilation_time = compilation_start.elapsed().as_secs_f32() * 1000.0;

        result
    }

    pub fn get_shader_uniforms(&self) -> ShaderStudioUniforms {
        let audio_uniforms = if self.config.enable_audio_processing {
            self.audio_midi_system.lock().unwrap().get_combined_shader_uniforms()
        } else {
            Default::default()
        };

        let gesture_uniforms = if self.config.enable_gesture_control {
            self.gesture_system.lock().unwrap().get_shader_uniforms()
        } else {
            Default::default()
        };

        let timeline_uniforms = if self.config.enable_timeline_animation {
            self.timeline_system.lock().unwrap().get_shader_uniforms()
        } else {
            Default::default()
        };

        ShaderStudioUniforms {
            audio: audio_uniforms,
            gesture: gesture_uniforms,
            timeline: timeline_uniforms,
            time: self.get_current_time(),
            frame: *self.frame_count.lock().unwrap() as f32,
            resolution: [1920.0, 1080.0], // Default resolution, should be updated
        }
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.lock().unwrap().clone()
    }

    pub fn is_initialized(&self) -> bool {
        *self.is_initialized.lock().unwrap()
    }

    pub fn get_current_time(&self) -> f32 {
        self.timeline_system.lock().unwrap().get_current_time()
    }

    pub fn get_frame_count(&self) -> u64 {
        *self.frame_count.lock().unwrap()
    }

    // Audio system control methods
    pub fn start_audio_capture(&self) -> Result<(), String> {
        if !self.config.enable_audio_processing {
            return Err("Audio processing is disabled".to_string());
        }

        // This would need to be implemented with proper async handling
        // For now, return a placeholder error
        Err("Audio capture start requires async context".to_string())
    }

    pub fn stop_audio_capture(&self) {
        if self.config.enable_audio_processing {
            // Implementation would depend on the audio system
        }
    }

    pub fn update_audio_config(&self, config: AudioConfig) {
        if self.config.enable_audio_processing {
            // Update the audio configuration
        }
    }

    // Gesture system control methods
    pub fn is_mediapipe_active(&self) -> bool {
        if self.config.enable_gesture_control {
            self.gesture_system.lock().unwrap().is_mediapipe_active()
        } else {
            false
        }
    }

    pub fn is_leapmotion_active(&self) -> bool {
        if self.config.enable_gesture_control {
            self.gesture_system.lock().unwrap().is_leapmotion_active()
        } else {
            false
        }
    }

    // Timeline system control methods
    pub fn play_timeline(&self) {
        if self.config.enable_timeline_animation {
            self.timeline_system.lock().unwrap().play();
        }
    }

    pub fn pause_timeline(&self) {
        if self.config.enable_timeline_animation {
            self.timeline_system.lock().unwrap().pause();
        }
    }

    pub fn stop_timeline(&self) {
        if self.config.enable_timeline_animation {
            self.timeline_system.lock().unwrap().stop();
        }
    }

    pub fn seek_timeline(&self, time: f32) {
        if self.config.enable_timeline_animation {
            self.timeline_system.lock().unwrap().seek_to_time(time);
        }
    }

    pub fn set_timeline_playback_speed(&self, speed: f32) {
        if self.config.enable_timeline_animation {
            self.timeline_system.lock().unwrap().set_playback_speed(speed);
        }
    }

    // File I/O methods
    pub fn save_project(&self, project_name: &str) -> Result<(), String> {
        if !self.config.enable_file_io {
            return Err("File I/O is disabled".to_string());
        }

        let mut file_io_manager = self.file_io_manager.lock().unwrap();
        
        // Create project data from current state
        let project_data = self.create_project_data();
        file_io_manager.save_project(project_name, project_data)
            .map_err(|e| format!("Failed to save project: {}", e))
    }

    pub fn load_project(&self, project_name: &str) -> Result<(), String> {
        if !self.config.enable_file_io {
            return Err("File I/O is disabled".to_string());
        }

        let mut file_io_manager = self.file_io_manager.lock().unwrap();
        
        let project_data = file_io_manager.load_project(project_name)
            .map_err(|e| format!("Failed to load project: {}", e))?;
        
        self.apply_project_data(project_data)
    }

    fn create_project_data(&self) -> ShaderProject {
        // Create project data from current system state
        ShaderProject {
            name: "Current Project".to_string(),
            shader_sources: HashMap::new(),
            uniform_data: self.get_shader_uniforms(),
            timeline_data: if self.config.enable_timeline_animation {
                self.timeline_system.lock().unwrap().export_timeline("default").ok()
            } else {
                None
            },
            audio_config: if self.config.enable_audio_processing {
                Some(self.audio_midi_system.lock().unwrap().audio_system.config.clone())
            } else {
                None
            },
            gesture_config: None,
            metadata: HashMap::new(),
        }
    }

    fn apply_project_data(&self, project_data: ShaderProject) -> Result<(), String> {
        // Apply project data to current system state
        if let Some(audio_config) = project_data.audio_config {
            self.update_audio_config(audio_config);
        }

        if let Some(timeline_data) = project_data.timeline_data {
            if self.config.enable_timeline_animation {
                self.timeline_system.lock().unwrap().import_timeline(&timeline_data)
                    .map_err(|e| format!("Failed to import timeline: {}", e))?;
            }
        }

        Ok(())
    }

    pub fn export_shader(&self, shader_name: &str, format: &str) -> Result<Vec<u8>, String> {
        if !self.config.enable_file_io {
            return Err("File I/O is disabled".to_string());
        }

        let file_io_manager = self.file_io_manager.lock().unwrap();
        file_io_manager.export_shader(shader_name, format)
            .map_err(|e| format!("Failed to export shader: {}", e))
    }

    pub fn take_screenshot(&self, width: u32, height: u32) -> Result<Vec<u8>, String> {
        if !self.config.enable_file_io {
            return Err("File I/O is disabled".to_string());
        }

        let file_io_manager = self.file_io_manager.lock().unwrap();
        file_io_manager.take_screenshot(width, height)
            .map_err(|e| format!("Failed to take screenshot: {}", e))
    }

    pub fn start_video_recording(&self, width: u32, height: u32, fps: f32) -> Result<(), String> {
        if !self.config.enable_file_io {
            return Err("File I/O is disabled".to_string());
        }

        let mut file_io_manager = self.file_io_manager.lock().unwrap();
        file_io_manager.start_video_recording(width, height, fps)
            .map_err(|e| format!("Failed to start video recording: {}", e))
    }

    pub fn stop_video_recording(&self) -> Result<Vec<u8>, String> {
        if !self.config.enable_file_io {
            return Err("File I/O is disabled".to_string());
        }

        let mut file_io_manager = self.file_io_manager.lock().unwrap();
        file_io_manager.stop_video_recording()
            .map_err(|e| format!("Failed to stop video recording: {}", e))
    }
}

#[derive(Debug, Clone)]
pub struct ShaderStudioUniforms {
    pub audio: crate::enhanced_audio_system::CombinedAudioMidiUniforms,
    pub gesture: GestureShaderUniforms,
    pub timeline: crate::timeline_animation_system::TimelineShaderUniforms,
    pub time: f32,
    pub frame: f32,
    pub resolution: [f32; 2],
}

impl Default for ShaderStudioUniforms {
    fn default() -> Self {
        Self {
            audio: Default::default(),
            gesture: Default::default(),
            timeline: Default::default(),
            time: 0.0,
            frame: 0.0,
            resolution: [1920.0, 1080.0],
        }
    }
}

impl ShaderStudioUniforms {
    pub fn to_wgsl_uniforms(&self) -> String {
        let mut uniforms = String::new();
        
        // Add time and frame uniforms
        uniforms.push_str(r#"
@group(4) @binding(0) var<uniform> u_time: f32;
@group(4) @binding(1) var<uniform> u_frame: f32;
@group(4) @binding(2) var<uniform> u_resolution: vec2<f32>;
"#);

        // Add audio uniforms
        uniforms.push_str(&self.audio.audio.to_wgsl_uniforms());
        
        // Add gesture uniforms
        uniforms.push_str(&self.gesture.to_wgsl_uniforms());
        
        // Add timeline uniforms
        uniforms.push_str(&self.timeline.to_wgsl_uniforms());

        uniforms
    }

    pub fn to_wgsl_struct(&self) -> String {
        let mut structs = String::new();
        
        structs.push_str(r#"
struct ShaderStudioData {
    time: f32,
    frame: f32,
    resolution: vec2<f32>,
}
"#);

        structs.push_str(&self.audio.audio.to_wgsl_struct());
        structs.push_str(&self.timeline.to_wgsl_struct());

        structs
    }

    pub fn get_binding_groups(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut entries = Vec::new();
        
        // Time and frame uniforms
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });

        // Add audio, gesture, and timeline binding entries
        // This would need to be implemented based on the specific uniform requirements

        entries
    }
}