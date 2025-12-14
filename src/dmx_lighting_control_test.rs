//! Test DMX lighting control system

use crate::dmx_lighting_control::*;

#[test]
fn test_dmx_config_creation() {
    let config = DmxConfig::default();
    assert_eq!(config.universe_count, 4);
    assert_eq!(config.frame_rate, 44);
    assert_eq!(config.artnet_enabled, true);
    assert_eq!(config.enabled, false);
}

#[test]
fn test_dmx_control_creation() {
    let config = DmxConfig::default();
    let control = DmxLightingControl::new(config);
    let status = control.get_status();
    
    assert_eq!(status.universe_count, 4);
    assert_eq!(status.frame_rate, 44);
    assert_eq!(status.is_running, false);
}

#[test]
fn test_dmx_channel_mapping() {
    let mapping = DmxChannelMapping {
        channel: 1,
        universe: 1,
        parameter_name: "test_param".to_string(),
        min_value: 0,
        max_value: 255,
        default_value: 128,
        channel_type: DmxChannelType::Intensity,
    };
    
    assert_eq!(mapping.channel, 1);
    assert_eq!(mapping.universe, 1);
    assert_eq!(mapping.parameter_name, "test_param");
    assert_eq!(mapping.channel_type, DmxChannelType::Intensity);
}

#[test]
fn test_dmx_channel_value() {
    let value = DmxChannelValue {
        channel: 1,
        universe: 1,
        value: 128,
        timestamp: std::time::Instant::now(),
        source: "test".to_string(),
    };
    
    assert_eq!(value.channel, 1);
    assert_eq!(value.universe, 1);
    assert_eq!(value.value, 128);
    assert_eq!(value.source, "test");
}

#[test]
fn test_dmx_status() {
    let status = DmxStatus {
        is_running: false,
        connection_status: "Not initialized".to_string(),
        universe_count: 4,
        active_universes: 4,
        num_mappings: 8,
        num_channels: 8,
        frame_counter: 0,
        frame_rate: 44,
    };
    
    assert_eq!(status.is_running, false);
    assert_eq!(status.universe_count, 4);
    assert_eq!(status.frame_rate, 44);
    assert_eq!(status.num_mappings, 8);
}