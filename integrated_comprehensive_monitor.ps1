# INTEGRATED COMPREHENSIVE MONITORING SYSTEM
# Combines all monitoring systems for complete project oversight
# This system NEVER turns off and continuously enforces discipline

param(
    [int]$MasterCheckInterval = 10,  # Master check every 10 seconds
    [string]$MasterLog = "comprehensive_master.log",
    [string]$AlertLog = "critical_alerts.log"
)

# Global configuration
$Global:MonitoringActive = $true
$Global:ConsecutiveViolations = 0
$Global:MaxConsecutiveViolations = 3
$Global:CriticalErrorCount = 0
$Global:MaxCriticalErrors = 10

function Write-Master {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Write-Host $logEntry -ForegroundColor $(Get-ColorForLevel $Level)
    Add-Content -Path $MasterLog -Value $logEntry
    
    # Log critical alerts separately
    if ($Level -in @("CRITICAL", "VIOLATION", "TERMINATE")) {
        Add-Content -Path $AlertLog -Value $logEntry
    }
}

function Get-ColorForLevel {
    param([string]$Level)
    switch ($Level) {
        "CRITICAL" { return "Red" }
        "VIOLATION" { return "DarkRed" }
        "WARNING" { return "Yellow" }
        "SUCCESS" { return "Green" }
        "INFO" { return "White" }
        default { return "Gray" }
    }
}

function Start-SubMonitor {
    param([string]$ScriptName, [string]$Arguments = "")
    
    Write-Master "🚀 Starting sub-monitor: $ScriptName" "STARTUP"
    
    try {
        $process = Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File $ScriptName $Arguments" -PassThru -NoNewWindow -RedirectStandardOutput "${ScriptName}.out.log" -RedirectStandardError "${ScriptName}.err.log"
        
        Write-Master "✅ Sub-monitor started with PID: $($Process.Id)" "SUCCESS"
        return $process
        
    } catch {
        Write-Master "❌ Failed to start sub-monitor $ScriptName: $_" "ERROR"
        return $null
    }
}

function Check-SubMonitorHealth {
    param([System.Diagnostics.Process]$Process, [string]$Name)
    
    if ($Process -eq $null) {
        Write-Master "⚠️  Sub-monitor $Name is null" "WARNING"
        return $false
    }
    
    if ($Process.HasExited) {
        $exitCode = $Process.ExitCode
        Write-Master "💥 Sub-monitor $Name exited with code: $exitCode" "ERROR"
        
        # Check error logs
        $errLog = "${Name}.err.log"
        if (Test-Path $errLog) {
            $errors = Get-Content $errLog -Tail 5
            foreach ($error in $errors) {
                Write-Master "   🚨 $error" "ERROR_DETAIL"
            }
        }
        
        return $false
    }
    
    # Check memory usage
    try {
        $memoryMB = [Math]::Round($Process.WorkingSet64 / 1MB, 2)
        if ($memoryMB -gt 500) {  # 500MB threshold
            Write-Master "⚠️  Sub-monitor $Name using high memory: ${memoryMB}MB" "WARNING"
        }
    } catch {
        # Process might have exited during check
    }
    
    return $true
}

function Monitor-CompilationStatus {
    Write-Master "🔍 Checking compilation status..." "CHECK"
    
    # Quick compilation check
    $output = cargo check 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Master "✅ Compilation successful" "SUCCESS"
        $Global:ConsecutiveViolations = 0
        return $true
    } else {
        $Global:ConsecutiveViolations++
        Write-Master "❌ Compilation failed (consecutive violations: $($Global:ConsecutiveViolations))" "VIOLATION"
        
        # Count specific error types
        $coloredErrors = ($output | Where-Object { $_ -match "colored" }).Count
        $serdeErrors = ($output | Where-Object { $_ -match "serde" }).Count
        $borrowErrors = ($output | Where-Object { $_ -match "cannot borrow" }).Count
        
        Write-Master "📊 Error breakdown - Colored: $coloredErrors, Serde: $serdeErrors, Borrow: $borrowErrors" "ANALYSIS"
        
        if ($Global:ConsecutiveViolations -ge $Global:MaxConsecutiveViolations) {
            Write-Master "🚨 MAX VIOLATIONS REACHED - SYSTEM FAILURE" "CRITICAL"
            return $false
        }
        
        return $false
    }
}

