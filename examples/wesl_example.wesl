// Example WESL shader with imports and conditional compilation

// Import common utilities
import "math_utils.wesl";

// Conditional compilation
#if DEBUG
    @group(0) @binding(0) var<uniform> debug_mode: f32;
#else
    @group(0) @binding(0) var<uniform> time: f32;
#endif

@group(0) @binding(1) var<uniform> resolution: vec2<f32>;

@fragment
fn main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / resolution;
    
    // Use imported utility function
    let color = vec3<f32>(sin(time + uv.x), cos(time + uv.y), 0.5);
    
    #if DEBUG
        // Debug visualization
        return vec4<f32>(color * debug_mode, 1.0);
    #else
        return vec4<f32>(color, 1.0);
    #endif
}