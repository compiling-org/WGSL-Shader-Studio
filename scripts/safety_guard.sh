#!/bin/bash

# STRICT SAFETY GUARD SCRIPT FOR WGSL SHADER STUDIO
# Prevents destructive actions and extensive code deletions

set -euo pipefail

# Configuration
MAX_DELETIONS=10
MAX_FILE_SIZE_REDUCTION=50  # percentage
BACKUP_DIR=".safety_backups"
LOG_FILE=".safety_log.txt"

# Initialize logging
log_action() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Function to backup file before modification
backup_file() {
    local file="$1"
    if [[ -f "$file" ]]; then
        local backup_name="$(basename "$file").$(date +%s).backup"
        cp "$file" "$BACKUP_DIR/$backup_name"
        log_action "Backed up $file to $backup_name"
    fi
}

# Function to check file size changes
check_file_size() {
    local file="$1"
    local original_size="$2"
    local new_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
    
    if [[ "$original_size" -gt 0 && "$new_size" -lt "$((original_size * (100 - MAX_FILE_SIZE_REDUCTION) / 100))" ]]; then
        echo "ERROR: File $file size reduced by more than $MAX_FILE_SIZE_REDUCTION%