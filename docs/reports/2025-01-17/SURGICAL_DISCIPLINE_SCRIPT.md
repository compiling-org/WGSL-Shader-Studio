# SURGICAL DISCIPLINE PROTOCOL

## IMMEDIATE MANDATORY ACTIONS - NO DEVIATION ALLOWED

### Step 1: VERIFY CURRENT STATE (DO NOT SKIP)
- [ ] Run `cargo check` and record exact error count
- [ ] List all files that need Bevy conversion
- [ ] Confirm which modules are missing/unresolved

### Step 2: FIX MODULE IMPORTS SURGICALLY
- [ ] Fix editor_ui.rs bevy imports ONE LINE AT A TIME
- [ ] Fix simple_ui_auditor.rs bevy imports ONE LINE AT A TIME  
- [ ] Fix timeline.rs bevy imports ONE LINE AT A TIME
- [ ] TEST AFTER EACH SINGLE FILE FIX

### Step 3: NO CREATIVE INTERPRETATION
- [ ] Only fix exactly what the compiler error shows
- [ ] Do NOT add features not already in the code
- [ ] Do NOT change working logic, only imports

### Step 4: TEST AFTER EACH MICRO-CHANGE
- [ ] Run `cargo check` after every single edit
- [ ] If error count increases, REVERT immediately
- [ ] Only proceed when error count decreases

### FORBIDDEN ACTIONS (INSTANT FAILURE)
- [ ] NO adding new dependencies without testing
- [ ] NO changing feature flags without understanding
- [ ] NO modifying working code logic
- [ ] NO batch changes - ONE LINE ONLY

### MANDATORY VERIFICATION SCRIPT
```bash
# After each single change, MUST run:
cargo check --features gui 2>&1 | head -20
echo "ERROR COUNT: $(cargo check --features gui 2>&1 | grep -c error)"
```

### EMERGENCY REVERT PROTOCOL
If error count increases at any point:
```bash
git checkout HEAD -- src/file_that_was_modified.rs
```

## CURRENT STATUS:
- bevy = "0.15" ✓ (correct, not optional)
- bevy_egui = { version = "0.32", optional = true } ✓ (correct for gui feature)
- gui = ["dep:bevy_egui"] ✓ (correct)

## NEXT SURGICAL STEPS:
1. Fix editor_ui.rs line 1: `use bevy::prelude::*;`
2. Fix editor_ui.rs line 2: `use bevy_egui::{egui, EguiContexts};`
3. Fix simple_ui_auditor.rs line 1: `use bevy::prelude::*;`
4. Continue ONE LINE AT A TIME until compilation succeeds

**NO DEVIATION FROM THIS PROTOCOL ALLOWED**