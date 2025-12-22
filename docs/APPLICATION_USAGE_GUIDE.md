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
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │
│        │                                    │               │