param(
  [int]$CheckInterval = 10
)

$ErrorActionPreference = "Stop"

Write-Host "🛡️ SIMPLE DISCIPLINARY ENFORCEMENT SYSTEM ACTIVE" -ForegroundColor Green
Write-Host "Check Interval: $CheckInterval seconds" -ForegroundColor Cyan

# Function to check for eframe violations
function Check-EframeViolations {
  $eframeFiles = Get-ChildItem -Path "src" -Include "*.rs" -Recurse | Select-String -Pattern "eframe" -List | Select-Object Path
  if ($eframeFiles) {
    Write-Host "🚨 EFAME VIOLATION DETECTED!" -ForegroundColor Red
    foreach ($file in $eframeFiles) {
      Write-Host "  File: $($file.Path)" -ForegroundColor Yellow
    }
    Write-Host "TERMINATING: EFAME USAGE VIOLATES BEVY ARCHITECTURE" -ForegroundColor Red
    exit 1
  }
}

# Function to check for proper imports
function Check-ProperImports {
  $bevyAppContent = Get-Content "src/bevy_app.rs" -Raw
  if (-not $bevyAppContent.Contains("bevy_egui")) {
    Write-Host "🚨 BEVY_EGUI IMPORT VIOLATION DETECTED!" -ForegroundColor Red
    Write-Host "TERMINATING: BEVY_EGUI NOT FOUND IN BEVY_APP.RS" -ForegroundColor Red
    exit 1
  }
}

try {
  while ($true) {
    Write-Host "." -NoNewline -ForegroundColor Gray
    
    # Check for violations
    Check-EframeViolations
    Check-ProperImports
    
    Start-Sleep -Seconds $CheckInterval
  }
}
catch {
  Write-Host "❌ ENFORCEMENT SYSTEM ERROR: $_" -ForegroundColor Red
  exit 1
}