# WGSL Shader Studio - Professional WebGPU Shader Development

A comprehensive standalone WGPU shader studio with ISF (Interactive Shader Format) support. Professional shader development environment for VJ artists, developers, and creative coders.

## 🎯 **Project Status: COMPLETE & PRODUCTION READY**

### **What This Is**
- **Standalone WGPU Shader Studio**: Complete graphical development environment
- **ISF Shader Support**: Full Interactive Shader Format compatibility
- **Cross-Platform Shader Development**: WGSL/GLSL/HLSL conversion and editing
- **Professional VJ Tools**: Real-time audio analysis, MIDI control, live preview
- **Node-Based Visual Programming**: Drag-and-drop shader composition



## ✨ **Core Features**

### 🎨 **Visual Shader Development**
- **Live Preview System**: Real-time WGPU rendering viewport
- **WGSL Syntax Highlighting**: Advanced code editor with error detection
- **Node-Based Editor**: Visual programming interface for shader composition
- **Template Library**: 15+ categorized shader templates and examples

### 🔄 **Shader Format Support**
- **WGSL First-Class**: Native WebGPU shader language support
- **Cross-Platform Conversion**: WGSL ↔ GLSL ↔ HLSL bidirectional conversion
- **ISF Import/Export**: Full Interactive Shader Format compatibility
- **Batch Processing**: Multiple file conversion operations

### 🎵 **Audio & MIDI Integration**
- **Real-Time Audio Analysis**: FFT-based spectral analysis with beat detection
- **MIDI Control**: Full parameter mapping with smoothing and automation
- **Audio-Reactive Shaders**: Combined audio/MIDI parameter modulation
- **Performance Optimized**: Low-latency real-time processing

### 🛠️ **Professional Tools**
- **Advanced File Management**: Native OS dialogs with recent files
- **Performance Monitoring**: Real-time FPS tracking and optimization
- **Shader Visualizer**: AST visualization and dependency graphs
- **Context Menus & Shortcuts**: Full keyboard shortcut system

## 🚀 **Getting Started**

### Prerequisites
- Rust 1.70+
- WebGPU-compatible graphics card
- Optional: MIDI controller for parameter automation

### Building & Running

```bash
# Clone the repository
git clone https://github.com/compiling-org/wgsl-shader-studio.git
cd wgsl-shader-studio

# Build in release mode
cargo build --release

# Run the GUI application
cargo run --features gui -- --gui

# Or run CLI tools
cargo run -- list          # List available ISF shaders
cargo run -- validate file.fs  # Validate ISF shader
cargo run -- convert input.fs output.wgsl  # Convert shader formats
```

### GUI Features
```bash
# Enable GUI features
cargo run --features gui -- --gui
```

## 📚 **Documentation**

- **[Frontend Features Guide](docs/FRONTEND_FEATURES.md)**: Complete UI and feature documentation
- **[WGSL ↔ GLSL Guide](docs/GLSL_GUIDE.md)**: Shader format conversion reference
- **[Node Shader Guide](docs/NODE_SHADER_GUIDE.md)**: Visual programming tutorial

## 🎯 **Use Cases**

### For VJ Artists
- Create custom real-time visual effects
- Audio-reactive shader development
- Live performance shader manipulation
- Cross-software compatibility

### For Developers
- WebGPU shader prototyping
- Cross-platform shader development
- Shader format conversion tools
- Educational shader programming

### For Creative Coders
- Visual programming interface
- Real-time shader experimentation
- Template-based development
- Performance-optimized rendering

## 🔧 **Technical Specifications**

- **Rendering**: WebGPU (WGPU) with real-time preview
- **UI Framework**: egui/eframe for responsive interface
- **Audio Engine**: Real-time FFT analysis with MIDI integration
- **Shader Support**: WGSL, GLSL, HLSL, ISF formats
- **Performance**: 60+ FPS GUI, real-time shader compilation
- **Platform**: Windows, macOS, Linux

## 📈 **Performance Targets**

- **GUI FPS**: 60+ frames per second interface
- **Shader Compilation**: <100ms compilation times
- **Live Preview**: Real-time rendering at target resolution
- **Audio Latency**: <10ms audio analysis latency
- **Memory Usage**: Optimized for continuous operation

## 🤝 **Contributing**

This project is part of the broader compiling-org ecosystem. Contributions welcome for:
- Additional shader templates
- Performance optimizations
- New conversion formats
- UI/UX improvements
- Documentation enhancements

## 📄 **License**

MIT License - see LICENSE file for details

## 🙏 **Credits**

- **ISF Shaders**: Sleepless Monk collection
- **WGPU**: WebGPU implementation for Rust
- **egui**: Immediate mode GUI framework
- **rustfft**: Audio analysis library

---

**Ready for professional shader development and live VJ performance.**