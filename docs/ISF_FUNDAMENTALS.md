# ISF Fundamentals - Complete Reference

## Table of Contents

1. [Introduction to ISF](#introduction-to-isf)
2. [ISF Structure](#isf-structure)
3. [JSON Metadata](#json-metadata)
4. [Input Types](#input-types)
5. [Special Variables](#special-variables)
6. [Coordinate Systems](#coordinate-systems)
7. [ISF Functions](#isf-functions)
8. [Render Passes](#render-passes)
9. [Persistent Buffers](#persistent-buffers)
10. [Audio Integration](#audio-integration)
11. [Filter Kernels](#filter-kernels)
12. [Blending Modes](#blending-modes)
13. [Temporal Effects](#temporal-effects)
14. [Best Practices](#best-practices)
15. [Common Patterns](#common-patterns)
16. [ISF in WGSL Shader Studio](#isf-in-wgsl-shader-studio)

## Introduction to ISF

Interactive Shader Format (ISF) is an open specification for cross-platform shader effects. Originally developed for VDMX and other video applications, ISF provides a standardized way to define shader parameters, inputs, and metadata that can be interpreted by various host applications.

ISF combines:
- Standard GLSL fragment shaders
- JSON metadata describing parameters and inputs
- Special variables and functions for interactive effects
- Support for multiple render passes
- Audio integration capabilities

The key benefits of ISF include:
- Portability across different host applications
- Standardized parameter handling
- Rich metadata for user interfaces
- Support for temporal effects
- Audio-reactive capabilities

## ISF Structure

An ISF effect consists of two main components:

1. **GLSL Fragment Shader**: The actual rendering code
2. **JSON Metadata**: Describes parameters, inputs, and effect properties

### Basic ISF Structure

```glsl
/*{
    "CATEGORIES": [
        "Stylize"
    ],
    "DESCRIPTION": "Applies a simple color tint effect",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "NAME": "tintColor",
            "TYPE": "color",
            "DEFAULT": [
                1.0,
                0.0,
                0.0,
                1.0
            ]
        }
    ],
    "ISFVSN": "2"
}*/

varying vec2 left_coord;
varying vec2 right_coord;
varying vec2 above_coord;
varying vec2 below_coord;

void main() {
    vec4 color = IMG_THIS_PIXEL(inputImage);
    vec4 tint = tintColor;
    gl_FragColor = color * tint;
}
```

### ISF Version History

- **ISF V1**: Original version with basic functionality
- **ISF V2**: Enhanced version with additional features:
  - Support for vertex shaders
  - Multiple render passes
  - Persistent buffers
  - Audio input
  - Filter kernels

## JSON Metadata

The JSON metadata describes the effect's interface and behavior:

### Basic Metadata Fields

```json
{
    "CATEGORIES": ["Generators", "Stylize"],
    "DESCRIPTION": "A brief description of what the effect does",
    "ISFVSN": "2",
    "INPUTS": [],
    "PASSES": [],
    "IMPORTED": [],
    "CREDIT": "Author Name",
    "URL": "http://example.com",
    "VSN": "1.0"
}
```

### Categories

Categories help organize effects in host applications:

```json
"CATEGORIES": [
    "Blur",
    "Color Adjustment",
    "Distortion",
    "Feedback",
    "Generator",
    "Stylize",
    "Tile Effect",
    "Transition",
    "Utility",
    "Wipe"
]
```

## Input Types

ISF supports various input types for user interaction:

### Image Inputs

```json
{
    "NAME": "inputImage",
    "TYPE": "image"
}
```

### Color Inputs

```json
{
    "NAME": "tintColor",
    "TYPE": "color",
    "DEFAULT": [1.0, 0.0, 0.0, 1.0]
}
```

### Float Inputs

```json
{
    "NAME": "blurAmount",
    "TYPE": "float",
    "DEFAULT": 0.5,
    "MIN": 0.0,
    "MAX": 1.0
}
```

### Point2D Inputs

```json
{
    "NAME": "centerPoint",
    "TYPE": "point2D",
    "DEFAULT": [0.5, 0.5],
    "MIN": [0.0, 0.0],
    "MAX": [1.0, 1.0]
}
```

### Event Inputs

```json
{
    "NAME": "triggerEvent",
    "TYPE": "event"
}
```

### Bool Inputs

```json
{
    "NAME": "enableEffect",
    "TYPE": "bool",
    "DEFAULT": true
}
```

### Long Inputs (Pop-up Menus)

```json
{
    "NAME": "blendMode",
    "TYPE": "long",
    "DEFAULT": 0,
    "VALUES": [0, 1, 2],
    "LABELS": ["Normal", "Multiply", "Screen"]
}
```

## Special Variables

ISF provides special variables and functions for common operations:

### Coordinate Variables

```glsl
// Current fragment coordinates (normalized 0.0-1.0)
vec2 isf_FragNormCoord;

// Current fragment coordinates (pixel coordinates)
vec2 isf_FragCoord;
```

### Time Variables

```glsl
// Time in seconds since effect started
float TIME;

// Time delta since last frame
float TIMEDELTA;

// Date components
vec4 DATE; // year, month, day, time in seconds
```

### Resolution Variables

```glsl
// Render target size in pixels
vec2 RENDERSIZE;
```

### Image Sampling Functions

```glsl
// Sample current pass image at coordinates
vec4 IMG_NORM_PIXEL(sampler2D image, vec2 normPt);
vec4 IMG_THIS_PIXEL(sampler2D image);
vec4 IMG_PIXEL(sampler2D image, vec2 pt);

// Sample with linear interpolation
vec4 IMG_NORM_LINEAR(sampler2D image, vec2 normPt);
vec4 IMG_LINEAR(sampler2D image, vec2 pt);
```

### Neighbor Sampling

```glsl
// Sample neighboring pixels (for convolution effects)
varying vec2 left_coord;
varying vec2 right_coord;
varying vec2 above_coord;
varying vec2 below_coord;

vec4 LEFT_PIXEL(sampler2D image) { return IMG_NORM_PIXEL(image, left_coord); }
vec4 RIGHT_PIXEL(sampler2D image) { return IMG_NORM_PIXEL(image, right_coord); }
vec4 ABOVE_PIXEL(sampler2D image) { return IMG_NORM_PIXEL(image, above_coord); }
vec4 BELOW_PIXEL(sampler2D image) { return IMG_NORM_PIXEL(image, below_coord); }
```

## Coordinate Systems

ISF uses normalized coordinate systems:

### Normalized Coordinates (0.0 - 1.0)

```glsl
// Convert from pixel coordinates to normalized coordinates
vec2 normCoord = gl_FragCoord.xy / RENDERSIZE;

// Use normalized coordinates for sampling
vec4 color = IMG_NORM_PIXEL(inputImage, normCoord);
```

### Pixel Coordinates

```glsl
// Convert from normalized coordinates to pixel coordinates
vec2 pixelCoord = isf_FragNormCoord * RENDERSIZE;

// Use pixel coordinates for sampling
vec4 color = IMG_PIXEL(inputImage, pixelCoord);
```

### Aspect Ratio Handling

```glsl
// Maintain aspect ratio
float aspectRatio = RENDERSIZE.x / RENDERSIZE.y;
vec2 uv = isf_FragNormCoord;
uv.x *= aspectRatio;
```

## ISF Functions

### Core Sampling Functions

```glsl
// Basic sampling
vec4 color = IMG_THIS_PIXEL(inputImage);

// Normalized coordinate sampling
vec4 color = IMG_NORM_PIXEL(inputImage, vec2(0.5, 0.5));

// Pixel coordinate sampling
vec4 color = IMG_PIXEL(inputImage, vec2(100.0, 100.0));

// Linear interpolation sampling
vec4 color = IMG_NORM_LINEAR(inputImage, vec2(0.5, 0.5));
```

### Utility Functions

```glsl
// Clamp coordinates to valid range
vec2 clampCoord(vec2 coord) {
    return clamp(coord, vec2(0.0), vec2(1.0));
}

// Wrap coordinates
vec2 wrapCoord(vec2 coord) {
    return fract(coord);
}

// Mirror coordinates
vec2 mirrorCoord(vec2 coord) {
    return abs(coord - floor(coord) * 2.0 - 1.0);
}
```

## Render Passes

Multiple render passes allow for complex effects:

### Basic Multi-Pass Setup

```json
{
    "PASSES": [
        {
            "TARGET": "buffer1",
            "FLOAT": true
        },
        {
            "TARGET": "buffer2",
            "FLOAT": true
        },
        {
            // Final pass renders to output
        }
    ]
}
```

### Multi-Pass Shader Example

```glsl
/*{
    "CATEGORIES": ["Feedback"],
    "DESCRIPTION": "Simple feedback effect",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "NAME": "feedbackAmount",
            "TYPE": "float",
            "DEFAULT": 0.9,
            "MIN": 0.0,
            "MAX": 1.0
        }
    ],
    "PASSES": [
        {
            "TARGET": "feedbackBuffer",
            "FLOAT": true,
            "PERSISTENT": true
        },
        {
            // Final pass
        }
    ]
}*/

void main() {
    vec4 currentFrame = IMG_THIS_PIXEL(inputImage);
    vec4 previousFrame = IMG_NORM_PIXEL(feedbackBuffer, isf_FragNormCoord);
    
    vec4 result = mix(currentFrame, previousFrame, feedbackAmount);
    gl_FragColor = result;
}
```

## Persistent Buffers

Persistent buffers maintain state between frames:

### Persistent Buffer Setup

```json
{
    "PASSES": [
        {
            "TARGET": "persistentBuffer",
            "FLOAT": true,
            "PERSISTENT": true
        },
        {
            // Final pass
        }
    ]
}
```

### Using Persistent Buffers

```glsl
void main() {
    // Read from persistent buffer
    vec4 prevState = IMG_NORM_PIXEL(persistentBuffer, isf_FragNormCoord);
    
    // Update state
    vec4 newState = ProcessState(prevState, IMG_THIS_PIXEL(inputImage));
    
    // Write to persistent buffer in first pass
    if (PASSINDEX == 0) {
        gl_FragColor = newState;
    } else {
        // Output final result in subsequent passes
        gl_FragColor = FinalProcess(newState);
    }
}
```

## Audio Integration

ISF supports audio-reactive effects through FFT data:

### Audio Input Setup

```json
{
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "NAME": "audioLevel",
            "TYPE": "audio"
        }
    ]
}
```

### Audio Processing

```glsl
void main() {
    vec4 color = IMG_THIS_PIXEL(inputImage);
    
    // Get audio level (0.0 - 1.0)
    float level = audioLevel;
    
    // Apply audio modulation
    color.rgb *= (1.0 + level);
    
    gl_FragColor = color;
}
```

### FFT Bands

```json
{
    "INPUTS": [
        {
            "NAME": "fftBands",
            "TYPE": "audioFFT"
        }
    ]
}
```

```glsl
void main() {
    // Access FFT bands (typically 8-16 bands)
    float bass = fftBands[0];      // Low frequencies
    float mid = fftBands[4];       // Mid frequencies
    float treble = fftBands[7];    // High frequencies
    
    // Use for visual effects
    float intensity = bass * 2.0 + mid * 1.0 + treble * 0.5;
    
    vec4 color = IMG_THIS_PIXEL(inputImage);
    color.rgb *= (1.0 + intensity);
    
    gl_FragColor = color;
}
```

## Filter Kernels

Filter kernels enable convolution operations:

### Kernel Definition

```json
{
    "KERNEL": {
        "RADIUS": 2,
        "WEIGHTS": [
            1, 2, 1,
            2, 4, 2,
            1, 2, 1
        ]
    }
}
```

### Using Kernels

```glsl
void main() {
    vec4 result = kernelSample();
    gl_FragColor = result;
}
```

## Blending Modes

ISF effects can implement various blending modes:

### Common Blending Functions

```glsl
// Multiply blend
vec3 multiply(vec3 a, vec3 b) {
    return a * b;
}

// Screen blend
vec3 screen(vec3 a, vec3 b) {
    return 1.0 - (1.0 - a) * (1.0 - b);
}

// Overlay blend
vec3 overlay(vec3 a, vec3 b) {
    return mix(1.0 - 2.0 * (1.0 - a) * (1.0 - b),
               2.0 * a * b,
               step(0.5, a));
}

// Soft light blend
vec3 softLight(vec3 a, vec3 b) {
    return mix(sqrt(a) * b, 1.0 - sqrt(1.0 - a) * (1.0 - b), step(0.5, b));
}
```

## Temporal Effects

ISF supports time-based animations:

### Time-Based Animations

```glsl
void main() {
    vec2 uv = isf_FragNormCoord;
    
    // Animate UV coordinates
    uv.x += sin(TIME) * 0.1;
    uv.y += cos(TIME) * 0.1;
    
    vec4 color = IMG_NORM_PIXEL(inputImage, uv);
    gl_FragColor = color;
}
```

### Frame-Based Effects

```glsl
void main() {
    // Create pulsing effect
    float pulse = sin(TIME * 3.14159 * 2.0) * 0.5 + 0.5;
    
    vec4 color = IMG_THIS_PIXEL(inputImage);
    color.rgb *= (0.5 + pulse * 0.5);
    
    gl_FragColor = color;
}
```

## Best Practices

### Performance Optimization

1. **Minimize Texture Lookups**
```glsl
// Inefficient: Multiple texture lookups
vec4 color1 = IMG_NORM_PIXEL(inputImage, uv + offset1);
vec4 color2 = IMG_NORM_PIXEL(inputImage, uv + offset2);
vec4 color3 = IMG_NORM_PIXEL(inputImage, uv + offset3);

// Better: Pre-calculate coordinates
vec2 baseUV = isf_FragNormCoord;
vec4 center = IMG_NORM_PIXEL(inputImage, baseUV);
vec4 left = IMG_NORM_PIXEL(inputImage, baseUV + vec2(-pixelSize.x, 0.0));
vec4 right = IMG_NORM_PIXEL(inputImage, baseUV + vec2(pixelSize.x, 0.0));
```

2. **Use Appropriate Precision**
```glsl
// High precision for positions
varying vec2 textureCoordinate;

// Medium precision for colors
varying mediump vec4 colorVarying;
```

3. **Avoid Dynamic Branching**
```glsl
// Inefficient: Dynamic branching
if (condition) {
    result = expensiveCalculationA();
} else {
    result = expensiveCalculationB();
}

// Better: Use mix functions
result = mix(expensiveCalculationA(), expensiveCalculationB(), float(condition));
```

### Code Organization

1. **Modular Functions**
```glsl
// Break complex operations into smaller functions
vec3 applyColorCorrection(vec3 color, vec3 correction) {
    return color * correction;
}

vec2 distortCoordinates(vec2 coord, float amount) {
    return coord + sin(coord * 10.0) * amount;
}
```

2. **Consistent Naming**
```glsl
// Use descriptive names
uniform sampler2D inputImage;
uniform vec4 tintColor;
uniform float blurRadius;

// Prefix temporary variables
vec4 tempColor;
vec2 tempCoord;
```

## Common Patterns

### Blur Effects

```glsl
/*{
    "CATEGORIES": ["Blur"],
    "DESCRIPTION": "Simple Gaussian blur",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "NAME": "blurAmount",
            "TYPE": "float",
            "DEFAULT": 1.0,
            "MIN": 0.0,
            "MAX": 10.0
        }
    ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    vec2 pixel = 1.0 / RENDERSIZE;
    
    vec4 color = vec4(0.0);
    float total = 0.0;
    
    // Simple box blur
    for (float x = -blurAmount; x <= blurAmount; x += 1.0) {
        for (float y = -blurAmount; y <= blurAmount; y += 1.0) {
            vec2 offset = vec2(x, y) * pixel;
            color += IMG_NORM_PIXEL(inputImage, uv + offset);
            total += 1.0;
        }
    }
    
    gl_FragColor = color / total;
}
```

### Distortion Effects

```glsl
/*{
    "CATEGORIES": ["Distortion"],
    "DESCRIPTION": "Simple wave distortion",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "NAME": "waveAmount",
            "TYPE": "float",
            "DEFAULT": 0.1,
            "MIN": 0.0,
            "MAX": 1.0
        },
        {
            "NAME": "waveFrequency",
            "TYPE": "float",
            "DEFAULT": 10.0,
            "MIN": 1.0,
            "MAX": 50.0
        }
    ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    
    // Apply wave distortion
    uv.y += sin(uv.x * waveFrequency + TIME) * waveAmount;
    uv.x += cos(uv.y * waveFrequency + TIME) * waveAmount;
    
    vec4 color = IMG_NORM_PIXEL(inputImage, uv);
    gl_FragColor = color;
}
```

### Color Effects

```glsl
/*{
    "CATEGORIES": ["Color Adjustment"],
    "DESCRIPTION": "HSV adjustment",
    "INPUTS": [
        {
            "NAME": "inputImage",
            "TYPE": "image"
        },
        {
            "NAME": "hueShift",
            "TYPE": "float",
            "DEFAULT": 0.0,
            "MIN": 0.0,
            "MAX": 1.0
        },
        {
            "NAME": "saturation",
            "TYPE": "float",
            "DEFAULT": 1.0,
            "MIN": 0.0,
            "MAX": 2.0
        },
        {
            "NAME": "brightness",
            "TYPE": "float",
            "DEFAULT": 1.0,
            "MIN": 0.0,
            "MAX": 2.0
        }
    ]
}*/

vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    
    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    vec4 color = IMG_THIS_PIXEL(inputImage);
    vec3 hsv = rgb2hsv(color.rgb);
    
    // Apply adjustments
    hsv.x = fract(hsv.x + hueShift);
    hsv.y = clamp(hsv.y * saturation, 0.0, 1.0);
    hsv.z = clamp(hsv.z * brightness, 0.0, 1.0);
    
    color.rgb = hsv2rgb(hsv);
    gl_FragColor = color;
}
```

## ISF in WGSL Shader Studio

### ISF Import and Conversion

WGSL Shader Studio provides tools to import and convert ISF effects:

1. **ISF Loader**: Parses ISF JSON metadata
2. **GLSL to WGSL Converter**: Converts ISF GLSL to WGSL
3. **Parameter Mapping**: Maps ISF inputs to WGSL uniforms
4. **Render Pass Management**: Handles multi-pass ISF effects

### ISF Integration Example

```wgsl
// Converted ISF effect to WGSL
struct ISFUniforms {
    // ISF special variables
    time: f32,
    timeDelta: f32,
    renderSize: vec2<f32>,
    
    // ISF inputs
    tintColor: vec4<f32>,
    blurAmount: f32,
}

@group(0) @binding(0) var<uniform> isfUniforms: ISFUniforms;
@group(0) @binding(1) var inputImage: texture_2d<f32>;
@group(0) @binding(2) var linearSampler: sampler;

@fragment
fn isfMain(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = fragCoord.xy / isfUniforms.renderSize;
    let color = textureSample(inputImage, linearSampler, uv);
    let result = color * isfUniforms.tintColor;
    
    return result;
}
```

### ISF Parameter Handling

```wgsl
// Dynamic parameter updates
fn updateISFParameters(paramName: ptr<function, u32>, paramValue: f32) {
    switch (*paramName) {
        case "blurAmount": {
            // Update blur amount uniform
        }
        case "tintColor": {
            // Update color uniform
        }
        default: {
            // Handle unknown parameters
        }
    }
}
```

### ISF Render Pipeline

```wgsl
// Multi-pass ISF rendering
struct ISFPass {
    target: texture_2d<f32>,
    persistent: bool,
    floatBuffer: bool,
}

fn renderISFEffect(passes: array<ISFPass>) {
    for (var i: u32 = 0u; i < arrayLength(passes); i++) {
        // Bind pass target
        // Execute pass shader
        // Handle persistent buffers
    }
}
```

---
*End of ISF Fundamentals*

*Next steps:*
*- Shader Conversion Framework*
*- Application Usage Guide*