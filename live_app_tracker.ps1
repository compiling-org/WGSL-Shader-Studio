# COMPREHENSIVE UI ANALYZER & APPLICATION TRACKER - CONTINUOUS MONITORING
# Tracks application compilation, runtime errors, system health, and UI component analysis

param(
    [int]$CheckInterval = 5,  # Check every 5 seconds
    [string]$LogFile = "live_app_tracking.log",
    [string]$ErrorLog = "live_errors.log"
)

# Ensure log files exist
New-Item -ItemType File -Path $LogFile -Force | Out-Null
New-Item -ItemType File -Path $ErrorLog -Force | Out-Null

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Write-Host $logEntry
    Add-Content -Path $LogFile -Value $logEntry
}

function Check-Compilation {
    Write-Log "Checking compilation status..." "CHECK"
    
    # Run cargo check and capture output
    $output = cargo check 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Log "Compilation SUCCESSFUL" "SUCCESS"
        return $true
    } else {
        Write-Log "Compilation FAILED with $exitCode errors" "ERROR"
        
        # Extract and log specific errors
        $errorLines = $output | Where-Object { $_ -match "error\[" }
        foreach ($error in $errorLines) {
            Write-Log "Error: $error" "ERROR_DETAIL"
            Add-Content -Path $ErrorLog -Value "$(Get-Date -Format 'HH:mm:ss') - $error"
        }
        
        return $false
    }
}

function Check-ApplicationRuntime {
    Write-Log "Checking application runtime status..." "CHECK"
    
    # Check if the application binary exists
    if (Test-Path "target\release\isf-shaders.exe") {
        Write-Log "Application binary EXISTS" "SUCCESS"
        return $true
    } elseif (Test-Path "target\debug\isf-shaders.exe") {
        Write-Log "Application binary exists in DEBUG mode" "WARNING"
        return $true
    } else {
        Write-Log "Application binary NOT FOUND" "ERROR"
        return $false
    }
}

function Check-SystemHealth {
    Write-Log "Checking system health..." "CHECK"
    
    # Check memory usage
    try {
        $memory = Get-WmiObject -Class Win32_OperatingSystem
        $freeMemory = [math]::Round($memory.FreePhysicalMemory / 1MB, 2)
        $totalMemory = [math]::Round($memory.TotalPhysicalMemory / 1MB, 2)
        
        # Prevent division by zero
        if ($totalMemory -gt 0) {
            $memoryUsage = [math]::Round((($totalMemory - $freeMemory) / $totalMemory) * 100, 1)
        } else {
            $memoryUsage = 0
        }
        
        Write-Log ("Memory Usage: {0}% ({1} MB free of {2} MB)" -f $memoryUsage, $freeMemory, $totalMemory) "METRIC"
    } catch {
        Write-Log "Failed to get memory info: $_" "ERROR"
        $memoryUsage = 0
    }
    
    # Check disk space
    try {
        $disk = Get-WmiObject -Class Win32_LogicalDisk -Filter "DeviceID='C:'"
        $freeSpace = [math]::Round($disk.FreeSpace / 1GB, 2)
        $totalSpace = [math]::Round($disk.Size / 1GB, 2)
        
        # Prevent division by zero
        if ($totalSpace -gt 0) {
            $diskUsage = [math]::Round((($totalSpace - $freeSpace) / $totalSpace) * 100, 1)
        } else {
            $diskUsage = 0
        }
        
        Write-Log ("Disk Usage: {0}% ({1} GB free of {2} GB)" -f $diskUsage, $freeSpace, $totalSpace) "METRIC"
    } catch {
        Write-Log "Failed to get disk info: $_" "ERROR"
        $diskUsage = 0
        $freeSpace = 0
    }
    
    # Check if memory usage is critical
    if ($memoryUsage -gt 90) {
        Write-Log "CRITICAL: Memory usage at $memoryUsage%" "CRITICAL"
        return $false
    }
    
    # Check if disk space is critical
    if ($freeSpace -lt 1) {
        Write-Log "CRITICAL: Low disk space - $freeSpace GB remaining" "CRITICAL"
        return $false
    }
    
    return $true
}

