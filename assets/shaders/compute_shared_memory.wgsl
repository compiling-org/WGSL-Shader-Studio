// Workgroup shared memory compute shader example
// Demonstrates efficient parallel reduction using shared memory

struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    input_size: u32,
    num_workgroups: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> input_data: array<f32>;
@group(0) @binding(2) var<storage, read_write> output_data: array<f32>;
@group(0) @binding(3) var output_texture: texture_storage_2d<rgba8unorm, write>;

var<workgroup> shared_data: array<f32, 256>;

@compute @workgroup_size(256, 1, 1)
fn parallel_reduction(@builtin(global_invocation_id) global_id: vec3<u32>,
                     @builtin(local_invocation_id) local_id: vec3<u32>,
                     @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let tid = local_id.x;
    let gid = global_id.x;
    
    // Load data into shared memory
    if (gid < uniforms.input_size) {
        shared_data[tid] = input_data[gid];
    } else {
        shared_data[tid] = 0.0;
    }
    
    workgroupBarrier();
    
    // Parallel reduction in shared memory
    for (var s = 256u / 2u; s > 0u; s = s >> 1u) {
        if (tid < s && (gid + s) < uniforms.input_size) {
            shared_data[tid] = shared_data[tid] + shared_data[tid + s];
        }
        workgroupBarrier();
    }
    
    // Write partial result
    if (tid == 0u) {
        output_data[workgroup_id.x] = shared_data[0];
    }
}

// Visualization shader that uses the reduction results
@compute @workgroup_size(8, 8, 1)
fn visualize_reduction(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coord = global_id.xy;
    let texture_size = textureDimensions(output_texture);
    
    if (coord.x >= texture_size.x || coord.y >= texture_size.y) {
        return;
    }
    
    let uv = vec2<f32>(f32(coord.x), f32(coord.y)) / vec2<f32>(f32(texture_size.x), f32(texture_size.y));
    
    // Calculate sum from reduction results
    var total_sum = 0.0;
    for (var i = 0u; i < uniforms.num_workgroups; i = i + 1u) {
        total_sum = total_sum + output_data[i];
    }
    
    // Create visualization based on the data
    let normalized_coord = f32(coord.x) / f32(texture_size.x);
    let data_index = u32(normalized_coord * f32(uniforms.input_size));
    
    var color = vec3<f32>(0.0);
    
    if (data_index < uniforms.input_size) {
        let value = input_data[data_index];
        let normalized_value = value / (total_sum + 0.001);
        
        // Color based on value
        let hue = fract(normalized_value * 3.0 + uniforms.time * 0.2);
        color = hsv2rgb(vec3<f32>(hue, 0.8, 0.8));
        
        // Add waveform visualization
        let wave_y = sin(normalized_coord * 20.0 + uniforms.time * 2.0) * 0.1 + 0.5;
        let wave_intensity = exp(-abs(uv.y - wave_y) * 50.0);
        color = mix(color, vec3<f32>(1.0), wave_intensity * 0.5);
        
        // Highlight the current data point
        if (abs(f32(coord.y) / f32(texture_size.y) - normalized_value) < 0.01) {
            color = mix(color, vec3<f32>(1.0), 0.8);
        }
    }
    
    // Add grid lines
    let grid_x = step(0.98, fract(uv.x * 20.0));
    let grid_y = step(0.98, fract(uv.y * 20.0));
    color = mix(color, vec3<f32>(0.3), max(grid_x, grid_y));
    
    textureStore(output_texture, coord, vec4<f32>(color, 1.0));
}

// Helper function to convert HSV to RGB
fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}

// Generate test data for reduction
@compute @workgroup_size(256, 1, 1)
fn generate_test_data(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let gid = global_id.x;
    if (gid >= uniforms.input_size) {
        return;
    }
    
    // Generate sinusoidal test data
    let t = f32(gid) / f32(uniforms.input_size);
    let value = sin(t * 10.0 + uniforms.time) * cos(t * 15.0 + uniforms.time * 1.5);
    input_data[gid] = value * value; // Square to make it positive
}