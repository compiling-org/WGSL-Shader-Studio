# Node-Based Shader Programming Guide

## Overview
The WGSL Shader Studio includes a powerful node-based visual programming system for creating shaders without writing code. This guide covers all aspects of the node editor.

## Getting Started

### Accessing the Node Editor
1. **Enable Node Editor**: View → Show Node Editor
2. **Create New Graph**: Shader → New → Node Graph
3. **Switch Modes**: Toggle between Code and Node views

### Basic Interface
- **Node Palette**: Left sidebar with node types
- **Canvas**: Central area for node connections
- **Properties Panel**: Right sidebar for node settings
- **Toolbar**: Top controls for graph operations

## Node Types

### Input Nodes
Generate or input data into the shader graph.

#### Time Node
- **Purpose**: Provides current time for animations
- **Properties**:
  - Speed multiplier (0.1-10.0)
  - Time offset
- **Output**: Float value representing time

#### Resolution Node
- **Purpose**: Provides render resolution
- **Properties**:
  - Width (read-only)
  - Height (read-only)
  - Aspect ratio
- **Output**: Vec2 containing resolution

#### Position Node
- **Purpose**: Provides pixel coordinates
- **Properties**:
  - Coordinate space (Screen, Normalized, UV)
  - Flip Y axis
- **Output**: Vec2 containing position

#### Texture Node
- **Purpose**: Load and sample textures
- **Properties**:
  - Texture file path
  - Wrap mode (Repeat, Clamp, Mirror)
  - Filter mode (Linear, Nearest)
  - Mipmap enabled
- **Output**: Texture sampler for color data

#### Audio Node
- **Purpose**: Audio-reactive parameter generation
- **Properties**:
  - Audio source (Volume, Bass, Mid, Treble)
  - Frequency band selection
  - Sensitivity (0.1-10.0)
  - Smoothing factor
- **Output**: Float value from audio analysis

#### MIDI Node
- **Purpose**: Hardware controller input
- **Properties**:
  - Controller number (0-127)
  - Channel (0-15)
  - Min/Max range
  - Curve type (Linear, Exponential)
- **Output**: Float value from MIDI controller

### Math Nodes
Perform mathematical operations on values.

#### Arithmetic Nodes
- **Add**: Element-wise addition
- **Subtract**: Element-wise subtraction  
- **Multiply**: Element-wise multiplication
- **Divide**: Element-wise division
- **Properties**: Two inputs, operation type

#### Math Functions
- **Sin**: Sine wave generation
- **Cos**: Cosine wave generation
- **Tan**: Tangent calculation
- **Exp**: Exponential function
- **Log**: Natural logarithm
- **Sqrt**: Square root
- **Pow**: Power function
- **Properties**: Input value, frequency, phase

#### Comparison Nodes
- **Greater Than**: Compare values
- **Less Than**: Compare values
- **Equal**: Equality test
- **Max**: Maximum of inputs
- **Min**: Minimum of inputs
- **Clamp**: Constrain value to range

#### Vector Operations
- **Dot Product**: Vector dot product
- **Cross Product**: Vector cross product
- **Length**: Vector magnitude
- **Normalize**: Unit vector
- **Distance**: Distance between points
- **Mix**: Linear interpolation

### Color Nodes
Handle color operations and generation.

#### Color Generation
- **RGB**: Red, Green, Blue components
- **HSV**: Hue, Saturation, Value
- **HSL**: Hue, Saturation, Lightness
- **Gradient**: Multi-color gradient
- **Properties**: Color picker, interpolation

#### Color Operations
- **Blend**: Color blending modes
  - Normal, Multiply, Screen
  - Overlay, Soft Light, Hard Light
  - Color Dodge, Color Burn
- **Adjust**: Color correction
  - Brightness, Contrast, Saturation
  - Hue shift, Gamma correction
- **Filter**: Color effects
  - Invert, Grayscale, Sepia
  - Threshold, Posterize

#### Color Space
- **Convert**: Color space conversion
  - RGB ↔ HSV ↔ HSL
  - Linear ↔ sRGB
- **Properties**: Input/output formats

### Texture Nodes
Work with textures and patterns.

#### Pattern Generation
- **Noise**: Perlin/Simplex noise
- **Fractal**: Fractal noise patterns
- **Cellular**: Voronoi diagrams
- **Wave**: Sine/cosine waves
- **Grid**: Grid patterns
- **Properties**: Scale, detail, seed

#### Texture Filters
- **Blur**: Gaussian blur
- **Sharpen**: Unsharp mask
- **Edge**: Edge detection
- **Emboss**: Emboss effect
- **Properties**: Radius, strength

#### UV Manipulation
- **Transform**: Translate, rotate, scale
- **Tiling**: Repeat textures
- **Polar**: Convert to polar coordinates
- **Properties**: Transformation matrix

### Control Nodes
Control flow and logic operations.

#### Conditional
- **If**: Conditional branching
- **Switch**: Multi-way branching
- **Step**: Step function
- **Smoothstep**: Smooth interpolation

