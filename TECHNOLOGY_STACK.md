# 🚨 CRITICAL TECHNOLOGY STACK - READ BEFORE ANY CHANGES

## ⚠️ FRAMEWORK DECISION - ABSOLUTELY NO EXCEPTIONS

**THIS PROJECT USES BEVY + BEVY_EGUI ONLY**

### ❌ FORBIDDEN (INSTANT FAILURE)
- **NEVER USE eframe** - This will completely break the application
- **NEVER USE eframe::egui** - Incompatible with our Bevy architecture
- **NEVER REFERENCE src/gui.rs** - This is eframe-based legacy code

### ✅ MANDATORY (REQUIRED)
- **Framework**: Bevy 0.15 + bevy_egui 0.32
- **Main Entry**: src/bevy_app.rs::run_app()
- **UI Context**: bevy_egui::EguiContexts
- **Window Management**: Bevy WindowPlugin

## 📋 CORRECT IMPORTS ONLY

```rust
// ✅ CORRECT - USE THESE ONLY
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

// ❌ WRONG - NEVER USE THESE
use eframe::egui;  // THIS WILL BREAK EVERYTHING
```

## 🎯 MAIN APPLICATION STRUCTURE

```rust
// Entry point in src/main.rs
#[cfg(feature = "gui")]
mod bevy_app;

// In main() function:
bevy_app::run_app();  // ✅ CORRECT
// gui::run_gui();     // ❌ WRONG - EFAME BASED
```

## 🏗️ BEVY APP ARCHITECTURE

```rust
// src/bevy_app.rs - CORRECT STRUCTURE
pub fn run_app() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            // Bevy window configuration
        }))
        .add_plugins(EguiPlugin::default())  // ✅ bevy_egui
        .add_systems(EguiPrimaryContextPass, editor_menu)
        .run();
}
```

## 🔍 VERIFICATION COMMANDS

```bash
# Before ANY changes - MANDATORY CHECK
grep -r "eframe" src/ --include="*.rs"
# If ANY results found - PURGE IMMEDIATELY

# Check correct imports
grep -r "bevy_egui" src/ --include="*.rs"
# Should show bevy_egui imports in working files
```

## 🚨 PUNISHMENT FOR VIOLATIONS

- **Using eframe**: INSTANT REVERT + DOCUMENTATION UPDATE REQUIRED
- **Wrong imports**: COMPLETE RESTART FROM GIT BACKUP
- **No verification**: FULL CODE REVIEW BEFORE PROCEEDING

## 📚 WORKING MODULES

### ✅ BEVY-COMPATIBLE MODULES
- `src/bevy_app.rs` - Main Bevy application
- `src/editor_ui.rs` - bevy_egui UI functions  
- `src/simple_ui_auditor.rs` - UI auditing system
- `src/timeline.rs` - Timeline system
- `src/node_graph.rs` - Node graph system

### ❌ EFAME-ONLY MODULES (DO NOT USE)
- `src/gui.rs` - Legacy eframe implementation
- `src/ui.rs` - eframe-based UI

## 🎯 RUNNING THE APPLICATION

```bash
# ✅ CORRECT WAY
cargo run --bin isf-shaders

# This uses bevy_app::run_app() automatically
```

## 📋 MANDATORY CHECKLIST BEFORE ANY CHANGES

1. **Search for eframe**: `grep -r "eframe" src/`
2. **Verify bevy imports**: Check for `bevy::prelude::*`
3. **Confirm bevy_egui**: Check for `bevy_egui::` imports
4. **Test compilation**: `cargo check --features gui`
5. **Verify main entry**: Confirm `bevy_app::run_app()` is called

---

**VIOLATION OF THIS TECHNOLOGY STACK WILL RESULT IN COMPLETE APPLICATION FAILURE**