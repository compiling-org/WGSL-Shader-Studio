use crate::audio_system::AudioData;
use crate::shader_renderer::{RenderParameters, ShaderRenderer, WorkingShaderExample};

/// Unified backend interface so different UIs (egui or bevy_egui)
/// can rely on the same rendering and conversion logic.
pub trait ShaderEngine {
    /// Render a frame from WGSL code with given parameters and optional audio.
    /// Returns RGBA8 pixel buffer laid out as `width * height * 4` bytes.
    fn render_frame(
        &mut self,
        wgsl_code: &str,
        params: &RenderParameters,
        audio: Option<AudioData>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    /// Expose working examples for quick loading.
    fn get_working_examples(&self) -> &[WorkingShaderExample];
}

impl ShaderEngine for ShaderRenderer {
    fn render_frame(
        &mut self,
        wgsl_code: &str,
        params: &RenderParameters,
        audio: Option<AudioData>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.render_frame(wgsl_code, params, audio)
    }

    fn get_working_examples(&self) -> &[WorkingShaderExample] {
        self.get_working_examples()
    }
}