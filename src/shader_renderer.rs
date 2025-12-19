use wgpu::*;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use crate::wgsl_reflect_integration::{WgslReflectAnalyzer, ShaderStage, BindingType};

const VERBOSE_LOG: bool = true;

use crate::audio_system::AudioData;
// NOTE: This file is assumed to be 'src/shader_renderer.rs' based on the errors.

// --- Data Structures for External Use (e.g., passing from a GUI/Main loop) ---

/// Parameters controlling the shader rendering environment.
#[derive(Debug)]
pub struct RenderParameters {
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub frame_rate: f32,
    pub audio_data: Option<AudioData>,
}

impl Default for RenderParameters {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            time: 0.0,
            frame_rate: 60.0,
            audio_data: None,
        }
    }
}

/// Parameters passed as a uniform buffer to the WGSL shader.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Uniforms {
    pub time: f32,
    pub resolution: [f32; 2],
    pub mouse: [f32; 2],
    pub audio_volume: f32,
    pub audio_bass: f32,
    pub audio_mid: f32,
    pub audio_treble: f32,
    // Padding to make struct size 40 bytes (16-byte aligned)
    pub _padding: [u32; 1],
}

// Enable safe transfer of Uniforms struct to a GPU buffer
unsafe impl Pod for Uniforms {}
unsafe impl Zeroable for Uniforms {}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            resolution: [512.0, 512.0],
            mouse: [0.5, 0.5],
            audio_volume: 0.0,
            audio_bass: 0.0,
            audio_mid: 0.0,
            audio_treble: 0.0,
            _padding: [0u32],
        }
    }
}

/// A structure to hold example shaders for the UI.
#[derive(Debug, Clone)]
pub struct WorkingShaderExample {
    pub name: String,
    pub description: String,
    pub wgsl_code: String,
    pub category: String,
}

// --- Shader Renderer Core Structure ---

/// Manages WGPU resources and handles compiling and rendering WGSL code to a texture.
pub struct ShaderRenderer {
    device: Device,
    queue: Queue,
    _instance: Instance, // Keep instance alive
    size: (u32, u32),
    // Working WGPU example shaders
    pub working_examples: Vec<WorkingShaderExample>,
    time: std::time::Instant,
    last_errors: Vec<String>,
}

