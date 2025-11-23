# STRICT DISCIPLINE SCRIPT - PREVENT HORRENDOUS FUCKUPS

## BEFORE ANY CODE CHANGES - MANDATORY CHECKLIST

### 1. FRAMEWORK VERIFICATION (CRITICAL - INSTANT FAIL IF WRONG)
- [ ] **VERIFY**: Are we using Bevy + bevy_egui? (NOT eframe - EVER!)
- [ ] **VERIFY**: Check Cargo.toml for bevy dependencies (NOT eframe dependencies)
- [ ] **VERIFY**: Check current file imports for bevy::* (NO eframe::* ALLOWED)
- [ ] **VERIFY**: NO eframe references anywhere in active code

### 2. TECHNOLOGY STACK CONFIRMATION
```rust
// CORRECT imports (MANDATORY):
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
// WRONG imports (INSTANT FAILURE - NEVER USE):
use eframe::egui;  // ❌ NEVER EVER USE THIS - WILL BREAK EVERYTHING
```

### 3. MANDATORY SEARCH BEFORE CHANGES
```bash
# Search for eframe in ALL files before any changes:
grep -r "eframe" src/ --include="*.rs"
# If found, PURGE IMMEDIATELY before proceeding
```

### 4. BINARY VERIFICATION
- [ ] **VERIFY**: Which binary should we run? (check Cargo.toml [[bin]] sections)
- [ ] **VERIFY**: Is it `cargo run --bin isf-shaders` or different?
- [ ] **VERIFY**: Check if bevy_app.rs exists and should be used

## CURRENT PROJECT STATUS

### CORRECT TECHNOLOGY STACK (NO EXCEPTIONS):
- Framework: **Bevy 0.15** + **bevy_egui** (NOT eframe - EVER!)
- Window Management: Bevy WindowPlugin (NOT eframe windowing)
- UI Rendering: bevy_egui (NOT eframe - INCOMPATIBLE)
- Main Application: src/bevy_app.rs (NOT src/gui.rs which is eframe)

### CORRECT BINARY (NO EXCEPTIONS):
- Primary: `cargo run --bin isf-shaders` (MUST use bevy_app.rs, NOT eframe)
- Main Entry: src/bevy_app.rs::run_app() (NOT gui::run_gui())

### IMMEDIATE ACTIONS REQUIRED (CRITICAL):
1. **PURGE ALL eframe references from src/gui.rs** (DELETE gui.rs if needed)
2. **Convert main.rs to use bevy_app::run_app() NOT gui::run_gui()**
3. **NEVER use eframe::egui imports - INSTANT FAILURE**
3. **Verify bevy_app.rs is the correct implementation**

## PUNISHMENT FOR VIOLATIONS
- Adding eframe: **INSTANT REVERT + APOLOGY**
- Wrong framework: **COMPLETE RESTART FROM BACKUP**
- No verification: **FULL CODE REVIEW BEFORE PROCEEDING**

## PROJECT RULES - CRITICAL FILE MANAGEMENT

### 1. File Management (ZERO TOLERANCE)
- **NEVER create duplicate files** - Always update existing files instead
- **No multiple versions** of the same file (e.g., visual_node_editor.rs, visual_node_editor_new.rs, etc.)
- Check for existing files before creating new ones
- Use existing files as templates for modifications

### 2. Feature Implementation Assessment (HONESTY REQUIRED)
- Provide honest assessment of what's actually implemented vs claimed
- Document actual functionality, not planned features
- Test compilation before claiming features work
- Reference repository code must be properly integrated

## EMERGENCY RESTORE POINTS
- Git commit before any framework changes
- Backup of working bevy implementation
- Original UI audit report as reference