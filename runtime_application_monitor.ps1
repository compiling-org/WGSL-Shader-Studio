# RUNTIME APPLICATION MONITOR - LIVE APP TRACKING
# Monitors the running application, captures crashes, performance, and logs

param(
    [int]$CheckInterval = 3,  # Check every 3 seconds
    [string]$AppName = "wgsl-shader-studio",
    [string]$RuntimeLog = "runtime_monitoring.log",
    [string]$CrashLog = "app_crashes.log",
    [int]$MaxCrashes = 5
)

function Write-Runtime {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Write-Host $logEntry
    Add-Content -Path $RuntimeLog -Value $logEntry
}

function Find-ApplicationProcess {
    $processes = Get-Process | Where-Object { $_.ProcessName -like "*$AppName*" -or $_.MainWindowTitle -like "*$AppName*" }
    return $processes
}

function Monitor-ApplicationProcess {
    param([System.Diagnostics.Process]$Process)
    
    Write-Runtime "🔍 Monitoring process ID: $($Process.Id)" "PROCESS"
    Write-Runtime "📊 Process name: $($Process.ProcessName)" "PROCESS"
    Write-Runtime "💾 Memory usage: $([Math]::Round($Process.WorkingSet64 / 1MB, 2)) MB" "METRIC"
    Write-Runtime "⚡ CPU usage: $($Process.CPU) seconds" "METRIC"
    
    # Monitor for crashes
    try {
        while (-not $Process.HasExited) {
            Start-Sleep -Seconds $CheckInterval
            
            # Update metrics
            $Process.Refresh()
            $memoryMB = [Math]::Round($Process.WorkingSet64 / 1MB, 2)
            $cpuTime = $Process.CPU
            
            Write-Runtime "💾 Memory: ${memoryMB}MB | CPU: ${cpuTime}s | Threads: $($Process.Threads.Count)" "METRICS"
            
            # Check for memory leaks
            if ($memoryMB -gt 1000) {  # 1GB threshold
                Write-Runtime "🚨 HIGH MEMORY USAGE: ${memoryMB}MB" "WARNING"
            }
            
            # Check for high CPU usage
            if ($cpuTime -gt 60) {  # 60 seconds threshold
                Write-Runtime "⚠️  HIGH CPU USAGE: ${cpuTime} seconds" "WARNING"
            }
        }
        
        # Process has exited
        $exitCode = $Process.ExitCode
        $exitTime = $Process.ExitTime
        $runTime = $Process.TotalProcessorTime
        
        Write-Runtime "💥 PROCESS EXITED - Code: $exitCode | Runtime: $runTime" "EXIT"
        
        # Log crash details
        $crashInfo = @"
=== APPLICATION CRASH DETECTED ===
Time: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
Process ID: $($Process.Id)
Exit Code: $exitCode
Runtime: $runTime
Memory at exit: $([Math]::Round($Process.WorkingSet64 / 1MB, 2)) MB
CPU time: $($Process.CPU) seconds
Start time: $($Process.StartTime)
Exit time: $exitTime

=== CRASH ANALYSIS ===
Exit code analysis:
$(switch ($exitCode) {
    0 { "✅ Normal exit" }
    1 { "❌ General error" }
    3 { "❌ Fatal error" }
    101 { "❌ Cargo build failed" }
    134 { "❌ Aborted (SIGABRT)" }
    139 { "❌ Segmentation fault (SIGSEGV)" }
    143 { "❌ Terminated by signal" }
    default { "❌ Unknown exit code - investigate" }
})

=== END CRASH REPORT ===
"@
        
        Add-Content -Path $CrashLog -Value $crashInfo
        
        return $exitCode
        
    } catch {
        Write-Runtime "💥 ERROR monitoring process: $_" "ERROR"
        return -1
    }
}

function Start-ApplicationWithMonitoring {
    Write-Runtime "🚀 Starting application with monitoring..." "STARTUP"
    
    # Try to build first
    Write-Runtime "🔨 Building application..." "BUILD"
    $buildOutput = cargo build --release 2>&1
    $buildExitCode = $LASTEXITCODE
    
    if ($buildExitCode -ne 0) {
        Write-Runtime "❌ Build failed with exit code $buildExitCode" "BUILD_FAIL"
        
        # Log build errors
        $buildErrors = $buildOutput | Where-Object { $_ -match "error\[" }
        foreach ($error in $buildErrors) {
            Write-Runtime "🚨 Build error: $error" "BUILD_ERROR"
        }
        
        return $false
    }
    
    Write-Runtime "✅ Build successful" "BUILD_SUCCESS"
    
    # Start the application
    Write-Runtime "▶️  Starting application..." "LAUNCH"
    
    try {
        $process = Start-Process -FilePath "target\release\$AppName.exe" -PassThru -NoNewWindow
        
        Write-Runtime "🎯 Application started with PID: $($Process.Id)" "LAUNCH_SUCCESS"
        
        # Monitor the process
        $exitCode = Monitor-ApplicationProcess -Process $process
        
        return $exitCode
        
    } catch {
        Write-Runtime "💥 Failed to start application: $_" "LAUNCH_FAIL"
        return -1
    }
}

