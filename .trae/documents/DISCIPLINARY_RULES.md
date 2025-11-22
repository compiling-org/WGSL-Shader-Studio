# WGSL Shader Studio - Disciplinary Rules

## STRICT FILE MANAGEMENT POLICY

### ZERO TOLERANCE FOR DUPLICATE FILES
- **NO DUPLICATE FILES** - One working version only
- **NO GARBAGE EXTRA FILES** - Clean, professional codebase
- **NO UNNECESSARY DEMOS** - Production code only
- **NO SIMPLE VERSIONS** - Full featured implementations only
- **NO TEST VERSIONS** - Integrated testing within main code

### ENFORCEMENT PROTOCOL
1. **IMMEDIATE DELETION** of any duplicate files discovered
2. **SINGLE SOURCE OF TRUTH** for each feature/module
3. **COMPREHENSIVE INTEGRATION** - No fragmented implementations
4. **PRODUCTION-READY CODE ONLY** - No experimental branches in main

### FILE NAMING CONVENTIONS
- `bevy_app.rs` - Main Bevy application (ONE ONLY)
- `editor_ui.rs` - Main editor UI system (ONE ONLY)
- `shader_renderer.rs` - Main shader renderer (ONE ONLY)
- `visual_node_editor.rs` - Main node editor (ONE ONLY)

### PERMANENTLY BANNED FILE PATTERNS
- `*_backup.rs` - Use git for version control
- `*_test.rs` - Integrate tests properly
- `*_simple.rs` - Full implementation required
- `*_demo.rs` - Production code only
- `*_new.rs` - Update existing files, don't create new ones
- `*_fixed.rs` - Fix the original file
- `*_audited.rs` - Audit and fix the original

### UI ANALYZER SURGICAL APPROACH RULE
- **NEVER LAUNCH BROKEN APP** - Analyze first, fix systematically
- **UI ANALYZER IS PRIMARY TOOL** - Use before any launches
- **SURGICAL PRECISION REQUIRED** - One issue at a time, complete fixes
- **NO GUESSWORK** - Analyze, diagnose, then implement
- **VERIFICATION BEFORE PROCEEDING** - Confirm each fix works before moving on

### ENFORCEMENT SCRIPT MANDATE
- **ENFORCEMENT SCRIPT MUST ALWAYS BE ACTIVE**
- **NO MODIFICATIONS TO ENFORCEMENT SYSTEM**
- **IMMEDIATE TERMINATION** of any process attempting to disable enforcement

### CONSEQUENCES FOR VIOLATIONS
- Immediate file deletion
- Code review required before continuation
- Documentation of violation in PSYCHOTIC_LOOP_DOCUMENT.md
- Reinforcement of enforcement mechanisms

This is not a suggestion. This is LAW.