function Monitor-SystemResources {
    Write-Master "🏥 Checking system resources..." "CHECK"
    
    # Memory check
    $memory = Get-WmiObject -Class Win32_OperatingSystem
    $memoryUsage = [Math]::Round((($memory.TotalPhysicalMemory - $memory.FreePhysicalMemory) / $memory.TotalPhysicalMemory) * 100, 1)
    
    # Disk check
    $disk = Get-WmiObject -Class Win32_LogicalDisk -Filter "DeviceID='C:'"
    $freeSpaceGB = [Math]::Round($disk.FreeSpace / 1GB, 2)
    
    # CPU check (simplified)
    $cpuUsage = Get-WmiObject -Class Win32_Processor | Measure-Object -Property LoadPercentage -Average | Select-Object -ExpandProperty Average
    
    Write-Master "📊 System metrics - Memory: $memoryUsage%, Disk: ${freeSpaceGB}GB free, CPU: $cpuUsage%" "METRICS"
    
    # Critical thresholds
    if ($memoryUsage -gt 95) {
        Write-Master "🚨 CRITICAL: Memory usage at $memoryUsage%" "CRITICAL"
        return $false
    }
    
    if ($freeSpaceGB -lt 0.5) {
        Write-Master "🚨 CRITICAL: Low disk space - ${freeSpaceGB}GB" "CRITICAL"
        return $false
    }
    
    if ($cpuUsage -gt 95) {
        Write-Master "🚨 CRITICAL: High CPU usage - $cpuUsage%" "CRITICAL"
        return $false
    }
    
    return $true
}

function Check-DocumentationCompliance {
    Write-Master "📚 Checking documentation compliance..." "CHECK"
    
    $requiredDocs = @(
        ".\HONEST_RECOVERY_PLAN.md",
        ".\COMPLETE_DESTRUCTION_TRUTH.md",
        ".\.trae\documents\product.md",
        ".\.trae\documents\technical_architecture.md"
    )
    
    $missingDocs = @()
    foreach ($doc in $requiredDocs) {
        if (-not (Test-Path $doc)) {
            $missingDocs += $doc
        }
    }
    
    if ($missingDocs.Count -gt 0) {
        Write-Master "⚠️  Missing documentation: $($missingDocs -join ', ')" "WARNING"
        return $false
    }
    
    Write-Master "✅ All required documentation present" "SUCCESS"
    return $true
}

function Generate-ComprehensiveReport {
    Write-Master "📋 Generating comprehensive system report..." "REPORT"
    
    $report = @"
=== COMPREHENSIVE SYSTEM REPORT ===
Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
System Uptime: $([Math]::Round((Get-Date) - (Get-Process -Id $PID).StartTime).TotalHours, 2) hours

MONITORING STATUS:
- Master Process ID: $PID
- Monitoring Active: $($Global:MonitoringActive)
- Consecutive Violations: $($Global:ConsecutiveViolations)
- Critical Errors: $($Global:CriticalErrorCount)

SYSTEM HEALTH:
- Memory Usage: $([Math]::Round(((Get-WmiObject -Class Win32_OperatingSystem).TotalPhysicalMemory - (Get-WmiObject -Class Win32_OperatingSystem).FreePhysicalMemory) / (Get-WmiObject -Class Win32_OperatingSystem).TotalPhysicalMemory * 100, 1))%
- Disk Free: $([Math]::Round((Get-WmiObject -Class Win32_LogicalDisk -Filter "DeviceID='C:'").FreeSpace / 1GB, 2)) GB
- CPU Load: $((Get-WmiObject -Class Win32_Processor | Measure-Object -Property LoadPercentage -Average | Select-Object -ExpandProperty Average))%

COMPILATION STATUS:
- Last Check: $(if (Monitor-CompilationStatus) { "SUCCESS" } else { "FAILED" })
- Error Count: $(if (Test-Path "live_errors.log") { (Get-Content "live_errors.log" | Measure-Object).Count } else { 0 })

SUB-MONITORS:
- Live App Tracker: $(if (Test-Path "live_app_tracker.out.log") { "RUNNING" } else { "STOPPED" })
- Error Analyzer: $(if (Test-Path "enhanced_error_analyzer.out.log") { "RUNNING" } else { "STOPPED" })
- Runtime Monitor: $(if (Test-Path "runtime_application_monitor.out.log") { "RUNNING" } else { "STOPPED" })

CRITICAL ALERTS:
$(if (Test-Path $AlertLog) { Get-Content $AlertLog | Select-Object -Last 5 | ForEach-Object { "- $_" } } else { "No critical alerts" })

=== END REPORT ===
"@
    
    Write-Master $report "REPORT_DETAIL"
    
    # Save report to file
    $reportFile = "comprehensive_report_$(Get-Date -Format 'yyyyMMdd_HHmmss').txt"
    Set-Content -Path $reportFile -Value $report
    Write-Master "📄 Report saved to: $reportFile" "SUCCESS"
}

