# 3D Scene Editor Integration Guide

## Overview

This document provides comprehensive guidance for integrating the 3D scene editor into the WGSL Shader Studio panel system. The 3D scene editor enables interactive manipulation of 3D objects, real-time shader preview, and scene hierarchy management.

## Architecture

### Core Components

1. **Scene Editor 3D** (`src/scene_editor_3d.rs`)
   - Main 3D scene management system
   - Entity manipulation with gizmos
   - Camera controls and viewport management
   - Integration with Bevy's ECS architecture

2. **Scene 3D Manager** (`src/scene_3d.rs`)
   - Camera and lighting configuration
   - Scene settings and rendering parameters
   - 3D transformation utilities

3. **Screenshot/Video Export** (`src/screenshot_video_export.rs`)
   - Capture 3D scenes as images or videos
   - Multiple export formats (PNG, JPEG, MP4, WebM, GIF)
   - Real-time recording capabilities

### Panel System Integration

The 3D scene editor integrates with the existing panel system through:

- **EditorUiState**: Central state management with `show_3d_scene_panel` flag
- **UI Functions**: Modular panel functions following established patterns
- **System Scheduling**: Integration with Bevy's update systems

## Implementation Status

### ✅ Completed Features

- [x] Core 3D scene editor structure
- [x] Entity manipulation modes (Translate, Rotate, Scale)
- [x] Camera controls (Orbit, Pan, Zoom)
- [x] Scene hierarchy display
- [x] Gizmo-based object manipulation
- [x] Screenshot and video export capabilities
- [x] Panel system integration framework

### 🚧 In Progress

- [ ] Render-to-texture for 3D viewport
- [ ] Real-time shader preview on 3D objects
- [ ] Node system integration
- [ ] Advanced lighting controls

### 📋 Planned Features

- [ ] Material editor integration
- [ ] Animation timeline support
- [ ] Particle system integration
- [ ] Advanced camera controls (depth of field, etc.)

## Usage Guide

### Enabling the 3D Scene Editor

1. **Panel Toggle**: The 3D scene editor can be enabled through the View menu
2. **Keyboard Shortcut**: Configurable hotkey for quick access
3. **Auto-enable**: Set `show_3d_scene_panel: true` in `EditorUiState`

### Basic Operations

#### Entity Manipulation
- **Select**: Click on 3D objects in viewport
- **Translate**: Drag gizmo arrows to move objects
- **Rotate**: Drag gizmo circles to rotate objects
- **Scale**: Drag gizmo cubes to scale objects

#### Camera Controls
- **Orbit**: Middle mouse button + drag
- **Pan**: Right mouse button + drag
- **Zoom**: Mouse wheel
- **Reset**: Double-click to reset camera position

#### Scene Management
- **Add Objects**: Use scene hierarchy panel
- **Delete**: Select object and press Delete key
- **Duplicate**: Ctrl+D to duplicate selected objects
- **Group**: Ctrl+G to group selected objects

### Integration with Shader System

The 3D scene editor connects to the shader system through:

1. **Material Parameters**: Real-time parameter updates
2. **Geometry Input**: 3D meshes as shader input
3. **Lighting Data**: Scene lighting information
4. **Camera Matrices**: View/projection matrices

## Technical Details

### System Architecture

```rust
// Main 3D scene editor resource
pub struct SceneEditor3DState {
    pub manipulation_mode: ManipulationMode,
    pub selected_entity: Option<Entity>,
    pub camera_state: CameraState,
    pub scene_entities: Vec<Entity>,
    pub show_grid: bool,
    pub show_axes: bool,
}

// Manipulation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManipulationMode {
    Translate,
    Rotate,
    Scale,
}
```

### UI Integration Pattern

```rust
// Panel function integration
pub fn scene_editor_3d_panel(
    mut egui_ctx: bevy_egui::EguiContexts,
    mut ui_state: ResMut<EditorUiState>,
    scene_state: ResMut<SceneEditor3DState>,
) {
    if ui_state.show_3d_scene_panel {
        egui::Window::new("3D Scene Editor")
            .open(&mut ui_state.show_3d_scene_panel)
            .show(ctx, |ui| {
                // Scene editor UI content
            });
    }
}
```

### Bevy System Integration

```rust
// Add to app builder
app.add_plugins(SceneEditor3DPlugin)
   .add_systems(Update, scene_editor_3d_panel)
   .add_systems(Update, update_scene_gizmos)
   .add_systems(Update, handle_scene_interactions);
```

## Configuration

### Scene Settings

Configure the 3D scene through `Scene3DConfig`:

```rust
pub struct Scene3DConfig {
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub ambient_light: Color,
    pub directional_lights: Vec<DirectionalLightConfig>,
    pub show_grid: bool,
    pub grid_size: f32,
    pub show_axes: bool,
}
```

### Export Settings

Configure export parameters through `ExportSettings`:

```rust
pub struct ExportSettings {
    pub format: ExportFormat,    // PNG, JPEG, BMP, TIFF, WebP
    pub quality: u8,              // 1-100 for JPEG/WebP
    pub compression: u8,          // 1-9 for PNG
    pub width: u32,
    pub height: u32,
    pub premultiplied_alpha: bool,
}
```

## Troubleshooting

### Common Issues

1. **3D Viewport Not Rendering**
   - Check if `show_3d_scene_panel` is enabled
   - Verify graphics drivers are up to date
   - Ensure Bevy render plugins are properly configured

2. **Gizmo Manipulation Not Working**
   - Verify entity selection is working
   - Check manipulation mode is set correctly
   - Ensure gizmo rendering systems are active

3. **Export Functionality Issues**
   - Check file permissions for export directory
   - Verify sufficient disk space
   - Ensure image/video codecs are available

### Performance Optimization

- **Level of Detail**: Use simplified meshes for complex scenes
- **Culling**: Implement frustum culling for large scenes
- **Batching**: Group similar objects for efficient rendering
- **Texture Optimization**: Use appropriate texture sizes and formats

## Future Enhancements

### Advanced Features

1. **Real-time Ray Tracing**: Integration with Bevy's ray tracing capabilities
2. **Advanced Materials**: PBR material editor with node-based workflow
3. **Animation System**: Keyframe animation and timeline integration
4. **Physics Integration**: Collision detection and physics simulation
5. **VR/AR Support**: Virtual and augmented reality rendering

### Integration Roadmap

1. **Phase 1**: Basic 3D viewport and object manipulation ✅
2. **Phase 2**: Shader preview and real-time updates 🚧
3. **Phase 3**: Node system integration 📋
4. **Phase 4**: Advanced lighting and materials 📋
5. **Phase 5**: Animation and physics 📋

## Related Documentation

- [Panel System Architecture](UI_UX_DESIGN_GUIDE.md)
- [Visual Node Editor Integration](VISUAL_NODE_EDITOR_INTEGRATION_GUIDE.md)
- [Technical Architecture Reference](TECHNICAL_ARCHITECTURE_REFERENCE.md)
- [Complete Systems Reference](COMPLETE_SYSTEMS_REFERENCE.md)

## API Reference

See the inline documentation in:
- `src/scene_editor_3d.rs` - Main 3D scene editor implementation
- `src/scene_3d.rs` - 3D scene management utilities
- `src/screenshot_video_export.rs` - Export functionality