use bevy::prelude::*;
use bevy::render::camera::{Camera, Projection};
// use bevy::render::primitives::Frustum;
// use bevy::render::view::VisibleEntities;
use std::collections::HashMap;

/// 3D scene configuration and management
#[derive(Resource, Clone, Debug)]
pub struct Scene3DConfig {
    pub camera_position: Vec3,
    pub camera_rotation: Quat,
    pub camera_fov: f32,
    pub camera_near: f32,
    pub camera_far: f32,
    pub camera_speed: f32,
    pub camera_sensitivity: f32,
    
    pub ambient_light_color: Color,
    pub ambient_light_brightness: f32,
    
    pub directional_lights: Vec<DirectionalLightConfig>,
    pub point_lights: Vec<PointLightConfig>,
    pub spot_lights: Vec<SpotLightConfig>,
    
    pub show_grid: bool,
    pub show_axes: bool,
    pub grid_size: f32,
    pub grid_divisions: u32,
    
    pub background_color: Color,
    pub fog_enabled: bool,
    pub fog_color: Color,
    pub fog_density: f32,
    pub fog_start: f32,
    pub fog_end: f32,
}

impl Default for Scene3DConfig {
    fn default() -> Self {
        Self {
            camera_position: Vec3::new(0.0, 2.0, 5.0),
            camera_rotation: Quat::from_rotation_x(-0.2),
            camera_fov: 60.0,
            camera_near: 0.1,
            camera_far: 1000.0,
            camera_speed: 5.0,
            camera_sensitivity: 0.002,
            
            ambient_light_color: Color::WHITE,
            ambient_light_brightness: 0.1,
            
            directional_lights: vec![
                DirectionalLightConfig {
                    direction: Vec3::new(-1.0, -1.0, -1.0),
                    color: Color::WHITE,
                    illuminance: 10000.0,
                    shadows_enabled: true,
                    shadow_depth_bias: 0.02,
                    shadow_normal_bias: 0.2,
                }
            ],
            point_lights: vec![
                PointLightConfig {
                    position: Vec3::new(2.0, 4.0, 2.0),
                    color: Color::rgb(1.0, 0.8, 0.6),
                    intensity: 800.0,
                    range: 20.0,
                    radius: 0.1,
                    shadows_enabled: true,
                    shadow_depth_bias: 0.02,
                    shadow_normal_bias: 0.2,
                }
            ],
            spot_lights: vec![],
            
            show_grid: true,
            show_axes: true,
            grid_size: 20.0,
            grid_divisions: 20,
            
            background_color: Color::rgb(0.1, 0.1, 0.1),
            fog_enabled: false,
            fog_color: Color::rgb(0.5, 0.5, 0.5),
            fog_density: 0.01,
            fog_start: 10.0,
            fog_end: 100.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirectionalLightConfig {
    pub direction: Vec3,
    pub color: Color,
    pub illuminance: f32,
    pub shadows_enabled: bool,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
}

#[derive(Clone, Debug)]
pub struct PointLightConfig {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub shadows_enabled: bool,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
}

#[derive(Clone, Debug)]
pub struct SpotLightConfig {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub angle: f32,
    pub inner_angle: f32,
    pub shadows_enabled: bool,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
}

/// 3D scene manager that handles camera, lighting, and scene entities
#[derive(Resource)]
pub struct Scene3DManager {
    pub config: Scene3DConfig,
    pub camera_entity: Option<Entity>,
    pub light_entities: HashMap<String, Entity>,
    pub grid_entity: Option<Entity>,
    pub axes_entity: Option<Entity>,
}

impl Default for Scene3DManager {
    fn default() -> Self {
        Self {
            config: Scene3DConfig::default(),
            camera_entity: None,
            light_entities: HashMap::new(),
            grid_entity: None,
            axes_entity: None,
        }
    }
}

impl Scene3DManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize the 3D scene with camera and lighting
    pub fn initialize_scene(&mut self, commands: &mut Commands) {
        // Create camera
        let camera_entity = commands.spawn((
            Camera3d::default(),
            Transform::from_translation(self.config.camera_position)
                .with_rotation(self.config.camera_rotation),
            PerspectiveProjection {
                fov: self.config.camera_fov.to_radians(),
                near: self.config.camera_near,
                far: self.config.camera_far,
                ..default()
            },
            VisibleEntities::default(),
            Frustum::default(),
            Camera { order: 1, ..Default::default() },
        )).id();
        
        self.camera_entity = Some(camera_entity);
        
        // Create ambient light
        commands.insert_resource(AmbientLight {
            color: self.config.ambient_light_color,
            brightness: self.config.ambient_light_brightness,
        });
        
        // Create directional lights
        for (i, light_config) in self.config.directional_lights.iter().enumerate() {
            let light_entity = commands.spawn((
                DirectionalLight {
                    color: light_config.color,
                    illuminance: light_config.illuminance,
                    shadows_enabled: light_config.shadows_enabled,
                    shadow_depth_bias: light_config.shadow_depth_bias,
                    shadow_normal_bias: light_config.shadow_normal_bias,
                    ..default()
                },
                Transform::from_rotation(Quat::from_rotation_arc(
                    Vec3::NEG_Z,
                    light_config.direction.normalize(),
                )),
                GlobalTransform::default(),
            )).id();
            
            self.light_entities.insert(format!("directional_{}", i), light_entity);
        }
        
        // Create point lights
        for (i, light_config) in self.config.point_lights.iter().enumerate() {
            let light_entity = commands.spawn((
                PointLight {
                    color: light_config.color,
                    intensity: light_config.intensity,
                    range: light_config.range,
                    radius: light_config.radius,
                    shadows_enabled: light_config.shadows_enabled,
                    shadow_depth_bias: light_config.shadow_depth_bias,
                    shadow_normal_bias: light_config.shadow_normal_bias,
                    ..default()
                },
                Transform::from_translation(light_config.position),
                GlobalTransform::default(),
            )).id();
            
            self.light_entities.insert(format!("point_{}", i), light_entity);
        }
        
        // Create spot lights
        for (i, light_config) in self.config.spot_lights.iter().enumerate() {
            let light_entity = commands.spawn((
                SpotLight {
                    color: light_config.color,
                    intensity: light_config.intensity,
                    range: light_config.range,
                    angle: light_config.angle,
                    inner_angle: light_config.inner_angle,
                    shadows_enabled: light_config.shadows_enabled,
                    shadow_depth_bias: light_config.shadow_depth_bias,
                    shadow_normal_bias: light_config.shadow_normal_bias,
                    ..default()
                },
                Transform::from_translation(light_config.position)
                    .looking_at(light_config.position + light_config.direction, Vec3::Y),
                GlobalTransform::default(),
            )).id();
            
            self.light_entities.insert(format!("spot_{}", i), light_entity);
        }
        
        // Create grid if enabled
        if self.config.show_grid {
            self.create_grid(commands);
        }
        
        // Create axes if enabled
        if self.config.show_axes {
            self.create_axes(commands);
        }
    }
    
    /// Create a visual grid
    fn create_grid(&mut self, commands: &mut Commands) {
        // This would create a mesh-based grid
        // For now, we'll use a placeholder
        let grid_entity = commands.spawn((
            Name::new("Grid"),
            Transform::default(),
            GlobalTransform::default(),
        )).id();
        
        self.grid_entity = Some(grid_entity);
    }
    
    /// Create coordinate axes
    fn create_axes(&mut self, commands: &mut Commands) {
        // This would create mesh-based axes (X=red, Y=green, Z=blue)
        // For now, we'll use a placeholder
        let axes_entity = commands.spawn((
            Name::new("Axes"),
            Transform::default(),
            GlobalTransform::default(),
        )).id();
        
        self.axes_entity = Some(axes_entity);
    }
    
    /// Update camera position
    pub fn update_camera_position(&mut self, position: Vec3, commands: &mut Commands) {
        self.config.camera_position = position;
        
        if let Some(camera_entity) = self.camera_entity {
            commands.entity(camera_entity).insert(
                Transform::from_translation(position)
                    .with_rotation(self.config.camera_rotation)
            );
        }
    }
    
    /// Update camera rotation
    pub fn update_camera_rotation(&mut self, rotation: Quat, commands: &mut Commands) {
        self.config.camera_rotation = rotation;
        
        if let Some(camera_entity) = self.camera_entity {
            commands.entity(camera_entity).insert(
                Transform::from_translation(self.config.camera_position)
                    .with_rotation(rotation)
            );
        }
    }
    
    /// Update camera FOV
    pub fn update_camera_fov(&mut self, fov: f32, commands: &mut Commands) {
        self.config.camera_fov = fov;
        
        if let Some(camera_entity) = self.camera_entity {
            commands.entity(camera_entity).insert(
                PerspectiveProjection {
                    fov: fov.to_radians(),
                    near: self.config.camera_near,
                    far: self.config.camera_far,
                    ..default()
                }
            );
        }
    }
    
    /// Update ambient light
    pub fn update_ambient_light(&mut self, color: Color, brightness: f32, commands: &mut Commands) {
        self.config.ambient_light_color = color;
        self.config.ambient_light_brightness = brightness;
        
        commands.insert_resource(AmbientLight {
            color,
            brightness,
        });
    }
    
    /// Update directional light
    pub fn update_directional_light(&mut self, index: usize, direction: Vec3, color: Color, commands: &mut Commands) {
        if let Some(light_config) = self.config.directional_lights.get_mut(index) {
            light_config.direction = direction;
            light_config.color = color;
            
            let light_name = format!("directional_{}", index);
            if let Some(&light_entity) = self.light_entities.get(&light_name) {
                commands.entity(light_entity).insert((
                    DirectionalLight {
                        color: light_config.color,
                        illuminance: light_config.illuminance,
                        shadows_enabled: light_config.shadows_enabled,
                        shadow_depth_bias: light_config.shadow_depth_bias,
                        shadow_normal_bias: light_config.shadow_normal_bias,
                        ..default()
                    },
                    Transform::from_rotation(Quat::from_rotation_arc(
                        Vec3::NEG_Z,
                        direction.normalize(),
                    )),
                ));
            }
        }
    }
    
    /// Update point light
    pub fn update_point_light(&mut self, index: usize, position: Vec3, color: Color, intensity: f32, commands: &mut Commands) {
        if let Some(light_config) = self.config.point_lights.get_mut(index) {
            light_config.position = position;
            light_config.color = color;
            light_config.intensity = intensity;
            
            let light_name = format!("point_{}", index);
            if let Some(&light_entity) = self.light_entities.get(&light_name) {
                commands.entity(light_entity).insert((
                    PointLight {
                        color: light_config.color,
                        intensity: light_config.intensity,
                        range: light_config.range,
                        radius: light_config.radius,
                        shadows_enabled: light_config.shadows_enabled,
                        shadow_depth_bias: light_config.shadow_depth_bias,
                        shadow_normal_bias: light_config.shadow_normal_bias,
                        ..default()
                    },
                    Transform::from_translation(position),
                ));
            }
        }
    }
    
    /// Get camera view matrix
    pub fn get_camera_view_matrix(&self) -> Mat4 {
        let transform = Transform::from_translation(self.config.camera_position)
            .with_rotation(self.config.camera_rotation);
        transform.compute_matrix().inverse()
    }
    
    /// Get camera projection matrix
    pub fn get_camera_projection_matrix(&self) -> Mat4 {
        let projection = PerspectiveProjection {
            fov: self.config.camera_fov.to_radians(),
            near: self.config.camera_near,
            far: self.config.camera_far,
            ..default()
        };
        projection.get_projection_matrix()
    }
    
    /// Get camera view-projection matrix
    pub fn get_camera_view_projection_matrix(&self) -> Mat4 {
        self.get_camera_projection_matrix() * self.get_camera_view_matrix()
    }
    
    /// Convert screen coordinates to world ray
    pub fn screen_to_world_ray(&self, screen_x: f32, screen_y: f32, screen_width: f32, screen_height: f32) -> (Vec3, Vec3) {
        let camera_pos = self.config.camera_position;
        let camera_rot = self.config.camera_rotation;
        
        // Normalize screen coordinates
        let x = (screen_x / screen_width) * 2.0 - 1.0;
        let y = -((screen_y / screen_height) * 2.0 - 1.0);
        
        // Create ray direction
        let fov_rad = self.config.camera_fov.to_radians();
        let aspect_ratio = screen_width / screen_height;
        let tan_half_fov = (fov_rad * 0.5).tan();
        
        let ray_dir = Vec3::new(
            x * aspect_ratio * tan_half_fov,
            y * tan_half_fov,
            -1.0,
        ).normalize();
        
        // Transform ray by camera rotation
        let ray_dir = camera_rot * ray_dir;
        
        (camera_pos, ray_dir)
    }
}

/// System to handle 3D scene updates
pub fn scene_3d_update_system(
    mut scene_manager: ResMut<Scene3DManager>,
    mut commands: Commands,
) {
    // Handle any scene updates here
    // This could include camera movement, light changes, etc.
}

/// Plugin to add 3D scene functionality
pub struct Scene3DPlugin;

impl Plugin for Scene3DPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scene3DManager>()
            .add_systems(Update, scene_3d_update_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_3d_manager_creation() {
        let manager = Scene3DManager::new();
        assert_eq!(manager.config.camera_position, Vec3::new(0.0, 2.0, 5.0));
        assert_eq!(manager.light_entities.len(), 2); // 1 directional + 1 point light
    }

    #[test]
    fn test_camera_matrices() {
        let manager = Scene3DManager::new();
        let view_matrix = manager.get_camera_view_matrix();
        let proj_matrix = manager.get_camera_projection_matrix();
        let vp_matrix = manager.get_camera_view_projection_matrix();
        
        assert_ne!(view_matrix, Mat4::IDENTITY);
        assert_ne!(proj_matrix, Mat4::IDENTITY);
        assert_ne!(vp_matrix, Mat4::IDENTITY);
    }

    #[test]
    fn test_screen_to_world_ray() {
        let manager = Scene3DManager::new();
        let (origin, direction) = manager.screen_to_world_ray(400.0, 300.0, 800.0, 600.0);
        
        assert_ne!(origin, Vec3::ZERO);
        assert_ne!(direction, Vec3::ZERO);
        assert!((direction.length() - 1.0).abs() < 0.001); // Should be normalized
    }
}
