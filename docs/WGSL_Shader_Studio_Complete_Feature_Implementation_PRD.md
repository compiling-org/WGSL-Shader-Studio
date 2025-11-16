# WGSL Shader Studio – Complete Feature Implementation PRD

## 1. Executive Summary

```mermaid
graph TD
    A[Current State] --> B[Responsive UI]
    B --> C[Most Features Stubbed]
    
    D[Production Goal] --> E[Professional Shader Studio]
    E --> F[VJ & Creative Tools]
    
    G[Key Deliverables] --> H[ISF/GLSL/HLSL ↔ WGSL]
    G --> I[Visual Node Editor]
    G --> J[Timeline Animation]
    G --> K[Audio Reactivity]
    G --> L[MIDI Control]
    G --> M[Gesture Control]
    G --> N[Professional Export]
    
    style A fill:#ffebee
    style B fill:#ffcdd2
    style C fill:#ef9a9a
    style D fill:#e8f5e9
    style E fill:#c8e6c9
    style F fill:#a5d6a7
    style G fill:#e3f2fd
    style H fill:#bbdefb
    style I fill:#bbdefb
    style J fill:#bbdefb
    style K fill:#bbdefb
    style L fill:#bbdefb
    style M fill:#bbdefb
    style N fill:#bbdefb
```

**Current State:** The desktop application UI is now responsive, but most headline features remain stubs or non-functional. Users encounter placeholder text instead of working Audio, MIDI, Gesture, Node-Editor, Timeline, Theme-switching, or professional Export flows.

**Goal:** Ship a production-grade desktop shader studio that satisfies creative professionals and VJ artists by delivering:
- Bullet-proof ISF/GLSL/HLSL ↔ WGSL conversion with live preview ≤ 2 s
- A visual node graph editor (20+ node types) with real-time WGSL codegen
- Full animation timeline with key-framing and ≥ 30 FPS playback
- Live audio-reactivity, MIDI learn, and camera-based gesture control
- Material theming (Dark/Light/High-Contrast) and accessible keyboard workflows
- One-click export of self-contained WGSL + metadata for WebGPU, FFGL, ISF

## 2. User Stories & Acceptance Criteria

```mermaid
graph TD
    subgraph "Core Features"
        US1[US-01: Shader Conversion]
        US2[US-02: Node Editor]
        US3[US-03: Timeline & Animation]
    end
    
    subgraph "Input Sources"
        US4[US-04: Audio Reactivity]
        US5[US-05: MIDI Learn]
        US6[US-06: Gesture Control]
    end
    
    subgraph "UI & Export"
        US7[US-07: Theming]
        US8[US-08: Export]
    end
    
    US1 --> A[≤2s Conversion]
    US2 --> B[20+ Node Types]
    US3 --> C[30+ FPS Playback]
    US4 --> D[≤50ms Latency]
    US5 --> E[Instant Mapping]
    US6 --> F[≤5% Jitter]
    US7 --> G[Theme Switching]
    US8 --> H[One-click Export]
    
    style US1 fill:#e3f2fd
    style US2 fill:#e3f2fd
    style US3 fill:#e3f2fd
    style US4 fill:#f3e5f5
    style US5 fill:#f3e5f5
    style US6 fill:#f3e5f5
    style US7 fill:#e8f5e9
    style US8 fill:#e8f5e9
```

| ID | Story | Measurable Acceptance Criteria |
|----|-------|-------------------------------|
| US-01 | Shader Conversion | User drops GLSL/HLSL/ISF file → studio auto-converts to WGSL and shows live preview within 2 s; uniforms preserved; error list ≤ 3 items; import/export round-trip bit-identical for ISF 1.2 spec. |
| US-02 | Node Editor | User drags 20+ node types, wires them, sees generated WGSL update in < 500 ms; pan/zoom, group, delete, undo/redo; type-mismatch badges turn red; graph runs ≥ 45 FPS. |
| US-03 | Timeline & Animation | User keyframes any uniform, hits Play → preview animates smoothly ≥ 30 FPS loop; copy/paste keys; Bezier handles; time ruler snap; export video sequence. |
| US-04 | Audio Reactivity | User selects audio input, maps FFT band to uniform → shader reacts live with < 50 ms latency; multi-channel FFT; beat/onset triggers; gain calibration window. |
| US-05 | MIDI Learn | User twists MIDI knob, clicks "Learn" on parameter → mapping works instantly; NRPN support; device hot-plug; MIDI clock sync; save/load maps. |
| US-06 | Gesture Control | User holds hand in camera → hand position modulates shader parameters with ≤ 5 % jitter; MediaPipe fallback; depth-camera support; calibration UI. |
| US-07 | Theming | User picks Dark/Light/High-Contrast theme → all panels update without restart; custom CSS variables; keyboard shortcuts configurable; command palette searchable. |
| US-08 | Export | User clicks Export → studio writes self-contained WGSL file + JSON metadata + thumbnail PNG; optional HTML wrapper; FFGL plugin generator produces Windows/macOS DLL. |

