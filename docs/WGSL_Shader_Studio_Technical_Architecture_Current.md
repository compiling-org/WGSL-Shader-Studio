# WGSL Shader Studio - Technical Architecture (Current Reality)

## Current Reality (Updated 2025-12-14)

```mermaid
flowchart TD
    App[Bevy + bevy_egui app boots] --> Panels[Multiple panels present]
    Panels --> Preview[Preview under repair]
    Panels --> NodeUI[Node editor UI present]
    NodeUI --> Wiring[Wiring exposure pending]
    Tooling[WGSL tooling modules present] --> Diagnostics[Diagnostics exist]
    Diagnostics --> Relax[Validation/binding rules being relaxed]
    Renderer[WGPU renderer modules present] --> Preview
    ThreeD[3D viewport modules exist] --> InitPending[Initialization pending]
    Analyzer[Analyzer scripts exist] --> Inconsistent[Reports inconsistent]
```

## Technology Stack (Updated Snapshot)

```mermaid
flowchart LR
    Bevy[Bevy 0.17] --> Windowing[Cross-platform Windowing]
    Egui[bevy_egui 0.38] --> UIF[UI Framework]
    WGPU[wgpu] --> Render[Rendering Modules]
    Naga[naga] --> Compile[Shader Validation/Compilation]
    Rfd[rfd] --> Dialogs[File Dialogs]
    Audio[Audio System] --> FFT[Analysis + Features]
    Midir[midir] --> MIDI[MIDI Support]
```

| Component | Current Version | Status | Required For |
|-----------|-----------------|---------|--------------|
| Bevy | 0.17 | ✅ Available | Window management |
| bevy_egui | 0.38 | ✅ Available | UI rendering |
| wgpu | Present (modules) | ⚠️ Wired; preview unstable | GPU rendering |
| naga | Available | ⚠️ Wired; validation active | Shader compilation |
| rfd | 0.15.x | ⚠️ Wired in UI | File dialogs |
| Audio system | Custom (dasp/ringbuf) | ⚠️ Wired; analysis present | Audio analysis |
| midir | 0.10.x | ⚠️ Wired; mapping pending | MIDI control |

## Application Architecture (Active Repair)

```mermaid
flowchart TD
    Main[src/main.rs] --> Flag[Feature Flag]
    Flag --> GUI[bevy_app::run_app()]
    Flag --> CLI[CLI Fallback]
    GUI --> App[App::new()]
    App --> Plugins[DefaultPlugins]
    App --> Egui[EguiPlugin]
    App --> EditorUI[EditorUI Systems]
    EditorUI --> PreviewRepair[Preview wiring repair]
    EditorUI --> NodeWiring[Expose node editor wiring]
```

## Core Systems Status

### Rendering Pipeline (Under Repair)

```mermaid
flowchart TD
    A[WGSL Shader] --> B[Shader Compilation]
    B --> C[WGPU Pipeline]
    C --> D[Uniform Binding]
    D --> E[Live Preview]
    F[Init/Resize Handling] --> G[Texture/Viewport Sync]
    G --> H[Reliable Frame Present]
```

### UI Layout System (Partial)

```mermaid
flowchart TD
    A[Three-Panel Layout] --> B[Left: Shader Browser]
    A --> C[Center: Preview/Editor]
    A --> D[Right: Parameters]
    A --> E[Bottom: Timeline/Code]
    Preview[Preview Panel] --> Repair[Wiring under repair]
    NodeUI[Node Editor UI] --> Expose[Wiring exposure pending]
    Timeline[Timeline] --> Verify[Integration verification pending]
```

### File System Integration (❌ MISSING)

```mermaid
flowchart TD
    Required[Required Operations] --> Dialogs[Open/Save WGSL]
    Required --> Import[Import ISF/GLSL/HLSL]
    Required --> Export[Export Formats]
    Required --> Projects[Project Management]
    Current[Current Status] --> NoDialogs[File dialogs partially wired]
    Current --> NoOps[Limited file operations]
    Current --> NoImportExport[Import/Export pending]
    Current --> NoSaveLoad[Project save/load pending]
    Missing[Missing Infrastructure] --> RfdIntegration[rfd integration completion]
    Missing --> Filters[File filters]
    Missing --> Recent[Recent files]
    Missing --> Format[Project format]
```

