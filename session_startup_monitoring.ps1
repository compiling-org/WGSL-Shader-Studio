# SESSION STARTUP MONITORING SYSTEM
# Launches all comprehensive monitoring systems for the session
# This runs at the start of every session to prevent psychotic loops

param(
    [switch]$ForceRestart = $false,
    [int]$StartupDelay = 5
)

Write-Host "🚀 SESSION STARTUP MONITORING SYSTEM" -ForegroundColor Green
Write-Host "💀 PREVENTING PSYCHOTIC LOOPS - DISCIPLINE ENFORCEMENT" -ForegroundColor Red

function Write-Startup {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    
    switch ($Level) {
        "CRITICAL" { Write-Host $logEntry -ForegroundColor Red }
        "WARNING" { Write-Host $logEntry -ForegroundColor Yellow }
        "SUCCESS" { Write-Host $logEntry -ForegroundColor Green }
        default { Write-Host $logEntry -ForegroundColor White }
    }
}

function Kill-ExistingMonitors {
    Write-Startup "🔍 Looking for existing monitoring processes..." "CHECK"
    
    # Find PowerShell processes running our scripts
    $monitoringScripts = @(
        "live_app_tracker.ps1",
        "enhanced_error_analyzer.ps1", 
        "runtime_application_monitor.ps1",
        "integrated_comprehensive_monitor.ps1",
        "comprehensive_enforcer.ps1",
        "comprehensive_ui_analyzer.ps1"
    )
    
    $killedCount = 0
    
    Get-Process | Where-Object { $_.ProcessName -eq "powershell" } | ForEach-Object {
        try {
            $cmdLine = (Get-WmiObject Win32_Process -Filter "ProcessId = $($_.Id)").CommandLine
            
            foreach ($script in $monitoringScripts) {
                if ($cmdLine -like "*$script*") {
                    Write-Startup "🛑 Killing existing monitor: $script (PID: $($_.Id))" "STOP"
                    Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
                    $killedCount++
                    break
                }
            }
        } catch {
            # Process might have already exited
        }
    }
    
    Write-Startup "💀 Killed $killedCount existing monitoring processes" "STOP_COMPLETE"
}

function Check-SystemPrerequisites {
    Write-Startup "🔍 Checking system prerequisites..." "CHECK"
    
    # Check if Rust is installed
    try {
        $rustVersion = rustc --version 2>$null
        if ($rustVersion) {
            Write-Startup "✅ Rust installed: $rustVersion" "SUCCESS"
        } else {
            Write-Startup "❌ Rust not found" "CRITICAL"
            return $false
        }
    } catch {
        Write-Startup "❌ Rust check failed: $_" "CRITICAL"
        return $false
    }
    
    # Check if Cargo is installed
    try {
        $cargoVersion = cargo --version 2>$null
        if ($cargoVersion) {
            Write-Startup "✅ Cargo installed: $cargoVersion" "SUCCESS"
        } else {
            Write-Startup "❌ Cargo not found" "CRITICAL"
            return $false
        }
    } catch {
        Write-Startup "❌ Cargo check failed: $_" "CRITICAL"
        return $false
    }
    
    # Check disk space
    $disk = Get-WmiObject -Class Win32_LogicalDisk -Filter "DeviceID='C:'"
    $freeSpaceGB = [Math]::Round($disk.FreeSpace / 1GB, 2)
    
    if ($freeSpaceGB -lt 2) {
        Write-Startup "❌ Insufficient disk space: ${freeSpaceGB}GB free (need 2GB+)" "CRITICAL"
        return $false
    }
    
    Write-Startup "✅ Disk space: ${freeSpaceGB}GB free" "SUCCESS"
    
    # Check memory
    $memory = Get-WmiObject -Class Win32_OperatingSystem
    $freeMemoryGB = [Math]::Round($memory.FreePhysicalMemory / 1MB / 1024, 2)
    
    if ($freeMemoryGB -lt 4) {
        Write-Startup "⚠️  Low memory: ${freeMemoryGB}GB free" "WARNING"
    } else {
        Write-Startup "✅ Memory: ${freeMemoryGB}GB free" "SUCCESS"
    }
    
    return $true
}

