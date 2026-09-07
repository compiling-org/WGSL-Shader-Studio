# Migration Assessment: Bevy/Egui → Makepad

## Executive Summary

The WGSL-Shader-Studio codebase can be migrated from Bevy/Egui to Makepad with the majority of existing logic reusable. The critical finding is that **~65% of the codebase is framework-agnostic** and requires zero changes. The remaining 35% (UI/app shell) is a complete rewrite but follows straightforward patterns documented below.

---

## 1. Reference Repositories & Crate Integrations

The `reference_repos/` directory contains all integrated reference repositories with their crate-level patterns.

### Active Reference: Makepad (`reference_repos/makepad/`)

Makepad is the target migration framework. Its crate structure:

| Makepad Crate | Purpose | Key Files |
|--------------|---------|-----------|
| `platform/` | Window, events, audio, MIDI, network, OS abstraction | `src/cx.rs`, `src/audio.rs`, `src/midi.rs` |
| `draw/` | 2D GPU drawing with MPSL shader language | `src/shader/draw_quad.rs`, `src/image_cache.rs` |
| `widgets/` | Full widget library | `src/lib.rs` (92 widgets), `src/view.rs`, `src/dock.rs` |
| `code_editor/` | Text editor with syntax highlighting | `src/code_editor.rs`, `src/tokenizer.rs` |
| `libs/render/` | 3D rendering with glTF, PBR | `src/scene.rs`, `src/material.rs` |
| `libs/midi_file/` | MIDI file parsing | `src/lib.rs` |
| `libs/audio_decode/` | MP3/Ogg Vorbis decoding | `src/lib.rs` |
| `libs/audio_encode/` | Audio encoding | `src/lib.rs` |
| `libs/video_flow/` | GPU optical flow | `src/flow.rs` |
| `libs/frametween/` | Realtime frame tweening | `src/lib.rs` |
| `libs/soundfont/` | SoundFont playback | `src/lib.rs` |
| `libs/voice/` | Whisper integration | `src/lib.rs` |

**Key Makepad Examples:**
- `examples/shader/` - Custom shader rendering with `DrawQuad` + `pixel: fn()` override
- `examples/render_to_texture/` - `CachedView` for offscreen rendering
- `examples/cad/` - `CodeEditor` integration with live script evaluation
- `apps/vj/` - Node graph + audio/video (VJ app with Makepad Flow)

### Legacy Reference Repos (archived, patterns absorbed into codebase)

| Reference Repo | Source Pattern | Absorbed Into |
|---------------|----------------|---------------|
| `use.gpu` | WGSL AST parser, shader module system, transpiler | `src/wgsl_ast_parser.rs`, `src/shader_module_system.rs`, `src/shader_transpiler.rs` |
| `bevy_shader_graph` | Node graph editor, type-safe ports, material system | `src/bevy_shader_graph_integration.rs`, `src/node_graph.rs` |
| `egui_node_graph2` | Advanced node UI, pan/zoom, multi-selection | `src/bevy_node_graph_integration_enhanced.rs` |
| `wgpu-compute-toy` | Compute pass integration | `src/compute_pass_integration.rs` |
| `qualia` | egui+wgpu standalone renderer | Pattern used in `src/shader_renderer.rs` |
| `jackdaw` | Bevy + Feathers UI patterns | Reference only |
| `sprinkles` | Bevy UI widgets | Reference only |
| `wgsl-analyzer` | WGSL analysis/inference | `src/wgsl_analyzer.rs` |
| `wgsl-bindgen` | WGSL to Rust type conversion | `src/wgsl_bindgen_integration.rs` |
| `Rust_Visual_Editor` | Visual node editing | Reference only |
| `space_editor` | 3D scene editor patterns | `src/scene_editor_3d.rs`, `src/scene_3d.rs` |

---

## 2. Current Architecture Dependency Map

### 1.1 Cargo.toml Crates — Categorized

