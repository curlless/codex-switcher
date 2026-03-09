param(
  [Parameter(Mandatory = $true)]
  [string]$ExecutablePath,
  [string]$OutputPath = "",
  [int]$StartupTimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

function Write-Utf8NoBom {
  param(
    [string]$Path,
    [string]$Value
  )

  $directory = Split-Path -Path $Path -Parent
  if ($directory) {
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
  }

  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($Path, $Value, $utf8NoBom)
}

function ConvertTo-Base64Url {
  param([string]$Value)

  $bytes = [System.Text.Encoding]::UTF8.GetBytes($Value)
  [Convert]::ToBase64String($bytes).TrimEnd("=").Replace("+", "-").Replace("/", "_")
}

function New-IdToken {
  param(
    [string]$Email,
    [string]$Plan
  )

  $header = '{"alg":"none","typ":"JWT"}'
  $payload = @{
    email = $Email
    "https://api.openai.com/auth" = @{
      chatgpt_plan_type = $Plan
    }
  } | ConvertTo-Json -Compress

  "{0}.{1}." -f (ConvertTo-Base64Url $header), (ConvertTo-Base64Url $payload)
}

function New-SmokeHome {
  $root = Join-Path ([System.IO.Path]::GetTempPath()) ("codex-switcher-desktop-smoke-{0}" -f ([guid]::NewGuid().ToString("N")))
  $codexDir = Join-Path $root ".codex"
  $profilesDir = Join-Path $codexDir "profiles"
  New-Item -ItemType Directory -Force -Path $profilesDir | Out-Null

  $primaryId = "primary@example.com-plus"
  $backupId = "backup@example.com-team"

  $primaryAuth = @{
    tokens = @{
      account_id = "acct-primary"
      id_token = New-IdToken -Email "primary@example.com" -Plan "plus"
      access_token = "access-primary"
    }
  }
  $backupAuth = @{
    tokens = @{
      account_id = "acct-backup"
      id_token = New-IdToken -Email "backup@example.com" -Plan "team"
      access_token = "access-backup"
    }
  }

  $primaryJson = $primaryAuth | ConvertTo-Json -Compress
  $backupJson = $backupAuth | ConvertTo-Json -Compress
  Write-Utf8NoBom -Path (Join-Path $codexDir "auth.json") -Value $primaryJson
  Write-Utf8NoBom -Path (Join-Path $profilesDir "$primaryId.json") -Value $primaryJson
  Write-Utf8NoBom -Path (Join-Path $profilesDir "$backupId.json") -Value $backupJson
  Write-Utf8NoBom -Path (Join-Path $codexDir "config.toml") -Value 'chatgpt_base_url = "http://127.0.0.1:1/backend-api"'

  $profilesIndex = @{
    version = 1
    active_profile_id = $primaryId
    profiles = @{
      $primaryId = @{
        label = "primary"
        added_at = 1
        last_used = 200
      }
      $backupId = @{
        label = "backup"
        added_at = 1
        last_used = 100
      }
    }
  } | ConvertTo-Json -Compress -Depth 6

  Write-Utf8NoBom -Path (Join-Path $profilesDir "profiles.json") -Value $profilesIndex

  return @{
    Root = $root
    PrimaryLabel = "primary"
    BackupLabel = "backup"
  }
}

function Wait-ForWindow {
  param(
    [System.Diagnostics.Process]$Process,
    [int]$TimeoutSeconds
  )

  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)

  while ((Get-Date) -lt $deadline) {
    $Process.Refresh()
    if ($Process.HasExited) {
      throw "Desktop executable exited before exposing a main window."
    }

    if ($Process.MainWindowHandle -ne 0) {
      try {
        $window = [System.Windows.Automation.AutomationElement]::FromHandle($Process.MainWindowHandle)
        if ($window) {
          return $window
        }
      } catch {
      }
    }

    Start-Sleep -Milliseconds 500
  }

  throw "Timed out waiting for desktop window for process $($Process.Id)."
}

