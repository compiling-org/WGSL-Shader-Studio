use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

pub fn probe_scaling_modes() {
    // Uncomment one by one or let compiler error on all to see which exists
    let _s1 = ScalingMode::WindowSize(1.0); 
    // let _s2 = ScalingMode::FixedVertical(1.0);
    // let _s3 = ScalingMode::FixedHorizontal(1.0);
    // let _s4 = ScalingMode::Fixed { width: 1.0, height: 1.0 };
    // let _s5 = ScalingMode::AutoMin { min_width: 1.0, min_height: 1.0 };
    // let _s6 = ScalingMode::AutoMax { max_width: 1.0, max_height: 1.0 };
    // let _s7 = ScalingMode::None;
}
