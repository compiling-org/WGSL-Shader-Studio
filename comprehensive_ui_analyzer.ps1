param(
  [int]$CheckInterval = 60,
  [string]$OutputFile = "UI_AUDIT_REPORT.md"
)

$ErrorActionPreference = "Stop"

function Analyze-ProjectStructure {
  $key = @{
    "src/lib.rs" = "Library main file"
    "src/editor_ui.rs" = "Editor UI implementation"
    "src/ui_analyzer.rs" = "UI analyzer module"
    "src/ui_analyzer_enhanced.rs" = "Enhanced UI analyzer"
    "Cargo.toml" = "Project configuration"
    ".trae/documents/comprehensive_work_documentation.md" = "Documentation"
  }
  $present = @()
  $missing = @()
  foreach ($p in $key.Keys) {
    if (Test-Path $p) { $present += [pscustomobject]@{Path=$p;Desc=$key[$p];Size=(Get-Item $p).Length} } else { $missing += [pscustomobject]@{Path=$p;Desc=$key[$p]} }
  }
  [pscustomobject]@{ Present=$present; Missing=$missing }
}

function Analyze-CargoToml {
  $content = Get-Content "Cargo.toml" -Raw -ErrorAction SilentlyContinue
  $deps = @("bevy","wgpu","egui","tokio","midir","rustfft","serde","regex","naga","image","rfd")
  $found = @(); $missing = @()
  foreach ($d in $deps) { if ($content -and ($content -match $d)) { $found += $d } else { $missing += $d } }
  [pscustomobject]@{ Found=$found; Missing=$missing; HasFeatures=($content -match "\[features\]"); HasBins=($content -match "\[\[bin\]\]") }
}

function Analyze-SourceFiles {
  $files = @("src/lib.rs","src/editor_ui.rs","src/ui_analyzer.rs","src/ui_analyzer_enhanced.rs")
  $results = @()
  foreach ($f in $files) {
    if (Test-Path $f) {
      $c = Get-Content $f -Raw
      $lines = ($c -split "\r?\n").Count
      $patterns = @{
        "UI Analyzer" = "UIAnalyzer|ui_analyzer"
        "Editor UI" = "EditorUi|editor_ui"
        "Shader Parameters" = "ShaderParameter|parse_shader_parameters"
        "WGSL Support" = "wgsl|WGSL"
        "ISF Support" = "isf|ISF"
        "Audio Integration" = "audio|midi|Audio|MIDI"
        "GPU Features" = "gpu|GPU|wgpu|WGPU"
        "Module System" = "module_system|ModuleId"
      }
      $found = @()
      foreach ($k in $patterns.Keys) { if ($c -match $patterns[$k]) { $found += $k } }
      $results += [pscustomobject]@{ File=$f; Lines=$lines; Found=$found }
    } else {
      $results += [pscustomobject]@{ File=$f; Lines=0; Found=@() }
    }
  }
  $results
}

function Analyze-FeatureState {
  $bevyApp = if (Test-Path "src/bevy_app.rs") { Get-Content "src/bevy_app.rs" -Raw } else { "" }
  $editorUi = if (Test-Path "src/editor_ui.rs") { Get-Content "src/editor_ui.rs" -Raw } else { "" }
  $wgpuOk = ($editorUi -match "COPY_BYTES_PER_ROW_ALIGNMENT") -or ($bevyApp -match "wgpu")
  $threePanel = ($bevyApp -match "shader_browser_panel") -and ($bevyApp -match "parameter_panel") -and ($bevyApp -match "code_editor_panel") -and ($bevyApp -match "CentralPanel")
  $paramPanel = ($bevyApp -match "Interactive shader parameters") -and ($bevyApp -match "Slider")
  $compilation = ($editorUi -match "Compiled") -or ($editorUi -match "Error") -or ($bevyApp -match "Compiled")
  [pscustomobject]@{ Wgpu=$wgpuOk; ThreePanel=$threePanel; ParameterPanel=$paramPanel; Compilation=$compilation }
}

function Write-Report {
  param($proj,$cargo,$src,$feat)
  $sb = New-Object System.Text.StringBuilder
  [void]$sb.AppendLine("# WGSL Shader Studio - COMPREHENSIVE UI ANALYSIS REPORT")
  [void]$sb.AppendLine("")
  [void]$sb.AppendLine("## Executive Summary")
  [void]$sb.AppendLine("")
  $total = $src.Count
  $present = $proj.Present.Count
  $missing = $proj.Missing.Count
  [void]$sb.AppendLine("- Total analyzed files: $total")
  [void]$sb.AppendLine("- Present key files: $present")
  [void]$sb.AppendLine("- Missing key files: $missing")
  [void]$sb.AppendLine("- Features: bins=$($cargo.HasBins) features_section=$($cargo.HasFeatures)")
  [void]$sb.AppendLine("")
  [void]$sb.AppendLine("## Phase 1 Status")
  [void]$sb.AppendLine("- GPU-Only Enforcement: $($feat.Wgpu)")
  [void]$sb.AppendLine("- Three-Panel Layout: $($feat.ThreePanel)")
  [void]$sb.AppendLine("- Parameter Panel: $($feat.ParameterPanel)")
  [void]$sb.AppendLine("- Basic Shader Compilation: $($feat.Compilation)")
  [void]$sb.AppendLine("")
  [void]$sb.AppendLine("## Source File Analysis")
  foreach ($r in $src) {
    [void]$sb.AppendLine("- $($r.File) ($($r.Lines) lines) -> $([string]::Join(', ', $r.Found))")
  }
  Set-Content -Path $OutputFile -Value $sb.ToString()
}

Write-Host "Starting Comprehensive UI Analyzer (PowerShell)"
for (;;) {
  try {
    $proj = Analyze-ProjectStructure
    $cargo = Analyze-CargoToml
    $src = Analyze-SourceFiles
    $feat = Analyze-FeatureState
    Write-Report -proj $proj -cargo $cargo -src $src -feat $feat
    Write-Host "UI Analyzer report updated: $OutputFile"
  } catch {
    Write-Host "Analyzer error: $($_.Exception.Message)" -ForegroundColor Red
  }
  Start-Sleep -Seconds $CheckInterval
}