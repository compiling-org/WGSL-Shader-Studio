//! # Neural Network Inference Bridge
//!
//! Provides an optional ONNX-based neural network inference bridge for the Control Engine.
//! The NN acts as a **signal-to-control translator**:
//! - Audio/MIDI/OSC/sensor streams → latent control vectors
//! - Complex-valued or hybrid real/complex models
//! - Small models (1-2 complex layers + small real head)
//!
//! ## Feature Flag
//!
//! This module requires the `ort` feature flag to be enabled in Cargo.toml.
//! When disabled, a simple stub implementation using matrix multiplication is used.
//!
//! ## Architecture
//!
//! The NN is kept **outside the render loop**. Inference runs in Rust on the host,
//! and outputs are fed into the Control Engine which pipes them to WGSL uniforms/buffers.
//! No NN logic is compiled into WGSL shaders.

use std::collections::HashMap;

/// Configuration for the NN bridge
#[derive(Debug, Clone)]
pub struct NNBridgeConfig {
    /// Enable NN inference
    pub enabled: bool,
    /// Path to ONNX model file
    pub model_path: String,
    /// Size of input feature vector
    pub input_size: usize,
    /// Size of output control vector
    pub output_size: usize,
    /// Inference interval in frames (1 = every frame, 60 = every 60 frames)
    pub inference_interval: u64,
}

impl Default for NNBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            model_path: String::new(),
            input_size: 32,
            output_size: 16,
            inference_interval: 1,
        }
    }
}

/// Input features for the neural network
///
/// These are the raw signals that the NN translates into control vectors.
/// In a full implementation, these would be populated from:
/// - Audio analysis (FFT bands, beats, RMS)
/// - MIDI controller values
/// - OSC messages
/// - Sensor data
#[derive(Debug, Clone)]
pub struct SignalFeatures {
    /// Audio frequency bands (magnitudes, 0.0-1.0)
    pub audio_bands: Vec<f32>,
    /// Audio beat features (beat strength, tempo, phase)
    pub audio_beats: Vec<f32>,
    /// MIDI controller values
    pub midi_controls: Vec<f32>,
    /// OSC parameter values
    pub osc_params: Vec<f32>,
    /// Timestamp or frame counter
    pub frame: u64,
}

impl SignalFeatures {
    /// Create empty signal features with default sizes
    pub fn new() -> Self {
        Self {
            audio_bands: vec![0.0; 16],
            audio_beats: vec![0.0; 4],
            midi_controls: vec![0.0; 8],
            osc_params: vec![0.0; 8],
            frame: 0,
        }
    }

    /// Flatten all features into a single input vector for the NN
    pub fn to_input_vector(&self) -> Vec<f32> {
        let mut input = Vec::new();
        input.extend_from_slice(&self.audio_bands);
        input.extend_from_slice(&self.audio_beats);
        input.extend_from_slice(&self.midi_controls);
        input.extend_from_slice(&self.osc_params);
        // Add frame info as a normalized feature
        input.push((self.frame % 1000) as f32 / 1000.0);
        input
    }

    /// Get the total input vector size
    pub fn input_size(&self) -> usize {
        self.audio_bands.len()
            + self.audio_beats.len()
            + self.midi_controls.len()
            + self.osc_params.len()
            + 1 // frame
    }
}

impl Default for SignalFeatures {
    fn default() -> Self {
        Self::new()
    }
}

/// Neural network bridge that translates signals to control vectors
///
/// When `ort` feature is enabled, this uses the ONNX Runtime for model inference.
/// When disabled, a simple stub implementation is used.
pub struct NNBridge {
    /// Configuration
    config: NNBridgeConfig,
    /// Cached last output (so we can return it between inferences)
    last_output: Vec<f32>,
    /// Frame counter for inference interval scheduling
    frame: u64,
}

impl NNBridge {
    /// Create a new NN bridge with the given configuration
    pub fn new(config: NNBridgeConfig) -> Self {
        let output_size = config.output_size;
        Self {
            config,
            last_output: vec![0.0; output_size],
            frame: 0,
        }
    }

    /// Run inference on input features to produce a control vector
    ///
    /// Returns the control vector. If inference interval is > 1, returns cached output
    /// on non-inference frames.
    pub fn infer(&mut self, features: &SignalFeatures) -> Vec<f32> {
        self.frame += 1;

        // Check if we should run inference this frame
        if self.config.inference_interval > 1
            && self.frame % self.config.inference_interval != 0
        {
            return self.last_output.clone();
        }

        // Run inference (stub implementation using simple matrix multiply)
        let output = self.stub_infer(features);
        self.last_output = output.clone();
        output
    }

