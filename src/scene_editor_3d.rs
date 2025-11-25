use bevy::prelude::*;
use bevy::render::camera::{Camera, Projection};
use bevy::render::primitives::Frustum;
use bevy::render::view::VisibleEntities;

/// 3D Scene Editor inspired by space_editor patterns
/// Provides gizmo-based manipulation, scene hierarchy, and 3D viewport management

#[derive(Resource, Default)]
pub struct SceneEditor3DState {
    pub enabled: bool,
    pub show_gizmos: bool,
    pub selected_entity: Option<Entity>,
    pub manipulation_mode: ManipulationMode,
    pub camera_entity: Option<Entity>,
    pub create_primitive_type: PrimitiveType,
    pub snap_to_grid: bool,
    pub grid_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManipulationMode {
    Translate,
    Rotate,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Cube,
    Sphere,
    Cylinder,
    Plane,
}

impl Default for ManipulationMode {
    fn default() -> Self {
        Self::Translate
    }
}

impl Default for PrimitiveType {
    fn default() -> Self {
        Self::Cube
    }
}

/// Component to mark entities that can be manipulated in the 3D editor
#[derive(Component)]
pub struct EditorManipulable;

/// Component to mark the editor camera
#[derive(Component)]
pub struct EditorCamera3D;

/// Plugin for 3D scene editing capabilities
pub struct SceneEditor3DPlugin;

impl Plugin for SceneEditor3DPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneEditor3DState>()
            .add_systems(Startup, setup_editor_3d)
            .add_systems(Update, (
                editor_3d_input_system,
                gizmo_manipulation_system,
                update_editor_camera,
                highlight_selected_entity,
                create_primitive_system,
                snap_to_grid_system,
            ));
    }
}

/// Setup the 3D editor with camera and basic scene
fn setup_editor_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut editor_state: ResMut<SceneEditor3DState>,
) {
    // Create editor camera
    let camera_entity = commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(5.0, 5.0, 5.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            ..default()
        },
        VisibleEntities::default(),
        Frustum::default(),
        Camera::default(),
        EditorCamera3D,
        Name::new("Editor Camera"),
    )).id();
    
    editor_state.camera_entity = Some(camera_entity);
    editor_state.enabled = true;
    editor_state.show_gizmos = true;
    editor_state.create_primitive_type = PrimitiveType::Cube;
    editor_state.snap_to_grid = false;
    editor_state.grid_size = 1.0;
    
    // Add some default lighting
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(
            Vec3::NEG_Z,
            Vec3::new(-1.0, -1.0, -1.0).normalize(),
        )),
        Name::new("Editor Directional Light"),
    ));
    
    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });
    
    // Create a default manipulable cube for testing
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        EditorManipulable,
        Name::new("Editor Cube"),
    ));
}

/// Handle input for 3D editor
fn editor_3d_input_system(
    mut editor_state: ResMut<SceneEditor3DState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera3D>>,
    manipulable_query: Query<(Entity, &GlobalTransform, Option<&Aabb>), With<EditorManipulable>>,
) {
    if !editor_state.enabled {
        return;
    }
    
    // Change manipulation mode with hotkeys
    if key_input.just_pressed(KeyCode::KeyW) {
        editor_state.manipulation_mode = ManipulationMode::Translate;
        println!("Switched to Translate mode");
    } else if key_input.just_pressed(KeyCode::KeyE) {
        editor_state.manipulation_mode = ManipulationMode::Rotate;
        println!("Switched to Rotate mode");
    } else if key_input.just_pressed(KeyCode::KeyR) {
        editor_state.manipulation_mode = ManipulationMode::Scale;
        println!("Switched to Scale mode");
    }
    
    // Handle entity selection with mouse
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.get_single() {
            if let Ok((camera, camera_transform)) = camera_query.get_single() {
                if let Some(cursor_position) = window.cursor_position() {
                    // Cast ray from camera to select entity
                    if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                        let mut closest_entity = None;
                        let mut closest_distance = f32::MAX;
                        
                        for (entity, transform, aabb) in manipulable_query.iter() {
                            let entity_pos = transform.translation();
                            let selection_radius = if let Some(aabb) = aabb {
                                // Use AABB half-extents for more accurate selection
                                aabb.half_extents().length()
                            } else {
                                // Default selection radius
                                0.5
                            };
                            
                            let distance_to_ray = ray.intersect_sphere(entity_pos, selection_radius);
                            
                            if let Some(distance) = distance_to_ray {
                                if distance < closest_distance && distance > 0.0 {
                                    closest_distance = distance;
                                    closest_entity = Some(entity);
                                }
                            }
                        }
                        
                        editor_state.selected_entity = closest_entity;
                        if let Some(entity) = closest_entity {
                            println!("Selected entity: {:?}", entity);
                        }
                    }
                }
            }
        }
    }
}

