#!/bin/bash

# MODULAR ENFORCEMENT SCRIPT - DIFFERENT MODES FOR DIFFERENT SITUATIONS
# Modes: STRICT (default), SURGICAL (syntax fixes), LENIENT (small changes)

set -euo pipefail

# Default mode
ENFORCEMENT_MODE="${1:-STRICT}"

echo "=== ENFORCEMENT SYSTEM: $ENFORCEMENT_MODE MODE ==="

# Mode-specific behavior
case "$ENFORCEMENT_MODE" in
    "STRICT")
        echo "🚨 MAXIMUM PROTECTION - NO FILE DELETIONS, NO LARGE CHANGES"
        MAX_LINES=20
        ALLOW_DELETIONS=false
        ;;
    "SURGICAL")
        echo "🔧 SURGICAL MODE - SYNTAX FIXES ALLOWED, SMALL CHANGES PERMITTED"
        MAX_LINES=10
        ALLOW_DELETIONS=false
        ;;
    "LENIENT")
        echo "✅ LENIENT MODE - MODERATE CHANGES ALLOWED, FILE DELETIONS RESTRICTED"
        MAX_LINES=50
        ALLOW_DELETIONS=false
        ;;
    *)
        echo "❌ UNKNOWN MODE: $ENFORCEMENT_MODE"
        echo "Available modes: STRICT, SURGICAL, LENIENT"
        exit 1
        ;;
esac

# Function to check for destructive patterns (mode-aware)
check_destructive_patterns() {
    local file="$1"
    local action="$2"
    
    # Check for file deletions (always restricted)
    if [[ "$action" == "delete" && "$ALLOW_DELETIONS" == "false" ]]; then
        local file_size=$(wc -c < "$file" 2>/dev/null || echo "0")
        if [[ $file_size -gt 1000 ]]; then
            echo "❌ LARGE FILE DELETION DETECTED: $file ($file_size bytes)"
            echo "❌ Manual approval required for large file deletions"
            exit 1
        else
            echo "✅ Small file deletion allowed: $file"
        fi
    fi
    
    # In surgical mode, allow syntax fixes
    if [[ "$ENFORCEMENT_MODE" == "SURGICAL" && "$action" == "modify" ]]; then
        echo "✅ Surgical modification allowed: $file"
        return 0
    fi
    
    # Check for large comment blocks (potential code removal)
    if grep -q "\/\*.*\*\/" "$file" 2>/dev/null; then
        echo "⚠️  Large comment block detected: $file"
        echo "⚠️  Review recommended but not blocking"
    fi
    
    # Check for function removals (only in strict mode)
    if [[ "$ENFORCEMENT_MODE" == "STRICT" && "$action" == "modify" ]]; then
        # Count functions before and after
        local before_count=$(grep -c "^fn " "$file.bak" 2>/dev/null || echo "0")
        local after_count=$(grep -c "^fn " "$file" 2>/dev/null || echo "0")
        
        if [[ $after_count -lt $before_count ]]; then
            echo "❌ FUNCTION REMOVAL DETECTED: $file"
            echo "❌ BEFORE: $before_count functions, AFTER: $after_count functions"
            exit 1
        fi
    fi
}

# Function to backup before changes
backup_file() {
    local file="$1"
    if [[ -f "$file" ]]; then
        cp "$file" "$file.bak"
        echo "✅ BACKUP CREATED: $file.bak"
    fi
}

# Function to enable surgical editing
enable_surgical_edit() {
    local file="$1"
    echo "🔧 ENABLING SURGICAL EDIT: $file"
    
    # Backup first
    backup_file "$file"
    
    # Enable write permissions
    chmod +w "$file" 2>/dev/null || true
    
    echo "✅ Surgical edit mode enabled for: $file"
    echo "✅ You can now make syntax fixes and small changes"
}

# Function to enforce minimal changes (mode-aware)
enforce_minimal_changes() {
    local file="$1"
    
    # Count lines changed
    if [[ -f "$file.bak" ]]; then
        local lines_changed=$(diff -u "$file.bak" "$file" 2>/dev/null | grep -c "^[+-]" || echo "0")
        
        if [[ $lines_changed -gt $MAX_LINES ]]; then
            echo "❌ EXCESSIVE CHANGES DETECTED: $file"
            echo "❌ $lines_changed lines changed (max: $MAX_LINES for $ENFORCEMENT_MODE mode)"
            
            # In surgical mode, allow larger changes if they're syntax fixes
            if [[ "$ENFORCEMENT_MODE" == "SURGICAL" ]]; then
                echo "⚠️  Surgical mode: allowing larger changes for syntax fixes"
            else
                echo "❌ RESTORE FROM BACKUP AND TRY SMALLER CHANGES"
                cp "$file.bak" "$file"
                exit 1
            fi
        fi
    fi
}

# Create safety wrapper functions
safe_edit() {
    local file="$1"
    echo "🔍 PREPARING SAFE EDIT: $file"
    
    # Backup first
    backup_file "$file"
    
    # Create edit command
    echo "code '$file'"
}

safe_check() {
    echo "🔍 RUNNING COMPILATION CHECK"
    cargo check 2>&1 | head -20
    
    if [[ $? -ne 0 ]]; then
        echo "❌ COMPILATION ERRORS FOUND - FIX THESE ONLY:"
        return 1
    else
        echo "✅ COMPILATION CLEAN"
        return 0
    fi
}

# Export functions for use
export -f check_destructive_patterns
export -f backup_file
export -f enforce_minimal_changes
export -f enable_surgical_edit
export -f safe_edit
export -f safe_check

echo "✅ SAFETY SYSTEMS ACTIVATED IN $ENFORCEMENT_MODE MODE"
echo ""
echo "AVAILABLE COMMANDS:"
echo "  enable_surgical_edit <file>  - Enable editing for syntax fixes"
echo "  safe_edit <file>              - Safe edit with backup"
echo "  safe_check                    - Verify compilation"
echo ""
echo "MODE-SPECIFIC RULES:"
case "$ENFORCEMENT_MODE" in
    "STRICT")
        echo "🚨 NO DELETIONS, NO REWRITES, MAX 20 LINE CHANGES"
        ;;
    "SURGICAL")
        echo "🔧 SYNTAX FIXES ALLOWED, MAX 10 LINE CHANGES"
        ;;
    "LENIENT")
        echo "✅ MODERATE CHANGES ALLOWED, MAX 50 LINE CHANGES"
        ;;
esac

# Set up file monitoring
if command -v inotifywait >/dev/null 2>&1; then
    echo "🔍 FILE MONITORING ACTIVE"
    # Monitor for file deletions and large changes
    inotifywait -m -r -e delete,modify src/ 2>/dev/null | while read path action file; do
        if [[ "$action" == "DELETE" ]]; then
            echo "❌ FILE DELETION DETECTED: $path$file"
            echo "❌ STOPPING IMMEDIATELY"
            exit 1
        fi
    done &
fi

echo ""
echo "=== READY FOR SAFE CODING ==="