function Validate-ProjectStructure {
    Write-Startup "📁 Validating project structure..." "CHECK"
    
    $requiredFiles = @(
        "Cargo.toml",
        "src\main.rs",
        "src\editor_ui.rs"
    )
    
    $requiredDirs = @(
        "src",
        ".trae\documents"
    )
    
    $missingFiles = @()
    $missingDirs = @()
    
    foreach ($file in $requiredFiles) {
        if (-not (Test-Path $file)) {
            $missingFiles += $file
        }
    }
    
    foreach ($dir in $requiredDirs) {
        if (-not (Test-Path $dir)) {
            $missingDirs += $dir
        }
    }
    
    if ($missingFiles.Count -gt 0 -or $missingDirs.Count -gt 0) {
        Write-Startup "❌ Missing files: $($missingFiles -join ', ')" "CRITICAL"
        Write-Startup "❌ Missing directories: $($missingDirs -join ', ')" "CRITICAL"
        return $false
    }
    
    Write-Startup "✅ Project structure validated" "SUCCESS"
    return $true
}

function Start-ComprehensiveMonitoring {
    Write-Startup "🚀 Starting comprehensive monitoring systems..." "LAUNCH"
    
    # Start with a clean slate
    Kill-ExistingMonitors
    
    # Wait a moment for processes to clean up
    Start-Sleep -Seconds 2
    
    # Start the integrated comprehensive monitor (this starts all sub-monitors)
    Write-Startup "🎯 Starting integrated comprehensive monitor..." "LAUNCH_SYSTEM"
    
    try {
        $integratedMonitor = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File integrated_comprehensive_monitor.ps1" -PassThru -NoNewWindow
        
        Write-Startup "✅ Integrated monitor started with PID: $($integratedMonitor.Id)" "SUCCESS"
        
        # Give it time to start up
        Start-Sleep -Seconds 5
        
        # Verify it's running
        if (-not $integratedMonitor.HasExited) {
            Write-Startup "🟢 Integrated monitor is running" "RUNNING"
            
            # Monitor the monitor (meta-monitoring)
            Write-Startup "🔍 Starting meta-monitoring of the integrated system..." "META"
            
            $metaCheckCount = 0
            while ($metaCheckCount -lt 10 -and -not $integratedMonitor.HasExited) {
                Start-Sleep -Seconds 10
                $metaCheckCount++
                
                # Check if integrated monitor is still healthy
                try {
                    $memoryMB = [Math]::Round($integratedMonitor.WorkingSet64 / 1MB, 2)
                    Write-Startup "📊 Integrated monitor health check $metaCheckCount - Memory: ${memoryMB}MB" "META_CHECK"
                    
                    if ($memoryMB -gt 1000) {  # 1GB threshold
                        Write-Startup "⚠️  High memory usage in integrated monitor: ${memoryMB}MB" "WARNING"
                    }
                } catch {
                    Write-Startup "❌ Failed to check integrated monitor health" "ERROR"
                    break
                }
            }
            
            if ($integratedMonitor.HasExited) {
                $exitCode = $integratedMonitor.ExitCode
                Write-Startup "💥 Integrated monitor exited with code: $exitCode" "CRASH"
                
                if ($exitCode -ne 0) {
                    Write-Startup "🚨 CRITICAL: Integrated monitor failed - check logs" "CRITICAL"
                    return $false
                }
            } else {
                Write-Startup "✅ Integrated monitor completed startup phase successfully" "SUCCESS"
            }
            
        } else {
            $exitCode = $integratedMonitor.ExitCode
            Write-Startup "💥 Integrated monitor failed immediately with code: $exitCode" "CRITICAL"
            return $false
        }
        
    } catch {
        Write-Startup "❌ Failed to start integrated monitor: $_" "CRITICAL"
        return $false
    }
    
    return $true
}

