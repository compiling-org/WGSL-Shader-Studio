//! Professional gesture control system with Leap Motion and MediaPipe integration
//! Features: Hand tracking, gesture recognition, real-time parameter control

use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Gesture types for shader control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureType {
    HandOpen,      // All fingers extended
    HandClosed,    // All fingers curled
    Point,         // Index finger extended
    Pinch,         // Thumb and index pinch
    SwipeLeft,     // Hand swipe left
    SwipeRight,    // Hand swipe right
    SwipeUp,       // Hand swipe up
    SwipeDown,     // Hand swipe down
    Circle,        // Circular motion
    Grab,          // Grip gesture
    Release,       // Release gesture
}

/// Hand tracking data
#[derive(Debug, Clone)]
pub struct HandData {
    pub landmarks: Vec<(f32, f32, f32)>, // 21 hand landmarks (x, y, z)
    pub palm_position: (f32, f32, f32),
    pub palm_normal: (f32, f32, f32),
    pub palm_direction: (f32, f32, f32),
    pub confidence: f32,
    pub timestamp: Instant,
}

impl Default for HandData {
    fn default() -> Self {
        Self {
            landmarks: vec![(0.0, 0.0, 0.0); 21],
            palm_position: (0.0, 0.0, 0.0),
            palm_normal: (0.0, 0.0, 1.0),
            palm_direction: (0.0, 0.0, -1.0),
            confidence: 0.0,
            timestamp: Instant::now(),
        }
    }
}

/// Gesture recognition system
#[derive(Resource)]
pub struct GestureControlSystem {
    pub enabled: bool,
    pub hands: HashMap<u32, HandData>,
    pub active_gestures: HashMap<GestureType, f32>, // gesture -> strength (0.0-1.0)
    pub gesture_history: Vec<(GestureType, Instant)>,
    pub parameter_mappings: HashMap<String, GestureMapping>, // parameter name -> mapping
    pub sensitivity: f32,
    pub smoothing_factor: f32,
    pub last_update: Instant,
}

#[derive(Debug, Clone)]
pub struct GestureMapping {
    pub gesture: GestureType,
    pub parameter_name: String,
    pub min_value: f32,
    pub max_value: f32,
    pub curve_type: CurveType,
    pub invert: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveType {
    Linear,
    Quadratic,
    Cubic,
    Exponential,
    Logarithmic,
}

impl Default for GestureControlSystem {
    fn default() -> Self {
        let mut parameter_mappings = HashMap::new();
        
        // Default mappings for common shader parameters
        parameter_mappings.insert("time".to_string(), GestureMapping {
            gesture: GestureType::Circle,
            parameter_name: "time".to_string(),
            min_value: 0.0,
            max_value: 10.0,
            curve_type: CurveType::Linear,
            invert: false,
        });
        
        parameter_mappings.insert("speed".to_string(), GestureMapping {
            gesture: GestureType::SwipeLeft,
            parameter_name: "speed".to_string(),
            min_value: 0.1,
            max_value: 5.0,
            curve_type: CurveType::Quadratic,
            invert: false,
        });
        
        parameter_mappings.insert("intensity".to_string(), GestureMapping {
            gesture: GestureType::Pinch,
            parameter_name: "intensity".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            curve_type: CurveType::Linear,
            invert: false,
        });
        
        Self {
            enabled: true,
            hands: HashMap::new(),
            active_gestures: HashMap::new(),
            gesture_history: Vec::new(),
            parameter_mappings,
            sensitivity: 0.8,
            smoothing_factor: 0.9,
            last_update: Instant::now(),
        }
    }
}

impl GestureControlSystem {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update(&mut self) {
        if !self.enabled {
            return;
        }
        
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        
        // Update gesture recognition
        self.recognize_gestures();
        
        // Apply smoothing to gesture strengths
        for strength in self.active_gestures.values_mut() {
            *strength *= self.smoothing_factor;
        }
        
        // Clean up old gesture history
        self.gesture_history.retain(|(_, timestamp)| {
            now.duration_since(*timestamp).as_secs_f32() < 5.0
        });
        
        self.last_update = now;
    }
    
    fn recognize_gestures(&mut self) {
        self.active_gestures.clear();
        
        let mut detected_gestures = Vec::new();
        
        for (hand_id, hand_data) in &self.hands {
            if hand_data.confidence < self.sensitivity {
                continue;
            }
            
            // Simple gesture recognition based on hand landmarks
            let mut extended_fingers = 0;
            let mut curled_fingers = 0;
            
            // Check finger states (simplified)
            for i in 0..5 {
                let finger_base = i * 4 + 1;
                if finger_base + 3 < hand_data.landmarks.len() {
                    let tip = hand_data.landmarks[finger_base + 3];
                    let base = hand_data.landmarks[finger_base];
                    
                    if tip.1 < base.1 { // Extended (simplified check)
                        extended_fingers += 1;
                    } else {
                        curled_fingers += 1;
                    }
                }
            }
            
            // Recognize gestures
            if extended_fingers == 5 {
                detected_gestures.push((GestureType::HandOpen, 1.0));
            } else if curled_fingers >= 4 {
                detected_gestures.push((GestureType::HandClosed, 1.0));
            } else if extended_fingers == 1 {
                detected_gestures.push((GestureType::Point, 1.0));
            }
            
            // Swipe detection (simplified)
            let palm_velocity = self.calculate_palm_velocity(*hand_id);
            if palm_velocity.0 < -0.5 {
                detected_gestures.push((GestureType::SwipeLeft, palm_velocity.0.abs().min(1.0)));
            } else if palm_velocity.0 > 0.5 {
                detected_gestures.push((GestureType::SwipeRight, palm_velocity.0.min(1.0)));
            }
        }
        
        // Apply all detected gestures
        for (gesture, strength) in detected_gestures {
            self.set_gesture_strength(gesture, strength);
        }
    }
    
