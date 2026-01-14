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
        # Reduce flicker by only clearing if we have data or periodically
        # But Clear-Host is smoothest for now
        Clear-Host
        Write-Header "LIVE APP DIAGNOSTICS (Ctrl+C to Stop)"
        $now = Get-Date
        Write-Host "Monitor Time: $($now.ToString('HH:mm:ss.fff'))" -ForegroundColor Gray
        
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
                # Read with retry to avoid lock contention
                $jsonStr = $null
                $attempts = 0
                while ($attempts -lt 5 -and $null -eq $jsonStr) {
                    try { $jsonStr = Get-Content $AuditFile -Raw -ErrorAction Stop } catch { Start-Sleep -Milliseconds 20 }
                    $attempts++
                }

                if ($jsonStr) {
                    $json = $jsonStr | ConvertFrom-Json
                    
                    # --- APP HEALTH ---
                    $appTime = [datetimeOffset]::FromUnixTimeSeconds($json.timestamp).LocalDateTime
                    $lag = ($now - $appTime).TotalSeconds
                    $statusColor = "Green"
                    $statusText = "ACTIVE (Updates: < 0.2s)"
                    
                    if ($lag -gt 5) { $statusColor = "Red"; $statusText = "STALLED ($([math]::Round($lag, 1))s lag)"; }
                    elseif ($lag -gt 1) { $statusColor = "Yellow"; $statusText = "SLOW ($([math]::Round($lag, 1))s lag)"; }
                    
                    Write-Host "`nApp Status: [$statusText]" -ForegroundColor $statusColor
                    
                    # --- INPUT DIAGNOSTICS ---
                    Write-Host "`n🖱️  INPUT DIAGNOSTICS:" -ForegroundColor Cyan
                    if ($json.input_stats) {
                        $s = $json.input_stats
                        
                        # Mouse Position
                        if ($s.mouse_pos) {
                            $mx = [math]::Round($s.mouse_pos[0], 0)
                            $my = [math]::Round($s.mouse_pos[1], 0)
                            Write-Host "   Mouse Pos: [$mx, $my]" -ForegroundColor Green
                        } else {
                            Write-Host "   Mouse Pos: NONE (Outside window?)" -ForegroundColor Red
                        }

                        # Mouse Buttons
                        $lmb = if ($s.primary_clicked) { "DOWN" } else { "UP" }
                        $rmb = if ($s.secondary_clicked) { "DOWN" } else { "UP" }
                        $mmb = if ($s.middle_clicked) { "DOWN" } else { "UP" }
                        
                        $btnColor = if ($s.any_button_clicked -or $s.any_button_hovered) { "Yellow" } else { "Gray" }
                        
                        Write-Host "   Buttons:   LMB:[$lmb]  RMB:[$rmb]  MMB:[$mmb]" -ForegroundColor $btnColor
                        
                        # Key Presses
                        if ($s.keys_pressed -and $s.keys_pressed.Count -gt 0) {
                            Write-Host "   Keys:      [ $($s.keys_pressed -join ' + ') ]" -ForegroundColor Magenta
                        } else {
                            Write-Host "   Keys:      (None)" -ForegroundColor DarkGray
                        }
                        
                        Write-Host "   Interact:  $($s.interactions) total clicks" -ForegroundColor Gray
                    } else {
                        Write-Host "   (No input stats available)" -ForegroundColor DarkGray
                    }

                    # --- EVENT LOG ---
                    Write-Host "`n📜 RECENT EVENTS (Last 5):" -ForegroundColor Cyan
                    if ($json.events -and $json.events.Count -gt 0) {
                        $json.events | Select-Object -Last 5 | ForEach-Object {
                            Write-Host "   $_" -ForegroundColor Yellow
                        }
                    } else {
                        Write-Host "   (No events triggered yet)" -ForegroundColor DarkGray
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
        
        Start-Sleep -Milliseconds 100
    }
}

Monitor-Loop