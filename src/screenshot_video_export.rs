use bevy::prelude::*;
use bevy_egui::egui;
use std::sync::Arc;
use std::sync::Mutex;
use std::path::Path;
use std::time::{Duration, Instant};
use image::{DynamicImage, ImageBuffer, Rgba};

/// Screenshot and video export capabilities
/// Based on proven patterns from space_editor, shadplay, and wgpu-compute-toy

#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Png,
    Jpeg,
    Bmp,
    Tiff,
    WebP,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoFormat {
    Mp4,
    WebM,
    Gif,
    Apng,
}

#[derive(Debug, Clone)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub quality: u8, // 1-100 for JPEG/WebP
    pub compression: u8, // 1-9 for PNG
    pub width: u32,
    pub height: u32,
    pub premultiplied_alpha: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            quality: 95,
            compression: 6,
            width: 1920,
            height: 1080,
            premultiplied_alpha: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoExportSettings {
    pub format: VideoFormat,
    pub fps: u32,
    pub bitrate: u32, // kbps
    pub quality: u8,  // 1-100
    pub width: u32,
    pub height: u32,
    pub duration: Duration,
    pub codec: String,
}

impl Default for VideoExportSettings {
    fn default() -> Self {
        Self {
            format: VideoFormat::Mp4,
            fps: 30,
            bitrate: 5000, // 5 Mbps
            quality: 90,
            width: 1920,
            height: 1080,
            duration: Duration::from_secs(10),
            codec: "h264".to_string(),
        }
    }
}

/// Export state for managing screenshot/video capture
#[derive(Resource, Default)]
pub struct ExportState {
    pub is_recording: bool,
    pub recording_start_time: Option<Instant>,
    pub recorded_frames: Vec<Vec<u8>>,
    pub current_video_settings: VideoExportSettings,
    pub current_export_settings: ExportSettings,
    pub export_queue: Vec<ExportRequest>,
    pub processing: bool,
}

#[derive(Debug, Clone)]
pub enum ExportRequest {
    Screenshot {
        file_path: String,
        settings: ExportSettings,
        pixel_data: Vec<u8>,
    },
    Video {
        file_path: String,
        settings: VideoExportSettings,
        frames: Vec<Vec<u8>>,
    },
}

/// Screenshot and video exporter
#[derive(Resource)]
pub struct ScreenshotVideoExporter {
    export_state: Arc<Mutex<ExportState>>,
}

impl ScreenshotVideoExporter {
    pub fn new() -> Self {
        Self {
            export_state: Arc::new(Mutex::new(ExportState::default())),
        }
    }

