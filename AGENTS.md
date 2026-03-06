# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Scope and source of truth
- There was no existing `AGENTS.md`, `WARP.md`, `CLAUDE.md`, Cursor rules, or Copilot instructions in this repo at the time this file was generated.
- The runtime path is centered on `src/main.rs` -> `src/bevy_app.rs` -> `src/editor_ui.rs` + `src/ui/*` + `src/shader_renderer.rs`.
- The repo contains many similarly named/legacy modules (for example `editor_ui_original.rs`, `complete_*`, `temp_*`, `working_*`). Prefer files that are actually wired into `bevy_app::run_app` and `lib.rs` exports.

## Common commands

### Build and check
- `cargo check`
- `cargo build`
- `cargo build --release`

### Run the app and tools
- `cargo run` (uses `default-run = "isf-shaders"`)
- `cargo run --bin isf-shaders`
- `cargo run --bin isf-shaders -- --cli` (forces CLI mode)
- `cargo run --bin ui-analyzer`
- `cargo run --bin ui-analyzer-enhanced`

### Tests
- `cargo test`
- `cargo test --lib`
- Run a single test by name: `cargo test <test_name>`
- Run one specific test with output: `cargo test <test_name> -- --nocapture`

### Lint/format
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`

## High-level architecture

### 1) App entry and execution modes
- `src/main.rs` decides GUI vs CLI:
  - GUI path: `run_gui()` -> `bevy_app::run_app()`
  - CLI path: `run_cli()` supports shader conversion, diagnostics, and subsystem test modes.
- `Cargo.toml` defines multiple binaries (`isf-shaders`, `ui-analyzer`, `ui-analyzer-enhanced`, etc.), with `isf-shaders` as default.

### 2) Bevy application composition
- `src/bevy_app.rs` is the main composition root:
  - Creates the Bevy `App`, window/render settings, and core plugins (`EguiPlugin`, node graph, timeline, audio/MIDI, outputs, analyzer, etc.).
  - Inserts key shared resources (`EditorUiState`, timeline/audio/renderer-related state).
  - Registers startup/update systems, especially:
    - UI orchestration (`editor_ui_system`)
    - async WGPU renderer initialization
    - shader scan background system
    - resize/time synchronization systems.

### 3) UI state + panel orchestration
- `src/ui/state.rs` is the central UI state model (`EditorUiState`) used across the app.
- `src/editor_ui.rs` orchestrates top-level egui behavior:
  - menu/status/side panels
  - shader discovery and loading
  - conversion hooks (ISF/GLSL/HLSL to WGSL paths)
  - preview compile/render path via the shared renderer.
- `src/ui/central_panel.rs` switches central workspace tabs:
  - Preview, Node Graph, 3D Editor, Timeline.

### 4) Shader rendering pipeline
- `src/shader_renderer.rs` owns low-level WGPU rendering:
  - WGPU device/queue setup
  - uniform/parameter buffers
  - render pipeline caching
  - texture readback to CPU pixels for egui preview.
- `editor_ui::compile_and_render_shader(...)` bridges UI state and renderer:
  - maps parameter values (reflection-based when available)
  - calls `ShaderRenderer::render_frame`
  - updates cached egui texture for preview display.

### 5) Node graph -> WGSL generation loop
- `src/bevy_node_graph_integration_enhanced.rs` provides:
  - graph resource/state, editor UI, interactions
  - `generate_wgsl()` for graph-to-shader code generation.
- Toolbar action **Generate WGSL** writes generated code into `EditorUiState.draft_code` and requests apply, feeding the same preview renderer path used by manually edited WGSL.

### 6) Shader ingestion and conversion layers
- Shader browsing/scanning is primarily in `editor_ui.rs`, with recursive collection from `assets`, `shaders`, and ISF locations.
- ISF parsing/loading is in `src/isf_loader.rs`; higher-level conversion path is in `src/isf_converter.rs`.
- Multi-language converter modules live in `src/converter/` and are used by menu actions and CLI paths.

## Documentation pointers from README
- Main docs entry points:
  - `docs/SHADER_STUDIO_COOKBOOK.md`
  - `docs/APPLICATION_USAGE_GUIDE_COMPLETE.md`
  - `docs/WGSL_SHADER_STUDIO_ARCHITECTURE.md`
  - `docs/COMPREHENSIVE_DOCUMENTATION_INDEX.md`
- Treat README status sections as potentially stale; verify behavior against code paths in `bevy_app.rs` and `editor_ui.rs` before making architectural assumptions.