| Crate | Version | Category | Notes |
|-------|---------|----------|-------|
| `bevy` | 0.18 | **Bevy-only** | ECS, App, Plugins, Resources, Window, Render |
| `bevy_egui` | 0.39 (opt) | **Egui-only** | egui integration via Bevy |
| `wgpu` | 27.0.1 | Cross-cutting | GPU abstraction — Makepad also uses wgpu |
| `winit` | 0.30.12 | Cross-cutting | Windowing — Bevy wraps it, Makepad wraps it |
| `naga` | 27.0.0 | Framework-agnostic | WGSL shader parsing/transpilation |
| `petgraph` | 0.6 | Framework-agnostic | Node graph data structure |
| `image` | 0.25 | Framework-agnostic | Image loading |
| `bytemuck` | 1.15 | Framework-agnostic | Memory casting |
| `serde`, `serde_json`, `serde_yaml` | — | Framework-agnostic | Serialization |
| `tokio` | 1.0 | Framework-agnostic | Async runtime |
| `midir` | 0.10.3 | Framework-agnostic | MIDI support |
| `dasp`, `ringbuf`, `rustfft` | — | Framework-agnostic | Audio processing |
| `hyper`, `hyper-staticfile` | — | Framework-agnostic | HTTP server |
| `all others` | — | Framework-agnostic | Logging, paths, compression, etc. |

### 1.2 Source File Classification

#### Fully Framework-Agnostic (Zero changes needed)
- `src/shader_renderer.rs` — Pure WGPU, no Bevy/Egui dependencies
- `src/isf_loader.rs` — serde-based JSON parsing
- `src/isf_converter.rs` — ISF→WGSL conversion
- `src/isf_auto_converter.rs` — Auto-conversion logic
- `src/isf_integration.rs`, `src/isf_integration_advanced.rs` — ISF pipeline
- `src/visual_language_parser.rs` — WGSL→node graph parsing
- `src/visual_language_manager.rs` — Language management
- `src/visual_language_compiler.rs` — Compiler logic
- `src/visual_language_bridge.rs` — Bridge layer
- `src/node_graph.rs` — Pure Rust node graph data model
- `src/wgsl_ast_parser.rs` — WGSL AST parsing
- `src/wgsl_reflect_integration.rs` — Shader reflection
- `src/wgsl_analyzer.rs`, `src/wgsl_analyzer_integration.rs` — Analysis
- `src/wgsl_diagnostics.rs` — Diagnostics
- `src/wgsl_bindgen_integration.rs` — Bindgen
- `src/wgsl_rendering_system.rs` — Rendering system
- `src/wgpu_renderer.rs`, `src/wgpu_integration.rs` — WGPU wrappers
- `src/shader_converter.rs`, `src/shader_transpiler.rs` — Language converters
- `src/converter/glsl.rs`, `src/converter/hlsl.rs` — GLSL/HLSL conversion
- `src/ffgl_plugin.rs`, `src/ffgl_plugin_architecture.rs` — FFGL
- `src/ffgl_exporter.rs` — FFGL export
- `src/utils/mod.rs`, `src/utils/color_utils.rs`, `src/utils/batch_converter.rs`, `src/utils/animation_utils.rs` — Utilities
- `src/shader_core.rs`, `src/shader_module_system.rs`, `src/shader_browser.rs` — Shader management
- `src/simple_wgpu.rs`, `src/real_shader_renderer.rs`, `src/shader_playground.rs`, `src/enhanced_shader_playground.rs` — Shader renderers
- `src/parameter_panel.rs`, `src/export_panel.rs` — Panel data models (UI logic only)
- `src/osc_control.rs`, `src/midi_system.rs` — OSC/MIDI
- `src/scene_3d.rs`, `src/scene_editor_3d.rs` — 3D scene data
- `src/audio_system.rs`, `src/enhanced_audio_system.rs` — Audio processing
- `src/spout_syphon_output.rs`, `src/ndi_output.rs` — Video output
- `src/screenshot_video_export.rs` — Video export
- `src/particle_system_gpu.rs`, `src/particle_physics.rs` — Particles
- `src/probe_variants.rs`, `src/probe_scaling.rs` — Probes
- `src/enforcement_system.rs` — System enforcement
- `src/wesl_integration.rs` — WESL
- `src/gyroflow_interop_integration.rs`, `src/gyroflow_wgpu_interop.rs`, `src/gyroflow_interop_example.rs` — Gyroflow

