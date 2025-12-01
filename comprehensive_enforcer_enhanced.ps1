# WGSL Shader Studio - ULTRA COMPREHENSIVE BEHAVIORAL ENFORCEMENT SYSTEM
# This script provides hardwired behavioral regulation to prevent psychotic development loops
# Based on disciplinary rules, comprehensive documentation, and UI audit requirements

param(
    [int]$CheckInterval = 2,  # Check every 2 seconds - more aggressive monitoring
    [int]$MaxViolations = 1,   # SINGLE violation before intervention - zero tolerance
    [string]$LogFile = "enforcement.log",
    [switch]$UltraStrictMode = $true,
    [switch]$DocumentationMode = $true  # Monitor against documentation violations
)

$ErrorActionPreference = "Stop"
$violations = 0
$startTime = Get-Date
$enforcementActive = $true
$lastDocumentationCheck = Get-Date
$comprehensiveFeatures = @()
$systematicProgress = @{}

# Load comprehensive documentation requirements
function Load-ComprehensiveRequirements {
    try {
        $docs = Get-Content ".trae\documents\comprehensive_work_documentation.md" -Raw
        
        # Extract Phase 1 requirements
        if ($docs -match "PHASE 1: CRITICAL FOUNDATION.*?PHASE 2") {
            $phase1 = $matches[0]
            # Extract specific tasks
            if ($phase1 -match "1\.1 GPU-Only Enforcement.*?1\.2 Three-Panel Layout") {
                $comprehensiveFeatures += "GPU-Only Enforcement"
                $comprehensiveFeatures += "Three-Panel Layout"
            }
            if ($phase1 -match "1\.2 Three-Panel Layout.*?1\.3 Basic Shader Compilation") {
                $comprehensiveFeatures += "Three-Panel Layout"
                $comprehensiveFeatures += "Basic Shader Compilation"
            }
            if ($phase1 -match "1\.3 Basic Shader Compilation.*?1\.4 Error Handling") {
                $comprehensiveFeatures += "Basic Shader Compilation"
                $comprehensiveFeatures += "Error Handling System"
            }
        }
        
        # Extract UI audit requirements
        $uiAudit = Get-Content "UI_AUDIT_REPORT.md" -Raw -ErrorAction SilentlyContinue
        if ($uiAudit) {
            if ($uiAudit -match "CRITICAL MISSING FEATURES.*?CRITICAL BROKEN FEATURES") {
                $criticalSection = $matches[0]
                if ($criticalSection -match "Live Shader Preview") {
                    $comprehensiveFeatures += "Live Shader Preview"
                }
                if ($criticalSection -match "Shader Browser Panel") {
                    $comprehensiveFeatures += "Shader Browser Panel"
                }
                if ($criticalSection -match "Parameter Panel") {
                    $comprehensiveFeatures += "Parameter Panel"
                }
                if ($criticalSection -match "Shader Compilation") {
                    $comprehensiveFeatures += "Shader Compilation"
                }
            }
        }
        
        Write-EnforcementLog "Loaded $($comprehensiveFeatures.Count) comprehensive requirements" "INFO"
    }
    catch {
        Write-EnforcementLog "Failed to load comprehensive requirements: $($_.Exception.Message)" "WARN"
    }
}

function Write-EnforcementLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [ENFORCEMENT] [$Level] $Message"
    Write-Host $logEntry -ForegroundColor $(if($Level -eq "ERROR"){"Red"} elseif($Level -eq "WARN"){"Yellow"} else{"Green"})
    Add-Content -Path $LogFile -Value $logEntry
}

