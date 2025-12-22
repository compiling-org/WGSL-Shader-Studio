# HLSL Fundamentals - Complete Reference

## Table of Contents

1. [Introduction to HLSL](#introduction-to-hlsl)
2. [Basic Syntax](#basic-syntax)
3. [Data Types](#data-types)
4. [Variables and Semantics](#variables-and-semantics)
5. [Functions](#functions)
6. [Shader Models](#shader-models)
7. [Shader Stages](#shader-stages)
8. [Built-in Variables](#built-in-variables)
9. [Textures and Samplers](#textures-and-samplers)
10. [Constant Buffers](#constant-buffers)
11. [Structured Buffers](#structured-buffers)
12. [Append/Consume Buffers](#appendconsume-buffers)
13. [Byte Address Buffers](#byte-address-buffers)
14. [Geometry Shaders](#geometry-shaders)
15. [Hull and Domain Shaders](#hull-and-domain-shaders)
16. [Compute Shaders](#compute-shaders)
17. [Best Practices](#best-practices)
18. [Common Patterns](#common-patterns)

## Introduction to HLSL

High Level Shading Language (HLSL) is Microsoft's shading language for DirectX. It's used to write shaders for Direct3D applications and is similar to C++ in syntax and structure.

HLSL has evolved through multiple shader models, each adding new features:

- **Shader Model 1.x**: Basic vertex and pixel shaders
- **Shader Model 2.0**: Improved instruction sets and flow control
- **Shader Model 3.0**: Dynamic branching and longer programs
- **Shader Model 4.0**: Geometry shaders, interface inheritance, and more
- **Shader Model 5.0**: Compute shaders, hull/domain shaders, and advanced features
- **Shader Model 5.1**: Enhanced resource binding and indexing
- **Shader Model 6.0**: Wave intrinsics and 16-bit types
- **Shader Model 6.1-6.6**: Additional features including ray tracing

HLSL is compiled to bytecode that runs on the GPU through the Direct3D runtime.

## Basic Syntax

HLSL syntax is similar to C++, with additions for graphics programming:

```hlsl
// Preprocessor directives
#define PI 3.14159265359
#define MAX_LIGHTS 8

// Global variables
cbuffer ConstantBuffer : register(b0)
{
    float4x4 modelViewProjection;
    float3 lightPosition;
    float time;
};

// Texture and sampler
Texture2D diffuseTexture : register(t0);
SamplerState linearSampler : register(s0);

// Function declaration
float3 calculateLighting(float3 normal, float3 lightDir);

// Vertex shader function
float4 VSMain(float3 position : POSITION, 
              float3 normal : NORMAL, 
              float2 texCoord : TEXCOORD0) : SV_POSITION
{
    // Local variables
    float3 worldPos = position;
    float3 lightDir = normalize(lightPosition - worldPos);
    
    // Function call
    float3 litColor = calculateLighting(normal, lightDir);
    
    // Return clip-space position
    return mul(modelViewProjection, float4(position, 1.0));
}

// Pixel shader function
float4 PSMain(float4 position : SV_POSITION, 
              float3 worldPos : WORLDPOS,
              float3 normal : NORMAL, 
              float2 texCoord : TEXCOORD0) : SV_TARGET
{
    // Sample texture
    float4 baseColor = diffuseTexture.Sample(linearSampler, texCoord);
    
    // Output final color
    return baseColor;
}
```

### Comments

```hlsl
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

HLSL supports several scalar types:

```hlsl
// Floating point types
float a = 1.0f;         // 32-bit IEEE 754 float
half b = 2.0h;          // 16-bit half float (SM 4.0+)
min16float c = 3.0;     // Minimum 16-bit float (SM 6.2+)
double d = 4.0;         // 64-bit IEEE 754 double (SM 5.0+)

// Integer types
int e = 5;              // 32-bit signed integer
uint f = 6u;            // 32-bit unsigned integer
bool g = true;          // Boolean value

// 16-bit integer types (SM 6.2+)
min16int h = 7;         // Minimum 16-bit signed integer
min16uint i = 8u;       // Minimum 16-bit unsigned integer
```

### Vector Types

Vector types combine multiple scalar values:

```hlsl
// 2-component vectors
float2 v2f = float2(1.0, 2.0);        // Float vector
int2 v2i = int2(1, 2);                // Integer vector
uint2 v2u = uint2(1u, 2u);            // Unsigned integer vector
bool2 v2b = bool2(true, false);       // Boolean vector

// 3-component vectors
float3 v3f = float3(1.0, 2.0, 3.0);
int3 v3i = int3(1, 2, 3);
uint3 v3u = uint3(1u, 2u, 3u);
bool3 v3b = bool3(true, false, true);

// 4-component vectors
float4 v4f = float4(1.0, 2.0, 3.0, 4.0);
int4 v4i = int4(1, 2, 3, 4);
uint4 v4u = uint4(1u, 2u, 3u, 4u);
bool4 v4b = bool4(true, false, true, false);

// Vector construction
float3 constructed = float3(v2f, 3.0);     // From float2 and scalar
float4 constructed2 = float4(v3f, 4.0);    // From float3 and scalar

// Vector swizzling
float2 xy = v4f.xy;       // First two components
float3 rgb = v4f.rgb;     // First three components as color
float3 bgr = v4f.bgr;     // Components in reverse order
float4 xyzw = v4f.xyzw;   // All components
```

### Matrix Types

Matrix types represent 2D arrays of scalar values:

```hlsl
// Matrix types (row-major by default, column-major with row_major keyword)
float2x2 m22 = float2x2(1.0, 0.0, 0.0, 1.0);  // 2x2 matrix
float3x3 m33 = float3x3(1.0);                  // 3x3 matrix
float4x4 m44 = float4x4(1.0);                  // 4x4 matrix

// Non-square matrices
float2x3 m23 = float2x3(1.0);                 // 2 rows, 3 columns
float3x2 m32 = float3x2(1.0);                 // 3 rows, 2 columns
float2x4 m24 = float2x4(1.0);                 // 2 rows, 4 columns
float4x2 m42 = float4x2(1.0);                 // 4 rows, 2 columns
float3x4 m34 = float3x4(1.0);                 // 3 rows, 4 columns
float4x3 m43 = float4x3(1.0);                 // 4 rows, 3 columns

// Matrix construction
float2x2 constructed = float2x2(1.0, 0.0, 0.0, 1.0);  // Row by row
float3x3 diagonal = float3x3(float3(1.0, 0.0, 0.0),    // Row by row
                             float3(0.0, 1.0, 0.0),
                             float3(0.0, 0.0, 1.0));

// Matrix access
float element = m44._12;               // First row, second column
float2 row = m44._m00_m01;             // First row (elements 0,0 and 0,1)
```

### Texture Types

Texture types are used for texture sampling:

```hlsl
// 1D textures
Texture1D<float4> tex1D;                       // 1D texture returning float4
Texture1D<float> tex1DSingle;                  // 1D texture returning float

// 2D textures
Texture2D<float4> tex2D;                       // 2D texture returning float4
Texture2D<float> tex2DSingle;                  // 2D texture returning float
Texture2DArray<float4> tex2DArray;             // 2D texture array

// 3D textures
Texture3D<float4> tex3D;                       // 3D texture

// Cube maps
TextureCube<float4> texCube;                   // Cube map texture
TextureCubeArray<float4> texCubeArray;         // Cube map array

// Multisampled textures
Texture2DMS<float4> tex2DMS;                   // 2D multisampled texture
Texture2DMSArray<float4> tex2DMSArray;         // 2D multisampled texture array

// Buffer textures
Buffer<float4> texBuffer;                      // Buffer texture
```

### RWTexture Types (Read-Write Textures)

RWTexture types allow read/write operations (SM 5.0+):

```hlsl
// Read-write textures
RWTexture1D<float4> rwTex1D;                   // 1D read-write texture
RWTexture2D<float4> rwTex2D;                   // 2D read-write texture
RWTexture3D<float4> rwTex3D;                   // 3D read-write texture
RWTexture2DArray<float4> rwTex2DArray;         // 2D array read-write texture
```

### Sampler States

Sampler states define how textures are sampled:

```hlsl
// Sampler states
SamplerState linearSampler;                    // Linear filtering
SamplerComparisonState shadowSampler;          // Comparison sampler for shadows

// Sampler state with explicit settings
SamplerState pointSampler
{
    Filter = MIN_MAG_MIP_POINT;
    AddressU = WRAP;
    AddressV = WRAP;
    AddressW = WRAP;
};
```

## Variables and Semantics

### Storage Classes

HLSL variables can have different storage classes:

```hlsl
// Static variables (default)
static float staticVar = 1.0;

// Uniform variables (constant across shader invocation)
uniform float uniformVar = 2.0;

// Groupshared variables (shared within thread group in compute shaders)
groupshared float sharedVar[256];

// Volatile variables (prevent optimization)
volatile float volatileVar = 3.0;

// Precise variables (ensure precise floating-point evaluation)
precise float preciseVar = 4.0;
```

### Semantic Annotations

Semantics define the purpose and usage of variables:

```hlsl
// Vertex shader input semantics
struct VSInput
{
    float3 position : POSITION;           // Vertex position
    float3 normal : NORMAL;               // Vertex normal
    float2 texCoord : TEXCOORD0;          // Texture coordinate
    float4 color : COLOR0;                // Vertex color
    uint instanceID : SV_InstanceID;      // Instance ID
};

// Vertex shader output semantics
struct VSOutput
{
    float4 position : SV_POSITION;        // Clip-space position
    float3 worldPos : WORLDPOS;           // World position
    float3 normal : NORMAL;               // Normal vector
    float2 texCoord : TEXCOORD0;          // Texture coordinate
    float4 color : COLOR0;                // Color
    uint vertexID : SV_VertexID;          // Vertex ID
};

// Pixel shader output semantics
struct PSOutput
{
    float4 color : SV_TARGET0;            // Render target 0
    float4 normal : SV_TARGET1;           // Render target 1
    float depth : SV_DEPTH;               // Depth output
};
```

### Register Assignment

Explicit register assignment for resources:

```hlsl
// Constant buffer assigned to register b0
cbuffer CameraConstants : register(b0)
{
    float4x4 viewProjection;
    float3 cameraPosition;
};

// Texture assigned to register t0
Texture2D diffuseTexture : register(t0);

// Sampler assigned to register s0
SamplerState linearSampler : register(s0);

// Unordered access view assigned to register u0
RWTexture2D<float4> outputTexture : register(u0);
```

## Functions

### Function Declaration and Definition

```hlsl
// Function with no parameters and no return value
void DoSomething()
{
    // Function body
}

// Function with parameters and return value
float CalculateDistance(float3 point1, float3 point2)
{
    float3 delta = point1 - point2;
    return length(delta);
}

// Function with in/out parameters
void ModifyValue(inout float3 value)
{
    value = value * 2.0;
}

// Function with out parameter
void GetMinMax(float3 input, out float minVal, out float maxVal)
{
    minVal = min(input.x, min(input.y, input.z));
    maxVal = max(input.x, max(input.y, input.z));
}

// Template functions (SM 6.0+)
template<typename T>
T MinValue(T a, T b)
{
    return min(a, b);
}
```

### Built-in Functions

HLSL provides a rich set of built-in functions:

#### Mathematical Functions

```hlsl
// Basic math
float absValue = abs(-5.0);
float ceilValue = ceil(3.14);
float floorValue = floor(3.14);
float roundValue = round(3.14);

// Trigonometric functions
float sinValue = sin(3.14159 / 2.0);
float cosValue = cos(3.14159 / 2.0);
float tanValue = tan(3.14159 / 4.0);

// Inverse trigonometric functions
float asinValue = asin(0.5);
float acosValue = acos(0.5);
float atanValue = atan(1.0);
float atan2Value = atan2(1.0, 1.0);

// Exponential functions
float expValue = exp(1.0);
float exp2Value = exp2(3.0);
float logValue = log(2.71828);
float log2Value = log2(8.0);
float powValue = pow(2.0, 3.0);
float sqrtValue = sqrt(16.0);
float rsqrtValue = rsqrt(16.0);  // 1/sqrt(x)
```

#### Geometric Functions

```hlsl
// Vector operations
float vectorLength = length(float3(1.0, 2.0, 3.0));
float distanceValue = distance(float3(0.0, 0.0, 0.0), float3(1.0, 1.0, 1.0));
float dotProduct = dot(float3(1.0, 0.0, 0.0), float3(0.0, 1.0, 0.0));
float3 crossProduct = cross(float3(1.0, 0.0, 0.0), float3(0.0, 1.0, 0.0));
float3 normalized = normalize(float3(1.0, 2.0, 3.0));
float3 reflected = reflect(float3(1.0, 0.0, 0.0), float3(0.0, 1.0, 0.0));
float3 refracted = refract(float3(1.0, 0.0, 0.0), float3(0.0, 1.0, 0.0), 1.5);

// Matrix operations
float4x4 transposed = transpose(matrix);
float det = determinant(matrix);
float4x4 inv = inverse(matrix);
```

#### Texture Functions

```hlsl
// Basic sampling
float4 color = diffuseTexture.Sample(linearSampler, texCoord);

// Level of detail sampling
float4 lodColor = diffuseTexture.SampleLevel(linearSampler, texCoord, lodLevel);

// Gradient-based sampling
float4 gradColor = diffuseTexture.SampleGrad(linearSampler, texCoord, ddX, ddY);

// Comparison sampling (for shadow maps)
float shadow = shadowTexture.SampleCmp(shadowSampler, texCoord, compareValue);

// Load operations (no filtering)
float4 loaded = diffuseTexture.Load(int3(x, y, mipLevel));

// Gather operations (SM 5.0+)
float4 gathered = diffuseTexture.Gather(linearSampler, texCoord);
```

#### Wave Intrinsics (SM 6.0+)

Wave intrinsics operate on all threads in a wavefront:

```hlsl
// Broadcast value from specific lane
float broadcastValue = WaveReadLaneFirst(value);

// Check if all lanes satisfy condition
bool allTrue = WaveActiveAllTrue(condition);

// Prefix sum
uint prefixSum = WavePrefixSum(value);

// Ballot operations
uint ballot = WaveActiveBallot(condition);
```

## Shader Models

### Shader Model Evolution

Each shader model adds new features and capabilities:

#### Shader Model 1.x
- Basic vertex and pixel shaders
- Limited instruction set
- No dynamic branching

#### Shader Model 2.0
- Increased instruction limits
- Improved flow control
- Multiple render targets

#### Shader Model 3.0
- Dynamic branching
- Longer programs
- Vertex texture fetch

#### Shader Model 4.0
- Geometry shaders
- Interface inheritance
- Integer operations
- Bit manipulation

#### Shader Model 5.0
- Compute shaders
- Hull and domain shaders
- Append/consume buffers
- Stream output

#### Shader Model 5.1
- Enhanced resource binding
- Dynamic indexing of resources
- Viewport and RT array indexing

#### Shader Model 6.0
- Wave intrinsics
- 16-bit types
- Conservative rasterization

#### Shader Model 6.1-6.6
- Ray tracing shaders
- Mesh and amplification shaders
- Additional wave operations
- Shader debugging features

## Shader Stages

### Vertex Shaders

Vertex shaders process vertices and output clip-space positions:

```hlsl
// Vertex shader input structure
struct VSInput
{
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 texCoord : TEXCOORD0;
};

// Vertex shader output structure
struct VSOutput
{
    float4 position : SV_POSITION;
    float3 worldPos : WORLDPOS;
    float3 normal : NORMAL;
    float2 texCoord : TEXCOORD0;
};

// Constant buffer
cbuffer TransformBuffer : register(b0)
{
    float4x4 modelViewProjection;
    float4x4 model;
    float3 lightPosition;
};

// Vertex shader main function
VSOutput VSMain(VSInput input)
{
    VSOutput output;
    
    // Transform vertex position to clip space
    output.position = mul(modelViewProjection, float4(input.position, 1.0));
    
    // Transform normal to world space
    output.normal = mul((float3x3)model, input.normal);
    
    // Pass through other attributes
    output.worldPos = mul(model, float4(input.position, 1.0)).xyz;
    output.texCoord = input.texCoord;
    
    return output;
}
```

### Pixel Shaders

Pixel shaders process fragments and output colors:

```hlsl
// Textures and samplers
Texture2D diffuseTexture : register(t0);
SamplerState linearSampler : register(s0);

// Pixel shader main function
float4 PSMain(VSOutput input) : SV_TARGET
{
    // Sample texture
    float4 baseColor = diffuseTexture.Sample(linearSampler, input.texCoord);
    
    // Simple lighting calculation
    float3 lightDir = normalize(lightPosition - input.worldPos);
    float3 normal = normalize(input.normal);
    float diffuse = max(dot(normal, lightDir), 0.0);
    
    // Apply lighting
    float3 litColor = baseColor.rgb * diffuse;
    
    // Output final color
    return float4(litColor, baseColor.a);
}
```

### Geometry Shaders

Geometry shaders process entire primitives and can emit new geometry:

```hlsl
// Geometry shader that converts points to quads
[maxvertexcount(4)]
void GSMain(point VSOutput input[1], inout TriangleStream<VSOutput> outputStream)
{
    float size = 0.1;
    
    // Create quad vertices
    VSOutput output;
    
    // Bottom-left
    output.position = input[0].position + float4(-size, -size, 0, 0);
    output.texCoord = float2(0, 0);
    output.normal = input[0].normal;
    output.worldPos = input[0].worldPos;
    outputStream.Append(output);
    
    // Bottom-right
    output.position = input[0].position + float4(size, -size, 0, 0);
    output.texCoord = float2(1, 0);
    outputStream.Append(output);
    
    // Top-left
    output.position = input[0].position + float4(-size, size, 0, 0);
    output.texCoord = float2(0, 1);
    outputStream.Append(output);
    
    // Top-right
    output.position = input[0].position + float4(size, size, 0, 0);
    output.texCoord = float2(1, 1);
    outputStream.Append(output);
    
    outputStream.RestartStrip();
}
```

## Built-in Variables

### System Value Semantics

System value semantics provide access to built-in GPU state:

```hlsl
// Vertex shader system values
uint vertexID : SV_VertexID;           // Index of current vertex
uint instanceID : SV_InstanceID;       // Index of current instance

// Geometry shader system values
uint primitiveID : SV_PrimitiveID;     // ID of current primitive

// Pixel shader system values
float4 position : SV_POSITION;        // Pixel position
bool isFrontFace : SV_IsFrontFace;    // True if front-facing
uint sampleIndex : SV_SampleIndex;    // Sample index for MSAA

// Compute shader system values
uint3 groupID : SV_GroupID;           // Thread group ID
uint3 groupThreadID : SV_GroupThreadID; // Thread ID within group
uint groupIndex : SV_GroupIndex;      // Flattened thread index within group
uint3 dispatchThreadID : SV_DispatchThreadID; // Global thread ID
```

## Textures and Samplers

### Texture Sampling

```hlsl
// Basic sampling
float4 color = texture.Sample(sampler, texCoord);

// Level of detail sampling
float4 lodColor = texture.SampleLevel(sampler, texCoord, lod);

// Gradient-based sampling
float4 gradColor = texture.SampleGrad(sampler, texCoord, ddX, ddY);

// Comparison sampling (for shadow maps)
float shadow = shadowTexture.SampleCmp(sampler, texCoord, compareValue);

// Offset sampling
float4 offsetColor = texture.Sample(sampler, texCoord, int2(1, 1));

// Load operations (no filtering)
float4 loaded = texture.Load(int3(x, y, mipLevel));
```

### Texture Object Methods

```hlsl
// Get texture dimensions
uint width, height, numberOfLevels;
texture.GetDimensions(width, height, numberOfLevels);

// Get specific mip level dimensions
texture.GetDimensions(mipLevel, width, height);

// Sample with bias
float4 biasedColor = texture.SampleBias(sampler, texCoord, bias);

// Sample with derivative offsets
float4 offsetColor = texture.Sample(sampler, texCoord, int2(1, 1));
```

## Constant Buffers

Constant buffers group related uniform data:

```hlsl
// Constant buffer with explicit register
cbuffer CameraBuffer : register(b0)
{
    float4x4 viewProjection;
    float3 cameraPosition;
    float padding0;  // Padding for alignment
    float4x4 view;
    float4x4 projection;
};

// Constant buffer with packing
cbuffer LightBuffer : register(b1)
{
    float3 lightDirection;
    float lightIntensity;
    float3 lightColor;
    float padding1;
    float4 ambientColor;
};

// Usage in shader
float4 position = mul(viewProjection, float4(worldPosition, 1.0));
```

### Constant Buffer Packing Rules

Constant buffers follow specific packing rules:

```hlsl
// Efficient packing (4-component vectors)
cbuffer EfficientBuffer : register(b0)
{
    float4 data1;      // Components 0-3
    float4 data2;      // Components 4-7
    float4 data3;      // Components 8-11
};

// Less efficient packing (scalar values)
cbuffer InefficientBuffer : register(b1)
{
    float data1;       // Component 0, 3 empty
    float data2;       // Component 1, 2 empty
    float data3;       // Component 2, 1 empty
    float data4;       // Component 3, 0 empty
    float data5;       // Component 4, 3 empty
};
```

## Structured Buffers

Structured buffers store arrays of structured data:

```hlsl
// Struct definition
struct Particle
{
    float3 position;
    float3 velocity;
    float life;
    float4 color;
};

// Structured buffer declaration
StructuredBuffer<Particle> particleBuffer : register(t0);
RWStructuredBuffer<Particle> rwParticleBuffer : register(u0);

// Usage in shader
Particle p = particleBuffer[particleIndex];
rwParticleBuffer[particleIndex].position += rwParticleBuffer[particleIndex].velocity * deltaTime;
```

## Append/Consume Buffers

Append/consume buffers support dynamic addition and removal of elements:

```hlsl
// Append buffer (write-only)
AppendStructuredBuffer<float4> appendBuffer : register(u0);

// Consume buffer (read-only)
ConsumeStructuredBuffer<float4> consumeBuffer : register(u1);

// Usage in compute shader
[numthreads(256, 1, 1)]
void CSMain(uint3 dispatchThreadID : SV_DispatchThreadID)
{
    // Append data
    appendBuffer.Append(float4(dispatchThreadID, 1.0));
    
    // Consume data (returns 0 if buffer is empty)
    float4 consumedData = consumeBuffer.Consume();
}
```

## Byte Address Buffers

Byte address buffers allow raw byte-level access:

```hlsl
// Byte address buffer declaration
ByteAddressBuffer byteBuffer : register(t0);
RWByteAddressBuffer rwByteBuffer : register(u0);

// Usage in shader
uint value = byteBuffer.Load<uint>(offset);
rwByteBuffer.Store<uint>(offset, newValue);

// Load multiple values
uint2 values = byteBuffer.Load2(offset);
uint3 values3 = byteBuffer.Load3(offset);
uint4 values4 = byteBuffer.Load4(offset);
```

## Hull and Domain Shaders

Hull and domain shaders implement tessellation:

### Hull Shader

```hlsl
// Patch constant function
struct PatchConstantData
{
    float edges[3] : SV_TessFactor;
    float inside : SV_InsideTessFactor;
};

PatchConstantData PatchConstantsHS(InputPatch<VSOutput, 3> patch,
                                  uint patchID : SV_PrimitiveID)
{
    PatchConstantData output;
    
    // Set tessellation factors
    output.edges[0] = 4.0;
    output.edges[1] = 4.0;
    output.edges[2] = 4.0;
    output.inside = 4.0;
    
    return output;
}

// Hull shader main function
[domain("tri")]
[partitioning("fractional_odd")]
[outputtopology("triangle_cw")]
[outputcontrolpoints(3)]
[patchconstantfunc("PatchConstantsHS")]
VSOutput HSMain(InputPatch<VSOutput, 3> patch, 
                uint controlPointID : SV_OutputControlPointID,
                uint patchID : SV_PrimitiveID)
{
    return patch[controlPointID];
}
```

### Domain Shader

```hlsl
// Domain shader main function
[domain("tri")]
VSOutput DSMain(PatchConstantData patchConstants,
                float3 barycentrics : SV_DomainLocation,
                const OutputPatch<VSOutput, 3> patch)
{
    VSOutput output;
    
    // Interpolate using barycentric coordinates
    output.position = patch[0].position * barycentrics.x +
                      patch[1].position * barycentrics.y +
                      patch[2].position * barycentrics.z;
    
    output.normal = patch[0].normal * barycentrics.x +
                    patch[1].normal * barycentrics.y +
                    patch[2].normal * barycentrics.z;
    
    output.texCoord = patch[0].texCoord * barycentrics.x +
                      patch[1].texCoord * barycentrics.y +
                      patch[2].texCoord * barycentrics.z;
    
    return output;
}
```

## Compute Shaders

Compute shaders perform general-purpose computations:

```hlsl
// Compute shader that processes a 2D image
RWTexture2D<float4> outputTexture : register(u0);
Texture2D<float4> inputTexture : register(t0);
SamplerState linearSampler : register(s0);

cbuffer ComputeConstants : register(b0)
{
    float time;
    float2 resolution;
};

[numthreads(16, 16, 1)]
void CSMain(uint3 dispatchThreadID : SV_DispatchThreadID)
{
    // Get thread indices
    uint2 texelCoord = dispatchThreadID.xy;
    
    // Check bounds
    if (texelCoord.x >= resolution.x || texelCoord.y >= resolution.y)
        return;
    
    // Calculate UV coordinates
    float2 uv = float2(texelCoord) / resolution;
    
    // Sample input texture
    float4 inputValue = inputTexture.SampleLevel(linearSampler, uv, 0);
    
    // Process the value
    float4 outputValue = ProcessValue(inputValue, time, uv);
    
    // Write to output texture
    outputTexture[texelCoord] = outputValue;
}

float4 ProcessValue(float4 input, float t, float2 uv)
{
    // Simple processing example
    return input * (0.5 + 0.5 * sin(t + uv.x * 10.0));
}
```

## Best Practices

### Performance Optimization

1. **Minimize Dynamic Branching**
```hlsl
// Inefficient: Dynamic branching
if (condition)
{
    result = expensiveCalculationA();
}
else
{
    result = expensiveCalculationB();
}

// Better: Use lerp/mix functions
result = lerp(expensiveCalculationA(), expensiveCalculationB(), condition ? 1.0 : 0.0);
```

2. **Use Appropriate Precision**
```hlsl
// High precision for positions
float3 worldPosition;

// Half precision for normals (SM 4.0+)
half3 normal;

// Minimum precision for colors (SM 6.2+)
min16float4 color;
```

3. **Optimize Memory Access**
```hlsl
// Coalesced memory access (preferred)
groupshared float sharedData[256];

[numthreads(256, 1, 1)]
void OptimizedCS(uint3 groupThreadID : SV_GroupThreadID)
{
    // Threads access consecutive memory locations
    sharedData[groupThreadID.x] = inputData[groupThreadID.x];
}

// Strided memory access (less efficient)
[numthreads(32, 1, 1)]
void InefficientCS(uint3 groupThreadID : SV_GroupThreadID)
{
    // Threads access memory locations far apart
    sharedData[groupThreadID.x * 8] = inputData[groupThreadID.x * 8];
}
```

### Code Organization

1. **Modular Functions**
```hlsl
// Break complex operations into smaller functions
float CalculateLighting(float3 normal, float3 lightDir)
{
    return max(dot(normalize(normal), normalize(lightDir)), 0.0);
}

float3 ApplyFog(float3 color, float distance)
{
    float fogFactor = 1.0 - exp(-fogDensity * distance);
    return lerp(color, fogColor, fogFactor);
}
```

2. **Consistent Naming**
```hlsl
// Use descriptive names
struct DirectionalLight
{
    float3 direction;
    float3 color;
    float intensity;
};

// Prefix constants consistently
cbuffer LightingConstants
{
    float3 u_lightDirection;
    float3 u_lightColor;
    float u_lightIntensity;
};
```

## Common Patterns

### Noise Generation

```hlsl
// Simple 2D noise function
float Hash(float2 p)
{
    return frac(sin(dot(p, float2(12.9898, 78.233))) * 43758.5453);
}

float Noise(float2 p)
{
    float2 i = floor(p);
    float2 f = frac(p);
    
    f = f * f * (3.0 - 2.0 * f);
    
    float n = dot(i, float2(1.0, 57.0));
    
    return lerp(lerp(Hash(n + 0.0), Hash(n + 1.0), f.x),
                lerp(Hash(n + 57.0), Hash(n + 58.0), f.x), f.y);
}
```

### Color Manipulation

```hlsl
// HSV to RGB conversion
float3 HSVToRGB(float3 c)
{
    float4 K = float4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    float3 p = abs(frac(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * lerp(K.xxx, saturate(p - K.xxx), c.y);
}

// Gamma correction
float3 GammaEncode(float3 color, float gamma)
{
    return pow(color, 1.0 / gamma);
}

float3 GammaDecode(float3 color, float gamma)
{
    return pow(color, gamma);
}
```

### Mathematical Utilities

```hlsl
// Smooth step function
float SmoothStep(float edge0, float edge1, float x)
{
    float t = saturate((x - edge0) / (edge1 - edge0));
    return t * t * (3.0 - 2.0 * t);
}

// Linear interpolation
float Lerp(float a, float b, float t)
{
    return a + t * (b - a);
}

// Remapping function
float Remap(float value, float inMin, float inMax, float outMin, float outMax)
{
    return outMin + (outMax - outMin) * (value - inMin) / (inMax - inMin);
}
```

---
*End of HLSL Fundamentals*

*Next steps:*
*- ISF Deep Dive*
*- Shader Conversion Framework*
*- Application Usage Guide*