#### Timing
- **Delay**: Time delay
- **Trigger**: One-shot trigger
- **Oscillator**: Periodic waveforms
- **Properties**: Frequency, duty cycle

#### Logic
- **AND**: Logical AND
- **OR**: Logical OR
- **NOT**: Logical NOT
- **XOR**: Exclusive OR

## Node Connections

### Connection Types
- **Float**: Single float value (0.0-1.0)
- **Vec2**: Two-component vector (x, y)
- **Vec3**: Three-component vector (r, g, b)
- **Vec4**: Four-component vector (r, g, b, a)
- **Texture**: Texture sampler
- **Sampler**: Texture sampling configuration

### Connecting Nodes
1. **Start Connection**: Drag from output pin
2. **End Connection**: Drop on input pin
3. **Auto-Connect**: Compatible types connect automatically
4. **Disconnect**: Click connection and press Delete

### Connection Rules
- **Single Input**: Each input pin accepts one connection
- **Multiple Outputs**: Output pins can connect to multiple inputs
- **Type Matching**: Compatible data types only
- **No Cycles**: No circular dependencies allowed

## Advanced Features

### Sub-graphs
Create reusable node groups:
1. **Group Selection**: Select nodes to group
2. **Create Sub-graph**: Shader → Group Nodes
3. **Expose Inputs**: Select which pins are external
4. **Use Sub-graph**: Drag from node palette

### Custom Nodes
Define your own node types:
1. **Code Interface**: Provide WGSL shader code
2. **Parameter Interface**: Define input/output pins
3. **Compilation**: Node compiles to WGSL
4. **Library**: Save to node library

### Animation Keyframes
Keyframe support for time-based animation:
1. **Add Keyframe**: Right-click parameter
2. **Edit Curves**: Bezier curve editor
3. **Playback**: Scrub through timeline
4. **Export**: Bake to WGSL uniforms

### GPU Performance
Optimized for real-time performance:
- **Parallel Execution**: Nodes execute in parallel
- **Memory Efficiency**: Minimal GPU memory usage
- **LOD System**: Level-of-detail for complex graphs
- **Profiling**: Node execution timing

## Example Graphs

### Basic Color Gradient
```
Time Node → Sin Node → Color Node
  ↓
Add Node → Output Node
  ↑
Position Node → Normalize Node
```

### Audio-Visualizer
```
Audio Node → Amplitude Node → Color Node → Output
                    ↓
Frequency Node → Blend Node → Color Node
  ↓
Time Node → Oscillator Node
```

### Fractal Pattern
```
Position Node → Fractal Node → Color Map → Output
  ↓
Time Node → Transform Node
  ↓
Noise Node → Scale Node
```

### Texture Blend
```
Texture Node A ──┐
                 ├── Blend Node → Output
Texture Node B ──┘
  ↓
Transform Node
```

## Best Practices

### Performance
1. **Minimize Nodes**: Use efficient node combinations
2. **Texture Caching**: Reuse texture nodes
3. **LOD Control**: Reduce detail for distant objects
4. **Batch Operations**: Combine similar operations

### Organization
1. **Group Related Nodes**: Use sub-graphs
2. **Clear Labels**: Name nodes descriptively
3. **Color Coding**: Use consistent colors
4. **Documentation**: Add comments for complex graphs

### Debugging
1. **Test Individually**: Test nodes in isolation
2. **Intermediate Outputs**: Break graphs to debug
3. **Performance Profiler**: Check execution time
4. **Memory Monitor**: Track GPU memory usage

### Code Generation
1. **Optimization**: Generated code is optimized
2. **Validation**: Graphs validated before compilation
3. **Export**: Can export generated WGSL code
4. **Debug Info**: Source maps for debugging

## Integration with Audio/MIDI

### Audio-Reactive Graphs
```mermaid
Audio Spectrum → Frequency Band → Color Mapping → Output
      ↓
Time Oscillator → Phase Modulation
```

### MIDI-Controlled Graphs
```mermaid
MIDI CC1 → Parameter A → Blend Amount
      ↓
MIDI CC2 → Parameter B → Transform Scale
      ↓
Mix Nodes → Final Output
```

## Workflow Integration

### From Code to Nodes
1. **Import WGSL**: Parse existing shaders
2. **Extract Functions**: Convert to node sub-graphs
3. **Parameter Mapping**: Create input nodes for uniforms
4. **Visual Editor**: Edit via node interface

### From Nodes to Code
1. **Generate WGSL**: Compile graph to shader code
2. **Optimize**: Apply shader optimizations
3. **Validate**: Check for errors
4. **Export**: Save as standalone shader

## Troubleshooting

### Common Issues
- **Slow Performance**: Reduce node complexity
- **Memory Errors**: Check texture usage
- **Connection Errors**: Verify type compatibility
- **Compilation Errors**: Check for cycles

### Debug Tools
- **Node Inspector**: View node properties
- **Execution Order**: See evaluation order
- **Performance Metrics**: GPU timing
- **Memory Usage**: VRAM monitoring

---

**WGSL Shader Studio** - Professional node-based shader programming environment with real-time audio/MIDI integration.