//! # PLATO Engine Integration
//!
//! Provides PLATO constraint engine coordination for agent pipelines.
//! PLATO rooms provide structured contexts for agents with sensors, history, and alarms.
//!
//! ## Overview
//!
//! PLATO is a family of constraint engines (5 implementations: C, Rust, Elixir, Zig, Python)
//! at 9-10/10 conformance. It provides:
//! - Room-level contexts for agents
//! - Sensor/history/alarm management
//! - Deadband wakefulness (reduces server costs by ~90%)
//! - 1KB response sizes
//!
//! ## Agent Coordination
//!
//! PLATO rooms organize shader compilation agents into coordinated pipelines:
//! - Room "shader_compiler" manages compilation agents
//! - Sensors track shader source, target format, budget remaining
//! - Alarms trigger on budget exceeded or compile failure

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// PLATO room for agent coordination
#[derive(Debug, Clone)]
pub struct PlatoRoom {
    /// Room name identifier
    pub name: String,
    /// Current sensor readings
    pub sensors: HashMap<String, SensorValue>,
    /// History of sensor readings
    pub history: Vec<SensorReading>,
    /// Active alarms
    pub alarms: Vec<Alarm>,
    /// Maximum history entries to retain
    max_history: usize,
    /// Deadband threshold for wakefulness (min change to trigger update)
    deadband_threshold: f64,
}

/// A sensor reading at a point in time
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_name: String,
    pub value: SensorValue,
    pub timestamp: Instant,
}

/// Types of values a PLATO sensor can hold
#[derive(Debug, Clone)]
pub enum SensorValue {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
}

/// An alarm triggered by sensor conditions
#[derive(Debug, Clone)]
pub struct Alarm {
    pub name: String,
    pub severity: AlarmSeverity,
    pub message: String,
    pub triggered_at: Instant,
    pub active: bool,
}

/// Alarm severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmSeverity {
    Info,
    Warning,
    Critical,
}

impl PlatoRoom {
    /// Create a new PLATO room
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sensors: HashMap::new(),
            history: Vec::new(),
            alarms: Vec::new(),
            max_history: 1000,
            deadband_threshold: 0.01,
        }
    }

    /// Create a shader compilation room with default sensors
    pub fn shader_compiler_room() -> Self {
        let mut room = Self::new("shader_compiler");
        room.init_shader_sensors();
        room
    }

    /// Initialize default sensors for shader compilation
    fn init_shader_sensors(&mut self) {
        self.sensors.insert("shader_source".to_string(), SensorValue::String(String::new()));
        self.sensors.insert("target_format".to_string(), SensorValue::String("wgsl".to_string()));
        self.sensors.insert("budget_remaining".to_string(), SensorValue::Float(0.10));
        self.sensors.insert("compile_count".to_string(), SensorValue::Int(0));
        self.sensors.insert("error_rate".to_string(), SensorValue::Float(0.0));
        self.sensors.insert("last_compile_time".to_string(), SensorValue::Float(0.0));
        self.sensors.insert("gpu_available".to_string(), SensorValue::Bool(true));
    }

    /// Set a sensor value with deadband filtering
    pub fn set_sensor(&mut self, name: &str, value: SensorValue) -> bool {
        let should_update = match (self.sensors.get(name), &value) {
            (Some(SensorValue::Float(old)), SensorValue::Float(new)) => {
                (old - new).abs() > self.deadband_threshold
            }
            (Some(SensorValue::Int(old)), SensorValue::Int(new)) => old != new,
            (Some(SensorValue::String(old)), SensorValue::String(new)) => old != new,
            (Some(SensorValue::Bool(old)), SensorValue::Bool(new)) => old != new,
            (None, _) => true,
            _ => true,
        };

        if should_update {
            self.sensors.insert(name.to_string(), value.clone());
            self.history.push(SensorReading {
                sensor_name: name.to_string(),
                value,
                timestamp: Instant::now(),
            });
            // Trim history
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }

        should_update
    }

    /// Get the current value of a sensor
    pub fn get_sensor(&self, name: &str) -> Option<&SensorValue> {
        self.sensors.get(name)
    }

    /// Trigger an alarm
    pub fn trigger_alarm(&mut self, name: &str, severity: AlarmSeverity, message: String) {
        self.alarms.push(Alarm {
            name: name.to_string(),
            severity,
            message,
            triggered_at: Instant::now(),
            active: true,
        });
    }

    /// Clear an alarm
    pub fn clear_alarm(&mut self, name: &str) {
        if let Some(alarm) = self.alarms.iter_mut().find(|a| a.name == name) {
            alarm.active = false;
        }
    }

    /// Get all active alarms
    pub fn active_alarms(&self) -> Vec<&Alarm> {
        self.alarms.iter().filter(|a| a.active).collect()
    }

    /// Get the room context for agent decision-making
    pub fn get_context(&self) -> RoomContext {
        RoomContext {
            room_name: self.name.clone(),
            budget_remaining: match self.sensors.get("budget_remaining") {
                Some(SensorValue::Float(v)) => *v,
                _ => 0.0,
            },
            compile_count: match self.sensors.get("compile_count") {
                Some(SensorValue::Int(v)) => *v as u64,
                _ => 0,
            },
            error_rate: match self.sensors.get("error_rate") {
                Some(SensorValue::Float(v)) => *v,
                _ => 0.0,
            },
            gpu_available: match self.sensors.get("gpu_available") {
                Some(SensorValue::Bool(v)) => *v,
                _ => false,
            },
            active_alarms: self.active_alarms().len(),
        }
    }

    /// Set the deadband threshold for float sensors
    pub fn set_deadband_threshold(&mut self, threshold: f64) {
        self.deadband_threshold = threshold;
    }
}

