# WGSL Shader Studio Application Usage Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Installation and Setup](#installation-and-setup)
3. [Getting Started](#getting-started)
4. [User Interface Overview](#user-interface-overview)
5. [Creating Shaders](#creating-shaders)
6. [Editing Shaders](#editing-shaders)
7. [Converting Shaders](#converting-shaders)
8. [Testing Shaders](#testing-shaders)
9. [Debugging Shaders](#debugging-shaders)
10. [Node-Based Editor](#node-based-editor)
11. [3D Scene Editor](#3d-scene-editor)
12. [Audio Integration](#audio-integration)
13. [MIDI Control](#midi-control)
14. [OSC Integration](#osc-integration)
15. [Timeline Animation](#timeline-animation)
16. [Exporting and Sharing](#exporting-and-sharing)
17. [Performance Profiling](#performance-profiling)
18. [Troubleshooting](#troubleshooting)

## Introduction

WGSL Shader Studio is a comprehensive development environment for creating, editing, testing, and converting shaders across multiple graphics APIs and shading languages. This guide provides detailed instructions on how to use all features of the application effectively.

The application supports:
- WebGPU Shading Language (WGSL)
- OpenGL Shading Language (GLSL)
- High-Level Shading Language (HLSL)
- Interactive Shader Format (ISF)
- Real-time preview and testing
- Node-based shader composition
- 3D scene editing
- Audio/MIDI/OSC integration
- Timeline animation
- Cross-platform shader conversion

## Installation and Setup

### System Requirements

- **Operating System**: Windows 10/11, macOS 10.15+, Ubuntu 20.04+
- **Graphics**: DirectX 12, Vulkan 1.2, or Metal support
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 2GB available space
- **WebGPU Support**: Compatible browser or native WebGPU implementation

### Installation Steps

1. **Download the Application**
   - Visit the official repository or download page
   - Download the appropriate version for your operating system

2. **Install Dependencies**
   ```bash
   # Install Rust toolchain
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install system dependencies (Ubuntu/Debian)
   sudo apt-get update
   sudo apt-get install cmake pkg-config libfreetype6-dev libfontconfig1-dev libxcb1-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libxss-dev
   ```

3. **Build from Source**
   ```bash
   git clone https://github.com/your-repo/wgsl-shader-studio.git
   cd wgsl-shader-studio
   cargo build --release
   ```

4. **Run the Application**
   ```bash
   cargo run --release
   ```

### Initial Configuration

Upon first launch, the application will:
1. Detect available graphics adapters
2. Configure WebGPU backend
3. Set up default workspace
4. Initialize documentation server

## Getting Started

### First Launch

When you first launch WGSL Shader Studio:

1. **Welcome Screen**: Review the welcome message and quick start guide
2. **Workspace Selection**: Choose or create a workspace directory
3. **Template Selection**: Select a starter template:
   - Empty Project
   - Basic 2D Shader
   - 3D Scene Template
   - ISF Effect Template
   - Compute Shader Example

### Quick Start Tutorial

1. **Create New Project**
   - File → New Project
   - Select "Basic 2D Shader"
   - Name your project and choose location

2. **Explore Interface**
   - Main editor window
   - Preview panel
   - Properties panel
   - Node graph (if applicable)

3. **Modify Shader**
   - Open the default shader file
   - Make a simple change (e.g., change color value)
   - Observe real-time preview update

4. **Test Export**
   - File → Export → WGSL
   - Save to desired location

## User Interface Overview

### Main Window Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Menu Bar                                                    │
├─────────────────────────────────────────────────────────────┤
│ Toolbar                                                     │
├────────┬────────────────────────────────────┬───────────────┤
│        │                                    │               │
│Project │          Editor Area               │   Properties  │
│Explorer│                                    │     Panel     │
│        │                                    │               │
├────────┼────────────────────────────────────┼───────────────┤
│        │                                    │               │
│        │          Preview Panel             │   Inspector   │
│        │                                    │               │
└────────┴────────────────────────────────────┴───────────────┘
Status Bar
```

### Menu Bar

The menu bar provides access to all major application features:

- **File**: New, Open, Save, Export, Import
- **Edit**: Undo, Redo, Cut, Copy, Paste
- **View**: Toggle panels, Zoom, Fullscreen
- **Shader**: Create, Convert, Validate, Optimize
- **Tools**: Node Editor, 3D Editor, Timeline
- **Window**: Manage layouts, Panels
- **Help**: Documentation, About, Report Issue

### Toolbar

The toolbar provides quick access to frequently used commands:

- New Shader
- Open Project
- Save
- Undo/Redo
- Compile Shader
- Run Preview
- Debug Shader
- Export

### Panels

1. **Project Explorer**: Shows project files and folders
2. **Editor Area**: Main code editing area with syntax highlighting
3. **Properties Panel**: Displays and edits shader properties
4. **Preview Panel**: Real-time shader preview
5. **Inspector**: Detailed inspection of shader elements
6. **Console**: Compilation output and error messages

## Creating Shaders

### New Shader Wizard

1. **Access Wizard**
   - Click "New Shader" button in toolbar
   - Or use File → New → Shader

2. **Select Shader Type**
   - Vertex Shader
   - Fragment/Pixel Shader
   - Compute Shader
   - Geometry Shader
   - Hull/Domain Shader
   - ISF Effect

3. **Configure Settings**
   - Language (WGSL/GLSL/HLSL/ISF)
   - Target API (WebGPU/DirectX/Vulkan/OpenGL)
   - Entry Point Name
   - Template Selection

4. **Generate Boilerplate**
   - The wizard creates appropriate boilerplate code
   - Sets up required imports/includes
   - Configures basic structure

### Manual Creation

1. **Create New File**
   - Right-click in Project Explorer
   - Select New → File
   - Enter filename with appropriate extension (.wgsl, .glsl, .hlsl, .fs)

2. **Add Basic Structure**
   ```wgsl
   // Example WGSL shader
   @group(0) @binding(0) var<uniform> time: f32;
   
   @vertex
   fn vertexMain(@builtin(vertex_index) vertexIndex: u32) -> @builtin(position) vec4<f32> {
       let pos = array(
           vec2(-0.5, -0.5),
           vec2( 0.5, -0.5),
           vec2( 0.0,  0.5)
       );
       return vec4(pos[vertexIndex], 0.0, 1.0);
   }
   
   @fragment
   fn fragmentMain() -> @location(0) vec4<f32> {
       return vec4(1.0, 0.0, 0.0, 1.0);
   }
   ```

## Editing Shaders

### Code Editor Features

1. **Syntax Highlighting**
   - Language-specific coloring
   - Bracket matching
   - Line numbering

2. **IntelliSense**
   - Auto-completion
   - Parameter hints
   - Error detection

3. **Refactoring Tools**
   - Rename symbols
   - Extract functions
   - Format code

4. **Navigation**
   - Go to definition
   - Find references
   - Symbol search

### Live Preview

1. **Real-time Compilation**
   - Automatic compilation on save
   - Error highlighting in editor
   - Performance indicators

2. **Preview Controls**
   - Play/Pause animation
   - Adjust time parameters
   - Change resolution
   - Toggle fullscreen

3. **Debug Visualization**
   - Intermediate render targets
   - Value inspection
   - Performance profiling

### Parameter Editing

1. **Uniform Inspector**
   - Edit uniform values in real-time
   - Visual sliders for numeric values
   - Color pickers for vector values
   - File selectors for textures

2. **Hot Reload**
   - Changes applied immediately
   - State preservation during reload
   - Error rollback on invalid changes

## Converting Shaders

### Conversion Wizard

1. **Access Converter**
   - Tools → Convert Shader
   - Right-click shader file → Convert

2. **Select Conversion Type**
   - WGSL ↔ GLSL
   - WGSL ↔ HLSL
   - GLSL ↔ HLSL
   - ISF ↔ Other formats

3. **Configure Options**
   - Target language version
   - Optimization level
   - Resource binding mapping
   - Extension handling

4. **Review and Apply**
   - Preview converted code
   - Compare with original
   - Apply conversion

### Batch Conversion

1. **Select Multiple Files**
   - Ctrl+Click or Shift+Click in Project Explorer
   - Select all with Ctrl+A

2. **Batch Convert**
   - Right-click selection
   - Choose Convert Selected
   - Configure batch options

3. **Monitor Progress**
   - Conversion progress dialog
   - Error reporting
   - Summary report

### Conversion Validation

1. **Automatic Validation**
   - Syntax checking
   - Semantic analysis
   - Resource binding verification

2. **Manual Verification**
   - Side-by-side comparison
   - Functionality testing
   - Performance benchmarking

## Testing Shaders

### Test Framework

1. **Unit Tests**
   - Create test cases for shader functions
   - Define input/output pairs
   - Automated execution

2. **Integration Tests**
   - Test shader in complete pipeline
   - Verify rendering output
   - Performance benchmarks

3. **Regression Tests**
   - Compare output with baseline
   - Detect unintended changes
   - Version tracking

### Test Runner

1. **Execute Tests**
   - Run individual tests
   - Run test suites
   - Continuous testing mode

2. **View Results**
   - Pass/fail indicators
   - Performance metrics
   - Error details

3. **Generate Reports**
   - Export test results
   - Create HTML reports
   - Integrate with CI/CD

### Debugging Shaders

### Debug Tools

1. **Shader Debugger**
   - Step through shader execution
   - Inspect variable values
   - Set breakpoints

2. **Frame Capture**
   - Capture render pipeline state
   - Analyze draw calls
   - Inspect resources

3. **Performance Profiler**
   - GPU time measurement
   - Resource usage tracking
   - Bottleneck identification

### Debug Visualization

1. **Intermediate Targets**
   - View render targets at each stage
   - Inspect MIP levels
   - Analyze format conversion

2. **Value Inspection**
   - Hover over variables for values
   - Pin variables for continuous monitoring
   - Export inspection data

3. **Error Highlighting**
   - Syntax errors with line markers
   - Runtime errors with stack traces
   - Performance warnings

## Node-Based Editor

### Node Graph Interface

1. **Canvas Navigation**
   - Pan with middle mouse button
   - Zoom with scroll wheel
   - Fit to view with double-click

2. **Node Operations**
   - Drag nodes to reposition
   - Connect outputs to inputs
   - Group related nodes

3. **Node Library**
   - Math operations
   - Texture sampling
   - Lighting models
   - Custom functions

### Creating Node Networks

1. **Add Nodes**
   - Drag from library panel
   - Right-click canvas → Add Node
   - Search for specific nodes

2. **Connect Nodes**
   - Drag from output to input
   - Automatic type matching
   - Visual connection lines

3. **Configure Nodes**
   - Double-click to edit properties
   - Inline parameter editing
   - Preset selection

### Node Compilation

1. **Graph to Code**
   - Automatic shader generation
   - Optimization of node networks
   - Comment generation

2. **Code Integration**
   - Generated code editable
   - Mixed node/code workflows
   - Version control friendly

## 3D Scene Editor

### Scene Hierarchy

1. **Object Management**
   - Add/remove objects
   - Parent/child relationships
   - Transform hierarchy

2. **Component System**
   - Mesh renderer
   - Material properties
   - Light sources
   - Cameras

3. **Scene Navigation**
   - Orbit, pan, zoom controls
   - Object selection
   - Gizmo manipulation

### Material Editor

1. **Property Editing**
   - Albedo color
   - Metallic/roughness
   - Normal mapping
   - Emission

2. **Texture Assignment**
   - Drag/drop texture files
   - UV coordinate editing
   - Tiling/offset controls

3. **Shader Assignment**
   - Assign custom shaders
   - Override material properties
   - Multi-pass materials

### Lighting System

1. **Light Types**
   - Directional lights
   - Point lights
   - Spot lights
   - Area lights

2. **Light Properties**
   - Color and intensity
   - Shadows
   - Attenuation
   - Cookies

3. **Environment Lighting**
   - Skyboxes
   - Image-based lighting
   - Ambient lighting

## Audio Integration

### Audio Sources

1. **File Input**
   - Load audio files (WAV, MP3, OGG)
   - Real-time playback control
   - Looping options

2. **Device Input**
   - Microphone/capture device
   - System audio capture
   - Latency settings

3. **Generated Audio**
   - Procedural sound generation
   - Synthesizer integration
   - MIDI input

### Audio Processing

1. **FFT Analysis**
   - Real-time frequency analysis
   - Band separation
   - Beat detection

2. **Audio Parameters**
   - Volume/RMS
   - Frequency spectrum
   - Beat timing

3. **Visualization**
   - Waveform display
   - Spectrum analyzer
   - Beat indicator

### Audio-Reactive Shaders

1. **Parameter Mapping**
   - Map audio to shader uniforms
   - Frequency band mapping
   - Trigger events

2. **Synchronization**
   - BPM synchronization
   - Beat-matched transitions
   - Audio timeline

## MIDI Control

### MIDI Device Support

1. **Device Detection**
   - Automatic device discovery
   - Hot-plug support
   - Device configuration

2. **Input Mapping**
   - Note on/off events
   - Control change messages
   - Pitch bend
   - Program change

3. **Output Feedback**
   - Send MIDI messages
   - Control external devices
   - Synchronize with DAWs

### MIDI Mapping

1. **Parameter Binding**
   - Map MIDI controls to shader parameters
   - Learn mode for easy mapping
   - Save/load mappings

2. **Mapping Editor**
   - Visual mapping interface
   - Range scaling
   - Curve editing

3. **Preset Management**
   - Save MIDI mappings as presets
   - Organize mapping collections
   - Share with others

## OSC Integration

### OSC Setup

1. **Network Configuration**
   - Set IP addresses and ports
   - Configure sending/receiving
   - Network security

2. **Message Routing**
   - Define OSC address patterns
   - Map to shader parameters
   - Filter incoming messages

3. **Protocol Support**
   - OSC 1.0 compliance
   - Bundle support
   - Timestamp handling

### OSC Controls

1. **Remote Control**
   - Control from external applications
   - TouchOSC integration
   - Tablet/mobile support

2. **Bidirectional Communication**
   - Send status updates
   - Feedback to controllers
   - Parameter synchronization

3. **Scripting Interface**
   - Custom OSC handlers
   - Message processing scripts
   - Automation sequences

## Timeline Animation

### Timeline Editor

1. **Track Management**
   - Add/remove animation tracks
   - Organize track hierarchy
   - Track grouping

2. **Keyframe Editing**
   - Insert keyframes
   - Adjust timing
   - Interpolation modes

3. **Curve Editing**
   - Bezier curve manipulation
   - Ease in/out controls
   - Custom curve shapes

### Animation Features

1. **Parameter Animation**
   - Animate any shader parameter
   - Multiple parameter tracks
   - Expression-based animation

2. **Sequencing**
   - Arrange clips on timeline
   - Crossfades and transitions
   - Loop regions

3. **Synchronization**
   - Audio sync
   - MIDI sync
   - External timecode

### Playback Controls

1. **Transport Controls**
   - Play, pause, stop
   - Scrubbing
   - Loop playback

2. **Time Display**
   - Current time indicator
   - Time format selection
   - Marker navigation

3. **Rendering**
   - Export animations
   - Frame-by-frame rendering
   - Video encoding

## Exporting and Sharing

### Export Formats

1. **Shader Code**
   - WGSL, GLSL, HLSL source
   - Minified versions
   - Header files

2. **Compiled Shaders**
   - SPIR-V bytecode
   - DXBC/DXIL
   - Metal IR

3. **Documentation**
   - HTML documentation
   - PDF manuals
   - API references

### Package Management

1. **Project Bundles**
   - Export entire projects
   - Dependency tracking
   - Version information

2. **Asset Collections**
   - Texture packs
   - Shader libraries
   - Preset collections

3. **Sharing Options**
   - Direct file export
   - Cloud storage integration
   - Version control export

### Integration Support

1. **Engine Integration**
   - Unity package
   - Unreal plugin
   - Custom engine support

2. **Framework Support**
   - Three.js modules
   - Babylon.js plugins
   - Custom framework templates

3. **Build Systems**
   - CMake integration
   - Makefile generation
   - Package managers

## Performance Profiling

### GPU Profiling

1. **Timing Analysis**
   - Shader execution time
   - Draw call overhead
   - State change costs

2. **Resource Usage**
   - Memory consumption
   - Bandwidth utilization
   - Cache efficiency

3. **Bottleneck Identification**
   - CPU vs GPU bound
   - Pipeline stalls
   - Resource contention

### Optimization Tools

1. **Performance Hints**
   - Automatic suggestions
   - Best practice recommendations
   - Platform-specific advice

2. **Code Analysis**
   - Static analysis
   - Complexity metrics
   - Optimization opportunities

3. **Benchmarking**
   - Frame rate measurement
   - Comparative testing
   - Regression detection

### Hardware Reporting

1. **System Information**
   - GPU specifications
   - Driver versions
   - Feature support

2. **Capability Queries**
   - Extension support
   - Limit queries
   - Format support

3. **Compatibility Reports**
   - Cross-platform compatibility
   - Feature level mapping
   - Fallback recommendations

## Troubleshooting

### Common Issues

1. **Compilation Errors**
   - Syntax errors
   - Type mismatches
   - Undefined identifiers

2. **Runtime Problems**
   - Black screens
   - Incorrect rendering
   - Performance issues

3. **Integration Failures**
   - Linking errors
   - Resource loading
   - Platform compatibility

### Diagnostic Tools

1. **Error Logs**
   - Detailed error messages
   - Stack traces
   - Context information

2. **Validation Layers**
   - API validation
   - Resource tracking
   - State monitoring

3. **Debug Rendering**
   - Wireframe mode
   - Depth visualization
   - Normal viewing

### Support Resources

1. **Documentation**
   - Online manuals
   - API references
   - Tutorial videos

2. **Community Support**
   - Forums
   - Discord channels
   - GitHub issues

3. **Professional Support**
   - Commercial licensing
   - Priority bug fixes
   - Custom development

---
*End of Application Usage Guide*