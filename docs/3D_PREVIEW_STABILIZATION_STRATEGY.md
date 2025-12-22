# 3D Preview Stabilization Strategy Implementation

## Overview

This document describes the implementation of the strategy to isolate and stabilize the 3D editor preview functionality that was causing panics and blocking the core workflow.

## Changes Made

### 1. Feature Flag Implementation

A feature flag `3d_preview` has been added to the Cargo.toml to control the 3D preview functionality:

```toml
[features]
default = ["gui", "networking", "naga_integration"]
gui = ["dep:bevy_egui"]
networking = ["dep:tun-rs"]
naga_integration = []
3d_preview = []
```

### 2. Conditional Compilation

The 3D preview functionality is now conditionally compiled based on the feature flag:

1. **Plugin Registration**: The `SceneEditor3DPlugin` is only added to the app when the `3d_preview` feature is enabled.
2. **Resource Initialization**: 3D-related resources are only inserted when the feature is enabled.
3. **System Registration**: 3D-related systems are only registered when the feature is enabled.
4. **UI Components**: The 3D editor UI is displayed with a disabled message when the feature is not enabled.

### 3. Code Modifications

#### bevy_app.rs
- Added `ENABLE_3D_PREVIEW` constant based on feature flag
- Made plugin registration conditional
- Made resource insertion conditional
- Made system registration conditional
- Updated editor UI system parameters to be optional

#### editor_ui.rs
- Added `ENABLE_3D_PREVIEW` constant based on feature flag
- Made 3D editor UI conditional, showing a disabled message when feature is not enabled

### 4. New Files

#### minimal_3d_preview.rs
A minimal 3D preview implementation for stabilization testing:

- Simple plugin with basic 3D scene (cube, camera, light)
- Can be used to isolate 3D rendering issues
- Provides a clean baseline for debugging

#### debug_3d_panic.rs
Debug script to help identify the exact failure point:

- Sets environment variables for backtrace and logging
- Provides instructions for running with debugging enabled
- Documents what to look for in stack traces

## Usage

### Running Without 3D Preview (Stable Mode)
```bash
cargo run --features gui
```

This runs the application with all functionality except the 3D preview, ensuring no panics from the 3D systems.

### Running With 3D Preview (Experimental Mode)
```bash
cargo run --features gui,3d_preview
```

This enables the 3D preview functionality for testing and development.

## Debugging Panics

When debugging 3D preview panics, use:

```bash
RUST_BACKTRACE=1 RUST_LOG=wgsl_shader_studio=debug,wgpu=warn cargo run --features gui,3d_preview
```

Look for these specific failure patterns in the stack trace:

1. **wgpu pipeline panic**: Indicates mismatch between WGSL, bind group layouts, or vertex formats
2. **Bevy system panic**: Indicates missing resources, camera, or render targets
3. **Custom renderer assertion**: Indicates issues with viewport size, texture, or node graph binding

## Next Steps

1. Run the application with 3D preview enabled and capture backtraces
2. Analyze the exact failure point using the guidance above
3. Fix the specific subsystem that is failing
4. Stabilize 3D integration in layers:
   - Static 3D preview with hardcoded shader
   - Parameter injection only
   - Full shader swapping with defensive validation

## Benefits

This approach provides:

- A stable baseline for the core application functionality
- Isolated experimentation with 3D preview features
- Clear path to identify and fix specific issues
- No interference between 3D preview experiments and core workflow