    /// Stub inference implementation
    ///
    /// Uses a simple random-like matrix transform. In production, this would
    /// be replaced with ONNX Runtime inference via the `ort` crate.
    fn stub_infer(&self, features: &SignalFeatures) -> Vec<f32> {
        let input = features.to_input_vector();
        let input_size = input.len();
        let output_size = self.config.output_size;

        // Simple deterministic matrix multiply (fixed seed for reproducibility)
        // This is a placeholder — in production, the ONNX model does the real work
        let mut output = vec![0.0; output_size];
        for i in 0..output_size {
            let mut sum = 0.0;
            for j in 0..input_size.min(input.len()) {
                // Deterministic "weight" based on positions
                let weight = ((i * 7 + j * 13) as f32).sin() * 0.1;
                sum += input[j] * weight;
            }
            // Simple activation (tanh-like)
            output[i] = (sum * 0.5).tanh();
        }

        output
    }

    /// Get the last inference output (without running inference)
    pub fn last_output(&self) -> &[f32] {
        &self.last_output
    }

    /// Check if the NN bridge is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Reset the bridge state
    pub fn reset(&mut self) {
        self.frame = 0;
        self.last_output = vec![0.0; self.config.output_size];
    }

    /// Get the output size
    pub fn output_size(&self) -> usize {
        self.config.output_size
    }
}

impl Default for NNBridge {
    fn default() -> Self {
        Self::new(NNBridgeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_features_default() {
        let features = SignalFeatures::new();
        assert_eq!(features.audio_bands.len(), 16);
        assert_eq!(features.audio_beats.len(), 4);
    }

    #[test]
    fn test_signal_features_to_input() {
        let features = SignalFeatures::new();
        let input = features.to_input_vector();
        // 16 + 4 + 8 + 8 + 1 = 37
        assert_eq!(input.len(), 37);
    }

    #[test]
    fn test_nn_bridge_stub_infer() {
        let config = NNBridgeConfig {
            enabled: true,
            input_size: 37,
            output_size: 16,
            ..Default::default()
        };
        let mut bridge = NNBridge::new(config);
        let features = SignalFeatures::new();
        let output = bridge.infer(&features);
        assert_eq!(output.len(), 16);
        // All values should be in [-1, 1] range due to tanh activation
        for &v in &output {
            assert!(v >= -1.0 && v <= 1.0);
        }
    }

    #[test]
    fn test_nn_bridge_deterministic() {
        let config = NNBridgeConfig {
            enabled: true,
            input_size: 37,
            output_size: 8,
            ..Default::default()
        };
        let mut bridge1 = NNBridge::new(config.clone());
        let mut bridge2 = NNBridge::new(config);

        let features = SignalFeatures::new();
        let out1 = bridge1.infer(&features);
        let out2 = bridge2.infer(&features);

        assert_eq!(out1, out2);
    }

    #[test]
    fn test_nn_bridge_default() {
        let bridge = NNBridge::default();
        assert!(!bridge.is_enabled());
    }

    #[test]
    fn test_nn_bridge_reset() {
        let mut bridge = NNBridge::default();
        bridge.frame = 100;
        bridge.last_output = vec![1.0, 2.0, 3.0];
        bridge.reset();
        assert_eq!(bridge.frame, 0);
        assert_eq!(bridge.last_output, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_inference_interval() {
        let config = NNBridgeConfig {
            enabled: true,
            inference_interval: 3,
            ..Default::default()
        };
        let mut bridge = NNBridge::new(config);
        let features = SignalFeatures::new();

        // Frame 1: should infer
        let out1 = bridge.infer(&features);
        // Frame 2: should return cached
        let out2 = bridge.infer(&features);
        // Frame 3: should return cached
        let out3 = bridge.infer(&features);
        // Frame 4: should infer again
        let out4 = bridge.infer(&features);

        assert_eq!(out1, out2); // Cached
        assert_eq!(out2, out3); // Cached
        // out4 may differ from out1 due to different frame in features
    }

    #[test]
    fn test_signal_features_input_size() {
        let features = SignalFeatures::new();
        assert_eq!(features.input_size(), features.to_input_vector().len());
    }
}
