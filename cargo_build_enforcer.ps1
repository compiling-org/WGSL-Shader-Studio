# Cargo Build Enforcer - Extreme Measures
# This script enforces proper cargo build discipline and prevents psychotic behavior

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("release", "dev")]
    [string]$Mode,
    
    [Parameter(Mandatory=$false)]
    [string]$Target = "",
    
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$global:BuildViolations = 0
$global:MaxViolations = 1

function Write-EnforcerLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Write-Host $logEntry -ForegroundColor $(if($Level -eq "ERROR"){"Red"}elseif($Level -eq "WARN"){"Yellow"}else{"Green"})
    
    if ($Level -eq "ERROR") {
        $global:BuildViolations++
        if ($global:BuildViolations -ge $global:MaxViolations) {
            Write-EnforcerLog "MAXIMUM VIOLATIONS REACHED - TERMINATING BUILD" "CRITICAL"
            exit 1
        }
    }
}

function Test-CompilationIntegrity {
    Write-EnforcerLog "Testing compilation integrity..."
    
    # Check for common psychotic patterns
    $psychoticPatterns = @(
        "unreachable!",
        "panic!.*WGPU GPU RENDERING FAILED",  # GPU panic without proper handling
        "// TODO.*psychotic",  # Psychotic TODOs
        "loop.*true.*//.*infinite"  # Infinite loops
    )
    
    foreach ($pattern in $psychoticPatterns) {
        $matches = Get-ChildItem -Path "src" -Filter "*.rs" -Recurse | 
            Select-String -Pattern $pattern -List
        
        if ($matches) {
            Write-EnforcerLog "PSYCHOTIC PATTERN DETECTED: $pattern in $($matches[0].FileName):$($matches[0].LineNumber)" "ERROR"
        }
    }
}

function Invoke-ProperCargoBuild {
    param([string]$BuildMode)
    
    Write-EnforcerLog "Starting PROPER cargo build in $BuildMode mode"
    
    # Pre-build validation
    Test-CompilationIntegrity
    
    # Set proper environment variables
    $env:RUST_BACKTRACE = "1"
    $env:CARGO_TERM_COLOR = "always"
    
    if ($BuildMode -eq "release") {
        Write-EnforcerLog "Building in RELEASE mode - optimized for production"
        $buildArgs = @("build", "--release")
    } else {
        Write-EnforcerLog "Building in DEV mode - for development testing"
        $buildArgs = @("build")
    }
    
    if ($Target) {
        $buildArgs += @("--target", $Target)
    }
    
    Write-EnforcerLog "Executing: cargo $buildArgs"
    
    try {
        $process = Start-Process -FilePath "cargo" -ArgumentList $buildArgs -NoNewWindow -PassThru -Wait
        
        if ($process.ExitCode -ne 0) {
            Write-EnforcerLog "BUILD FAILED with exit code $($process.ExitCode)" "ERROR"
            
            # Analyze the failure
            Write-EnforcerLog "Analyzing build failure..."
            
            # Check for specific error patterns
            $errorPatterns = @{
                "unreachable_code" = "Unreachable code detected - remove after panic statements"
                "pattern_matching" = "Pattern matching error - check enum variants"
                "type_mismatch" = "Type mismatch - check function signatures"
                "missing_field" = "Missing field in struct initialization"
                "unresolved_import" = "Unresolved import - check module paths"
            }
            
            foreach ($pattern in $errorPatterns.GetEnumerator()) {
                $logContent = Get-Content -Path "Cargo.lock" -ErrorAction SilentlyContinue
                if ($logContent -match $pattern.Key) {
                    Write-EnforcerLog "DETAILED ANALYSIS: $($pattern.Value)" "WARN"
                }
            }
            
            return $false
        } else {
            Write-EnforcerLog "BUILD SUCCESSFUL in $BuildMode mode" "SUCCESS"
            return $true
        }
    }
    catch {
        Write-EnforcerLog "BUILD EXCEPTION: $($_.Exception.Message)" "ERROR"
        return $false
    }
}

function Test-BuildArtifacts {
    param([string]$BuildMode)
    
    Write-EnforcerLog "Validating build artifacts..."
    
    $targetDir = if ($BuildMode -eq "release") { "target\release" } else { "target\debug" }
    
    $requiredBinaries = @(
        "isf-shaders.exe",
        "ui-analyzer-enhanced.exe",
        "ui-analyzer.exe"
    )
    
    foreach ($binary in $requiredBinaries) {
        $binaryPath = Join-Path $targetDir $binary
        if (Test-Path $binaryPath) {
            Write-EnforcerLog "✅ Binary found: $binary"
            
            # Test binary execution
            try {
                $testProcess = Start-Process -FilePath $binaryPath -ArgumentList @("--help") -NoNewWindow -PassThru -Wait -Timeout 5
                Write-EnforcerLog "✅ Binary executable: $binary"
            }
            catch {
                Write-EnforcerLog "❌ Binary execution failed: $binary - $($_.Exception.Message)" "WARN"
            }
        } else {
            Write-EnforcerLog "❌ Missing binary: $binary" "WARN"
        }
    }
}

# MAIN EXECUTION
Write-EnforcerLog "=== CARGO BUILD ENFORCER ACTIVATED ==="
Write-EnforcerLog "Mode: $Mode"
Write-EnforcerLog "Force: $Force"
Write-EnforcerLog "Max Violations: $global:MaxViolations"

# Validate environment
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-EnforcerLog "CRITICAL: Cargo not found in PATH" "ERROR"
    exit 1
}

# Execute proper build
$success = Invoke-ProperCargoBuild -BuildMode $Mode

if ($success) {
    Test-BuildArtifacts -BuildMode $Mode
    Write-EnforcerLog "=== BUILD ENFORCEMENT COMPLETED SUCCESSFULLY ==="
} else {
    Write-EnforcerLog "=== BUILD ENFORCEMENT FAILED - PSYCHOTIC BEHAVIOR DETECTED ===" "ERROR"
    exit 1
}

Write-EnforcerLog "Final violation count: $global:BuildViolations"

if ($global:BuildViolations -gt 0) {
    Write-EnforcerLog "Build completed with warnings - review violations above" "WARN"
}

exit 0