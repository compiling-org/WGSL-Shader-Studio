# Development Roadmap - Wiring Reality (Phase 1)

**🎯 PHASE 1 STATUS - DECEMBER 2025**: 
- ⚙️ **All core systems present** — wiring and refinement in progress
- 🧩 **UI panels, node editor, file ops, audio/MIDI** — present; integration pending
- ✅ **No compilation errors reported** — focus on stabilizing integration points
- ✅ **Reference patterns present** — `use.gpu`, `bevy_shader_graph`, `egui_node_graph2` wiring ongoing

```mermaid
flowchart TD
    Phase1["Critical Phase (Week 1-2)"] --> Fix["Stabilize Wiring Across Modules"]
    Phase1 --> UI["Refine UI Panel Wiring"]
    Phase2["Foundation (Week 3-6)"] --> WGPU["WGPU Integration MVP"]
    Phase2 --> Compile["Basic Shader Compilation"]
    Phase2 --> Files["File Operations (Open/Save)"]
    Phase3["Core Features (Week 7-12)"] --> Preview["Live Preview Window"]
    Phase3 --> Params["Parameter Controls"]
    Phase3 --> ISF["ISF Import/Export"]
    Phase4["Advanced (Week 13-20)"] --> Node["Node Editor MVP"]
    Phase4 --> AudioMIDI["Audio/MIDI Integration"]
    Phase4 --> FFGL["FFGL Plugin Export"]
    Phase5["Polish (Week 21-24)"] --> Perf["Performance Optimization"]
    Phase5 --> Testing["Cross-platform Testing"]
    Phase5 --> Docs["Documentation/Testing"]
```

## CRITICAL PHASE (Week 1-2) - STABILIZE WIRING

```mermaid
graph TD
    A[CRITICAL TASKS] --> B[Stabilize Wiring Across Modules]
    A --> C[Normalize EditorState Structure]
    A --> D[Verify Function Signatures]
    A --> E[Implement Missing Glue Code]
    
    B --> B1[End-to-end data flow]
    B --> B2[UI ↔ Engine integration]
    B --> B3[Preview control wiring]
    
    C --> C1[Consistent UI state fields]
    C --> C2[Struct definitions aligned]
    C --> C3[Unified app state schema]
    
    D --> D1[Return types verified]
    D --> D2[Parameter types consistent]
    D --> D3[Trait implementations aligned]
    
    E --> E1[UI→Engine handlers]
    E --> E2[Repo code integration]
    E --> E3[Diagnostics wiring]
    
```

## 🎯 Current Implementation Reality

- **All core systems present** — WGPU scaffolding, shader compile path, panel UI, node editor UI, file ops, audio/MIDI input layer
- **Wiring incomplete** — end-to-end connections between UI, compiler, and preview need stabilization
- **Preview unstable** — rendering path present; refine pipeline creation and state transitions
- **Reference patterns integrated** — `use.gpu`, `bevy_shader_graph`, `egui_node_graph2` used; continue aligning types and data flow
- **Focus** — remove brittle edges, unify state, wire controls to preview, and stabilize diagnostics

## FOUNDATION PHASE (Week 3-6) - BUILD CORE SYSTEMS

```mermaid
graph TD
    A[FOUNDATION SYSTEMS] --> B[WGPU Integration MVP]
    A --> C[Basic Shader Compilation]
    A --> D[File Operations]
    A --> E[Basic UI Framework]
    
    B --> B1[Create WGPU device]
    B --> B2[Setup render pipeline]
    B --> B3[Basic shader loading]
    
    C --> C1[naga WGSL parsing]
    C --> C2[Shader validation]
    C --> C3[Diagnostics pipeline]
    
    D --> D1[File dialogs]
    D --> D2[Open/Save shaders]
    D --> D3[Recent files list]
    
    E --> E1[Three-panel layout]
    E --> E2[Basic docking]
    E --> E3[Panel resizing]
    
```

**FOUNDATION GOALS**: Build working core systems
1. **WGPU Integration** - Basic rendering pipeline
2. **Shader Compilation** - naga integration for WGSL
3. **File Operations** - Open, save, recent files
4. **UI Framework** - Working panel system

## CORE FEATURES PHASE (Week 7-12) - ESSENTIAL FUNCTIONALITY

```mermaid
graph TD
    A[CORE FEATURES] --> B[Live Preview Window]
    A --> C[Parameter Controls]
    A --> D[ISF Import/Export]
    A --> E[Menu System]
    
    B --> B1[Shader rendering]
    B --> B2[Real-time preview]
    B --> B3[Preview controls]
    
    C --> C1[Slider controls]
    C --> C2[Color pickers]
    C --> C3[Checkbox toggles]
    
    D --> D1[ISF file parsing]
    D --> D2[Parameter extraction]
    D --> D3[Metadata handling]
    
    E --> E1[File menu]
    E --> E2[Edit menu]
    E --> E3[View menu]
    
    style A fill:#4caf50
    style B fill:#e8f5e9
    style C fill:#e8f5e9
    style D fill:#e8f5e9
    style E fill:#e8f5e9
```

**CORE FEATURES**: Essential functionality for basic shader editing
1. **Live Preview** - Real-time shader rendering
2. **Parameter Controls** - UI for shader uniforms
3. **ISF Support** - Import/export ISF format
4. **Menu System** - Standard application menus

## Risks & Mitigations

```mermaid
graph TD
    A[Project Risks] --> B[Bevy/egui Timing Issues]
    A --> C[Cross-platform Differences]
    A --> D[Performance Bottlenecks]
    A --> E[WebGPU Compatibility]
    
    B --> F[Startup Guards]
    B --> G[Status Overlays]
    B --> H[Upstream Tracking]
    
    C --> I[CI Runners]
    C --> J[Manual QA Matrix]
    C --> K[Platform Testing]
    
    D --> L[GPU Profiling]
    D --> M[Adaptive Quality]
    D --> N[Optimization Passes]
    
    E --> O[WASM Assessment]
    E --> P[Browser Testing]
    E --> Q[Fallback Systems]
    
    style A fill:#ffebee
    style B fill:#ffcdd2
    style C fill:#ffcdd2
    style D fill:#ffcdd2
    style E fill:#ffcdd2
    style F fill:#e8f5e9
    style G fill:#e8f5e9
    style H fill:#e8f5e9
    style I fill:#e8f5e9
    style J fill:#e8f5e9
    style K fill:#e8f5e9
```

- **Bevy/egui timing issues**: Guard with gates and overlays; upstream tracking.
- **Cross-platform differences**: Dedicate CI runners; manual QA matrix.
- **Performance bottlenecks**: GPU profiling, adaptive quality, optimization passes.
- **WebGPU compatibility**: WASM assessment, browser testing, fallback systems.
