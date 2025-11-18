use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::Mutex;
use crate::{
    wgsl_rendering_system::WgslRenderPipeline,
    gyroflow_wgpu_interop::{
        WgpuInteropManager, InteropConfig, NativeTextureInfo, GraphicsApi,
        ZeroCopyTexture, InteropResult,
    },
};

#[derive(Debug, Clone)]
pub struct InteropIntegrationConfig {
    pub enable_zero_copy_preview: bool,
    pub enable_multi_api_support: bool,
    pub max_preview_resolution: (u32, u32),
    pub enable_performance_monitoring: bool,
    pub texture_cache_size: usize,
}

impl Default for InteropIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_zero_copy_preview: true,
            enable_multi_api_support: true,
            max_preview_resolution: (4096, 4096),
            enable_performance_monitoring: true,
            texture_cache_size: 32,
        }
    }
}

#[derive(Debug)]
pub struct InteropIntegration {
    config: InteropIntegrationConfig,
    interop_manager: Arc<WgpuInteropManager>,
    
    // Integration with existing rendering pipeline
    render_pipeline: Arc<Mutex<Option<WgslRenderPipeline>>>,
    
    // Preview texture management
    preview_textures: Arc<Mutex<HashMap<String, ZeroCopyTexture>>>,
    
    // Performance monitoring
    frame_times: Arc<Mutex<Vec<f64>>>,
    total_frames_processed: Arc<Mutex<u64>>,
    
    // Cross-platform compatibility flags
    platform_support: HashMap<String, bool>,
}

#[derive(Debug, Clone)]
pub struct InteropFrameStats {
    pub frame_time_ms: f64,
    pub zero_copy_used: bool,
    pub texture_cache_hit: bool,
    pub graphics_api: String,
    pub resolution: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct InteropPerformanceReport {
    pub average_frame_time_ms: f64,
    pub min_frame_time_ms: f64,
    pub max_frame_time_ms: f64,
    pub total_frames_processed: u64,
    pub zero_copy_usage_percentage: f64,
    pub cache_hit_rate: f64,
    pub graphics_apis_used: Vec<String>,
}

impl InteropIntegration {
    pub async fn new(
        config: InteropIntegrationConfig,
        interop_config: InteropConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let interop_manager = Arc::new(WgpuInteropManager::new(interop_config).await?);
        
        let mut platform_support = HashMap::new();
        platform_support.insert("directx11".to_string(), cfg!(windows));
        platform_support.insert("directx12".to_string(), cfg!(windows));
        platform_support.insert("opengl".to_string(), true);
        platform_support.insert("opengles".to_string(), true);
        platform_support.insert("metal".to_string(), cfg!(target_os = "macos"));
        platform_support.insert("vulkan".to_string(), true);

        Ok(Self {
            config,
            interop_manager,
            render_pipeline: Arc::new(Mutex::new(None)),
            preview_textures: Arc::new(Mutex::new(HashMap::new())),
            frame_times: Arc::new(Mutex::new(Vec::new())),
            total_frames_processed: Arc::new(Mutex::new(0)),
            platform_support,
        })
    }

    /// Initialize integration with existing render pipeline
    pub fn initialize_with_pipeline(&self, pipeline: WgslRenderPipeline) {
        let mut render_guard = self.render_pipeline.lock();
        *render_guard = Some(pipeline);
    }

    /// Create zero-copy preview texture for shader output
    pub async fn create_preview_texture(
        &self,
        width: u32,
        height: u32,
        label: &str,
        graphics_api: GraphicsApi,
    ) -> Result<ZeroCopyTexture, Box<dyn std::error::Error>> {
        // Validate resolution limits
        if width > self.config.max_preview_resolution.0 || height > self.config.max_preview_resolution.1 {
            return Err(format!(
                "Resolution {}x{} exceeds maximum allowed {}x{}",
                width, height,
                self.config.max_preview_resolution.0,
                self.config.max_preview_resolution.1
            ).into());
        }

        // Create native texture info (simplified for cross-platform compatibility)
        let native_info = NativeTextureInfo {
            api: graphics_api,
            texture_ptr: std::ptr::null_mut(), // Would be actual native pointer in real implementation
            width,
            height,
            format: wgpu::TextureFormat::Rgba8Unorm,
            is_srgb: true,
        };

        // Create zero-copy texture
        let zero_copy_texture = self.interop_manager.create_zero_copy_texture(
            native_info,
            label,
        )?;

        // Cache the preview texture
        let mut preview_guard = self.preview_textures.lock();
        preview_guard.insert(label.to_string(), zero_copy_texture.clone());

        // Maintain cache size limit
        if preview_guard.len() > self.config.texture_cache_size {
            let keys_to_remove: Vec<String> = preview_guard
                .keys()
                .take(preview_guard.len() - self.config.texture_cache_size)
                .cloned()
                .collect();
            
            for key in keys_to_remove {
                preview_guard.remove(&key);
            }
        }

        Ok(zero_copy_texture)
    }