#### Data Model (Minor changes: remove Bevy Resource annotations)
- `src/ui/state.rs` — `EditorUiState` with `#[derive(Resource)]` → keep struct, remove derive
- `src/ui/mod.rs` — UI module declarations
- `src/preview_window.rs` — Preview window config
- `src/performance_overlay.rs` — Performance data model
- `src/midi_panel.rs`, `src/audio_panel.rs`, `src/export_panel.rs` — Panel configs
- `src/timeline.rs`, `src/timeline_animation.rs`, `src/timeline_animation_system.rs` — Timeline data
- `src/node_based_system.rs`, `src/working_shader_graph.rs` — Node system data
- `src/bevy_node_graph_integration.rs` — Node graph data (not the UI)

#### Requires Complete Rewrite (UI/App Shell)
- `src/bevy_app.rs` → Replace with Makepad platform entry point
- `src/editor_ui.rs` → Replace with Makepad widget tree + script_mod!
- `src/ui/central_panel.rs` → Makepad widgets + DrawQuad shaders
- `src/ui/code_panel.rs` → makepad-code-editor integration
- `src/ui/side_panels.rs` → Makepad panels/dock system
- `src/egui_node_graph_integration.rs` → Makepad Flow or custom node widget
- `src/new_visual_node_editor.rs`, `src/visual_node_editor.rs`, `src/visual_node_editor_plugin.rs`, `src/visual_node_editor_adapter.rs` → Makepad node graph widget
- `src/enhanced_visual_node_editor_plugin.rs`, `src/enhanced_visual_node_editor.rs` → Makepad node editor
- `src/bevy_node_graph_integration_enhanced.rs` → Makepad node graph
- `src/code_editor.rs` → makepad-code-editor
- `src/bevy_shader_graph_integration.rs` → Makepad equivalent
- `src/complete_integration.rs` → Makepad integration
- `src/backend_systems.rs` → Makepad platform services
- `src/complete_shader_studio.rs` → Makepad app
- `src/preview_window.rs` → Makepad window + shader preview
- `src/parameter_panel.rs` → Makepad UI controls
- `src/export_panel.rs` → Makepad UI controls
- `src/performance_overlay.rs` → Makepad overlay widget
- `src/minimal_3d_preview.rs` → Makepad 3D viewport
- `src/scene_editor_3d.rs` → Makepad 3D scene editor
- `src/gesture_control.rs`, `src/gesture_control_system.rs`, etc. → Makepad input handling
- `src/touch_gesture.rs` → Makepad gesture system
- `src/midi_panel.rs` → Makepad MIDI UI
- `src/osc_control.rs` → Makepad OSC UI
- `src/isf_support.rs` → Makepad ISF UI
- `src/isf_converter_simple.rs` → Keep (framework-agnostic)
- `src/temp_editor_ui_fix.rs`, `src/editor_ui_original.rs`, `src/ui_auditor*.rs`, `src/ui_analyzer*.rs`, `src/simple_ui_auditor.rs` → Archive/discard

---

## 2. Makepad Architecture Overview

### 2.1 Core Components

| Component | Crate | Purpose |
|-----------|-------|---------|
| **Platform** | `makepad-platform` | Window, events, audio, MIDI, network, OS abstraction |
| **Draw** | `makepad-draw` | 2D GPU drawing with custom shader language (MPSL) |
| **Widgets** | `makepad-widgets` | UI widget library (View, Button, Label, Dock, etc.) |
| **Code Editor** | `makepad-code-editor` | Text editor with syntax highlighting, LSP support |
| **Live System** | `makepad-live-*` | Hot-reloadable UI DSL with script evaluation |
| **Render** | `makepad-render` | 3D rendering with glTF, PBR materials |
| **Math** | `makepad-math` | Math primitives |

### 2.2 Shader Rendering System

Makepad uses **MPSL (Makepad Shader Language)** — a custom shader language that compiles to:
- Metal (macOS/iOS)
- DirectX (Windows)
- OpenGL/Vulkan (Linux)
- WebGL (Web)

Key shader primitives:
- `DrawQuad` — Base 2D quad shader with `pixel: fn()` override
- `DrawColor` — Colored quad (extends DrawQuad)
- `DrawPbr` — Physically-based rendering with textures
- Custom shaders extend `DrawQuad` via `#[deref] draw_super: DrawQuad`