function Test-PsychoticBehavior {
    # Check for psychotic development patterns
    $psychoticPatterns = @(
        "minimal.*test",
        "simple.*gui",
        "test.*window",
        "basic.*test",
        "prototype.*ui"
    )
    
    $allFiles = Get-ChildItem -Path "." -Filter "*.rs" -Recurse
    foreach ($file in $allFiles) {
        $content = Get-Content $file.FullName -Raw
        foreach ($pattern in $psychoticPatterns) {
            if ($content -match $pattern) {
                Write-EnforcementLog "PSYCHOTIC VIOLATION: Creating test files instead of real features in $($file.Name)" "ERROR"
                return $true
            }
        }
    }
    return $false
}

function Test-DocumentationViolation {
    # Check if work deviates from comprehensive documentation
    if ($DocumentationMode -and (Get-Date) -gt $lastDocumentationCheck.AddMinutes(5)) {
        $lastDocumentationCheck = Get-Date
        
        # Check if we're following Phase 1 requirements
        $currentPhase = Get-CurrentPhase
        $requiredFeatures = Get-PhaseRequirements $currentPhase
        
        foreach ($feature in $requiredFeatures) {
            if (-not (Test-FeatureImplemented $feature)) {
                Write-EnforcementLog "DOCUMENTATION VIOLATION: Required feature '$feature' not implemented per Phase $currentPhase" "ERROR"
                return $true
            }
        }
    }
    return $false
}

function Get-CurrentPhase {
    # Determine current development phase based on implemented features
    $gpuOnlyImplemented = Test-GPUOnlyEnforcement
    $threePanelImplemented = Test-ThreePanelLayout
    $compilationImplemented = Test-ShaderCompilation
    
    if (-not $gpuOnlyImplemented) { return 1 }
    if (-not $threePanelImplemented) { return 1 }
    if (-not $compilationImplemented) { return 1 }
    
    return 2  # Move to Phase 2 after Phase 1 complete
}

function Get-PhaseRequirements {
    param([int]$Phase)
    
    switch ($Phase) {
        1 { return @("GPU-Only Enforcement", "Three-Panel Layout", "Basic Shader Compilation", "Error Handling System") }
        2 { return @("Live Shader Preview", "Shader Browser Panel", "Parameter Panel", "ISF Support") }
        3 { return @("Node-Based Editor", "Audio/MIDI Integration", "Performance Monitoring", "File Dialogs") }
        default { return @() }
    }
}

function Test-FeatureImplemented {
    param([string]$Feature)
    
    switch ($Feature) {
        "GPU-Only Enforcement" { return Test-GPUOnlyEnforcement }
        "Three-Panel Layout" { return Test-ThreePanelLayout }
        "Basic Shader Compilation" { return Test-ShaderCompilation }
        "Error Handling System" { return Test-ErrorHandlingSystem }
        "Live Shader Preview" { return Test-LiveShaderPreview }
        "Shader Browser Panel" { return Test-ShaderBrowserPanel }
        "Parameter Panel" { return Test-ParameterPanel }
        default { return $false }
    }
}

function Test-GPUOnlyEnforcement {
    # Check if GPU-only enforcement is properly implemented
    $editorUI = Get-Content "src/editor_ui.rs" -Raw -ErrorAction SilentlyContinue
    if ($editorUI -and $editorUI -match "NO SOFTWARE FALLBACK ALLOWED" -and $editorUI -match "GPU-only enforcement active") {
        return $true
    }
    return $false
}

function Test-ThreePanelLayout {
    # Check if three-panel layout is implemented
    $bevyApp = Get-Content "src/bevy_app.rs" -Raw -ErrorAction SilentlyContinue
    if ($bevyApp -and $bevyApp -match "draw_editor_central_panel") {
        return $true
    }
    return $false
}

function Test-ShaderCompilation {
    # Check if shader compilation is working
    $shaderModule = Get-Content "src/shader_module_system.rs" -Raw -ErrorAction SilentlyContinue
    if ($shaderModule -and $shaderModule -match "compile.*shader" -and $shaderModule -match "ModuleSystemError") {
        return $true
    }
    return $false
}

