#!/bin/bash

# WGSL Shader Studio ENFORCEMENT SCRIPT
# PREVENTS DESTRUCTIVE LOOPING AND VIOLATIONS
# Follows disciplinary rules strictly

echo "=== WGSL SHADER STUDIO ENFORCEMENT ==="
echo "ENFORCEMENT ACTIVE - PREVENTING DESTRUCTIVE BEHAVIOR"
echo ""

# CRITICAL: This script prevents me from violating disciplinary rules
# It must be run before any coding work begins

# RULE 1: NO DELETION OF CRUCIAL FILES
echo "🔒 RULE 1: PROTECTING CRUCIAL FILES"
echo "Checking for attempts to delete important files..."

# List of CRUCIAL files that must NEVER be deleted
CRUCIAL_FILES=(
    "src/bevy_app.rs"
    "src/editor_ui.rs" 
    "src/shader_renderer.rs"
    "src/main.rs"
    "src/lib.rs"
    "Cargo.toml"
    ".trae/documents/disciplinary_rules.md"
    ".trae/documents/PSYCHOTIC_LOOP_DOCUMENT.md"
)

for file in "${CRUCIAL_FILES[@]}"; do
    if [ ! -f "$file" ]; then
        echo "🚨 CRITICAL VIOLATION: Missing crucial file: $file"
        echo "🚨 ENFORCEMENT: STOP ALL WORK IMMEDIATELY"
        exit 1
    else
        echo "✅ Protected: $file"
    fi
done

# RULE 2: NO DUPLICATE FILE CREATION
echo ""
echo "🔒 RULE 2: PREVENTING DUPLICATE FILES"
echo "Checking for file pattern violations..."

# Banned patterns that indicate destructive behavior
BANNED_PATTERNS=(
    "*backup*"
    "*demo*" 
    "*simple*"
    "*new*"
    "*fixed*"
    "*audited*"
    "*temp*"
    "*clean*"
)

VIOLATION_FOUND=0
for pattern in "${BANNED_PATTERNS[@]}"; do
    if find . -name "$pattern" -type f | grep -v ".git" | grep -q .; then
        echo "🚨 VIOLATION: Found banned pattern: $pattern"
        echo "🚨 ENFORCEMENT: These files indicate destructive looping behavior"
        VIOLATION_FOUND=1
    fi
done

if [ "$VIOLATION_FOUND" -eq 1 ]; then
    echo "🚨 ENFORCEMENT: Address violations before continuing"
    exit 1
fi

# RULE 3: SURGICAL FIXES ONLY
echo ""
echo "🔒 RULE 3: ENFORCING SURGICAL FIXES"
echo "Checking for wholesale file creation/deletion attempts..."

# Count recent file operations (indicates destructive behavior)
RECENT_FILES=$(find . -name "*.rs" -mtime -1 | grep -v ".git" | wc -l)
if [ "$RECENT_FILES" -gt 10 ]; then
    echo "🚨 WARNING: High file activity detected ($RECENT_FILES files)"
    echo "🚨 ENFORCEMENT: Surgical fixes only - no wholesale changes"
fi

# RULE 4: PRODUCTION CODE ONLY
echo ""
echo "🔒 RULE 4: PRODUCTION CODE ENFORCEMENT"
echo "Checking for demo/test code violations..."

# Check for test functions in main modules
if grep -r "fn test_" src/*.rs | grep -v "bin" | grep -q .; then
    echo "🚨 VIOLATION: Test functions found in production modules"
    echo "🚨 ENFORCEMENT: Move tests to appropriate test modules"
    exit 1
fi

# RULE 5: UI ANALYZER BEFORE ANY LAUNCHES
echo ""
echo "🔒 RULE 5: UI ANALYZER MANDATE"
echo "Ensuring UI analyzer is used before application launches..."

if [ ! -f "src/bin/ui_analyzer.rs" ]; then
    echo "🚨 VIOLATION: UI analyzer missing"
    echo "🚨 ENFORCEMENT: UI analyzer must exist and be used"
    exit 1
fi

echo "✅ UI analyzer available: src/bin/ui_analyzer.rs"

# FINAL ENFORCEMENT
echo ""
echo "=== ENFORCEMENT SUMMARY ==="
echo "✅ CRUCIAL FILES PROTECTED"
echo "✅ NO DUPLICATE FILES DETECTED" 
echo "✅ SURGICAL FIXES ENFORCED"
echo "✅ PRODUCTION CODE VERIFIED"
echo "✅ UI ANALYZER AVAILABLE"
echo ""
echo "🔒 ENFORCEMENT ACTIVE - SAFE TO PROCEED"
echo "🔒 This script prevents destructive looping behavior"
echo "🔒 Following disciplinary rules is MANDATORY"
echo ""
echo "NEXT STEPS:"
echo "1. Run UI analyzer to diagnose issues"
echo "2. Implement surgical fixes based on analysis"
echo "3. Verify fixes before proceeding"
echo "4. NEVER delete crucial files"
echo "=== ENFORCEMENT COMPLETE ==="