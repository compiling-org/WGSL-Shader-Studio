pub fn draw_editor_side_panels(
    ctx: &egui::Context, 
    ui_state: &mut EditorUiState, 
    audio_analyzer: &AudioAnalyzer, 
    gesture_control: &mut crate::gesture_control::GestureControlSystem,
    compute_pass_manager: &mut ComputePassManager,
    video_exporter: Option<&ScreenshotVideoExporter>,
    editor_state: Option<&SceneEditor3DState>,
    global_renderer: Option<&GlobalShaderRenderer>
) {
    // CRITICAL FIX: Use proper panel hierarchy - NO CentralPanel here to avoid conflicts
    
    // Draw individual panels
    draw_editor_shader_browser_panel(ctx, ui_state);
    draw_editor_parameter_panel(ctx, ui_state);

    // Additional panels as windows (simplified for now)
    if ui_state.show_node_studio {
        let mut show = ui_state.show_node_studio;
        egui::Window::new("Node Studio").open(&mut show).show(ctx, |ui| {
            ui.heading("Node-based Shader Authoring");
            ui.label("Node studio functionality will be implemented here");
        });
        ui_state.show_node_studio = show;
    }

    if ui_state.show_timeline {
        let mut show = ui_state.show_timeline;
        egui::Window::new("Timeline").open(&mut show).show(ctx, |ui| {
            ui.heading("Timeline Animation Editor");
            ui.label("Timeline controls will be implemented here");
        });
        ui_state.show_timeline = show;
    }
}