    /// Capture screenshot from render texture
    pub fn capture_screenshot(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        file_path: &str,
        settings: ExportSettings,
    ) -> Result<(), String> {
        let size = texture.size();
        let width = settings.width.min(size.width);
        let height = settings.height.min(size.height);

        // Create buffer for reading texture data
        let buffer_size = (width * height * 4) as wgpu::BufferAddress;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screenshot Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create encoder and copy texture to buffer
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Screenshot Encoder"),
        });

        // Fix texture alignment for WGPU COPY_BYTES_PER_ROW_ALIGNMENT (256)
        let bytes_per_row = width * 4;
        let aligned_bytes_per_row = ((bytes_per_row + 255) / 256) * 256;
        
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(aligned_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        device.poll(wgpu::PollType::Wait);
        
        if rx.recv().unwrap().is_err() {
            return Err("Failed to map buffer".to_string());
        }

        let data = buffer_slice.get_mapped_range();
        let pixel_data: Vec<u8> = data.to_vec();
        drop(data);
        buffer.unmap();

        // Save image
        self.save_image(&pixel_data, file_path, settings)?;
        
        Ok(())
    }

    /// Start video recording
    pub fn start_recording(&self, settings: VideoExportSettings) -> Result<(), String> {
        let mut state = self.export_state.lock().unwrap();
        
        if state.is_recording {
            return Err("Already recording".to_string());
        }

        state.is_recording = true;
        state.recording_start_time = Some(Instant::now());
        state.recorded_frames.clear();
        state.current_video_settings = settings;
        
        Ok(())
    }

    /// Stop video recording and save
    pub fn stop_recording(&self, file_path: &str) -> Result<(), String> {
        let mut state = self.export_state.lock().unwrap();
        
        if !state.is_recording {
            return Err("Not recording".to_string());
        }

        let frames = state.recorded_frames.clone();
        let settings = state.current_video_settings.clone();
        
        state.is_recording = false;
        state.recording_start_time = None;
        state.recorded_frames.clear();

        // Add to export queue for processing
        state.export_queue.push(ExportRequest::Video {
            file_path: file_path.to_string(),
            settings,
            frames,
        });

        Ok(())
    }

    /// Capture frame during recording
    pub fn capture_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
    ) -> Result<(), String> {
        let mut state = self.export_state.lock().unwrap();
        
        if !state.is_recording {
            return Ok(());
        }

        // Check if we should capture this frame based on FPS
        if let Some(start_time) = state.recording_start_time {
            let elapsed = start_time.elapsed();
            let expected_frames = (elapsed.as_secs_f64() * state.current_video_settings.fps as f64) as usize;
            
            if state.recorded_frames.len() >= expected_frames {
                return Ok(());
            }
        }

        // Capture frame
        let size = texture.size();
        let width = state.current_video_settings.width.min(size.width);
        let height = state.current_video_settings.height.min(size.height);

        let buffer_size = (width * height * 4) as wgpu::BufferAddress;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Video Frame Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Video Frame Encoder"),
        });

        // Fix texture alignment for WGPU COPY_BYTES_PER_ROW_ALIGNMENT (256)
        let bytes_per_row = width * 4;
        let aligned_bytes_per_row = ((bytes_per_row + 255) / 256) * 256;

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(aligned_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        device.poll(wgpu::PollType::Wait);
        
        if rx.recv().unwrap().is_err() {
            return Err("Failed to map buffer".to_string());
        }

        let data = buffer_slice.get_mapped_range();
        let frame_data: Vec<u8> = data.to_vec();
        drop(data);
        buffer.unmap();

        state.recorded_frames.push(frame_data);
        
        Ok(())
    }

    /// Save image to file
    fn save_image(
        &self,
        pixel_data: &[u8],
        file_path: &str,
        settings: ExportSettings,
    ) -> Result<(), String> {
        let img_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
            settings.width,
            settings.height,
            pixel_data.to_vec(),
        ).ok_or("Failed to create image buffer")?;

        let dynamic_img = DynamicImage::ImageRgba8(img_buffer);
        
        // Apply settings
        let final_img = match settings.format {
            ExportFormat::Png => {
                dynamic_img.save_with_format(file_path, image::ImageFormat::Png)
                    .map_err(|e| format!("Failed to save PNG: {}", e))?;
            }
            ExportFormat::Jpeg => {
                let rgb_img = dynamic_img.to_rgb8();
                rgb_img.save_with_format(file_path, image::ImageFormat::Jpeg)
                    .map_err(|e| format!("Failed to save JPEG: {}", e))?;
            }
            ExportFormat::Bmp => {
                dynamic_img.save_with_format(file_path, image::ImageFormat::Bmp)
                    .map_err(|e| format!("Failed to save BMP: {}", e))?;
            }
            ExportFormat::Tiff => {
                dynamic_img.save_with_format(file_path, image::ImageFormat::Tiff)
                    .map_err(|e| format!("Failed to save TIFF: {}", e))?;
            }
            ExportFormat::WebP => {
                dynamic_img.save_with_format(file_path, image::ImageFormat::WebP)
                    .map_err(|e| format!("Failed to save WebP: {}", e))?;
            }
        };

        Ok(())
    }

    /// Process export queue
    pub fn process_export_queue(&self) -> Result<(), String> {
        let mut state = self.export_state.lock().unwrap();
        
        if state.processing || state.export_queue.is_empty() {
            return Ok(());
        }

        state.processing = true;
        let requests = std::mem::take(&mut state.export_queue);
        drop(state);

        for request in requests {
            match request {
                ExportRequest::Screenshot { file_path, settings, pixel_data } => {
                    self.save_image(&pixel_data, &file_path, settings)?;
                }
                ExportRequest::Video { file_path, settings, frames } => {
                    self.encode_video(&frames, &file_path, settings)?;
                }
            }
        }

        let mut state = self.export_state.lock().unwrap();
        state.processing = false;
        
        Ok(())
    }

    /// Encode video from frames (placeholder - would use ffmpeg or similar)
    fn encode_video(
        &self,
        frames: &[Vec<u8>],
        file_path: &str,
        settings: VideoExportSettings,
    ) -> Result<(), String> {
        // This is a placeholder implementation
        // In a real implementation, you would use ffmpeg or a similar library
        // to encode the frames into a video file
        
        println!("Encoding {} frames to video: {}", frames.len(), file_path);
        println!("Format: {:?}, FPS: {}, Quality: {}", 
                 settings.format, settings.fps, settings.quality);
        
        // For now, save frames as individual images
        for (i, frame) in frames.iter().enumerate() {
            let frame_path = format!("{}_frame_{:04}.png", file_path, i);
            let export_settings = ExportSettings {
                format: ExportFormat::Png,
                width: settings.width,
                height: settings.height,
                ..Default::default()
            };
            
            self.save_image(frame, &frame_path, export_settings)?;
        }
        
        println!("Saved {} frames as individual images", frames.len());
        Ok(())
    }

    /// Get recording status
    pub fn is_recording(&self) -> bool {
        self.export_state.lock().unwrap().is_recording
    }

    /// Get recording progress
    pub fn get_recording_progress(&self) -> f32 {
        let state = self.export_state.lock().unwrap();
        
        if !state.is_recording || state.recording_start_time.is_none() {
            return 0.0;
        }

        let elapsed = state.recording_start_time.unwrap().elapsed();
        let total_duration = state.current_video_settings.duration;
        
        (elapsed.as_secs_f32() / total_duration.as_secs_f32()).min(1.0)
    }

    /// Get recorded frame count
    pub fn get_recorded_frame_count(&self) -> usize {
        self.export_state.lock().unwrap().recorded_frames.len()
    }
}

