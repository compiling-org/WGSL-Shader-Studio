use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Plugin for visual node editor functionality
pub struct VisualNodeEditorPlugin;

impl Plugin for VisualNodeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualNodeEditorState>()
            .add_systems(bevy_egui::EguiPrimaryContextPass, visual_node_editor_ui);
    }
}

#[derive(Resource, Default)]
pub struct VisualNodeEditorState {
    pub auto_compile: bool,
    pub show_node_editor: bool,
}

fn visual_node_editor_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<VisualNodeEditorState>,
    mut ui_state: ResMut<crate::editor_ui::EditorUiState>,
    mut node_graph: ResMut<crate::bevy_node_graph_integration_enhanced::NodeGraphResource>,
) {
    if !state.show_node_editor {
        return;
    }

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    egui::Window::new("Visual Node Editor")
        .default_size([600.0, 400.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.toggle_value(&mut state.auto_compile, "Auto Compile");
                if ui.button("Compile").clicked() {
                    match node_graph.graph.generate_wgsl() {
                        Ok(compiled_shader) => {
                            ui_state.draft_code = compiled_shader;
                            ui_state.code_changed = true;
                        }
                        Err(err) => {
                            ui_state.compilation_error = format!("Node graph compile error: {}", err);
                        }
                    }
                }
            });

            ui.separator();

            ui.label("Use the Shader Graph Editor window to edit nodes.");
            ui.label("This panel provides compile controls and status.");
        });
}