## Data Flow Architecture (Operational Flow)

```mermaid
flowchart TD
    User --> UI
    UI --> Diagnostics
    Diagnostics --> UI
    UI --> Compile
    Compile --> Pipeline
    Pipeline --> Present
    Present --> UI
```

## Feature Implementation Matrix

```mermaid
flowchart LR
    P1[WGPU Integration] --> P1s[⚠️ Partial]
    P2[Shader Compilation] --> P2s[⚠️ Partial]
    P3[Three-Panel UI] --> P3s[⚠️ Partial]
    P4[File Operations] --> P4s[⚠️ Partial]
    P5[WGSL Highlighting] --> P5s[⚠️ Partial]
    P6[Parameter Controls] --> P6s[❌ Missing]
    P7[Live Preview] --> P7s[⚠️ Unstable]
    P8[Error System] --> P8s[❌ Missing]
    P9[Node Editor] --> P9s[⚠️ UI present; wiring pending]
    P10[Timeline] --> P10s[❌ Missing]
    P11[Audio/MIDI] --> P11s[⚠️ Wired; mapping pending]
    P12[Export/Import] --> P12s[❌ Missing]
```

## Compilation Error Analysis

```mermaid
flowchart TD
    Errors[Compilation Errors Present] --> Fields[Field Missing: ~8]
    Errors --> Signatures[Function Signature: ~12]
    Errors --> Types[Type Mismatch: ~7]
    Errors --> Imports[Import Issues: ~6]
    Fields --> ShaderBrowser[shader_browser field missing]
    Fields --> Diagnostics[diagnostic methods missing]
    Signatures --> CompileFunctions[Broken compile functions]
    Signatures --> ParamTypes[Missing parameter types]
    Types --> Return[Wrong return types]
    Types --> Conflicts[Parameter type conflicts]
    Imports --> Missing[Missing imports]
    Imports --> WrongPaths[Wrong module paths]
```

## Recovery Roadmap

```mermaid
flowchart LR
    Phase1[Phase 1: Foundation] --> FixErrors[Fix Compilation Errors]
    Phase1 --> WGPUCore[Implement WGPU Core]
    Phase1 --> BasicUI[Basic UI Layout]
    Phase2[Phase 2: Core Features] --> ShaderComp[Shader Compilation]
    Phase2 --> FileOps[File Operations]
    Phase2 --> LivePreview[Live Preview]
    Phase3[Phase 3: Advanced] --> NodeEditor[Node Editor]
    Phase3 --> AudioMIDI[Audio/MIDI]
    Phase3 --> ExportImport[Export/Import]
```

## Critical Dependencies Required

```mermaid
flowchart TD
    Immediate[Immediate Dependencies] --> WGPU[wgpu]
    Immediate --> NAGA[naga]
    Immediate --> EGUI[bevy_egui]
    Systems[System Dependencies] --> RFD[rfd]
    Systems --> MIDIR[midir]
    Infra[Infrastructure] --> Errors[Error handling]
    Infra --> Logging[Logging]
    Infra --> Config[Configuration]
```

---

## Summary

**CURRENT REALITY**: This project is **partially wired** with:
- ⚙️ Core systems present; wiring and integrations incomplete
- ⚠️ Preview path unstable; renderer lifecycle refinement needed
- 🧩 UI panels present; wire controls and diagnostics reliably
- 🔗 File dialog hooks present; project system wiring pending

**FOCUS**: Stabilize wiring, unify state, refine integrations; progress features to reliable baseline.

**⚠️ CRITICAL**: Development should align with Bevy 0.17 + bevy_egui 0.38 and the current wired dependencies.

---

*This document reflects the **ACTUAL CURRENT STATE** based on comprehensive code analysis - not wishful thinking or false claims.*
