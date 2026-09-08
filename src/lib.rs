#! # Resolume ISF Shaders Rust FFGL
//!
//! FFGL plugin for Resolume with ISF (Interactive Shader Format) support.
//! Professional VJ shader effects for live video performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Expose integration modules used by the UI and app
// Bevy-dependent modules (only compiled when NOT using Makepad UI)
#[cfg(not(feature = "makepad_ui"))]
pub mod audio_midi_integration;
#[cfg(not(feature = "makepad_ui"))]
pub mod audio_system;
#[cfg(not(feature = "makepad_ui"))]
pub mod bevy_app;
#[cfg(not(feature = "makepad_ui"))]
pub mod bevy_node_graph_integration_enhanced;
#[cfg(not(feature = "makepad_ui"))]
pub mod compute_pass_integration;
pub mod converter;
pub mod documentation_server;
#[cfg(not(feature = "makepad_ui"))]
pub mod editor_ui;
pub mod enforcement_system;
#[cfg(not(feature = "makepad_ui"))]
pub mod ffgl_exporter;
#[cfg(not(feature = "makepad_ui"))]
pub mod ffgl_plugin;
#[cfg(not(feature = "makepad_ui"))]
pub mod gesture_control;
#[cfg(not(feature = "makepad_ui"))]
pub mod gyroflow_interop_integration;
pub mod isf_converter;
pub mod isf_loader;
#[cfg(not(feature = "makepad_ui"))]
pub mod midi_system;
#[cfg(not(feature = "makepad_ui"))]
pub mod ndi_output;
#[cfg(not(feature = "makepad_ui"))]
pub mod osc_control;
#[cfg(not(feature = "makepad_ui"))]
pub mod particle_system_gpu;
#[cfg(not(feature = "makepad_ui"))]
pub mod performance_overlay;
#[cfg(not(feature = "makepad_ui"))]
pub mod scene_editor_3d;
#[cfg(not(feature = "makepad_ui"))]
pub mod screenshot_video_export;
pub mod shader_converter;
pub mod shader_renderer;
pub mod shader_transpiler;
#[cfg(not(feature = "makepad_ui"))]
pub mod spout_syphon_output;
pub mod utils;
#[cfg(not(feature = "makepad_ui"))]
pub mod wgsl_analyzer;
pub mod wgsl_ast_parser;

// UI panel modules - temporarily commented out due to compilation issues
// pub mod parameter_panel;
// pub mod audio_panel;
// pub mod preview_window;
// pub mod diagnostics_panel;
// pub mod midi_panel;
// pub mod code_editor;
// pub mod export_panel;
// pub mod d3_scene;

// Makepad migration (only compiled with makepad_ui feature)
#[cfg(feature = "makepad_ui")]
pub mod makepad_main;

// Module declarations - Bevy-dependent modules
#[cfg(not(feature = "makepad_ui"))]
pub mod isf_auto_converter;
#[cfg(not(feature = "makepad_ui"))]
pub mod ui;
#[cfg(not(feature = "makepad_ui"))]
pub mod wgsl_bindgen_integration;
#[cfg(not(feature = "makepad_ui"))]
pub mod wgsl_diagnostics;
// pub mod isf_conversion_tester;
#[cfg(not(feature = "makepad_ui"))]
pub mod wgsl_reflect_integration;
// pub mod wgslsmith_integration;
#[cfg(not(feature = "makepad_ui"))]
pub mod node_graph;
#[cfg(not(feature = "makepad_ui"))]
pub mod simple_ui_auditor;
#[cfg(not(feature = "makepad_ui"))]
pub mod timeline;
#[cfg(not(feature = "makepad_ui"))]
pub mod ui_analyzer;
#[cfg(not(feature = "makepad_ui"))]
pub mod ui_analyzer_enhanced;
pub mod wesl_integration;
#[cfg(not(feature = "makepad_ui"))]
pub mod wgpu_integration;

// Re-export UI analyzer types for external use
#[cfg(not(feature = "makepad_ui"))]
pub use ui_analyzer::{
    FeatureCheck, FeatureStatus, Priority, UIAnalyzer, UiStateDiagnostics, WgpuDiagnostics,
};
#[cfg(not(feature = "makepad_ui"))]
pub use ui_analyzer_enhanced::{AnalysisSummary, UIAnalyzerEnhanced};

#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod advanced_shader_compilation;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod backend_systems;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod enhanced_visual_node_editor;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod enhanced_visual_node_editor_plugin;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod gyroflow_wgpu_interop;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod new_visual_node_editor;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod particle_physics;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod shader_module_system;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod visual_language_bridge;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod visual_language_compiler;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod visual_language_integration;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod visual_language_manager;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod visual_language_parser;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod visual_node_editor;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod visual_node_editor_adapter;
#[cfg(feature = "naga_integration")]
#[cfg(not(feature = "makepad_ui"))]
pub mod visual_node_editor_plugin;

// Re-export main types for easier use
#[cfg(not(feature = "makepad_ui"))]
pub use ffgl_plugin::*;
pub use isf_loader::*;
pub use shader_converter::*;
pub use shader_renderer::*;

// Types are already defined in this module, no need to re-export