function Monitor-ExistingApplication {
    Write-Runtime "🔍 Looking for existing application processes..." "SEARCH"
    
    $processes = Find-ApplicationProcess
    
    if ($processes.Count -eq 0) {
        Write-Runtime "❌ No application processes found" "NOT_FOUND"
        return $false
    }
    
    Write-Runtime "✅ Found $($processes.Count) application process(es)" "FOUND"
    
    foreach ($process in $processes) {
        Monitor-ApplicationProcess -Process $process
    }
    
    return $true
}

function Track-CrashHistory {
    Write-Runtime "📈 Analyzing crash history..." "HISTORY"
    
    if (Test-Path $CrashLog) {
        $crashes = Get-Content $CrashLog | Where-Object { $_ -match "APPLICATION CRASH DETECTED" }
        
        Write-Runtime "💥 Total crashes recorded: $($crashes.Count)" "CRASH_COUNT"
        
        if ($crashes.Count -ge $MaxCrashes) {
            Write-Runtime "🚨 CRITICAL: Too many crashes ($($crashes.Count) >= $MaxCrashes)" "CRITICAL"
            
            # Get recent crash details
            $recentCrashes = Get-Content $CrashLog | Select-Object -Last 50
            $exitCodes = $recentCrashes | Where-Object { $_ -match "Exit Code: (\d+)" } | ForEach-Object { $matches[1] }
            
            $exitCodeCounts = $exitCodes | Group-Object | Sort-Object Count -Descending
            
            Write-Runtime "📊 Recent exit codes:" "CRASH_ANALYSIS"
            foreach ($code in $exitCodeCounts) {
                Write-Runtime "   Exit code $($code.Name): $($code.Count) times" "CRASH_DETAIL"
            }
            
            return $false
        }
    }
    
    return $true
}

function Monitor-ApplicationLogs {
    Write-Runtime "📄 Monitoring application logs..." "LOGS"
    
    # Look for common log files
    $logFiles = @(
        "app.log",
        "application.log", 
        "wgsl-shader-studio.log",
        "debug.log",
        "error.log"
    )
    
    foreach ($logFile in $logFiles) {
        if (Test-Path $logFile) {
            Write-Runtime "📁 Found log file: $logFile" "LOG_FILE"
            
            # Check for recent errors in the log
            $recentLogEntries = Get-Content $logFile -Tail 10 | Where-Object { 
                $_ -match "(ERROR|FATAL|PANIC|CRASH)" 
            }
            
            if ($recentLogEntries.Count -gt 0) {
                Write-Runtime "⚠️  Found $($recentLogEntries.Count) recent error entries in $logFile" "LOG_ERROR"
                
                foreach ($entry in $recentLogEntries) {
                    Write-Runtime "   📝 $entry" "LOG_DETAIL"
                }
            }
        }
    }
}

# MAIN MONITORING EXECUTION
try {
    Write-Runtime "🚀 STARTING RUNTIME APPLICATION MONITOR" "STARTUP"
    Write-Runtime "📊 App name: $AppName" "CONFIG"
    Write-Runtime "⏰ Check interval: $CheckInterval seconds" "CONFIG"
    Write-Runtime "📄 Runtime log: $RuntimeLog" "CONFIG"
    Write-Runtime "💥 Crash log: $CrashLog" "CONFIG"
    
    # Check crash history first
    if (-not (Track-CrashHistory)) {
        Write-Runtime "🛑 Stopping due to excessive crashes" "STOP"
        exit 1
    }
    
    # Monitor existing application or start new one
    $foundExisting = Monitor-ExistingApplication
    
    if (-not $foundExisting) {
        Write-Runtime "🚀 No existing application found, starting new one..." "LAUNCH_NEW"
        
        while ($true) {
            $exitCode = Start-ApplicationWithMonitoring
            
            if ($exitCode -eq 0) {
                Write-Runtime "✅ Application exited normally" "EXIT_NORMAL"
                break
            } else {
                Write-Runtime "💥 Application crashed with exit code $exitCode" "CRASH"
                
                # Check if we should restart
                if (-not (Track-CrashHistory)) {
                    Write-Runtime "🛑 Too many crashes, stopping restart attempts" "STOP_RESTART"
                    break
                }
                
                Write-Runtime "🔄 Restarting application in 5 seconds..." "RESTART"
                Start-Sleep -Seconds 5
            }
        }
    }
    
    Write-Runtime "✅ Runtime monitoring completed" "COMPLETE"
    
} catch {
    Write-Runtime "💥 MONITOR CRASHED: $_" "CRASH"
    exit 1
}