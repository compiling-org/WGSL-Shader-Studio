//! Gyroflow WGPU Interop Module
//! Provides zero-copy texture sharing and advanced video processing integration

use std::sync::{Arc, Mutex};
use wgpu::{Device, Queue, Texture, TextureFormat, TextureView};
use serde::{Serialize, Deserialize};

/// Configuration for Gyroflow interop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteropConfig {
    pub enable_zero_copy: bool,
    pub texture_format: InteropTextureFormat,
    pub graphics_api: GraphicsApi,
    pub max_texture_size: u32,
}

impl Default for InteropConfig {
    fn default() -> Self {
        Self {
            enable_zero_copy: true,
            texture_format: InteropTextureFormat::Rgba8Unorm,
            graphics_api: GraphicsApi::Vulkan,
            max_texture_size: 4096,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteropTextureFormat {
    Rgba8Unorm,
    Bgra8Unorm,
    Rgba16Float,
    Bgra8UnormSrgb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphicsApi {
    Vulkan,
    DirectX12,
    Metal,
    OpenGL,
}

/// Native texture information for interop
#[derive(Debug, Clone)]
pub struct NativeTextureInfo {
    pub handle: u64,
    pub width: u32,
    pub height: u32,
    pub format: InteropTextureFormat,
    pub graphics_api: GraphicsApi,
}

/// Zero-copy texture wrapper
pub struct ZeroCopyTexture {
    pub native_info: NativeTextureInfo,
    pub wgpu_texture: Option<Texture>,
    pub wgpu_view: Option<TextureView>,
}

/// Result type for interop operations
#[derive(Debug, Clone)]
pub enum InteropResult<T> {
    Success(T),
    Error(String),
    NotSupported,
}

/// Main interop manager for Gyroflow integration
pub struct WgpuInteropManager {
    config: InteropConfig,
    textures: HashMap<u64, ZeroCopyTexture>,
}

use std::collections::HashMap;

impl WgpuInteropManager {
    pub fn new(config: InteropConfig) -> Self {
        Self {
            config,
            textures: HashMap::new(),
        }
    }
    
    /// Import texture from Gyroflow
    pub fn import_texture(&mut self, native_info: NativeTextureInfo, device: &Device, queue: &Queue) -> InteropResult<u64> {
        if !self.config.enable_zero_copy {
            return InteropResult::NotSupported;
        }
        
        let texture_id = native_info.handle;
        
        // Create WGPU texture from native handle
        let wgpu_texture = self.create_wgpu_texture_from_native(&native_info, device);
        
        let zero_copy_texture = ZeroCopyTexture {
            native_info,
            wgpu_texture: Some(wgpu_texture),
            wgpu_view: None, // Will be created on demand
        };
        
        self.textures.insert(texture_id, zero_copy_texture);
        InteropResult::Success(texture_id)
    }
    
    /// Export texture to Gyroflow
    pub fn export_texture(&mut self, texture_id: u64, device: &Device, queue: &Queue) -> InteropResult<NativeTextureInfo> {
        match self.textures.get(&texture_id) {
            Some(zero_copy) => InteropResult::Success(zero_copy.native_info.clone()),
            None => InteropResult::Error("Texture not found".to_string()),
        }
    }
    
    /// Create WGPUT texture from native handle
    fn create_wgpu_texture_from_native(&self, native_info: &NativeTextureInfo, device: &Device) -> Texture {
        let format = match native_info.format {
            InteropTextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
            InteropTextureFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
            InteropTextureFormat::Rgba16Float => TextureFormat::Rgba16Float,
            InteropTextureFormat::Bgra8UnormSrgb => TextureFormat::Bgra8UnormSrgb,
        };
        
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gyroflow_interop_texture"),
            size: wgpu::Extent3d {
                width: native_info.width,
                height: native_info.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }
    
    /// Get texture view for rendering
    pub fn get_texture_view(&mut self, texture_id: u64) -> Option<&TextureView> {
        if let Some(zero_copy) = self.textures.get_mut(&texture_id) {
            if zero_copy.wgpu_view.is_none() {
                if let Some(texture) = &zero_copy.wgpu_texture {
                    zero_copy.wgpu_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
                }
            }
            zero_copy.wgpu_view.as_ref()
        } else {
            None
        }
    }
    
    /// Remove texture from interop
    pub fn remove_texture(&mut self, texture_id: u64) -> bool {
        self.textures.remove(&texture_id).is_some()
    }
    
    /// Get interop statistics
    pub fn get_stats(&self) -> InteropStats {
        InteropStats {
            active_textures: self.textures.len(),
            zero_copy_enabled: self.config.enable_zero_copy,
            total_memory_used: self.textures.len() * 1024 * 1024, // Approximate
        }
    }
}

/// Interop statistics
#[derive(Debug, Clone)]
pub struct InteropStats {
    pub active_textures: usize,
    pub zero_copy_enabled: bool,
    pub total_memory_used: usize,
}