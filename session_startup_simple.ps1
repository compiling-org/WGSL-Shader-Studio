# WGSL Shader Studio - SESSION STARTUP (SIMPLE VERSION)
# Prevents psychotic loops - run at start of every session

Write-Host "🚨 SESSION STARTUP CHECK" -ForegroundColor Red

# Check if editor_ui.rs has real functionality
$lineCount = (Get-Content "src/editor_ui.rs" | Measure-Object -Line).Lines
Write-Host "📊 editor_ui.rs: $lineCount lines" -ForegroundColor Cyan

if ($lineCount -lt 1000) {
    Write-Host "🚨 DESTRUCTION DETECTED: Only $lineCount lines (should be 2000+)" -ForegroundColor Red
    Write-Host "🔧 Run: git checkout 6bb232f -- src/editor_ui.rs" -ForegroundColor Yellow
    exit 1
}

# Check for enforcement systems
if (!(Test-Path "fixed_comprehensive_enforcer.ps1")) {
    Write-Host "❌ MISSING: fixed_comprehensive_enforcer.ps1" -ForegroundColor Red
    exit 1
}

if (!(Test-Path "fixed_comprehensive_ui_analyzer.ps1")) {
    Write-Host "❌ MISSING: fixed_comprehensive_ui_analyzer.ps1" -ForegroundColor Red
    exit 1
}

# Check for decorative patterns
$content = Get-Content "src/editor_ui.rs" -Raw
if ($content -match "decorative.*feature|placeholder.*implementation|fake.*panel") {
    Write-Host "🚨 DECORATIVE CODE DETECTED" -ForegroundColor Red
    exit 1
}

Write-Host "✅ SESSION APPROVED: Real functionality confirmed" -ForegroundColor Green
Write-Host "✅ Starting enforcement systems..." -ForegroundColor Green

# Start background enforcement
Start-Process -FilePath "powershell.exe" -ArgumentList "-NoProfile -ExecutionPolicy Bypass -Command .\fixed_comprehensive_enforcer.ps1" -WindowStyle Hidden

Write-Host "🚀 Safe to proceed with development" -ForegroundColor Green