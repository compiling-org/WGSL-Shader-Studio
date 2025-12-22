# GLSL Fundamentals - Complete Reference

## Table of Contents

1. [Introduction to GLSL](#introduction-to-glsl)
2. [Basic Syntax](#basic-syntax)
3. [Data Types](#data-types)
4. [Variables and Qualifiers](#variables-and-qualifiers)
5. [Functions](#functions)
6. [Shader Stages](#shader-stages)
7. [Built-in Variables](#built-in-variables)
8. [Textures and Samplers](#textures-and-samplers)
9. [Uniforms and Attributes](#uniforms-and-attributes)
10. [Interface Blocks](#interface-blocks)
11. [Geometry Shaders](#geometry-shaders)
12. [Compute Shaders](#compute-shaders)
13. [Best Practices](#best-practices)
14. [Common Patterns](#common-patterns)

## Introduction to GLSL

OpenGL Shading Language (GLSL) is a high-level shading language based on the C programming language. It's used to write shaders for the OpenGL family of graphics APIs, including OpenGL ES for mobile platforms and WebGL for web browsers.

GLSL has evolved through multiple versions, with each version adding new features and capabilities:

- **GLSL 1.10**: Original version for OpenGL 2.0
- **GLSL 1.20**: Added support for OpenGL 2.1
- **GLSL 1.30**: Major revision for OpenGL 3.0
- **GLSL 1.40**: For OpenGL 3.1
- **GLSL 1.50**: For OpenGL 3.2
- **GLSL 3.30**: For OpenGL 3.3
- **GLSL 4.00**: For OpenGL 4.0
- **GLSL 4.10**: For OpenGL 4.1
- **GLSL 4.20**: For OpenGL 4.2
- **GLSL 4.30**: For OpenGL 4.3
- **GLSL 4.40**: For OpenGL 4.4
- **GLSL 4.50**: For OpenGL 4.5
- **GLSL 4.60**: For OpenGL 4.6

For WebGL, there are two versions:
- **WebGL GLSL ES 1.0**: Based on OpenGL ES Shading Language 1.0
- **WebGL GLSL ES 3.0**: Based on OpenGL ES Shading Language 3.0

## Basic Syntax

GLSL syntax is similar to C, with some additions for graphics programming:

```glsl
#version 330 core

// Preprocessor directives
#define PI 3.14159265359
#define MAX_LIGHTS 8

// Global variables
uniform float time;
uniform vec2 resolution;
attribute vec3 position;
varying vec3 fragColor;

// Function declaration
vec3 calculateLighting(vec3 normal, vec3 lightDir);

// Main function
void main() {
    // Local variables
    vec3 normal = normalize(cross(dFdx(position), dFdy(position)));
    vec3 lightDir = normalize(vec3(1.0, 1.0, 1.0));
    
    // Function call
    fragColor = calculateLighting(normal, lightDir);
    
    // Built-in variable assignment
    gl_Position = vec4(position, 1.0);
}
```

### Comments

```glsl
// Single line comment

/*
Multi-line comment
Can span multiple lines
*/

/**
 * Documentation comment
 * Used for documenting functions and variables
 */
```

## Data Types

### Scalar Types

GLSL supports several scalar types:

```glsl
// Floating point types
float a = 1.0;        // 32-bit IEEE 754 float
double b = 2.0;       // 64-bit IEEE 754 double (GLSL 4.00+)
half c = 3.0;         // 16-bit half float (OpenGL ES 3.0+)

// Integer types
int d = 4;            // 32-bit signed integer
uint e = 5u;          // 32-bit unsigned integer
short f = 6;          // 16-bit signed integer (OpenGL ES 3.1+)
ushort g = 7;         // 16-bit unsigned integer (OpenGL ES 3.1+)
int64_t h = 8;        // 64-bit signed integer (GLSL 4.50+)
uint64_t i = 9;       // 64-bit unsigned integer (GLSL 4.50+)

// Boolean type
bool j = true;        // Boolean value
```

### Vector Types

Vector types combine multiple scalar values:

```glsl
// 2-component vectors
vec2 v2f = vec2(1.0, 2.0);        // Float vector
ivec2 v2i = ivec2(1, 2);          // Integer vector
uvec2 v2u = uvec2(1u, 2u);        // Unsigned integer vector
bvec2 v2b = bvec2(true, false);    // Boolean vector

// 3-component vectors
vec3 v3f = vec3(1.0, 2.0, 3.0);
ivec3 v3i = ivec3(1, 2, 3);
uvec3 v3u = uvec3(1u, 2u, 3u);
bvec3 v3b = bvec3(true, false, true);

// 4-component vectors
vec4 v4f = vec4(1.0, 2.0, 3.0, 4.0);
ivec4 v4i = ivec4(1, 2, 3, 4);
uvec4 v4u = uvec4(1u, 2u, 3u, 4u);
bvec4 v4b = bvec4(true, false, true, false);

// Vector construction
vec3 constructed = vec3(v2f, 3.0);     // From vec2 and scalar
vec4 constructed2 = vec4(v3f, 4.0);    // From vec3 and scalar

// Vector swizzling
vec2 xy = v4f.xy;       // First two components
vec3 rgb = v4f.rgb;     // First three components as color
vec3 bgr = v4f.bgr;     // Components in reverse order
vec4 xyzw = v4f.xyzw;   // All components
```

### Matrix Types

Matrix types represent 2D arrays of scalar values:

```glsl
// Matrix types (column-major storage)
mat2 m2 = mat2(1.0);                    // 2x2 matrix
mat3 m3 = mat3(1.0);                    // 3x3 matrix
mat4 m4 = mat4(1.0);                    // 4x4 matrix

// Non-square matrices (GLSL 1.20+)
mat2x3 m23 = mat2x3(1.0);              // 2 columns, 3 rows
mat3x2 m32 = mat3x2(1.0);              // 3 columns, 2 rows
mat2x4 m24 = mat2x4(1.0);              // 2 columns, 4 rows
mat4x2 m42 = mat4x2(1.0);              // 4 columns, 2 rows
mat3x4 m34 = mat3x4(1.0);              // 3 columns, 4 rows
mat4x3 m43 = mat4x3(1.0);              // 4 columns, 3 rows

// Matrix construction
mat2 constructed = mat2(1.0, 0.0, 0.0, 1.0);  // Column by column
mat3 diagonal = mat3(vec3(1.0, 0.0, 0.0),      // Column by column
                     vec3(0.0, 1.0, 0.0),
                     vec3(0.0, 0.0, 1.0));

// Matrix access
float element = m4[0][1];               // First column, second row
vec4 column = m4[0];                    // First column
```

### Sampler Types

Sampler types are used for texture sampling:

```glsl
// 1D samplers
sampler1D s1D;                          // Floating-point 1D texture
isampler1D is1D;                        // Integer 1D texture
usampler1D us1D;                        // Unsigned integer 1D texture

// 2D samplers
sampler2D s2D;                          // Floating-point 2D texture
isampler2D is2D;                        // Integer 2D texture
usampler2D us2D;                        // Unsigned integer 2D texture

// 3D samplers
sampler3D s3D;                          // Floating-point 3D texture
isampler3D is3D;                        // Integer 3D texture
usampler3D us3D;                        // Unsigned integer 3D texture

// Cube map samplers
samplerCube sCube;                      // Floating-point cube map
isamplerCube isCube;                    // Integer cube map
usamplerCube usCube;                    // Unsigned integer cube map

// Array texture samplers (GLSL 1.30+)
sampler1DArray s1DArray;                // Floating-point 1D array
isampler1DArray is1DArray;              // Integer 1D array
usampler1DArray us1DArray;              // Unsigned integer 1D array

sampler2DArray s2DArray;                // Floating-point 2D array
isampler2DArray is2DArray;              // Integer 2D array
usampler2DArray us2DArray;              // Unsigned integer 2D array

// Shadow samplers
sampler1DShadow s1DShadow;              // 1D depth texture with comparison
sampler2DShadow s2DShadow;              // 2D depth texture with comparison
samplerCubeShadow sCubeShadow;          // Cube map depth texture with comparison

// Multisample texture samplers (GLSL 1.50+)
sampler2DMS s2DMS;                      // 2D multisample texture
isampler2DMS is2DMS;                    // Integer 2D multisample texture
usampler2DMS us2DMS;                    // Unsigned integer 2D multisample texture

// Buffer texture samplers (GLSL 1.40+)
samplerBuffer sBuffer;                  // Floating-point buffer texture
isamplerBuffer isBuffer;                // Integer buffer texture
usamplerBuffer usBuffer;                // Unsigned integer buffer texture
```

### Image Types (GLSL 4.20+)

Image types allow read/write operations on textures:

```glsl
// 1D images
image1D i1D;                            // Floating-point 1D image
iimage1D ii1D;                          // Integer 1D image
uimage1D ui1D;                          // Unsigned integer 1D image

// 2D images
image2D i2D;                            // Floating-point 2D image
iimage2D ii2D;                          // Integer 2D image
uimage2D ui2D;                          // Unsigned integer 2D image

// 3D images
image3D i3D;                            // Floating-point 3D image
iimage3D ii3D;                          // Integer 3D image
uimage3D ui3D;                          // Unsigned integer 3D image

// Cube map images
imageCube iCube;                        // Floating-point cube map image
iimageCube iiCube;                      // Integer cube map image
uimageCube uiCube;                      // Unsigned integer cube map image

// Array images
image1DArray i1DArray;                  // Floating-point 1D array image
iimage1DArray ii1DArray;                // Integer 1D array image
uimage1DArray ui1DArray;                // Unsigned integer 1D array image

image2DArray i2DArray;                  // Floating-point 2D array image
iimage2DArray ii2DArray;                // Integer 2D array image
uimage2DArray ui2DArray;                // Unsigned integer 2D array image

// Multisample images
image2DMS i2DMS;                        // Floating-point 2D multisample image
iimage2DMS ii2DMS;                      // Integer 2D multisample image
uimage2DMS ui2DMS;                      // Unsigned integer 2D multisample image
```

## Variables and Qualifiers

### Storage Qualifiers

Storage qualifiers define where and how variables are stored:

```glsl
// Const qualifier - compile-time constant
const float PI = 3.14159265359;

// Uniform qualifier - read-only, uniform across all shader invocations
uniform mat4 modelViewProjection;
uniform vec3 lightPosition;

// Attribute qualifier (vertex shader only) - per-vertex input
attribute vec3 position;
attribute vec3 normal;
attribute vec2 texCoord;

// Varying qualifier - interpolated output from vertex shader to fragment shader
varying vec3 worldPosition;
varying vec3 normalInterpolated;

// In/out qualifiers (GLSL 1.30+) - explicit input/output
in vec3 vertexPosition;                 // Input to vertex shader
out vec3 fragmentNormal;                // Output from vertex shader

// In qualifier in fragment shader
in vec3 interpolatedNormal;             // Input to fragment shader

// Centroid qualifier - sample at centroid for multisampling
centroid in vec2 centroidTexCoord;

// Patch qualifier (tessellation shaders)
patch in vec3 patchControlPoint;

// Sample qualifier (GLSL 4.00+) - per-sample variables
sample in vec3 sampleNormal;
```

### Precision Qualifiers (OpenGL ES/WebGL)

Precision qualifiers specify the precision of variables:

```glsl
// High precision - highest available precision
highp float preciseValue;

// Medium precision - balanced precision and performance
mediump vec3 mediumVector;

// Low precision - lowest available precision
lowp vec4 lowVector;

// Default precision for different types
precision highp float;
precision mediump int;
precision lowp sampler2D;
```

### Invariant Qualifier

The invariant qualifier ensures consistent results across shader stages:

```glsl
// Make a varying variable invariant
invariant varying vec3 worldPosition;

// Or declare as invariant when defining
invariant varying vec3 normalInterpolated;
```

### Interpolation Qualifiers (GLSL 1.30+)

Interpolation qualifiers control how vertex outputs are interpolated:

```glsl
// Smooth interpolation (default)
smooth out vec3 smoothColor;

// Flat interpolation - no interpolation, uses provoking vertex value
flat out int instanceID;

// No perspective correction - linear interpolation in screen space
noperspective out vec2 screenTexCoord;
```

## Functions

### Function Declaration and Definition

```glsl
// Function with no parameters and no return value
void doSomething() {
    // Function body
}

// Function with parameters and return value
float calculateDistance(vec3 point1, vec3 point2) {
    vec3 delta = point1 - point2;
    return length(delta);
}

// Function with inout parameter (pass by reference)
void modifyValue(inout vec3 value) {
    value = value * 2.0;
}

// Function with out parameter
void getMinMax(vec3 input, out float minVal, out float maxVal) {
    minVal = min(input.x, min(input.y, input.z));
    maxVal = max(input.x, max(input.y, input.z));
}

// Overloaded functions
vec3 transform(vec3 position, mat4 matrix) {
    return (matrix * vec4(position, 1.0)).xyz;
}

vec4 transform(vec4 position, mat4 matrix) {
    return matrix * position;
}
```

### Built-in Functions

GLSL provides a rich set of built-in functions:

#### Angle and Trigonometry Functions

```glsl
// Trigonometric functions
float sine = sin(3.14159 / 2.0);
float cosine = cos(3.14159 / 2.0);
float tangent = tan(3.14159 / 4.0);

// Inverse trigonometric functions
float arcsine = asin(0.5);
float arccosine = acos(0.5);
float arctangent = atan(1.0);
float arctan2 = atan(1.0, 1.0);

// Hyperbolic functions
float sinhValue = sinh(1.0);
float coshValue = cosh(1.0);
float tanhValue = tanh(1.0);

// Radians and degrees conversion
float radians = radians(180.0);
float degrees = degrees(3.14159);
```

#### Exponential Functions

```glsl
// Power functions
float power = pow(2.0, 3.0);
float square = sqrt(16.0);
float inverseSqrt = inversesqrt(16.0);

// Exponential functions
float expValue = exp(1.0);
float exp2Value = exp2(3.0);
float logValue = log(2.71828);
float log2Value = log2(8.0);
```

#### Common Functions

```glsl
// Absolute value
float absValue = abs(-5.0);
int absInt = abs(-5);

// Sign function
float signValue = sign(-3.0);

// Floor, ceiling, and rounding
float floorValue = floor(3.7);
float ceilValue = ceil(3.2);
float rounded = round(3.5);
float truncated = trunc(3.7);

// Fractional part
float fractional = fract(3.14);

// Modulo operations
float modulo = mod(7.0, 3.0);
vec3 modVector = mod(vec3(7.0, 8.0, 9.0), 3.0);

// Minimum and maximum
float minValue = min(3.0, 5.0);
float maxValue = max(3.0, 5.0);
vec3 minVector = min(vec3(1.0, 2.0, 3.0), vec3(3.0, 2.0, 1.0));

// Clamping
float clamped = clamp(5.0, 0.0, 10.0);
vec3 clampedVector = clamp(vec3(5.0, 15.0, -5.0), 0.0, 10.0);

// Mixing and linear interpolation
float mixed = mix(0.0, 10.0, 0.3);
vec3 mixedVector = mix(vec3(0.0), vec3(10.0), 0.3);
float stepped = step(0.5, 0.3);  // Returns 0.0 if 0.3 < 0.5, otherwise 1.0
float smooth = smoothstep(0.0, 1.0, 0.3);  // Hermite interpolation
```

#### Geometric Functions

```glsl
// Length of a vector
float vectorLength = length(vec3(1.0, 2.0, 3.0));

// Distance between two points
float distanceValue = distance(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));

// Dot product
float dotProduct = dot(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));

// Cross product
vec3 crossProduct = cross(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));

// Normalize vector
vec3 normalized = normalize(vec3(1.0, 2.0, 3.0));

// Face forwarding
vec3 faceForwardVec = faceforward(N, I, Nref);

// Vector reflection
vec3 reflected = reflect(I, N);

// Vector refraction
vec3 refracted = refract(I, N, eta);
```

#### Matrix Functions

```glsl
// Matrix multiplication
mat4 result = matrix1 * matrix2;

// Matrix transpose
mat4 transposed = transpose(matrix);

// Matrix determinant
float det = determinant(matrix);

// Matrix inverse
mat4 inv = inverse(matrix);

// Outer product
mat3 outer = outerProduct(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0));

// Matrix construction from vectors
mat3 fromVectors = mat3(vec3(1.0, 0.0, 0.0),
                        vec3(0.0, 1.0, 0.0),
                        vec3(0.0, 0.0, 1.0));

// Extract matrix components
vec4 column0 = matrix[0];  // First column
float element = matrix[0][1];  // Element at column 0, row 1
```

#### Vector Relational Functions

```glsl
// Less than
bvec3 less = lessThan(vec3(1.0, 2.0, 3.0), vec3(2.0, 2.0, 2.0));

// Less than or equal
bvec3 lessEqual = lessThanEqual(vec3(1.0, 2.0, 3.0), vec3(2.0, 2.0, 2.0));

// Greater than
bvec3 greater = greaterThan(vec3(1.0, 2.0, 3.0), vec3(2.0, 2.0, 2.0));

// Greater than or equal
bvec3 greaterEqual = greaterThanEqual(vec3(1.0, 2.0, 3.0), vec3(2.0, 2.0, 2.0));

// Equal
bvec3 equal = equal(vec3(1.0, 2.0, 3.0), vec3(1.0, 2.0, 3.0));

// Not equal
bvec3 notEqual = notEqual(vec3(1.0, 2.0, 3.0), vec3(2.0, 2.0, 2.0));

// Logical operations on boolean vectors
bool allTrue = all(equal);
bool anyTrue = any(equal);
bvec3 inverted = not(equal);
```

#### Texture Lookup Functions

```glsl
// Basic texture sampling
vec4 color = texture2D(sampler, texCoord);

// Level of detail sampling
vec4 lodColor = texture2DLod(sampler, texCoord, lodLevel);

// Gradient-based sampling
vec4 gradColor = texture2DGrad(sampler, texCoord, ddx, ddy);

// Projective texture sampling
vec4 projColor = texture2DProj(sampler, projTexCoord);

// Offset sampling
vec4 offsetColor = texture2DOffset(sampler, texCoord, ivec2(1, 1));

// Size queries
ivec2 size = textureSize(sampler, 0);  // 0 = base level

// Shadow sampling
float shadow = shadow2D(shadowSampler, shadowCoord).r;

// Cube map sampling
vec4 cubeColor = textureCube(cubeSampler, direction);
```

#### Fragment Processing Functions

```glsl
// Derivative functions
vec2 dx = dFdx(value);
vec2 dy = dFdy(value);
vec2 gradient = fwidth(value);  // abs(dFdx(value)) + abs(dFdy(value))

// Interpolation control
vec3 interpolated = interpolateAtCentroid(value);
vec3 sampleInterpolated = interpolateAtSample(value, sampleIndex);
vec3 offsetInterpolated = interpolateAtOffset(value, offset);
```

#### Noise Functions

```glsl
// Noise generation (availability depends on implementation)
float noiseValue = noise1(position);
vec2 noiseVector2 = noise2(position);
vec3 noiseVector3 = noise3(position);
vec4 noiseVector4 = noise4(position);
```

## Shader Stages

### Vertex Shaders

Vertex shaders process vertices and output clip-space positions:

```glsl
#version 330 core

// Input attributes
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoord;

// Uniforms
uniform mat4 modelViewProjection;
uniform mat3 normalMatrix;
uniform vec3 lightPosition;

// Output varyings
out vec3 worldPosition;
out vec3 normalInterpolated;
out vec2 texCoordInterpolated;

void main() {
    // Transform vertex position to clip space
    gl_Position = modelViewProjection * vec4(position, 1.0);
    
    // Pass through other attributes
    worldPosition = position;
    normalInterpolated = normalMatrix * normal;
    texCoordInterpolated = texCoord;
}
```

### Fragment Shaders

Fragment shaders process fragments and output colors:

```glsl
#version 330 core

// Input varyings
in vec3 worldPosition;
in vec3 normalInterpolated;
in vec2 texCoordInterpolated;

// Uniforms
uniform sampler2D diffuseTexture;
uniform vec3 lightPosition;
uniform vec3 cameraPosition;

// Output
out vec4 fragColor;

void main() {
    // Sample texture
    vec4 baseColor = texture2D(diffuseTexture, texCoordInterpolated);
    
    // Simple lighting calculation
    vec3 lightDir = normalize(lightPosition - worldPosition);
    vec3 normal = normalize(normalInterpolated);
    float diffuse = max(dot(normal, lightDir), 0.0);
    
    // Apply lighting
    vec3 litColor = baseColor.rgb * diffuse;
    
    // Output final color
    fragColor = vec4(litColor, baseColor.a);
}
```

### Tessellation Control Shaders (GLSL 4.00+)

Tessellation control shaders control tessellation levels:

```glsl
#version 400 core

layout(vertices = 3) out;

in vec3 tcPosition[];
in vec3 tcNormal[];
in vec2 tcTexCoord[];

out vec3 tePosition[];
out vec3 teNormal[];
out vec2 teTexCoord[];

uniform int tessellationLevel;

void main() {
    // Pass through attributes
    tePosition[gl_InvocationID] = tcPosition[gl_InvocationID];
    teNormal[gl_InvocationID] = tcNormal[gl_InvocationID];
    teTexCoord[gl_InvocationID] = tcTexCoord[gl_InvocationID];
    
    // Set tessellation levels (only one invocation should do this)
    if (gl_InvocationID == 0) {
        gl_TessLevelInner[0] = tessellationLevel;
        gl_TessLevelOuter[0] = tessellationLevel;
        gl_TessLevelOuter[1] = tessellationLevel;
        gl_TessLevelOuter[2] = tessellationLevel;
    }
}
```

### Tessellation Evaluation Shaders (GLSL 4.00+)

Tessellation evaluation shaders evaluate new vertices:

```glsl
#version 400 core

layout(triangles, equal_spacing, ccw) in;

in vec3 tePosition[];
in vec3 teNormal[];
in vec2 teTexCoord[];

out vec3 worldPosition;
out vec3 normalInterpolated;
out vec2 texCoordInterpolated;

uniform mat4 modelViewProjection;

void main() {
    // Barycentric interpolation
    vec3 position = gl_TessCoord.x * tePosition[0] +
                    gl_TessCoord.y * tePosition[1] +
                    gl_TessCoord.z * tePosition[2];
    
    vec3 normal = gl_TessCoord.x * teNormal[0] +
                  gl_TessCoord.y * teNormal[1] +
                  gl_TessCoord.z * teNormal[2];
    
    vec2 texCoord = gl_TessCoord.x * teTexCoord[0] +
                    gl_TessCoord.y * teTexCoord[1] +
                    gl_TessCoord.z * teTexCoord[2];
    
    // Output transformed position
    gl_Position = modelViewProjection * vec4(position, 1.0);
    
    // Pass through attributes
    worldPosition = position;
    normalInterpolated = normal;
    texCoordInterpolated = texCoord;
}
```

## Built-in Variables

### Vertex Shader Built-ins

```glsl
// Input built-ins
int gl_VertexID;        // Index of the current vertex
int gl_InstanceID;      // Index of the current instance

// Output built-ins
vec4 gl_Position;       // Clip-space output position
float gl_PointSize;     // Size of point primitives
```

### Fragment Shader Built-ins

```glsl
// Input built-ins
vec4 gl_FragCoord;      // Window-relative fragment coordinates
bool gl_FrontFacing;    // True if fragment is from front-facing primitive
vec2 gl_PointCoord;     // Point sprite coordinates

// Output built-ins
vec4 gl_FragColor;      // Fragment color (GLSL 1.10, deprecated)
vec4 gl_FragData[gl_MaxDrawBuffers];  // Multiple render target outputs

// OpenGL 3.0+ output syntax
out vec4 fragColor;     // Primary fragment color output
```

### Geometry Shader Built-ins (GLSL 1.50+)

```glsl
// Input built-ins
in gl_PerVertex {
    vec4 gl_Position;
    float gl_PointSize;
    float gl_ClipDistance[];
} gl_in[];

// Output built-ins
out gl_PerVertex {
    vec4 gl_Position;
    float gl_PointSize;
    float gl_ClipDistance[];
};

// Other built-ins
int gl_PrimitiveIDIn;   // ID of input primitive
int gl_PrimitiveID;     // ID of output primitive
int gl_InvocationID;    // Invocation ID
```

### Tessellation Control Shader Built-ins (GLSL 4.00+)

```glsl
// Input built-ins
in gl_PerVertex {
    vec4 gl_Position;
    float gl_PointSize;
    float gl_ClipDistance[];
} gl_in[gl_MaxPatchVertices];

// Output built-ins
out gl_PerVertex {
    vec4 gl_Position;
    float gl_PointSize;
    float gl_ClipDistance[];
} gl_out[];

// Other built-ins
int gl_PatchVerticesIn;     // Number of vertices in input patch
int gl_PrimitiveID;         // ID of current patch
int gl_InvocationID;        // Invocation ID
```

### Tessellation Evaluation Shader Built-ins (GLSL 4.00+)

```glsl
// Input built-ins
in gl_PerVertex {
    vec4 gl_Position;
    float gl_PointSize;
    float gl_ClipDistance[];
} gl_in[gl_MaxPatchVertices];

// Output built-ins
out gl_PerVertex {
    vec4 gl_Position;
    float gl_PointSize;
    float gl_ClipDistance[];
};

// Other built-ins
int gl_PatchVerticesIn;     // Number of vertices in input patch
int gl_PrimitiveID;         // ID of current patch
vec3 gl_TessCoord;          // Tessellation coordinate
```

### Compute Shader Built-ins (GLSL 4.30+)

```glsl
// Input built-ins
in uvec3 gl_NumWorkGroups;      // Number of work groups
in uvec3 gl_WorkGroupID;        // Work group ID
in uvec3 gl_LocalInvocationID;  // Local invocation ID
in uvec3 gl_GlobalInvocationID; // Global invocation ID
in uint gl_LocalInvocationIndex; // Local invocation index
in uint gl_WorkGroupSize;       // Work group size

// Output built-ins
out uint gl_DeviceIndex;        // Device index (GLSL 4.60+)
```

## Textures and Samplers

### Texture Sampling

```glsl
// Basic sampling
vec4 color = texture(sampler2D, texCoord);

// Level of detail sampling
vec4 lodColor = textureLod(sampler2D, texCoord, lodLevel);

// Gradient-based sampling
vec4 gradColor = textureGrad(sampler2D, texCoord, ddx, ddy);

// Projective sampling
vec4 projColor = textureProj(sampler2D, projTexCoord);

// Offset sampling
vec4 offsetColor = textureOffset(sampler2D, texCoord, ivec2(1, 1));

// Size queries
ivec2 size = textureSize(sampler2D, 0);  // 0 = base level

// Shadow sampling
float shadow = texture(sampler2DShadow, shadowCoord);

// Cube map sampling
vec4 cubeColor = texture(samplerCube, direction);
```

### Image Load/Store Operations (GLSL 4.20+)

```glsl
// Image load
vec4 pixel = imageLoad(image2D, ivec2(x, y));

// Image store
imageStore(image2D, ivec2(x, y), vec4(1.0, 0.0, 0.0, 1.0));

// Atomic operations
uint oldValue = imageAtomicAdd(image2D, ivec2(x, y), 1u);
uint exchanged = imageAtomicExchange(image2D, ivec2(x, y), newValue);
```

## Uniforms and Attributes

### Uniform Blocks (GLSL 1.40+)

Uniform blocks group related uniform variables:

```glsl
// Uniform block definition
layout(std140) uniform CameraBlock {
    mat4 viewProjection;
    vec3 cameraPosition;
    float time;
} camera;

// Usage
vec4 clipPosition = camera.viewProjection * vec4(worldPosition, 1.0);
```

### Buffer Blocks (GLSL 4.30+)

Buffer blocks provide shader storage buffer objects:

```glsl
// Shader storage buffer
layout(std430, binding = 0) buffer ParticleBuffer {
    uint particleCount;
    Particle particles[];
};

// Usage
Particle p = particles[gl_GlobalInvocationID.x];
```

## Interface Blocks

Interface blocks group related input/output variables:

```glsl
// Vertex shader output block
out VertexData {
    vec3 worldPosition;
    vec3 normal;
    vec2 texCoord;
} vertexOut;

// Fragment shader input block
in VertexData {
    vec3 worldPosition;
    vec3 normal;
    vec2 texCoord;
} vertexIn;

// Geometry shader input block
in VertexData {
    vec3 worldPosition;
    vec3 normal;
    vec2 texCoord;
} gsIn[];
```

## Geometry Shaders

Geometry shaders process entire primitives:

```glsl
#version 330 core

layout(triangles) in;
layout(triangle_strip, max_vertices = 3) out;

in VertexData {
    vec3 worldPosition;
    vec3 normal;
    vec2 texCoord;
} gsIn[];

out vec3 gsWorldPosition;
out vec3 gsNormal;
out vec2 gsTexCoord;

uniform mat4 viewProjection;

void main() {
    // Emit each vertex of the triangle
    for(int i = 0; i < 3; i++) {
        gsWorldPosition = gsIn[i].worldPosition;
        gsNormal = gsIn[i].normal;
        gsTexCoord = gsIn[i].texCoord;
        
        gl_Position = viewProjection * vec4(gsWorldPosition, 1.0);
        EmitVertex();
    }
    
    EndPrimitive();
}
```

## Compute Shaders

Compute shaders perform general-purpose computations:

```glsl
#version 430 core

layout(local_size_x = 16, local_size_y = 16) in;

layout(binding = 0, rgba8) uniform writeonly image2D outputImage;
layout(binding = 1) uniform sampler2D inputImage;

uniform float time;

void main() {
    // Get thread indices
    ivec2 texelCoord = ivec2(gl_GlobalInvocationID.xy);
    
    // Read from input texture
    vec4 inputValue = texelFetch(inputImage, texelCoord, 0);
    
    // Process the value
    vec4 outputValue = processValue(inputValue, time);
    
    // Write to output image
    imageStore(outputImage, texelCoord, outputValue);
}

vec4 processValue(vec4 input, float t) {
    // Simple processing example
    return input * (0.5 + 0.5 * sin(t));
}
```

## Best Practices

### Performance Optimization

1. **Minimize Dynamic Branching**
```glsl
// Inefficient: Dynamic branching
if (condition) {
    result = expensiveCalculationA();
} else {
    result = expensiveCalculationB();
}

// Better: Use mix/select functions
result = mix(expensiveCalculationA(), expensiveCalculationB(), float(condition));
```

2. **Use Appropriate Precision**
```glsl
// High precision for positions
highp vec3 worldPosition;

// Medium precision for normals
mediump vec3 normal;

// Low precision for colors
lowp vec4 color;
```

3. **Avoid Expensive Functions in Loops**
```glsl
// Inefficient: Recalculating in loop
for (int i = 0; i < count; i++) {
    float value = sin(time + float(i) * 0.1);
    // ...
}

// Better: Pre-calculate outside loop
float timeBase = sin(time);
for (int i = 0; i < count; i++) {
    float value = timeBase + float(i) * 0.1;
    // ...
}
```

### Code Organization

1. **Modular Functions**
```glsl
// Break complex operations into smaller functions
float calculateLighting(vec3 normal, vec3 lightDir) {
    return max(dot(normalize(normal), normalize(lightDir)), 0.0);
}

vec3 applyFog(vec3 color, float distance) {
    float fogFactor = 1.0 - exp(-fogDensity * distance);
    return mix(color, fogColor, fogFactor);
}
```

2. **Consistent Naming**
```glsl
// Use descriptive names
struct DirectionalLight {
    vec3 direction;
    vec3 color;
    float intensity;
};

// Prefix uniforms consistently
uniform mat4 u_modelViewProjection;
uniform vec3 u_cameraPosition;
uniform float u_time;
```

## Common Patterns

### Noise Generation

```glsl
// Simple 2D noise function
float hash(float n) {
    return fract(sin(n) * 43758.5453);
}

float noise(vec2 x) {
    vec2 p = floor(x);
    vec2 f = fract(x);
    
    f = f * f * (3.0 - 2.0 * f);
    
    float n = p.x + p.y * 57.0;
    
    return mix(mix(hash(n + 0.0), hash(n + 1.0), f.x),
               mix(hash(n + 57.0), hash(n + 58.0), f.x), f.y);
}
```

### Color Manipulation

```glsl
// HSV to RGB conversion
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// Gamma correction
vec3 gammaEncode(vec3 color, float gamma) {
    return pow(color, vec3(1.0 / gamma));
}

vec3 gammaDecode(vec3 color, float gamma) {
    return pow(color, vec3(gamma));
}
```

### Mathematical Utilities

```glsl
// Smooth step function
float smoothStep(float edge0, float edge1, float x) {
    float t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// Linear interpolation
float lerp(float a, float b, float t) {
    return a + t * (b - a);
}

// Remapping function
float remap(float value, float inMin, float inMax, float outMin, float outMax) {
    return outMin + (outMax - outMin) * (value - inMin) / (inMax - inMin);
}
```

---
*End of GLSL Fundamentals*

*Next steps:*
*- HLSL Fundamentals*
*- ISF Deep Dive*
*- Shader Conversion Framework*
*- Application Usage Guide*