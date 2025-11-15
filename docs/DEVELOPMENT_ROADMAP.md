# Development Roadmap

Time-bound milestones. Update at each sprint.

## Short-Term (2–4 Weeks)
- Stabilize UI startup: PostUpdate scheduling, startup gate, defensive guards.
- Add status overlay indicating initialization state and readiness.
- Implement global error surface for shader compilation and resource errors.
- Keyboard shortcuts: Open, Save, Compile, Toggle panels, Focus editor.
- Shader Browser MVP: list, search, open, favorites; recent files.
- Logging improvements: structured logs, levels, file logging toggle.
- Build releases: Windows binary with crash-safe startup.

Success Criteria:
- App opens reliably; no panics in first-run tests.
- UI renders consistently after gate; overlay reflects progress.
- Basic workflows: load shader, edit, compile, preview.

## Medium-Term (5–8 Weeks)
- Code editor diagnostics, error squiggles, inline messages.
- Templates/snippets; ISF import mapping to parameter schema.
- Parameters presets; universal controls with validation.
- Command palette with searchable actions and shortcut hints.

## Long-Term (6–12 Weeks)
- Audio/MIDI integration with UI mapping and latency tuning.
- Automation sources bound to parameters (LFO, envelope, audio/MIDI).
- Node Editor MVP: visual graph, type-safe connections, WGSL generation.
- Performance profiling: GPU timers, adaptive quality, framerate floor.
- FFGL plugin parity, ISF exporter/importer.
- Web build (WASM/WebGPU) feasibility assessment.

Risks & Mitigations:
- Bevy/egui timing issues: guard with gates and overlays; upstream tracking.
- Cross-platform differences: dedicate CI runners; manual QA matrix.