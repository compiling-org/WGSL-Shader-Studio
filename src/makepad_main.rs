use makepad_widgets::*;
use makepad_draw;
use makepad_code_editor::{
    code_editor::{CodeEditor, CodeEditorAction, KeepCursorInView},
    decoration::DecorationSet,
    document::CodeDocument,
    session::CodeSession,
};
use crate::shader_renderer::{ShaderRenderer, RenderParameters};
use crate::isf_loader;
use crate::isf_converter;
use std::sync::{Arc, Mutex};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use pollster;
use makepad_widgets::ArcStringMut;
use std::fs;
use std::path::Path;
use makepad_platform::event::MouseButton; // Import MouseButton from event module

/// Global app state shared between DSL and Rust code
pub struct AppState {
    pub renderer: Option<ShaderRenderer>,
    pub shader_code: String,
    pub time: f32,
    pub param_a: f32,
    pub param_b: f32,
    pub last_frame: Option<Vec<u8>>,
    pub tex_width: u32,
    pub tex_height: u32,
    pub render_requested: bool,
    pub compilation_error: Option<String>,
    pub status_message: String,
    pub available_shaders: Vec<String>,
    pub last_shader_scan: std::time::Instant,
    pub shader_labels: Vec<Label>,
    pub current_shader_path: Option<String>,
    pub shader_parameters: Vec<ShaderParameter>,
    pub parameter_states: HashMap<String, f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderParamType {
    Float,
    Bool,
    Color,
    Point2D,
}

#[derive(Debug, Clone)]
pub struct ShaderParameter {
    pub name: String,
    pub input_type: ShaderParamType,
    pub value: ShaderValue,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub default: Option<f32>,
}

impl Default for ShaderParameter {
    fn default() -> Self {
        Self {
            name: String::from("parameter"),
            input_type: ShaderParamType::Float,
            value: ShaderValue::Float(0.0),
            min: None,
            max: None,
            default: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShaderValue {
    Float(f32),
    Bool(bool),
    Color([f32; 4]),
    Point2D([f32; 2]),
}

impl Default for ShaderValue {
    fn default() -> Self {
        Self::Float(0.0)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            renderer: None,
            shader_code: String::from(
                "@vertex\nfn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {\n    var pos = vec2<f32>(0.0, 0.0);\n    switch vertex_index {\n        case 0u: { pos = vec2<f32>(-1.0, -1.0); }\n        case 1u: { pos = vec2<f32>( 3.0, -1.0); }\n        case 2u: { pos = vec2<f32>(-1.0,  3.0); }\n        default: { pos = vec2<f32>(0.0, 0.0); }\n    }\n    return vec4<f32>(pos, 0.0, 1.0);\n}\n\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n    return vec4<f32>(0.2, 0.2, 0.2, 1.0);\n}",
            ),
            time: 0.0,
            param_a: 0.5,
            param_b: 0.5,
            last_frame: None,
            tex_width: 512,
            tex_height: 512,
            render_requested: true,
            compilation_error: None,
            status_message: String::new(),
            available_shaders: Vec::new(),
            last_shader_scan: std::time::Instant::now(),
            shader_labels: Vec::new(),
            current_shader_path: None,
            shader_parameters: Vec::new(),
            parameter_states: HashMap::new(),
        }
    }
}

/// Scan the project's shader directories for ISF (.fs) shaders
fn scan_shader_directories() -> Vec<String> {
    let mut found_all = Vec::new();
    let paths = [
        Path::new("./assets"),
        Path::new("./shaders"),
        Path::new("./isf-shaders"),
    ];
    for path in paths.iter() {
        if path.exists() {
            collect_isf_files(path, &mut found_all);
        }
    }
    found_all.sort();
    found_all.dedup();
    found_all
}

/// Recursively collect .fs files from a directory
fn collect_isf_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_isf_files(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("fs") {
                    if let Some(s) = p.to_str() {
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
}

/// Check if a WGSL file is compatible with the renderer (has vertex and fragment stages)
#[allow(dead_code)]
fn is_wgsl_shader_compatible(src: &str) -> bool {
    src.contains("@vertex") && src.contains("@fragment")
}

/// Custom widget ref for ShaderCodeEditor with action support
#[derive(Script, ScriptHook, WidgetRef, WidgetSet, WidgetRegister)]
pub struct ShaderCodeEditor {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    editor: CodeEditor,
    #[rust]
    session: Option<CodeSession>,
    #[live]
    text: ArcStringMut,
}

impl ShaderCodeEditor {
    fn lazy_init_session(&mut self) {
        if self.session.is_none() {
            let doc = CodeDocument::new(self.text.as_ref().into(), DecorationSet::new());
            let mut session = CodeSession::new(doc);
            session.handle_changes();
            self.session = Some(session);
            self.editor.keep_cursor_in_view = KeepCursorInView::Once;
        }
    }
}

impl WidgetNode for ShaderCodeEditor {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }

    fn area(&self) -> Area {
        self.editor.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.editor.redraw(cx)
    }

    fn find_widgets_from_point(&self, cx: &Cx, point: DVec2, found: &mut dyn FnMut(&WidgetRef)) {
        self.editor.find_widgets_from_point(cx, point, found)
    }

    fn visible(&self) -> bool {
        self.editor.visible()
    }

    fn set_visible(&mut self, cx: &mut Cx, visible: bool) {
        self.editor.set_visible(cx, visible)
    }
}

impl Widget for ShaderCodeEditor {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.lazy_init_session();
        let session = self.session.as_mut().unwrap();
        self.editor.draw_walk_editor(cx, session, walk);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.lazy_init_session();
        let session = self.session.as_mut().unwrap();
        let old_text = self.text.as_ref().to_string();
        for action in self
            .editor
            .handle_event(cx, event, &mut Scope::empty(), session)
        {
            if matches!(action, CodeEditorAction::TextDidChange) {
                session.handle_changes();
            }
        }

        session.handle_changes();
        let text = session.document().as_text().to_string();
        if text != old_text {
            self.text.set(&text);
            cx.widget_action(self.uid, ShaderCodeEditorAction::TextDidChange);
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
#[repr(u32)]
pub enum ShaderCodeEditorAction {
    #[default]
    None,
    TextDidChange,
}

#[derive(Clone, Default, Debug, PartialEq)]
#[repr(u32)]
pub enum ShaderListWidgetAction {
    #[default]
    None,
    ShaderSelected(String),
}

impl ShaderListWidgetRef {
    pub fn set_items(&mut self, items: &[String]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(items);
        }
    }
    
    pub fn rescan_shaders(&mut self) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.rescan();
        }
    }
}

impl ShaderCodeEditorRef {
    pub fn set_text(&mut self, cx: &mut Cx, value: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.text.set(value);
            if let Some(session) = inner.session.as_mut() {
                let doc = CodeDocument::new(value.into(), DecorationSet::new());
                *session = CodeSession::new(doc);
                session.handle_changes();
            }
        }
    }
}

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let ShaderCodeEditor = #(ShaderCodeEditor::register_widget(vm)) {
        editor := CodeEditor {
            height: Fill
            width: Fill
            margin: 0
            pad_left_top: vec2(10.0, 10.0)
            scroll_bars: mod.widgets.ScrollBars {}
            draw_bg +: { color: #x1e1e1e }
            draw_gutter +: {
                text_style: theme.font_code
                color: theme.color_label_outer
            }
            draw_text +: {
                text_style: theme.font_code
                get_brightness: fn() {
                    return 1.1
                }
                blend_color: fn(incol: vec4) -> vec4 {
                    if self.outline < 0.5 {
                        return incol
                    }
                    if self.pos.y < 0.12 {
                        return #f
                    }
                    return incol
                }
            }
        }
    }

