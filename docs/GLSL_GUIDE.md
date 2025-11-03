# GLSL Shader Guide for WGSL Shader Studio

## Overview
This guide covers GLSL (OpenGL Shading Language) integration, conversion, and development within the WGSL Shader Studio environment.

## GLSL ↔ WGSL Conversion

### Key Differences

| GLSL Feature | WGSL Equivalent | Notes |
|-------------|----------------|-------|
| `gl_FragCoord` | `vec4<f32>(global_id.xy, 0.0, 1.0)` | Fragment coordinate handling |
| `texture2D` | `textureSample` | Texture sampling syntax |
| `void main()` | `fn main()` | Function declaration |
| `vec2(x, y)` | `vec2<f32>(x, y)` | Explicit type annotations |
| `float(x)` | `f32(x)` | Numeric type conversion |
| `gl_FragColor` | Return value | Direct return from fragment shader |

### Common Conversions

#### Vertex Shader
```glsl
// GLSL Vertex Shader
attribute vec3 position;
attribute vec2 texCoord;

varying vec2 vTexCoord;

void main() {
    vTexCoord = texCoord;
    gl_Position = vec4(position, 1.0);
}
```

```wgsl
// WGSL Vertex Shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texCoord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texCoord: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 1.0);
    output.texCoord = input.texCoord;
    return output;
}
```

#### Fragment Shader
```glsl
// GLSL Fragment Shader
uniform float time;
uniform vec2 resolution;
uniform sampler2D texture0;

varying vec2 vTexCoord;

void main() {
    vec2 uv = vTexCoord;
    vec3 color = texture2D(texture0, uv).rgb;
    color += sin(time + uv.x * 10.0) * 0.1;
    gl_FragColor = vec4(color, 1.0);
}
```

```wgsl
// WGSL Fragment Shader
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var texture0: texture_2d<f32>;
@group(1) @binding(1) var texture0_sampler: sampler;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let color = textureSample(texture0, texture0_sampler, uv).rgb;
    let final_color = color + sin(uniforms.time + uv.x * 10.0) * 0.1;
    return vec4<f32>(final_color, 1.0);
}
```

## WGSL Shader Studio GLSL Features

### Import GLSL Shaders
1. **File → Import → GLSL Shader**
2. **Automatic Conversion**: GLSL → WGSL conversion
3. **Validation**: Shader validation before import
4. **Preview**: Live preview of converted shader

### Export to GLSL
1. **Shader → Export → GLSL**
2. **Format Selection**: 
   - OpenGL 3.3 Core
   - OpenGL ES 3.0
   - WebGL 2.0
3. **Optimization**: GLSL-specific optimizations applied

### GLSL Syntax Highlighting
- **Vertex Attributes**: `attribute` → `@location` highlighting
- **Uniforms**: `uniform` → `var<uniform>` highlighting
- **Varyings**: `varying` → struct member highlighting
- **Built-ins**: `gl_FragColor`, `gl_Position` highlighting

### Error Detection
- **Type Mismatches**: GLSL → WGSL type conversion errors
- **Function Calls**: Deprecated GLSL function warnings
- **Precision Qualifiers**: Missing precision qualifier alerts

## Common GLSL Patterns

### Texture Sampling
```glsl
// GLSL
vec4 textureColor = texture2D(texture, uv);
vec4 textureColorLod = texture2DLod(texture, uv, lod);
```

```wgsl
// WGSL
var textureColor = textureSample(texture, sampler, uv);
var textureColorLod = textureSampleLevel(texture, sampler, uv, lod);
```

### Math Operations
```glsl
// GLSL
float length_vec = length(vec2(x, y));
float dot_product = dot(vecA, vecB);
vec2 normalize_vec = normalize(vec2(x, y));
```

```wgsl
// WGSL
var length_vec = distance(vec2<f32>(x, y), vec2<f32>(0.0, 0.0));
var dot_product = dot(vecA, vecB);
var normalize_vec = normalize(vec2<f32>(x, y));
```

