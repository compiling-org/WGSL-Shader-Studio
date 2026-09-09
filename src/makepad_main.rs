use makepad_widgets::*;
use makepad_draw;
use makepad_code_editor::{
    code_editor::{CodeEditor, CodeEditorAction, KeepCursorInView},
    decoration::DecorationSet,
    document::CodeDocument,
    session::CodeSession,
};
use crate::shader_renderer::{ShaderRenderer, RenderParameters};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
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
        }
    }
}

/// Scan the project's shader directories for WGSL files
fn scan_shader_directories() -> Vec<String> {
    let mut found_all = Vec::new();
    let paths = [Path::new("./assets"), Path::new("./shaders")];
    for path in paths.iter() {
        if path.exists() {
            collect_shader_files(path, &mut found_all);
        }
    }
    found_all.sort();
    found_all.dedup();
    found_all
}

/// Recursively collect .wgsl files from a directory
fn collect_shader_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_shader_files(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("wgsl") {
                    if let Some(s) = p.to_str() {
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
}

/// Check if a WGSL file is compatible with the renderer (has vertex and fragment stages)
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
            draw_bg +: { color: #1e1e1e }
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

    let ShaderPreviewWidget = #(ShaderPreviewWidget::register_widget(vm)) {}

    let ShaderListWidget = #(ShaderListWidget::register_widget(vm)) {}

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1600, 900)
                pass.clear_color: #x1e1e2f
                body +: {
                    flow: Down
                    dock := Dock{
                        height: Fill
                        width: Fill

                        root := DockTabs{
                            tabs: [@left_tab @center_tab @right_tab]
                            selected: 1
                            closable: false
                        }

                        left_tab := DockTab{
                            name: "Shader Library"
                            template: @PermanentTab
                            kind: @LeftPanel
                        }
                        center_tab := DockTab{
                            name: "Editor"
                            template: @PermanentTab
                            kind: @CenterPanel
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
                            shader_list := ShaderListWidget{
                                width: Fill
                                height: 200
                                draw_bg +: { color: #xFF0000 }
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

                            preview := ShaderPreviewWidget{
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

                            param_section := View{
                                width: Fill
                                height: Fit
                                flow: Down
                                padding: 4
                                param_a_label := Label{
                                    text: "Parameter A"
                                    draw_text +: { color: #xffffff }
                                }
                                param_a_slider := Slider{
                                    width: Fill
                                    min: 0.0
                                    max: 1.0
                                    default: 0.5
                                }
                                param_b_label := Label{
                                    text: "Parameter B"
                                    draw_text +: { color: #xffffff }
                                }
                                param_b_slider := Slider{
                                    width: Fill
                                    min: 0.0
                                    max: 1.0
                                    default: 0.5
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
                        flow: Down
                        padding: 4
                        status_msg := Label{
                            text: "Ready"
                            draw_text +: { color: #xffffff }
                        }
                    }
                }
            }
        }
    }
}

impl App {
    fn run(vm: &mut ScriptVm) -> Self {
        // App::run is not called during normal startup; registrations happen in AppMain::script_mod
        let state = Arc::new(Mutex::new(AppState::default()));
        
        // Initialize shader renderer
        {
            let mut state = state.lock().unwrap();
            state.renderer = Some(pollster::block_on(ShaderRenderer::new()).expect("Init shader renderer"));
            // Scan for shaders on startup
            state.available_shaders = scan_shader_directories();
            // Update status message
            state.status_message = format!("Found {} shaders", state.available_shaders.len());
        }
        
        App::from_script_mod(vm, self::script_mod)
    }

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
                match renderer.render_frame(&shader_code, &params, None) {
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
        // Handle the Apply button press
        if self.ui.button(cx, ids![apply_button]).clicked(actions) {
            // Get shader code from CodeEditor
            let shader_code = {
                let code_editor = self.ui.widget(cx, ids![code_editor]);
                let x = if let Some(editor) = code_editor.borrow_mut::<ShaderCodeEditor>() {
                    editor.text.as_ref().to_string()
                } else {
                    String::new()
                };
                x
            }; // code_editor dropped here

            // Render the shader
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

                    // Render the shader frame
                    match renderer.render_frame(
                        &shader_code,
                        &params,
                        None,
                    ) {
                        Ok(pixels) => {
                            state.last_frame = Some(pixels.clone());
                            state.render_requested = true;
                            
                            // Update the ShaderPreviewWidget's last_frame field and clear cached texture
                            self.ui.widget(cx, ids![preview_widget]).borrow_mut::<ShaderPreviewWidget>().map(|mut preview| {
                                preview.last_frame = Some(pixels);
                                preview.tex_width = tex_width;
                                preview.tex_height = tex_height;
                                preview.cached_texture = None;
                            });
                        }
                        Err(e) => {
                            eprintln!("Shader render error: {:?}", e);
                        }
                    }
                }
            } // state lock dropped here
        }
        
        // Handle shader selection from the shader list
        for action in self.ui.widget(cx, ids![shader_list]).filter_actions(actions) {
            if let ShaderListWidgetAction::ShaderSelected(path) = action.cast() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Some(mut editor) = self.ui.widget(cx, ids![code_editor]).borrow_mut::<ShaderCodeEditor>() {
                        editor.set_text(cx, &content);
                    }
                }
            }
        }
        
        // Also handle slider actions to update params
        if let Some(val) = self.ui.slider(cx, ids![param_a_slider]).value() {
            let mut state = self.state.lock().unwrap();
            state.param_a = val as f32;
            
            // Update time for animation
            state.time += 0.016; // ~60 FPS
            
            // Update ShaderPreviewWidget time field
            self.ui.widget(cx, ids![preview_widget]).borrow_mut::<ShaderPreviewWidget>().map(|mut preview| {
                preview.time = state.time;
            });
        }
        if let Some(val) = self.ui.slider(cx, ids![param_b_slider]).value() {
            let mut state = self.state.lock().unwrap();
            state.param_b = val as f32;
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // Trigger initial render on startup so the preview shows the default shader
        if matches!(event, Event::Startup) {
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
}
impl Widget for ShaderPreviewWidget {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Let the base View handle its own drawing (background, etc.)
        let _ = self.view.draw_walk(cx, scope, walk);

        // Render the shader preview if we have frame data
        if let Some(frame) = &self.last_frame {
            if frame.len() >= (self.tex_width * self.tex_height * 4) as usize {
                // Get the turtle rect for our position
                let rect = cx.peek_walk_turtle(walk);

                // Only recreate texture if frame data changed
                let needs_new_texture = self.cached_texture.is_none();
                if needs_new_texture {
                    // Create texture from frame data.
                    // shader_renderer.rs outputs RGBA8Unorm pixels.
                    // ImageBuffer::new converts to internal Makepad BGRA format automatically.
                    let img_buf = makepad_draw::image_cache::ImageBuffer::new(
                        frame,
                        self.tex_width as usize,
                        self.tex_height as usize,
                    ).unwrap();

                    self.cached_texture = Some(img_buf.into_new_texture(cx));
                }

                if let Some(texture) = &self.cached_texture {
                    // Set the texture and draw using the view's draw_bg (DrawQuad)
                    self.view.draw_bg.draw_vars.set_texture(0, texture);
                    self.view.draw_bg.draw_abs(cx, rect);
                }
            }
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
    #[rust]
    hover_index: Option<usize>,
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