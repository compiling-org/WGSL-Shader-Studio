# SuperInstance Architecture Integration Guide

## Overview

This document describes how the [SuperInstance](https://github.com/SuperInstance/SuperInstance) ecosystem —
FLUX VM, PLATO constraint engines, and Conservation Theory (γ + η = C) — integrates with WGSL Shader Studio
as an infrastructure and methodology layer for deterministic, budget-constrained shader compilation.

SuperInstance is a polyglot software ecosystem built around two motifs (hermit crab, 12V fishing boat)
and one conservation law. The integration here applies those principles to shader development:

- **FLUX VM** → Deterministic bytecode execution for shader compilation, replacing expensive LLM inference
- **PLATO Engines** → Constraint-based agent coordination for compilation pipelines
- **Conservation Theory** → Budget enforcement across crystallized (γ) vs. live (η) compute

---

## 1. FLUX VM Integration

### What FLUX Provides

FLUX is a register-based virtual machine with:
- 16 general-purpose + 16 floating-point registers
- A2A (any-to-any) opcodes
- Three implementations (Python, Rust, JS) verified byte-identical
- Deterministic execution with measurable budget consumption

### Shader Compilation with FLUX Bytecode

Instead of invoking LLM calls for every shader conversion operation (ISF→WGSL, GLSL→WGSL, etc.),
FLUX bytecode represents the **crystallized (γ)** compilation patterns:

```
┌──────────────────────────────────────────────────────────┐
│                    Shader Source                         │
│        (WGSL / GLSL / HLSL / ISF / WESL)                │
└────────────────────────┬─────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  FLUX Compiler (γ — crystallized bytecode)              │
│  ┌────────────────────────────────────────────────────┐ │
│  │  FLUX Program: "wgsl_compile"                     │ │
│  │  - Register allocation (16 GP + 16 FP)            │ │
│  │  - AST traversal encoded as A2A opcodes           │ │
│  │  - Budget: ~0.0001¢ / compilation                 │ │
│  └────────────────────────────────────────────────────┘ │
└────────────────────────┬─────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│  Shader bytecode → Execution → Compiled output          │
└──────────────────────────────────────────────────────────┘
```

### Pipeline: Crystallized (γ) vs. Live (η)

```python
# Conceptual integration
from flux_framework import Interpreter
from conservation_enforcer import ConservationEnforcer

interpreter = Interpreter()
enforcer = ConservationEnforcer()

def compile_shader(wgsl_code: str) -> bytes:
    # Phase 1: Try cached bytecode (γ — $0.0001/call)
    bytecode = interpreter.compile(wgsl_code)
    if enforcer.enforce("compile", bytecode).allowed:
        return interpreter.execute(bytecode)
    
    # Phase 2: Fall back to LLM (η — $0.01/call) 
    result = llm_compile(wgsl_code)
    # Crystallize the result into bytecode for future use
    interpreter.learn(result)
    return result
```

---

## 2. PLATO Engine Integration

### What PLATO Provides

PLATO is a family of constraint engines (5 implementations: C, Rust, Elixir, Zig, Python)
at 9–10/10 conformance. It provides:

- Room-level contexts for agents
- Sensor/history/alarm management
- Deadband wakefulness (reduces server costs by ~90%)
- 1KB response sizes

### Agent Coordination for Shader Pipelines

PLATO rooms organize shader compilation agents:

```
┌────────────────────────────────────────────────────────────┐
│  PLATO Room: "shader_compiler"                            │
│  Sensors:                                                  │
│    - shader_source: WGSL/GLSL/HLSL text                    │
│    - target_format: wgsl | glsl | hlsl | spirv             │
│    - budget_remaining: f32                                 │
│    - gpu_capabilities: [dx12, vulkan, metal, webgpu]       │
│  History:                                                   │
│    - last_compile_time: timestamp                           │
│    - error_rate: f32                                        │
│  Alarms:                                                    │
│    - budget_exceeded: if budget_remaining < 0.0             │
│    - compile_failed: if error_rate > threshold              │
└────────────────────────────────────────────────────────────┘
```

### Protocol Example (Conceptual)

```rust
// PLATO room interaction for shader compilation
use plato_core_rs::{Room, Sensor, Alarm};

struct ShaderCompilationAgent {
    room: Room,
    budget: f32,
}

impl ShaderCompilationAgent {
    fn compile(&mut self, source: &str) -> Result<Vec<u8>, String> {
        let context = self.room.get_context();
        
        // Check budget remaining
        if context.budget_remaining <= 0.0 {
            self.room.trigger_alarm("budget_exceeded");
            return Err("Budget exhausted".into());
        }
        
        // Execute compilation through FLUX
        let bytecode = self.compile_to_flux(source)?;
        let result = self.execute_bytecode(&bytecode)?;
        
        // Update sensor readings
        self.room.update_sensor("budget_remaining", self.budget);
        self.room.update_sensor("last_compile_time", chrono::Utc::now());
        
        Ok(result)
    }
}
```

---

## 3. Conservation Theory (γ + η = C)

### Applied to Shader Compilation

The conservation law states: **γ + η = C**

Where:
- **γ** = crystallized cognition (deterministic bytecode, cached patterns, FLUX programs)
- **η** = live entropy (LLM inference, heuristic search, uncertain computation)
- **C** = total budget (fixed, measurable, cannot be exceeded)

### Budget Categories for Shader Studio

| Operation | γ Cost | η Cost | C Budget | Strategy |
|-----------|--------|--------|----------|----------|
| WGSL → SPIRV (naga) | ~0.0001¢ | — | Fixed | Always γ (pure deterministic) |
| ISF → WGSL conversion | ~0.0001¢ | ~0.01¢ | 0.05¢/day | Try γ first; fallback η |
| GLSL → WGSL transpile | ~0.0005¢ | ~0.02¢ | 0.10¢/day | Crystallize common patterns |
| Node graph → WGSL gen | ~0.0002¢ | ~0.015¢ | 0.08¢/day | Cache generated bytecode |
| LLM-assisted shader fix | — | ~0.05¢ | 0.20¢/day | η only (LLM required) |

### Implementation in Shader Pipeline

```rust
/// Conservation-aware shader compiler
struct ConservationShaderCompiler {
    budget_tracker: BudgetTracker,
    flux_interpreter: FluxInterpreter,
    llm_client: Option<LlmClient>,
}

struct BudgetTracker {
    /// Daily budget for crystallized operations (γ)
    crystallized_budget: f64,
    /// Per-call budget for live operations (η)
    live_budget: f64,
    /// Total consumed budget
    consumed: f64,
}

impl BudgetTracker {
    fn can_compile(&self, operation: &str) -> bool {
        match operation {
            "wgsl_parse" | "spirv_generate" => true, // Always free (γ)
            "isf_convert" | "glsl_transpile" => self.consumed < self.crystallized_budget,
            "llm_optimize" | "error_fix" => self.consumed < self.crystallized_budget + self.live_budget,
            _ => false,
        }
    }
}
```

### Crystallization Over Time

As shaders are compiled repeatedly, patterns crystallize into FLUX bytecode:

```
Day 1:  η-heavy (LLM compiles everything)
        Cost: $0.50/day
        Latency: ~5s per compile

Day 7:  γ/η hybrid (common patterns cached)
        Cost: $0.15/day  
        Latency: ~500ms per compile

Day 30: γ-dominant (all patterns crystallized)
        Cost: $0.02/day
        Latency: ~50ms per compile
```

---

## 4. Mapping SuperInstance 7-Layer Architecture to Shader Studio

| Layer | SuperInstance | WGSL Shader Studio Mapping |
|-------|---------------|---------------------------|
| **1. Substrate** | 12V boat, Signal-K bus | WGPU device, GPU adapter, surface config |
| **2. VM** | FLUX register-based bytecode | naga WGSL parser, SPIRV generation |
| **3. Engines** | PLATO constraint family | Shader compiler, converter pipeline |
| **4. Policy/Enforce** | Conservation Enforcer | Budget tracker, compile gate checks |
| **5. Orchestration** | Cloudflare Workers fleet | Background shader scan, async compile queue |
| **6. Agents & Rooms** | PLATO rooms, working animals | Shader compilation agents, conversion workers |
| **7. Artifacts** | ~1,800 essays, manifestos | Compiled shaders, project files, documentation |

---

## 5. Getting Started with SuperInstance in Shader Studio

### Prerequisites

```bash
# Install FLUX VM (Python)
pip install flux-vm

# Install Conservation Enforcer
pip install conservation-enforcer

# Install PLATO core (Python)
pip install plato-core

# Rust crates (optional, for native performance)
cargo add fluxvm
cargo add conservation-enforcer-rs
cargo add plato-core-rs
```

### CLI Usage

```bash
# Compile with FLUX bytecode (crystallized path)
cargo run -- --flux-compile shaders/my_shader.wgsl

# Track budget usage
cargo run -- --budget-track --budget-daily 0.10

# Run with PLATO room coordination
cargo run -- --plato-room "shader_pipeline" --plato-port 8847
```

### Configuration

Add to your project's `project.yaml` or `Cargo.toml` features:

```yaml
# project.yaml
superinstance:
  flux:
    enabled: true
    cache_dir: .flux-cache
  conservation:
    daily_budget: 0.10  # $0.10/day total
    crystallized_ratio: 0.8  # 80% crystallized, 20% live
  plato:
    enabled: false  # Enable for multi-agent coordination
    room: shader_compiler
    port: 8847
```

---

## 6. Benefits Summary

| Benefit | Before (LLM-only) | After (FLUX + Conservation) |
|---------|-------------------|---------------------------|
| Cost per compilation | ~$0.01–0.05 | ~$0.0001 (crystallized) |
| Latency | ~2–5s | ~10–50ms (bytecode) |
| Determinism | Non-deterministic | Bit-identical across runs |
| Budget control | None (pay per call) | Fixed, enforceable |
| Offline capability | Requires API access | Fully offline (γ path) |
| Scaling cost | Linear with usage | Sub-linear (crystallization) |

---

## References

- [SuperInstance Canonical Guide](https://github.com/SuperInstance/SuperInstance)
- [FLUX Core](https://github.com/SuperInstance/flux-core) — Register-based bytecode VM
- [PLATO Engine (C)](https://github.com/SuperInstance/plato-engine-block-c) — Constraint engine
- [Conservation Enforcer](https://github.com/SuperInstance/conservation-enforcer) — Policy layer
- [Conservation Theory Core (Rust)](https://github.com/SuperInstance/constraint-theory-core) — Rust twin

---

*Integration Guide v1.0 — Aligned with SuperInstance 2026-07-22 architecture*