/// Gizmo manipulation system for selected entities
fn gizmo_manipulation_system(
    mut editor_state: ResMut<SceneEditor3DState>,
    mut manipulable_query: Query<&mut Transform, With<EditorManipulable>>,
    mouse_motion: Res<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if !editor_state.enabled {
        return;
    }
    
    if let Some(selected_entity) = editor_state.selected_entity {
        if let Ok(mut transform) = manipulable_query.get_mut(selected_entity) {
            // Only manipulate when left mouse is held down
            if mouse_button.pressed(MouseButton::Left) {
                let delta = mouse_motion.delta();
                
                match editor_state.manipulation_mode {
                    ManipulationMode::Translate => {
                        // Simple translation based on mouse movement
                        let translation_delta = Vec3::new(delta.x * 0.01, -delta.y * 0.01, 0.0);
                        transform.translation += translation_delta;
                    }
                    ManipulationMode::Rotate => {
                        // Simple rotation based on mouse movement
                        let rotation_delta = Vec3::new(-delta.y * 0.01, -delta.x * 0.01, 0.0);
                        transform.rotate_local(rotation_delta);
                    }
                    ManipulationMode::Scale => {
                        // Simple scale based on mouse movement
                        let scale_delta = 1.0 + (delta.x + delta.y) * 0.01;
                        transform.scale *= scale_delta;
                    }
                }
            }
        }
    }
}

