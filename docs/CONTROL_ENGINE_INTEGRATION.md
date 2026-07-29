# Control Engine Integration Guide

## Prime + Complex Neural Orchestration Layer for WGSL Shader Studio

### Overview

The Control Engine is a higher-level orchestration layer that sits **above** SuperInstance and feeds control signals **into** WGSL shaders via uniforms/buffers. It implements the architecture described in the Prime + Complex Neural integration strategy:

- **Complex NN role** (signal + phase): Map audio/MIDI/OSC/sensor streams into latent control states
- **Prime role** (structure + scheduling): Use primes as discrete structure for time signatures, channel indexing, sampling windows

All logic runs in **Rust host** — no NN or prime computation runs per-fragment in WGSL.

### Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Control Engine                      │
│  ┌──────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │  primes   │  │ complex_math │  │  nn_bridge    │  │
│  │ (struct)  │  │  (signal→    │  │  (optional)   │  │
│  │ schedule  │  │   control)   │  │  (ONNX/ort)   │  │
│  └─────┬─────┘  └──────┬───────┘  └──────┬────────┘  │
│        └───────────────┬┴─────────────────┘           │
│                        ▼                              │
│              ┌─────────────────┐                      │
│              │  ControlState   │                      │
│              │  (frame output) │                      │
│              └────────┬────────┘                      │
│                       │                               │
│              ┌────────▼────────┐                      │
│              │ superinstance_  │                      │
│              │ bridge          │                      │
│              │ (→ Flux, Plato, │                      │
│              │  BudgetTracker)  │                      │
│              └─────────────────┘                      │
└───────────────────────────────────────────────────────┘
```

### Module Structure

```
src/control_engine/
├── mod.rs                  # ControlEngine, ControlState, ControlEngineConfig
├── primes.rs               # PrimeSchedule, PrimeMask, PrimeRotator, utilities
├── complex_math.rs         # OscillatorBank, ComplexSmoother, signal→control
├── nn_bridge.rs            # NNBridge, SignalFeatures, ONNX inference bridge
└── superinstance_bridge.rs # SuperInstanceController — bridges to Flux/Plato/Conservation
```

### Key Components

#### ControlState (per-frame output)

```rust
pub struct ControlState {
    pub frame: u64,                    // Monotonically increasing frame counter
    pub prime_phase: f32,              // Prime-scheduled phase (0.0 - 1.0)
    pub active_groups: Vec<String>,    // Which parameter groups update this frame
    pub complex_latent: Vec<f32>,      // Complex-valued signal processing output
    pub instance_mod: Vec<f32>,        // Per-instance modulation values
    pub global_uniforms: Vec<f32>,     // Directly mappable to WGSL uniform blocks
    pub node_graph_params: HashMap<String, f32>,  // Named → value for node graph
}
```

#### Prime Scheduling

| Group | Prime | Update Rate | Role |
|-------|-------|-------------|------|
| group_a | 2 | Every 2 frames | Fastest modulation (color, noise) |
| group_b | 3 | Every 3 frames | Medium-fast (distortion, warp) |
| group_c | 5 | Every 5 frames | Medium (camera, geometry) |
| group_d | 7 | Every 7 frames | Medium-slow (scene transitions) |
| group_e | 11 | Every 11 frames | Slowest (global effects) |

#### Complex Math

- OscillatorBank with harmonic ratios (1, 2, 3, 5, 7, 11, 13, 17)
- ComplexSmoother for low-pass filtering control signals
- Frequency band → control vector mapping
- Complex → color conversion (phase → hue, magnitude → saturation)

#### Neural Network Bridge

- Optional ONNX inference via `ort` crate (behind feature flag)
- Stub implementation using deterministic matrix multiply
- Configurable inference interval (1 = every frame, 60 = every 60 frames)

### CLI Usage

```bash
# Test the control engine
cargo run -- --cli --test-control-engine

# Run with control engine enabled
cargo run -- --cli --control-engine shaders/my_shader.wgsl

# Custom prime schedule
cargo run -- --cli --control-engine --prime-schedule 2,3,5,7,11

# With SuperInstance flags
cargo run -- --cli --control-engine --flux-compile --budget-track --budget-daily 0.10
```

### SuperInstance Bridge

The Control Engine integrates with SuperInstance via `SuperInstanceController`:

- **BudgetTracker**: Prime groups determine gamma/eta routing priorities
- **FluxCompiler**: Prime phase enables/disables flux compilation
- **PlatoRoom**: Control state updates sensors and triggers budget alarms
- **Instance modulation**: Prime masks × complex latent → per-instance modulations

### Dependencies

```toml
num-complex = "0.4"    # Complex-valued control operations
primal = "0.3"         # Prime generation (optional, we have custom sieve)

# Optional: ONNX neural network inference
# ort = { version = "2.0", optional = true }
```

### Integration Points

1. **Rust host**: All prime logic + NN inference + complex math
2. **GPU (WGSL)**: Pure shader, no NN, just uniforms/textures/buffers
3. **Bridge**: ControlState → SuperInstance → WGSL uniforms

### Keeping it out of the fragile preview path

The Control Engine does NOT touch:
- `editor_ui.rs` — preview rendering
- `shader_renderer.rs` — WGPU rendering pipeline
- `bevy_app.rs` — Bevy app setup (not yet wired)

Instead, it runs in a separate system that only touches:
- A small "control buffer" resource
- Minimal uniforms
- A simple instance set

### Future Work

- [ ] Train a small hybrid real/complex NN model in Python
- [ ] Export to ONNX and wire via `ort` crate
- [ ] Wire Control Engine as a Bevy system in `bevy_app.rs`
- [ ] Real audio/MIDI/OSC → control engine input pipeline
- [ ] Node graph integration: prime intervals determine node activation
