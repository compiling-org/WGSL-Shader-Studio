# LIVE APP MONITOR
# Connects to the running WGSL Shader Studio instance by reading 'ui_audit.json'.
# Does NOT launch the app. Run the app separately!
# Usage: .\live_app_tracker.ps1

$AuditFile = "ui_audit.json"
$PanicLog = "panic_log.txt"

function Write-Header {
    param([string]$Title)
    Write-Host "======================================================================" -ForegroundColor Cyan
    Write-Host "  $Title" -ForegroundColor White
    Write-Host "======================================================================" -ForegroundColor Cyan
}

function Monitor-Loop {
    Write-Header "LIVE MONITORING - WAITING FOR APP..."
    
    while ($true) {
        Clear-Host
        Write-Header "LIVE APP DIAGNOSTICS (Ctrl+C to Stop)"
        $now = Get-Date
        Write-Host "Monitor Time: $($now.ToString('HH:mm:ss'))" -ForegroundColor Gray
        
        # 1. Check for Panic Log
        if (Test-Path $PanicLog) {
            $panicContent = Get-Content $PanicLog -Raw
            if (-not [string]::IsNullOrWhiteSpace($panicContent)) {
                 Write-Host "`nCRITICAL: APP PANIC DETECTED!" -ForegroundColor Red -BackgroundColor Black
                 Write-Host $panicContent -ForegroundColor Red
            }
        }

        # 2. Check UI Audit State
        if (Test-Path $AuditFile) {
            try {
                # Read with retry
                $jsonStr = $null
                $attempts = 0
                while ($attempts -lt 3 -and $null -eq $jsonStr) {
                    try { $jsonStr = Get-Content $AuditFile -Raw -ErrorAction Stop } catch { Start-Sleep -Milliseconds 50 }
                    $attempts++
                }

                if ($jsonStr) {
                    $json = $jsonStr | ConvertFrom-Json
                    
                    # --- APP HEALTH ---
                    $appTime = [datetimeOffset]::FromUnixTimeSeconds($json.timestamp).LocalDateTime
                    $lag = ($now - $appTime).TotalSeconds
                    $statusColor = "Green"
                    $statusText = "ACTIVE"
                    if ($lag -gt 5) { $statusColor = "Red"; $statusText = "STALLED ($([math]::Round($lag, 1))s lag)"; }
                    elseif ($lag -gt 2) { $statusColor = "Yellow"; $statusText = "SLOW ($([math]::Round($lag, 1))s lag)"; }
                    
                    Write-Host "`nApp Status: [$statusText]" -ForegroundColor $statusColor
                    
                    # --- INPUT DIAGNOSTICS ---
                    Write-Host "`n🖱️  INPUT DIAGNOSTICS:" -ForegroundColor Cyan
                    if ($json.input_stats) {
                        $s = $json.input_stats
                        if ($s.mouse_pos) {
                            $mx = [math]::Round($s.mouse_pos[0], 0)
                            $my = [math]::Round($s.mouse_pos[1], 0)
                            Write-Host "   Mouse: [$mx, $my]" -ForegroundColor Green
                            
                            $clickColor = "Gray"
                            if ($s.any_button_clicked) { $clickColor = "Yellow" }
                            Write-Host "   Click: $($s.any_button_clicked)" -ForegroundColor $clickColor
                            
                            $hoverColor = "Gray"
                            if ($s.any_button_hovered) { $hoverColor = "White" }
                            Write-Host "   Hover: $($s.any_button_hovered)" -ForegroundColor $hoverColor
                        } else {
                            Write-Host "   Mouse: NONE (Not detected)" -ForegroundColor Red
                        }
                    } else {
                        Write-Host "   (No input stats available)" -ForegroundColor DarkGray
                    }

                    # --- EVENT LOG ---
                    Write-Host "`n📜 RECENT EVENTS:" -ForegroundColor Cyan
                    if ($json.events -and $json.events.Count -gt 0) {
                        $json.events | Select-Object -Last 10 | ForEach-Object {
                            Write-Host "   $_" -ForegroundColor Yellow
                        }
                    } else {
                        Write-Host "   (No events triggered yet - Try clicking 'File')" -ForegroundColor DarkGray
                    }
                    
                    # --- PANEL STATUS ---
                    Write-Host "`n🔲 UI PANELS ($($json.panel_count)):" -ForegroundColor Cyan
                    $panels = $json.panels
                    if ($panels) {
                        foreach ($key in $panels.PSObject.Properties.Name) {
                            $p = $panels.$key
                            if ($p.has_real_content) {
                                Write-Host "   ✅ $key" -ForegroundColor Green
                            } else {
                                Write-Host "   ❌ $key" -NoNewline -ForegroundColor Red
                                Write-Host " (Placeholder: $($p.placeholder_reasons -join ', '))" -ForegroundColor DarkGray
                            }
                        }
                    }
                }
            } catch {
                Write-Host "Error reading audit file: $_" -ForegroundColor Red
            }
        } else {
            Write-Host "Waiting for ui_audit.json..." -ForegroundColor Yellow
        }
        
        Start-Sleep -Seconds 0.5
    }
}

Monitor-Loop