function Test-ErrorHandlingSystem {
    # Check if error handling system is implemented
    $errorFiles = @("src/error_handling.rs", "src/errors.rs", "src/error_system.rs")
    foreach ($file in $errorFiles) {
        if (Test-Path $file) {
            $content = Get-Content $file -Raw
            if ($content -match "user.*notification|error.*display|graceful.*recovery") {
                return $true
            }
        }
    }
    return $false
}

function Test-LiveShaderPreview {
    # Check if live shader preview is implemented
    $previewFiles = @("src/shader_preview.rs", "src/preview.rs", "src/live_preview.rs")
    foreach ($file in $previewFiles) {
        if (Test-Path $file) {
            return $true
        }
    }
    return $false
}

function Test-ShaderBrowserPanel {
    # Check if shader browser panel is implemented
    $browserFiles = @("src/shader_browser.rs", "src/browser.rs", "src/isf_browser.rs")
    foreach ($file in $browserFiles) {
        if (Test-Path $file) {
            return $true
        }
    }
    return $false
}

function Test-ParameterPanel {
    # Check if parameter panel is implemented
    $paramFiles = @("src/parameter_panel.rs", "src/controls.rs", "src/parameters.rs")
    foreach ($file in $paramFiles) {
        if (Test-Path $file) {
            return $true
        }
    }
    return $false
}

function Test-SystematicProgress {
    # Check if we're making systematic progress
    $currentTime = Get-Date
    $timeSinceStart = ($currentTime - $startTime).TotalMinutes
    
    if ($timeSinceStart -gt 30) {  # After 30 minutes
        $phase1Complete = (Get-CurrentPhase) -gt 1
        if (-not $phase1Complete) {
            Write-EnforcementLog "SYSTEMATIC VIOLATION: Phase 1 not completed after 30 minutes - stuck in psychotic loop" "ERROR"
            return $false
        }
    }
    
    return $true
}

function Test-BackendSystemWiring {
    # Check if backend systems are being wired to UI properly
    $backendSystems = @(
        "src/audio_system.rs",
        "src/isf_integration.rs", 
        "src/node_graph.rs",
        "src/performance_monitor.rs",
        "src/file_dialogs.rs",
        "src/menu_system.rs"
    )
    
    $wiredSystems = 0
    foreach ($system in $backendSystems) {
        if (Test-Path $system) {
            $content = Get-Content $system -Raw
            # Check if system has UI integration
            if ($content -match "ui.*integration|panel.*render|draw.*ui|egui") {
                $wiredSystems++
            }
        }
    }
    
    if ($wiredSystems -lt 3 -and (Get-Date) -gt $startTime.AddMinutes(45)) {
        Write-EnforcementLog "BACKEND VIOLATION: Only $wiredSystems backend systems wired to UI after 45 minutes" "ERROR"
        return $true
    }
    
    return $false
}

function Test-LoopBehavior {
    # Enhanced loop detection with stricter criteria
    $recentCompilations = Get-Content $LogFile -ErrorAction SilentlyContinue | Where-Object { 
        $_ -match "cargo build|cargo check|compiling" -and $_.Length -gt 0 
    } | Select-Object -Last 10
    
    if ($recentCompilations.Count -ge 8) {
        $timePattern = $recentCompilations | ForEach-Object { 
            if ($_ -match "(\d{2}:\d{2}:\d{2})") { $matches[1] } 
        }
        
        if ($timePattern.Count -ge 6) {
            Write-EnforcementLog "LOOP VIOLATION: Excessive compilation pattern detected - likely stuck in error loop" "ERROR"
            return $true
        }
    }
    return $false
}

function Test-TestBinaryCompilation {
    # Enhanced test binary detection
    $testFiles = @(
        "minimal_test.rs", "test_window.rs", "gui_test.rs", "simple_test.rs", 
        "test_gui.rs", "basic_test.rs", "prototype_test.rs", "ui_test.rs",
        "preview_test.rs", "shader_test.rs", "app_test.rs"
    )
    foreach ($file in $testFiles) {
        if (Test-Path $file) {
            Write-EnforcementLog "VIOLATION: Test binary file detected: $file - STOP CREATING TESTS" "ERROR"
            return $true
        }
    }
    return $false
}

