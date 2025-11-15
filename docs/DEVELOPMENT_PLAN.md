# Development Plan

Living plan organizing work into phases and sprints, with acceptance criteria and owners. Update every sprint.

## ✅ Phase 0 – Foundation Complete (Nov 2025)
- Stabilize Bevy + egui startup; eliminate timing panics. ✅ DONE
- Implement explicit UI startup gate and panel guards. ✅ DONE
- Add logging overlays and non-blocking error surfaces. ✅ DONE
- Keyboard shortcuts baseline; command palette. ✅ DONE
- Shader browser MVP with load/save and recent files. ✅ DONE

Acceptance:
- App launches reliably across target OS; no startup panics. ✅
- UI panels render after initialization with visible status. ✅
- Basic shortcuts functional; help overlay lists them. ✅

## 🚧 Phase 1 – Converter & Preview (2 wks)
- Build ISF/GLSL/HLSL → WGSL converter with live error list
- Integrate converter into code editor; auto-compile on change
- Add working preview panel with resolution selector, pause, step
- Implement fallback pattern when shader fails
- Dark theme switcher (toolbar menu)

Acceptance:
- Any dropped ISF/GLSL/HLSL compiles to WGSL ≤ 2 s
- Preview updates live; ≥ 60 FPS at 1080 p
- Error diagnostics clickable in editor
- Fallback pattern visible on compile failure

## 🚧 Phase 2 – Node Editor MVP (3 wks)
- Visual graph canvas: pan/zoom, drag wires, delete, undo/redo
- Node library panel with 20+ node types (Math, Texture, Time, Audio, MIDI, Geometry, UV, Color, Blur, Noise)
- Type-safe pins with auto-cast; red badge on mismatch
- Real-time WGSL codegen with comments; highlight node ↔ code link
- JSON save/load graphs; mini-preview on each node

Acceptance:
- User builds 20-node graph, sees WGSL update < 500 ms
- Graph runs ≥ 45 FPS with live preview
- Undo/redo 50 ops; save/load round-trip exact
- Node mini-previews render correctly

## 📋 Phase 3 – Animation & Input (3 wks)
- Timeline dope-sheet: add/remove keys, Bezier handles, copy/paste
- Playback: space play/pause, loop region, scrub, frame-step
- Audio FFT: multi-channel, beat/onset detectors, gain calibration
- MIDI learn: CC/NRPN, device hot-plug, clock sync, save/load maps
- Gesture: MediaPipe hand tracking, depth-camera, calibration window

Acceptance:
- Keyframe animation plays smoothly ≥ 30 FPS loop
- Audio reactivity latency < 50 ms
- MIDI learn works instantly; maps persist across restarts
- Hand gesture modulates shader with ≤ 5 % jitter

## 📋 Phase 4 – Theming & Export (2 wks)
- Light & High-Contrast themes; CSS variables file; user override folder
- Command palette searchable; keyboard shortcuts customizable JSON
- Export: WGSL bundle, JSON meta, thumbnail PNG; optional HTML wrapper
- FFGL plugin generator: Windows DLL, macOS dylib, parameter map
- Accessibility: focus rings, aria-labels, screen-reader, font-size slider

Acceptance:
- Theme switch without restart; user CSS loads automatically
- Export produces self-contained package that runs in any WebGPU app
- FFGL plugin loads in Resolume/Ableton with identical parameters
- WCAG 2.2 AA compliance audit pass

## Definition of Done
- All acceptance criteria pass with automated tests and manual QA checklist signed off
- Performance budgets met: startup ≤ 3 s, memory < 2 GB, export 10 s ≤ 30 s
- Documentation updated: user guide, shortcut cheat-sheet, API docs for export formats
- Release package produced for Windows/macOS/Linux with installer/uninstaller
- Known issues list ≤ 5 low-severity items; no critical or high-severity bugs open

> Last updated: 2025-11-11 – Phase 0 complete; Phase 1-4 tracked in PRD