### Control Flow
```glsl
// GLSL
for (int i = 0; i < count; i++) {
    sum += values[i];
}
```

```wgsl
// WGSL
var sum = 0.0;
for (var i = 0u; i < count; i = i + 1u) {
    sum = sum + values[i];
}
```

## Best Practices

### Performance
1. **Avoid Dynamic Branches**: Use step functions instead
2. **Texture Format Optimization**: Use appropriate formats
3. **Precision Selection**: Use `f32` for most calculations
4. **Loop Optimization**: Prefer bounded loops

### Compatibility
1. **WebGL Limitations**: Avoid features not supported in WebGL
2. **Mobile Considerations**: Use lower precision where acceptable
3. **Driver Variations**: Test across different GPU drivers

### Code Organization
1. **Struct Usage**: Use structs for complex data
2. **Function Libraries**: Group related functions
3. **Constant Folding**: Use compile-time constants
4. **Uniform Blocks**: Group related uniforms

## Troubleshooting

### Common Conversion Errors
- **Missing Types**: Add explicit type annotations
- **Function Names**: Update to WGSL equivalents
- **Precision Issues**: Specify precision qualifiers
- **Binding Conflicts**: Check @binding assignments

### Performance Issues
- **Fragment Shader Bottlenecks**: Optimize texture operations
- **Vertex Shader Limitations**: Minimize complex calculations
- **Memory Bandwidth**: Reduce texture fetches

### Debug Techniques
1. **Color Debugging**: Output diagnostic colors
2. **Step Functions**: Use step() for conditional logic
3. **Debug Output**: Temporary uniform debugging

## Integration with Audio/MIDI

### Audio-Reactive GLSL Shaders
```glsl
// GLSL with audio reactivity
uniform float audioLevel;
uniform float bassFreq;
uniform float trebleFreq;

void main() {
    vec2 uv = gl_FragCoord.xy / resolution;
    float intensity = audioLevel + bassFreq * 0.5 + trebleFreq * 0.3;
    vec3 color = sin(uv * intensity * 10.0) * 0.5 + 0.5;
    gl_FragColor = vec4(color, 1.0);
}
```

### MIDI-Controlled Parameters
```glsl
// GLSL with MIDI control
uniform float param1; // CC1 (Mod Wheel)
uniform float param2; // CC2
uniform float param3; // CC3

void main() {
    vec2 uv = gl_FragCoord.xy / resolution;
    float pattern = sin(uv.x * param1 + uv.y * param2 + time * param3);
    gl_FragColor = vec4(vec3(pattern * 0.5 + 0.5), 1.0);
}
```

## Examples

### Basic Texture Shader
```glsl
// Simple texture display with color correction
uniform sampler2D inputTexture;
uniform float brightness;
uniform float contrast;

void main() {
    vec2 uv = gl_FragCoord.xy / resolution;
    vec4 color = texture2D(inputTexture, uv);
    
    // Apply brightness and contrast
    color.rgb = (color.rgb - 0.5) * contrast + 0.5 + brightness;
    
    gl_FragColor = color;
}
```

### Audio-Visualizer Shader
```glsl
// Audio-reactive visualization
uniform float spectrum[256];
uniform float time;

void main() {
    vec2 uv = gl_FragCoord.xy / resolution;
    float freq = floor(uv.x * 256.0);
    float intensity = spectrum[int(freq)];
    
    vec3 color = vec3(
        sin(uv.x * 10.0 + time) * intensity,
        sin(uv.x * 5.0 + time * 1.5) * intensity * 0.7,
        sin(uv.x * 15.0 + time * 0.8) * intensity * 0.3
    );
    
    gl_FragColor = vec4(color, 1.0);
}
```

## Further Reading
- [OpenGL Shading Language Specification](https://www.opengl.org/documentation/glsl/)
- [WebGL Shader Reference](https://developer.mozilla.org/en-US/docs/Web/API/WebGLShader)
- [WGSL Specification](https://www.w3.org/TR/WGSL/)

---

**WGSL Shader Studio** - Professional GLSL development environment with real-time audio/MIDI integration.