function Test-FeatureReplacement {
    # Enhanced feature replacement detection
    $dangerousPatterns = @(
        "// TODO: Replace with real implementation",
        "// Placeholder for actual feature", 
        "// Stub implementation",
        "unimplemented!()",
        "todo!()",
        "// FIXME: Implement properly",
        "// HACK: Temporary solution",
        "// TEMP: Will fix later"
    )
    
    $sourceFiles = Get-ChildItem -Path "src" -Filter "*.rs" -Recurse
    foreach ($file in $sourceFiles) {
        $content = Get-Content $file.FullName -Raw
        foreach ($pattern in $dangerousPatterns) {
            if ($content -match $pattern) {
                Write-EnforcementLog "VIOLATION: Feature replacement pattern detected in $($file.Name)" "ERROR"
                return $true
            }
        }
    }
    return $false
}

function Test-DocumentationOverCode {
    # Enhanced documentation over code detection
    $docFiles = Get-ChildItem -Path "." -Filter "*.md" -Recurse | Where-Object { $_.LastWriteTime -gt (Get-Date).AddMinutes(-5) }
    if ($docFiles.Count -gt 1) {
        Write-EnforcementLog "VIOLATION: Excessive documentation creation detected ($($docFiles.Count) files in 5 minutes)" "ERROR"
        return $true
    }
    return $false
}

function Test-BackendNeglect {
    # Enhanced backend neglect detection
    $backendFiles = @(
        "src/audio_system.rs", "src/isf_integration.rs", "src/timeline.rs", 
        "src/node_graph.rs", "src/performance_monitor.rs", "src/file_dialogs.rs",
        "src/menu_system.rs", "src/error_handling.rs", "src/shader_transpiler.rs"
    )
    $neglectedFiles = 0
    
    foreach ($file in $backendFiles) {
        if (Test-Path $file) {
            $lastWrite = (Get-Item $file).LastWriteTime
            if ($lastWrite -lt (Get-Date).AddHours(-1)) {
                $neglectedFiles++
            }
        }
    }
    
    if ($neglectedFiles -ge 5) {
        Write-EnforcementLog "VIOLATION: Backend systems severely neglected ($neglectedFiles files unchanged for 1+ hours)" "ERROR"
        return $true
    }
    return $false
}

function Invoke-EnforcementAction {
    param([string]$ViolationType)
    
    $script:violations++
    $violationCount = $script:violations
    Write-EnforcementLog "ENFORCEMENT VIOLATION ${violationCount}/${MaxViolations}: ${ViolationType}" "ERROR"
    
    if ($script:violations -ge $MaxViolations) {
        Write-EnforcementLog "MAXIMUM VIOLATIONS REACHED - TAKING IMMEDIATE DRASTIC ACTION" "ERROR"
        Write-EnforcementLog "ENFORCER: STOP ALL PSYCHOTIC BEHAVIOR NOW" "ERROR"
        Write-EnforcementLog "ENFORCER: FOCUS ON SYSTEMATIC BACKEND WIRING PER DOCUMENTATION" "ERROR"
        
        # Kill all development processes
        Get-Process | Where-Object { 
            $_.ProcessName -match "cargo|rustc|test|minimal|simple|gui" 
        } | Stop-Process -Force -ErrorAction SilentlyContinue
        
        # Remove all test files
        Remove-Item -Path "minimal_test.rs", "minimal_gui_test.rs", "test_window.rs", "simple_test.rs", "gui_test.rs", "basic_test.rs", "prototype_test.rs", "ui_test.rs", "preview_test.rs", "shader_test.rs", "app_test.rs" -ErrorAction SilentlyContinue
        
        Write-EnforcementLog "DRASTIC MEASURE: All processes terminated and test files removed" "ERROR"
        Write-EnforcementLog "ENFORCER: You MUST now systematically wire backend systems to UI" "ERROR"
        Write-EnforcementLog "ENFORCER: Follow Phase 1 requirements: GPU-only, Three-panel, Compilation, Error handling" "ERROR"
        
        # Show current requirements
        $currentPhase = Get-CurrentPhase
        $requirements = Get-PhaseRequirements $currentPhase
        Write-EnforcementLog "CURRENT PHASE $currentPhase REQUIREMENTS: $($requirements -join ', ')" "ERROR"
        
        # Reset violations but keep system in enforcement mode
        $script:violations = 0
        
        # Force restart of enforcement with increased strictness
        Start-Sleep -Seconds 5
        & $PSCommandPath @PSBoundParameters
    }
}