function Show-SystemStatus {
    Write-Startup "📊 Current system status:" "STATUS"
    
    # Show running monitors
    $monitoringScripts = @(
        "live_app_tracker.ps1",
        "enhanced_error_analyzer.ps1", 
        "runtime_application_monitor.ps1",
        "integrated_comprehensive_monitor.ps1",
        "comprehensive_enforcer.ps1",
        "comprehensive_ui_analyzer.ps1"
    )
    
    $runningMonitors = @()
    Get-Process | Where-Object { $_.ProcessName -eq "powershell" } | ForEach-Object {
        try {
            $cmdLine = (Get-WmiObject Win32_Process -Filter "ProcessId = $($_.Id)").CommandLine
            foreach ($script in $monitoringScripts) {
                if ($cmdLine -like "*$script*") {
                    $runningMonitors += @{
                        Script = $script
                        PID = $_.Id
                        Memory = [Math]::Round($_.WorkingSet64 / 1MB, 2)
                    }
                    break
                }
            }
        } catch {
            # Ignore
        }
    }
    
    if ($runningMonitors.Count -gt 0) {
        Write-Startup "🟢 Running monitors: $($runningMonitors.Count)" "STATUS_GOOD"
        foreach ($monitor in $runningMonitors) {
            Write-Startup "   📊 $($monitor.Script) - PID: $($monitor.PID) - Memory: $($monitor.Memory)MB" "STATUS_DETAIL"
        }
    } else {
        Write-Startup "🔴 No monitors running" "STATUS_BAD"
    }
    
    # Show log files
    $logFiles = @(
        "live_app_tracking.log",
        "live_errors.log", 
        "runtime_monitoring.log",
        "app_crashes.log",
        "comprehensive_master.log",
        "critical_alerts.log"
    )
    
    Write-Startup "📁 Log files status:" "LOGS"
    foreach ($logFile in $logFiles) {
        if (Test-Path $logFile) {
            $size = [Math]::Round((Get-Item $logFile).Length / 1KB, 2)
            $recent = if ((Get-Item $logFile).LastWriteTime -gt (Get-Date).AddMinutes(-5)) { "🟢" } else { "⚠️" }
            Write-Startup "   $recent $logFile - ${size}KB" "LOG_DETAIL"
        } else {
            Write-Startup "   ❌ $logFile - NOT FOUND" "LOG_MISSING"
        }
    }
}

# MAIN STARTUP SEQUENCE
try {
    Write-Startup "🚀 SESSION STARTUP MONITORING SYSTEM INITIATED" "SYSTEM_START"
    Write-Startup "⏰ Startup delay: $StartupDelay seconds" "CONFIG"
    
    # Initial delay
    Start-Sleep -Seconds $StartupDelay
    
    # Kill existing monitors if forced restart
    if ($ForceRestart) {
        Write-Startup "🔥 FORCE RESTART REQUESTED - Killing all existing monitors" "FORCE"
        Kill-ExistingMonitors
        Start-Sleep -Seconds 3
    }
    
    # Check system prerequisites
    Write-Startup "🔍 Phase 1: System prerequisites check" "PHASE_START"
    if (-not (Check-SystemPrerequisites)) {
        Write-Startup "💥 CRITICAL: System prerequisites check failed" "CRITICAL"
        exit 1
    }
    
    # Validate project structure
    Write-Startup "🔍 Phase 2: Project structure validation" "PHASE_START"
    if (-not (Validate-ProjectStructure)) {
        Write-Startup "💥 CRITICAL: Project structure validation failed" "CRITICAL"
        exit 1
    }
    
    # Start comprehensive monitoring
    Write-Startup "🚀 Phase 3: Starting comprehensive monitoring" "PHASE_START"
    if (-not (Start-ComprehensiveMonitoring)) {
        Write-Startup "💥 CRITICAL: Failed to start comprehensive monitoring" "CRITICAL"
        exit 1
    }
    
    # Show final status
    Write-Startup "✅ SESSION STARTUP COMPLETE" "SUCCESS"
    Show-SystemStatus
    
    Write-Startup "🟢 ALL SYSTEMS OPERATIONAL - DISCIPLINE ENFORCEMENT ACTIVE" "OPERATIONAL"
    Write-Startup "💀 MONITORING WILL CONTINUE UNTIL SESSION END" "SESSION_ACTIVE"
    
    # Keep the script running to show status
    Write-Startup "⏰ Press Ctrl+C to stop monitoring (NOT RECOMMENDED)" "WARNING"
    
    while ($true) {
        Start-Sleep -Seconds 30
        Show-SystemStatus
    }
    
} catch {
    Write-Startup "💥 SESSION STARTUP FAILED: $_" "CRITICAL"
    exit 1
}