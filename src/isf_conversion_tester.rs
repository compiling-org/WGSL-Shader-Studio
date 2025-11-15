//! Comprehensive ISF to WGSL conversion testing and validation
//! 
//! This module provides thorough testing of ISF shader conversion to ensure
//! compatibility and correctness across various ISF shader types and complexity levels.

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};

/// Test case for ISF conversion validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsfConversionTest {
    pub name: String,
    pub description: String,
    pub isf_code: String,
    pub expected_wgsl_patterns: Vec<String>,
    pub expected_uniforms: Vec<String>,
    pub performance_requirements: PerformanceRequirements,
    pub validation_checks: Vec<ValidationCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    pub max_texture_samples: Option<usize>,
    pub max_instructions: Option<usize>,
    pub max_uniforms: Option<usize>,
    pub acceptable_fps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationCheck {
    FunctionMapping(String, String), // ISF function -> WGSL equivalent
    VariableMapping(String, String), // ISF variable -> WGSL equivalent
    TypeMapping(String, String),     // ISF type -> WGSL type
    UniformExistence(String),        // Uniform should exist
    EntryPointExistence(String),     // Entry point should exist
    NoSyntaxErrors,
    PerformanceOptimized,
}

/// Comprehensive ISF conversion tester
pub struct IsfConversionTester {
    test_cases: Vec<IsfConversionTest>,
    test_results: Vec<ConversionTestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionTestResult {
    pub test_name: String,
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub performance_metrics: PerformanceMetrics,
    pub conversion_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub texture_sample_count: usize,
    pub instruction_count: usize,
    pub uniform_count: usize,
    pub estimated_fps: f32,
}

impl IsfConversionTester {
    pub fn new() -> Self {
        Self {
            test_cases: Self::create_comprehensive_test_suite(),
            test_results: Vec::new(),
        }
    }
    
    /// Create comprehensive test suite covering various ISF shader types
    fn create_comprehensive_test_suite() -> Vec<IsfConversionTest> {
        vec![
            Self::create_basic_shader_test(),
            Self::create_texture_sampling_test(),
            Self::create_audio_reactive_test(),
            Self::create_multi_pass_test(),
            Self::create_complex_math_test(),
            Self::create_particle_system_test(),
            Self::create_ray_marching_test(),
            Self::create_fractal_test(),
            Self::create_noise_generation_test(),
            Self::create_color_grading_test(),
        ]
    }
    
