//! Animation easing functions
//! Ported and adapted from FluxReel: https://github.com/makalin/FluxReel/blob/main/rust/fluxreel-core/src/utils.rs

use std::f32::consts::PI;

/// Comprehensive easing functions for animations
pub fn ease_function(ease_type: &str, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match ease_type {
        "linear" => t,
        
        // Quadratic
        "ease_in" | "quad_in" => t * t,
        "ease_out" | "quad_out" => 1.0 - (1.0 - t) * (1.0 - t),
        "ease_in_out" | "quad_in_out" => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - 2.0 * (1.0 - t) * (1.0 - t)
            }
        }
        
        // Cubic
        "cubic_in" => t * t * t,
        "cubic_out" => 1.0 - (1.0 - t).powi(3),
        "cubic_in_out" => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - 4.0 * (1.0 - t).powi(3)
            }
        }
        
        // Quartic
        "quart_in" => t * t * t * t,
        "quart_out" => 1.0 - (1.0 - t).powi(4),
        "quart_in_out" => {
            if t < 0.5 {
                8.0 * t * t * t * t
            } else {
                1.0 - 8.0 * (1.0 - t).powi(4)
            }
        }
        
        // Sine
        "sine_in" => 1.0 - (t * PI / 2.0).cos(),
        "sine_out" => (t * PI / 2.0).sin(),
        "sine_in_out" => -((PI * t).cos() - 1.0) / 2.0,
        
        // Exponential
        "expo_in" => if t == 0.0 { 0.0 } else { 2.0_f32.powf(10.0 * (t - 1.0)) },
        "expo_out" => if t == 1.0 { 1.0 } else { 1.0 - 2.0_f32.powf(-10.0 * t) },
        "expo_in_out" => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                2.0_f32.powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
            }
        }
        
        // Circular
        "circ_in" => 1.0 - (1.0 - t * t).sqrt(),
        "circ_out" => (1.0 - (t - 1.0) * (t - 1.0)).sqrt(),
        "circ_in_out" => {
            if t < 0.5 {
                (1.0 - (1.0 - 2.0 * t) * (1.0 - 2.0 * t)).sqrt() / 2.0
            } else {
                (1.0 + (2.0 * t - 1.0) * (2.0 * t - 1.0)).sqrt() / 2.0
            }
        }
        
        // Elastic
        "elastic_in" => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                -2.0_f32.powf(10.0 * (t - 1.0)) * ((t - 1.0 - 0.075) * (2.0 * PI) / 0.3).sin()
            }
        }
        "elastic_out" | "elastic" => {
            let c4 = (2.0 * PI) / 3.0;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            }
        }
        
        // Bounce
        "bounce_in" => 1.0 - bounce_out(1.0 - t),
        "bounce_out" => bounce_out(t),
        "bounce_in_out" => {
            if t < 0.5 {
                (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
            } else {
                (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
            }
        }
        
        _ => t,
    }
}

fn bounce_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}