    /// Process shader output with zero-copy interop
    pub fn process_shader_output(
        &self,
        shader_output: &ZeroCopyTexture,
        preview_label: &str,
        operation: &str,
    ) -> Result<InteropResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // Get or create preview texture
        let mut preview_guard = self.preview_textures.lock();
        let preview_texture = match preview_guard.get(preview_label) {
            Some(texture) => texture.clone(),
            None => {
                // Create new preview texture if not exists
                drop(preview_guard); // Release lock before creating new texture
                
                let new_texture = self.create_preview_texture(
                    shader_output.native_info.width,
                    shader_output.native_info.height,
                    preview_label,
                    shader_output.native_info.api,
                ).await?;
                
                preview_guard = self.preview_textures.lock();
                preview_guard.insert(preview_label.to_string(), new_texture.clone());
                new_texture
            }
        };

        // Process texture using interop
        let result = self.interop_manager.process_texture(
            shader_output,
            &preview_texture,
            operation,
        )?;

        // Record performance metrics
        if self.config.enable_performance_monitoring {
            let frame_time = start_time.elapsed().as_millis() as f64;
            let mut frame_times = self.frame_times.lock();
            frame_times.push(frame_time);
            
            // Keep only last 100 frame times
            if frame_times.len() > 100 {
                frame_times.remove(0);
            }
            
            *self.total_frames_processed.lock() += 1;
        }

        Ok(result)
    }

    /// Get performance report
    pub fn get_performance_report(&self) -> InteropPerformanceReport {
        let frame_times = self.frame_times.lock();
        let total_frames = *self.total_frames_processed.lock();
        
        if frame_times.is_empty() {
            return InteropPerformanceReport {
                average_frame_time_ms: 0.0,
                min_frame_time_ms: 0.0,
                max_frame_time_ms: 0.0,
                total_frames_processed: total_frames,
                zero_copy_usage_percentage: 0.0,
                cache_hit_rate: 0.0,
                graphics_apis_used: vec![],
            };
        }

        let avg_time = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
        let min_time = *frame_times.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_time = *frame_times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        // Get metrics from interop manager
        let metrics = self.interop_manager.get_metrics();
        let zero_copy_percentage = if total_frames > 0 {
            (metrics.zero_copy_operations as f64 / total_frames as f64) * 100.0
        } else {
            0.0
        };

        let cache_hit_rate = if metrics.texture_cache_hits + metrics.texture_cache_misses > 0 {
            (metrics.texture_cache_hits as f64 / 
                (metrics.texture_cache_hits + metrics.texture_cache_misses) as f64) * 100.0
        } else {
            0.0
        };

        InteropPerformanceReport {
            average_frame_time_ms: avg_time,
            min_frame_time_ms: min_time,
            max_frame_time_ms: max_time,
            total_frames_processed: total_frames,
            zero_copy_usage_percentage: zero_copy_percentage,
            cache_hit_rate,
            graphics_apis_used: self.get_supported_graphics_apis(),
        }
    }

    /// Get supported graphics APIs for current platform
    pub fn get_supported_graphics_apis(&self) -> Vec<String> {
        self.platform_support
            .iter()
            .filter(|(_, supported)| **supported)
            .map(|(api, _)| api.clone())
            .collect()
    }

    /// Check if specific graphics API is supported
    pub fn is_graphics_api_supported(&self, api: &str) -> bool {
        self.platform_support.get(api).copied().unwrap_or(false)
    }

    /// Clear all cached textures
    pub fn clear_cache(&self) {
        self.interop_manager.clear_texture_cache();
        self.preview_textures.lock().clear();
        self.frame_times.lock().clear();
        *self.total_frames_processed.lock() = 0;
    }

    /// Get current frame statistics
    pub fn get_frame_stats(&self) -> Option<InteropFrameStats> {
        let frame_times = self.frame_times.lock();
        if frame_times.is_empty() {
            return None;
        }

        let last_frame_time = *frame_times.last().unwrap();
        let metrics = self.interop_manager.get_metrics();
        
        Some(InteropFrameStats {
            frame_time_ms: last_frame_time,
            zero_copy_used: metrics.zero_copy_operations > 0,
            texture_cache_hit: metrics.texture_cache_hits > 0,
            graphics_api: "Multi-API".to_string(), // Would be actual API in real implementation
            resolution: (1920, 1080), // Would be actual resolution
        })
    }

    /// Update integration configuration
    pub fn update_config(&mut self, new_config: InteropIntegrationConfig) {
        self.config = new_config;
    }

    /// Get device and queue for external integration
    pub fn get_device_queue(&self) -> (Arc<wgpu::Device>, Arc<wgpu::Queue>) {
        self.interop_manager.get_device_queue()
    }

