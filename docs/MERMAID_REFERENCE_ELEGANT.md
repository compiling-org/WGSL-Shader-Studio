# WGSL Shader Studio - Elegant Mermaid Reference Document

## 🎯 PURPOSE
This document serves as the **AUTHORITATIVE REFERENCE** for all elegant mermaid diagrams used throughout the WGSL Shader Studio documentation. These diagrams represent the **ACTUAL CURRENT STATE** and working architecture of the project.

## 📋 TECHNOLOGY STACK REFERENCE

### Framework Architecture
```mermaid
graph TD
    A[WGSL Shader Studio] --> B{Technology Stack}
    B --> C[Bevy 0.17 + bevy_egui 0.38]
    B --> D[wgpu 0.19+]
    B --> E[naga]
    B --> F[rfd]
    B --> G[cpal + midir]
    
    C --> H[Cross-platform UI]
    D --> I[GPU Rendering]
    E --> J[Shader Compilation]
    F --> K[File Dialogs]
    G --> L[Audio/MIDI]
    
    style A fill:#e3f2fe
    style C fill:#2196f3
    style D fill:#9c27b0
    style E fill:#4caf50
    style F fill:#ff9800
    style G fill:#f44336
```

### Application Entry Flow
```mermaid
graph TD
    A[src/main.rs] --> B{Feature Detection}
    B -->|gui| C[bevy_app::run_app()]
    B -->|cli| D[CLI Fallback]
    
    C --> E[App::new()]
    E --> F[DefaultPlugins]
    E --> G[EguiPlugin]
    E --> H[EditorUI Systems]
    
    style A fill:#fff3e0
    style C fill:#4caf50
    style E fill:#e3f2fd
    style F fill:#bbdefb
    style G fill:#4fc3f7
    style H fill:#29b6f6
```

## 🏗️ CORE SYSTEMS ARCHITECTURE

### Rendering Pipeline
```mermaid
graph TD
    A[WGSL Shader Input] --> B[Shader Compiler]
    B --> C[WGPU Pipeline]
    C --> D[Uniform Binding]
    D --> E[Parameter Updates]
    E --> F[Live Preview]
    
    G[Timeline] --> D
    H[UI Controls] --> D
    I[Audio Analysis] --> D
    J[MIDI Control] --> D
    
    style A fill:#e8f5e9
    style B fill:#c8e6c9
    style C fill:#a5d6a7
    style D fill:#81c784
    style E fill:#66bb6a
    style F fill:#4caf50
```

### UI Layout System
```mermaid
graph TD
    A[Three-Panel Layout] --> B[Left Panel: 25%]
    A --> C[Center Panel: 50%]
    A --> D[Right Panel: 25%]
    A --> E[Bottom Panel: 200px]
    
    B --> F[Shader Browser]
    C --> G[Preview Viewport]
    C --> H[Code Editor]
    D --> I[Parameter Controls]
    E --> J[Timeline]
    E --> K[Error Console]
    
    L[Panel Management] --> M[Resizing]
    L --> N[Visibility Toggle]
    L --> O[Docking System]
    
    style A fill:#e3f2fd
    style B fill:#bbdefb
    style C fill:#90caf9
    style D fill:#64b5f6
    style E fill:#42a5f5
    style L fill:#2196f3
```

### File System Integration
```mermaid
graph TD
    A[File Operations] --> B[Native OS Dialogs]
    B --> C[Recent Files]
    C --> D[Project Management]
    
    E[Supported Formats] --> F[WGSL]
    E --> G[ISF 1.2]
    E --> H[GLSL 4.5]
    E --> I[HLSL 6.0]
    
    J[Import Pipeline] --> K[Format Detection]
    K --> L[Automatic Conversion]
    L --> M[WGSL Output]
    
    N[Export Pipeline] --> O[Multiple Formats]
    O --> P[FFGL Plugin]
    O --> Q[Video Export]
    O --> R[Image Sequence]
    
    style A fill:#f3e5f5
    style B fill:#e1bee7
    style E fill:#ce93d8
    style J fill:#ba68c8
    style N fill:#ab47bc
```

## 🎨 NODE EDITOR SYSTEM

### Node Graph Architecture
```mermaid
graph TD
    A[Node Graph] --> B[Node Library]
    B --> C[Node Creation]
    C --> D[Connection System]
    D --> E[Topological Sort]
    E --> F[Code Generation]
    F --> G[WGSL Output]
    
    H[Node Categories] --> I[Math Nodes]
    H --> J[Time Nodes]
    H --> K[UV Nodes]
    H --> L[Texture Nodes]
    H --> M[Color Nodes]
    H --> N[Audio Nodes]
    
    O[Editor Features] --> P[Drag & Drop]
    O --> Q[Pan/Zoom]
    O --> R[Box Selection]
    O --> S[Connection Drawing]
    
    style A fill:#e8f5e9
    style B fill:#c8e6c9
    style C fill:#a5d6a7
    style D fill:#81c784
    style E fill:#66bb6a
    style F fill:#4caf50
    style O fill:#2196f3
```

