use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

pub fn probe() {
    // 1. Probe ScalingMode Variants
    let _ = ScalingMode::WindowSize(1.0);
    let _ = ScalingMode::WindowSize;
    let _ = ScalingMode::FixedVertical(1.0);
    let _ = ScalingMode::FixedVertical;
    let _ = ScalingMode::FixedHorizontal(1.0);
    let _ = ScalingMode::FixedHorizontal;
    let _ = ScalingMode::Fixed { width: 1.0, height: 1.0 };
    let _ = ScalingMode::Fixed(1.0, 1.0);
    let _ = ScalingMode::AutoMin { min_width: 1.0, min_height: 1.0 };
    let _ = ScalingMode::AutoMax { max_width: 1.0, max_height: 1.0 };
    let _ = ScalingMode::None;

    // 2. Probe ClearColorConfig paths
    let _ = bevy::core_pipeline::clear_color::ClearColorConfig::None;
    // let _ = bevy::core_pipeline::core_2d::ClearColorConfig::None;
}
