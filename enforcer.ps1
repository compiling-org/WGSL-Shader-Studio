# WGSL Shader Studio - Psychotic Loop Prevention Enforcer
# This script runs constantly in the background to prevent development loops

param(
    [int]$CheckInterval = 5,  # Check every 5 seconds
    [int]$MaxWarnings = 3,    # Maximum warnings before termination
    [string]$LogFile = "enforcer.log"
)

$ErrorActionPreference = "Stop"
$warnings = 0
$startTime = Get-Date

function Write-EnforcerLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Write-Host $logEntry -ForegroundColor $(if($Level -eq "ERROR"){"Red"} elseif($Level -eq "WARN"){"Yellow"} else{"Green"})
    Add-Content -Path $LogFile -Value $logEntry
}

function Check-DevelopmentLoop {
    # Check for signs of psychotic development loops
    $loopIndicators = @(
        "minimal-test",
        "minimal_gui_test", 
        "test-window",
        "simple-test",
        "gui-test"
    )
    
    $processes = Get-Process -ErrorAction SilentlyContinue
    $foundLoop = $false
    
    foreach ($indicator in $loopIndicators) {
        if ($processes.ProcessName -contains $indicator) {
            Write-EnforcerLog "LOOP DETECTED: Found process '$indicator' running" "WARN"
            $foundLoop = $true
            break
        }
    }
    
    # Check for repeated compilation of test binaries
    $cargoProcesses = $processes | Where-Object { $_.ProcessName -eq "cargo" }
    if ($cargoProcesses.Count -gt 2) {
        Write-EnforcerLog "LOOP DETECTED: Multiple cargo processes running ($($cargoProcesses.Count))" "WARN"
        $foundLoop = $true
    }
    
    return $foundLoop
}

function Check-ProjectState {
    # Check if main application is being neglected
    $mainBinaries = @("isf-shaders", "main")
    $hasMainBinary = $false
    
    foreach ($binary in $mainBinaries) {
        if (Test-Path "target\debug\$binary.exe") {
            $hasMainBinary = $true
            break
        }
    }
    
    if (-not $hasMainBinary) {
        Write-EnforcerLog "WARNING: Main application binary not found - focus on comprehensive features needed" "WARN"
        return $true
    }
    
    return $false
}

function Enforce-Progress {
    # Check for signs of actual progress vs loop behavior
    $progressIndicators = @(
        "src\editor_ui.rs",
        "src\bevy_app.rs",
        "src\main.rs"
    )
    
    $recentChanges = $false
    $cutoffTime = (Get-Date).AddMinutes(-10)
    
    foreach ($file in $progressIndicators) {
        if (Test-Path $file) {
            $lastWrite = (Get-Item $file).LastWriteTime
            if ($lastWrite -gt $cutoffTime) {
                $recentChanges = $true
                break
            }
        }
    }
    
    if (-not $recentChanges) {
        Write-EnforcerLog "WARNING: No recent changes to core UI files - development may be stuck" "WARN"
        return $true
    }
    
    return $false
}

# Main enforcement loop
Write-EnforcerLog "WGSL Shader Studio Enforcer Started"
Write-EnforcerLog "Monitoring for psychotic development loops..."
Write-EnforcerLog "Check interval: $CheckInterval seconds"
Write-EnforcerLog "Max warnings before intervention: $MaxWarnings"

try {
    while ($true) {
        $loopDetected = Check-DevelopmentLoop
        $neglectDetected = Check-ProjectState  
        $stagnationDetected = Enforce-Progress
        
        if ($loopDetected -or $neglectDetected -or $stagnationDetected) {
            $warnings++
            Write-EnforcerLog "Warning count: $warnings/$MaxWarnings" "WARN"
            
            if ($warnings -ge $MaxWarnings) {
                Write-EnforcerLog "MAXIMUM WARNINGS REACHED - TAKING DRASTIC ACTION" "ERROR"
                Write-EnforcerLog "Terminating all test processes and forcing focus on comprehensive features" "ERROR"
                
                # Kill all test-related processes
                Get-Process | Where-Object { 
                    $_.ProcessName -match "test|minimal|simple|gui" 
                } | Stop-Process -Force -ErrorAction SilentlyContinue
                
                Write-EnforcerLog "DRASTIC MEASURE: All test processes terminated" "ERROR"
                Write-EnforcerLog "ENFORCER: Focus must shift to main application and comprehensive features NOW" "ERROR"
                
                # Reset warnings after intervention
                $warnings = 0
            }
        } else {
            # Reset warnings when progress is detected
            if ($warnings -gt 0) {
                Write-EnforcerLog "Progress detected - resetting warning count" "INFO"
                $warnings = 0
            }
        }
        
        Start-Sleep -Seconds $CheckInterval
    }
}
catch {
    Write-EnforcerLog "Enforcer crashed: $($_.Exception.Message)" "ERROR"
    Write-EnforcerLog "Restarting enforcer in 10 seconds..." "ERROR"
    Start-Sleep -Seconds 10
    & $PSCommandPath @PSBoundParameters
}