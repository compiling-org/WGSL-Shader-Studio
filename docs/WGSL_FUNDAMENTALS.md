# WGSL Fundamentals - Complete Reference

## Table of Contents

1. [Introduction to WGSL](#introduction-to-wgsl)
2. [Syntax and Semantics](#syntax-and-semantics)
3. [Data Types](#data-types)
4. [Functions and Control Flow](#functions-and-control-flow)
5. [Built-in Variables and Functions](#built-in-variables-and-functions)
6. [Resource Binding](#resource-binding)
7. [Memory Model](#memory-model)
8. [Shader Stages](#shader-stages)
9. [Best Practices](#best-practices)
10. [Common Patterns](#common-patterns)

## Introduction to WGSL

WebGPU Shading Language (WGSL) is a modern, portable shading language designed specifically for WebGPU. Unlike GLSL or HLSL which evolved from C-like syntax, WGSL was designed from the ground up with explicit goals:

- **Safety**: Strong typing, explicit memory model, bounds checking
- **Portability**: Consistent behavior across all WebGPU implementations
- **Clarity**: Explicit syntax that makes intentions clear
- **Performance**: Designed for efficient compilation and execution

WGSL serves as the native shading language for WebGPU, which is the next-generation graphics API for the web. It replaces WebGL's reliance on OpenGL ES Shading Language (ESSL) with a more modern and safer alternative.

## Syntax and Semantics

### Basic Structure

WGSL shaders consist of declarations at module scope, which include:
- Variable declarations
- Function definitions
- Type aliases
- Struct definitions
- Constant values

```wgsl
// Module-scope declarations
@group(0) @binding(0) var<uniform> time: f32;
@group(0) @binding(1) var myTexture: texture_2d<f32>;
@group(0) @binding(2) var mySampler: sampler;

// Struct definitions
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Function definitions
@vertex
fn vertexMain(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn fragmentMain(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let color = textureSample(myTexture, mySampler, uv);
    return vec4<f32>(color.rgb, 1.0);
}
```

### Variable Declarations

WGSL requires explicit type annotations for all variable declarations:

```wgsl
// Scalar variables
let a: f32 = 1.0;    // Immutable float
var b: i32 = 2;      // Mutable integer
const c: bool = true; // Compile-time constant

// Vector variables
let vec2Var: vec2<f32> = vec2(1.0, 2.0);
let vec3Var: vec3<i32> = vec3(1, 2, 3);
let vec4Var: vec4<bool> = vec4(true, false, true, false);

// Matrix variables
let mat2x2Var: mat2x2<f32> = mat2x2(1.0, 0.0, 0.0, 1.0);
let mat4x4Var: mat4x4<f32> = mat4x4(); // Identity matrix

// Array variables
let arrayVar: array<f32, 4> = array(1.0, 2.0, 3.0, 4.0);
var runtimeArrayVar: array<f32>; // Runtime-sized array (compute shaders only)
```

### Type Aliases

You can create type aliases to simplify complex type signatures:

```wgsl
alias Position2D = vec2<f32>;
alias Color = vec4<f32>;
alias Matrix4x4 = mat4x4<f32>;

let pos: Position2D = vec2(0.5, 0.5);
let color: Color = vec4(1.0, 0.0, 0.0, 1.0);
```

## Data Types

### Scalar Types

WGSL supports several scalar types:

- `f32`: 32-bit floating-point number
- `f16`: 16-bit floating-point number (requires extension)
- `i32`: 32-bit signed integer
- `u32`: 32-bit unsigned integer
- `bool`: Boolean value (true or false)

```wgsl
let floatValue: f32 = 3.14159;
let intValue: i32 = -42;
let uintValue: u32 = 42u; // Note the 'u' suffix
let boolValue: bool = true;
```

### Vector Types

Vector types combine multiple scalar values:

- `vec2<T>`: 2-component vector
- `vec3<T>`: 3-component vector
- `vec4<T>`: 4-component vector

Where T is a scalar type (f32, i32, u32, bool).

```wgsl
// Creating vectors
let v1: vec2<f32> = vec2(1.0, 2.0);
let v2: vec3<f32> = vec3(1.0, 2.0, 3.0);
let v3: vec4<f32> = vec4(1.0, 2.0, 3.0, 4.0);

// Alternative construction methods
let v4: vec3<f32> = vec3(v1, 3.0); // Construct from vec2 and scalar
let v5: vec4<f32> = vec4(v2, 4.0); // Construct from vec3 and scalar

// Accessing vector components
let x: f32 = v1.x;  // First component
let y: f32 = v1.y;  // Second component
let r: f32 = v1.r;  // First component (color notation)
let g: f32 = v1.g;  // Second component (color notation)

// Swizzling
let xy: vec2<f32> = v2.xy;  // First two components
let rgb: vec3<f32> = v3.rgb; // First three components as color
let bgr: vec3<f32> = v3.bgr; // Components in reverse order
```

### Matrix Types

Matrix types represent 2D arrays of scalar values:

- `matCxR<T>`: C columns, R rows matrix
- Common matrices: `mat2x2<f32>`, `mat3x3<f32>`, `mat4x4<f32>`

```wgsl
// Creating matrices
let m1: mat2x2<f32> = mat2x2(1.0, 0.0, 0.0, 1.0);
let m2: mat3x3<f32> = mat3x3(); // Identity matrix
let m3: mat4x4<f32> = mat4x4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
);

// Matrix operations
let result: vec4<f32> = m3 * vec4(1.0, 2.0, 3.0, 1.0);
let determinant: f32 = determinant(m1);
let inverse: mat2x2<f32> = inverse(m1);
```

### Array Types

Arrays can be fixed-size or runtime-sized (in compute shaders):

```wgsl
// Fixed-size arrays
let fixedArray: array<f32, 4> = array(1.0, 2.0, 3.0, 4.0);
let size: u32 = arrayLength(&fixedArray); // Returns 4

// Accessing array elements
let firstElement: f32 = fixedArray[0];
let lastElement: f32 = fixedArray[3];

// Runtime-sized arrays (compute shaders only)
var<storage, read_write> dynamicArray: array<f32>;
```

### Struct Types

Structures group related data together:

```wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct UniformData {
    modelViewProjection: mat4x4<f32>,
    time: f32,
    color: vec4<f32>,
}

// Nested structures
struct Material {
    diffuse: vec3<f32>,
    specular: vec3<f32>,
    shininess: f32,
}

struct PerObjectData {
    transform: mat4x4<f32>,
    material: Material,
}
```

## Functions and Control Flow

### Function Definitions

Functions in WGSL are defined with explicit parameter and return types:

```wgsl
// Simple function
fn add(a: f32, b: f32) -> f32 {
    return a + b;
}

// Function with no parameters
fn getCurrentTime() -> f32 {
    return time; // Assuming 'time' is a module-scope variable
}

// Function with no return value
fn debugPrint(value: f32) {
    // Debug output (implementation dependent)
}

// Function with multiple return values using structs
struct CalculationResult {
    sum: f32,
    product: f32,
    difference: f32,
}

fn calculate(a: f32, b: f32) -> CalculationResult {
    return CalculationResult(
        a + b,  // sum
        a * b,  // product
        a - b   // difference
    );
}
```

### Control Flow Statements

WGSL supports standard control flow constructs:

```wgsl
// If statements
fn conditionalExample(value: f32) -> f32 {
    if (value > 0.0) {
        return value;
    } else if (value < 0.0) {
        return -value;
    } else {
        return 0.0;
    }
}

// Loops
fn loopExample(count: u32) -> f32 {
    var result: f32 = 0.0;
    var i: u32 = 0u;
    
    // For-like loop
    loop {
        if (i >= count) {
            break;
        }
        result = result + f32(i);
        continuing {
            i = i + 1u;
        }
    }
    
    return result;
}

// While loop
fn whileExample(threshold: f32) -> f32 {
    var value: f32 = 1.0;
    while (value < threshold) {
        value = value * 2.0;
    }
    return value;
}

// Switch statement
fn switchExample(selector: u32) -> f32 {
    switch (selector) {
        case 0u: {
            return 1.0;
        }
        case 1u, 2u: {
            return 2.0;
        }
        default: {
            return 0.0;
        }
    }
}
```

### Short-Circuit Evaluation

Logical operators use short-circuit evaluation:

```wgsl
fn shortCircuitExample(a: bool, b: bool) -> bool {
    // If 'a' is false, 'expensiveFunction()' will not be called
    return a && expensiveFunction(b);
    
    // If 'a' is true, 'anotherExpensiveFunction()' will not be called
    return a || anotherExpensiveFunction(b);
}
```

## Built-in Variables and Functions

### Shader Stage Built-ins

Different shader stages have specific built-in variables:

#### Vertex Shader Built-ins

```wgsl
@builtin(vertex_index) vertexIndex: u32          // Index of the current vertex
@builtin(instance_index) instanceIndex: u32      // Index of the current instance
@builtin(position) position: vec4<f32>          // Clip-space output position
```

#### Fragment Shader Built-ins

```wgsl
@builtin(front_facing) frontFacing: bool         // True if front-facing primitive
@builtin(sample_index) sampleIndex: u32         // Sample index for MSAA
@builtin(sample_mask) sampleMask: u32           // Sample mask input/output
@builtin(position) position: vec4<f32>          // Fragment position
```

#### Compute Shader Built-ins

```wgsl
@builtin(global_invocation_id) globalId: vec3<u32>    // Global invocation ID
@builtin(local_invocation_id) localId: vec3<u32>      // Local invocation ID
@builtin(workgroup_id) workgroupId: vec3<u32>         // Workgroup ID
@builtin(num_workgroups) numWorkgroups: vec3<u32>     // Number of workgroups
```

### Built-in Functions

WGSL provides a rich set of built-in functions for mathematical operations:

#### Mathematical Functions

```wgsl
// Basic math
let absValue: f32 = abs(-5.0);
let ceilValue: f32 = ceil(3.14);
let floorValue: f32 = floor(3.14);
let roundValue: f32 = round(3.14);

// Trigonometric functions
let sinValue: f32 = sin(3.14159 / 2.0);
let cosValue: f32 = cos(3.14159 / 2.0);
let tanValue: f32 = tan(3.14159 / 4.0);

// Exponential functions
let expValue: f32 = exp(1.0);
let logValue: f32 = log(2.71828);
let powValue: f32 = pow(2.0, 3.0);

// Geometric functions
let lengthValue: f32 = length(vec3(1.0, 2.0, 3.0));
let distanceValue: f32 = distance(vec2(0.0, 0.0), vec2(1.0, 1.0));
let dotProduct: f32 = dot(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
let crossProduct: vec3<f32> = cross(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
let normalized: vec3<f32> = normalize(vec3(1.0, 2.0, 3.0));
```

#### Texture Functions

```wgsl
// Basic sampling
let sampledColor: vec4<f32> = textureSample(myTexture, mySampler, uv);

// Level-of-detail sampling
let lodColor: vec4<f32> = textureSampleLevel(myTexture, mySampler, uv, 1.5);

// Gradient sampling
let gradColor: vec4<f32> = textureSampleGrad(myTexture, mySampler, uv, dpdx, dpdy);

// Size queries
let textureSize: vec2<u32> = textureDimensions(myTexture);
let textureLevels: u32 = textureNumLevels(myTexture);
```

#### Atomic Operations

```wgsl
// Atomic operations on storage variables
var<storage, read_write> atomicCounter: atomic<u32>;

fn incrementCounter() -> u32 {
    return atomicAdd(&atomicCounter, 1u);
}

fn compareAndSwap(oldValue: u32, newValue: u32) -> u32 {
    return atomicCompareExchangeWeak(&atomicCounter, oldValue, newValue).old_value;
}
```

## Resource Binding

### Binding Model

WGSL uses a binding model based on groups and bindings:

```wgsl
// Uniform buffer
@group(0) @binding(0) var<uniform> uniformData: UniformStruct;

// Storage buffer
@group(0) @binding(1) var<storage, read> readOnlyBuffer: array<f32>;
@group(0) @binding(2) var<storage, read_write> readWriteBuffer: array<f32>;

// Textures
@group(1) @binding(0) var diffuseTexture: texture_2d<f32>;
@group(1) @binding(1) var normalTexture: texture_2d<f32>;

// Samplers
@group(1) @binding(2) var linearSampler: sampler;
@group(1) @binding(3) var nearestSampler: sampler;

// Storage textures
@group(2) @binding(0) var storageTexture: texture_storage_2d<rgba8unorm, write>;
```

### Buffer Layout

Uniform and storage buffers require explicit layout specifications:

```wgsl
// Uniform buffer layout
struct CameraUniforms {
    viewProjection: mat4x4<f32>,  // 64 bytes (16-byte aligned)
    position: vec3<f32>,          // 12 bytes
    padding: f32,                 // 4 bytes (for alignment)
    time: f32,                    // 4 bytes
    // Implicit padding to 16-byte boundary
};

// Storage buffer layout
struct Particle {
    position: vec3<f32>,  // 12 bytes
    lifetime: f32,        // 4 bytes
    velocity: vec3<f32>,  // 12 bytes
    padding: f32,         // 4 bytes (for alignment)
    // Total: 32 bytes per particle
};
```

## Memory Model

### Address Spaces

WGSL defines several address spaces for variables:

```wgsl
// Function address space (default for function-local variables)
var localVar: f32 = 1.0;

// Private address space (module-scope variables)
var<private> privateVar: f32 = 2.0;

// Workgroup address space (shared among workgroup invocations)
var<workgroup> workgroupArray: array<f32, 256>;

// Uniform address space (read-only, uniform across invocations)
@group(0) @binding(0) var<uniform> uniformBuffer: UniformStruct;

// Storage address space (can be read-write)
@group(0) @binding(1) var<storage, read_write> storageBuffer: array<f32>;

// Handle address space (textures, samplers)
@group(1) @binding(0) var myTexture: texture_2d<f32>;
@group(1) @binding(1) var mySampler: sampler;
```

### Memory Access Patterns

```wgsl
// Coalesced memory access (preferred)
@compute @workgroup_size(64)
fn coalescedAccess(@builtin(global_invocation_id) gid: vec3<u32>) {
    // Adjacent work items access adjacent memory locations
    storageBuffer[gid.x] = f32(gid.x);
}

// Strided memory access (less efficient)
@compute @workgroup_size(64)
fn stridedAccess(@builtin(global_invocation_id) gid: vec3<u32>) {
    // Work items access memory locations far apart
    storageBuffer[gid.x * 16u] = f32(gid.x);
}
```

## Shader Stages

### Vertex Shaders

Vertex shaders process vertices and output clip-space positions:

```wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) worldPosition: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertexMain(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Transform vertex position to clip space
    output.position = uniforms.viewProjection * vec4(input.position, 1.0);
    
    // Pass through other attributes
    output.worldPosition = input.position;
    output.normal = input.normal;
    output.uv = input.uv;
    
    return output;
}
```

### Fragment Shaders

Fragment shaders process fragments and output colors:

```wgsl
@fragment
fn fragmentMain(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let baseColor = textureSample(diffuseTexture, linearSampler, input.uv);
    
    // Simple lighting calculation
    let lightDir = normalize(uniforms.lightPosition - input.worldPosition);
    let normal = normalize(input.normal);
    let diffuse = max(dot(normal, lightDir), 0.0);
    
    // Apply lighting
    let litColor = baseColor.rgb * diffuse;
    
    return vec4(litColor, baseColor.a);
}
```

### Compute Shaders

Compute shaders perform general-purpose computations:

```wgsl
@compute @workgroup_size(8, 8, 1)
fn computeMain(@builtin(global_invocation_id) gid: vec3<u32>) {
    // Bounds checking
    if (gid.x >= uniforms.textureWidth || gid.y >= uniforms.textureHeight) {
        return;
    }
    
    // Calculate texture coordinates
    let uv = vec2(f32(gid.x) / f32(uniforms.textureWidth),
                  f32(gid.y) / f32(uniforms.textureHeight));
    
    // Perform computation
    let result = processPixel(uv);
    
    // Write result to storage texture
    textureStore(outputTexture, gid.xy, result);
}
```

## Best Practices

### Performance Optimization

1. **Minimize Dynamic Branching**
```wgsl
// Inefficient: Dynamic branching
if (condition) {
    result = expensiveCalculationA();
} else {
    result = expensiveCalculationB();
}

// Better: Use mix/select functions
result = select(expensiveCalculationA(), expensiveCalculationB(), condition);
```

2. **Coalesce Memory Access**
```wgsl
// Good: Adjacent threads access adjacent memory
let value = buffer[globalId.x];

// Poor: Strided access pattern
let value = buffer[globalId.x * stride];
```

3. **Use Appropriate Precision**
```wgsl
// Use f32 for most calculations
let precise: f32 = 1.0;

// Use f16 for storage when precision allows (with extension)
let compact: f16 = 1.0h;
```

### Code Organization

1. **Modular Functions**
```wgsl
// Break complex operations into smaller functions
fn calculateLighting(normal: vec3<f32>, lightDir: vec3<f32>) -> f32 {
    return max(dot(normalize(normal), normalize(lightDir)), 0.0);
}

fn applyFog(color: vec3<f32>, distance: f32) -> vec3<f32> {
    let fogFactor = 1.0 - exp(-uniforms.fogDensity * distance);
    return mix(color, uniforms.fogColor, fogFactor);
}
```

2. **Consistent Naming**
```wgsl
// Use descriptive names
struct DirectionalLight {
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
}

// Prefix uniforms consistently
struct Uniforms {
    uModelViewProjection: mat4x4<f32>,
    uCameraPosition: vec3<f32>,
    uTime: f32,
}
```

## Common Patterns

### Noise Generation

```wgsl
// Simple 2D noise function
fn hash22(p: vec2<f32>) -> vec2<f32> {
    var p3: vec3<f32> = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yxz + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn noise2D(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash22(i);
    let b = hash22(i + vec2(1.0, 0.0));
    let c = hash22(i + vec2(0.0, 1.0));
    let d = hash22(i + vec2(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a.x, b.x, u.x) +
           (c.x - a.x) * u.y * (1.0 - u.x) +
           (d.x - b.x) * u.x * u.y;
}
```

### Color Manipulation

```wgsl
// HSV to RGB conversion
fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// Gamma correction
fn gammaEncode(color: vec3<f32>, gamma: f32) -> vec3<f32> {
    return pow(color, vec3(1.0 / gamma));
}

fn gammaDecode(color: vec3<f32>, gamma: f32) -> vec3<f32> {
    return pow(color, vec3(gamma));
}
```

### Mathematical Utilities

```wgsl
// Smooth step function
fn smoothStep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + t * (b - a);
}

// Remapping function
fn remap(value: f32, inMin: f32, inMax: f32, outMin: f32, outMax: f32) -> f32 {
    return outMin + (outMax - outMin) * (value - inMin) / (inMax - inMin);
}
```

---
*End of WGSL Fundamentals*

*Next steps:*
*- GLSL Fundamentals*
*- HLSL Fundamentals*
*- ISF Deep Dive*
*- Shader Conversion Framework*
*- Application Usage Guide*