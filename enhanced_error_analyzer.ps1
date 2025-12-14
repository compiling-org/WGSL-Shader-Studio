# ENHANCED ERROR ANALYZER - DEEP COMPILATION ERROR TRACKING
# Analyzes compilation errors in detail and provides actionable insights

param(
    [string]$ErrorLog = "live_errors.log",
    [string]$AnalysisLog = "error_analysis.log",
    [int]$MaxErrorsPerType = 10
)

function Write-Analysis {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Write-Host $logEntry
    Add-Content -Path $AnalysisLog -Value $logEntry
}

function Analyze-CompilationErrors {
    Write-Analysis "🔍 Starting deep error analysis..." "START"
    
    # Run cargo check and capture full output
    $output = cargo check 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Analysis "✅ No compilation errors found" "SUCCESS"
        return
    }
    
    Write-Analysis "🚨 Found compilation errors (exit code: $exitCode)" "ERROR"
    
    # Parse errors by category
    $errorCategories = @{
        "E0061" = "Method Argument Mismatch"
        "E0277" = "Trait Bound Not Satisfied" 
        "E0308" = "Type Mismatch"
        "E0502" = "Borrow Checker Error"
        "E0599" = "Method Not Found"
        "E0063" = "Missing Fields"
        "E0425" = "Unresolved Name"
        "E053" = "Pattern Matching Error"
    }
    
    $errorSummary = @{}
    $errorDetails = @()
    
    foreach ($line in $output) {
        # Extract error codes and file locations
        if ($line -match "error\[([E0-9]+)\]: (.+)") {
            $errorCode = $matches[1]
            $errorMessage = $matches[2]
            
            if (-not $errorSummary.ContainsKey($errorCode)) {
                $errorSummary[$errorCode] = @{
                    Count = 0
                    Message = $errorMessage
                    Category = if ($errorCategories.ContainsKey($errorCode)) { $errorCategories[$errorCode] } else { "Unknown" }
                    Files = @()
                }
            }
            $errorSummary[$errorCode].Count++
        }
        
        # Extract file locations
        if ($line -match "--> src\\(.+):(\d+):(\d+)") {
            $file = $matches[1]
            $lineNum = $matches[2]
            $colNum = $matches[3]
            
            if ($errorSummary.Count -gt 0) {
                $lastError = $errorSummary.Keys | Select-Object -Last 1
                $errorSummary[$lastError].Files += @{
                    File = $file
                    Line = $lineNum
                    Column = $colNum
                }
            }
        }
        
        # Store full error details
        if ($line -match "error\[") {
            $currentError = $line
            $errorDetails += $currentError
        }
    }
    
    # Generate analysis report
    Write-Analysis "📊 ERROR ANALYSIS REPORT" "REPORT"
    Write-Analysis "Total Error Types: $($errorSummary.Count)" "SUMMARY"
    Write-Analysis "Total Error Instances: $($errorSummary.Values | Measure-Object -Property Count -Sum | Select-Object -ExpandProperty Sum)" "SUMMARY"
    
    foreach ($errorType in $errorSummary.GetEnumerator() | Sort-Object { $_.Value.Count } -Descending) {
        $code = $errorType.Key
        $info = $errorType.Value
        
        Write-Analysis "" 
        Write-Analysis "🔍 Error [$code] - $($info.Category)" "ERROR_TYPE"
        Write-Analysis "   Count: $($info.Count) instances" "DETAIL"
        Write-Analysis "   Message: $($info.Message)" "DETAIL"
        Write-Analysis "   Files affected: $($info.Files.Count)" "DETAIL"
        
        # Show top file locations
        $topFiles = $info.Files | Group-Object File | Sort-Object Count -Descending | Select-Object -First 3
        foreach ($file in $topFiles) {
            Write-Analysis "   📁 $($file.Name): $($file.Count) occurrences" "FILE"
        }
        
        # Provide specific fixes
        switch ($code) {
            "E0599" {
                Write-Analysis "   💡 FIX: Method 'colored' not found. Use 'colored_label' instead" "FIX"
                Write-Analysis "   🔧 Replace: ui.colored(...) → ui.colored_label(...)" "FIX_DETAIL"
            }
            "E0061" {
                Write-Analysis "   💡 FIX: Wrong number of arguments. Check method signature" "FIX"
                Write-Analysis "   🔧 Review the method documentation for correct parameters" "FIX_DETAIL"
            }
            "E0277" {
                Write-Analysis "   💡 FIX: Missing trait implementation. Add Serialize/Deserialize derives" "FIX"
                Write-Analysis "   🔧 Add: #[derive(Serialize, Deserialize)] to struct/enum" "FIX_DETAIL"
            }
            "E0308" {
                Write-Analysis "   💡 FIX: Type mismatch. Check expected vs actual types" "FIX"
                Write-Analysis "   🔧 Use proper type conversion or fix the type annotation" "FIX_DETAIL"
            }
        }
    }
    
    # Log detailed errors for reference
    Write-Analysis ""
    Write-Analysis "📝 DETAILED ERROR LIST:" "DETAIL_HEADER"
    foreach ($error in $errorDetails | Select-Object -Last 20) {
        Write-Analysis "   $error" "DETAIL_LINE"
    }
    
    # Save to error log
    Add-Content -Path $ErrorLog -Value "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') - Error Analysis: $($errorSummary.Count) types, $($errorSummary.Values | Measure-Object -Property Count -Sum | Select-Object -ExpandProperty Sum) total errors"
}