MPSL syntax for custom shaders:
```rust
script_mod! {
    use mod.pod.*
    use mod.math.*
    use mod.shader.*
    use mod.draw

    set_type_default() do #(DrawFullscreenShader::script_shader(vm)){
        ..mod.draw.DrawQuad
        time: 0.0

        pixel: fn() {
            let uv = self.pos * 2.0 - vec2(1.0, 1.0)
            let t = self.time * 1.35
            // ... shader logic ...
            return vec4(color, 1.0)
        }
    }
}
```

### 2.3 Widget System

Makepad uses `script_mod!` macro for declarative UI:

```rust
script_mod! {
    use mod.prelude.widgets.*

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1280, 720)
                body +: {
                    // UI content here
                    shader_preview := FullscreenShader{}
                    code_editor := CodeEditor{}
                    sidebar := View{ width: 300 height: Fill }
                }
            }
        }
    }
}
```

Key widgets: `View`, `SolidView`, `ScrollXYView`, `Dock`, `DockTabs`, `CodeEditor`, `Window`, `Root`, `Slider`, `Button`, `Label`, `FoldButton`, `Splitter`, `CachedView`.

### 2.4 Texture/Render-to-Texture Support

- `CachedView` — Renders children into an offscreen texture and composites it back
- `Image` widget — Displays textures in the UI
- `Texture` struct — GPU texture with BGRA format
- `ImageBuffer` — CPU-side image buffer that uploads to GPU

For shader preview: Use a `CachedView` with a custom `DrawQuad` widget that renders the WGSL shader output. The shader preview can be a full-screen `DrawQuad` with the MPSL `pixel` function replaced with the WGSL shader logic.

### 2.5 Node Graph Support

Makepad has **Makepad Flow** — an interactive node graph system:
- `libs/flow` — Node graph infrastructure
- `apps/vj` — VJ app shows node-based video editing (audio/video nodes)
- `libs/render` — 3D rendering pipeline with node graph support

The existing `node_graph.rs` data model maps directly to Makepad Flow's node graph structures.

### 2.6 Audio/MIDI Support

Makepad has built-in:
- `libs/audio_decode` — MP3/Ogg Vorbis decoding
- `libs/audio_encode` — Audio encoding
- `libs/audio_sidechannels` — Stem separation
- `libs/soundfont` — SoundFont playback
- `libs/midi_file` — MIDI file parsing
- `platform/src/midi.rs` — MIDI device support
- `platform/src/audio.rs` — Audio playback
- `libs/voice` — Whisper integration

The existing `midi_system.rs`, `audio_system.rs`, `rustfft` usage maps to Makepad's audio stack.

---

## 3. Migration Strategy — Phase by Phase

### Phase 1: Replace App Entry and Platform

**Goal:** Replace `src/main.rs` + `src/bevy_app.rs` with Makepad platform entry point.

**Actions:**
1. Create new `src/main.rs` using `app_main!(App)` macro
2. Replace Bevy App creation with Makepad platform initialization
3. Set up `makepad-widgets` dependency path
4. Create `Cargo.toml` with Makepad dependencies pointing to `reference_repos/makepad`
5. Preserve CLI mode (`--cli` flag) using Makepad's platform args
6. Remove all Bevy dependencies from Cargo.toml

**New Cargo.toml dependencies:**
```toml
[dependencies]
makepad-widgets = { path = "../reference_repos/makepad/widgets" }
makepad-code-editor = { path = "../reference_repos/makepad/code_editor" }
makepad-platform = { path = "../reference_repos/makepad/platform" }
makepad-draw = { path = "../reference_repos/makepad/draw" }
# Framework-agnostic crates remain
wgpu = "27.0.1"
naga = { version = "27.0.0", features = ["wgsl-in"] }
petgraph = "0.6"
serde = { version = "1.0", features = ["derive"] }
# ... all existing framework-agnostic crates
```

### Phase 2: Reuse Framework-Agnostic Modules

**Goal:** Copy/link all framework-agnostic modules directly.

**Actions:**
1. Copy ALL files from the "Fully Framework-Agnostic" list above
2. Remove Bevy imports from `shader_renderer.rs` (it has none — already clean)
3. Remove `#[derive(Resource)]` from `src/ui/state.rs` — keep the struct fields
4. Adapt `ui/state.rs` to use Makepad's state management (`RefCell`, `Rc`, or Makepad's `Live` system)
5. Wire up ISF loading, WGSL parsing, shader conversion modules unchanged
6. The `shader_renderer.rs` WGPU pipeline works unchanged — Makepad uses wgpu internally on Linux