## 3. Detailed Functional Requirements

### 3.1 Conversion Engine

```mermaid
graph TD
    A[Input Formats] --> B[ISF 1.2]
    A --> C[GLSL 4.5]
    A --> D[HLSL 6.0]
    
    B --> E[Parser]
    C --> E
    D --> E
    
    E --> F[Validation]
    F --> G[Conversion Engine]
    G --> H[WGSL Output]
    
    I[Features] --> J[Live Error Gutter]
    I --> K[≤3 Diagnostics]
    I --> L[Batch Convert]
    I --> M[Include Graph]
    
    style A fill:#e3f2fd
    style B fill:#bbdefb
    style C fill:#bbdefb
    style D fill:#bbdefb
    style G fill:#4fc3f7
    style H fill:#29b6f6
```

- **ISF 1.2** full import/export (passes official test suite)
- **GLSL 4.5 → WGSL** preserves semantics, comments, struct/uniform blocks
- **HLSL 6.0 → WGSL** handles DXC features, Texture2D, SamplerState
- Live error gutter in editor; ≤ 3 actionable diagnostics per compile
- Batch convert folder; maintain folder hierarchy and #include graph

### 3.2 Node Editor

```mermaid
graph TD
    A[Visual Canvas] --> B[Pan/Zoom]
    A --> C[Box Select]
    A --> D[Delete/Duplicate]
    
    E[Library Panel] --> F[Math Nodes]
    E --> G[Texture Nodes]
    E --> H[Time Nodes]
    E --> I[Audio Nodes]
    E --> J[MIDI Nodes]
    E --> K[Geometry Nodes]
    
    L[Node Features] --> M[Type-safe Pins]
    L --> N[Auto-cast]
    L --> O[Real-time Codegen]
    L --> P[Undo/Redo]
    L --> Q[JSON Save/Load]
    L --> R[Mini-preview]
    
    style A fill:#f3e5f5
    style E fill:#e1bee7
    style L fill:#ce93d8
```

- Visual canvas: pan (MMB), zoom (scroll), box-select, delete (Del), duplicate (Ctrl-D)
- Library panel: Math, Texture, Time, Audio, MIDI, Geometry, UV, Color, Blur, Noise, etc.
- Type-safe pins: float, vec2/3/4, mat4, texture; auto-cast when lossless
- Real-time WGSL codegen with comments; highlight corresponding node in editor
- Undo-stack 50 ops; JSON save/load; mini-preview on each node (toggle)

### 3.3 Timeline & Animation

```mermaid
graph LR
    A[Dope-sheet] --> B[Add/Remove Keys]
    A --> C[Bezier Handles]
    A --> D[Copy/Paste]
    
    E[Playback] --> F[Space Play/Pause]
    E --> G[Loop Region]
    E --> H[Scrub/Frame-step]
    
    I[Time Ruler] --> J[Beat Snap]
    I --> K[BPM Lock]
    I --> L[Time Signature]
    
    M[Export] --> N[PNG Sequence]
    M --> O[H.264 MP4]
    M --> P[GIF Export]
    
    style A fill:#fff3e0
    style E fill:#ffe0b2
    style I fill:#ffcc02
    style M fill:#ffb74d
```

- Dope-sheet: add key (K), remove (Shift-K), Bezier handles, copy/paste (Ctrl-C/V)
- Playback: space play/pause, loop region (drag ruler), scrub, frame-step (→)
- Time ruler: snap to beat (sub-divisions 1/4, 1/8, 1/16), BPM lock, time-signature
- Export: PNG sequence, H.264 MP4, GIF; include audio track; resolution selector

### 3.4 Input Sources

```mermaid
graph TD
    subgraph "Audio"
        A[Multi-channel FFT] --> B[512 Bins]
        A --> C[Beat Detection]
        A --> D[Onset Detection]
        A --> E[RMS/Centroid]
    end
    
    subgraph "MIDI"
        F[MIDI In/Out] --> G[CC Learn]
        F --> H[NRPN Support]
        F --> I[Clock Sync]
        F --> J[Device Hot-plug]
    end
    
    subgraph "Gesture"
        K[MediaPipe] --> L[Hand Landmarks]
        K --> M[Pose Landmarks]
        N[Depth Camera] --> O[Point Cloud]
    end
    
    style A fill:#e8f5e9
    style F fill:#e3f2fd
    style K fill:#f3e5f5
```