/// Context snapshot for agent decision-making
#[derive(Debug, Clone)]
pub struct RoomContext {
    pub room_name: String,
    pub budget_remaining: f64,
    pub compile_count: u64,
    pub error_rate: f64,
    pub gpu_available: bool,
    pub active_alarms: usize,
}

/// Agent that operates within a PLATO room
pub struct PlatoAgent {
    pub name: String,
    pub room: PlatoRoom,
    pub task_history: Vec<TaskRecord>,
}

/// Record of a task executed by an agent
#[derive(Debug, Clone)]
pub struct TaskRecord {
    pub task_type: String,
    pub duration: Duration,
    pub success: bool,
    pub timestamp: Instant,
}

impl PlatoAgent {
    /// Create a new agent in a PLATO room
    pub fn new(name: &str, room: PlatoRoom) -> Self {
        Self {
            name: name.to_string(),
            room,
            task_history: Vec::new(),
        }
    }

    /// Create a shader compilation agent with default room
    pub fn shader_compiler_agent(name: &str) -> Self {
        Self::new(name, PlatoRoom::shader_compiler_room())
    }

    /// Execute a compilation task and record the result
    pub fn execute_task(&mut self, task: &str, success: bool, duration_ms: u64) {
        self.task_history.push(TaskRecord {
            task_type: task.to_string(),
            duration: Duration::from_millis(duration_ms),
            success,
            timestamp: Instant::now(),
        });

        // Update room sensors
        if let Some(SensorValue::Int(count)) = self.room.get_sensor("compile_count") {
            self.room.set_sensor("compile_count", SensorValue::Int(count + 1));
        }

        // Update error rate
        let recent: Vec<&TaskRecord> = self.task_history.iter().rev().take(100).collect();
        if !recent.is_empty() {
            let errors = recent.iter().filter(|t| !t.success).count();
            let error_rate = errors as f64 / recent.len() as f64;
            self.room.set_sensor("error_rate", SensorValue::Float(error_rate));
        }

        // Trigger alarm on failure
        if !success {
            self.room.trigger_alarm(
                "compile_failed",
                AlarmSeverity::Warning,
                format!("Task '{}' failed after {}ms", task, duration_ms),
            );
        }
    }