    let ShaderPreviewWidget = #(ShaderPreviewWidget::register_widget(vm)) {
        width: Fill
        height: Fill
    }

    let ShaderListWidget = #(ShaderListWidget::register_widget(vm)) {
        height: Fit
        width: Fill
    }

    let ShaderLibraryItem = ButtonFlat{
        width: Fill
        height: Fit
        padding: Inset{left: 8.0, right: 8.0, top: 4.0, bottom: 4.0}
        draw_text +: { color: #xffffff text_style: theme.font_regular }
        draw_bg +: { color: #x2d2d2d color_hover: #x3a3a3a color_active: #x404040 }
    }

    load_all_resources() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1600, 900)
                pass.clear_color: #x1e1e2f
                body +: {
                    flow: Down
                    dock := Dock{
                        height: Fill
                        width: Fill

                        root := DockSplitter{
                            axis: SplitterAxis.Horizontal
                            align: SplitterAlign.FromA(250.0)
                            a: @left_tabs
                            b: @center_right_split
                        }

                        left_tabs := DockTabs{
                            tabs: [@left_tab]
                            selected: 0
                            closable: false
                        }

                        left_tab := DockTab{
                            name: "Shader Library"
                            template: @PermanentTab
                            kind: @LeftPanel
                        }

                        center_right_split := DockSplitter{
                            axis: SplitterAxis.Horizontal
                            align: SplitterAlign.FromB(300.0)
                            a: @center_tabs
                            b: @right_tabs
                        }

                        center_tabs := DockTabs{
                            tabs: [@center_tab]
                            selected: 0
                            closable: false
                        }

                        center_tab := DockTab{
                            name: "Editor"
                            template: @PermanentTab
                            kind: @CenterPanel
                        }

                        right_tabs := DockTabs{
                            tabs: [@right_tab]
                            selected: 0
                            closable: false
                        }

                        right_tab := DockTab{
                            name: "Properties"
                            template: @PermanentTab
                            kind: @RightPanel
                        }

                        LeftPanel := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            padding: 10
                            label := Label{
                                text: "Shader Library"
                                draw_text +: { color: #xffffff }
                            }
                            rescan_button := Button{
                                text: "Rescan Shaders"
                                width: Fill
                                height: Fit
                            }
                            shader_list_scroll := ScrollYView{
                                width: Fill
                                height: Fill
                                padding: 0
                                scroll_bars: ScrollBars{}
                                content := View{
                                    width: Fill
                                    height: Fit
                                    flow: Down
                                    spacing: 2
                                    shader_list_widget := ShaderListWidget{
                                        width: Fill
                                        height: Fit
                                    }
                                }
                            }
                        }

                        CenterPanel := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            padding: 0

                            code_editor := ShaderCodeEditor{
                                height: 240
                                width: Fill
                                text: "@vertex\nfn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {\n    var pos = vec2<f32>(0.0, 0.0);\n    switch vertex_index {\n        case 0u: { pos = vec2<f32>(-1.0, -1.0); }\n        case 1u: { pos = vec2<f32>( 3.0, -1.0); }\n        case 2u: { pos = vec2<f32>(-1.0,  3.0); }\n        default: { pos = vec2<f32>(0.0, 0.0); }\n    }\n    return vec4<f32>(pos, 0.0, 1.0);\n}\n\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n    return vec4<f32>(0.2, 0.2, 0.2, 1.0);\n}"
                            }

                            preview_widget := ShaderPreviewWidget{
                                width: Fill
                                height: Fill
                            }
                        }

                        RightPanel := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            padding: 10
                            label := Label{
                                text: "Properties"
                                draw_text +: { color: #xffffff }
                            }

                            param_scroll := ScrollYView{
                                width: Fill
                                height: Fill
                                padding: 0
                                scroll_bars: ScrollBars{}
                                content := View{
                                    width: Fill
                                    height: Fit
                                    flow: Down
                                    spacing: 2
                                    param_section := View{
                                        width: Fill
                                        height: Fit
                                        flow: Down
                                        spacing: 4
                                        padding: 4
                                        new_batch: true
                                        draw_bg +: { color: #x2d2d2d }
                                    }
                                }
                            }

                            apply_button := Button{
                                text: "Apply to Preview"
                            }
                        }
                    }
                    status_bar := View{
                        height: Fit
                        width: Fill
                        padding: Inset{left: 10, right: 10, top: 4, bottom: 4}
                        draw_bg +: { color: #x1a1a2e }
                        flow: Right
                        spacing: 10
                        align: Align{y: 0.5}
                        status_msg := Label{
                            text: "Ready"
                            draw_text +: { color: #xffffff }
                        }
                        fps_label := Label{
                            text: "FPS: --"
                            draw_text +: { color: #x88cc88 }
                        }
                        gpu_label := Label{
                            text: "GPU: --"
                            draw_text +: { color: #x88aaff }
                        }
                        error_label := Label{
                            text: ""
                            draw_text +: { color: #xff5555 }
                        }
                    }
                }
            }
        }
    }
}

impl App {
    fn trigger_render(&mut self, cx: &mut Cx) {
        let shader_code = {
            let code_editor = self.ui.widget(cx, ids![code_editor]);
            let x = if let Some(editor) = code_editor.borrow_mut::<ShaderCodeEditor>() {
                editor.text.as_ref().to_string()
            } else {
                String::new()
            };
            x
        };
        let param_values: Vec<f32> = {
            let state = self.state.lock().unwrap();
            state.shader_parameters.iter().map(|p| {
                match &p.value {
                    ShaderValue::Float(v) => *v,
                    ShaderValue::Bool(b) => if *b { 1.0 } else { 0.0 },
                    ShaderValue::Color(c) => c[0],
                    ShaderValue::Point2D(p) => p[0],
                }
            }).collect()
        };
        {
            let mut state = self.state.lock().unwrap();
            let (tex_width, tex_height, time) = (state.tex_width, state.tex_height, state.time);
            if let Some(renderer) = state.renderer.as_mut() {
                let params = RenderParameters {
                    width: tex_width,
                    height: tex_height,
                    time,
                    frame_rate: 60.0,
                    audio_data: None,
                };
                match renderer.render_frame(&shader_code, &params, Some(&param_values)) {
                    Ok(pixels) => {
                        state.last_frame = Some(pixels.clone());
                        state.compilation_error = None;
                        state.render_requested = true;
                        state.status_message = String::from("Compiled OK");
                        drop(state);
                        self.ui.widget(cx, ids![preview_widget]).borrow_mut::<ShaderPreviewWidget>().map(|mut preview| {
                            preview.last_frame = Some(pixels);
                            preview.tex_width = tex_width;
                            preview.tex_height = tex_height;
                            preview.cached_texture = None;
                        });
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        state.compilation_error = Some(error_msg.clone());
                        state.status_message = format!("Compile failed: {}", error_msg);
                        eprintln!("Shader render error: {:?}", e);
                    }
                }
            }
        }
    }

    fn load_isf_parameters(&mut self, path: &str) -> Vec<ShaderParameter> {
        if let Ok(content) = fs::read_to_string(path) {
            if path.to_lowercase().ends_with(".fs") {
                match isf_loader::IsfShader::parse(path, &content) {
                    Ok(isf_shader) => {
                        let mut parameters = Vec::new();
                        
                        for input in isf_shader.inputs {
                            let (input_type, value, min, max, default) = match input.input_type {
                                isf_loader::InputType::Float => {
                                    let val = match input.value {
                                        isf_loader::ShaderValue::Float(v) => v,
                                        _ => 0.0,
                                    };
                                    (
                                        ShaderParamType::Float,
                                        ShaderValue::Float(val),
                                        input.min,
                                        input.max,
                                        input.default,
                                    )
                                }
                                isf_loader::InputType::Bool => {
                                    let val = match input.value {
                                        isf_loader::ShaderValue::Bool(v) => v,
                                        _ => false,
                                    };
                                    (
                                        ShaderParamType::Bool,
                                        ShaderValue::Bool(val),
                                        None,
                                        None,
                                        input.default,
                                    )
                                }
                                isf_loader::InputType::Color => {
                                    let color = match input.value {
                                        isf_loader::ShaderValue::Color(c) => c,
                                        _ => [1.0, 1.0, 1.0, 1.0],
                                    };
                                    (
                                        ShaderParamType::Color,
                                        ShaderValue::Color(color),
                                        None,
                                        None,
                                        None,
                                    )
                                }
                                isf_loader::InputType::Point2D => {
                                    let point = match input.value {
                                        isf_loader::ShaderValue::Point2D(p) => p,
                                        _ => [0.0, 0.0],
                                    };
                                    (
                                        ShaderParamType::Point2D,
                                        ShaderValue::Point2D(point),
                                        None,
                                        None,
                                        None,
                                    )
                                }
                                _ => (
                                    ShaderParamType::Float,
                                    ShaderValue::Float(0.0),
                                    None,
                                    None,
                                    None,
                                ),
                            };
                            
                            parameters.push(ShaderParameter {
                                name: input.name,
                                input_type,
                                value,
                                min,
                                max,
                                default,
                            });
                        }
                        
                        return parameters;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse ISF shader: {}", e);
                    }
                }
            }
        }
        Vec::new()
    }

    fn update_properties_panel(&self, _cx: &Cx) {
        let parameters = {
            let state = self.state.lock().unwrap();
            state.shader_parameters.clone()
        };
        
        // Store parameter values in state for the renderer
        let mut state = self.state.lock().unwrap();
        for param in &parameters {
            let val = match &param.value {
                ShaderValue::Float(v) => *v,
                ShaderValue::Bool(b) => if *b { 1.0 } else { 0.0 },
                ShaderValue::Color(c) => c[0],
                ShaderValue::Point2D(p) => p[0],
            };
            state.parameter_states.insert(param.name.clone(), val);
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    state: Arc<Mutex<AppState>>,
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Handle rescan shaders button
        if self.ui.button(cx, ids![rescan_button]).clicked(actions) {
            self.ui.widget(cx, ids![shader_list_widget]).borrow_mut::<ShaderListWidget>().map(|mut list| {
                list.rescan();
            });
        }

        // Handle shader selection from the shader list
        for action in self.ui.widget(cx, ids![shader_list_widget]).filter_actions(actions) {
            if let ShaderListWidgetAction::ShaderSelected(path) = action.cast() {
                let path_for_ext = path.clone();
                if let Ok(content) = fs::read_to_string(path) {
                    let wgsl_code = if path_for_ext.to_lowercase().ends_with(".fs") {
                        match isf_loader::IsfShader::parse(&path_for_ext, &content) {
                            Ok(isf_shader) => {
                                let mut converter = isf_converter::IsfConverter::new();
                                match converter.convert_to_wgsl(&isf_shader) {
                                    Ok(wgsl) => Some(wgsl),
                                    Err(e) => {
                                        eprintln!("ISF conversion failed: {}", e);
                                        Some(content)
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("ISF parse failed: {}", e);
                                Some(content)
                            }
                        }
                    } else {
                        Some(content)
                    };

                    if let Some(code) = wgsl_code {
                        if let Some(mut editor) = self.ui.widget(cx, ids![code_editor]).borrow_mut::<ShaderCodeEditor>() {
                            editor.set_text(cx, &code);
                        }
                    }

                    // Load ISF parameters and update Properties panel
                    let parameters = self.load_isf_parameters(&path_for_ext);
                    {
                        let mut state = self.state.lock().unwrap();
                        state.current_shader_path = Some(path_for_ext.clone());
                        state.shader_parameters = parameters;
                        state.status_message = format!("Loaded parameters for {}", path_for_ext);
                    }
                    self.update_properties_panel(cx);
                    self.trigger_render(cx);
                }
            }
        }

        // Update status bar labels
        let state = self.state.lock().unwrap();
        self.ui.widget(cx, ids![status_msg]).set_text(cx, &state.status_message);
        self.ui.widget(cx, ids![fps_label]).set_text(cx, &format!("FPS: {}", state.time as u32 % 60));
        self.ui.widget(cx, ids![gpu_label]).set_text(cx, &format!("GPU: {:?}", state.renderer.is_some()));
        self.ui.widget(cx, ids![error_label]).set_text(cx, state.compilation_error.as_deref().unwrap_or(""));
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // Trigger initial render on startup so the preview shows the default shader
        if matches!(event, Event::Startup) {
            // Initialize the WGPU renderer
            match pollster::block_on(ShaderRenderer::new_with_size((512, 512))) {
                Ok(renderer) => {
                    self.state.lock().unwrap().renderer = Some(renderer);
                }
                Err(e) => {
                    let mut state = self.state.lock().unwrap();
                    state.renderer = None;
                    state.compilation_error = Some(format!("Failed to initialize GPU renderer: {}", e));
                    state.status_message = "GPU init failed".to_string();
                }
            }
            // Scan shader directories and populate the shader list
            let shaders = scan_shader_directories();
            self.state.lock().unwrap().available_shaders = shaders;
            // Trigger render after initialization
            self.trigger_render(cx);
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
    
    fn script_mod(_vm: &mut ScriptVm) -> ScriptValue {
        // Order matters: widgets first, then our macro (which registers ShaderCodeEditor, ShaderPreviewWidget, etc.),
        // then code_editor-specific types
        makepad_widgets::script_mod(_vm);
        makepad_code_editor::script_mod(_vm);
        self::script_mod(_vm)
    }
}
#[derive(Script, ScriptHook, Widget)]
pub struct ShaderPreviewWidget {
    #[source]
    source: ScriptObjectRef,

    #[deref]
    view: View,

    #[live]
    time: f32,

    #[rust]
    tex_width: u32,

    #[rust]
    tex_height: u32,

    #[rust]
    last_frame: Option<Vec<u8>>,

    #[rust]
    cached_texture: Option<Texture>,

    #[rust]
    cached_texture_hash: u64,
}
impl Widget for ShaderPreviewWidget {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle(walk);

        if let Some(frame) = &self.last_frame {
            if frame.len() >= (self.tex_width * self.tex_height * 4) as usize {
                // Compute a hash of the frame data so we only recreate the texture when it changes
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                frame.hash(&mut hasher);
                let frame_hash = hasher.finish();

                let needs_new_texture = self.cached_texture.is_none()
                    || frame_hash != self.cached_texture_hash
                    || self.tex_width == 0
                    || self.tex_height == 0;

                if needs_new_texture {
                    if let Ok(img_buf) = makepad_draw::image_cache::ImageBuffer::new(
                        frame,
                        self.tex_width as usize,
                        self.tex_height as usize,
                    ) {
                        self.cached_texture = Some(img_buf.into_new_texture(cx));
                        self.cached_texture_hash = frame_hash;
                    }
                }

                if let Some(texture) = &self.cached_texture {
                    self.view.draw_bg.draw_vars.set_texture(0, texture);
                    self.view.draw_bg.draw_abs(cx, rect);
                }
            }
        } else {
            // No frame data — draw a dark placeholder
            self.view.draw_bg.draw_vars.empty_texture(0);
            self.view.draw_bg.draw_abs(cx, rect);
        }

        DrawStep::done()
    }
}

#[derive(Script, ScriptHook, WidgetRef, WidgetSet, WidgetRegister)]
pub struct ShaderListWidget {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    draw_text: DrawText,
    #[live]
    items: ArcStringMut,
    #[rust]
    selected_index: Option<usize>,
}

impl WidgetNode for ShaderListWidget {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }
    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }
    fn area(&self) -> Area {
        self.view.area()
    }
    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx)
    }
    fn find_widgets_from_point(&self, cx: &Cx, point: DVec2, found: &mut dyn FnMut(&WidgetRef)) {
        self.view.find_widgets_from_point(cx, point, found)
    }
    fn visible(&self) -> bool {
        self.view.visible()
    }
    fn set_visible(&mut self, cx: &mut Cx, visible: bool) {
        self.view.set_visible(cx, visible)
    }
}

impl ShaderListWidget {
    fn init_shaders(&mut self) {
        let shaders = scan_shader_directories();
        self.set_items(&shaders);
    }

    fn set_items(&mut self, items: &[String]) {
        let text = items.join("\n");
        self.items.set(&text);
    }

    fn rescan(&mut self) {
        let shaders = scan_shader_directories();
        self.set_items(&shaders);
    }

    fn get_items(&self) -> Vec<String> {
        self.items.as_ref().split('\n').map(|s| s.to_string()).collect()
    }

    fn select_and_load(&mut self, index: usize, cx: &mut Cx) {
        let items = self.get_items();
        if let Some(path) = items.get(index).cloned() {
            self.selected_index = Some(index);
            cx.widget_action(self.uid, ShaderListWidgetAction::ShaderSelected(path));
        }
    }
}

impl Widget for ShaderListWidget {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.get_items().is_empty() {
            self.init_shaders();
        }

        // Draw background
        let _ = self.view.draw_walk(cx, scope, walk);

        // Draw shader list items
        let items = self.get_items();
        if items.is_empty() {
            return DrawStep::done();
        }

        let rect = cx.walk_turtle(walk);
        let item_height = 24.0;
        let max_visible = (rect.size.y / item_height) as usize;

        for (i, name) in items.iter().enumerate().take(max_visible) {
            let y = rect.pos.y + (i as f64 * item_height);
            let item_rect = dvec2(rect.size.x, item_height);
            
            // Draw selection background
            if self.selected_index == Some(i) {
                let sel_rect = Rect { pos: dvec2(rect.pos.x, y), size: item_rect };
                let bg = self.view.draw_bg.draw_abs(cx, sel_rect);
                let _ = bg;
            }

            // Draw item text (just the filename, not full path)
            let display_name = Path::new(name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(name);

            let text = self.draw_text.draw_abs(cx, dvec2(rect.pos.x + 8.0, y + 4.0), display_name);

            let _ = text;
        }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let _ = self.view.handle_event(cx, event, scope);

        if let Event::MouseDown(mouse) = event {
            if mouse.button.is_primary() {
                let rect = self.view.area().rect(cx);
                if rect.contains(mouse.abs) {
                    let item_height = 24.0;
                    let index = ((mouse.abs.y - rect.pos.y) / item_height) as usize;
                    self.select_and_load(index, cx);
                }
            }
        }
    }
}