**Critical:** `shader_renderer.rs` uses `wgpu` directly. Makepad also uses wgpu (via `libs/vulkan/naga`, `libs/vulkan/ash` on Linux). The renderer can be reused as-is. The only change needed is integrating its output into Makepad's draw system.

### Phase 3: Rebuild UI Shell with Makepad

**Goal:** Replace all egui-based UI with Makepad widgets.

**Layout mapping:**
| Bevy/Egui Component | Makepad Replacement |
|---------------------|---------------------|
| Central Panel (tabs) | `Dock` with `DockTabs` |
| Code Editor | `CodeEditor` widget |
| Side Panels | `Dock` sidebar or `FoldButton` + `ExpandablePanel` |
| Parameter Sliders | `Slider`, `DropDown` |
| Node Graph Editor | Makepad Flow or custom `View` with `DrawQuad` nodes |
| Shader Preview | `CachedView` + custom `DrawQuad` with shader |
| Status Bar | `View` with `Hr` + `Label` |
| Menu Bar | `Window` menu or `Dock` top bar |
| Timeline | Custom `View` with `DrawQuad` timeline rendering |
| 3D Preview | `makepad-render` + `View` |
| File Browser | `FileTree` widget |
| Export Panel | Form widgets |

**Shader Preview Integration:**
The key challenge is getting the WGPU shader output into a Makepad widget. Two approaches:

**Approach A — Pure Makepad Shaders:**
Convert WGSL shaders to MPSL and define them as `DrawQuad` extensions. This requires a WGSL→MPSL transpiler (the existing `shader_transpiler.rs` can be adapted).

**Approach B — WGPU Texture → Makepad Texture:**
1. Keep `shader_renderer.rs` running its own wgpu instance
2. Read back pixels via `TextureView::read_texture`
3. Upload to Makepad's `Texture` via `ImageBuffer::into_new_texture(cx)`
4. Display in an `Image` widget or custom `DrawQuad`

Approach B is faster to implement. Approach A gives better performance and is the long-term goal.

**Node Graph Editor:**
The existing `node_graph.rs` data model is directly usable. The egui-based editor UI (`bevy_node_graph_integration_enhanced.rs`) must be rewritten using:
- Makepad `View` widgets for nodes
- Custom `DrawQuad` shaders for node rendering (rounded rectangles, connections)
- Mouse event handling via Makepad's `handle_event`
- Makepad Flow if the node graph system proves sufficient

### Phase 4: Integrate Audio/MIDI/OSC

**Goal:** Wire up existing audio/MIDI/OSC code to Makepad's platform services.

**Actions:**
1. Replace `midir` usage with Makepad's `platform/src/midi.rs`
2. Replace `dasp`/`ringbuf` audio pipeline with Makepad's audio system
3. Wire `rustfft` into Makepad's audio processing
4. Connect OSC control to Makepad's network layer
5. The existing `audio_system.rs` logic is reusable — only the platform bindings change

### Phase 5: Testing and Polish

**Goal:** Verify all features work, fix edge cases, optimize performance.

**Actions:**
1. Test shader preview rendering (Approach B first, then migrate to Approach A)
2. Test ISF loading and conversion
3. Test WGSL→node graph parsing
4. Test node graph editor interactions
5. Test audio/MIDI input
6. Test 3D preview
7. Test file browser/shader scanning
8. Test timeline animation
9. Test export functionality
10. Performance profiling

---

## 4. Specific Code Changes Required

### 4.1 `src/ui/state.rs` — Remove Bevy Resource Annotations

```rust
// BEFORE (Bevy):
#[derive(Resource, Default)]
pub struct EditorUiState {
    pub selected_shader: Option<PathBuf>,
    pub parameter_values: HashMap<String, f32>,
    // ...
}

// AFTER (Makepad):
#[derive(Default)]
pub struct EditorUiState {
    pub selected_shader: Option<PathBuf>,
    pub parameter_values: HashMap<String, f32>,
    // ...
}
// State management via Makepad's RefCell<EditorUiState> or Live system
```

