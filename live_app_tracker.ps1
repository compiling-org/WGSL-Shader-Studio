# LIVE APPLICATION TRACKER - CONTINUOUS MONITORING
# Tracks application compilation, runtime errors, and system health

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
    Write-Log "🔍 Checking compilation status..." "CHECK"
    
    # Run cargo check and capture output
    $output = cargo check 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Log "✅ Compilation SUCCESSFUL" "SUCCESS"
        return $true
    } else {
        Write-Log "❌ Compilation FAILED with $exitCode errors" "ERROR"
        
        # Extract and log specific errors
        $errorLines = $output | Where-Object { $_ -match "error\[" }
        foreach ($error in $errorLines) {
            Write-Log "🚨 $error" "ERROR_DETAIL"
            Add-Content -Path $ErrorLog -Value "$(Get-Date -Format 'HH:mm:ss') - $error"
        }
        
        return $false
    }
}

function Check-ApplicationRuntime {
    Write-Log "🏃 Checking application runtime status..." "CHECK"
    
    # Check if the application binary exists
    if (Test-Path "target\release\wgsl-shader-studio.exe") {
        Write-Log "✅ Application binary EXISTS" "SUCCESS"
        return $true
    } elseif (Test-Path "target\debug\wgsl-shader-studio.exe") {
        Write-Log "⚠️  Application binary exists in DEBUG mode" "WARNING"
        return $true
    } else {
        Write-Log "❌ Application binary NOT FOUND" "ERROR"
        return $false
    }
}

function Check-SystemHealth {
    Write-Log "🏥 Checking system health..." "CHECK"
    
    # Check memory usage
    $memory = Get-WmiObject -Class Win32_OperatingSystem
    $freeMemory = [math]::Round($memory.FreePhysicalMemory / 1MB, 2)
    $totalMemory = [math]::Round($memory.TotalPhysicalMemory / 1MB, 2)
    $memoryUsage = [math]::Round((($totalMemory - $freeMemory) / $totalMemory) * 100, 1)
    
    Write-Log "💾 Memory Usage: $memoryUsage% ($freeMemory MB free of $totalMemory MB)" "METRIC"
    
    # Check disk space
    $disk = Get-WmiObject -Class Win32_LogicalDisk -Filter "DeviceID='C:'"
    $freeSpace = [math]::Round($disk.FreeSpace / 1GB, 2)
    $totalSpace = [math]::Round($disk.Size / 1GB, 2)
    $diskUsage = [math]::Round((($totalSpace - $freeSpace) / $totalSpace) * 100, 1)
    
    Write-Log "💿 Disk Usage: $diskUsage% ($freeSpace GB free of $totalSpace GB)" "METRIC"
    
    # Check if memory usage is critical
    if ($memoryUsage -gt 90) {
        Write-Log "🚨 CRITICAL: Memory usage at $memoryUsage%" "CRITICAL"
        return $false
    }
    
    # Check if disk space is critical
    if ($freeSpace -lt 1) {
        Write-Log "🚨 CRITICAL: Low disk space - $freeSpace GB remaining" "CRITICAL"
        return $false
    }
    
    return $true
}

function Analyze-ErrorPatterns {
    Write-Log "🔬 Analyzing error patterns..." "ANALYSIS"
    
    if (Test-Path $ErrorLog) {
        $recentErrors = Get-Content $ErrorLog | Where-Object { $_ -match "$(Get-Date -Format 'yyyy-MM-dd')" }
        
        if ($recentErrors.Count -gt 0) {
            Write-Log "📊 Found $($recentErrors.Count) errors today" "ANALYSIS"
            
            # Count error types
            $errorTypes = @{}
            foreach ($error in $recentErrors) {
                if ($error -match "error\[([E0-9]+)\]") {
                    $errorCode = $matches[1]
                    if ($errorTypes.ContainsKey($errorCode)) {
                        $errorTypes[$errorCode]++
                    } else {
                        $errorTypes[$errorCode] = 1
                    }
                }
            }
            
            foreach ($errorType in $errorTypes.GetEnumerator()) {
                Write-Log "📈 Error $($errorType.Key): $($errorType.Value) occurrences" "ANALYSIS"
            }
        }
    }
}