/// UI component for export controls
pub struct ExportUI;

impl ExportUI {
    pub fn render_export_controls(
        ui: &mut egui::Ui,
        exporter: &ScreenshotVideoExporter,
        settings: &mut ExportSettings,
        video_settings: &mut VideoExportSettings,
    ) {
        ui.heading("Export Controls");
        
        ui.separator();
        
        // Screenshot section
        ui.collapsing("📸 Screenshot", |ui| {
            ui.horizontal(|ui| {
                ui.label("Format:");
                egui::ComboBox::from_id_source("screenshot_format")
                    .selected_text(format!("{:?}", settings.format))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.format, ExportFormat::Png, "PNG");
                        ui.selectable_value(&mut settings.format, ExportFormat::Jpeg, "JPEG");
                        ui.selectable_value(&mut settings.format, ExportFormat::Bmp, "BMP");
                        ui.selectable_value(&mut settings.format, ExportFormat::Tiff, "TIFF");
                        ui.selectable_value(&mut settings.format, ExportFormat::WebP, "WebP");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Width:");
                ui.add(egui::DragValue::new(&mut settings.width).speed(1.0));
                ui.label("Height:");
                ui.add(egui::DragValue::new(&mut settings.height).speed(1.0));
            });
            
            if matches!(settings.format, ExportFormat::Jpeg) {
                ui.horizontal(|ui| {
                    ui.label("Quality:");
                    ui.add(egui::Slider::new(&mut settings.quality, 1..=100).text("%"));
                });
            }
            
            if matches!(settings.format, ExportFormat::Png) {
                ui.horizontal(|ui| {
                    ui.label("Compression:");
                    ui.add(egui::Slider::new(&mut settings.compression, 1..=9));
                });
            }
            
            if ui.button("📸 Capture Screenshot").clicked() {
                // This would trigger screenshot capture
                println!("Screenshot capture requested");
            }
        });
        
