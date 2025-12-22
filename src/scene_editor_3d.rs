use bevy::prelude::*;
use bevy::prelude::Projection;
use bevy::prelude::PerspectiveProjection;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::asset::RenderAssetUsages;
use crate::bevy_app::Viewport3DTexture;
// use bevy::render::primitives::{Frustum, Aabb};
// use bevy::render::view::VisibleEntities;

/// 3D Scene Editor inspired by space_editor patterns
/// Provides gizmo-based manipulation, scene hierarchy, and 3D viewport management

#[derive(Resource)]
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

fn sync_shader_preview_texture_size(
    mut images: ResMut<Assets<Image>>,
    mut preview_tex: ResMut<ShaderPreviewTexture>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_query: Query<(Entity, &bevy::pbr::MeshMaterial3d<StandardMaterial>, &Name)>,
    ui_state: Res<crate::editor_ui::EditorUiState>,
) {
    // Ensure we have valid dimensions to prevent pixel data size mismatches
    // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
    let desired_w = ui_state.preview_resolution.0.max(50);
    let desired_h = ui_state.preview_resolution.1.max(50);
    
    // Additional safeguard against extremely small dimensions that could cause issues
    let desired_w = desired_w.max(100);
    let desired_h = desired_h.max(100);
    
    if preview_tex.width == desired_w && preview_tex.height == desired_h {
        return;
    }
    
    let size = Extent3d {
        width: desired_w,
        height: desired_h,
        depth_or_array_layers: 1,
    };
    println!("Creating preview texture with size: {}x{}", desired_w, desired_h);
    // Create valid pixel data with correct size using new_fill for safety
    let mut new_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    new_image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    let new_handle = images.add(new_image);
    preview_tex.handle = new_handle.clone();
    preview_tex.width = desired_w;
    preview_tex.height = desired_h;
    for (_e, mat_comp, name) in material_query.iter_mut() {
        if name.as_str() == "Shader Preview Quad" {
            if let Some(mat) = materials.get_mut(&mat_comp.0) {
                mat.base_color_texture = Some(new_handle.clone());
            }
        }
    }
}

