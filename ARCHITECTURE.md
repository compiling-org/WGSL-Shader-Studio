# WGSL Shader Studio - Architecture & Status

## Current State (2026-09-08)

### Build Status
- ✅ `cargo check --features makepad_ui` passes (warnings only)
- ✅ `cargo build --release --features makepad_ui` succeeds
- ✅ App launches with `--remote` and HTTP control surface works
- ✅ Makepad UI is the default (Bevy/Egui removed from Cargo.toml)

### Core Architecture
The app now uses **Makepad** as the UI framework with `script_mod!` DSL.

**Key integration points:**
- `src/main.rs` - Entry point with `app_main!(App)` macro
- `src/makepad_main.rs` - Makepad app entry with Dock layout, ShaderCodeEditor, ShaderPreviewWidget
- `src/shader_renderer.rs` - WGPU shader rendering (standalone, framework-agnostic)
- `src/lib.rs` - Framework-agnostic modules gated behind `#[cfg(not(feature = "makepad_ui"))]` for legacy Bevy/Egui
- `src/isf_loader.rs`, `src/isf_converter.rs` - ISF parsing/conversion
- `src/converter/` - GLSL/HLSL→WGSL transpilation
- `src/shader_transpiler.rs` - Multi-language transpiler

### Known Issues
1. **Preview Texture Display**: `ShaderPreviewWidget::draw_walk` recreates texture every frame; needs caching.
2. **Shader Library**: Left panel is a placeholder; no browsing functionality.
3. **Audio/MIDI/OSC**: Not yet migrated to Makepad platform.
4. **Node Graph**: Not yet migrated to Makepad Flow.

### Integrated Features (Makepad)
- Dock layout with 3 panels (Shader Library, CodeEditor+Preview, Properties)
- ShaderCodeEditor widget wrapping makepad-code-editor
- ShaderPreviewWidget with ImageBuffer texture upload
- Synchronous `renderer.render_frame()` from Apply button
- Parameter sliders wired to state updates
- `--remote` HTTP control surface

### Makepad Migration Notes
- Bevy and Egui dependencies removed from Cargo.toml
- All Bevy-dependent modules gated behind `#[cfg(not(feature = "makepad_ui"))]`
- `shader_renderer.rs` unchanged; framework-agnostic
- Makepad docs: `docs/WGSL_SHADER_STUDIO_ARCHITECTURE.md#ui-framework`
  - `code_editor/` - Text editor with syntax highlighting, LSP
  - `libs/render/` - 3D rendering with glTF, PBR
  - `libs/midi_file/` - MIDI file parsing
  - `libs/audio_decode/` - MP3/Ogg Vorbis decoding
  - `libs/audio_encode/` - Audio encoding
  - `libs/audio_sidechannels/` - Stem separation
  - `libs/video_flow/` - GPU optical flow
  - `libs/frametween/` - Realtime frame tweening
  - `libs/render/` - 3D renderer with texture support
  - `examples/shader/` - Custom shader rendering example
  - `examples/render_to_texture/` - Offscreen rendering example
  - `examples/cad/` - Code editor + live script evaluation integration
  - `apps/vj/` - Node graph + audio/video integration (VJ app)
  - `libs/makepad_test/` - Test framework with widget tree inspection

**Legacy Reference Repos (archived, content absorbed into codebase):**
- **use.gpu** - WGSL AST parser, shader module system, transpiler framework
- **bevy_shader_graph** - Node graph editor, type-safe port connections, material system
- **egui_node_graph2** - Advanced node UI system, pan/zoom, multi-selection
- **wgpu-compute-toy** - Compute pass integration
- **qualia** - egui+wgpu standalone renderer patterns
- **jackdaw** - Bevy 0.18 + Feathers UI editor patterns
- **sprinkles** - Bevy 0.18 UI widgets
- **wgsl-analyzer** - WGSL analysis/inference
- **wgsl-bindgen** - WGSL to Rust type conversion
- **Rust_Visual_Editor** - Visual node editing
- **space_editor** - 3D scene editor patterns (adapted in `src/scene_editor_3d.rs`)

### Crate Dependency Summary

| Crate | Category | Bevy Dep | Makepad Replacement |
|-------|----------|----------|---------------------|
| `bevy` | Framework | ✅ | `makepad-widgets` + `makepad-platform` |
| `bevy_egui` | UI | ✅ | Makepad widget system |
| `wgpu` | GPU | ❌ | Native wgpu (Makepad uses wgpu on Linux) |
| `naga` | Shader parsing | ❌ | Native naga |
| `petgraph` | Graph data | ❌ | Direct reuse |
| `serde` | Serialization | ❌ | Direct reuse |
| `midir` | MIDI | ❌ | Makepad platform MIDI |
| `dasp`/`ringbuf`/`rustfft` | Audio | ❌ | Makepad audio libs |
| `tokio` | Async | ❌ | Direct reuse |
| `hyper` | HTTP | ❌ | Makepad network |
| All others | Utilities | ❌ | Direct reuse |

### Next Steps
1. Refactor `ShaderRenderer` to use Bevy's render pipeline instead of standalone wgpu instance
2. Eliminate GPU→CPU→GPU round-trip in `editor_ui.rs:208-222`
3. Fix texture format mismatch between `shader_renderer.rs` and Bevy
4. Remove dead code warnings
5. Cross-verify documentation against actual implementation