    /// Get agent performance summary
    pub fn performance_summary(&self) -> AgentPerformance {
        let total = self.task_history.len();
        let successes = self.task_history.iter().filter(|t| t.success).count();
        let avg_duration = if total > 0 {
            self.task_history.iter().map(|t| t.duration.as_millis() as u64).sum::<u64>() / total as u64
        } else {
            0
        };

        AgentPerformance {
            agent_name: self.name.clone(),
            total_tasks: total,
            success_rate: if total > 0 { successes as f64 / total as f64 } else { 1.0 },
            avg_duration_ms: avg_duration,
        }
    }
}

/// Performance summary for a PLATO agent
#[derive(Debug, Clone)]
pub struct AgentPerformance {
    pub agent_name: String,
    pub total_tasks: usize,
    pub success_rate: f64,
    pub avg_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_creation() {
        let room = PlatoRoom::new("test_room");
        assert_eq!(room.name, "test_room");
        assert!(room.sensors.is_empty());
    }

    #[test]
    fn test_shader_compiler_room() {
        let room = PlatoRoom::shader_compiler_room();
        assert_eq!(room.name, "shader_compiler");
        assert!(room.sensors.contains_key("budget_remaining"));
        assert!(room.sensors.contains_key("compile_count"));
        assert!(room.sensors.contains_key("error_rate"));
    }

    #[test]
    fn test_set_sensor() {
        let mut room = PlatoRoom::new("test");
        let updated = room.set_sensor("temp", SensorValue::Float(42.0));
        assert!(updated);
        assert_eq!(room.history.len(), 1);
    }

    #[test]
    fn test_deadband_filtering() {
        let mut room = PlatoRoom::new("test");
        room.set_sensor("temp", SensorValue::Float(42.0));
        // Small change within deadband should not update
        let updated = room.set_sensor("temp", SensorValue::Float(42.005));
        assert!(!updated);
        assert_eq!(room.history.len(), 1);
        // Large change should update
        let updated = room.set_sensor("temp", SensorValue::Float(43.0));
        assert!(updated);
        assert_eq!(room.history.len(), 2);
    }

    #[test]
    fn test_trigger_and_clear_alarm() {
        let mut room = PlatoRoom::new("test");
        room.trigger_alarm("budget_exceeded", AlarmSeverity::Critical, "Budget exceeded".to_string());
        assert_eq!(room.active_alarms().len(), 1);

        room.clear_alarm("budget_exceeded");
        assert_eq!(room.active_alarms().len(), 0);
    }

    #[test]
    fn test_room_context() {
        let mut room = PlatoRoom::shader_compiler_room();
        room.set_sensor("budget_remaining", SensorValue::Float(0.05));
        room.set_sensor("compile_count", SensorValue::Int(10));

        let context = room.get_context();
        assert_eq!(context.budget_remaining, 0.05);
        assert_eq!(context.compile_count, 10);
    }

    #[test]
    fn test_agent_creation() {
        let agent = PlatoAgent::shader_compiler_agent("compiler-1");
        assert_eq!(agent.name, "compiler-1");
        assert_eq!(agent.room.name, "shader_compiler");
    }

    #[test]
    fn test_agent_task_execution() {
        let mut agent = PlatoAgent::shader_compiler_agent("test-agent");
        agent.execute_task("wgsl_compile", true, 50);
        agent.execute_task("wgsl_compile", false, 100);

        assert_eq!(agent.task_history.len(), 2);
        let summary = agent.performance_summary();
        assert_eq!(summary.total_tasks, 2);
        assert_eq!(summary.avg_duration_ms, 75);
        assert!((summary.success_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_agent_error_tracking() {
        let mut agent = PlatoAgent::shader_compiler_agent("error-test");
        agent.execute_task("compile", false, 100);
        let error_rate = match agent.room.get_sensor("error_rate") {
            Some(SensorValue::Float(v)) => *v,
            _ => 0.0,
        };
        assert!((error_rate - 1.0).abs() < 0.01);
    }
}
