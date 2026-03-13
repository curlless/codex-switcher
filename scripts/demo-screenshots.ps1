param(
  [string]$ExecutablePath = "",
  [string]$DemoRoot = "",
  [switch]$Cleanup
)

$ErrorActionPreference = "Stop"

function Resolve-DesktopExecutable {
  param([string]$RequestedPath)

  $candidates = @()

  if ($RequestedPath) {
    $candidates += $RequestedPath
  }

  $candidates += @(
    (Join-Path $env:LOCALAPPDATA "Codex Switcher Desktop\codex-switcher-desktop.exe"),
    "F:\cursor projects\codex-switcher\apps\desktop\src-tauri\target\debug\codex-switcher-desktop.exe",
    "F:\cursor projects\codex-switcher\apps\desktop\src-tauri\target\release\codex-switcher-desktop.exe",
    "F:\cursor projects\codex-switcher\apps\desktop\src-tauri\target\x86_64-pc-windows-msvc\release\codex-switcher-desktop.exe"
  )

  foreach ($candidate in $candidates) {
    if ($candidate -and (Test-Path -LiteralPath $candidate)) {
      return (Resolve-Path -LiteralPath $candidate).Path
    }
  }

  throw "Could not resolve Codex Switcher Desktop executable. Pass -ExecutablePath explicitly."
}

function New-DemoRoot {
  $root = Join-Path $env:TEMP ("codex-switcher-gui-demo-" + [guid]::NewGuid().ToString("N"))
  $profilesDir = Join-Path $root ".codex\profiles"
  $screenshotsDir = Join-Path $root "screenshots"
  New-Item -ItemType Directory -Force -Path $profilesDir | Out-Null
  New-Item -ItemType Directory -Force -Path $screenshotsDir | Out-Null
  return $root
}

function Write-DemoReadme {
  param(
    [string]$Root,
    [string]$Executable
  )

  $readmePath = Join-Path $Root "README.txt"
  $cleanupCommand = "powershell -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Cleanup -DemoRoot `"$Root`""
  $content = @"
Codex Switcher Desktop screenshot demo environment

Executable:
$Executable

This folder is disposable. The GUI is launched with:
- CODEX_SWITCHER_GUI_DEMO=1
- CODEX_SWITCHER_HOME=$Root
- CODEX_SWITCHER_AUTH_DIR=$Root\.codex

Demo dataset includes:
- ready profiles with different headroom values
- reserved profile
- API key only profile
- usage fetch error profile (404-style)
- missing access token profile
- missing account id profile
- free plan profile
- missing 5h window profile
- missing 7d window profile

Cleanup:
$cleanupCommand
"@
  Set-Content -LiteralPath $readmePath -Value $content -Encoding UTF8
}

function Remove-DemoRoot {
  param([string]$Root)

  if (-not $Root) {
    throw "Pass -DemoRoot when using -Cleanup."
  }

  Get-Process -Name "codex-switcher-desktop" -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Seconds 1

  if (Test-Path -LiteralPath $Root) {
    Remove-Item -LiteralPath $Root -Recurse -Force
  }
}

$resolvedExecutable = Resolve-DesktopExecutable -RequestedPath $ExecutablePath

if ($Cleanup) {
  Remove-DemoRoot -Root $DemoRoot
  Write-Host "Removed demo environment: $DemoRoot"
  exit 0
}

$resolvedDemoRoot = if ($DemoRoot) {
  $DemoRoot
} else {
  New-DemoRoot
}

New-Item -ItemType Directory -Force -Path (Join-Path $resolvedDemoRoot ".codex\profiles") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $resolvedDemoRoot "screenshots") | Out-Null
Write-DemoReadme -Root $resolvedDemoRoot -Executable $resolvedExecutable

Get-Process -Name "codex-switcher-desktop" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $resolvedExecutable
$psi.WorkingDirectory = Split-Path -Path $resolvedExecutable -Parent
$psi.UseShellExecute = $false
$psi.Environment["HOME"] = $resolvedDemoRoot
$psi.Environment["USERPROFILE"] = $resolvedDemoRoot
$psi.Environment["CODEX_SWITCHER_HOME"] = $resolvedDemoRoot
$psi.Environment["CODEX_SWITCHER_AUTH_DIR"] = (Join-Path $resolvedDemoRoot ".codex")
$psi.Environment["CODEX_SWITCHER_SKIP_UPDATE"] = "1"
$psi.Environment["CODEX_SWITCHER_GUI_DEMO"] = "1"
$psi.Environment["CODEX_SWITCHER_COMMAND"] = "codex-switcher"
$psi.Environment["NO_COLOR"] = "1"

$process = [System.Diagnostics.Process]::Start($psi)
if (-not $process) {
  throw "Failed to launch Codex Switcher Desktop demo process."
}

$cleanupCommand = "powershell -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Cleanup -DemoRoot `"$resolvedDemoRoot`""
[PSCustomObject]@{
  ExecutablePath = $resolvedExecutable
  DemoRoot = $resolvedDemoRoot
  ProcessId = $process.Id
  CleanupCommand = $cleanupCommand
  SuggestedShots = @(
    "Profiles view with mixed tags",
    "Quick Switch hero card",
    "Switch preview with availability labels"
  ) -join "; "
} | Format-List | Out-String -Width 240 | Write-Output
