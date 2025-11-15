use crate::ui_auditor::{UiAuditorPlugin, UiAuditState, UiAuditCollector, PanelAudit, PanelAuditor};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::prelude::Camera2d;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::window::{PresentMode, WindowResolution};
use bevy_egui::{
    egui,
    EguiContexts,
    EguiPlugin,
    EguiTextureHandle,
    EguiPrimaryContextPass,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use resolume_isf_shaders_rust_ffgl::shader_renderer::{RenderParameters, ShaderRenderer};
use resolume_isf_shaders_rust_ffgl::audio::AudioMidiSystem;
use resolume_isf_shaders_rust_ffgl::gesture_control::GestureControlSystem;
use resolume_isf_shaders_rust_ffgl::timeline::Timeline;
use resolume_isf_shaders_rust_ffgl::editor_ui::{
    EditorUiState,
    editor_menu,
    editor_side_panels,
    populate_shader_list,
    editor_code_panel,
    apply_shader_selection,
    validate_wgsl_for_mode,
    PipelineMode,
    UiStartupGate,
};