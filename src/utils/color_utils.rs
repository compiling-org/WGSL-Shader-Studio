//! Professional color blending utilities
//! Ported from FluxReel: https://github.com/makalin/FluxReel/blob/main/rust/fluxreel-core/src/blend_modes.rs

/// Apply blend mode to two RGB colors (0.0 - 1.0 range)
pub fn apply_blend_mode(
    base: [f32; 3],
    blend: [f32; 3],
    mode: &str,
    opacity: f32,
) -> [f32; 3] {
    let result = match mode.to_lowercase().as_str() {
        "multiply" => multiply(base, blend),
        "screen" => screen(base, blend),
        "overlay" => overlay(base, blend),
        "soft_light" => soft_light(base, blend),
        "hard_light" => hard_light(base, blend),
        "color_dodge" => color_dodge(base, blend),
        "color_burn" => color_burn(base, blend),
        "darken" => darken(base, blend),
        "lighten" => lighten(base, blend),
        "difference" => difference(base, blend),
        "exclusion" => exclusion(base, blend),
        "add" => add(base, blend),
        "subtract" => subtract(base, blend),
        _ => blend, // Default to blend color (Normal-ish if opacity is 1.0)
    };
    
    // Lerp with opacity
    [
        base[0] + (result[0] - base[0]) * opacity,
        base[1] + (result[1] - base[1]) * opacity,
        base[2] + (result[2] - base[2]) * opacity,
    ]
}

fn multiply(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [base[0] * blend[0], base[1] * blend[1], base[2] * blend[2]]
}

fn screen(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [1.0 - (1.0 - base[0]) * (1.0 - blend[0]),
     1.0 - (1.0 - base[1]) * (1.0 - blend[1]),
     1.0 - (1.0 - base[2]) * (1.0 - blend[2])]
}

fn overlay(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [
        if base[0] < 0.5 { 2.0 * base[0] * blend[0] } else { 1.0 - 2.0 * (1.0 - base[0]) * (1.0 - blend[0]) },
        if base[1] < 0.5 { 2.0 * base[1] * blend[1] } else { 1.0 - 2.0 * (1.0 - base[1]) * (1.0 - blend[1]) },
        if base[2] < 0.5 { 2.0 * base[2] * blend[2] } else { 1.0 - 2.0 * (1.0 - base[2]) * (1.0 - blend[2]) },
    ]
}

fn soft_light(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    fn sl(b: f32, s: f32) -> f32 {
        if s < 0.5 {
            b - (1.0 - 2.0 * s) * b * (1.0 - b)
        } else {
            b + (2.0 * s - 1.0) * (b.sqrt() - b)
        }
    }
    [sl(base[0], blend[0]), sl(base[1], blend[1]), sl(base[2], blend[2])]
}

fn hard_light(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    overlay(blend, base)
}

fn color_dodge(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [
        if blend[0] == 1.0 { 1.0 } else { (base[0] / (1.0 - blend[0])).min(1.0) },
        if blend[1] == 1.0 { 1.0 } else { (base[1] / (1.0 - blend[1])).min(1.0) },
        if blend[2] == 1.0 { 1.0 } else { (base[2] / (1.0 - blend[2])).min(1.0) },
    ]
}

fn color_burn(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [
        if blend[0] == 0.0 { 0.0 } else { (1.0 - (1.0 - base[0]) / blend[0]).max(0.0) },
        if blend[1] == 0.0 { 0.0 } else { (1.0 - (1.0 - base[1]) / blend[1]).max(0.0) },
        if blend[2] == 0.0 { 0.0 } else { (1.0 - (1.0 - base[2]) / blend[2]).max(0.0) },
    ]
}

fn darken(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [base[0].min(blend[0]), base[1].min(blend[1]), base[2].min(blend[2])]
}

fn lighten(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [base[0].max(blend[0]), base[1].max(blend[1]), base[2].max(blend[2])]
}

fn difference(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [(base[0] - blend[0]).abs(), (base[1] - blend[1]).abs(), (base[2] - blend[2]).abs()]
}

fn exclusion(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [base[0] + blend[0] - 2.0 * base[0] * blend[0],
     base[1] + blend[1] - 2.0 * base[1] * blend[1],
     base[2] + blend[2] - 2.0 * base[2] * blend[2]]
}

fn add(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [(base[0] + blend[0]).min(1.0), (base[1] + blend[1]).min(1.0), (base[2] + blend[2]).min(1.0)]
}

fn subtract(base: [f32; 3], blend: [f32; 3]) -> [f32; 3] {
    [(base[0] - blend[0]).max(0.0), (base[1] - blend[1]).max(0.0), (base[2] - blend[2]).max(0.0)]
}
