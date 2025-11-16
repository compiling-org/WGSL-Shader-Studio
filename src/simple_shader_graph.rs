use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::sprite::{Material2d, Material2dPlugin};

/// Simplified shader graph integration that works with Bevy 0.17
pub struct SimpleShaderGraphPlugin;

impl Plugin for SimpleShaderGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<ShaderGraphMaterial>::default())
            .init_resource::<ShaderGraphState>()
            .add_systems(Update, update_shader_graph);
    }
}

/// State for managing shader graph
#[derive(Resource, Default)]
pub struct ShaderGraphState {
    pub current_shader: Option<String>,
    pub shader_handle: Option<Handle<ShaderGraphMaterial>>,
    pub needs_update: bool,
}

/// Material for shader graph rendering
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ShaderGraphMaterial {
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub resolution: Vec2,
    #[uniform(0)]
    pub mouse: Vec2,
    #[uniform(0)]
    pub audio_volume: f32,
    #[uniform(0)]
    pub audio_bass: f32,
    #[uniform(0)]
    pub audio_mid: f32,
    #[uniform(0)]
    pub audio_treble: f32,
}

impl Default for ShaderGraphMaterial {
    fn default() -> Self {
        Self {
            time: 0.0,
            resolution: Vec2::new(512.0, 512.0),
            mouse: Vec2::new(0.5, 0.5),
            audio_volume: 0.0,
            audio_bass: 0.0,
            audio_mid: 0.0,
            audio_treble: 0.0,
        }
    }
}

impl Material2d for ShaderGraphMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/default_shader.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/default_vertex.wgsl".into()
    }
}

/// System to update shader graph
fn update_shader_graph(
    mut shader_graph_state: ResMut<ShaderGraphState>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
) {
    if shader_graph_state.needs_update {
        if let Some(handle) = &shader_graph_state.shader_handle {
            if let Some(material) = materials.get_mut(handle) {
                material.time = time.elapsed_seconds();
                shader_graph_state.needs_update = false;
            }
        }
    }
}

/// Function to create a shader from WGSL code
pub fn create_shader_from_wgsl(wgsl_code: &str) -> Shader {
    Shader::from_wgsl(wgsl_code.into(), "user_shader.wgsl")
}

/// Default vertex shader for full-screen rendering
pub const DEFAULT_VERTEX_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var pos = vec2<f32>(0.0, 0.0);
    switch vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>(3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0, 3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }
    return vec4<f32>(pos, 0.0, 1.0);
}
"#;

/// Default fragment shader
pub const DEFAULT_FRAGMENT_SHADER: &str = r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    audio_volume: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_treble: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;
    
    let r = 0.5 + 0.5 * sin(time + uv.x * 6.28318);
    let g = 0.5 + 0.5 * sin(time * 0.8 + uv.x * 6.28318);
    let b = 0.5 + 0.5 * sin(time * 1.2 + uv.x * 6.28318);
    
    return vec4<f32>(r, g, b, 1.0);
}
"#;

/// System to spawn shader graph quad
pub fn spawn_shader_graph_quad(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShaderGraphMaterial>>,
    shader_graph_state: Res<ShaderGraphState>,
) {
    if shader_graph_state.current_shader.is_some() && shader_graph_state.shader_handle.is_none() {
        let mesh = meshes.add(Mesh::from(Rectangle::new(2.0, 2.0)));
        let material = materials.add(ShaderGraphMaterial::default());
        
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material.clone()),
            Transform::default(),
        ));
        
        // Store the handle for future updates
        // This would need to be stored in a resource or component
    }
}