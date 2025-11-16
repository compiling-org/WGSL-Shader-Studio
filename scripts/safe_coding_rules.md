# STRICT DISCIPLINARY CODING RULES - ZERO DESTRUCTION POLICY

## ABSOLUTE PROHIBITIONS

### 1. NO CODE DELETIONS
- **NEVER delete entire files or functions**
- **NEVER comment out large blocks of code**
- **NEVER remove existing functionality**
- **ONLY fix syntax errors with minimal changes**

### 2. NO REWRITES FROM SCRATCH
- **NEVER rewrite entire modules**
- **NEVER replace working code with new implementations**
- **NEVER abandon existing functionality**
- **ONLY restore what was broken**

### 3. NO FRAMEWORK CHANGES
- **NEVER change from Bevy to eframe or vice versa**
- **NEVER modify core architecture**
- **NEVER change dependency versions without explicit approval**

## MANDATORY RESTORATION PROCESS

### Step 1: Identify Exact Error
- Run `cargo check` to see specific compilation errors
- Identify line numbers and exact error messages
- **NO GUESSING - ONLY fix what compiler reports**

### Step 2: Minimal Syntax Fixes
- Fix missing imports by adding them
- Fix typos by correcting spelling
- Fix missing fields by adding them
- **NEVER remove code that "might" be wrong**

### Step 3: Preserve All Existing Code
- Comment out only the specific line causing error
- Add minimal fix to make it compile
- **KEEP ALL ORIGINAL LOGIC INTACT**

### Step 4: Verify Each Fix
- Run `cargo check` after each minimal change
- Ensure no new errors introduced
- **STOP if any functionality is lost**

## EMERGENCY PROTOCOLS

### If Compilation Fails
1. **READ EXACT ERROR MESSAGE**
2. **FIX ONLY THAT SPECIFIC ERROR**
3. **RUN CARGO CHECK AGAIN**
4. **REPEAT UNTIL CLEAN BUILD**

### If Functionality Missing
1. **CHECK ORIGINAL DOCUMENTATION**
2. **RESTORE FROM BACKUP IF AVAILABLE**
3. **NEVER REIMPLEMENT FROM SCRATCH**
4. **PRESERVE ALL WORKING CODE**

## ZERO TOLERANCE VIOLATIONS

### Automatic Failure Conditions
- Any file deletion = **IMMEDIATE STOP**
- Any function removal = **IMMEDIATE STOP**
- Any framework change = **IMMEDIATE STOP**
- Any rewrite = **IMMEDIATE STOP**

### Required Actions
- **BACKUP CURRENT STATE** before any changes
- **DOCUMENT EVERY CHANGE** with reason
- **VERIFY NO FUNCTIONALITY LOST** after each fix
- **STOP AT FIRST SIGN OF DESTRUCTION**

## CURRENT MISSION: RESTORE LOST FEATURES

### Priority Order (NO DEVIATIONS)
1. **Fix compilation errors** (syntax only)
2. **Restore converter functionality** (HLSL/GLSL/ISF)
3. **Restore shader files** (preserve all existing)
4. **Restore UI functionality** (fix rendering only)
5. **Restore export/save** (minimal fixes only)

### ABSOLUTE RULES FOR THIS SESSION
- **NO NEW FEATURES**
- **NO REWRITES**
- **NO DELETIONS**
- **ONLY RESTORATION**
- **MINIMAL CHANGES ONLY**

**VIOLATION OF THESE RULES = IMMEDIATE TERMINATION OF WORK**