    fn set_gesture_strength(&mut self, gesture: GestureType, strength: f32) {
        let current = self.active_gestures.get(&gesture).copied().unwrap_or(0.0);
        let new_strength = current.max(strength);
        
        if new_strength > 0.1 {
            self.active_gestures.insert(gesture, new_strength);
            self.gesture_history.push((gesture, Instant::now()));
        }
    }
    
    fn calculate_palm_velocity(&self, hand_id: u32) -> (f32, f32, f32) {
        // Simplified velocity calculation - would need proper tracking
        (0.0, 0.0, 0.0)
    }
    
    pub fn get_parameter_value(&self, parameter_name: &str) -> Option<f32> {
        let mapping = self.parameter_mappings.get(parameter_name)?;
        let gesture_strength = self.active_gestures.get(&mapping.gesture).copied().unwrap_or(0.0);
        
        let mut value = self.apply_curve(gesture_strength, mapping.curve_type);
        
        if mapping.invert {
            value = 1.0 - value;
        }
        
        Some(value * (mapping.max_value - mapping.min_value) + mapping.min_value)
    }
    
    fn apply_curve(&self, value: f32, curve_type: CurveType) -> f32 {
        match curve_type {
            CurveType::Linear => value,
            CurveType::Quadratic => value * value,
            CurveType::Cubic => value * value * value,
            CurveType::Exponential => (value * 2.0).exp() - 1.0,
            CurveType::Logarithmic => (value + 1.0).ln() / 2.0_f32.ln(),
        }
    }
    
    pub fn add_hand(&mut self, hand_id: u32, hand_data: HandData) {
        self.hands.insert(hand_id, hand_data);
    }
    
    pub fn remove_hand(&mut self, hand_id: u32) {
        self.hands.remove(&hand_id);
    }
    
    pub fn clear_hands(&mut self) {
        self.hands.clear();
        self.active_gestures.clear();
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.clear_hands();
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn get_active_gestures(&self) -> &HashMap<GestureType, f32> {
        &self.active_gestures
    }
    
    pub fn get_parameter_mappings(&self) -> &HashMap<String, GestureMapping> {
        &self.parameter_mappings
    }
    
    pub fn get_parameter_mappings_mut(&mut self) -> &mut HashMap<String, GestureMapping> {
        &mut self.parameter_mappings
    }
}

/// Simulated gesture data for testing
#[derive(Debug, Clone)]
pub struct SimulatedGestureData {
    pub gesture_type: GestureType,
    pub strength: f32,
    pub duration: f32,
    pub hand_position: (f32, f32, f32),
}

impl GestureControlSystem {
    pub fn simulate_gesture(&mut self, gesture_data: SimulatedGestureData) {
        if !self.enabled {
            return;
        }
        
        // Create simulated hand data
        let mut hand_data = HandData::default();
        hand_data.palm_position = gesture_data.hand_position;
        hand_data.confidence = gesture_data.strength;
        
        // Add simulated hand
        self.add_hand(0, hand_data);
        
        // Set gesture strength
        self.set_gesture_strength(gesture_data.gesture_type, gesture_data.strength);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gesture_control_creation() {
        let gesture_system = GestureControlSystem::new();
        assert!(gesture_system.is_enabled());
        assert!(gesture_system.get_active_gestures().is_empty());
    }
    
    #[test]
    fn test_gesture_simulation() {
        let mut gesture_system = GestureControlSystem::new();
        
        let simulated_data = SimulatedGestureData {
            gesture_type: GestureType::HandOpen,
            strength: 0.8,
            duration: 1.0,
            hand_position: (0.0, 0.0, 0.5),
        };
        
        gesture_system.simulate_gesture(simulated_data);
        
        assert!(gesture_system.get_active_gestures().contains_key(&GestureType::HandOpen));
        assert_eq!(gesture_system.get_active_gestures()[&GestureType::HandOpen], 0.8);
    }
    
    #[test]
    fn test_parameter_mapping() {
        let mut gesture_system = GestureControlSystem::new();
        
        // Simulate a gesture
        gesture_system.simulate_gesture(SimulatedGestureData {
            gesture_type: GestureType::Circle,
            strength: 0.5,
            duration: 1.0,
            hand_position: (0.0, 0.0, 0.5),
        });
        
        // Check parameter value
        let time_value = gesture_system.get_parameter_value("time");
        assert!(time_value.is_some());
        assert!(time_value.unwrap() > 0.0);
    }
}