function Track-ErrorProgress {
    Write-Analysis "📈 Tracking error progress over time..." "PROGRESS"
    
    if (Test-Path $ErrorLog) {
        $allErrors = Get-Content $ErrorLog | Where-Object { $_ -match "Error Analysis:" }
        
        if ($allErrors.Count -gt 1) {
            Write-Analysis "📊 ERROR PROGRESSION:" "PROGRESS_CHART"
            
            $recentErrors = $allErrors | Select-Object -Last 10
            for ($i = 0; $i -lt $recentErrors.Count; $i++) {
                $error = $recentErrors[$i]
                if ($error -match '(\d+) types, (\d+) total errors') {
                    $types = $matches[1]
                    $total = $matches[2]
                    $time = $error.Substring(0, 19)
                    
                    Write-Analysis "   $time - Types: $types, Total: $total" "PROGRESS_POINT"
                }
            }
            
            # Calculate trend
            $firstError = $recentErrors[0]
            $lastError = $recentErrors[-1]
            
            if ($firstError -match '(\d+) total errors' -and $lastError -match '(\d+) total errors') {
                $firstCount = [int]$matches[1]
                $lastCount = [int]$matches[1]
                
                $trend = $lastCount - $firstCount
                if ($trend -lt 0) {
                    Write-Analysis "📉 IMPROVING: Error count decreased by $([Math]::Abs($trend))" "PROGRESS_GOOD"
                } elseif ($trend -gt 0) {
                    Write-Analysis "📈 WORSENING: Error count increased by $trend" "PROGRESS_BAD"
                } else {
                    Write-Analysis "📊 STABLE: Error count unchanged" "PROGRESS_STABLE"
                }
            }
        }
    }
}

function Monitor-BuildProcess {
    Write-Analysis "🔨 Monitoring build process..." "BUILD"
    
    # Track build time
    $startTime = Get-Date
    
    # Run cargo build with timing
    $output = cargo build 2>&1
    $exitCode = $LASTEXITCODE
    
    $endTime = Get-Date
    $buildTime = ($endTime - $startTime).TotalSeconds
    
    Write-Analysis "⏱️  Build completed in $([Math]::Round($buildTime, 2)) seconds" "BUILD_TIME"
    Write-Analysis "📊 Exit code: $exitCode" "BUILD_RESULT"
    
    if ($exitCode -ne 0) {
        # Count warnings
        $warnings = $output | Where-Object { $_ -match "warning\[" }
        Write-Analysis "⚠️  Build warnings: $($warnings.Count)" "BUILD_WARNINGS"
        
        # Check for specific build issues
        if ($output -match "out of memory") {
            Write-Analysis "🚨 MEMORY ISSUE: Build failed due to memory constraints" "BUILD_MEMORY"
        }
        if ($output -match "disk space") {
            Write-Analysis "💾 DISK ISSUE: Build failed due to disk space" "BUILD_DISK"
        }
        if ($output -match "timeout") {
            Write-Analysis "⏰ TIMEOUT: Build failed due to timeout" "BUILD_TIMEOUT"
        }
    }
}

# MAIN ANALYSIS EXECUTION
try {
    Write-Analysis "🚀 STARTING ENHANCED ERROR ANALYZER" "STARTUP"
    
    # Run comprehensive analysis
    Analyze-CompilationErrors
    Track-ErrorProgress
    Monitor-BuildProcess
    
    Write-Analysis "✅ ERROR ANALYSIS COMPLETE" "COMPLETE"
    
} catch {
    Write-Analysis "💥 ANALYZER CRASHED: $_" "CRASH"
    exit 1
}