pub fn sync_scene_viewport_texture_size(
    mut images: ResMut<Assets<Image>>,
    mut scene_viewport_tex: ResMut<SceneViewportTexture>,
    mut viewport_3d_texture: ResMut<Viewport3DTexture>,
) {
    if !viewport_3d_texture.needs_update {
        return;
    }

    let desired_w = viewport_3d_texture.width;
    let desired_h = viewport_3d_texture.height;
    
    println!("Syncing scene viewport texture: desired {}x{}", desired_w, desired_h);

    // Ensure we have valid dimensions to prevent pixel data size mismatches
    // Using larger minimum size to avoid Bevy 0.17 + bevy_egui issues
    let safe_width = desired_w.max(50);
    let safe_height = desired_h.max(50);
    
    // Additional safeguard against extremely small dimensions that could cause issues
    let safe_width = safe_width.max(100);
    let safe_height = safe_height.max(100);
    
    // Additional debugging
    println!("Final safe dimensions: {}x{}", safe_width, safe_height);
    
    println!("Safe dimensions: {}x{}", safe_width, safe_height);

    if scene_viewport_tex.width == safe_width && scene_viewport_tex.height == safe_height {
        viewport_3d_texture.needs_update = false;
        return;
    }

    let size = Extent3d {
        width: safe_width,
        height: safe_height,
        depth_or_array_layers: 1,
    };
    println!("Creating texture with size: {}x{}", safe_width, safe_height);
    // Create valid pixel data with correct size
    let expected_size = (safe_width * safe_height * 4) as usize;
    println!("Expected pixel data size: {}", expected_size);
    let initial_pixel_data = vec![0; expected_size];
    println!("Actual pixel data size: {}", initial_pixel_data.len());
    let mut new_image = Image::new(
        size,
        TextureDimension::D2,
        initial_pixel_data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    new_image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST;
    let new_handle = images.add(new_image);

    scene_viewport_tex.handle = new_handle.clone();
    scene_viewport_tex.width = safe_width;
    scene_viewport_tex.height = safe_height;
    viewport_3d_texture.needs_update = false;
}

impl Default for SceneEditor3DState {
    fn default() -> Self {
        Self {
            enabled: true,
            show_gizmos: true,
            selected_entity: None,
            manipulation_mode: ManipulationMode::default(),
            camera_entity: None,
            create_primitive_type: PrimitiveType::default(),
            snap_to_grid: false,
            grid_size: 1.0,
        }
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
            .init_resource::<SceneViewportTexture>()
            .init_resource::<ShaderPreviewTexture>()
            .add_systems(Startup, setup_editor_3d)
            .add_systems(Update, (
                editor_3d_input_system,
                gizmo_manipulation_system,
                update_editor_camera,
                highlight_selected_entity,
                create_primitive_system,
                snap_to_grid_system,
        update_shader_preview_texture,
        sync_shader_preview_texture_size,
        sync_scene_viewport_texture_size,
    ));
    }
}

#[derive(Resource, Clone)]
pub struct SceneViewportTexture {
    pub handle: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

impl Default for SceneViewportTexture {
    fn default() -> Self {
        Self {
            handle: Handle::default(),
            width: 512,
            height: 512,
        }
    }
}

#[derive(Resource, Clone)]
pub struct ShaderPreviewTexture {
    pub handle: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

impl Default for ShaderPreviewTexture {
    fn default() -> Self {
        Self {
            handle: Handle::default(),
            width: 512,
            height: 512,
        }
    }
}

/// Setup the 3D editor with camera and basic scene
fn setup_editor_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut editor_state: ResMut<SceneEditor3DState>,
    mut images: ResMut<Assets<Image>>,
    mut viewport_tex: ResMut<SceneViewportTexture>,
    mut preview_tex: ResMut<ShaderPreviewTexture>,
) {
    // Use consistent dimensions with the default texture size
    let size = Extent3d {
        width: 512,
        height: 512,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST;
    let image_handle = images.add(image);
    viewport_tex.handle = image_handle.clone();
    viewport_tex.width = size.width;
    viewport_tex.height = size.height;
    
    let mut preview_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    preview_image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    let preview_handle = images.add(preview_image);
    preview_tex.handle = preview_handle.clone();
    preview_tex.width = size.width;
    preview_tex.height = size.height;
    
    // Create editor camera
    let camera_entity = commands.spawn((
        Camera3d::default(),
        Camera { order: 1, target: image_handle.clone().into(), ..Default::default() },
        Transform::from_translation(Vec3::new(5.0, 5.0, 5.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            ..default()
        }),
        EditorCamera3D,
        Name::new("Editor Camera"),
    )).id();
    
    editor_state.camera_entity = Some(camera_entity);
    editor_state.enabled = true;
    editor_state.show_gizmos = true;
    editor_state.create_primitive_type = PrimitiveType::Cube;
    editor_state.snap_to_grid = false;
    editor_state.grid_size = 1.0;
    
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(preview_handle.clone()),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)).with_scale(Vec3::new(4.0, 2.25, 1.0)),
        Name::new("Shader Preview Quad"),
    ));
    
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
        affects_lightmapped_meshes: false,
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
    manipulable_query: Query<(Entity, &GlobalTransform), With<EditorManipulable>>,
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
        if let Ok(window) = windows.single() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                if let Some(cursor_position) = window.cursor_position() {
                    // Cast ray from camera to select entity
                    if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                        let mut closest_entity = None;
                        let mut closest_distance = f32::MAX;
                        
                        for (entity, transform) in manipulable_query.iter() {
                            let entity_pos = transform.translation();
                            let selection_radius = 0.5; // Default selection radius
                            
                            // Simple distance-based selection - check if entity is close to ray origin
                            let ray_origin = ray.origin;
                            let distance_to_ray = ray_origin.distance(entity_pos);
                            
                            if distance_to_ray < selection_radius && distance_to_ray < closest_distance {
                                closest_distance = distance_to_ray;
                                closest_entity = Some(entity);
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

fn update_shader_preview_texture(
    mut images: ResMut<Assets<Image>>,
    preview_tex: Res<ShaderPreviewTexture>,
    ui_state: Res<crate::editor_ui::EditorUiState>,
    audio_analyzer: Res<crate::audio_system::AudioAnalyzer>,
) {
    if let Ok(mut guard) = ui_state.global_renderer.renderer.lock() {
        if let Some(ref mut renderer) = *guard {
            let params = crate::shader_renderer::RenderParameters {
                width: preview_tex.width,
                height: preview_tex.height,
                time: ui_state.time as f32,
                frame_rate: 60.0,
                audio_data: Some(audio_analyzer.get_audio_data()),
            };
            if let Ok(pixels) = renderer.render_frame(&ui_state.draft_code, &params, params.audio_data.clone()) {
                if let Some(img) = images.get_mut(&preview_tex.handle) {
                    let expected_len = (preview_tex.width as usize) * (preview_tex.height as usize) * 4;
                    if pixels.len() == expected_len {
                        img.data = Some(pixels);
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
    mut mouse_motion_events: MessageReader<bevy::input::mouse::MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if !editor_state.enabled {
        return;
    }
    
    if let Some(selected_entity) = editor_state.selected_entity {
        if let Ok(mut transform) = manipulable_query.get_mut(selected_entity) {
            // Only manipulate when left mouse is held down
            if mouse_button.pressed(MouseButton::Left) {
                // Process mouse motion events
                let mut total_delta = Vec2::ZERO;
                for event in mouse_motion_events.read() {
                    total_delta += event.delta;
                }
                let delta = total_delta;
                
                match editor_state.manipulation_mode {
                    ManipulationMode::Translate => {
                        // Simple translation based on mouse movement
                        let translation_delta = Vec3::new(delta.x * 0.01, -delta.y * 0.01, 0.0);
                        transform.translation += translation_delta;
                    }
                    ManipulationMode::Rotate => {
                        // Simple rotation based on mouse movement
                        let rotation_delta = Quat::from_euler(
                            bevy::math::EulerRot::XYZ,
                            -delta.y * 0.01,
                            -delta.x * 0.01,
                            0.0
                        );
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
    mut mouse_motion_events: MessageReader<bevy::input::mouse::MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    time: Res<Time>,
) {
    if let Ok((mut transform, _projection)) = camera_query.single_mut() {
        // Orbit camera with right mouse button
        if mouse_button.pressed(MouseButton::Right) {
            // Process mouse motion events
            let mut total_delta = Vec2::ZERO;
            for event in mouse_motion_events.read() {
                total_delta += event.delta;
            }
            let delta = total_delta;
            
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
            // Process mouse motion events
            let mut total_delta = Vec2::ZERO;
            for event in mouse_motion_events.read() {
                total_delta += event.delta;
            }
            let delta = total_delta;
            let pan_speed = 0.1;
            
            let right = transform.right();
            let up = transform.up();
            
            transform.translation += right.as_vec3() * -delta.x * pan_speed;
            transform.translation += up.as_vec3() * delta.y * pan_speed;
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
                Color::srgb(1.0, 1.0, 0.0), // Yellow
            );
            
            // Draw manipulation gizmo based on current mode
            match editor_state.manipulation_mode {
                ManipulationMode::Translate => {
                    // Draw translation axes
                    gizmos.arrow(pos, pos + Vec3::X * 0.5, Color::srgb(1.0, 0.0, 0.0)); // Red
                    gizmos.arrow(pos, pos + Vec3::Y * 0.5, Color::srgb(0.0, 1.0, 0.0)); // Green
                    gizmos.arrow(pos, pos + Vec3::Z * 0.5, Color::srgb(0.0, 0.0, 1.0)); // Blue
                }
                ManipulationMode::Rotate => {
                    // Draw rotation circles
                    gizmos.circle(pos, 0.3, Color::srgb(1.0, 0.0, 0.0)); // Red
                    gizmos.circle(pos, 0.3, Color::srgb(0.0, 1.0, 0.0)); // Green  
                    gizmos.circle(pos, 0.3, Color::srgb(0.0, 0.0, 1.0)); // Blue
                }
                ManipulationMode::Scale => {
                    // Draw scale handles
                    gizmos.cuboid(
                        Transform::from_translation(pos + Vec3::X * 0.4).with_scale(Vec3::splat(0.1)),
                        Color::srgb(1.0, 0.0, 0.0), // Red
                    );
                    gizmos.cuboid(
                        Transform::from_translation(pos + Vec3::Y * 0.4).with_scale(Vec3::splat(0.1)),
                        Color::srgb(0.0, 1.0, 0.0), // Green
                    );
                    gizmos.cuboid(
                        Transform::from_translation(pos + Vec3::Z * 0.4).with_scale(Vec3::splat(0.1)),
                        Color::srgb(0.0, 0.0, 1.0), // Blue
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