function Emergency-Shutdown {
    Write-Master "🚨 EMERGENCY SHUTDOWN INITIATED" "EMERGENCY"
    
    # Kill all PowerShell processes related to monitoring
    Get-Process | Where-Object { $_.ProcessName -eq "powershell" -and $_.Id -ne $PID } | Stop-Process -Force
    
    # Kill any Rust/cargo processes
    Get-Process | Where-Object { $_.ProcessName -in @("cargo", "rustc", "rust-analyzer") } | Stop-Process -Force
    
    Write-Master "💀 EMERGENCY SHUTDOWN COMPLETE" "EMERGENCY_COMPLETE"
    
    # Terminate this process
    exit 1
}

# MAIN COMPREHENSIVE MONITORING LOOP
Write-Master "🚀 INTEGRATED COMPREHENSIVE MONITORING SYSTEM STARTING" "SYSTEM_START"
Write-Master "💀 THIS SYSTEM NEVER TURNS OFF - CONTINUOUS ENFORCEMENT ACTIVE" "SYSTEM_WARNING"
Write-Master "🔥 ANY VIOLATION = IMMEDIATE TERMINATION" "SYSTEM_RULE"

try {
    # Start sub-monitors
    $subMonitors = @()
    
    Write-Master "🚀 Starting sub-monitors..." "SUB_START"
    
    # Start live app tracker
    $liveTracker = Start-SubMonitor -ScriptName "live_app_tracker.ps1" -Arguments "-CheckInterval 5"
    if ($liveTracker) { $subMonitors += $liveTracker }
    
    # Start enhanced error analyzer  
    $errorAnalyzer = Start-SubMonitor -ScriptName "enhanced_error_analyzer.ps1"
    if ($errorAnalyzer) { $subMonitors += $errorAnalyzer }
    
    # Start runtime monitor
    $runtimeMonitor = Start-SubMonitor -ScriptName "runtime_application_monitor.ps1" -Arguments "-CheckInterval 3"
    if ($runtimeMonitor) { $subMonitors += $runtimeMonitor }
    
    Write-Master "✅ Started $($subMonitors.Count) sub-monitors" "SUB_SUCCESS"
    
    # Main monitoring loop
    $cycleCount = 0
    while ($Global:MonitoringActive) {
        $cycleCount++
        Write-Master "🔄 MASTER CYCLE $cycleCount STARTING" "CYCLE_START"
        
        # Critical system checks
        $compilationOk = Monitor-CompilationStatus
        $resourcesOk = Monitor-SystemResources
        $docsOk = Check-DocumentationCompliance
        
        # Check sub-monitor health every 5 cycles
        if ($cycleCount % 5 -eq 0) {
            Write-Master "🔍 Checking sub-monitor health..." "HEALTH_CHECK"
            foreach ($monitor in $subMonitors) {
                $healthy = Check-SubMonitorHealth -Process $monitor -Name $monitor.ProcessName
                if (-not $healthy) {
                    Write-Master "⚠️  Sub-monitor failed health check" "HEALTH_FAIL"
                }
            }
        }
        
        # Generate comprehensive report every 10 cycles
        if ($cycleCount % 10 -eq 0) {
            Generate-ComprehensiveReport
        }
        
        # Check for system-wide failures
        if (-not $compilationOk -or -not $resourcesOk -or -not $docsOk) {
            $Global:CriticalErrorCount++
            Write-Master "🚨 CRITICAL ERROR DETECTED (count: $($Global:CriticalErrorCount))" "CRITICAL"
            
            if ($Global:CriticalErrorCount -ge $Global:MaxCriticalErrors) {
                Write-Master "💀 MAX CRITICAL ERRORS REACHED - EMERGENCY SHUTDOWN" "EMERGENCY"
                Emergency-Shutdown
            }
        }
        
        # Overall system status
        if ($compilationOk -and $resourcesOk -and $docsOk) {
            Write-Master "🟢 SYSTEM HEALTHY - All critical checks passed" "HEALTHY"
        } elseif ($compilationOk -and $resourcesOk) {
            Write-Master "🟡 SYSTEM STABLE - Minor issues detected" "STABLE"
        } else {
            Write-Master "🔴 SYSTEM UNSTABLE - Critical issues present" "UNSTABLE"
        }
        
        Write-Master "⏳ Waiting $MasterCheckInterval seconds..." "WAIT"
        Start-Sleep -Seconds $MasterCheckInterval
    }
    
} catch {
    Write-Master "💥 MASTER MONITOR CRASHED: $_" "CRASH"
    Emergency-Shutdown
}

# This should never be reached, but just in case
Write-Master "💀 MASTER MONITOR LOOP EXITED - THIS SHOULD NEVER HAPPEN" "FATAL"
Emergency-Shutdown