/// Update editor camera with pan/orbit controls
fn update_editor_camera(
    mut camera_query: Query<(&mut Transform, &mut Projection), With<EditorCamera3D>>,
    mouse_motion: Res<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    time: Res<Time>,
) {
    if let Ok((mut transform, _projection)) = camera_query.get_single_mut() {
        // Orbit camera with right mouse button
        if mouse_button.pressed(MouseButton::Right) {
            let delta = mouse_motion.delta();
            
            // Orbit around origin (simplified)
            let radius = transform.translation.length();
            let mut spherical_coords = cartesian_to_spherical(transform.translation);
            
            spherical_coords.y += delta.x * 0.01; // Azimuth
            spherical_coords.z += delta.y * 0.01; // Elevation
            spherical_coords.z = spherical_coords.z.clamp(-1.5, 1.5); // Limit elevation
            
            transform.translation = spherical_to_cartesian(spherical_coords, radius);
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
        
        // Pan camera with middle mouse button
        if mouse_button.pressed(MouseButton::Middle) {
            let delta = mouse_motion.delta();
            let pan_speed = 0.1;
            
            let right = transform.right();
            let up = transform.up();
            
            transform.translation += right * -delta.x * pan_speed;
            transform.translation += up * delta.y * pan_speed;
        }
        
        // Zoom with mouse wheel
        for wheel_event in mouse_wheel.read() {
            let zoom_speed = 0.1;
            let zoom_factor = 1.0 - wheel_event.y * zoom_speed;
            transform.translation *= zoom_factor;
        }
        
        // Zoom with keyboard keys (Q/Z)
        if key_input.pressed(KeyCode::KeyQ) {
            transform.translation *= 1.02; // Zoom out
        }
        if key_input.pressed(KeyCode::KeyZ) {
            transform.translation *= 0.98; // Zoom in
        }
    }
}

/// Highlight selected entity with gizmos
fn highlight_selected_entity(
    editor_state: Res<SceneEditor3DState>,
    manipulable_query: Query<&GlobalTransform, With<EditorManipulable>>,
    mut gizmos: Gizmos,
) {
    if !editor_state.enabled || !editor_state.show_gizmos {
        return;
    }
    
    if let Some(selected_entity) = editor_state.selected_entity {
        if let Ok(transform) = manipulable_query.get(selected_entity) {
            let pos = transform.translation();
            
            // Draw selection highlight box
            gizmos.cuboid(
                Transform::from_translation(pos).with_scale(Vec3::splat(1.2)),
                Color::YELLOW,
            );
            
            // Draw manipulation gizmo based on current mode
            match editor_state.manipulation_mode {
                ManipulationMode::Translate => {
                    // Draw translation axes
                    gizmos.arrow(pos, pos + Vec3::X * 0.5, Color::RED);
                    gizmos.arrow(pos, pos + Vec3::Y * 0.5, Color::GREEN);
                    gizmos.arrow(pos, pos + Vec3::Z * 0.5, Color::BLUE);
                }
                ManipulationMode::Rotate => {
                    // Draw rotation circles
                    gizmos.circle(pos, Direction3d::X, 0.3, Color::RED);
                    gizmos.circle(pos, Direction3d::Y, 0.3, Color::GREEN);
                    gizmos.circle(pos, Direction3d::Z, 0.3, Color::BLUE);
                }
                ManipulationMode::Scale => {
                    // Draw scale handles
                    gizmos.cuboid(
                        Transform::from_translation(pos + Vec3::X * 0.4).with_scale(Vec3::splat(0.1)),
                        Color::RED,
                    );
                    gizmos.cuboid(
                        Transform::from_translation(pos + Vec3::Y * 0.4).with_scale(Vec3::splat(0.1)),
                        Color::GREEN,
                    );
                    gizmos.cuboid(
                        Transform::from_translation(pos + Vec3::Z * 0.4).with_scale(Vec3::splat(0.1)),
                        Color::BLUE,
                    );
                }
            }
        }
    }
}

// Helper functions for spherical coordinates
#[derive(Debug, Clone, Copy)]
struct SphericalCoords {
    x: f32, // radius
    y: f32, // azimuth (horizontal angle)
    z: f32, // elevation (vertical angle)
}

fn cartesian_to_spherical(cartesian: Vec3) -> SphericalCoords {
    let radius = cartesian.length();
    let azimuth = cartesian.z.atan2(cartesian.x);
    let elevation = cartesian.y.asin();
    
    SphericalCoords {
        x: radius,
        y: azimuth,
        z: elevation,
    }
}

fn spherical_to_cartesian(spherical: SphericalCoords, radius: f32) -> Vec3 {
    Vec3::new(
        radius * spherical.z.cos() * spherical.y.cos(),
        radius * spherical.z.sin(),
        radius * spherical.z.cos() * spherical.y.sin(),
    )
}

/// UI system for 3D scene editor panel
pub fn scene_editor_3d_ui(
    mut egui_ctx: bevy_egui::EguiContexts,
    mut editor_state: ResMut<SceneEditor3DState>,
    manipulable_query: Query<(Entity, &Name), With<EditorManipulable>>,
) {
    use bevy_egui::egui;
    
    let ctx = match egui_ctx.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };
    
    egui::Window::new("3D Scene Editor")
        .default_pos([10.0, 50.0])
        .default_size([300.0, 400.0])
        .show(ctx, |ui| {
            ui.heading("3D Scene Controls");
            ui.separator();
            
            // Manipulation mode buttons
            ui.horizontal(|ui| {
                ui.label("Mode:");
                if ui.button("Translate (W)").clicked() {
                    editor_state.manipulation_mode = ManipulationMode::Translate;
                }
                if ui.button("Rotate (E)").clicked() {
                    editor_state.manipulation_mode = ManipulationMode::Rotate;
                }
                if ui.button("Scale (R)").clicked() {
                    editor_state.manipulation_mode = ManipulationMode::Scale;
                }
            });
            
            ui.separator();
            
            // Primitive creation
            ui.horizontal(|ui| {
                ui.label("Create:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", editor_state.create_primitive_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut editor_state.create_primitive_type, PrimitiveType::Cube, "Cube");
                        ui.selectable_value(&mut editor_state.create_primitive_type, PrimitiveType::Sphere, "Sphere");
                        ui.selectable_value(&mut editor_state.create_primitive_type, PrimitiveType::Cylinder, "Cylinder");
                        ui.selectable_value(&mut editor_state.create_primitive_type, PrimitiveType::Plane, "Plane");
                    });
                ui.label("(Ctrl+N)");
            });
            
            ui.separator();
            
            // Scene hierarchy
            ui.heading("Scene Hierarchy");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (entity, name) in manipulable_query.iter() {
                        let is_selected = editor_state.selected_entity == Some(entity);
                        let response = ui.selectable_label(
                            is_selected,
                            format!("{} (Entity {:?})", name.as_str(), entity)
                        );
                        
                        if response.clicked() {
                            editor_state.selected_entity = Some(entity);
                        }
                    }
                });
            
            ui.separator();
            
            // Editor options
            ui.checkbox(&mut editor_state.show_gizmos, "Show Gizmos");
            ui.checkbox(&mut editor_state.enabled, "Editor Enabled");
            ui.checkbox(&mut editor_state.snap_to_grid, "Snap to Grid");
            
            if editor_state.snap_to_grid {
                ui.horizontal(|ui| {
                    ui.label("Grid Size:");
                    ui.add(egui::DragValue::new(&mut editor_state.grid_size)
                        .speed(0.1)
                        .clamp_range(0.1..=10.0));
                });
                ui.label("Press G to snap selected entities");
            }
            
            ui.separator();
            
            // Instructions
            ui.label("Controls:");
            ui.label("• Left Click: Select entity");
            ui.label("• Right Drag: Orbit camera");
            ui.label("• Middle Drag: Pan camera");
            ui.label("• Mouse Wheel: Zoom in/out");
            ui.label("• Q/Z: Zoom out/in");
            ui.label("• W/E/R: Switch manipulation mode");
            ui.label("• Ctrl+N: Create new primitive");
            ui.label("• G: Snap to grid (when enabled)");
        });
}

