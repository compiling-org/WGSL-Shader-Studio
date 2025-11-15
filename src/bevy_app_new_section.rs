        .add_plugins(EguiPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(UiAuditorPlugin)
        .insert_resource(UiAuditState { enabled: true, trigger_this_frame: false })
        .insert_resource(UiAuditCollector::default())
        .insert_resource(EditorUiState {
            show_shader_browser: true,
            show_parameter_panel: true,
            show_preview: true,
            show_code_editor: true,
            ..Default::default()
        })