function Get-NamedElements {
  param([System.Windows.Automation.AutomationElement]$Root)

  $named = @()
  $elements = $Root.FindAll(
    [System.Windows.Automation.TreeScope]::Descendants,
    [System.Windows.Automation.Condition]::TrueCondition
  )

  for ($index = 0; $index -lt $elements.Count; $index++) {
    $element = $elements.Item($index)
    $name = $element.Current.Name
    if (![string]::IsNullOrWhiteSpace($name)) {
      $named += $name.Trim()
    }
  }

  return $named | Select-Object -Unique
}

function Find-NamedElement {
  param(
    [System.Windows.Automation.AutomationElement]$Root,
    [string]$Name,
    [int]$TimeoutSeconds = 10
  )

  $condition = New-Object System.Windows.Automation.PropertyCondition(
    [System.Windows.Automation.AutomationElement]::NameProperty,
    $Name
  )
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)

  while ((Get-Date) -lt $deadline) {
    $found = $Root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
    if ($found) {
      return $found
    }
    Start-Sleep -Milliseconds 400
  }

  throw "Timed out waiting for UI element '$Name'."
}

function Get-BrowserSurfaceName {
  param([System.Windows.Automation.AutomationElement]$Root)

  $browserRootCondition = New-Object System.Windows.Automation.PropertyCondition(
    [System.Windows.Automation.AutomationElement]::ClassNameProperty,
    "BrowserRootView"
  )
  $browserRoot = $Root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $browserRootCondition)
  if ($browserRoot -and -not [string]::IsNullOrWhiteSpace($browserRoot.Current.Name)) {
    return $browserRoot.Current.Name.Trim()
  }

  $fallbackName = $Root.Current.Name
  if ($null -eq $fallbackName) {
    return ""
  }

  return $fallbackName.Trim()
}

function Get-SmokeTracePath {
  param([string]$SmokeRoot)

  return (Join-Path $SmokeRoot ".codex\desktop-smoke-trace.json")
}

function Read-SmokeTrace {
  param([string]$Path)

  if (-not (Test-Path -LiteralPath $Path)) {
    return $null
  }

  try {
    return (Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json)
  } catch {
    return $null
  }
}

function Wait-ForSmokeTrace {
  param(
    [string]$Path,
    [scriptblock]$Predicate,
    [int]$TimeoutSeconds = 15
  )

  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    $trace = Read-SmokeTrace -Path $Path
    if ($trace -and (& $Predicate $trace)) {
      return $trace
    }

    Start-Sleep -Milliseconds 500
  }

  throw "Timed out waiting for smoke trace condition at $Path."
}

function Invoke-UiElement {
  param([System.Windows.Automation.AutomationElement]$Element)

  $invokePattern = [System.Windows.Automation.InvokePattern]::Pattern
  if ($Element.TryGetCurrentPattern($invokePattern, [ref]$patternObject)) {
    $patternObject.Invoke()
    return
  }

  throw "Element '$($Element.Current.Name)' does not support InvokePattern."
}

if (-not (Test-Path -LiteralPath $ExecutablePath)) {
  throw "Executable not found: $ExecutablePath"
}

$smokeHome = New-SmokeHome
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = (Resolve-Path -LiteralPath $ExecutablePath).Path
$psi.WorkingDirectory = Split-Path -Path $psi.FileName -Parent
$psi.UseShellExecute = $false
$psi.Environment["HOME"] = $smokeHome.Root
$psi.Environment["USERPROFILE"] = $smokeHome.Root
$psi.Environment["CODEX_SWITCHER_HOME"] = $smokeHome.Root
$psi.Environment["CODEX_SWITCHER_SKIP_UPDATE"] = "1"
$psi.Environment["CODEX_SWITCHER_COMMAND"] = "codex-switcher"
$psi.Environment["CODEX_SWITCHER_SMOKE_AUTOMATION"] = "1"
$psi.Environment["NO_COLOR"] = "1"

