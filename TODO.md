# TODO

- [x] Fix preview caching/interactivity in `src/editor_ui.rs`:
  - [x] Remove early `return` in `draw_preview_area` so egui stays responsive.
  - [x] Make cache reuse dependent on shader code + size + relevant parameter/node-slot changes.
  - [x] Ensure preview upload happens deterministically after `apply_requested`.


- [x] Harden quick params + preview refresh in `src/ui/central_panel.rs` (Preview tab):
  - [x] Ensure quick params also trigger preview refresh via `apply_requested` and/or parameter slot/value update.

- [x] Harden parameter sliders in `src/ui/side_panels.rs`:
  - [x] Ensure every slider interaction triggers `apply_requested = true`.
  - [x] Add best-effort mapping to node slots if parameter names don't map cleanly.

- [x] Consistency for Apply/Reset actions in `src/ui/code_panel.rs`:
  - [x] Ensure Apply always invalidates preview cache so render/upload happens next frame.

- [x] Build & run to validate preview output and interactivity.

- [x] Phase 1.1: Compilation Safety:
  - [x] Fix E0522 borrow conflicts in central_panel.rs (parameter slot mapping)
  - [x] Resolve quick_params_enabled UI interaction patterns
  - [x] Ensure parameter_values HashMap updates work correctly

- [x] Phase 1.2: Infrastructure Stability:
  - [x] Fix renderer initialization/resize handling patterns
  - [x] Implement proper surface size handling
  - [x] Add viewport texture recreation with deterministic cache invalidation
  - [x] Ensure WGPU resource lifecycle management

- [x] Phase 1.3: Parameter System Completeness:
  - [x] Implement complete node slot parameter mapping
  - [x] Add proper uniform binding validation
  - [x] Create parameter change propagation system
  - [x] Ensure bidirectional sync between UI controls, parameter values, and node slots