    fn create_basic_shader_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Basic Color Shader".to_string(),
            description: "Simple color manipulation shader".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Basic Color",
                "DESCRIPTION": "Simple color shader",
                "INPUTS": [
                    {"NAME": "brightness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0}
                ]
            }*/
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                vec3 color = vec3(uv.x, uv.y, 0.5) * brightness;
                gl_FragColor = vec4(color, 1.0);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "@fragment".to_string(),
                "fn fs_main".to_string(),
                "brightness: f32".to_string(),
                "uniforms.brightness".to_string(),
                "isf_FragNormCoord".to_string(),
            ],
            expected_uniforms: vec!["brightness".to_string()],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(0),
                max_instructions: Some(50),
                max_uniforms: Some(10),
                acceptable_fps: 60.0,
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("gl_FragColor".to_string(), "return".to_string()),
                ValidationCheck::VariableMapping("isf_FragNormCoord".to_string(), "uv".to_string()),
                ValidationCheck::UniformExistence("brightness".to_string()),
                ValidationCheck::EntryPointExistence("fs_main".to_string()),
                ValidationCheck::NoSyntaxErrors,
            ],
        }
    }
    
    fn create_texture_sampling_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Texture Sampling Shader".to_string(),
            description: "Shader with texture sampling operations".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Texture Sampler",
                "DESCRIPTION": "Texture sampling test",
                "INPUTS": [
                    {"NAME": "inputImage", "TYPE": "image"},
                    {"NAME": "scale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0}
                ]
            }*/
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                vec2 scaledUV = uv * scale;
                vec4 color = IMG_NORM_PIXEL(inputImage, scaledUV);
                vec2 imgSize = IMG_SIZE(inputImage);
                float aspect = imgSize.x / imgSize.y;
                gl_FragColor = vec4(color.rgb * aspect, color.a);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "textureSample".to_string(),
                "textureDimensions".to_string(),
                "inputImage: texture_2d<f32>".to_string(),
                "scale: f32".to_string(),
            ],
            expected_uniforms: vec!["inputImage".to_string(), "scale".to_string()],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(1),
                max_instructions: Some(100),
                max_uniforms: Some(15),
                acceptable_fps: 60.0,
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("IMG_NORM_PIXEL".to_string(), "textureSample".to_string()),
                ValidationCheck::FunctionMapping("IMG_SIZE".to_string(), "textureDimensions".to_string()),
                ValidationCheck::UniformExistence("inputImage".to_string()),
                ValidationCheck::UniformExistence("scale".to_string()),
            ],
        }
    }
    
    fn create_audio_reactive_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Audio Reactive Shader".to_string(),
            description: "Audio-reactive shader with beat detection".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Audio Reactive",
                "DESCRIPTION": "Audio reactive shader",
                "INPUTS": [
                    {"NAME": "audioTexture", "TYPE": "audio"},
                    {"NAME": "audioFFT", "TYPE": "audiofft"},
                    {"NAME": "intensity", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0}
                ]
            }*/
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                float bass = IMG_NORM_PIXEL(audioFFT, vec2(0.1, 0.5)).r;
                float treble = IMG_NORM_PIXEL(audioFFT, vec2(0.8, 0.5)).r;
                float time = TIME * 0.5;
                float wave = sin(uv.x * 10.0 + time) * bass * intensity;
                vec3 color = vec3(wave, treble * intensity, bass);
                gl_FragColor = vec4(color, 1.0);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "audioTexture: texture_2d<f32>".to_string(),
                "audioFFT: texture_2d<f32>".to_string(),
                "intensity: f32".to_string(),
                "TIME".to_string(),
                "sin".to_string(),
            ],
            expected_uniforms: vec!["audioTexture".to_string(), "audioFFT".to_string(), "intensity".to_string()],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(2),
                max_instructions: Some(150),
                max_uniforms: Some(20),
                acceptable_fps: 60.0,
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("IMG_NORM_PIXEL".to_string(), "textureSample".to_string()),
                ValidationCheck::VariableMapping("TIME".to_string(), "uniforms.time".to_string()),
                ValidationCheck::UniformExistence("audioTexture".to_string()),
                ValidationCheck::UniformExistence("audioFFT".to_string()),
            ],
        }
    }
    
    fn create_multi_pass_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Multi-pass Shader".to_string(),
            description: "Shader with multiple render passes".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Multi-pass Effect",
                "DESCRIPTION": "Multi-pass rendering test",
                "INPUTS": [
                    {"NAME": "blurAmount", "TYPE": "float", "DEFAULT": 5.0, "MIN": 0.0, "MAX": 20.0}
                ],
                "PASSES": [
                    {"TARGET": "blurTemp", "WIDTH": "${RENDERSIZE}/2", "HEIGHT": "${RENDERSIZE}/2"},
                    {"TARGET": "blurTemp2", "WIDTH": "${RENDERSIZE}/2", "HEIGHT": "${RENDERSIZE}/2"}
                ]
            }*/
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                // First pass - horizontal blur
                vec4 color = vec4(0.0);
                for (int i = -5; i <= 5; i++) {
                    vec2 offset = vec2(float(i) * blurAmount / 100.0, 0.0);
                    color += IMG_NORM_PIXEL(blurTemp, uv + offset);
                }
                color /= 11.0;
                gl_FragColor = color;
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "blurAmount: f32".to_string(),
                "for".to_string(),
                "textureSample".to_string(),
            ],
            expected_uniforms: vec!["blurAmount".to_string()],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(11),
                max_instructions: Some(200),
                max_uniforms: Some(15),
                acceptable_fps: 30.0, // Lower FPS due to loop
            },
            validation_checks: vec![
                ValidationCheck::UniformExistence("blurAmount".to_string()),
                ValidationCheck::PerformanceOptimized,
            ],
        }
    }
    
    fn create_complex_math_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Complex Math Shader".to_string(),
            description: "Shader with complex mathematical operations".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Complex Math",
                "DESCRIPTION": "Complex mathematical operations",
                "INPUTS": [
                    {"NAME": "frequency", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 10.0},
                    {"NAME": "amplitude", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0}
                ]
            }*/
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                float time = TIME * frequency;
                float wave1 = sin(uv.x * 10.0 + time) * cos(uv.y * 8.0 + time * 0.5);
                float wave2 = tan(uv.x * 5.0) * amplitude;
                float noise = fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453);
                vec3 color = vec3(wave1 * amplitude, wave2, noise);
                gl_FragColor = vec4(color, 1.0);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "frequency: f32".to_string(),
                "amplitude: f32".to_string(),
                "sin".to_string(),
                "cos".to_string(),
                "tan".to_string(),
                "fract".to_string(),
                "dot".to_string(),
            ],
            expected_uniforms: vec!["frequency".to_string(), "amplitude".to_string()],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(0),
                max_instructions: Some(75),
                max_uniforms: Some(10),
                acceptable_fps: 60.0,
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("sin".to_string(), "sin".to_string()),
                ValidationCheck::FunctionMapping("cos".to_string(), "cos".to_string()),
                ValidationCheck::FunctionMapping("tan".to_string(), "tan".to_string()),
                ValidationCheck::FunctionMapping("fract".to_string(), "fract".to_string()),
                ValidationCheck::VariableMapping("TIME".to_string(), "uniforms.time".to_string()),
            ],
        }
    }
    
    fn create_particle_system_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Particle System".to_string(),
            description: "GPU particle system with physics simulation".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Particle System",
                "DESCRIPTION": "GPU particle simulation",
                "INPUTS": [
                    {"NAME": "particleCount", "TYPE": "float", "DEFAULT": 1000.0, "MIN": 100.0, "MAX": 10000.0},
                    {"NAME": "gravity", "TYPE": "float", "DEFAULT": 0.98, "MIN": 0.0, "MAX": 2.0},
                    {"NAME": "damping", "TYPE": "float", "DEFAULT": 0.99, "MIN": 0.9, "MAX": 1.0}
                ]
            }*/
            
            struct Particle {
                vec3 position;
                vec3 velocity;
                float life;
                float size;
            };
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                float id = floor(uv.x * particleCount);
                
                // Initialize particle
                vec3 pos = vec3(
                    sin(id * 0.1) * 2.0,
                    cos(id * 0.15) * 2.0,
                    id * 0.01
                );
                
                vec3 vel = vec3(
                    sin(TIME + id * 0.1) * 0.5,
                    cos(TIME + id * 0.2) * 0.5,
                    sin(TIME * 0.5 + id * 0.05) * 0.3
                );
                
                // Apply physics
                vel.y -= gravity * 0.016;
                vel *= damping;
                pos += vel * 0.016;
                
                // Render particle
                float dist = length(uv - pos.xy * 0.5 + 0.5);
                float alpha = 1.0 - smoothstep(0.0, 0.1, dist);
                
                gl_FragColor = vec4(1.0, 0.5, 0.2, alpha);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "struct Particle".to_string(),
                "fn main()".to_string(),
                "var<private> uv: vec2<f32>".to_string(),
                "var particleCount: f32".to_string(),
                "var gravity: f32".to_string(),
                "var damping: f32".to_string(),
            ],
            expected_uniforms: vec![
                "particleCount".to_string(),
                "gravity".to_string(),
                "damping".to_string(),
            ],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(0),
                max_instructions: Some(1000),
                max_uniforms: Some(10),
                acceptable_fps: 30.0,
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("main".to_string(), "main".to_string()),
                ValidationCheck::VariableMapping("uv".to_string(), "uv".to_string()),
                ValidationCheck::TypeMapping("vec3".to_string(), "vec3<f32>".to_string()),
                ValidationCheck::UniformExistence("particleCount".to_string()),
                ValidationCheck::EntryPointExistence("main".to_string()),
                ValidationCheck::NoSyntaxErrors,
                ValidationCheck::PerformanceOptimized,
            ],
        }
    }
    
    fn create_ray_marching_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Ray Marching Shader".to_string(),
            description: "Ray marching with distance fields".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Ray Marching",
                "DESCRIPTION": "Ray marching distance fields",
                "INPUTS": [
                    {"NAME": "maxSteps", "TYPE": "float", "DEFAULT": 64.0, "MIN": 16.0, "MAX": 256.0},
                    {"NAME": "maxDistance", "TYPE": "float", "DEFAULT": 10.0, "MIN": 1.0, "MAX": 100.0}
                ]
            }*/
            
            float sdSphere(vec3 p, float r) {
                return length(p) - r;
            }
            
            float sdBox(vec3 p, vec3 b) {
                vec3 q = abs(p) - b;
                return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
            }
            
            float map(vec3 p) {
                float sphere = sdSphere(p - vec3(0.0, 0.0, 0.0), 1.0);
                float box = sdBox(p - vec3(3.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
                return min(sphere, box);
            }
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                vec3 ro = vec3(0.0, 0.0, -3.0);
                vec3 rd = normalize(vec3(uv - 0.5, 1.0));
                
                float t = 0.0;
                for (int i = 0; i < int(maxSteps); i++) {
                    vec3 p = ro + rd * t;
                    float d = map(p);
                    if (d < 0.001 || t > maxDistance) break;
                    t += d;
                }
                
                vec3 color = vec3(t / maxDistance);
                gl_FragColor = vec4(color, 1.0);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "maxSteps: f32".to_string(),
                "maxDistance: f32".to_string(),
                "length".to_string(),
                "normalize".to_string(),
                "min".to_string(),
                "max".to_string(),
                "abs".to_string(),
            ],
            expected_uniforms: vec!["maxSteps".to_string(), "maxDistance".to_string()],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(0),
                max_instructions: Some(500),
                max_uniforms: Some(10),
                acceptable_fps: 30.0, // Lower FPS due to ray marching
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("length".to_string(), "length".to_string()),
                ValidationCheck::FunctionMapping("normalize".to_string(), "normalize".to_string()),
                ValidationCheck::FunctionMapping("min".to_string(), "min".to_string()),
                ValidationCheck::FunctionMapping("max".to_string(), "max".to_string()),
                ValidationCheck::FunctionMapping("abs".to_string(), "abs".to_string()),
            ],
        }
    }
    
    fn create_fractal_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Fractal Shader".to_string(),
            description: "Mandelbrot set fractal shader".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Mandelbrot Set",
                "DESCRIPTION": "Mandelbrot fractal",
                "INPUTS": [
                    {"NAME": "zoom", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 100.0},
                    {"NAME": "maxIterations", "TYPE": "float", "DEFAULT": 100.0, "MIN": 10.0, "MAX": 1000.0}
                ]
            }*/
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                vec2 c = (uv - 0.5) * 4.0 / zoom;
                vec2 z = vec2(0.0, 0.0);
                float iter = 0.0;
                
                for (float i = 0.0; i < maxIterations; i++) {
                    if (length(z) > 2.0) break;
                    z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
                    iter = i;
                }
                
                float color = iter / maxIterations;
                gl_FragColor = vec4(vec3(color), 1.0);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "zoom: f32".to_string(),
                "maxIterations: f32".to_string(),
                "length".to_string(),
                "for".to_string(),
                "break".to_string(),
            ],
            expected_uniforms: vec!["zoom".to_string(), "maxIterations".to_string()],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(0),
                max_instructions: Some(300),
                max_uniforms: Some(10),
                acceptable_fps: 45.0, // Medium FPS due to iterations
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("length".to_string(), "length".to_string()),
                ValidationCheck::UniformExistence("zoom".to_string()),
                ValidationCheck::UniformExistence("maxIterations".to_string()),
            ],
        }
    }
    
    fn create_noise_generation_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Noise Generation Shader".to_string(),
            description: "Procedural noise generation shader".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Noise Generator",
                "DESCRIPTION": "Procedural noise",
                "INPUTS": [
                    {"NAME": "scale", "TYPE": "float", "DEFAULT": 10.0, "MIN": 1.0, "MAX": 100.0},
                    {"NAME": "octaves", "TYPE": "float", "DEFAULT": 4.0, "MIN": 1.0, "MAX": 8.0}
                ]
            }*/
            
            float random(vec2 st) {
                return fract(sin(dot(st.xy, vec2(12.9898,78.233))) * 43758.5453123);
            }
            
            float noise(vec2 st) {
                vec2 i = floor(st);
                vec2 f = fract(st);
                
                float a = random(i);
                float b = random(i + vec2(1.0, 0.0));
                float c = random(i + vec2(0.0, 1.0));
                float d = random(i + vec2(1.0, 1.0));
                
                vec2 u = f * f * (3.0 - 2.0 * f);
                return mix(a, b, u.x) + (c - a)* u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
            }
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                vec2 pos = uv * scale;
                float n = 0.0;
                
                for (float i = 0.0; i < octaves; i++) {
                    n += noise(pos) * (1.0 / pow(2.0, i));
                    pos *= 2.0;
                }
                
                gl_FragColor = vec4(vec3(n), 1.0);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "scale: f32".to_string(),
                "octaves: f32".to_string(),
                "fract".to_string(),
                "sin".to_string(),
                "dot".to_string(),
                "floor".to_string(),
                "mix".to_string(),
                "pow".to_string(),
            ],
            expected_uniforms: vec!["scale".to_string(), "octaves".to_string()],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(0),
                max_instructions: Some(200),
                max_uniforms: Some(10),
                acceptable_fps: 45.0,
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("fract".to_string(), "fract".to_string()),
                ValidationCheck::FunctionMapping("sin".to_string(), "sin".to_string()),
                ValidationCheck::FunctionMapping("dot".to_string(), "dot".to_string()),
                ValidationCheck::FunctionMapping("floor".to_string(), "floor".to_string()),
                ValidationCheck::FunctionMapping("mix".to_string(), "mix".to_string()),
                ValidationCheck::FunctionMapping("pow".to_string(), "pow".to_string()),
            ],
        }
    }
    
    fn create_color_grading_test() -> IsfConversionTest {
        IsfConversionTest {
            name: "Color Grading Shader".to_string(),
            description: "Professional color grading shader".to_string(),
            isf_code: r#"
            /*{
                "NAME": "Color Grading",
                "DESCRIPTION": "Professional color grading",
                "INPUTS": [
                    {"NAME": "inputImage", "TYPE": "image"},
                    {"NAME": "contrast", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0},
                    {"NAME": "saturation", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0},
                    {"NAME": "brightness", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0},
                    {"NAME": "temperature", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0}
                ]
            }*/
            
            vec3 contrastSaturationBrightness(vec3 color, float contrast, float saturation, float brightness) {
                color = mix(vec3(0.5), color, contrast);
                float gray = dot(color, vec3(0.299, 0.587, 0.114));
                color = mix(vec3(gray), color, saturation);
                return color + brightness;
            }
            
            vec3 temperatureAdjust(vec3 color, float temperature) {
                vec3 warm = vec3(1.0, 0.9, 0.7);
                vec3 cool = vec3(0.7, 0.9, 1.0);
                return mix(color, temperature > 0.0 ? warm : cool, abs(temperature));
            }
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                vec4 inputColor = IMG_NORM_PIXEL(inputImage, uv);
                vec3 color = inputColor.rgb;
                color = contrastSaturationBrightness(color, contrast, saturation, brightness);
                color = temperatureAdjust(color, temperature);
                gl_FragColor = vec4(color, inputColor.a);
            }
            "#.to_string(),
            expected_wgsl_patterns: vec![
                "inputImage: texture_2d<f32>".to_string(),
                "contrast: f32".to_string(),
                "saturation: f32".to_string(),
                "brightness: f32".to_string(),
                "temperature: f32".to_string(),
                "mix".to_string(),
                "dot".to_string(),
                "abs".to_string(),
            ],
            expected_uniforms: vec![
                "inputImage".to_string(),
                "contrast".to_string(),
                "saturation".to_string(),
                "brightness".to_string(),
                "temperature".to_string(),
            ],
            performance_requirements: PerformanceRequirements {
                max_texture_samples: Some(1),
                max_instructions: Some(75),
                max_uniforms: Some(25),
                acceptable_fps: 60.0,
            },
            validation_checks: vec![
                ValidationCheck::FunctionMapping("mix".to_string(), "mix".to_string()),
                ValidationCheck::FunctionMapping("dot".to_string(), "dot".to_string()),
                ValidationCheck::FunctionMapping("abs".to_string(), "abs".to_string()),
                ValidationCheck::UniformExistence("inputImage".to_string()),
                ValidationCheck::UniformExistence("contrast".to_string()),
                ValidationCheck::UniformExistence("saturation".to_string()),
            ],
        }
    }
    
    /// Run all conversion tests
    pub fn run_all_tests(&mut self, converter: &crate::isf_auto_converter::IsfAutoConverter) -> Vec<ConversionTestResult> {
        println!("🧪 Running comprehensive ISF conversion tests...");
        self.test_results.clear();
        
        for test_case in &self.test_cases.clone() {
            println!("  Testing: {}", test_case.name);
            let result = self.run_single_test(test_case, converter);
            self.test_results.push(result.clone());
        }
        
        self.print_test_summary();
        self.test_results.clone()
    }
    
    fn run_single_test(&self, test: &IsfConversionTest, converter: &crate::isf_auto_converter::IsfAutoConverter) -> ConversionTestResult {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Parse the ISF shader
        match converter.parse_isf_advanced(&test.isf_code) {
            Ok(isf_shader) => {
                // Convert to WGSL (isf_shader is already the conversion result)
                let conversion_result = isf_shader;
                let wgsl_code = format!("{}\n{}", conversion_result.vertex_shader, conversion_result.fragment_shader);
                
                // Validate patterns
                for pattern in &test.expected_wgsl_patterns {
                    if !wgsl_code.contains(pattern) {
                        errors.push(format!("Missing expected pattern: {}", pattern));
                    }
                }
                
                // Validate uniforms
                for uniform in &test.expected_uniforms {
                    if !wgsl_code.contains(&format!("{}: ", uniform)) {
                        errors.push(format!("Missing expected uniform: {}", uniform));
                    }
                }
                
                // Performance analysis
                let metrics = self.analyze_performance(&wgsl_code);
                
                // Check performance requirements
                if let Some(max_samples) = test.performance_requirements.max_texture_samples {
                    if metrics.texture_sample_count > max_samples {
                        warnings.push(format!("Texture samples ({}) exceed limit ({})", metrics.texture_sample_count, max_samples));
                    }
                }
                
                if let Some(max_uniforms) = test.performance_requirements.max_uniforms {
                    if metrics.uniform_count > max_uniforms {
                        warnings.push(format!("Uniform count ({}) exceeds limit ({})", metrics.uniform_count, max_uniforms));
                    }
                }
                
                // Run validation checks
                for check in &test.validation_checks {
                    match self.run_validation_check(check, &wgsl_code, &conversion_result) {
                        Ok(()) => {},
                        Err(e) => errors.push(e.to_string()),
                    }
                }
                
                let conversion_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                
                return ConversionTestResult {
                    test_name: test.name.clone(),
                    success: errors.is_empty(),
                    errors,
                    warnings,
                    performance_metrics: metrics,
                    conversion_time_ms,
                };
            }
            Err(e) => {
                return ConversionTestResult {
                    test_name: test.name.clone(),
                    success: false,
                    errors: vec![format!("ISF parsing failed: {}", e)],
                    warnings: Vec::new(),
                    performance_metrics: PerformanceMetrics {
                        texture_sample_count: 0,
                        instruction_count: 0,
                        uniform_count: 0,
                        estimated_fps: 0.0,
                    },
                    conversion_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                };
            }
        }
    }
    
    fn run_validation_check(&self, check: &ValidationCheck, wgsl_code: &str, conversion_result: &crate::isf_auto_converter::WgslConversionResult) -> anyhow::Result<()> {
        match check {
            ValidationCheck::FunctionMapping(isf_func, wgsl_func) => {
                if !wgsl_code.contains(wgsl_func) {
                    return Err(anyhow::anyhow!("Function mapping failed: {} -> {} not found", isf_func, wgsl_func));
                }
                Ok(())
            }
            ValidationCheck::VariableMapping(isf_var, wgsl_var) => {
                if !wgsl_code.contains(wgsl_var) {
                    return Err(anyhow::anyhow!("Variable mapping failed: {} -> {} not found", isf_var, wgsl_var));
                }
                Ok(())
            }
            ValidationCheck::TypeMapping(isf_type, wgsl_type) => {
                if !wgsl_code.contains(wgsl_type) {
                    return Err(anyhow::anyhow!("Type mapping failed: {} -> {} not found", isf_type, wgsl_type));
                }
                Ok(())
            }
            ValidationCheck::UniformExistence(uniform) => {
                if !wgsl_code.contains(&format!("{}: ", uniform)) {
                    return Err(anyhow::anyhow!("Uniform not found: {}", uniform));
                }
                Ok(())
            }
            ValidationCheck::EntryPointExistence(entry_point) => {
                if !conversion_result.entry_points.contains(entry_point) {
                    return Err(anyhow::anyhow!("Entry point not found: {}", entry_point));
                }
                Ok(())
            }
            ValidationCheck::NoSyntaxErrors => {
                // Basic syntax check - look for common syntax issues
                if wgsl_code.contains("gl_FragColor") || wgsl_code.contains("void main()") {
                    return Err(anyhow::anyhow!("Contains ISF/GLSL syntax instead of WGSL"));
                }
                Ok(())
            }
            ValidationCheck::PerformanceOptimized => {
                // Basic performance check
                let metrics = self.analyze_performance(wgsl_code);
                if metrics.texture_sample_count > 16 {
                    return Err(anyhow::anyhow!("Too many texture samples: {}", metrics.texture_sample_count));
                }
                Ok(())
            }
        }
    }
    
    fn analyze_performance(&self, wgsl_code: &str) -> PerformanceMetrics {
        let texture_sample_count = wgsl_code.matches("textureSample").count();
        let instruction_count = wgsl_code.lines().count();
        let uniform_count = wgsl_code.matches(": ").count() - wgsl_code.matches("fn ").count();
        
        // Estimate FPS based on complexity (very rough estimation)
        let estimated_fps = if texture_sample_count > 8 {
            30.0
        } else if instruction_count > 100 {
            45.0
        } else {
            60.0
        };
        
        PerformanceMetrics {
            texture_sample_count,
            instruction_count,
            uniform_count,
            estimated_fps,
        }
    }
    
    fn print_test_summary(&self) {
        println!("\n📊 ISF Conversion Test Summary:");
        println!("================================");
        
        let total_tests = self.test_results.len();
        let passed_tests = self.test_results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - passed_tests;
        
        println!("Total Tests: {}", total_tests);
        println!("✅ Passed: {}", passed_tests);
        println!("❌ Failed: {}", failed_tests);
        
        if failed_tests > 0 {
            println!("\nFailed Tests:");
            for result in &self.test_results {
                if !result.success {
                    println!("  - {}: {} errors", result.test_name, result.errors.len());
                    for error in &result.errors {
                        println!("    • {}", error);
                    }
                }
            }
        }
        
        // Performance summary
        let avg_conversion_time: f64 = self.test_results.iter().map(|r| r.conversion_time_ms).sum::<f64>() / self.test_results.len() as f64;
        let avg_fps: f32 = self.test_results.iter().map(|r| r.performance_metrics.estimated_fps).sum::<f32>() / self.test_results.len() as f32;
        
        println!("\nPerformance Summary:");
        println!("Average Conversion Time: {:.2}ms", avg_conversion_time);
        println!("Average Estimated FPS: {:.1}", avg_fps);
    }
}

