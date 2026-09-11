# TODO

## Makepad UI Migration (Current Phase)

### Completed
- [x] Remove Bevy/Egui dependencies from Cargo.toml; Makepad UI is default.
- [x] Create `src/makepad_main.rs` with `App` struct, `script_mod!` DSL, `app_main!(App)` entry point.
- [x] Implement `ShaderCodeEditor` widget wrapping `makepad-code-editor` with `CodeSession`.
- [x] Implement `ShaderPreviewWidget` with `#[derive(Script, ScriptHook, Widget)]` and `[deref] view: View`.
- [x] Wire Apply button to `renderer.render_frame()` synchronously; store result in `state.last_frame`.
- [x] Wire sliders to `state.param_a` / `state.param_b`.
- [x] Build release binary with `cargo build --release --features makepad_ui`.
- [x] Launch app with `--remote` and verify HTTP control surface works.

### Active / Next
- [x] Fix `ShaderPreviewWidget::draw_walk` to actually display the rendered texture (currently placeholder).
- [x] Populate left panel (Shader Library) with actual shader list.
- [ ] Wire preview resize to recreate texture when needed.
- [ ] Add error display when shader compilation fails.
- [ ] Wire status bar labels (FPS, GPU, compilation status) to live data.
- [ ] Wire rescan button to refresh shader list.
- [ ] Wire shader item click to load shader into code editor.
- [ ] Migrate audio/MIDI/OSC integration to Makepad platform.
- [ ] Migrate node graph editor to Makepad Flow.
- [ ] Add menu bar (File, Edit, Shader, View, Help).
- [ ] Add toolbar with New/Open/Save/Compile/Run/Export quick actions.
- [ ] Add error console / diagnostics panel — parse WGSL errors, show line/column, clickable to editor.
- [ ] Add preview transport controls — play/pause, scrub time, resolution selector, fullscreen toggle.

### Legacy Bevy/Egui TODOs (Archived)
- [x] Fix preview caching/interactivity in `src/editor_ui.rs` (gated behind `#[cfg(not(feature = "makepad_ui"))]`).
- [x] Harden quick params + preview refresh in `src/ui/central_panel.rs` (gated).
- [x] Harden parameter sliders in `src/ui/side_panels.rs` (gated).
- [x] Consistency for Apply/Reset actions in `src/ui/code_panel.rs` (gated).
- [x] Build & run to validate preview output and interactivity (gated).
- [x] Phase 1.1: Compilation Safety (gated).
- [x] Phase 1.2: Infrastructure Stability (gated).
- [x] Phase 1.3: Parameter System Completeness (gated).

---

## Makepad UI Development Roadmap (Multi-Week)

**Scope Reality Check:** The original `editor_ui.rs` (1,723 lines) exposed a comprehensive editor: node graph, 3D scene editor, timeline, audio analysis, MIDI, OSC, export, diagnostics, performance, gesture control, and color grading. The current `src/makepad_main.rs` (~760 lines) is a preliminary skeleton (Dock + CodeEditor + Preview + 2 sliders). The Makepad UI rebuild is a multi-week effort, not a patch. All phases below must be verified against `reference_repos/makepad/` source before writing code.

### Phase 1 — Foundation & Core Panels (Week 1)
- [ ] Replace scattered `Arc<Mutex<AppState>>` with a single centralized app state (Elm-style: state is source of truth, UI is projection)
- [ ] Fix `ShaderPreviewWidget::draw_walk` texture caching — recreate `Texture` only when frame data or dimensions change; handle resize
- [ ] Build real `ShaderLibrary` panel — recursive browse, search/filter, categories, selection, rescan
- [ ] Add menu bar (File, Edit, View, Shader, Tools, Window, Help) with commands wired to state/actions
- [ ] Add toolbar with New/Open/Save/Compile/Run/Export quick actions
- [ ] Add status bar — FPS, WGPU state, compilation status, current shader path
- [ ] Add error console / diagnostics panel — parse WGSL errors, show line/column, clickable to editor
- [ ] Add preview transport controls — play/pause, scrub time, resolution selector, fullscreen toggle

### Phase 2 — Code Editor & Shader Workflow (Week 2)
- [ ] Enhance `ShaderCodeEditor` — tabs (Editor/Diagnostics/Analyzer/AI), line numbers, bracket matching, WGSL syntax highlighting
- [ ] Wire WGSL reflection (`wgsl_reflect_integration`) to auto-discover uniforms and generate parameter controls
- [ ] Add shader file operations — new/open/save/recent, project JSON import/export, auto-save
- [ ] Add shader validation pipeline — AST parse, semantic validation, Naga diagnostics surfaced in editor
- [ ] Add conversion wizard UI — WGSL↔GLSL↔HLSL↔ISF via `shader_transpiler`, batch convert
- [ ] Add ISF import with parameter auto-discovery and WGSL generation

### Phase 3 — Visual Programming & Scene (Weeks 3-4)
- [ ] Migrate node graph editor to Makepad — canvas pan/zoom, node add/drag/connect, node library, type-checked ports
- [ ] Wire node graph → WGSL codegen (`visual_language_compiler`) → preview pipeline
- [ ] Migrate 3D scene editor to Makepad — viewport, hierarchy tree, gizmo manipulation (translate/rotate/scale), camera orbit/pan/zoom
- [ ] Wire 3D viewport to `makepad-render` / render-to-texture; assign WGSL shaders to materials
- [ ] Add material/light/camera inspector panel for 3D scene

### Phase 4 — Timeline & Animation (Weeks 5-6)
- [ ] Migrate timeline editor to Makepad — ruler, multi-track view, keyframe add/move/delete, interpolation modes
- [ ] Wire timeline playback (play/pause/stop/scrub/loop) to shader `time` and parameter evaluation
- [ ] Add curve editor for keyframe interpolation
- [ ] Wire timeline → video/image-sequence export

### Phase 5 — Live Inputs & Reactive Systems (Weeks 7-8)
- [ ] Migrate audio analysis to Makepad platform — FFT bands (bass/mid/treble), beat detection, waveform/spectrum visualization
- [ ] Wire audio uniforms to shader render parameters (`AUDIOBASS`, `AUDIOMID`, `AUDIOTREBLE`, `AUDIOLEVEL`)
- [ ] Migrate MIDI to Makepad platform — device detection, note/CC/pitch-bend mapping, learn mode, curve editing, presets
- [ ] Migrate OSC to Makepad networking — send/receive config, address-pattern routing, bidirectional feedback
- [ ] Migrate gesture control panel (MediaPipe/Leap-style curve mapping)

### Phase 6 — Output, Profiling & Polish (Weeks 9-10)
- [ ] Migrate export panel — NDI, Spout/Syphon, screenshots, video (MP4/WebM), FFGL plugin generation
- [ ] Add performance profiler overlay — FPS, GPU time, memory, draw-call stats
- [ ] Add color grading panel (curves, levels, LUT) and WGSL-Smith AI generation panel
- [ ] Add layout presets, panel docking/floating, workspace save/restore
- [ ] Cross-platform validation (Windows D3D11, macOS Metal, Linux Vulkan), CI, and end-to-end test suite

### Non-Negotiables (from `AGENTS.md`)
- Verify every Makepad API/pattern against `reference_repos/makepad/` before use — no guessing.
- Launch standalone with `--remote`; finish every session with `GET /gq`. Never leave a test window open.
- Use `height: Fit` on all containers, `width: Fill` on root, `new_batch: true` for text-on-background views.
- Keep business logic in Rust; UI declarations in `script_mod!`. Framework-agnostic modules stay in `src/` and are wired into Makepad.