### 4.2 `src/shader_renderer.rs` — No Changes Needed

Already pure WGPU. The only integration point is reading pixels back and uploading to Makepad's texture system.

### 4.3 `src/isf_loader.rs`, `src/isf_converter.rs` — No Changes Needed

Already pure Rust with serde. Directly usable.

### 4.4 `src/node_graph.rs` — No Changes Needed

Pure Rust data model. Directly usable.

### 4.5 `src/visual_language_parser.rs` — No Changes Needed

Pure Rust parsing. Directly usable.

### 4.6 `src/bevy_app.rs` → `src/makepad_app.rs` (new file)

Replace Bevy App composition with Makepad platform initialization. The entire `bevy_app.rs` (600+ lines) is replaced by ~100 lines of Makepad setup.

### 4.7 `src/editor_ui.rs` → `src/ui/makepad_ui.rs` (new file)

Replace egui immediate-mode UI with Makepad `script_mod!` widget tree. The `editor_ui.rs` (~1000+ lines) is replaced by a `script_mod!` block (~200-300 lines) plus Rust widget implementations for complex panels.

---

## 5. Makepad Features That Directly Benefit the Shader Studio

### 5.1 Live Reload DSL
Makepad's `script_mod!` macro enables hot-reloading UI definitions. When the UI DSL changes, the UI updates instantly without recompilation. This is a **massive improvement** over Bevy's compile-rerun cycle for UI iteration.

### 5.2 Built-in Code Editor
`makepad-code-editor` provides syntax highlighting, LSP support, and code folding — exactly what's needed for the WGSL shader editor panel. The existing `code_editor.rs` can be replaced entirely.

### 5.3 Custom Shader Rendering
Makepad's MPSL shader system is designed for exactly this use case — real-time shader preview with GPU-accelerated rendering. The `DrawQuad` + `pixel: fn()` pattern maps directly to the shader preview concept.

### 5.4 Node Graph System
Makepad Flow provides a built-in node graph system with connections, ports, and visual editing — directly applicable to the visual node editor.

### 5.5 Cross-Platform
Makepad targets macOS, Windows, Linux, WebAssembly, iOS, Android from a single codebase.

### 5.6 Performance
Makepad's GPU-first rendering eliminates the overhead of Bevy's ECS for UI rendering. The entire UI is rendered as GPU draw calls directly, avoiding the egui → wgpu conversion bottleneck that caused the `epaint-0.33.3` assertion failure.

---

## 6. Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Makepad shader language (MPSL) vs WGSL mismatch | Medium | Approach B (WGPU texture → Makepad texture) works immediately; Approach A (MPSL conversion) is the long-term goal |
| Makepad API maturity | Low | Makepad 1.0 released, VJ app is production-quality |
| Learning curve for Live DSL | Medium | CLAUDE.md provides comprehensive syntax reference; `script_mod!` is more intuitive than `live_design!` |
| WGPU integration in Makepad | Medium | Makepad uses wgpu on Linux via ash+naga; can reuse `shader_renderer.rs` directly |
| Audio/MIDI platform-specific issues | Low | Makepad has built-in platform-specific audio/MIDI support |
| Missing widget for specific UI element | Low | Makepad has 90+ widgets; custom widgets are straightforward via `Widget` trait |
| Texture readback performance | Low | `CachedView` handles this; GPU-direct path avoids CPU readback |
| Window/Platform issues on Windows | Low | Makepad has full Windows/D3D11 support |

---

## 7. File Inventory — What to Keep, What to Discard

### Keep (copy unchanged):
- `src/shader_renderer.rs`
- `src/isf_loader.rs`, `src/isf_converter.rs`, `src/isf_auto_converter.rs`, `src/isf_integration*.rs`
- `src/visual_language_*.rs`
- `src/node_graph.rs`
- `src/wgsl_*.rs`
- `src/converter/*.rs`
- `src/shader_transpiler.rs`, `src/shader_converter.rs`
- `src/ffgl_plugin.rs`, `src/ffgl_plugin_architecture.rs`, `src/ffgl_exporter.rs`
- `src/utils/*.rs`
- `src/parameter_panel.rs`, `src/export_panel.rs`
- `src/osc_control.rs`, `src/midi_system.rs`
- `src/scene_3d.rs`, `src/scene_editor_3d.rs`
- `src/audio_system.rs`, `src/enhanced_audio_system.rs`
- `src/spout_syphon_output.rs`, `src/ndi_output.rs`
- `src/screenshot_video_export.rs`
- `src/particle_*.rs`
- `src/probe_*.rs`
- `src/enforcement_system.rs`
- `src/wesl_integration.rs`
- `src/gyroflow_*.rs`
- `tests/*.rs`