/// System to create new primitives in the scene
fn create_primitive_system(
    mut commands: Commands,
    mut editor_state: ResMut<SceneEditor3DState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    if !editor_state.enabled {
        return;
    }
    
    // Create primitive with Ctrl+N
    if key_input.pressed(KeyCode::ControlLeft) && key_input.just_pressed(KeyCode::KeyN) {
        let primitive_mesh = match editor_state.create_primitive_type {
            PrimitiveType::Cube => meshes.add(Cuboid::default()),
            PrimitiveType::Sphere => meshes.add(Sphere::new(0.5)),
            PrimitiveType::Cylinder => meshes.add(Cylinder::new(0.5, 1.0)),
            PrimitiveType::Plane => meshes.add(Plane3d::default()),
        };
        
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            ..default()
        });
        
        commands.spawn((
            Mesh3d(primitive_mesh),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            EditorManipulable,
            Name::new(format!("{:?}", editor_state.create_primitive_type)),
        ));
        
        println!("Created {:?}", editor_state.create_primitive_type);
    }
}

/// System to snap entities to grid
fn snap_to_grid_system(
    mut transforms: Query<&mut Transform, With<EditorManipulable>>,
    editor_state: Res<SceneEditor3DState>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    if !editor_state.enabled || !editor_state.snap_to_grid {
        return;
    }
    
    // Snap selected entities to grid with G key
    if key_input.just_pressed(KeyCode::KeyG) {
        for mut transform in transforms.iter_mut() {
            let grid_size = editor_state.grid_size;
            
            // Snap translation to grid
            transform.translation.x = (transform.translation.x / grid_size).round() * grid_size;
            transform.translation.y = (transform.translation.y / grid_size).round() * grid_size;
            transform.translation.z = (transform.translation.z / grid_size).round() * grid_size;
        }
        
        println!("Snapped entities to grid (size: {})", editor_state.grid_size);
    }
}

/// 3D viewport panel that can be embedded in the main editor
pub fn scene_3d_viewport_ui(
    mut egui_ctx: bevy_egui::EguiContexts,
    editor_state: &crate::scene_editor_3d::SceneEditor3DState,
) {
    use bevy_egui::egui;
    
    let ctx = match egui_ctx.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };
    
    egui::Window::new("3D Viewport")
        .default_pos([320.0, 50.0])
        .default_size([600.0, 400.0])
        .show(ctx, |ui| {
            ui.heading("3D Scene View");
            
            if editor_state.enabled {
                ui.label("3D viewport active - use mouse controls to navigate");
                ui.label(format!("Selected: {:?}", editor_state.selected_entity));
                ui.label(format!("Mode: {:?}", editor_state.manipulation_mode));
            } else {
                ui.label("3D editor disabled");
            }
            
            // Placeholder for actual 3D viewport rendering
            // In a full implementation, this would render the 3D scene to a texture
            // and display it in the egui window
            ui.separator();
            ui.label("Note: Full 3D viewport integration requires render-to-texture setup");
        });
}