- **Audio**: multi-channel FFT 512 bins, beat detection (energy flux), onset (spectral flux), RMS, centroid; latency ≤ 50 ms
- **MIDI**: in/out ports, CC learn, NRPN, clock sync, device hot-plug indicator, save/load maps JSON
- **Gesture**: MediaPipe hand landmarks, pose landmarks, depth-camera point-cloud; calibration window; fallback to mouse if no camera

### 3.5 Preview & Performance

```mermaid
graph LR
    A[Preview Panel] --> B[Resolution ½, 1, 2×]
    A --> C[Pause/Step]
    A --> D[Background Render]
    
    E[Perf Overlay] --> F[FPS Display]
    E --> G[GPU ms]
    E --> H[Uniform Count]
    E --> I[Texture Memory]
    
    J[Fallback] --> K[CPU Path]
    J --> L[Warning Banner]
    
    style A fill:#fff3e0
    style E fill:#e8f5e9
    style J fill:#ffebee
```

- Real-time preview panel: resolution ½, 1, 2×; pause (P), step (→), background render toggle
- Perf overlay: FPS, GPU ms, uniform count, texture mem; color-coded thresholds
- Fallback: CPU path if WebGPU unavailable; warning banner

### 3.6 Theming & Accessibility

```mermaid
graph TD
    A[Themes] --> B[Dark Mode]
    A --> C[Light Mode]
    A --> D[High Contrast]
    
    E[Customization] --> F[CSS Variables]
    E --> G[User Override]
    
    H[Accessibility] --> I[Focus Rings]
    H --> J[ARIA Labels]
    H --> K[Screen Reader]
    H --> L[Font Size]
    
    M[Keyboard] --> N[Custom Shortcuts]
    M --> O[Command Palette]
    
    style A fill:#e8f5e9
    style E fill:#e3f2fd
    style H fill:#f3e5f5
    style M fill:#fff3e0
```

- Dark, Light, High-Contrast palettes; CSS variables file; user override folder
- Keyboard shortcuts: customizable JSON; command palette (Ctrl-Shift-P); searchable
- Accessibility: focus rings, aria-labels, screen-reader descriptions, font-size slider

### 3.7 Export & Integration

```mermaid
graph TD
    A[Export Formats] --> B[WGSL + JSON]
    A --> C[Thumbnail PNG]
    A --> D[HTML Wrapper]
    
    E[FFGL Plugin] --> F[Windows DLL]
    E --> G[macOS dylib]
    E --> H[Parameter Map]
    
    I[ISF Round-trip] --> J[Import ISF]
    I --> K[Edit]
    I --> L[Export ISF]
    I --> M[Validate]
    
    style A fill:#e8f5e9
    style E fill:#e3f2fd
    style I fill:#f3e5f5
```

- Export WGSL, JSON uniforms, thumbnail PNG; optional single-file HTML wrapper
- FFGL plugin generator: Windows DLL, macOS dylib, parameter map identical to UI
- ISF round-trip: import ISF → edit → export ISF; validate with official tool

## 4. Non-Functional Requirements

```mermaid
graph LR
    A[Performance] --> B[≤3s Startup]
    A --> C[≥60 FPS Preview]
    A --> D[<2GB Memory]
    
    E[Platforms] --> F[Windows 10+]
    E --> G[macOS 11+]
    E --> H[Ubuntu 20.04+]
    
    I[Reliability] --> J[<1 Crash/8h]
    I --> K[Graceful Fallback]
    I --> L[Autosave 30s]
    
    M[Security] --> N[No Network]
    M --> O[Shader Sandbox]
    
    P[Accessibility] --> Q[WCAG 2.2 AA]
    P --> R[Keyboard Only]
    P --> S[Color-blind Safe]
    
    style A fill:#ffebee
    style E fill:#e8f5e9
    style I fill:#e3f2fd
    style M fill:#f3e5f5
    style P fill:#fff3e0
```

- **Performance:** startup ≤ 3 s on M1/RTX 3060; preview ≥ 60 FPS at 1080 p; memory < 2 GB typical project
- **Platforms:** Windows 10+ (x64, ARM64), macOS 11+ (x64, Apple Silicon), Ubuntu 20.04+ (x64, ARM64)
- **Reliability:** crash rate < 1 per 8 h continuous use; graceful fallback on GPU loss; autosave every 30 s
- **Security:** no network calls unless user opts-in; shader sandbox prevents file system access
- **Accessibility:** WCAG 2.2 AA compliant; keyboard-only operation possible; color-blind safe palettes