### Discard (archive to `archive/` directory):
- `src/bevy_app.rs`
- `src/editor_ui.rs`
- `src/ui/*.rs` (all panel files)
- `src/bevy_node_graph_integration*.rs`
- `src/egui_node_graph_integration.rs`
- `src/new_visual_node_editor.rs`
- `src/visual_node_editor*.rs`
- `src/enhanced_visual_node_editor*.rs`
- `src/code_editor.rs`
- `src/bevy_shader_graph_integration.rs`
- `src/complete_integration.rs`
- `src/complete_shader_studio.rs`
- `src/backend_systems.rs`
- `src/temp_editor_ui_fix.rs`
- `src/editor_ui_original.rs`
- `src/ui_auditor*.rs`
- `src/ui_analyzer*.rs`
- `src/simple_ui_auditor.rs`
- `src/timeline_animation_system.rs`
- `src/gesture_control*.rs`
- All `src/bin/*.rs` executables (rewrite for Makepad)

### Adapt (rewrite with Makepad):
- `src/main.rs`
- `src/ui/state.rs` (remove Bevy Resource derive)
- `src/bevy_node_graph_integration.rs` (keep data, rewrite UI)
- `src/timeline.rs`, `src/timeline_animation.rs` (keep data, rewrite UI)
- `src/modular_plugin_architecture.rs` (if still referenced)

---

## 9. References

- Makepad Book: https://makepad.rs/
- Makepad GitHub: https://github.com/makepad/makepad
- Makepad CLAUDE.md: `reference_repos/makepad/CLAUDE.md`
- Makepad AGENTS.md: `reference_repos/makepad/AGENTS.md`
- Makepad shader example: `reference_repos/makepad/examples/shader/src/main.rs`
- Makepad render-to-texture example: `reference_repos/makepad/examples/render_to_texture/src/main.rs`
- Makepad CAD example (code editor integration): `reference_repos/makepad/examples/cad/src/main.rs`
- Makepad VJ app (node graph + audio/video): `reference_repos/makepad/apps/vj/src/`
- Makepad platform architecture: `reference_repos/makepad/platform/src/`
- Makepad widget library: `reference_repos/makepad/widgets/src/`
- Makepad draw/shader system: `reference_repos/makepad/draw/src/shader/`

---

## Appendix: Makepad Syntax Quick Reference

### App Entry
```rust
use makepad_widgets::*;
app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{ window.inner_size: vec2(1280, 720) }
        }
    }
}
```

### Custom Draw Shader
```rust
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMyShader {
    #[deref] draw_super: DrawQuad,
    #[live] time: f32,
}

// In script_mod!:
set_type_default() do #(DrawMyShader::script_shader(vm)){
    ..mod.draw.DrawQuad
    pixel: fn() {
        // MPSL shader code
        return vec4(1.0, 0.0, 0.0, 1.0);
    }
}
```

### Widget Pattern
```rust
#[derive(Script, ScriptHook, Widget)]
pub struct MyWidget {
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[redraw] #[live] draw_bg: DrawMyShader,
}

impl Widget for MyWidget {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        let rect = cx.turtle().rect();
        self.draw_bg.draw_abs(cx, rect);
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}
```

### Texture Upload
```rust
let image_buffer = ImageBuffer::new(pixels, width, height)?;
let texture = image_buffer.into_new_texture(cx);
```

### NextFrame Event (for animation)
```rust
if let Event::NextFrame(ne) = event {
    self.draw_bg.time = ne.time as f32;
    self.area.redraw(cx);
}
```

### Dock Layout
```rust
ui: Root{
    dock := Dock{
        left_panel := View{ width: 300 }
        center := View{ width: Fill }
        right_panel := View{ width: 250 }
    }
}
```