    /// Create integration report
    pub fn create_integration_report(&self) -> String {
        let report = self.get_performance_report();
        let supported_apis = self.get_supported_graphics_apis();
        
        format!(
            "WGSL Shader Studio - Gyroflow Interop Integration Report\n\
             =====================================================\n\
             Performance Metrics:\n\
             - Average Frame Time: {:.2}ms\n\
             - Min Frame Time: {:.2}ms\n\
             - Max Frame Time: {:.2}ms\n\
             - Total Frames Processed: {}\n\
             - Zero-copy Usage: {:.1}%\n\
             - Cache Hit Rate: {:.1}%\n\
             \n\
             Supported Graphics APIs: {:?}\n\
             \n\
             Integration Status: Active\n\
             Zero-copy Preview: {}\n\
             Multi-API Support: {}\n\
             Performance Monitoring: {}\n",
            report.average_frame_time_ms,
            report.min_frame_time_ms,
            report.max_frame_time_ms,
            report.total_frames_processed,
            report.zero_copy_usage_percentage,
            report.cache_hit_rate,
            supported_apis,
            self.config.enable_zero_copy_preview,
            self.config.enable_multi_api_support,
            self.config.enable_performance_monitoring,
        )
    }
}

// Integration helper functions
pub mod interop_utils {
    use super::*;

    /// Convert string to GraphicsApi
    pub fn string_to_graphics_api(api_str: &str) -> Option<GraphicsApi> {
        match api_str.to_lowercase().as_str() {
            "directx11" | "dx11" => Some(GraphicsApi::DirectX11),
            "directx12" | "dx12" => Some(GraphicsApi::DirectX12),
            "opengl" | "gl" => Some(GraphicsApi::OpenGL),
            "opengles" | "gles" => Some(GraphicsApi::OpenGLES),
            "metal" => Some(GraphicsApi::Metal),
            "vulkan" => Some(GraphicsApi::Vulkan),
            _ => None,
        }
    }

    /// Convert GraphicsApi to string
    pub fn graphics_api_to_string(api: GraphicsApi) -> &'static str {
        match api {
            GraphicsApi::DirectX11 => "DirectX11",
            GraphicsApi::DirectX12 => "DirectX12",
            GraphicsApi::OpenGL => "OpenGL",
            GraphicsApi::OpenGLES => "OpenGLES",
            GraphicsApi::Metal => "Metal",
            GraphicsApi::Vulkan => "Vulkan",
        }
    }

    /// Create optimal interop config for current platform
    pub fn create_optimal_config() -> InteropConfig {
        let mut config = InteropConfig::default();
        
        // Platform-specific optimizations
        #[cfg(target_os = "windows")]
        {
            config.enable_zero_copy = true;
            config.allow_cpu_fallback = true;
            config.enable_multi_threading = true;
        }
        
        #[cfg(target_os = "macos")]
        {
            config.enable_zero_copy = true;
            config.allow_cpu_fallback = true;
            config.enable_multi_threading = true;
        }
        
        #[cfg(target_os = "linux")]
        {
            config.enable_zero_copy = true;
            config.allow_cpu_fallback = true;
            config.enable_multi_threading = true;
        }
        
        config
    }

    /// Create optimal integration config
    pub fn create_optimal_integration_config() -> InteropIntegrationConfig {
        let mut config = InteropIntegrationConfig::default();
        
        // Platform-specific optimizations
        #[cfg(target_os = "windows")]
        {
            config.enable_zero_copy_preview = true;
            config.enable_multi_api_support = true;
            config.max_preview_resolution = (4096, 4096);
        }
        
        #[cfg(target_os = "macos")]
        {
            config.enable_zero_copy_preview = true;
            config.enable_multi_api_support = true;
            config.max_preview_resolution = (4096, 4096);
        }
        
        #[cfg(target_os = "linux")]
        {
            config.enable_zero_copy_preview = true;
            config.enable_multi_api_support = true;
            config.max_preview_resolution = (4096, 4096);
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_interop_integration_creation() {
        let config = InteropIntegrationConfig::default();
        let interop_config = InteropConfig::default();
        
        let integration = InteropIntegration::new(config, interop_config).await;
        assert!(integration.is_ok());
    }

    #[test]
    fn test_graphics_api_conversion() {
        assert_eq!(interop_utils::string_to_graphics_api("directx11"), Some(GraphicsApi::DirectX11));
        assert_eq!(interop_utils::string_to_graphics_api("vulkan"), Some(GraphicsApi::Vulkan));
        assert_eq!(interop_utils::string_to_graphics_api("invalid"), None);
        
        assert_eq!(interop_utils::graphics_api_to_string(GraphicsApi::Metal), "Metal");
        assert_eq!(interop_utils::graphics_api_to_string(GraphicsApi::OpenGL), "OpenGL");
    }

    #[test]
    fn test_optimal_config_creation() {
        let config = interop_utils::create_optimal_config();
        assert!(config.enable_zero_copy);
        assert!(config.allow_cpu_fallback);
        
        let integration_config = interop_utils::create_optimal_integration_config();
        assert!(integration_config.enable_zero_copy_preview);
        assert!(integration_config.enable_multi_api_support);
    }
}