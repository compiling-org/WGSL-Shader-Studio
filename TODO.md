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
- [ ] Fix `ShaderPreviewWidget::draw_walk` to actually display the rendered texture (currently placeholder).
- [ ] Populate left panel (Shader Library) with actual shader list.
- [ ] Wire preview resize to recreate texture when needed.
- [ ] Add error display when shader compilation fails.
- [ ] Migrate audio/MIDI/OSC integration to Makepad platform.
- [ ] Migrate node graph editor to Makepad Flow.
- [ ] Add menu bar (File, Edit, Shader, View, Help).
- [ ] Add status bar with FPS and compilation status.

### Legacy Bevy/Egui TODOs (Archived)
- [x] Fix preview caching/interactivity in `src/editor_ui.rs` (gated behind `#[cfg(not(feature = "makepad_ui"))]`).
- [x] Harden quick params + preview refresh in `src/ui/central_panel.rs` (gated).
- [x] Harden parameter sliders in `src/ui/side_panels.rs` (gated).
- [x] Consistency for Apply/Reset actions in `src/ui/code_panel.rs` (gated).
- [x] Build & run to validate preview output and interactivity (gated).
- [x] Phase 1.1: Compilation Safety (gated).
- [x] Phase 1.2: Infrastructure Stability (gated).
- [x] Phase 1.3: Parameter System Completeness (gated).