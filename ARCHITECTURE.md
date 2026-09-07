# WGSL Shader Studio - Architecture & Status

## Current State (2026-09-06)

### Build Status
- ✅ `cargo check` passes (263 warnings, no errors)
- ✅ `ash` downgraded to 0.37.0 to fix Windows MSVC compilation failure
- ✅ Bevy 0.18 + bevy_egui 0.39 integration compiles

### Core Architecture
The app uses **Bevy 0.18** as the game engine with **bevy_egui 0.39** for the UI layer.

**Key integration points:**
- `src/bevy_app.rs` - App composition root, plugin registration, window/render setup
- `src/editor_ui.rs` - Top-level egui UI orchestration (1,723 lines)
- `src/ui/state.rs` - Central UI state model (`EditorUiState`)
- `src/ui/central_panel.rs` - Workspace tab switching (Preview, Node Graph, 3D Editor, Timeline)
- `src/ui/code_panel.rs` - Code editor panel
- `src/ui/side_panels.rs` - Sidebar panels (shader browser, parameters, outputs)
- `src/shader_renderer.rs` - WGPU shader rendering (standalone instance)

### Known Issues
1. **Dual WGPU Instances**: `ShaderRenderer` creates its own `wgpu::Device/Queue` while Bevy has its own. This causes GPU→CPU→GPU round-trip in `editor_ui.rs:208-222` via `map_async()`.
2. **Texture Format Mismatch**: `shader_renderer.rs` uses `Rgba8Unorm` while Bevy uses `Rgba8UnormSrgb` for preview textures.
3. **Panic Hook**: Previously suppressed Bevy 0.18+bevy_egui 0.39 validation errors. Fixed to show actual error messages.
4. **Dead Code**: Many unused functions/fields across modules (263 warnings).

### Integrated Features
- ISF→WGSL conversion (`src/isf_loader.rs`, `src/isf_converter.rs`)
- GLSL/HLSL→WGSL transpilation (`src/converter/`)
- 3D scene editor (`src/scene_editor_3d.rs`)
- Audio analysis (`src/audio_system.rs`)
- MIDI control (`src/midi_system.rs`)
- Timeline animation (`src/timeline.rs`)
- Node graph editor (`src/bevy_node_graph_integration_enhanced.rs`)
- FFGL export (`src/ffgl_plugin.rs`)
- NDI/Spout/Syphon/OSC/DMX outputs
- Gesture control (`src/gesture_control.rs`)
- Particle physics (`src/particle_physics.rs`)
- Performance overlay (`src/performance_overlay.rs`)

### Reference Repositories & Crate Integrations

The `reference_repos/` directory contains 20+ integrated reference repositories. Full crate-level analysis in `MIGRATION_TO_MAKEPAD.md`.

**Active Reference Repos:**
- **makepad** - GPU-first UI framework; replaces Bevy/Egui entirely
  - `platform/` - Window, events, audio, MIDI, network, OS abstraction
  - `draw/` - 2D GPU drawing with MPSL shader language
  - `widgets/` - Full widget library (View, Dock, CodeEditor, etc.)
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