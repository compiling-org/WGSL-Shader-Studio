# WGSL Shader Studio - SESSION STARTUP ENFORCER
# PREVENTS PSYCHOTIC LOOPS IN EVERY NEW SESSION
# MUST be run at the start of every development session

param(
    [switch]$Force = $false
)

$ErrorActionPreference = "Stop"

Write-Host "🚨 SESSION STARTUP ENFORCER ACTIVE" -ForegroundColor Red
Write-Host "🔍 Preventing psychotic development loops..." -ForegroundColor Yellow

# CRITICAL: Check for psychotic patterns BEFORE allowing development
function Test-PsychoticState {
    $violations = @()
    
    # Check if comprehensive enforcer exists and is working
    if (!(Test-Path "comprehensive_enforcer.ps1")) {
        $violations += "MISSING: comprehensive_enforcer.ps1"
    }
    
    # Check for decorative vs real functionality
    if (Test-Path "src/editor_ui.rs") {
        $content = Get-Content "src/editor_ui.rs" -Raw
        $lineCount = ($content -split "`n").Count
        
        if ($lineCount -lt 1000) {
            $violations += "DESTRUCTION DETECTED: editor_ui.rs only $lineCount lines (should be 2000+)"
        }
        
        if ($content -match "decorative.*feature|placeholder.*implementation|fake.*panel") {
            $violations += "DECORATIVE CODE DETECTED: Found simulation patterns"
        }
        
        # Check if real systems are hidden
        $realSystems = @(
            @{Pattern = "show_node_studio.*false"; System = "Node Graph"},
            @{Pattern = "show_timeline.*false"; System = "Timeline"},
            @{Pattern = "show_audio_panel.*false"; System = "Audio System"},
            @{Pattern = "show_gesture_panel.*false"; System = "Gesture Control"}
        )
        
        foreach ($system in $realSystems) {
            if ($content -match $system.Pattern) {
                $violations += "SYSTEM HIDDEN: $($system.System) disabled in UI"
            }
        }
    }
    
    # Check for duplicate files (psychotic pattern)
    $duplicatePatterns = @("*backup*", "*test*", "*simple*", "*demo*", "*new*", "*fixed*", "*audited*")
    foreach ($pattern in $duplicatePatterns) {
        $files = Get-ChildItem -Path . -Name $pattern
        if ($files.Count -gt 0) {
            $violations += "DUPLICATE FILES: $($files -join ', ')"
        }
    }
    
    return $violations
}

# Check for documentation compliance
function Test-DocumentationCompliance {
    $violations = @()
    
    $requiredDocs = @(
        ".trae\documents\DISCIPLINARY_RULES.md",
        "HONEST_RECOVERY_PLAN.md",
        "COMPLETE_FEATURE_AUDIT.md"
    )
    
    foreach ($doc in $requiredDocs) {
        if (!(Test-Path $doc)) {
            $violations += "MISSING DOCUMENTATION: $doc"
        }
    }
    
    return $violations
}

# SESSION STARTUP CHECKS
Write-Host "📋 Running session startup checks..." -ForegroundColor Cyan

$psychoticViolations = Test-PsychoticState
$docViolations = Test-DocumentationCompliance

$totalViolations = @($psychoticViolations).Count + @($docViolations).Count

if ($totalViolations -gt 0) {
    Write-Host "🚨 SESSION BLOCKED: Psychotic state detected!" -ForegroundColor Red
    Write-Host "❌ Violations found:" -ForegroundColor Red
    
    foreach ($violation in @($psychoticViolations)) {
        Write-Host "  • $violation" -ForegroundColor Red
    }
    foreach ($violation in @($docViolations)) {
        Write-Host "  • $violation" -ForegroundColor Red
    }
    
    Write-Host "🔧 REQUIRED ACTIONS:" -ForegroundColor Yellow
    Write-Host "1. Run: .\comprehensive_enforcer.ps1" -ForegroundColor White
    Write-Host "2. Fix all violations before continuing" -ForegroundColor White
    Write-Host "3. Verify real functionality is restored" -ForegroundColor White
    Write-Host "4. Run session startup enforcer again" -ForegroundColor White
    
    if (!$Force) {
        Write-Host "`n🚫 SESSION TERMINATED - No development allowed until violations fixed" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "`n⚠️  WARNING: Forcing session start with violations present" -ForegroundColor Yellow
    }
} else {
    Write-Host "✅ SESSION APPROVED: No psychotic patterns detected" -ForegroundColor Green
    Write-Host "✅ Documentation compliance verified" -ForegroundColor Green
    Write-Host "✅ Real functionality confirmed" -ForegroundColor Green
    Write-Host "`n🚀 Safe to proceed with development" -ForegroundColor Green
    
    # Start the enforcement systems
    Write-Host "🔒 Starting enforcement systems..." -ForegroundColor Cyan
    
    # Start comprehensive enforcer in background
    Start-Process -FilePath "powershell.exe" -ArgumentList "-ExecutionPolicy Bypass -File .\comprehensive_enforcer.ps1" -WindowStyle Hidden
    
    Write-Host "✅ Enforcement systems active - development protected" -ForegroundColor Green
}