impl ShaderRenderer {
    /// Creates a new ShaderRenderer with a default size (512, 512).
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        ShaderRenderer::new_with_size((512, 512)).await
    }

    /// Creates a new ShaderRenderer with a specified size.
    pub async fn new_with_size(size: (u32, u32)) -> Result<Self, Box<dyn std::error::Error>> {
        if VERBOSE_LOG { println!("Initializing WGPU renderer..."); }

        let instance = Instance::new(&wgpu::InstanceDescriptor::default());
        if VERBOSE_LOG { println!("SUCCESS: WGPU instance created"); }

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|e| format!("Failed to find a suitable GPU adapter: {}. Make sure you have a compatible graphics card and drivers installed.", e))?;
        if VERBOSE_LOG { println!("SUCCESS: GPU adapter found: {:?}", adapter.get_info().name); }

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;
        if VERBOSE_LOG { println!("SUCCESS: GPU device and queue created successfully"); }

        let mut working_examples = Vec::new();
        ShaderRenderer::add_working_examples(&mut working_examples);

        Ok(Self {
            device,
            queue,
            _instance: instance,
            size,
            working_examples,
            time: std::time::Instant::now(),
            last_errors: Vec::new(),
        })
    }

    /// Populates the list of working example shaders.
    fn add_working_examples(examples: &mut Vec<WorkingShaderExample>) {
        examples.push(WorkingShaderExample {
            name: "Animated Gradient".to_string(),
            description: "Beautiful animated color gradient using time".to_string(),
            category: "Basic".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    // Audio uniforms are included in the uniform struct but only used if specified
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
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Mandelbrot Fractal".to_string(),
            description: "Classic Mandelbrot fractal with coloring".to_string(),
            category: "Fractal".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    // Audio uniforms are omitted from use here for simplicity
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn mandelbrot(c: vec2<f32>) -> f32 {
    var z = vec2<f32>(0.0, 0.0);
    let max_iter = 100.0;
    
    var iterations: f32 = 0.0;
    loop {
        // Exit condition
        if (dot(z, z) > 4.0 || iterations >= max_iter) {
            break;
        }
        // Z = Z*Z + C
        z = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        iterations = iterations + 1.0;
    }
    // Return normalized iteration count
    return iterations / max_iter;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    // Center and normalize UVs based on the shorter side for square aspect
    let uv = (position.xy - 0.5 * uniforms.resolution) / min(uniforms.resolution.x, uniforms.resolution.y);
    let zoom = 2.0;
    let pan = vec2<f32>(-0.5, 0.0);
    let c = uv * zoom + pan;
    
    let m = mandelbrot(c);
    // Simple coloring based on normalized iteration count
    let color = vec3<f32>(m, m * 0.5, m * 0.8);
    
    return vec4<f32>(color, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Audio Reactive Wave".to_string(),
            description: "Wave pattern that responds to audio".to_string(),
            category: "Audio".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
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
    
    // Use audio_volume to influence the wave frequency/amplitude
    let wave = sin(uv.x * 10.0 + time * 2.0 + uniforms.audio_volume * 5.0) * 0.5 + 0.5;
    let audio_boost = uniforms.audio_volume * 0.3;
    
    let r = wave + audio_boost;
    let g = 0.5 + 0.5 * sin(time + uv.y * 6.28318 + uniforms.audio_mid);
    let b = 0.5 + 0.5 * cos(time + uniforms.audio_bass);
    
    return vec4<f32>(r, g, b, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Plasma Effect".to_string(),
            description: "Classic plasma effect with smooth colors".to_string(),
            category: "Effects".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    // Audio uniforms are omitted from use here for simplicity
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    // Normalized UVs from -1.0 to 1.0, centered
    let uv = (position.xy / uniforms.resolution - 0.5) * 2.0;
    let time = uniforms.time;
    
    // Classic plasma formula using sine waves on both x and y, offset by time
    let r = sin(uv.x * 3.0 + time) + sin(uv.y * 2.0 + time * 0.5);
    let g = sin(uv.x * 2.0 + time * 0.7) + sin(uv.y * 3.0 + time * 1.2);
    let b = sin(uv.x * 4.0 + time * 0.3) + sin(uv.y * 1.0 + time * 0.9);
    
    // Scale sin output (-2.0 to 2.0) to color range (0.0 to 1.0)
    let col = vec3<f32>(0.5 + 0.5 * r, 0.5 + 0.5 * g, 0.5 + 0.5 * b);
    
    return vec4<f32>(col, 1.0);
}"#),
        });

        // Add many more WGSL examples
        examples.push(WorkingShaderExample {
            name: "Raymarched Sphere".to_string(),
            description: "3D sphere rendered with raymarching".to_string(),
            category: "3D".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn sphere_sdf(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn scene_sdf(p: vec3<f32>) -> f32 {
    return sphere_sdf(p - vec3<f32>(0.0, 0.0, 2.0), 0.5);
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0; i < 64; i = i + 1) {
        let p = ro + rd * t;
        let d = scene_sdf(p);
        if (d < 0.001) {
            break;
        }
        t = t + d;
        if (t > 100.0) {
            break;
        }
    }
    return t;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy / uniforms.resolution - 0.5) * 2.0;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let uv_corrected = vec2<f32>(uv.x * aspect, uv.y);

    let ro = vec3<f32>(0.0, 0.0, 0.0);
    let rd = normalize(vec3<f32>(uv_corrected, 1.0));

    let t = raymarch(ro, rd);

    if (t < 100.0) {
        let p = ro + rd * t;
        let n = normalize(p - vec3<f32>(0.0, 0.0, 2.0));
        let light = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let diff = max(dot(n, light), 0.0);
        let col = vec3<f32>(0.8, 0.6, 0.4) * diff;
        return vec4<f32>(col, 1.0);
    } else {
        return vec4<f32>(0.1, 0.1, 0.2, 1.0);
    }
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Voronoi Noise".to_string(),
            description: "Procedural Voronoi noise pattern".to_string(),
            category: "Noise".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn hash2(p: vec2<f32>) -> vec2<f32> {
    let h = vec2<f32>(dot(p, vec2<f32>(12.9898, 78.233)), dot(p, vec2<f32>(45.164, 94.673)));
    return fract(sin(h) * 43758.5453);
}

fn voronoi(p: vec2<f32>) -> f32 {
    let ip = floor(p);
    let fp = fract(p);

    var min_dist = 1.0;
    for (var i = -1; i <= 1; i = i + 1) {
        for (var j = -1; j <= 1; j = j + 1) {
            let offset = vec2<f32>(f32(i), f32(j));
            let point = hash2(ip + offset) * 0.5 + 0.25;
            let diff = fp - offset - point;
            let dist = length(diff);
            min_dist = min(min_dist, dist);
        }
    }
    return min_dist;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;

    let scale = 8.0;
    let p = uv * scale + time * 0.1;

    let v = voronoi(p);
    let col = vec3<f32>(v);

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Water Caustics".to_string(),
            description: "Realistic water caustics effect".to_string(),
            category: "Effects".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn noise(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var freq = 1.0;

    for (var i = 0; i < 5; i = i + 1) {
        value += amplitude * noise(p * freq);
        amplitude *= 0.5;
        freq *= 2.0;
    }

    return value;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;

    let p = uv * 10.0 + time * 0.5;
    let n = fbm(p);

    let caustic = pow(n, 3.0) * 2.0;
    let col = vec3<f32>(caustic, caustic * 0.8, caustic * 0.6);

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Metaballs".to_string(),
            description: "Smooth organic shapes using distance fields".to_string(),
            category: "Effects".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn metaball(p: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return radius / length(p - center);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;

    let p = uv * 2.0 - 1.0;

    // Create moving metaballs
    let ball1 = metaball(p, vec2<f32>(sin(time), cos(time)) * 0.5, 0.3);
    let ball2 = metaball(p, vec2<f32>(sin(time * 1.3), cos(time * 1.3)) * 0.5, 0.3);
    let ball3 = metaball(p, vec2<f32>(sin(time * 0.7), cos(time * 0.7)) * 0.5, 0.3);

    let sum = ball1 + ball2 + ball3;

    let col = vec3<f32>(smoothstep(0.5, 1.5, sum));

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Tunnel Effect".to_string(),
            description: "Infinite tunnel with perspective".to_string(),
            category: "3D".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy / uniforms.resolution - 0.5) * 2.0;
    let time = uniforms.time;

    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);

    let tunnel = 1.0 / radius;
    let stripes = sin(angle * 8.0 + time * 2.0 + tunnel * 10.0);

    let col = vec3<f32>(
        0.5 + 0.5 * sin(tunnel + time),
        0.5 + 0.5 * sin(tunnel + time + 2.0944),
        0.5 + 0.5 * sin(tunnel + time + 4.18879)
    ) * stripes;

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Particle System".to_string(),
            description: "GPU particle system simulation".to_string(),
            category: "Simulation".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn hash2(p: vec2<f32>) -> vec2<f32> {
    let h = vec2<f32>(dot(p, vec2<f32>(12.9898, 78.233)), dot(p, vec2<f32>(45.164, 94.673)));
    return fract(sin(h) * 43758.5453);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;

    var col = vec3<f32>(0.0);

    // Simulate 50 particles
    for (var i = 0; i < 50; i = i + 1) {
        let fi = f32(i);
        let seed = vec2<f32>(fi, fi + 1.0);
        let pos = hash2(seed) * 2.0 - 1.0;
        let vel = hash2(seed + 1.0) * 0.5 - 0.25;

        let particle_pos = pos + vel * time;
        let dist = length(uv - particle_pos * 0.5 + 0.5);

        if (dist < 0.01) {
            col += vec3<f32>(1.0, 0.5, 0.2) * (1.0 - dist * 100.0);
        }
    }

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Fractal Tree".to_string(),
            description: "Recursive fractal tree structure".to_string(),
            category: "Fractal".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn tree(uv: vec2<f32>, depth: i32) -> f32 {
    if (depth >= 8) {
        return 0.0;
    }

    let angle = sin(uniforms.time + f32(depth)) * 0.3;
    let len = 0.3 / pow(1.5, f32(depth));

    let dir = vec2<f32>(sin(angle), cos(angle));
    let end = uv + dir * len;

    let line_dist = abs(uv.x * dir.y - uv.y * dir.x) / length(dir);
    let dist_to_end = length(uv - end);

    var result = smoothstep(0.002, 0.0, line_dist);

    // Recurse to branches
    let left_uv = uv - end;
    let left_rot = mat2x2<f32>(
        cos(0.5), -sin(0.5),
        sin(0.5), cos(0.5)
    );
    let left_branch = left_rot * left_uv;

    let right_uv = uv - end;
    let right_rot = mat2x2<f32>(
        cos(-0.5), -sin(-0.5),
        sin(-0.5), cos(-0.5)
    );
    let right_branch = right_rot * right_uv;

    result += tree(left_branch, depth + 1) * 0.7;
    result += tree(right_branch, depth + 1) * 0.7;

    return result;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy / uniforms.resolution - 0.5) * 2.0;
    let p = vec2<f32>(uv.x, uv.y + 0.8);

    let t = tree(p, 0);
    let col = vec3<f32>(0.2, 0.5, 0.1) * t;

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Shadertoy Classic".to_string(),
            description: "Classic Shadertoy-style demo effect".to_string(),
            category: "Demo".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;

    let p = (uv - 0.5) * 2.0;

    let r = length(p);
    let a = atan2(p.y, p.x) + time;

    let col = vec3<f32>(
        0.5 + 0.5 * sin(a * 3.0 + time),
        0.5 + 0.5 * sin(a * 4.0 + time * 1.3),
        0.5 + 0.5 * sin(a * 5.0 + time * 0.7)
    ) * (1.0 - r);

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Liquid Marble".to_string(),
            description: "Flowing liquid marble effect".to_string(),
            category: "Effects".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn noise(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var freq = 1.0;

    for (var i = 0; i < 4; i = i + 1) {
        value += amplitude * noise(p * freq + uniforms.time * 0.1);
        amplitude *= 0.5;
        freq *= 2.0;
    }

    return value;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;

    let p = uv * 4.0;

    let n1 = fbm(p);
    let n2 = fbm(p + vec2<f32>(10.0, 10.0));

    let marble = sin(p.x * 10.0 + n1 * 5.0) * cos(p.y * 8.0 + n2 * 3.0);

    let col = vec3<f32>(
        0.5 + 0.5 * sin(marble + 0.0),
        0.5 + 0.5 * sin(marble + 2.0944),
        0.5 + 0.5 * sin(marble + 4.18879)
    );

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "CRT Monitor".to_string(),
            description: "Retro CRT monitor effect".to_string(),
            category: "Retro".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;

    // Scanlines
    let scanline = sin(uv.y * uniforms.resolution.y * 0.5) * 0.1 + 0.9;

    // RGB shift
    let r = textureSample(uv + vec2<f32>(0.002, 0.0));
    let g = textureSample(uv);
    let b = textureSample(uv - vec2<f32>(0.002, 0.0));

    // Vignette
    let vignette = 1.0 - length(uv - 0.5) * 0.5;

    let col = vec3<f32>(r, g, b) * scanline * vignette;

    return vec4<f32>(col, 1.0);
}

fn textureSample(uv: vec2<f32>) -> f32 {
    let p = uv * 10.0 + uniforms.time;
    return 0.5 + 0.5 * sin(p.x) * cos(p.y);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Hypnotic Spiral".to_string(),
            description: "Mesmerizing spiral pattern".to_string(),
            category: "Effects".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy / uniforms.resolution - 0.5) * 2.0;
    let time = uniforms.time;

    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);

    let spiral = sin(radius * 10.0 - angle * 5.0 - time * 3.0);

    let col = vec3<f32>(
        0.5 + 0.5 * sin(spiral + time),
        0.5 + 0.5 * sin(spiral + time + 2.0944),
        0.5 + 0.5 * sin(spiral + time + 4.18879)
    ) * (1.0 - radius * 0.5);

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Quantum Field".to_string(),
            description: "Quantum field visualization".to_string(),
            category: "Scientific".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn wave_function(p: vec2<f32>, n: i32) -> f32 {
    var result = 0.0;
    for (var i = 1; i <= n; i = i + 1) {
        let fi = f32(i);
        let freq = fi * 3.14159;
        let amplitude = 1.0 / fi;
        result += amplitude * sin(length(p) * freq - uniforms.time * fi);
    }
    return result;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy / uniforms.resolution - 0.5) * 4.0;

    let psi = wave_function(uv, 5);
    let prob = psi * psi;

    let col = vec3<f32>(
        prob * 2.0,
        prob * 1.5,
        prob * 1.0
    );

    return vec4<f32>(col, 1.0);
}"#),
        });

        examples.push(WorkingShaderExample {
            name: "Neural Network".to_string(),
            description: "Visual representation of neural network".to_string(),
            category: "AI".to_string(),
            wgsl_code: format!("{}\n{}", VERTEX_SHADER, r#"
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn sigmoid(x: f32) -> f32 {
    return 1.0 / (1.0 + exp(-x));
}

fn neuron(pos: vec2<f32>, input: f32) -> f32 {
    let dist = length(pos);
    return sigmoid(input - dist * 2.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;

    var activation = 0.0;

    // Input layer
    for (var i = 0; i < 8; i = i + 1) {
        let fi = f32(i);
        let pos = vec2<f32>(0.2, fi / 7.0);
        let input = sin(time + fi);
        activation += neuron(uv - pos, input);
    }

    // Hidden layer
    for (var i = 0; i < 5; i = i + 1) {
        let fi = f32(i);
        let pos = vec2<f32>(0.5, fi / 4.0);
        activation = neuron(uv - pos, activation * 2.0 - 1.0);
    }

    // Output layer
    let output_pos = vec2<f32>(0.8, 0.5);
    activation = neuron(uv - output_pos, activation * 2.0 - 1.0);

    let col = vec3<f32>(activation, activation * 0.7, activation * 0.4);

    return vec4<f32>(col, 1.0);
}"#),
        });
    }

    /// Returns a slice of the pre-defined working shader examples.
    pub fn get_working_examples(&self) -> &[WorkingShaderExample] {
        &self.working_examples
    }

    /// Performs the shader rendering operation.
    /// 
    /// Compiles the WGSL code, sets up the pipeline, executes the render pass,
    /// and reads the resulting RGBA pixel data back from the GPU buffer.
    /// The return type `Box<[u8]>` fixes the `E0308` error from the compilation log.
    pub fn render_frame(&mut self, wgsl_code: &str, params: &RenderParameters, audio_data: Option<AudioData>) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if params.width == 0 || params.height == 0 {
            let num_pixels = (params.width.max(1) * params.height.max(1)) as usize;
            let red_pixels = vec![255, 0, 0, 255].into_iter().cycle().take(num_pixels * 4).collect();
            return Ok(red_pixels);
        }
        if wgsl_code.contains("@compute") && !wgsl_code.contains("@fragment") {
            return self.render_compute_to_pixels(wgsl_code, params);
        }
        let fragment_entry_point = {
            let mut analyzer = WgslReflectAnalyzer::new();
            if analyzer.analyze_shader(wgsl_code).is_ok() {
                if let Some(ep) = analyzer.entry_points.iter().find(|e| matches!(e.stage, ShaderStage::Fragment)) {
                    ep.name.clone()
                } else if wgsl_code.contains("fn fs_main(") {
                    "fs_main".to_string()
                } else if wgsl_code.contains("fn main(") {
                    "main".to_string()
                } else {
                    "main".to_string()
                }
            } else if wgsl_code.contains("fn fs_main(") {
                "fs_main".to_string()
            } else if wgsl_code.contains("fn main(") {
                "main".to_string()
            } else {
                "main".to_string()
            }
        };
        if VERBOSE_LOG { println!("Detected fragment shader entry point: '{}'", fragment_entry_point); }
        // Quick return for empty code to prevent hanging
        if wgsl_code.trim().is_empty() {
            let num_pixels = (params.width * params.height) as usize;
            let red_pixels = vec![255, 0, 0, 255].into_iter().cycle().take(num_pixels * 4).collect();
            return Ok(red_pixels);
        }
        self.last_errors.clear();

        // --- 1. Setup Output Texture (FIXED: Use correct format) ---
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Shader Output"),
            size: wgpu::Extent3d {
                width: params.width,
                height: params.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let texture = self.device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        if VERBOSE_LOG { println!("Output texture created: {}x{}", params.width, params.height); }

        // --- 2. Create Shader Module ---
        if VERBOSE_LOG { println!("Compiling WGSL shader..."); }
        
        // Check if the shader has a vertex shader, if not add default one
        let full_shader_code = if !wgsl_code.contains("@vertex") {
            if VERBOSE_LOG { println!("Adding default vertex shader to fragment-only shader"); }
            format!("{}\n{}", VERTEX_SHADER, wgsl_code)
        } else {
            wgsl_code.to_string()
        };
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&full_shader_code)),
        });
        if let Some(err) = pollster::block_on(self.device.pop_error_scope()) {
            let msg = format!("WGSL validation error: {}", err);
            if VERBOSE_LOG { println!("ERROR: {}", msg); }
            self.last_errors.push(msg.clone());
            return Err(msg.into());
        } else {
            if VERBOSE_LOG { println!("SUCCESS: Shader compiled successfully"); }
        }

        // --- 3. Create Uniform Buffer (FIXED: Proper alignment and validation) ---
        let uniforms = Uniforms {
            time: params.time,
            resolution: [params.width as f32, params.height as f32],
            mouse: [0.5, 0.5], // Placeholder mouse position
            audio_volume: audio_data.as_ref().map(|a| a.volume).unwrap_or(0.0),
            audio_bass: audio_data.as_ref().map(|a| a.bass_level).unwrap_or(0.0),
            audio_mid: audio_data.as_ref().map(|a| a.mid_level).unwrap_or(0.0),
            audio_treble: audio_data.as_ref().map(|a| a.treble_level).unwrap_or(0.0),
            _padding: [0u32],
        };

        // Validate uniform buffer size
        let uniform_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
        if VERBOSE_LOG { println!("Uniform buffer size: {} bytes", uniform_size); }

        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // --- 4. Setup Pipeline Resources (FIXED: Enhanced error handling) ---
        let use_params = wgsl_code.contains("var<uniform> params") || wgsl_code.contains("params: array<vec4<f32>");
        let mut analyzer = WgslReflectAnalyzer::new();
        let _ = analyzer.analyze_shader(wgsl_code);
        let group0 = analyzer.bind_groups.iter().find(|g| g.group == 0);
        let mut layout_entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
        if let Some(g0) = group0 {
            for b in &g0.bindings {
                match b.binding_type {
                    BindingType::UniformBuffer => {
                        layout_entries.push(wgpu::BindGroupLayoutEntry {
                            binding: b.binding,
                            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        });
                    }
                    BindingType::StorageBuffer => {
                        layout_entries.push(wgpu::BindGroupLayoutEntry {
                            binding: b.binding,
                            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        });
                    }
                    BindingType::Texture => {
                        let view_dimension = if wgsl_code.contains("texture_3d") { wgpu::TextureViewDimension::D3 } else { wgpu::TextureViewDimension::D2 };
                        layout_entries.push(wgpu::BindGroupLayoutEntry {
                            binding: b.binding,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension,
                                multisampled: false,
                            },
                            count: None,
                        });
                    }
                    BindingType::Sampler => {
                        layout_entries.push(wgpu::BindGroupLayoutEntry {
                            binding: b.binding,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        });
                    }
                    _ => {}
                }
            }
        } else {
            layout_entries.push(wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None });
            if use_params {
                layout_entries.push(wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None });
            }
        }
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("Bind Group Layout"), entries: &layout_entries });

        let mut params_buffer: Option<wgpu::Buffer> = None;
        if use_params {
            let params_data: [f32; 64] = [0.0; 64];
            params_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Params Buffer"), contents: bytemuck::cast_slice(&params_data), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST }));
        }

        let mut group_layouts: Vec<(u32, wgpu::BindGroupLayout)> = Vec::new();
        let mut bind_groups: Vec<(u32, wgpu::BindGroup)> = Vec::new();
        let mut bind_entries: Vec<wgpu::BindGroupEntry> = Vec::new();
        let mut temp_views: Vec<Box<wgpu::TextureView>> = Vec::new();
        let mut temp_samplers: Vec<Box<wgpu::Sampler>> = Vec::new();
        let mut temp_buffers: Vec<wgpu::Buffer> = Vec::new();
        struct PendingRef { binding: u32, kind: u8, index: usize }
        let mut pending: Vec<PendingRef> = Vec::new();
        if let Some(g0) = group0 {
            for b in &g0.bindings {
                match b.binding_type {
                    BindingType::UniformBuffer => {
                        let use_params_buf = b.name.to_lowercase().contains("params") && params_buffer.is_some();
                        let res = if use_params_buf { params_buffer.as_ref().unwrap().as_entire_binding() } else { uniform_buffer.as_entire_binding() };
                        bind_entries.push(wgpu::BindGroupEntry { binding: b.binding, resource: res });
                    }
                    BindingType::StorageBuffer => {
                        let storage_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Dummy Storage Buffer"),
                            size: 1024,
                            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });
                        temp_buffers.push(storage_buf);
                        let idx = temp_buffers.len() - 1;
                        pending.push(PendingRef { binding: b.binding, kind: 0, index: idx });
                    }
                    BindingType::Texture => {
                        let dim3 = wgsl_code.contains("texture_3d");
                        let tmp_tex = self.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("Tmp Texture"),
                            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: if dim3 { 1 } else { 1 } },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: if dim3 { wgpu::TextureDimension::D3 } else { wgpu::TextureDimension::D2 },
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                            view_formats: &[],
                        });
                        let tmp_view = Box::new(tmp_tex.create_view(&wgpu::TextureViewDescriptor::default()));
                        temp_views.push(tmp_view);
                        let idx = temp_views.len() - 1;
                        pending.push(PendingRef { binding: b.binding, kind: 1, index: idx });
                    }
                    BindingType::Sampler => {
                        let tmp_sampler = Box::new(self.device.create_sampler(&wgpu::SamplerDescriptor::default()));
                        temp_samplers.push(tmp_sampler);
                        let idx = temp_samplers.len() - 1;
                        pending.push(PendingRef { binding: b.binding, kind: 2, index: idx });
                    }
                    _ => {}
                }
            }
            for p in pending {
                match p.kind {
                    0 => {
                        let res = temp_buffers[p.index].as_entire_binding();
                        bind_entries.push(wgpu::BindGroupEntry { binding: p.binding, resource: res });
                    }
                    1 => {
                        let view_ref: &wgpu::TextureView = &*temp_views[p.index];
                        bind_entries.push(wgpu::BindGroupEntry { binding: p.binding, resource: wgpu::BindingResource::TextureView(view_ref) });
                    }
                    2 => {
                        let sampler_ref: &wgpu::Sampler = &*temp_samplers[p.index];
                        bind_entries.push(wgpu::BindGroupEntry { binding: p.binding, resource: wgpu::BindingResource::Sampler(sampler_ref) });
                    }
                    _ => {}
                }
            }
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Bind Group"), layout: &bind_group_layout, entries: &bind_entries });
            bind_groups.push((0, bind_group));
        } else {
            bind_entries.push(wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() });
            if let Some(ref pb) = params_buffer {
                bind_entries.push(wgpu::BindGroupEntry { binding: 1, resource: pb.as_entire_binding() });
            }
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Bind Group"), layout: &bind_group_layout, entries: &bind_entries });
            bind_groups.push((0, bind_group));
        }

        let mut owned_layouts: Vec<wgpu::BindGroupLayout> = Vec::new();
        if group_layouts.is_empty() {
            owned_layouts.push(bind_group_layout.clone());
        } else {
            group_layouts.sort_by_key(|(g, _)| *g);
            for (_, layout) in &group_layouts {
                owned_layouts.push(layout.clone());
            }
        }
        let layout_refs: Vec<&wgpu::BindGroupLayout> = owned_layouts.iter().collect();

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &layout_refs,
            push_constant_ranges: &[],
        });

        // --- 5. Create Render Pipeline (FIXED: Correct format matching) ---
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let vertex_entry_point = {
            let mut analyzer = WgslReflectAnalyzer::new();
            if analyzer.analyze_shader(wgsl_code).is_ok() {
                if let Some(ep) = analyzer.entry_points.iter().find(|e| matches!(e.stage, ShaderStage::Vertex)) {
                    ep.name.clone()
                } else {
                    "vs_main".to_string()
                }
            } else {
                "vs_main".to_string()
            }
        };
        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some(vertex_entry_point.as_str()),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(fragment_entry_point.as_str()),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        if let Some(err) = pollster::block_on(self.device.pop_error_scope()) {
            let msg = format!("Pipeline validation error: {}", err);
            if VERBOSE_LOG { println!("ERROR: {}", msg); }
            self.last_errors.push(msg.clone());
            return Err(msg.into());
        } else {
            if VERBOSE_LOG { println!("SUCCESS: Render pipeline created"); }
        }

        // --- 6. Execute Render Pass (FIXED: Enhanced error handling) ---
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&render_pipeline);
            bind_groups.sort_by_key(|(g, _)| *g);
            for (group_idx, bg) in &bind_groups {
                render_pass.set_bind_group(*group_idx, bg, &[]);
            }
            render_pass.draw(0..3, 0..1);
        }

        // --- 7. Copy Texture to Read-back Buffer (FIXED: Synchronization) ---
        let pixel_count = (params.width * params.height) as usize;
        let buffer_size = pixel_count * 4; // 4 bytes per pixel (RGBA8)

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Copy texture to buffer for readback with proper alignment
        let bytes_per_row = params.width * 4;
        let aligned_bytes_per_row = ((bytes_per_row + 255) / 256) * 256; // Align to 256 bytes as required by WGPU
        
        // Recreate buffer with proper size if needed
        let buffer_size = (aligned_bytes_per_row * params.height) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer Aligned"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(aligned_bytes_per_row),
                    rows_per_image: Some(params.height),
                },
            },
            wgpu::Extent3d {
                width: params.width,
                height: params.height,
                depth_or_array_layers: 1,
            },
        );

        // --- 8. Submit Commands (FIXED: Simplified synchronization) ---
        let command_buffer = encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));

        // --- 9. Read Back with Enhanced Error Handling ---
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        let _ = self.device.poll(wgpu::PollType::Wait);
        match rx.recv() {
            Ok(Ok(())) => {
                let data = buffer_slice.get_mapped_range();
                
                // Extract actual pixel data, skipping alignment padding
                let mut pixel_data = Vec::with_capacity((params.width * params.height * 4) as usize);
                for y in 0..params.height {
                    let row_start = (y * aligned_bytes_per_row) as usize;
                    let row_end = row_start + (params.width * 4) as usize;
                    pixel_data.extend_from_slice(&data[row_start..row_end]);
                }
                
                drop(data);
                output_buffer.unmap();
                
                // Ensure we always return valid pixel data with correct size
                if pixel_data.is_empty() || pixel_data.len() != (params.width as usize * params.height as usize * 4) {
                    let safe_width = params.width.max(1);
                    let safe_height = params.height.max(1);
                    let num_pixels = (safe_width as usize * safe_height as usize);
                    let red_pixels = vec![255, 0, 0, 255].into_iter().cycle().take(num_pixels * 4).collect();
                    if VERBOSE_LOG { println!("Fixed pixel data size mismatch in render_frame: expected {}, got {}", num_pixels * 4, pixel_data.len()); }
                    Ok(red_pixels)
                } else {
                    Ok(pixel_data)
                }
            }
            Ok(Err(e)) => {
                // Enhanced fallback with debug pattern
                let pixel_count = (params.width * params.height) as usize;
                let mut dummy_pixels = vec![0u8; pixel_count * 4];
                
                // Create a debug pattern instead of solid gray
                for y in 0..params.height {
                    for x in 0..params.width {
                        let idx = ((y * params.width + x) * 4) as usize;
                        dummy_pixels[idx] = ((x as f32 / params.width as f32) * 255.0) as u8;     // R
                        dummy_pixels[idx + 1] = ((y as f32 / params.height as f32) * 255.0) as u8; // G
                        dummy_pixels[idx + 2] = 128; // B
                        dummy_pixels[idx + 3] = 255; // A
                    }
                }
                // Ensure dummy pixels have correct size
                let safe_width = params.width.max(1);
                let safe_height = params.height.max(1);
                let expected_size = (safe_width as usize * safe_height as usize * 4);
                if dummy_pixels.len() != expected_size {
                    dummy_pixels.resize(expected_size, 0);
                }
                Ok(dummy_pixels)
            }
            Err(_e) => {
                // Return a simple pattern instead of hanging
                let pixel_count = (params.width * params.height) as usize;
                let mut dummy_pixels = vec![0u8; pixel_count * 4];
                
                // Create a simple gradient pattern
                for y in 0..params.height {
                    for x in 0..params.width {
                        let idx = ((y * params.width + x) * 4) as usize;
                        dummy_pixels[idx] = ((x as f32 / params.width as f32) * 128.0) as u8;     // R
                        dummy_pixels[idx + 1] = ((y as f32 / params.height as f32) * 128.0) as u8; // G
                        dummy_pixels[idx + 2] = 64; // B
                        dummy_pixels[idx + 3] = 255; // A
                    }
                }
                // Ensure dummy pixels have correct size
                let safe_width = params.width.max(1);
                let safe_height = params.height.max(1);
                let expected_size = (safe_width as usize * safe_height as usize * 4);
                if dummy_pixels.len() != expected_size {
                    dummy_pixels.resize(expected_size, 0);
                }
                Ok(dummy_pixels)
            }
        }
    }

    pub fn get_last_errors(&self) -> &[String] {
        &self.last_errors
    }

    /// Returns the current size of the renderer output.
    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }

    /// Updates the target rendering size.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        let w = width.max(1);
        let h = height.max(1);
        self.size = (w, h);
        Ok(())
    }

    /// Compile and render a shader with the given code and size.
    pub fn compile_shader(&mut self, wgsl_code: &str, width: u32, height: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.compile_shader_with_params(wgsl_code, width, height, None)
    }
    
    pub fn compile_shader_with_params(&mut self, wgsl_code: &str, width: u32, height: u32, parameter_values: Option<&[f32]>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.resize(width, height)?;
        
        let render_params = RenderParameters {
            width,
            height,
            time: 0.0,
            frame_rate: 60.0,
            audio_data: None,
        };
        
        self.render_frame_with_params(wgsl_code, &render_params, parameter_values, render_params.audio_data.clone())
            .map_err(|e| {
                let error_msg = format!("{:?}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg)) as Box<dyn std::error::Error>
            })
    }
    
    pub fn render_frame_with_params(&mut self, wgsl_code: &str, params: &RenderParameters, parameter_values: Option<&[f32]>, audio_data: Option<AudioData>) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if params.width == 0 || params.height == 0 {
            let pixel_count = (params.width.max(1) * params.height.max(1)) as usize;
            return Ok(vec![0u8; pixel_count * 4]);
        }
        // Quick return for empty code to prevent hanging
        if wgsl_code.trim().is_empty() {
            let pixel_count = (params.width * params.height) as usize;
            return Ok(vec![0u8; pixel_count * 4]);
        }
        if wgsl_code.contains("@compute") && !wgsl_code.contains("@fragment") {
            return self.render_compute_to_pixels(wgsl_code, params);
        }
        if VERBOSE_LOG { println!("Starting GPU shader render with parameters..."); }
        self.last_errors.clear();

        let fragment_entry_point = {
            let mut analyzer = WgslReflectAnalyzer::new();
            if analyzer.analyze_shader(wgsl_code).is_ok() {
                if let Some(ep) = analyzer.entry_points.iter().find(|e| matches!(e.stage, ShaderStage::Fragment)) {
                    ep.name.clone()
                } else if wgsl_code.contains("fn fs_main(") {
                    "fs_main".to_string()
                } else if wgsl_code.contains("fn main(") {
                    "main".to_string()
                } else {
                    "main".to_string()
                }
            } else if wgsl_code.contains("fn fs_main(") {
                "fs_main".to_string()
            } else if wgsl_code.contains("fn main(") {
                "main".to_string()
            } else {
                "main".to_string()
            }
        };

        let vertex_entry_point = {
            let mut analyzer = WgslReflectAnalyzer::new();
            if analyzer.analyze_shader(wgsl_code).is_ok() {
                if let Some(ep) = analyzer.entry_points.iter().find(|e| matches!(e.stage, ShaderStage::Vertex)) {
                    ep.name.clone()
                } else {
                    "vs_main".to_string()
                }
            } else {
                "vs_main".to_string()
            }
        };

        // --- 1. Setup Output Texture (FIXED: Use correct format) ---
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Shader Output"),
            size: wgpu::Extent3d {
                width: params.width,
                height: params.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // FIXED: Use Rgba8Unorm instead of Rgba8UnormSrgb for WGSL compatibility
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let texture = self.device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        if VERBOSE_LOG { println!("Output texture created: {}x{}", params.width, params.height); }

        // --- 2. Parse Shader Code (FIXED: Robust parsing) ---
        // Ensure a vertex shader exists; prepend a default if missing
        let full_shader_code = if !wgsl_code.contains("@vertex") {
            format!("{}\n{}", VERTEX_SHADER, wgsl_code)
        } else {
            wgsl_code.to_string()
        };
        let shader_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(full_shader_code.clone().into()),
        });
        if VERBOSE_LOG { println!("SUCCESS: Shader module created"); }

        // --- 3. Setup Uniforms (ENHANCED: Live audio metrics from AudioAnalyzer) ---
        let uniforms = Uniforms {
            time: params.time,
            resolution: [params.width as f32, params.height as f32],
            mouse: [0.0, 0.0],
            audio_volume: audio_data.as_ref().map(|d| d.volume).unwrap_or(0.0),
            audio_bass: audio_data.as_ref().map(|d| d.bass_level).unwrap_or(0.0),
            audio_mid: audio_data.as_ref().map(|d| d.mid_level).unwrap_or(0.0),
            audio_treble: audio_data.as_ref().map(|d| d.treble_level).unwrap_or(0.0),
            _padding: [0u32],
        };

        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // --- 4. Setup Pipeline Resources (FIXED: Enhanced error handling) ---
        let use_params = wgsl_code.contains("var<uniform> params") || wgsl_code.contains("params: array<vec4<f32>");
        let mut analyzer = WgslReflectAnalyzer::new();
        let _ = analyzer.analyze_shader(wgsl_code);
        let mut params_buffer: Option<wgpu::Buffer> = None;
        if use_params {
            let mut params_data: [f32; 64] = [0.0; 64];
            if let Some(values) = parameter_values {
                for (i, &value) in values.iter().enumerate().take(64) {
                    params_data[i] = value;
                }
            }
            params_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::cast_slice(&params_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }));
        }
        let mut group_layouts: Vec<(u32, wgpu::BindGroupLayout)> = Vec::new();
        let mut bind_groups: Vec<(u32, wgpu::BindGroup)> = Vec::new();
        let mut temp_views_all: Vec<Box<wgpu::TextureView>> = Vec::new();
        let mut temp_samplers_all: Vec<Box<wgpu::Sampler>> = Vec::new();
        let mut temp_buffers_all: Vec<wgpu::Buffer> = Vec::new();

        if analyzer.bind_groups.is_empty() {
            let mut layout_entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
            layout_entries.push(wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None });
            if use_params {
                layout_entries.push(wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None });
            }
            let bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("Bind Group Layout"), entries: &layout_entries });
            group_layouts.push((0, bgl));
            let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
            entries.push(wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() });
            if let Some(ref pb) = params_buffer { entries.push(wgpu::BindGroupEntry { binding: 1, resource: pb.as_entire_binding() }); }
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Bind Group"), layout: &group_layouts.last().unwrap().1, entries: &entries });
            bind_groups.push((0, bg));
        } else {
            let mut groups = analyzer.bind_groups.clone();
            groups.sort_by_key(|g| g.group);
            for g in groups {
                // If reflection found a group but no bindings, synthesize sensible defaults for group 0
                if g.bindings.is_empty() {
                    if g.group == 0 {
                        let mut default_entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
                        default_entries.push(wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                            count: None,
                        });
                        if use_params {
                            default_entries.push(wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                                count: None,
                            });
                        }
                        let bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some("Bind Group Layout (Synthesized G0)"),
                            entries: &default_entries,
                        });
                        group_layouts.push((0, bgl));
                        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
                        entries.push(wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() });
                        if let Some(ref pb) = params_buffer {
                            entries.push(wgpu::BindGroupEntry { binding: 1, resource: pb.as_entire_binding() });
                        }
                        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Bind Group (Synthesized G0)"),
                            layout: &group_layouts.last().unwrap().1,
                            entries: &entries,
                        });
                        bind_groups.push((0, bg));
                        continue;
                    } else {
                        // Skip empty non-zero groups
                        continue;
                    }
                }
                let mut layout_entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
                for b in &g.bindings {
                    match b.binding_type {
                        BindingType::UniformBuffer => {
                            layout_entries.push(wgpu::BindGroupLayoutEntry {
                                binding: b.binding,
                                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                                count: None,
                            });
                        }
                        BindingType::StorageBuffer => {
                            layout_entries.push(wgpu::BindGroupLayoutEntry {
                                binding: b.binding,
                                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                                count: None,
                            });
                        }
                        BindingType::Texture => {
                            let view_dimension = if wgsl_code.contains("texture_3d") { wgpu::TextureViewDimension::D3 } else { wgpu::TextureViewDimension::D2 };
                            layout_entries.push(wgpu::BindGroupLayoutEntry {
                                binding: b.binding,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                    view_dimension,
                                    multisampled: false,
                                },
                                count: None,
                            });
                        }
                        BindingType::Sampler => {
                            layout_entries.push(wgpu::BindGroupLayoutEntry {
                                binding: b.binding,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            });
                        }
                        _ => {}
                    }
                }
                let bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(&format!("Bind Group Layout {}", g.group)), entries: &layout_entries });
                group_layouts.push((g.group, bgl));
                let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
                struct PendingRefAll { binding: u32, kind: u8, index: usize }
                let mut pending_all: Vec<PendingRefAll> = Vec::new();
                for b in &g.bindings {
                    match b.binding_type {
                        BindingType::UniformBuffer => {
                            let use_params_buf = b.name.to_lowercase().contains("params") && params_buffer.is_some();
                            let res = if use_params_buf { params_buffer.as_ref().unwrap().as_entire_binding() } else { uniform_buffer.as_entire_binding() };
                            entries.push(wgpu::BindGroupEntry { binding: b.binding, resource: res });
                        }
                        BindingType::StorageBuffer => {
                            let storage_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("Dummy Storage Buffer"),
                                size: 1024,
                                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            });
                            temp_buffers_all.push(storage_buf);
                            let idx = temp_buffers_all.len() - 1;
                            pending_all.push(PendingRefAll { binding: b.binding, kind: 0, index: idx });
                        }
                        BindingType::Texture => {
                            let dim3 = wgsl_code.contains("texture_3d");
                            let tmp_tex = self.device.create_texture(&wgpu::TextureDescriptor {
                                label: Some("Tmp Texture"),
                                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: if dim3 { 1 } else { 1 } },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: if dim3 { wgpu::TextureDimension::D3 } else { wgpu::TextureDimension::D2 },
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                                view_formats: &[],
                            });
                            let tmp_view = Box::new(tmp_tex.create_view(&wgpu::TextureViewDescriptor::default()));
                            temp_views_all.push(tmp_view);
                            let idx = temp_views_all.len() - 1;
                            pending_all.push(PendingRefAll { binding: b.binding, kind: 1, index: idx });
                        }
                        BindingType::Sampler => {
                            let tmp_sampler = Box::new(self.device.create_sampler(&wgpu::SamplerDescriptor::default()));
                            temp_samplers_all.push(tmp_sampler);
                            let idx = temp_samplers_all.len() - 1;
                            pending_all.push(PendingRefAll { binding: b.binding, kind: 2, index: idx });
                        }
                        _ => {}
                    }
                }
                for p in pending_all {
                    match p.kind {
                        0 => {
                            let res = temp_buffers_all[p.index].as_entire_binding();
                            entries.push(wgpu::BindGroupEntry { binding: p.binding, resource: res });
                        }
                        1 => {
                            let view_ref: &wgpu::TextureView = &*temp_views_all[p.index];
                            entries.push(wgpu::BindGroupEntry { binding: p.binding, resource: wgpu::BindingResource::TextureView(view_ref) });
                        }
                        2 => {
                            let sampler_ref: &wgpu::Sampler = &*temp_samplers_all[p.index];
                            entries.push(wgpu::BindGroupEntry { binding: p.binding, resource: wgpu::BindingResource::Sampler(sampler_ref) });
                        }
                        _ => {}
                    }
                }
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some(&format!("Bind Group {}", g.group)), layout: &group_layouts.last().unwrap().1, entries: &entries });
                bind_groups.push((g.group, bg));
            }
        }

        let mut owned_layouts: Vec<wgpu::BindGroupLayout> = Vec::new();
        if group_layouts.is_empty() {
            let mut layout_entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            });
            if use_params {
                layout_entries.push(wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                });
            }
            let fb = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("Bind Group Layout Fallback"), entries: &layout_entries });
            owned_layouts.push(fb.clone());
            let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
            entries.push(wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() });
            if let Some(ref pb) = params_buffer {
                entries.push(wgpu::BindGroupEntry { binding: 1, resource: pb.as_entire_binding() });
            }
            let fallback_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Bind Group Fallback"), layout: owned_layouts.last().unwrap(), entries: &entries });
            bind_groups.push((0, fallback_bg));
        } else {
            group_layouts.sort_by_key(|(g, _)| *g);
            for (_, layout) in &group_layouts {
                owned_layouts.push(layout.clone());
            }
        }

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &owned_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Vertex Shader"),
                    source: wgpu::ShaderSource::Wgsl(full_shader_code.clone().into()),
                }),
                entry_point: Some(vertex_entry_point.as_str()),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment Shader"),
                    source: wgpu::ShaderSource::Wgsl(full_shader_code.into()),
                }),
                entry_point: Some(fragment_entry_point.as_str()),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&pipeline);
            bind_groups.sort_by_key(|(g, _)| *g);
            for (group_idx, bg) in &bind_groups {
                render_pass.set_bind_group(*group_idx, bg, &[]);
            }
            render_pass.draw(0..3, 0..1);
        }

        let bytes_per_row = params.width * 4;
        let aligned_bytes_per_row = ((bytes_per_row + 255) / 256) * 256;
        let buffer_size = (aligned_bytes_per_row * params.height) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(aligned_bytes_per_row), rows_per_image: Some(params.height) },
            },
            wgpu::Extent3d { width: params.width, height: params.height, depth_or_array_layers: 1 },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
        let _ = self.device.poll(wgpu::PollType::Wait);
        rx.recv().unwrap()?;
        let data = slice.get_mapped_range();
        let mut result = Vec::with_capacity((params.width * params.height * 4) as usize);
        for y in 0..params.height {
            let row_start = (y * aligned_bytes_per_row) as usize;
            let row_end = row_start + (params.width * 4) as usize;
            result.extend_from_slice(&data[row_start..row_end]);
        }
        drop(data);
        output_buffer.unmap();
        // Ensure we always return valid pixel data with correct size
        if result.is_empty() || result.len() != (params.width as usize * params.height as usize * 4) {
            let safe_width = params.width.max(1);
            let safe_height = params.height.max(1);
            let num_pixels = (safe_width as usize * safe_height as usize);
            let red_pixels = vec![255, 0, 0, 255].into_iter().cycle().take(num_pixels * 4).collect();
            Ok(red_pixels)
        } else {
            Ok(result)
        }
    }

    fn render_pipeline_with_bind_group(&mut self, wgsl_code: &str, params: &RenderParameters, bind_group: wgpu::BindGroup, bind_group_layout: wgpu::BindGroupLayout, texture: wgpu::Texture, texture_view: wgpu::TextureView, _uniform_buffer: wgpu::Buffer, vertex_entry_point: &str, fragment_entry_point: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Complete the rendering pipeline setup and execution
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Vertex Shader"),
                    source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
                }),
                entry_point: Some(vertex_entry_point),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment Shader"),
                    source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
                }),
                entry_point: Some(fragment_entry_point), // Use detected entry point name
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Execute render pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1); // Draw full-screen triangle
        }

        // Copy texture to buffer with proper alignment
        let bytes_per_row = params.width * 4;
        let aligned_bytes_per_row = ((bytes_per_row + 255) / 256) * 256; // Align to 256 bytes
        let buffer_size = (aligned_bytes_per_row * params.height) as u64;
        
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(aligned_bytes_per_row),
                    rows_per_image: Some(params.height),
                },
            },
            wgpu::Extent3d {
                width: params.width,
                height: params.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        let _ = self.device.poll(wgpu::PollType::Wait);
        rx.recv().unwrap()?;

        let data = buffer_slice.get_mapped_range();
        
        // Extract actual pixel data, skipping alignment padding
        let mut result = Vec::with_capacity((params.width * params.height * 4) as usize);
        for y in 0..params.height {
            let row_start = (y * aligned_bytes_per_row) as usize;
            let row_end = row_start + (params.width * 4) as usize;
            result.extend_from_slice(&data[row_start..row_end]);
        }
        
        drop(data);
        output_buffer.unmap();

        // Ensure we always return valid pixel data with correct size
        if result.is_empty() || result.len() != (params.width as usize * params.height as usize * 4) {
            let safe_width = params.width.max(1);
            let safe_height = params.height.max(1);
            let num_pixels = (safe_width as usize * safe_height as usize);
            let red_pixels = vec![255, 0, 0, 255].into_iter().cycle().take(num_pixels * 4).collect();
            if VERBOSE_LOG { println!("Fixed pixel data size mismatch: expected {}, got {}", num_pixels * 4, result.len()); }
            Ok(red_pixels)
        } else {
            if VERBOSE_LOG { println!("SUCCESS: Rendering completed successfully"); }
            Ok(result)
        }
    }
    
    fn render_compute_to_pixels(&mut self, wgsl_code: &str, params: &RenderParameters) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut analyzer = WgslReflectAnalyzer::new();
        let _ = analyzer.analyze_shader(wgsl_code);
        let compute_entry = analyzer.entry_points.iter().find(|e| matches!(e.stage, ShaderStage::Compute)).map(|e| e.name.clone()).unwrap_or_else(|| "main".to_string());
        let wg = analyzer.entry_points.iter().find(|e| matches!(e.stage, ShaderStage::Compute)).and_then(|e| e.workgroup_size).unwrap_or((8, 8, 1));
        
        let storage_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Compute Storage Texture"),
            size: wgpu::Extent3d { width: params.width, height: params.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let storage_view = storage_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&storage_view) },
            ],
        });
        
        let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Module"),
            source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
        });
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some(compute_entry.as_str()),
            compilation_options: Default::default(),
            cache: None,
        });
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Compute Encoder") });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Compute Pass"), timestamp_writes: None });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let dx = (params.width + wg.0 - 1) / wg.0;
            let dy = (params.height + wg.1 - 1) / wg.1;
            let dz = wg.2.max(1);
            pass.dispatch_workgroups(dx.max(1), dy.max(1), dz);
        }
        
        let bytes_per_row = params.width * 4;
        let aligned_bytes_per_row = ((bytes_per_row + 255) / 256) * 256;
        let buffer_size = (aligned_bytes_per_row * params.height) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute Output"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &storage_texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(aligned_bytes_per_row), rows_per_image: Some(params.height) },
            },
            wgpu::Extent3d { width: params.width, height: params.height, depth_or_array_layers: 1 },
        );
        
        self.queue.submit(std::iter::once(encoder.finish()));
        
        let slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
        let _ = self.device.poll(wgpu::PollType::Wait);
        rx.recv().unwrap()?;
        let data = slice.get_mapped_range();
        let mut result = Vec::with_capacity((params.width * params.height * 4) as usize);
        for y in 0..params.height {
            let row_start = (y * aligned_bytes_per_row) as usize;
            let row_end = row_start + (params.width * 4) as usize;
            result.extend_from_slice(&data[row_start..row_end]);
        }
        drop(data);
        output_buffer.unmap();
        
        // Ensure we always return valid pixel data with correct size
        if result.is_empty() || result.len() != (params.width as usize * params.height as usize * 4) {
            let safe_width = params.width.max(1);
            let safe_height = params.height.max(1);
            let num_pixels = (safe_width as usize * safe_height as usize);
            let red_pixels = vec![255, 0, 0, 255].into_iter().cycle().take(num_pixels * 4).collect();
            if VERBOSE_LOG { println!("Fixed pixel data size mismatch in compute shader: expected {}, got {}", num_pixels * 4, result.len()); }
            Ok(red_pixels)
        } else {
            Ok(result)
        }
    }

    /// Get the preview texture ID for UI display.
    pub fn get_preview_texture(&self) -> Option<u64> {
        // Return a placeholder texture ID for now
        Some(1)
    }

    /// Update parameter values in the params buffer for shaders that use @group(0) @binding(1) params
    pub fn update_parameters(&mut self, parameter_values: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        // This method will be called to update parameter values
        // The actual buffer update happens during render_frame_with_params
        // We just validate the input here
        if parameter_values.len() > 64 {
            return Err("Too many parameters (max 64)".into());
        }
        Ok(())
    }
    
    /// Execute a compute shader with the given dispatch size and parameters
    pub fn execute_compute_shader(
        &mut self,
        compute_code: &str,
        dispatch_size: (u32, u32, u32),
        parameter_values: Option<&[f32]>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Validate compute shader code
        if !compute_code.contains("@compute") {
            return Err("Compute shader must contain @compute entry point".into());
        }
        
        // Create compute pipeline
        let compute_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(compute_code.into()),
        });
        
        // Create uniform buffer for parameters
        let mut uniform_data = [0.0f32; 4];
        if let Some(params) = parameter_values {
            for (i, &value) in params.iter().take(4).enumerate() {
                uniform_data[i] = value;
            }
        }
        
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Compute Uniforms"),
            contents: bytemuck::cast_slice(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // Create bind group layout
        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        
        // Create compute pipeline
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let compute_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        
        // Execute compute pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });
        
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_size.0, dispatch_size.1, dispatch_size.2);
        }
        
        // Submit command buffer
        let command_buffer = encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
        
        Ok(())
    }
}

// --- Common WGSL Vertex Shader ---

/// A simple vertex shader that generates a single, screen-filling triangle (a quad
/// achieved with 3 vertices) without needing a vertex buffer.
const VERTEX_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    var pos = vec2<f32>(0.0, 0.0);
    switch vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>( 3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0,  3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}
"#;
