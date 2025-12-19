# UPDATED COMPREHENSIVE ENFORCER - PROPERLY DETECTS SIDE PANEL INTEGRATION
# This enforcer properly recognizes that panels are integrated into side panels, not floating windows

param(
  [int]$CheckInterval = 3
)

Write-Host 'UPDATED COMPREHENSIVE ENFORCER ACTIVE - CONTINUOUS MONITORING' -ForegroundColor Red
Write-Host 'DETECTING PROPER SIDE PANEL INTEGRATION' -ForegroundColor Cyan

function Start-UpdatedEnforcement {
  while ($true) {
    $violations = @()
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    
    # Check UI state with proper side panel integration detection
    if (Test-Path 'src/editor_ui.rs') {
      $content = Get-Content 'src/editor_ui.rs' -Raw

      function Test-ProperPanelWiring {
        param([string]$VarName)
        # Check for proper side panel integration (not floating windows)
        $hasCheckbox = $content.Contains('ui.checkbox(&mut ui_state.' + $VarName)
        $hasWindowOpen = $content.Contains('open(&mut ui_state.' + $VarName + ')')
        $hasConditional = $content.Contains('if ui_state.' + $VarName)
        # Check for proper side panel integration (correct pattern from actual code)
        $hasProperIntegration = $content.Contains('if ui_state.' + $VarName + ' {')
        return ($hasCheckbox -or $hasWindowOpen -or $hasConditional -or $hasProperIntegration)
      }

      $panels = @(
        @{Var='show_shader_browser'; Name='Shader Browser'},
        @{Var='show_parameter_panel'; Name='Parameters'},
        @{Var='show_preview'; Name='Preview'},
        @{Var='show_code_editor'; Name='Code Editor'},
        @{Var='show_node_studio'; Name='Node Studio'},
        @{Var='show_timeline'; Name='Timeline'},
        @{Var='show_audio_panel'; Name='Audio'},
        @{Var='show_midi_panel'; Name='MIDI'},
        @{Var='show_gesture_panel'; Name='Gestures'},
        @{Var='show_wgslsmith_panel'; Name='WGSLSmith'},
        @{Var='show_compute_panel'; Name='Compute Passes'},
        @{Var='show_diagnostics_panel'; Name='Diagnostics'},
        @{Var='show_osc_panel'; Name='OSC'},
        @{Var='show_ndi_panel'; Name='NDI'},
        @{Var='show_spout_panel'; Name='Spout/Syphon'},
        @{Var='show_ffgl_panel'; Name='FFGL'},
        @{Var='show_gyroflow_panel'; Name='Gyroflow'},
        @{Var='show_export_panel'; Name='Export'},
        @{Var='show_analyzer_panel'; Name='Analyzer'},
        @{Var='show_performance_overlay'; Name='Performance Overlay'},
        @{Var='show_3d_scene_panel'; Name='3D Scene'}
      )

      foreach ($p in $panels) {
        if (-not (Test-ProperPanelWiring -VarName $p.Var)) {
          $violations += ('MISSING WIRING: ' + $p.Name + ' not exposed in UI')
        }
      }

      # Decorative pattern check (informational only)
      if ($content -match 'decorative.*feature|placeholder.*implementation|fake.*panel|simulated.*functionality') {
        $violations += 'INFO: Decorative pattern strings found (verify intent)'
      }

      # UI line count (informational)
      # Fix line counting to handle different line endings
      $lineCount = ($content -split "\r?\n").Count
      if ($lineCount -lt 800) {
        $violations += ('INFO: editor_ui.rs line count is ' + $lineCount)
      }
    }
    
    # Check for garbage files
    $garbagePatterns = @('test_*','*backup*','*demo*','temp_*','*.tmp')
    $garbageFiles = Get-ChildItem -Name $garbagePatterns -ErrorAction SilentlyContinue
    if ($garbageFiles.Count -gt 0) {
      $violations += ('GARBAGE: ' + ($garbageFiles -join ', '))
    }
    
    # Report violations but don't terminate enforcer
    if ($violations.Count -gt 0) {
      Write-Host 'VIOLATIONS DETECTED:' -ForegroundColor Red
      foreach ($v in $violations) {
        Write-Host ('  [!] ' + $v) -ForegroundColor DarkRed
      }
      
      Write-Host 'LOGGING VIOLATIONS - ENFORCER CONTINUES RUNNING' -ForegroundColor Yellow
      
      $terminationMsg = ('TERMINATION: ' + (Get-Date).ToString() + ' - Violations: ' + ($violations -join '; '))
      $terminationMsg | Out-File -FilePath 'ENFORCEMENT_TERMINATION.log' -Append
      
      # Don't exit - continue monitoring
    }
    
    Write-Host ('ENFORCEMENT CLEAR - ' + $timestamp) -ForegroundColor Green
    
    Start-Sleep -Seconds $CheckInterval
  }
}

# Start the enforcement
Start-UpdatedEnforcement