function Start-EnforcementMonitoring {
    Write-EnforcementLog "=== ULTRA COMPREHENSIVE ENFORCEMENT SYSTEM STARTED ==="
    Write-EnforcementLog "Ultra Strict Mode: $UltraStrictMode"
    Write-EnforcementLog "Check interval: $CheckInterval seconds"
    Write-EnforcementLog "Max violations before intervention: $MaxViolations (ZERO TOLERANCE)"
    Write-EnforcementLog "Documentation Enforcement: $DocumentationMode"
    
    # Load comprehensive requirements
    Load-ComprehensiveRequirements
    
    if ($UltraStrictMode) {
        Write-EnforcementLog "ULTRA STRICT MODE: ANY PSYCHOTIC BEHAVIOR = IMMEDIATE INTERVENTION"
        Write-EnforcementLog "ULTRA STRICT MODE: SYSTEMATIC PROGRESS REQUIRED PER DOCUMENTATION"
        Write-EnforcementLog "ULTRA STRICT MODE: ALL TEST BINARIES PROHIBITED"
        Write-EnforcementLog "ULTRA STRICT MODE: BACKEND WIRING MANDATORY"
    }
    
    Write-EnforcementLog "CURRENT PHASE: $(Get-CurrentPhase)"
    Write-EnforcementLog "REQUIRED FEATURES: $(Get-PhaseRequirements (Get-CurrentPhase) -join ', ')"
    
    try {
        while ($enforcementActive) {
            $violationDetected = $false
            
            # Ultra-comprehensive violation testing
            if (Test-PsychoticBehavior) { $violationDetected = $true }
            if (Test-DocumentationViolation) { $violationDetected = $true }
            if (Test-SystematicProgress) { $violationDetected = $true }
            if (Test-BackendSystemWiring) { $violationDetected = $true }
            if (Test-TestBinaryCompilation) { $violationDetected = $true }
            if (Test-FeatureReplacement) { $violationDetected = $true }
            if (Test-DocumentationOverCode) { $violationDetected = $true }
            if (Test-LoopBehavior) { $violationDetected = $true }
            if (Test-BackendNeglect) { $violationDetected = $true }
            
            if ($violationDetected) {
                Invoke-EnforcementAction("Multiple comprehensive violations detected")
            } else {
                # Reset violations when systematic progress is detected
                if ($script:violations -gt 0) {
                    Write-EnforcementLog "Systematic progress detected - resetting violation count" "INFO"
                    $script:violations = 0
                }
            }
            
            Start-Sleep -Seconds $CheckInterval
        }
    }
    catch {
        Write-EnforcementLog "Enforcement system crashed: $($_.Exception.Message)" "ERROR"
        Write-EnforcementLog "Restarting enforcement in 5 seconds..." "ERROR"
        Start-Sleep -Seconds 5
        & $PSCommandPath @PSBoundParameters
    }
}

# Start the ultra-comprehensive enforcement system
Start-EnforcementMonitoring