### Node Types Reference
```mermaid
graph LR
    subgraph "Math Nodes"
        A[Add, Multiply] --> B[Divide, Subtract]
        B --> C[Sin, Cos, Tan]
        C --> D[Floor, Ceil, Fract]
    end
    
    subgraph "Time Nodes"
        E[Time] --> F[Delta Time]
        F --> G[Beat Detection]
        G --> H[Timeline Sync]
    end
    
    subgraph "Input Nodes"
        I[UV Coordinates] --> J[Screen Position]
        J --> K[Mouse Position]
        K --> L[Audio FFT]
    end
    
    subgraph "Output Nodes"
        M[Fragment Color] --> N[Vertex Position]
        N --> O[Custom Variables]
    end
    
    style A fill:#ffebee
    style E fill:#e3f2fd
    style I fill:#f3e5f5
    style M fill:#e8f5e9
```

## ⏱️ TIMELINE & ANIMATION

### Animation System
```mermaid
graph TD
    A[Timeline System] --> B[Keyframe Creation]
    B --> C[Interpolation]
    C --> D[Parameter Animation]
    D --> E[Real-time Updates]
    
    F[Keyframe Types] --> G[Linear]
    F --> H[Bezier]
    F --> I[Ease In/Out]
    F --> J[Step]
    
    K[Timeline Features] --> L[Copy/Paste Keys]
    K --> M[Loop Regions]
    K --> N[Time Ruler]
    K --> O[Beat Snap]
    
    P[Export Options] --> Q[Video MP4]
    P --> R[PNG Sequence]
    P --> S[GIF Animation]
    
    style A fill:#fff3e0
    style B fill:#ffe0b2
    style C fill:#ffcc02
    style D fill:#ffb74d
    style F fill:#ff9800
    style K fill:#ff5722
    style P fill:#f44336
```

### Playback Control
```mermaid
graph LR
    A[Playback Controls] --> B[Play/Pause]
    A --> C[Stop]
    A --> D[Loop Toggle]
    A --> E[Frame Step]
    
    F[Time Controls] --> G[Scrubbing]
    F --> H[Time Input]
    F --> I[Duration Display]
    
    J[Performance] --> K[60 FPS Target]
    J --> L[Smooth Interpolation]
    J --> M[Real-time Updates]
    
    style A fill:#e8f5e9
    style B fill:#c8e6c9
    style F fill:#a5d6a7
    style J fill:#81c784
```

## 🎵 AUDIO & MIDI INTEGRATION

### Audio Analysis System
```mermaid
graph TD
    A[Audio Input] --> B[FFT Processing]
    B --> C[512 Frequency Bins]
    C --> D[Beat Detection]
    D --> E[Onset Detection]
    E --> F[Parameter Mapping]
    
    G[Audio Features] --> H[RMS Energy]
    G --> I[Spectral Centroid]
    G --> J[Zero Crossing Rate]
    G --> K[Frequency Bands]
    
    L[Performance] --> M[< 50ms Latency]
    L --> N[< 5% CPU Usage]
    L --> O[Multi-channel Support]
    
    style A fill:#e3f2fd
    style B fill:#bbdefb
    style C fill:#90caf9
    style D fill:#64b5f6
    style E fill:#42a5f5
    style L fill:#2196f3
```

### MIDI Control System
```mermaid
graph LR
    A[MIDI Input] --> B[Device Detection]
    B --> C[Message Parsing]
    C --> D[CC Mapping]
    D --> E[Parameter Control]
    
    F[MIDI Features] --> G[Learn Function]
    F --> H[NRPN Support]
    F --> I[Clock Sync]
    F --> J[Device Hot-plug]
    
    K[Mapping Options] --> L[Direct Mapping]
    L --> M[Smoothing Filter]
    M --> N[Range Scaling]
    
    style A fill:#f3e5f5
    style B fill:#e1bee7
    style C fill:#ce93d8
    style D fill:#ba68c8
    style K fill:#ab47bc
```

## 📊 PERFORMANCE & MONITORING

### Performance Metrics
```mermaid
graph TD
    A[Performance Monitoring] --> B[FPS Counter]
    B --> C[Frame Time]
    C --> D[GPU Time]
    D --> E[Memory Usage]
    
    F[Target Metrics] --> G[60 FPS Minimum]
    F --> H[< 16ms Frame Time]
    F --> I[< 2GB Memory]
    F --> J[< 5% CPU Audio]
    
    K[Optimization] --> L[GPU Batch Rendering]
    L --> M[Texture Compression]
    M --> N[Uniform Buffer Reuse]
    
    style A fill:#ffebee
    style B fill:#ffcdd2
    style C fill:#ef9a9a
    style D fill:#e57373
    style F fill:#f44336
    style K fill:#d32f2f
```

