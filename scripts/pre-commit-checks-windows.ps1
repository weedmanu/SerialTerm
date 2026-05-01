<#
Hook de validation Windows pour SerialTerm.

Usage :
    pwsh -ExecutionPolicy Bypass -File .\scripts\pre-commit-checks-windows.ps1
#>

param()

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$targetTriple = "x86_64-pc-windows-gnu"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir
Set-Location $projectRoot

function Require-Command {
    param(
        [string]$Command,
        [string]$Hint
    )

    if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) {
        throw "[pre-commit-windows] outil manquant: $Command`n[pre-commit-windows] correction: $Hint"
    }
}

Require-Command -Command "cargo" -Hint "installez Rust puis relancez scripts/install-deps-windows.ps1"
Require-Command -Command "rustup" -Hint "installez rustup puis relancez scripts/install-deps-windows.ps1"
Require-Command -Command "cargo-audit" -Hint "installez cargo-audit ou relancez scripts/install-deps-windows.ps1"
Require-Command -Command "cargo-deny" -Hint "installez cargo-deny ou relancez scripts/install-deps-windows.ps1"

Write-Host "[pre-commit-windows] rustup target add $targetTriple"
rustup target add $targetTriple | Out-Null

Write-Host "[pre-commit-windows] cargo fmt --all -- --check"
cargo fmt --all -- --check

Write-Host "[pre-commit-windows] cargo check --all-targets"
cargo check --all-targets --target $targetTriple

Write-Host "[pre-commit-windows] cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features --target $targetTriple -- -D warnings

Write-Host "[pre-commit-windows] RUSTDOCFLAGS=-D warnings cargo doc --no-deps --document-private-items"
$previousRustdocFlags = $env:RUSTDOCFLAGS
$env:RUSTDOCFLAGS = "-D warnings"
try {
    cargo doc --no-deps --document-private-items
} finally {
    if ($null -eq $previousRustdocFlags) {
        Remove-Item Env:RUSTDOCFLAGS -ErrorAction SilentlyContinue
    } else {
        $env:RUSTDOCFLAGS = $previousRustdocFlags
    }
}

Write-Host "[pre-commit-windows] cargo test --all-targets --target $targetTriple"
cargo test --all-targets --target $targetTriple

Write-Host "[pre-commit-windows] cargo audit -q"
cargo audit -q

Write-Host "[pre-commit-windows] cargo deny check"
cargo deny check

if (Get-Command cargo-machete -ErrorAction SilentlyContinue) {
    Write-Host "[pre-commit-windows] cargo machete --with-metadata (hors vendor/)"
    $macheteOut = cargo machete --with-metadata 2>&1
    if ($LASTEXITCODE -ne 0) {
        $findings = $macheteOut | Where-Object { $_ -match " -- " -and $_ -notmatch "vendor/" }
        if ($findings) {
            $macheteOut | ForEach-Object { Write-Host $_ }
            throw "[pre-commit-windows] cargo machete a détecté des dépendances inutilisées."
        }
    }
} else {
    Write-Host "[pre-commit-windows] cargo-machete absent — skipped (relancer scripts/install-deps-windows.ps1)"
}

Write-Host "[pre-commit-windows] cargo build --release --target $targetTriple"
cargo build --release --target $targetTriple

Write-Host "[pre-commit-windows] OK"