## 5. Work Breakdown & Priority

```mermaid
gantt
    title Development Timeline
    dateFormat  YYYY-MM-DD
    section Foundation
    Solid Converter           :done,    des1, 2024-01-01,2024-01-14
    Working Preview           :done,    des2, 2024-01-07,2024-01-14
    Dark Theme               :done,    des3, 2024-01-10,2024-01-14
    
    section Node Editor
    Full Graph UI            :active,  des4, 2024-01-15,2024-02-05
    20+ Nodes                :          des5, 2024-01-20,2024-02-05
    WGSL Codegen             :          des6, 2024-01-25,2024-02-05
    Undo/Redo                :          des7, 2024-02-01,2024-02-05
    
    section Animation & Input
    Timeline System          :          des8, 2024-02-06,2024-02-26
    Audio Reactivity         :          des9, 2024-02-12,2024-02-26
    MIDI Control             :          des10, 2024-02-18,2024-02-26
    Gesture Control          :          des11, 2024-02-20,2024-02-26
    
    section Polish & Export
    Theme System             :          des12, 2024-02-27,2024-03-12
    Export Pipeline          :          des13, 2024-03-01,2024-03-12
    QA & Testing             :          des14, 2024-03-05,2024-03-12
```

| Phase | Duration | Focus | Key Deliverables |
|-------|----------|-------|------------------|
| A – Foundation | 2 wks | Solid converter, working preview, dark theme | US-01 pass, US-07 dark theme, perf budget met |
| B – Node Editor | 3 wks | Full graph UI, 20 nodes, WGSL codegen | US-02 pass, undo/redo, JSON save/load |
| C – Animation & Input | 3 wks | Timeline, audio, MIDI, gesture | US-03, US-04, US-05, US-06 pass |
| D – Polish & Export | 2 wks | Theming, shortcuts, export, QA | US-07 light/high-contrast, US-08 pass, manual QA sign-off |

## 6. Open Questions & Risks

```mermaid
graph TD
    A[Open Questions] --> B[MediaPipe Bundle Size]
    A --> C[ISF Array Ambiguity]
    A --> D[Graph Layout Algorithm]
    A --> E[FFGL SDK License]
    
    B --> F[>100MB Bundle?]
    B --> G[Fallback Options]
    
    C --> H[Max Array Length]
    C --> I[Naming Convention]
    
    D --> J[Force-directed vs Hierarchical]
    
    E --> K[Resolume SDK Terms]
    
    style A fill:#fff3e0
    style B fill:#ffe0b2
    style C fill:#ffe0b2
    style D fill:#ffe0b2
    style E fill:#ffe0b2
```

- **MediaPipe native library size vs web-runtime:** bundle size could exceed 100 MB; fallback to lighter tracker or optional download?
- **ISF spec ambiguity on array uniforms:** decision needed on max array length and naming convention
- **Node editor graph layout algorithm:** use force-directed (organic) or layered (hierarchical) for auto-layout?
- **FFGL SDK availability:** confirm Resolume SDK license allows redistribution

## 7. Definition of Done

```mermaid
graph TD
    A[Definition of Done] --> B[All US-x Pass]
    B --> C[Automated Tests]
    B --> D[Manual QA Sign-off]
    
    A --> E[Performance Budgets]
    E --> F[1080p Preview]
    E --> G[20-node Graph]
    E --> H[4 Audio Mappings]
    
    A --> I[Documentation]
    I --> J[User Guide]
    I --> K[Shortcut Cheat-sheet]
    I --> L[API Docs]
    
    A --> M[Release Package]
    M --> N[Windows Installer]
    M --> O[macOS Installer]
    M --> P[Linux Package]
    
    A --> Q[Quality Gates]
    Q --> R[≤5 Low Issues]
    Q --> S[No Critical Bugs]
    Q --> T[No High Bugs]
    
    style A fill:#e8f5e9
    style B fill:#c8e6c9
    style E fill:#c8e6c9
    style I fill:#c8e6c9
    style M fill:#c8e6c9
    style Q fill:#c8e6c9
```

- All US-x acceptance criteria pass with automated tests and manual QA checklist signed off
- Performance budgets met under typical load (1080 p preview, 20-node graph, 4 audio mappings)
- Documentation updated: user guide, shortcut cheat-sheet, API docs for export formats
- Release package produced for Windows/macOS/Linux with installer/uninstaller
- Known issues list ≤ 5 low-severity items; no critical or high-severity bugs open

---

*This PRD is automatically updated to reflect current implementation status and requirements.*