# Features Status & Acceptance

Track feature status, owner, and acceptance criteria.

## Status Legend
- Planned, In Progress, Complete, Blocked.

## Core UI Panels
| Panel | Status | Owner | Acceptance Criteria |
|-------|--------|-------|---------------------|
| Menu Bar | ✅ Complete | SOLO | Commands work, shortcuts documented, theme switcher present |
| Shader Browser | ✅ Complete | SOLO | Lists/searches, opens, favorites, recent files, ISF loader with 71 shaders |
| Code Editor | ✅ Complete | SOLO | Syntax highlighting, live diagnostics, compile/run |
| Live Preview | ✅ Complete | SOLO | Stable render, controls, pause/step, perf overlay, WGPU integration |
| Parameters Panel | ✅ Complete | SOLO | Bind/unbind uniforms, presets, MIDI learn button, ISF parameter mapping |

## Engines & Systems
| System | Status | Owner | Acceptance Criteria |
|--------|--------|-------|---------------------|
| Renderer | ✅ Complete | SOLO | Stable startup, reload handles errors, ≥ 60 FPS, WGPU renderer |
| Audio FFT | ✅ Complete | SOLO | FFT, beat detection, UI mapping, < 50 ms latency |
| MIDI Controller | 📋 Planned | SOLO | Low-latency mapping, learn mode, device hot-plug |
| Node Editor | 📋 Planned | SOLO | 20+ nodes, WGSL generation, type-safe wires |
| Converter (ISF/GLSL/HLSL→WGSL) | ✅ Complete | SOLO | Round-trip fidelity, error list, ≤ 2 s compile |

## Integration & Export
| Feature | Status | Owner | Acceptance Criteria |
|---------|--------|-------|---------------------|
| FFGL Plugin Export | 📋 Planned | SOLO | Parity with desktop preview, Windows/macOS DLL |
| ISF Import/Export | ✅ Complete | SOLO | Full spec 1.2, validates with official tool |
| Theme System (Dark/Light/HC) | 📋 Planned | SOLO | Switch without restart, user CSS override |
| Command Palette | 📋 Planned | SOLO | Searchable actions, shortcut hints |

## Observability & Quality
| Area | Status | Owner | Acceptance Criteria |
|------|--------|-------|---------------------|
| Logging | 🚧 In Progress | SOLO | Structured logs, levels, overlay, file rotation |
| Unit Tests | 📋 Planned | SOLO | Parser round-trips, node codegen, keyframe interp |
| Performance Budget | 📋 Planned | SOLO | Startup ≤ 3 s, memory < 2 GB, export 10 s ≤ 30 s |

## Known Issues
- Bevy+egui startup timing can cause early layout panics; mitigated by `EguiPrimaryContextPass` scheduling and startup gate.
- Audio panel now functional with real-time analysis
- Placeholder panels (MIDI, Node, Timeline) visible but non-functional until Phase C implementation.

> Last updated: 2025-11-16 – Audio analysis system implemented, ISF loader with 71 shaders complete, WGPU renderer functional