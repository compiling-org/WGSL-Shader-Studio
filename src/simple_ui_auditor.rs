use bevy::prelude::*;
use bevy_egui::EguiContexts;
use std::collections::HashMap;
use serde::Serialize;
use bevy::input::keyboard::KeyCode;

/// Simple UI auditor that tracks what's actually rendered vs placeholders
#[derive(Resource)]
pub struct SimpleUiAuditor {
    pub enabled: bool,
    pub panels_found: HashMap<String, PanelInfo>,
    pub events: Vec<String>, // Circular buffer of last N events
    pub input_stats: InputStats,
    pub last_save_time: f64,
}

#[derive(Default, Clone, Serialize)]
pub struct InputStats {
    pub mouse_pos: Option<Vec2>,
    pub any_button_hovered: bool,
    pub any_button_clicked: bool,
    pub primary_clicked: bool,
    pub secondary_clicked: bool,
    pub middle_clicked: bool,
    pub keys_pressed: Vec<String>,
    pub interactions: usize,
}

impl Default for SimpleUiAuditor {
    fn default() -> Self {
        Self {
            enabled: true,
            panels_found: HashMap::new(),
            events: Vec::new(),
            input_stats: InputStats::default(),
            last_save_time: 0.0,
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct PanelInfo {
    pub has_real_content: bool,
    pub widget_count: usize,
    pub placeholder_reasons: Vec<String>,
}

#[derive(Serialize)]
pub struct AuditReport {
    pub timestamp: u64,
    pub panel_count: usize,
    pub panels: HashMap<String, PanelInfo>,
    pub events: Vec<String>,
    pub input_stats: InputStats,
}

impl SimpleUiAuditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_panel(&mut self, name: &str, has_content: bool, reason: Option<String>) {
        let info = self.panels_found.entry(name.to_string()).or_default();
        info.has_real_content = has_content;
        
        if let Some(r) = reason {
            if !info.placeholder_reasons.contains(&r) {
                info.placeholder_reasons.push(r);
            }
        }
    }
    
    pub fn log_event(&mut self, event: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() % 10000;
            
        let entry = format!("[{}] {}", timestamp, event);
        if self.events.len() >= 50 { // Increased buffer size
            self.events.remove(0);
        }
        self.events.push(entry);
    }
    
    pub fn update_input_stats(&mut self, ctx: &bevy_egui::egui::Context, keys: &ButtonInput<KeyCode>) {
        // Safe readout of input state
        ctx.input(|i| {
           self.input_stats.mouse_pos = i.pointer.latest_pos().map(|p| Vec2::new(p.x, p.y));
           self.input_stats.any_button_hovered = i.pointer.any_down(); 
           
           self.input_stats.primary_clicked = i.pointer.primary_down();
           self.input_stats.secondary_clicked = i.pointer.secondary_down();
           self.input_stats.middle_clicked = i.pointer.middle_down();
           
           if i.pointer.primary_clicked() {
               self.log_event(format!("Global Input: Left Click at {:?}", i.pointer.interact_pos()));
               self.input_stats.interactions += 1;
               self.input_stats.any_button_clicked = true;
           }
           if i.pointer.secondary_clicked() {
               self.log_event("Global Input: Right Click".to_string());
               self.input_stats.any_button_clicked = true;
           }
        });
        
        // Track pressed keys
        self.input_stats.keys_pressed = keys.get_pressed()
            .take(5) // Limit to 5 simultaneous keys to avoid spam
            .map(|k| format!("{:?}", k))
            .collect();
    }

    pub fn save_report(&self) {
        let report = AuditReport {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            panel_count: self.panels_found.len(),
            panels: self.panels_found.clone(),
            events: self.events.clone(),
            input_stats: self.input_stats.clone(),
        };

        if let Ok(json_str) = serde_json::to_string_pretty(&report) {
           // Use a temp file and rename to avoid read/write collisions
           let _ = std::fs::write("ui_audit.json.tmp", &json_str);
           let _ = std::fs::rename("ui_audit.json.tmp", "ui_audit.json");
        }
    }
    
    pub fn clear(&mut self) {
        self.panels_found.clear();
    }
}

/// System that listens for F12 and prints audit report
pub fn ui_audit_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut auditor: ResMut<SimpleUiAuditor>,
    time: Res<Time>,
    mut egui_ctx: EguiContexts,
) {
    // Capture Input Stats every frame
    let ctx = match egui_ctx.ctx_mut() {
        Ok(c) => c,
        Err(_) => return, // Skip if egui context not available
    };
    
    // We need to clone keys or pass reference. Since update_input_stats needs specific types,
    // let's just pass the resource reference.
    auditor.update_input_stats(ctx, &keys);

    // High frequency updates (10Hz / every 0.1s) for live tracking feel
    let now = time.elapsed_seconds_f64();
    if now - auditor.last_save_time > 0.1 {
         auditor.save_report();
         auditor.last_save_time = now;
    }
    
    // Also save on F12
    if keys.just_pressed(KeyCode::F12) {
        println!("\n=== EVENTS LOG ===");
        for e in &auditor.events {
            println!("{}", e);
        }
        println!("==================\n");
        auditor.save_report();
    }
}

/// Plugin to add the simple auditor
pub struct SimpleUiAuditorPlugin;

impl Plugin for SimpleUiAuditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimpleUiAuditor>()
            .add_systems(Update, ui_audit_system);
    }
}
