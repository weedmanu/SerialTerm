<#
Script d'installation des dépendances Windows pour SerialTerm

Usage (PowerShell admin recommandé) :
    powershell -ExecutionPolicy Bypass -File .\scripts\install-deps-windows.ps1
#>

$ErrorActionPreference = "Stop"

function Ensure-CargoTool {
    param(
        [string]$Binary,
        [string]$Package
    )

    if (Get-Command $Binary -ErrorAction SilentlyContinue) {
        Write-Host "✓ $Binary déjà installé"
        return
    }

    Write-Host "📦 Installation $Package via cargo install --locked..."
    try {
        cargo install --locked $Package
    } catch {
        Write-Host "⚠ Échec avec --locked, nouvelle tentative sans --locked..." -ForegroundColor Yellow
        cargo install $Package
    }
}

function Add-PathIfMissing {
    param([string]$PathToAdd)

    if (-not (Test-Path $PathToAdd)) {
        return
    }

    $segments = $env:PATH -split ';'
    if ($segments -notcontains $PathToAdd) {
        $env:PATH = "$PathToAdd;$env:PATH"
    }
}

Write-Host "===============================================================" -ForegroundColor Cyan
Write-Host "  SerialTerm - Installation dépendances Windows" -ForegroundColor Cyan
Write-Host "===============================================================" -ForegroundColor Cyan

if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Error "winget n'est pas disponible. Installez 'App Installer' depuis Microsoft Store."
}

function Ensure-Command {
    param(
        [string]$Command,
        [string]$WingetId,
        [string]$DisplayName
    )

    if (Get-Command $Command -ErrorAction SilentlyContinue) {
        Write-Host "✓ $DisplayName déjà installé"
        return
    }

    Write-Host "📦 Installation $DisplayName..."
    winget install --id $WingetId --accept-source-agreements --accept-package-agreements --silent
}

Ensure-Command -Command "cargo" -WingetId "Rustlang.Rustup" -DisplayName "Rust"
Ensure-Command -Command "git" -WingetId "Git.Git" -DisplayName "Git"
Ensure-Command -Command "pwsh" -WingetId "Microsoft.PowerShell" -DisplayName "PowerShell 7"

if (-not (Test-Path "C:\msys64\usr\bin\bash.exe")) {
    Write-Host "📦 Installation MSYS2 (GTK runtime/build)..."
    winget install --id "MSYS2.MSYS2" --accept-source-agreements --accept-package-agreements --silent
} else {
    Write-Host "✓ MSYS2 déjà installé"
}

if (Test-Path "C:\msys64\usr\bin\bash.exe") {
    Write-Host "↻ Mise à jour MSYS2 et installation toolchain mingw64 GTK4..."

    & "C:\msys64\usr\bin\bash.exe" -lc "pacman -Syu --noconfirm" | Out-Host
    & "C:\msys64\usr\bin\bash.exe" -lc "pacman -Su --noconfirm" | Out-Host
    & "C:\msys64\usr\bin\bash.exe" -lc "pacman -S --noconfirm --needed mingw-w64-x86_64-toolchain mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita" | Out-Host
}

# Rendre immédiatement disponibles gcc/pkg-config GTK du toolchain mingw64
Add-PathIfMissing -PathToAdd "C:\msys64\mingw64\bin"
Add-PathIfMissing -PathToAdd "C:\msys64\usr\bin"

if (-not (Get-Command x86_64-w64-mingw32-gcc -ErrorAction SilentlyContinue)) {
    Write-Error "x86_64-w64-mingw32-gcc introuvable. Vérifiez l'installation MSYS2 mingw64."
}

if (-not (Get-Command pkg-config -ErrorAction SilentlyContinue)) {
    Write-Error "pkg-config introuvable. Vérifiez l'installation MSYS2 mingw64."
}

if (Get-Command rustup -ErrorAction SilentlyContinue) {
    Write-Host "↻ Installation des composants Rust requis..."
    rustup component add rustfmt clippy
    rustup target add x86_64-pc-windows-gnu
}

Write-Host "↻ Installation des outils Cargo de validation Windows..."
Ensure-CargoTool -Binary "cargo-audit" -Package "cargo-audit"
Ensure-CargoTool -Binary "cargo-deny" -Package "cargo-deny"
Ensure-CargoTool -Binary "cargo-machete" -Package "cargo-machete"

Write-Host ""
Write-Host "✓ Dépendances Windows installées" -ForegroundColor Green
Write-Host "Étape suivante :" -ForegroundColor Yellow
Write-Host "  pwsh -ExecutionPolicy Bypass -File .\scripts\pre-commit-checks-windows.ps1"
Write-Host "  pwsh -ExecutionPolicy Bypass -File .\scripts\build-exe.ps1 -IncludeGtkRuntime"
