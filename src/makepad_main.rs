use makepad_widgets::*;
use makepad_code_editor::{
    code_editor::{CodeEditor, CodeEditorAction, KeepCursorInView},
    decoration::DecorationSet,
    document::CodeDocument,
    session::CodeSession,
};
use crate::shader_renderer::{ShaderRenderer, RenderParameters};
use std::sync::{Arc, Mutex};
use pollster;
use makepad_widgets::ArcStringMut;

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
        }
    }
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

    mod.widgets.ShaderCodeEditorBase = #(ShaderCodeEditor::register_widget(vm))

    mod.widgets.ShaderCodeEditor = set_type_default() do mod.widgets.ShaderCodeEditorBase {
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
                            tabs: [@left_panel @center_panel @right_panel]
                            selected: 0
                        }

                        left_panel := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            padding: 10
                            label := Label{
                                text: "Shader Library"
                                draw_text: { color: #fff }
                            }
                            shader_list := View{
                                width: Fill
                                height: Fill
                            }
                        }

                        center_panel := View{
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

                        right_panel := View{
                            width: Fill
                            height: Fill
                            flow: Down
                            padding: 10
                            label := Label{
                                text: "Properties"
                                draw_text: { color: #fff }
                            }

                            param_section := View{
                                width: Fill
                                height: Fit
                                flow: Down
                                padding: 4
                                param_a_label := Label{
                                    text: "Parameter A"
                                    draw_text: { color: #fff }
                                }
                                param_a_slider := Slider{
                                    width: Fill
                                    value: instance(0.5)
                                }
                                param_b_label := Label{
                                    text: "Parameter B"
                                    draw_text: { color: #fff }
                                }
                                param_b_slider := Slider{
                                    width: Fill
                                    value: instance(0.5)
                                }
                            }

                            apply_button := Button{
                                text: "Apply to Preview"
                            }
                        }
                    }
                }
            }
        }
    }
}

impl App {
    fn run(vm: &mut ScriptVm) -> Self {
        makepad_widgets::script_mod(vm);
        makepad_code_editor::script_mod(vm);
        crate::makepad_main::script_mod(vm);  // Register our custom widgets
        let state = Arc::new(Mutex::new(AppState::default()));
        
        // Initialize shader renderer
        {
            let mut state = state.lock().unwrap();
            state.renderer = Some(pollster::block_on(ShaderRenderer::new()).expect("Init shader renderer"));
        }
        
        App::from_script_mod(vm, self::script_mod)
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
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
    
    fn script_mod(_vm: &mut ScriptVm) -> ScriptValue {
        // The macro expects script_mod to return a ScriptValue from the module-level function
        // Let's call the module-level script_mod directly
        script_mod(_vm)
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