### Error Handling System
```mermaid
graph TD
    A[Error Detection] --> B[Compilation Errors]
    B --> C[Runtime Errors]
    C --> D[User Notification]
    
    E[Error Types] --> F[Shader Compilation]
    E --> G[File I/O]
    E --> H[GPU Errors]
    E --> I[Audio/MIDI]
    
    J[Recovery] --> K[Graceful Degradation]
    K --> L[Fallback Rendering]
    L --> M[User Guidance]
    
    style A fill:#fff3e0
    style B fill:#ffe0b2
    style C fill:#ffcc02
    style E fill:#ffb74d
    style J fill:#ff9800
```

## 🚀 EXPORT & INTEGRATION

### Export Pipeline
```mermaid
graph TD
    A[Export System] --> B[Format Selection]
    B --> C[WGSL + JSON]
    B --> D[FFGL Plugin]
    B --> E[Video Export]
    B --> F[Image Sequence]
    
    G[FFGL Generation] --> H[Windows DLL]
    G --> I[macOS dylib]
    H --> J[Parameter Mapping]
    
    K[Video Export] --> L[H.264 MP4]
    K --> M[PNG Sequence]
    K --> N[GIF Animation]
    
    style A fill:#e8f5e9
    style B fill:#c8e6c9
    style G fill:#a5d6a7
    style K fill:#81c784
```

### Cross-platform Support
```mermaid
graph LR
    A[Platform Support] --> B[Windows 10+]
    A --> C[macOS 11+]
    A --> D[Ubuntu 20.04+]
    
    E[Architecture] --> F[x64 Intel/AMD]
    E --> G[ARM64 Apple Silicon]
    E --> H[ARM64 Linux]
    
    I[Dependencies] --> J[Native Dialogs]
    I --> K[GPU Drivers]
    I --> L[Audio APIs]
    
    style A fill:#e3f2fd
    style B fill:#bbdefb
    style C fill:#90caf9
    style D fill:#64b5f6
    style E fill:#42a5f5
```

## 🎨 VISUAL DESIGN SYSTEM

### Theme Architecture
```mermaid
graph TD
    A[Theme System] --> B[Dark Theme]
    A --> C[Light Theme]
    A --> D[High Contrast]
    
    E[Customization] --> F[CSS Variables]
    F --> G[User Overrides]
    G --> H[Custom Palettes]
    
    I[Accessibility] --> J[WCAG 2.2 AA]
    I --> K[Keyboard Navigation]
    I --> L[Screen Reader]
    
    style A fill:#f3e5f5
    style B fill:#e1bee7
    style C fill:#ce93d8
    style D fill:#ba68c8
    style E fill:#ab47bc
    style I fill:#9c27b0
```

### UI Component Hierarchy
```mermaid
graph TD
    A[UI Components] --> B[Layout Containers]
    A --> C[Input Controls]
    A --> D[Display Components]
    A --> E[Feedback Elements]
    
    B --> F[Panels]
    B --> G[Splitters]
    B --> H[Scroll Areas]
    
    C --> I[Sliders]
    C --> J[Color Pickers]
    C --> K[Text Inputs]
    C --> L[Buttons]
    
    D --> M[Code Editor]
    D --> N[Node Graph]
    D --> O[Timeline]
    D --> P[Preview Viewport]
    
    E --> Q[Error Messages]
    E --> R[Status Bar]
    E --> S[Progress Indicators]
    
    style A fill:#e8f5e9
    style B fill:#c8e6c9
    style C fill:#a5d6a7
    style D fill:#81c784
    style E fill:#66bb6a
```

---

## 📋 USAGE GUIDELINES

### When to Use These Diagrams
- **Technical Architecture**: Use the Technology Stack and Core Systems diagrams
- **Feature Implementation**: Reference the Node Editor and Timeline diagrams
- **Performance Discussion**: Use Performance & Monitoring diagrams
- **Export/Integration**: Reference the Export Pipeline diagrams

### Color Coding Standard
- **🟢 Green**: Working/Implemented features
- **🟡 Yellow**: Partial/In-progress features
- **🟠 Orange**: Framework/Structure exists
- **🔴 Red**: Missing/Required features
- **🔵 Blue**: UI/UX components
- **🟣 Purple**: Audio/MIDI systems

### Diagram Maintenance
- All diagrams must reflect **ACTUAL CURRENT STATE**
- Update colors based on implementation status
- Add new diagrams for new features
- Remove obsolete diagrams
- Test mermaid rendering before committing

---

**⚠️ CRITICAL**: This document contains the **AUTHORITATIVE REFERENCE** for all elegant mermaid diagrams. Any changes to project architecture must be reflected here first.

*Last Updated: 2025-11-30 - Based on actual code analysis and current implementation state*