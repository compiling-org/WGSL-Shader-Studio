
# LIVE APP MONITOR V2
# Directly tails app_log.txt for Heartbeat and Input diagnostics.
# Bypass complex JSON parsing if the file isn't being written.
# Usage: .\live_app_tracker.ps1

$LogFile = "app_log.txt"

Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "  LIVE LOG MONITOR (Tailing $LogFile)" -ForegroundColor White
Write-Host "  Watching for 'Heartbeat' and 'Input'..." -ForegroundColor Gray
Write-Host "======================================================================" -ForegroundColor Cyan

if (-not (Test-Path $LogFile)) {
    Write-Host "Log file not found yet. Waiting..." -ForegroundColor Yellow
}

while (-not (Test-Path $LogFile)) { Start-Sleep -Seconds 1 }

Get-Content -Path $LogFile -Wait -Tail 20 | ForEach-Object {
    $line = $_
    if ($line -match "Heartbeat: Frame (\d+)") {
        $frame = $matches[1]
        Write-Host "💓 Frame $frame" -ForegroundColor Green
    }
    elseif ($line -match "Input: No mouse position") {
        Write-Host "🖱️  Input: NO MOUSE (Disconnected)" -ForegroundColor Red
    }
    elseif ($line -match "Input: Mouse at (.*)") {
        Write-Host "🖱️  Input: ACTIVE at $($matches[1])" -ForegroundColor Cyan
    }
    elseif ($line -match "panicked at") {
        Write-Host "🔥 PANIC DETECTED: $line" -ForegroundColor Red -BackgroundColor Black
    }
    else {
        # Print other relevant lines in dark gray
        if ($line.Trim().Length -gt 0) {
            Write-Host $line -ForegroundColor DarkGray
        }
    }
}