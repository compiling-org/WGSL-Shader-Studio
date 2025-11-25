@group(0) @binding(0) var<uniform> params: array<f32, 4>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<f32>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x + global_id.y * 8u;
    if (index >= arrayLength(&output_buffer)) {
        return;
    }
    
    let speed = params[0];
    let intensity = params[1];
    let offset = params[2];
    let scale = params[3];
    
    let x = f32(global_id.x) * 0.1;
    let y = f32(global_id.y) * 0.1;
    let time = f32(global_id.x + global_id.y) * 0.01;
    
    output_buffer[index] = sin(x * scale + time * speed) * cos(y * scale + time * speed) * intensity + offset;
}