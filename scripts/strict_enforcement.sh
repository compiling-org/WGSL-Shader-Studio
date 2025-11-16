#!/bin/bash

# STRICT ENFORCEMENT SCRIPT - PREVENTS DESTRUCTIVE ACTIONS
# This script must be run before any coding session

set -euo pipefail

echo "=== STRICT CODING ENFORCEMENT ACTIVATED ==="
echo "VIOLATION = IMMEDIATE TERMINATION"
echo ""

# Function to check for destructive patterns
check_destructive_patterns() {
    local file="$1"
    local action="$2"
    
    # Check for file deletions
    if [[ "$action" == "delete" ]]; then
        echo "❌ FILE DELETION DETECTED: $file"
        echo "❌ STOPPING IMMEDIATELY - NO FILE DELETIONS ALLOWED"
        exit 1
    fi
    
    # Check for large comment blocks (potential code removal)
    if grep -q "\/\*.*\*\/" "$file" 2>/dev/null; then
        echo "❌ LARGE COMMENT BLOCK DETECTED: $file"
        echo "❌ POTENTIAL CODE HIDING - MANUAL REVIEW REQUIRED"
        exit 1
    fi
    
    # Check for function removals
    if [[ "$action" == "modify" ]]; then
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

# Function to enforce minimal changes
enforce_minimal_changes() {
    local file="$1"
    
    # Count lines changed
    if [[ -f "$file.bak" ]]; then
        local lines_changed=$(diff -u "$file.bak" "$file" 2>/dev/null | grep -c "^[+-]" || echo "0")
        
        if [[ $lines_changed -gt 20 ]]; then
            echo "❌ EXCESSIVE CHANGES DETECTED: $file"
            echo "❌ $lines_changed lines changed (max: 20)"
            echo "❌ RESTORE FROM BACKUP AND TRY SMALLER CHANGES"
            cp "$file.bak" "$file"
            exit 1
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
export -f safe_edit
export -f safe_check

echo "✅ SAFETY SYSTEMS ACTIVATED"
echo "✅ Use 'safe_edit <file>' to edit files safely"
echo "✅ Use 'safe_check' to verify compilation"
echo "✅ All changes will be monitored"
echo ""
echo "🚨 REMEMBER: ONE ERROR FIX AT A TIME 🚨"
echo "🚨 NO DELETIONS, NO REWRITES, NO EXCESSIVE CHANGES 🚨"

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