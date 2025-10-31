# WGSL ↔ GLSL Conversion Guide

## Overview
WGSL Shader Studio provides comprehensive bidirectional conversion between WGSL (WebGPU Shading Language) and GLSL (OpenGL Shading Language), enabling cross-platform shader compatibility.

## Conversion Architecture

### Core Components
- **WGSL Parser**: AST-based WGSL source analysis
- **GLSL Generator**: OpenGL-compatible code generation
- **Type System**: Unified type representation
- **Validation**: Syntax and semantic correctness checking

### Supported GLSL Versions
- **GLSL 330**: OpenGL 3.3+ compatibility
- **GLSL 450**: Modern OpenGL features
- **ES 300**: OpenGL ES 3.0+ mobile support

## WGSL to GLSL Conversion

### Basic Syntax Mapping

#### Data Types
```wgsl
// WGSL
var<uniform> time: f32;
var<uniform> resolution: vec2<f32>;
var<uniform> color: vec4<f32>;
var texture: texture_2d<f32>;
var sampler: sampler;

// GLSL Output
uniform float time;
uniform vec2 resolution;
uniform vec4 color;
uniform sampler2D texture;
uniform sampler sampler;
```

#### Functions
```wgsl
// WGSL
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    return vec4<f32>(uv, 0.0, 1.0);
}

// GLSL Output
#version 330 core
out vec4 fragColor;

void main() {
    vec2 uv = gl_FragCoord.xy / resolution;
    fragColor = vec4(uv, 0.0, 1.0);
}
```

#### Built-in Variables
```wgsl
// WGSL Built-ins
@builtin(position) position: vec4<f32>
@builtin(vertex_index) vertex_index: u32
@builtin(instance_index) instance_index: u32

// GLSL Equivalents
gl_Position
gl_VertexID
gl_InstanceID
```

### Advanced Features

#### Texture Sampling
```wgsl
// WGSL
@group(0) @binding(0) var myTexture: texture_2d<f32>;
@group(0) @binding(1) var mySampler: sampler;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    let color = textureSample(myTexture, mySampler, uv);
    return color;
}

// GLSL Output
uniform sampler2D myTexture;
uniform vec2 resolution;

void main() {
    vec2 uv = gl_FragCoord.xy / resolution;
    vec4 color = texture(myTexture, uv);
    fragColor = color;
}
```

#### Control Flow
```wgsl
// WGSL
if (condition) {
    // code
} else if (other_condition) {
    // code
} else {
    // code
}

for (var i = 0; i < 10; i = i + 1) {
    // loop body
}

// GLSL (identical)
if (condition) {
    // code
} else if (other_condition) {
    // code
} else {
    // code
}

for (int i = 0; i < 10; i++) {
    // loop body
}
```

## GLSL to WGSL Conversion

### Shader Stage Detection
```glsl
// Vertex Shader (contains gl_Position)
#version 330 core
layout(location = 0) in vec3 position;
uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
}

// WGSL Output
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    let mvp: mat4x4<f32> = // ... uniform binding
    return mvp * vec4<f32>(position, 1.0);
}
```

```glsl
// Fragment Shader (contains fragColor/gl_FragColor)
#version 330 core
out vec4 fragColor;
uniform vec2 resolution;

void main() {
    vec2 uv = gl_FragCoord.xy / resolution;
    fragColor = vec4(uv, 0.0, 1.0);
}

// WGSL Output
@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    return vec4<f32>(uv, 0.0, 1.0);
}
```

### Uniform Handling
```glsl
// GLSL Uniforms
uniform float time;
uniform vec2 resolution;
uniform vec3 cameraPosition;
uniform mat4 viewMatrix;

// WGSL Output
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;
@group(0) @binding(2) var<uniform> cameraPosition: vec3<f32>;
@group(0) @binding(3) var<uniform> viewMatrix: mat4x4<f32>;
```

## Conversion Limitations