$process = [System.Diagnostics.Process]::Start($psi)
if (-not $process) {
  throw "Failed to launch desktop executable."
}

$result = [ordered]@{
  executablePath = $psi.FileName
  smokeHome = $smokeHome.Root
  processId = $process.Id
  windowTitle = $null
  smokeTracePath = (Get-SmokeTracePath -SmokeRoot $smokeHome.Root)
  smokeTrace = $null
  checks = @()
  observedNames = @()
}

try {
  $process.WaitForInputIdle($StartupTimeoutSeconds * 1000) | Out-Null
} catch {
}

try {
  $window = Wait-ForWindow -Process $process -TimeoutSeconds $StartupTimeoutSeconds
  $result.windowTitle = $window.Current.Name
  Start-Sleep -Seconds 2
  $observed = Get-NamedElements -Root $window
  $result.observedNames = $observed
  $profilesTrace = Wait-ForSmokeTrace -Path $result.smokeTracePath -TimeoutSeconds $StartupTimeoutSeconds -Predicate {
    param($trace)
    $trace.phase -eq "ready" -and $trace.view -eq "profiles" -and $trace.profileCount -eq 2 -and $trace.activeProfile -eq "primary"
  }
  $result.smokeTrace = $profilesTrace

  $result.checks += @{
    name = "startup"
    ok = $true
    detail = "Native window launched and smoke trace reached ready profiles state."
  }

  $result.checks += @{
    name = "profiles-view"
    ok = $true
    detail = "Smoke trace proved bridge-loaded profile metadata in the profiles view."
  }

  $switchTrace = Wait-ForSmokeTrace -Path $result.smokeTracePath -TimeoutSeconds $StartupTimeoutSeconds -Predicate {
    param($trace)
    $trace.phase -eq "ready" -and $trace.view -eq "quick-switch" -and $trace.profileCount -eq 2 -and $trace.activeProfile -eq "primary"
  }
  $result.smokeTrace = $switchTrace

  $result.checks += @{
    name = "switch-view"
    ok = $true
    detail = "Smoke trace reached the quick-switch view."
  }

  $reloadTrace = Wait-ForSmokeTrace -Path $result.smokeTracePath -TimeoutSeconds $StartupTimeoutSeconds -Predicate {
    param($trace)
    $trace.phase -eq "ready" -and $trace.view -eq "reload" -and $trace.profileCount -eq 2 -and $trace.activeProfile -eq "primary"
  }
  $result.smokeTrace = $reloadTrace
  $result.checks += @{
    name = "reload-view"
    ok = $true
    detail = "Smoke trace reached the reload view."
  }

  $refreshTrace = Wait-ForSmokeTrace -Path $result.smokeTracePath -TimeoutSeconds $StartupTimeoutSeconds -Predicate {
    param($trace)
    $trace.view -eq "reload" -and $trace.refreshCount -ge 1 -and $trace.event -eq "refresh-success"
  }
  $result.smokeTrace = $refreshTrace
  $result.checks += @{
    name = "bridge-refresh"
    ok = $true
    detail = "Smoke trace proved a bridge-backed refresh action from the packaged runtime."
  }
} catch {
  try {
    if ($window) {
      $result.observedNames = Get-NamedElements -Root $window
    }
    $result.smokeTrace = Read-SmokeTrace -Path $result.smokeTracePath
  } catch {
  }
  $result.checks += @{
    name = "failure"
    ok = $false
    detail = $_.Exception.Message
  }
  throw
} finally {
  $json = $result | ConvertTo-Json -Depth 6
  if ($OutputPath) {
    $outputDir = Split-Path -Path $OutputPath -Parent
    if ($outputDir) {
      New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
    }
    Write-Utf8NoBom -Path $OutputPath -Value $json
  } else {
    $json
  }

  if (-not $process.HasExited) {
    $null = $process.CloseMainWindow()
    Start-Sleep -Seconds 2
  }
  if (-not $process.HasExited) {
    $process.Kill()
    $process.WaitForExit()
  }
}
