# COMPREHENSIVE MONITOR (Formerly Enforcer) - HELPFUL MODE
# This script monitors the codebase for potential configuration issues.
# IT DOES NOT TERMINATE THE APPLICATION.

param(
  [int]$CheckInterval = 5
)
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "  COMPREHENSIVE DEV MONITOR ACTIVE" -ForegroundColor White
Write-Host "  Scanning for configuration issues/warnings..." -ForegroundColor Gray
Write-Host "  (Passive Mode: Will NOT kill processes)" -ForegroundColor Green
Write-Host "======================================================================" -ForegroundColor Cyan

while ($true) {
  $violations = @()
  $warnings = @()
  
  # Check UI state with wiring-based heuristics
  if (Test-Path 'src/editor_ui.rs') {
    $content = Get-Content 'src/editor_ui.rs' -Raw

    function Test-PanelWiring {
      param([string]$VarName)
      # Heuristic: Check if the variable is used in checkbox, open(), if(), or side_panels assignment
      $hasCheckbox = $content.Contains('ui.checkbox(&mut ui_state.' + $VarName)
      $hasWindowOpen = $content.Contains('open(&mut ui_state.' + $VarName + ')')
      $hasConditional = $content.Contains('if ui_state.' + $VarName)
      $hasSidePanelIntegration = $content.Contains('ui_state.' + $VarName) -and $content.Contains('side_panels')
      
      return ($hasCheckbox -or $hasWindowOpen -or $hasConditional -or $hasSidePanelIntegration)
    }

    $panels = @(
      @{Var = 'show_shader_browser'; Name = 'Shader Browser' },
      @{Var = 'show_parameter_panel'; Name = 'Parameters' },
      @{Var = 'show_preview'; Name = 'Preview' },
      @{Var = 'show_code_editor'; Name = 'Code Editor' },
      @{Var = 'show_node_studio'; Name = 'Node Studio' },
      @{Var = 'show_timeline'; Name = 'Timeline' },
      @{Var = 'show_audio_panel'; Name = 'Audio' },
      @{Var = 'show_midi_panel'; Name = 'MIDI' },
      @{Var = 'show_gesture_panel'; Name = 'Gestures' },
      @{Var = 'show_compute_panel'; Name = 'Compute Passes' },
      @{Var = 'show_diagnostics_panel'; Name = 'Diagnostics' }
    )

    foreach ($p in $panels) {
      if (-not (Test-PanelWiring -VarName $p.Var)) {
        $warnings += ("UI Wiring Hint: '{0}' ({1}) might not be exposed in UI" -f $p.Name, $p.Var)
      }
    }

    # Check for hardcoded 'false' in Default impl which might hide features effectively permanently
    $realSystems = @(
      @{Pattern = "show_node_studio:\s*false"; System = "Node Graph" },
      @{Pattern = "show_timeline:\s*false"; System = "Timeline" },
      @{Pattern = "show_audio_panel:\s*false"; System = "Audio System" }
    )
    foreach ($system in $realSystems) {
      if ($content -match $system.Pattern) {
        $warnings += ("Config Warning: {0} appears disabled by default (found '{1}')" -f $system.System, $system.Pattern)
      }
    }
  }
  
  # Check for garbage files (just informational)
  $garbagePatterns = @('test_*', 'temp_*', '*.tmp')
  $garbageFiles = Get-ChildItem -Name $garbagePatterns -ErrorAction SilentlyContinue
  if ($garbageFiles.Count -gt 0) {
    $warnings += ("Cleanup Hint: Found temporary files: " + ($garbageFiles -join ', '))
  }
  
  # OUTPUT REPORT
  if ($warnings.Count -gt 0) {
    Write-Host ("[{0}] scan found potential items:" -f (Get-Date -Format 'HH:mm:ss')) -ForegroundColor Yellow
    foreach ($w in $warnings) {
      Write-Host "  - $w" -ForegroundColor Yellow
    }
  }
  else {
    # Occasional heartbeat
    # Write-Host "." -NoNewline -ForegroundColor DarkGray
  }
  
  Start-Sleep -Seconds $CheckInterval
}
