param(
    [switch]$NoAudit,
    [switch]$NoTests
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Show-Usage {
    Write-Output "Usage: scripts/check.ps1 [-NoAudit] [-NoTests]"
}

function Test-CommandAvailable {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    Write-Host "==> $Label"
    $global:LASTEXITCODE = 0
    & $Action

    if (-not $?) {
        throw "Step failed: $Label"
    }

    $exitCode = 0
    if (Test-Path variable:global:LASTEXITCODE) {
        $exitCode = $global:LASTEXITCODE
    }

    if ($exitCode -ne 0) {
        throw ("Step failed with exit code {0}: {1}" -f $exitCode, $Label)
    }
}

if ($args -contains "-h" -or $args -contains "--help") {
    Show-Usage
    exit 0
}

Invoke-Step "cargo fetch --locked" { cargo fetch --locked }
Invoke-Step "cargo fmt --all -- --check" { cargo fmt --all -- --check }
Invoke-Step "cargo clippy --all-targets --locked -- -D warnings" {
    cargo clippy --all-targets --locked -- -D warnings
}

if (Test-CommandAvailable "node") {
    Invoke-Step "node scripts/verify-node-packaging.mjs" {
        node scripts/verify-node-packaging.mjs
    }
}

if (-not $NoTests) {
    if (Test-CommandAvailable "cargo-nextest") {
        Invoke-Step "cargo nextest run --tests --locked" {
            cargo nextest run --tests --locked
        }
    }
    else {
        Invoke-Step "cargo test --tests --locked" {
            cargo test --tests --locked
        }
    }
}

if (-not $NoAudit) {
    if (-not (Test-CommandAvailable "cargo-audit")) {
        throw "cargo-audit is not installed. Install it or run scripts/check.ps1 -NoAudit."
    }

    Invoke-Step "cargo audit" { cargo audit }
}