### WGSL-Only Features
- **Workgroup Barriers**: `workgroupBarrier()` not available in GLSL
- **Storage Textures**: `texture_storage_2d<rgba8unorm, write>` requires GLSL 420+
- **Pointer Operations**: WGSL pointer semantics not directly mappable

### GLSL-Only Features
- **Legacy Features**: `gl_FragColor`, `attribute`, `varying`
- **Platform Extensions**: Vendor-specific extensions
- **Version-Specific**: GLSL version-dependent features

## Best Practices

### Optimization Tips
1. **Minimize Conversions**: Convert once, reuse compiled shaders
2. **Validate Output**: Always test converted shaders
3. **Platform Testing**: Test on target platforms
4. **Performance Monitoring**: Compare performance between formats

### Common Patterns
```wgsl
// WGSL Pattern
let uv = position.xy / resolution;
let color = mix(vec3<f32>(0.0), vec3<f32>(1.0), uv.x);

// GLSL Equivalent
vec2 uv = gl_FragCoord.xy / resolution;
vec3 color = mix(vec3(0.0), vec3(1.0), uv.x);
```

### Error Handling
```rust
// Conversion with error handling
match wgsl_to_glsl(wgsl_source) {
    Ok(glsl_code) => {
        // Save or compile GLSL
        println!("Conversion successful");
    }
    Err(e) => {
        eprintln!("Conversion failed: {}", e);
        // Handle conversion errors
    }
}
```

## Integration Examples

### FFGL Plugin Usage
```rust
// Convert WGSL shader for FFGL plugin
let wgsl_shader = load_wgsl_from_file("shader.wgsl")?;
let glsl_shader = wgsl_to_glsl(&wgsl_shader)?;

// Use in FFGL plugin
plugin.load_glsl_shader(&glsl_shader)?;
```

### Cross-Platform Deployment
```rust
// Convert for multiple platforms
let wgsl_source = fs::read_to_string("shader.wgsl")?;

// WebGPU (native WGSL)
let wgsl_module = device.create_shader_module_wgsl(&wgsl_source);

// OpenGL (converted GLSL)
let glsl_source = wgsl_to_glsl(&wgsl_source)?;
let glsl_program = compile_glsl_program(&glsl_source);

// DirectX (WGSL -> HLSL)
let hlsl_source = wgsl_to_hlsl(&wgsl_source)?;
let hlsl_shader = compile_hlsl_shader(&hlsl_source);
```

## Troubleshooting

### Common Issues
1. **Type Mismatches**: Check vec/f32 vs vec/float usage
2. **Built-in Variables**: Ensure correct @builtin attributes
3. **Texture Sampling**: Verify sampler/texture binding groups
4. **Precision Qualifiers**: Add `precision mediump float;` for ES

### Debug Output
```rust
// Enable debug conversion output
std::env::set_var("WGSL_DEBUG", "1");
let result = wgsl_to_glsl(source)?;
println!("Converted GLSL:\n{}", result);
```

## Performance Considerations

### Conversion Overhead
- **Compile-time**: Conversion happens at build/load time
- **Runtime**: Zero overhead after conversion
- **Memory**: Minimal memory usage for conversion process

### Optimization Strategies
1. **Pre-conversion**: Convert shaders during build process
2. **Caching**: Cache converted shaders to disk
3. **Validation**: Validate converted shaders before use
4. **Fallbacks**: Provide fallback shaders for failed conversions

## Future Enhancements

### Planned Features
- **SPIR-V Support**: Direct WGSL ↔ SPIR-V conversion
- **Advanced Optimizations**: Automatic shader optimization
- **Platform-Specific**: Target-specific code generation
- **Debug Information**: Source mapping for debugging

### Extension Points
- **Custom Converters**: Plugin system for custom conversions
- **Shader Libraries**: Pre-converted shader collections
- **Validation Rules**: Configurable validation rules
- **Performance Metrics**: Conversion performance tracking