        ui.separator();
        
        // Video section
        ui.collapsing("🎥 Video Recording", |ui| {
            let is_recording = exporter.is_recording();
            
            ui.horizontal(|ui| {
                if is_recording {
                    if ui.button("⏹ Stop Recording").clicked() {
                        // This would trigger stop recording
                        println!("Stop recording requested");
                    }
                } else {
                    if ui.button("⏺ Start Recording").clicked() {
                        // This would trigger start recording
                        println!("Start recording requested");
                    }
                }
            });
            
            if is_recording {
                let progress = exporter.get_recording_progress();
                let frame_count = exporter.get_recorded_frame_count();
                
                ui.horizontal(|ui| {
                    ui.label("Progress:");
                    ui.add(egui::ProgressBar::new(progress)
                        .text(format!("{:.1}%", progress * 100.0)));
                });
                
                ui.label(format!("Frames captured: {}", frame_count));
            }
            
            ui.separator();
            
            ui.collapsing("Video Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    egui::ComboBox::from_id_source("video_format")
                        .selected_text(format!("{:?}", video_settings.format))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut video_settings.format, VideoFormat::Mp4, "MP4");
                            ui.selectable_value(&mut video_settings.format, VideoFormat::WebM, "WebM");
                            ui.selectable_value(&mut video_settings.format, VideoFormat::Gif, "GIF");
                            ui.selectable_value(&mut video_settings.format, VideoFormat::Apng, "APNG");
                        });
                });
                
                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    ui.add(egui::DragValue::new(&mut video_settings.fps).speed(1.0));
                    ui.label("Bitrate (kbps):");
                    ui.add(egui::DragValue::new(&mut video_settings.bitrate).speed(100.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Quality:");
                    ui.add(egui::Slider::new(&mut video_settings.quality, 1..=100).text("%"));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(egui::DragValue::new(&mut video_settings.width).speed(1.0));
                    ui.label("Height:");
                    ui.add(egui::DragValue::new(&mut video_settings.height).speed(1.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Duration (seconds):");
                    let mut seconds = video_settings.duration.as_secs();
                    if ui.add(egui::DragValue::new(&mut seconds).speed(1.0)).changed() {
                        video_settings.duration = Duration::from_secs(seconds);
                    }
                });
            });
        });
        
        ui.separator();
        
        // Export queue
        ui.collapsing("📤 Export Queue", |ui| {
            ui.label("Export queue functionality would appear here");
        });
    }
}

/// Plugin for export functionality
pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExportState>();
    }
}

/// CLI interface for standalone export tool
pub fn run_standalone_exporter(
    input_files: Vec<String>,
    output_dir: String,
    format: String,
    fps: Option<u32>,
) {
    println!("Screenshot/Video Exporter - Standalone Tool");
    println!("==========================================");
    
    let exporter = ScreenshotVideoExporter::new();
    
    for input_file in input_files {
        println!("\nProcessing: {}", input_file);
        
        // This would load the input file and process it
        // For now, just show what would happen
        match format.as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "tiff" | "webp" => {
                let output_file = format!("{}/{}.png", output_dir, 
                    Path::new(&input_file).file_stem().unwrap().to_str().unwrap());
                println!("Would export screenshot to: {}", output_file);
            }
            "mp4" | "webm" | "gif" | "apng" => {
                let output_file = format!("{}/{}.mp4", output_dir,
                    Path::new(&input_file).file_stem().unwrap().to_str().unwrap());
                println!("Would export video to: {}", output_file);
                if let Some(fps) = fps {
                    println!("FPS: {}", fps);
                }
            }
            _ => {
                println!("Unknown format: {}", format);
            }
        }
    }
    
    println!("\nExport complete.");
}