#[cfg(test)]
mod tests {
    use super::super::gesture_control::*;
    
    #[test]
    fn test_gesture_control_creation() {
        let gesture_control = GestureControlSystem::new();
        assert!(!gesture_control.is_enabled());
        assert_eq!(gesture_control.get_parameter_value("time"), None);
    }
    
    #[test]
    fn test_gesture_control_enable_disable() {
        let mut gesture_control = GestureControlSystem::new();
        gesture_control.set_enabled(true);
        assert!(gesture_control.is_enabled());
        
        gesture_control.set_enabled(false);
        assert!(!gesture_control.is_enabled());
    }
    
    #[test]
    fn test_simulated_gesture() {
        let mut gesture_control = GestureControlSystem::new();
        gesture_control.set_enabled(true);
        
        // Simulate a hand open gesture
        gesture_control.simulate_gesture(SimulatedGestureData {
            gesture_type: GestureType::HandOpen,
            strength: 0.8,
            duration: 2.0,
            hand_position: (0.5, 0.3, 0.2),
        });
        
        // Update the system
        gesture_control.update();
        
        // Check if gesture parameters are available
        assert!(gesture_control.get_parameter_value("time").is_some());
        assert!(gesture_control.get_parameter_value("speed").is_some());
        assert!(gesture_control.get_parameter_value("intensity").is_some());
    }
    
    #[test]
    fn test_gesture_parameter_mapping() {
        let mut gesture_control = GestureControlSystem::new();
        gesture_control.set_enabled(true);
        
        // Simulate multiple gestures
        gesture_control.simulate_gesture(SimulatedGestureData {
            gesture_type: GestureType::SwipeLeft,
            strength: 1.0,
            duration: 1.5,
            hand_position: (-0.2, 0.1, 0.3),
        });
        
        gesture_control.update();
        
        // Verify parameter values are in expected ranges
        if let Some(time_val) = gesture_control.get_parameter_value("time") {
            assert!(time_val >= 0.0 && time_val <= 10.0);
        }
        
        if let Some(speed_val) = gesture_control.get_parameter_value("speed") {
            assert!(speed_val >= 0.0 && speed_val <= 2.0);
        }
        
        if let Some(intensity_val) = gesture_control.get_parameter_value("intensity") {
            assert!(intensity_val >= 0.0 && intensity_val <= 1.0);
        }
    }
}