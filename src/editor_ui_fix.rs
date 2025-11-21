pub fn editor_central_panel(mut egui_ctx: EguiContexts, mut ui_state: ResMut<EditorUiState>) {
    let ctx = egui_ctx.ctx_mut().expect("Failed to get egui context");
    draw_editor_central_panel(ctx, &mut *ui_state);
}

/// Populate UI state's shader list by scanning common directories and Magic ISF folders.
/// This runs at Startup from the Bevy app.
pub fn populate_shader_list(mut ui_state: ResMut<EditorUiState>) {