impl Default for IsfConversionTester {
    fn default() -> Self {
        Self::new()
    }
}

// Fix the missing function
fn create_particle_system_test() -> IsfConversionTest {
    IsfConversionTest {
        name: "Particle System Shader".to_string(),
        description: "Particle system with physics simulation".to_string(),
        isf_code: r#"
        /*{
            "NAME": "Particle System",
            "DESCRIPTION": "Particle physics simulation",
            "INPUTS": [
                {"NAME": "particleCount", "TYPE": "float", "DEFAULT": 100.0, "MIN": 10.0, "MAX": 1000.0},
                {"NAME": "gravity", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.0, "MAX": 1.0},
                {"NAME": "speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.1, "MAX": 5.0}
            ]
        }*/
        
        void main() {
            vec2 uv = isf_FragNormCoord;
            vec3 color = vec3(0.0);
            
            for (float i = 0.0; i < particleCount; i++) {
                float particleID = i / particleCount;
                vec2 particlePos = vec2(sin(TIME * speed + particleID * 6.28), cos(TIME * speed + particleID * 6.28));
                vec2 particleVel = vec2(cos(TIME * speed + particleID * 3.14), sin(TIME * speed + particleID * 3.14));
                
                float dist = distance(uv, particlePos * 0.5 + 0.5);
                float size = 0.02 + sin(TIME * 2.0 + particleID * 10.0) * 0.01;
                
                if (dist < size) {
                    float alpha = 1.0 - (dist / size);
                    vec3 particleColor = vec3(sin(particleID * 6.28), cos(particleID * 4.0), tan(particleID * 2.0));
                    color += particleColor * alpha * gravity;
                }
            }
            
            gl_FragColor = vec4(color, 1.0);
        }
        "#.to_string(),
        expected_wgsl_patterns: vec![
            "particleCount: f32".to_string(),
            "gravity: f32".to_string(),
            "speed: f32".to_string(),
            "sin".to_string(),
            "cos".to_string(),
            "tan".to_string(),
            "distance".to_string(),
            "TIME".to_string(),
        ],
        expected_uniforms: vec!["particleCount".to_string(), "gravity".to_string(), "speed".to_string()],
        performance_requirements: PerformanceRequirements {
            max_texture_samples: Some(0),
            max_instructions: Some(400),
            max_uniforms: Some(15),
            acceptable_fps: 30.0, // Lower FPS due to particle loop
        },
        validation_checks: vec![
            ValidationCheck::FunctionMapping("sin".to_string(), "sin".to_string()),
            ValidationCheck::FunctionMapping("cos".to_string(), "cos".to_string()),
            ValidationCheck::FunctionMapping("tan".to_string(), "tan".to_string()),
            ValidationCheck::FunctionMapping("distance".to_string(), "distance".to_string()),
            ValidationCheck::VariableMapping("TIME".to_string(), "uniforms.time".to_string()),
            ValidationCheck::UniformExistence("particleCount".to_string()),
            ValidationCheck::UniformExistence("gravity".to_string()),
            ValidationCheck::UniformExistence("speed".to_string()),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isf_auto_converter::IsfAutoConverter;

    #[test]
    fn test_comprehensive_isf_conversion() {
        println!("🧪 Starting comprehensive ISF conversion tests...");
        
        let converter = IsfAutoConverter::new();
        let mut tester = IsfConversionTester::new();
        
        // Run all tests using the existing method
        let results = tester.run_all_tests(&converter);
        
        let passed = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();
        
        println!("\n📊 Test Summary:");
        println!("   Total Tests: {}", results.len());
        println!("   Passed: {}", passed);
        println!("   Failed: {}", failed);
        println!("   Success Rate: {:.1}%", (passed as f64 / results.len() as f64) * 100.0);
        
        // Print details of failed tests
        for result in results.iter().filter(|r| !r.success) {
            println!("\n❌ Failed Test: {}", result.test_name);
            if !result.errors.is_empty() {
                println!("   Errors: {:?}", result.errors);
            }
            if !result.warnings.is_empty() {
                println!("   Warnings: {:?}", result.warnings);
            }
        }
        
        assert!(failed == 0, "Some ISF conversion tests failed");
    }

    #[test]
    fn test_individual_isf_conversions() {
        let converter = IsfAutoConverter::new();
        
        // Test basic shader conversion
        let basic_isf = r#"
        /*{
            "NAME": "Basic Test",
            "DESCRIPTION": "Simple color shader",
            "INPUTS": [
                {"NAME": "color", "TYPE": "color", "DEFAULT": [1.0, 0.0, 0.0, 1.0]}
            ]
        }*/
        
        void main() {
            gl_FragColor = color;
        }
        "#;
        
        // Test ISF parsing
        let parse_result = converter.parse_isf_advanced(basic_isf);
        assert!(parse_result.is_ok(), "Basic ISF parsing should succeed");
        
        let isf_shader = parse_result.unwrap();
        
        // Test WGSL conversion
        let convert_result = converter.convert_to_wgsl_advanced(&isf_shader);
        assert!(convert_result.is_ok(), "Basic ISF to WGSL conversion should succeed");
        
        let conversion_result = convert_result.unwrap();
        let wgsl_code = format!("{}\n{}", conversion_result.vertex_shader, conversion_result.fragment_shader);
        
        assert!(wgsl_code.contains("@fragment"), "WGSL should contain fragment shader declaration");
        assert!(wgsl_code.contains("uniforms.color"), "WGSL should contain color uniform");
    }
}