#!/bin/bash

# WGSL Shader Studio Session Enforcement Script
# Monitors for psychotic loops and enforces disciplinary guidelines
# Runs every few minutes, not constantly

SESSION_LOG="/tmp/wgsl_session_$(date +%Y%m%d_%H%M%S).log"
PROJECT_DIR="/c/Users/kapil/compiling/WGSL-Shader-Studio"
MAX_FILE_CHANGES=3
CHECK_INTERVAL=180  # 3 minutes

log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$SESSION_LOG"
}

check_file_modification_patterns() {
    local file="$1"
    local changes=$(git log --oneline --since="10 minutes ago" -- "$file" | wc -l)
    
    if [ "$changes" -gt "$MAX_FILE_CHANGES" ]; then
        log_message "WARNING: Excessive modifications detected in $file ($changes changes in 10 minutes)"
        return 1
    fi
    return 0
}

check_visual_node_editor_corruption() {
    if [ -f "$PROJECT_DIR/src/visual_node_editor.rs" ]; then
        # Check for common corruption patterns
        if grep -q "unclosed delimiter" "$PROJECT_DIR/src/visual_node_editor.rs"; then
            log_message "CRITICAL: visual_node_editor.rs has syntax errors - needs immediate fix"
            return 1
        fi
        
        # Check for API mismatch patterns
        if grep -q "NodeKind::Math" "$PROJECT_DIR/src/visual_node_editor.rs"; then
            log_message "WARNING: visual_node_editor.rs has API mismatches with non-existent NodeKind variants"
            return 1
        fi
    fi
    return 0
}

check_test_file_proliferation() {
    local test_count=$(find "$PROJECT_DIR" -name "*test*" -o -name "*demo*" -o -name "*placeholder*" | wc -l)
    
    if [ "$test_count" -gt 5 ]; then
        log_message "WARNING: Excessive test/demo files detected ($test_count files)"
        return 1
    fi
    return 0
}

check_reference_integration_status() {
    local has_use_gpu=$(grep -r "use\.gpu" "$PROJECT_DIR/src" 2>/dev/null | wc -l)
    local has_wgsl_analyzer=$(grep -r "wgsl-analyzer" "$PROJECT_DIR/src" 2>/dev/null | wc -l)
    
    if [ "$has_use_gpu" -eq 0 ] && [ "$has_wgsl_analyzer" -eq 0 ]; then
        log_message "INFO: Reference repository patterns not yet integrated from use.gpu/wgsl-analyzer"
    fi
}

check_compilation_status() {
    cd "$PROJECT_DIR"
    local cargo_result=$(cargo check 2>&1)
    local error_count=$(echo "$cargo_result" | grep -c "error\[" || echo "0")
    
    if [ "$error_count" -gt 0 ]; then
        log_message "COMPILATION ERRORS: $error_count errors detected"
        echo "$cargo_result" | grep "error\[" | head -5 | tee -a "$SESSION_LOG"
        return 1
    fi
    return 0
}

generate_compliance_report() {
    log_message "=== SESSION COMPLIANCE REPORT ==="
    
    # Check critical files
    local issues=0
    
    if ! check_visual_node_editor_corruption; then
        ((issues++))
    fi
    
    if ! check_test_file_proliferation; then
        ((issues++))
    fi
    
    if ! check_compilation_status; then
        ((issues++))
    fi
    
    # Check file modification patterns for critical files
    for file in "src/visual_node_editor.rs" "src/lib.rs" "src/main.rs"; do
        if [ -f "$PROJECT_DIR/$file" ]; then
            if ! check_file_modification_patterns "$PROJECT_DIR/$file"; then
                ((issues++))
            fi
        fi
    done
    
    check_reference_integration_status
    
    if [ "$issues" -eq 0 ]; then
        log_message "✅ SESSION COMPLIANT - No issues detected"
    else
        log_message "⚠️  SESSION VIOLATIONS DETECTED: $issues issues found"
        log_message "Next check in $CHECK_INTERVAL seconds"
    fi
    
    log_message "=================================="
}

# Main execution
log_message "Starting WGSL Shader Studio Session Enforcement"
log_message "Project directory: $PROJECT_DIR"
log_message "Check interval: ${CHECK_INTERVAL}s"

# Initial check
generate_compliance_report

# Periodic monitoring loop
while true; do
    sleep "$CHECK_INTERVAL"
    generate_compliance_report
done