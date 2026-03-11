Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$localToolsRoot = Join-Path $repoRoot ".local\tauri-tools"
$globalToolsRoot = Join-Path $env:LOCALAPPDATA "tauri"

function Write-Step {
  param([string]$Message)
  Write-Host "[tauri-tools] $Message"
}

function Ensure-Directory {
  param([string]$Path)
  if (-not (Test-Path -LiteralPath $Path)) {
    New-Item -ItemType Directory -Path $Path -Force | Out-Null
  }
}

function Get-HashAlgorithm {
  param([string]$ExpectedHash)
  switch ($ExpectedHash.Length) {
    40 { return "SHA1" }
    64 { return "SHA256" }
    default { throw "Unsupported hash length for '$ExpectedHash'." }
  }
}

function Ensure-FileDownload {
  param(
    [string]$Path,
    [string]$Url,
    [string]$ExpectedHash
  )

  $algorithm = Get-HashAlgorithm $ExpectedHash
  if (Test-Path -LiteralPath $Path) {
    $currentHash = (Get-FileHash -LiteralPath $Path -Algorithm $algorithm).Hash.ToUpperInvariant()
    if ($currentHash -eq $ExpectedHash.ToUpperInvariant()) {
      return
    }
    Remove-Item -LiteralPath $Path -Force
  }

  Ensure-Directory -Path (Split-Path -Parent $Path)
  Write-Step "Downloading $(Split-Path -Leaf $Path)"
  & curl.exe -L $Url -o $Path | Out-Null
  if ($LASTEXITCODE -ne 0) {
    throw "Failed to download '$Url'."
  }
  $downloadedHash = (Get-FileHash -LiteralPath $Path -Algorithm $algorithm).Hash.ToUpperInvariant()
  if ($downloadedHash -ne $ExpectedHash.ToUpperInvariant()) {
    throw "Hash mismatch for '$Path'. Expected $ExpectedHash, got $downloadedHash."
  }
}

function Ensure-ZipExtracted {
  param(
    [string]$ZipPath,
    [string]$DestinationPath,
    [string]$SentinelPath
  )

  if (Test-Path -LiteralPath $SentinelPath) {
    return
  }

  if (Test-Path -LiteralPath $DestinationPath) {
    Remove-Item -LiteralPath $DestinationPath -Recurse -Force
  }

  Ensure-Directory -Path $DestinationPath
  Write-Step "Extracting $(Split-Path -Leaf $ZipPath)"
  Expand-Archive -LiteralPath $ZipPath -DestinationPath $DestinationPath -Force
}

function Copy-Tree {
  param(
    [string]$Source,
    [string]$Destination
  )

  Ensure-Directory -Path $Destination
  Copy-Item -Path (Join-Path $Source "*") -Destination $Destination -Recurse -Force
}

function Assert-PathExists {
  param([string]$Path)
  if (-not (Test-Path -LiteralPath $Path)) {
    throw "Required Tauri bundler path is missing: $Path"
  }
}

Write-Step "Preparing Windows bundler cache"

$nsisZip = Join-Path $localToolsRoot "nsis-3.11.zip"
$nsisExtracted = Join-Path $localToolsRoot "nsis-3.11"
$wixZip = Join-Path $localToolsRoot "wix314-binaries.zip"
$wixExtracted = Join-Path $localToolsRoot "wix314"
$applicationIdZip = Join-Path $localToolsRoot "NSIS-ApplicationID.zip"
$nsisUtilsDll = Join-Path $localToolsRoot "nsis_tauri_utils.dll"

Ensure-FileDownload `
  -Path $nsisZip `
  -Url "https://github.com/tauri-apps/binary-releases/releases/download/nsis-3.11/nsis-3.11.zip" `
  -ExpectedHash "EF7FF767E5CBD9EDD22ADD3A32C9B8F4500BB10D"

Ensure-ZipExtracted `
  -ZipPath $nsisZip `
  -DestinationPath $nsisExtracted `
  -SentinelPath (Join-Path $nsisExtracted "Bin\makensis.exe")

Ensure-FileDownload `
  -Path $wixZip `
  -Url "https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip" `
  -ExpectedHash "6AC824E1642D6F7277D0ED7EA09411A508F6116BA6FAE0AA5F2C7DAA2FF43D31"

Ensure-ZipExtracted `
  -ZipPath $wixZip `
  -DestinationPath $wixExtracted `
  -SentinelPath (Join-Path $wixExtracted "candle.exe")

Ensure-FileDownload `
  -Path $applicationIdZip `
  -Url "https://github.com/tauri-apps/binary-releases/releases/download/nsis-plugins-v0/NSIS-ApplicationID.zip" `
  -ExpectedHash "E0951DC4AC0DF5E34FB7CCCF04EC5E1B747091FB"

Ensure-FileDownload `
  -Path $nsisUtilsDll `
  -Url "https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.3/nsis_tauri_utils.dll" `
  -ExpectedHash "75197FEE3C6A814FE035788D1C34EAD39349B860"

$globalNsisDir = Join-Path $globalToolsRoot "NSIS"
$globalWixDir = Join-Path $globalToolsRoot "WixTools314"

Copy-Tree -Source $nsisExtracted -Destination $globalNsisDir
Copy-Tree -Source $wixExtracted -Destination $globalWixDir

$nsisAdditionalDir = Join-Path $globalNsisDir "Plugins\x86-unicode\additional"
Ensure-Directory -Path $nsisAdditionalDir
Copy-Item -LiteralPath $nsisUtilsDll -Destination (Join-Path $nsisAdditionalDir "nsis_tauri_utils.dll") -Force

$applicationIdSentinel = Join-Path $globalNsisDir "Plugins\x86-unicode\ReleaseUnicode\ApplicationID.dll"
if (-not (Test-Path -LiteralPath $applicationIdSentinel)) {
  Write-Step "Extracting NSIS ApplicationID plugin"
  Expand-Archive -LiteralPath $applicationIdZip -DestinationPath (Join-Path $globalNsisDir "Plugins\x86-unicode") -Force
}

Assert-PathExists -Path (Join-Path $globalNsisDir "Bin\makensis.exe")
Assert-PathExists -Path (Join-Path $globalNsisDir "Plugins\x86-unicode\additional\nsis_tauri_utils.dll")
Assert-PathExists -Path $applicationIdSentinel
Assert-PathExists -Path (Join-Path $globalWixDir "candle.exe")
Assert-PathExists -Path (Join-Path $globalWixDir "light.exe")

Write-Step "Bundler cache is ready at $globalToolsRoot"