function Analyze-UIComponents {
    Write-Log "Analyzing UI components..." "CHECK"
    
    # Check for UI panel definitions in editor_ui.rs
    $uiStateFile = "src/editor_ui.rs"
    if (Test-Path $uiStateFile) {
        $content = Get-Content $uiStateFile -Raw
        
        # Count total UI panels
        $panelCount = ($content | Select-String -Pattern "pub show_[a-zA-Z_]+_panel:" -AllMatches).Matches.Count
        Write-Log "Total UI panels defined: $panelCount" "UI_ANALYSIS"
        
        # Check for 3D Scene panel specifically
        if ($content -match "show_3d_scene_panel") {
            Write-Log "3D Scene panel is properly defined" "UI_ANALYSIS"
        } else {
            Write-Log "3D Scene panel definition missing" "UI_WARNING"
        }
        
        # Check for UI integration patterns
        $checkboxCount = ($content | Select-String -Pattern "ui.checkbox\(" -AllMatches).Matches.Count
        $menuButtonCount = ($content | Select-String -Pattern "ui.menu_button\(" -AllMatches).Matches.Count
        Write-Log "UI Elements - Checkboxes: $checkboxCount, Menu Buttons: $menuButtonCount" "UI_METRICS"
        
        # Check for deprecated UI patterns
        if ($content -match "close_menu\(") {
            $deprecatedCount = ($content | Select-String -Pattern "close_menu\(").Matches.Count
            Write-Log "Deprecated close_menu() calls found: $deprecatedCount" "UI_DEPRECATED"
        }
        
        # Check for ComboBox usage
        if ($content -match "ComboBox::from_id_source") {
            $comboBoxCount = ($content | Select-String -Pattern "ComboBox::from_id_source").Matches.Count
            Write-Log "ComboBox::from_id_source usage found: $comboBoxCount (deprecated)" "UI_DEPRECATED"
        }
        
        return $true
    } else {
        Write-Log "UI state file not found: $uiStateFile" "UI_ERROR"
        return $false
    }
    
    # Check disk space
    try {
        $disk = Get-WmiObject -Class Win32_LogicalDisk -Filter "DeviceID='C:'"
        $freeSpace = [math]::Round($disk.FreeSpace / 1GB, 2)
        $totalSpace = [math]::Round($disk.Size / 1GB, 2)
        
        # Prevent division by zero
        if ($totalSpace -gt 0) {
            $diskUsage = [math]::Round((($totalSpace - $freeSpace) / $totalSpace) * 100, 1)
        } else {
            $diskUsage = 0
        }
        
        Write-Log ("Disk Usage: {0}% ({1} GB free of {2} GB)" -f $diskUsage, $freeSpace, $totalSpace) "METRIC"
    } catch {
        Write-Log "Failed to get disk info: $_" "ERROR"
        $diskUsage = 0
        $freeSpace = 0
    }
    
    # Check if memory usage is critical
    if ($memoryUsage -gt 90) {
        Write-Log "CRITICAL: Memory usage at $memoryUsage%" "CRITICAL"
        return $false
    }
    
    # Check if disk space is critical
    if ($freeSpace -lt 1) {
        Write-Log "CRITICAL: Low disk space - $freeSpace GB remaining" "CRITICAL"
        return $false
    }
    
    return $true
}

function Analyze-UIComponents {
    Write-Log "Analyzing UI components..." "CHECK"
    
    # Check for UI panel definitions in editor_ui.rs
    $uiStateFile = "src/editor_ui.rs"
    if (Test-Path $uiStateFile) {
        $content = Get-Content $uiStateFile -Raw
        
        # Count total UI panels
        $panelCount = ($content | Select-String -Pattern "pub show_[a-zA-Z_]+_panel:" -AllMatches).Matches.Count
        Write-Log "Total UI panels defined: $panelCount" "UI_ANALYSIS"
        
        # Check for 3D Scene panel specifically
        if ($content -match "show_3d_scene_panel") {
            Write-Log "3D Scene panel is properly defined" "UI_ANALYSIS"
        } else {
            Write-Log "3D Scene panel definition missing" "UI_WARNING"
        }
        
        # Check for UI integration patterns
        $checkboxCount = ($content | Select-String -Pattern "ui.checkbox\(" -AllMatches).Matches.Count
        $menuButtonCount = ($content | Select-String -Pattern "ui.menu_button\(" -AllMatches).Matches.Count
        Write-Log "UI Elements - Checkboxes: $checkboxCount, Menu Buttons: $menuButtonCount" "UI_METRICS"
        
        # Check for deprecated UI patterns
        if ($content -match "close_menu\(") {
            $deprecatedCount = ($content | Select-String -Pattern "close_menu\(").Matches.Count
            Write-Log "Deprecated close_menu() calls found: $deprecatedCount" "UI_DEPRECATED"
        }
        
        # Check for ComboBox usage
        if ($content -match "ComboBox::from_id_source") {
            $comboBoxCount = ($content | Select-String -Pattern "ComboBox::from_id_source").Matches.Count
            Write-Log "ComboBox::from_id_source usage found: $comboBoxCount (deprecated)" "UI_DEPRECATED"
        }
        
        return $true
    } else {
        Write-Log "UI state file not found: $uiStateFile" "UI_ERROR"
        return $false
    }
}

# MAIN TRACKING LOOP
Write-Log "STARTING LIVE APPLICATION TRACKER" "STARTUP"
Write-Log "Monitoring interval: $CheckInterval seconds" "CONFIG"
Write-Log "Log file: $LogFile" "CONFIG"
Write-Log "Error log: $ErrorLog" "CONFIG"

$consecutiveFailures = 0
$maxConsecutiveFailures = 3
$cycleCount = 0

try {
    while ($true) {
        Write-Log "Starting new monitoring cycle..." "CYCLE"
        $cycleCount++
        
        # Check compilation
        $compilationOk = Check-Compilation
        
        # Check runtime
        $runtimeOk = Check-ApplicationRuntime
        
        # Check system health
        $systemOk = Check-SystemHealth
        
        # Analyze UI components
        $uiOk = Analyze-UIComponents
        
        # Track consecutive failures
        if (-not $compilationOk) {
            $consecutiveFailures++
            Write-Log "Consecutive compilation failures: $consecutiveFailures" "WARNING"
            
            if ($consecutiveFailures -ge $maxConsecutiveFailures) {
                Write-Log "CRITICAL: Too many consecutive failures!" "CRITICAL"
                Write-Log "TERMINATING TRACKING - SYSTEM UNSTABLE" "TERMINATE"
                exit 1
            }
        } else {
            $consecutiveFailures = 0
        }
        
        # Overall status
        if ($compilationOk -and $runtimeOk -and $systemOk -and $uiOk) {
            Write-Log "SYSTEM HEALTHY - All checks passed" "HEALTHY"
        } elseif ($compilationOk -and $runtimeOk -and $uiOk) {
            Write-Log "SYSTEM STABLE - Minor issues detected" "STABLE"
        } else {
            Write-Log "SYSTEM UNSTABLE - Critical issues detected" "UNSTABLE"
        }
        
        Write-Log "Waiting $CheckInterval seconds..." "WAIT"
        Start-Sleep -Seconds $CheckInterval
    }
} catch {
    Write-Log "TRACKER CRASHED: $_" "CRASH"
    exit 1
}