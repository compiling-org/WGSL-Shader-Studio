// Audio Reactive Test Shader - WGSL Shader Studio
// This shader demonstrates real-time audio analysis integration

struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    audio_volume: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_treble: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / uniforms.resolution;
    let time = uniforms.time;
    
    // Audio-reactive color mixing
    let bass_boost = uniforms.audio_bass * 2.0;
    let mid_boost = uniforms.audio_mid * 1.5;
    let treble_boost = uniforms.audio_treble * 3.0;
    let volume_boost = uniforms.audio_volume * 0.5;
    
    // Create audio-reactive wave patterns
    let wave_freq = 8.0 + bass_boost * 12.0;
    let wave_amp = 0.1 + volume_boost * 0.3;
    let wave_speed = 2.0 + treble_boost * 4.0;
    
    // Primary wave (bass reactive)
    let wave1 = sin(uv.x * wave_freq + time * wave_speed) * wave_amp;
    
    // Secondary wave (mid reactive)
    let wave2 = sin(uv.y * (wave_freq * 1.5) + time * wave_speed * 0.7) * (wave_amp * 0.8);
    
    // Tertiary wave (treble reactive)
    let wave3 = sin((uv.x + uv.y) * (wave_freq * 2.0) + time * wave_speed * 1.5) * (wave_amp * 0.5);
    
    // Combine waves
    let combined_wave = wave1 + wave2 + wave3;
    
    // Audio-reactive color channels
    let r = 0.5 + 0.5 * sin(time + uv.x * 6.28318 + bass_boost) + combined_wave * bass_boost;
    let g = 0.5 + 0.5 * cos(time * 0.8 + uv.y * 4.0 + mid_boost) + combined_wave * mid_boost;
    let b = 0.5 + 0.5 * sin(time * 1.2 + (uv.x + uv.y) * 8.0 + treble_boost) + combined_wave * treble_boost;
    
    // Volume-based brightness modulation
    let brightness = 0.7 + volume_boost * 0.6;
    
    // Beat detection flash effect
    let flash = step(0.8, uniforms.audio_volume);
    
    let final_color = vec3<f32>(
        (r * brightness + flash * 0.3).clamp(0.0, 1.0),
        (g * brightness + flash * 0.2).clamp(0.0, 1.0),
        (b * brightness + flash * 0.4).clamp(0.0, 1.0)
    );
    
    return vec4<f32>(final_color, 1.0);
}

// Helper function (WGSL doesn't have built-in step function)
fn step(edge: f32, x: f32) -> f32 {
    return select(0.0, 1.0, x >= edge);
}