function Generate-HealthReport {
    Write-Log "📋 Generating health report..." "REPORT"
    
    $report = @"
=== WGSL SHADER STUDIO HEALTH REPORT ===
Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

COMPILATION STATUS:
- Last Check: $(if (Check-Compilation) { "SUCCESS" } else { "FAILED" })

RUNTIME STATUS:
- Binary Available: $(if (Check-ApplicationRuntime) { "YES" } else { "NO" })

SYSTEM METRICS:
- Memory Usage: $([math]::Round(((Get-WmiObject -Class Win32_OperatingSystem).TotalPhysicalMemory - (Get-WmiObject -Class Win32_OperatingSystem).FreePhysicalMemory) / (Get-WmiObject -Class Win32_OperatingSystem).TotalPhysicalMemory * 100, 1))%
- Disk Free: $([math]::Round((Get-WmiObject -Class Win32_LogicalDisk -Filter "DeviceID='C:'").FreeSpace / 1GB, 2)) GB

RECENT ERRORS:
$(if (Test-Path $ErrorLog) { Get-Content $ErrorLog | Select-Object -Last 5 | ForEach-Object { "- $_" } })

=== END REPORT ===
"@
    
    Write-Log $report "REPORT_DETAIL"
}

# MAIN TRACKING LOOP
Write-Log "🚀 STARTING LIVE APPLICATION TRACKER" "STARTUP"
Write-Log "📊 Monitoring interval: $CheckInterval seconds" "CONFIG"
Write-Log "📝 Log file: $LogFile" "CONFIG"
Write-Log "⚠️  Error log: $ErrorLog" "CONFIG"

$consecutiveFailures = 0
$maxConsecutiveFailures = 3

try {
    while ($true) {
        Write-Log "🔄 Starting new monitoring cycle..." "CYCLE"
        
        # Check compilation
        $compilationOk = Check-Compilation
        
        # Check runtime
        $runtimeOk = Check-ApplicationRuntime
        
        # Check system health
        $systemOk = Check-SystemHealth
        
        # Analyze error patterns
        Analyze-ErrorPatterns
        
        # Generate health report every 10 cycles
        if ($cycleCount % 10 -eq 0) {
            Generate-HealthReport
        }
        
        # Track consecutive failures
        if (-not $compilationOk) {
            $consecutiveFailures++
            Write-Log "⚠️  Consecutive compilation failures: $consecutiveFailures" "WARNING"
            
            if ($consecutiveFailures -ge $maxConsecutiveFailures) {
                Write-Log "🚨 CRITICAL: Too many consecutive failures!" "CRITICAL"
                Write-Log "💀 TERMINATING TRACKING - SYSTEM UNSTABLE" "TERMINATE"
                exit 1
            }
        } else {
            $consecutiveFailures = 0
        }
        
        # Overall status
        if ($compilationOk -and $runtimeOk -and $systemOk) {
            Write-Log "🟢 SYSTEM HEALTHY - All checks passed" "HEALTHY"
        } elseif ($compilationOk -and $runtimeOk) {
            Write-Log "🟡 SYSTEM STABLE - Minor issues detected" "STABLE"
        } else {
            Write-Log "🔴 SYSTEM UNSTABLE - Critical issues detected" "UNSTABLE"
        }
        
        Write-Log "⏳ Waiting $CheckInterval seconds..." "WAIT"
        Start-Sleep -Seconds $CheckInterval
        $cycleCount++
    }
} catch {
    Write-Log "💥 TRACKER CRASHED: $_" "CRASH"
    exit 1
}