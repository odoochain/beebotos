#!/usr/bin/env pwsh
# BeeBotOS Gateway Runner for Windows PowerShell
# This script loads .env file and runs the Gateway

param(
    [switch]$CheckOnly,
    [switch]$Help
)

if ($Help) {
    Write-Host "Usage: .\run-gateway.ps1 [-CheckOnly] [-Help]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -CheckOnly    Only check compilation, don't run"
    Write-Host "  -Help         Show this help message"
    exit 0
}

Write-Host "=== BeeBotOS Gateway Runner ===" -ForegroundColor Cyan

# Get project root
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Resolve-Path "$scriptDir\.."
Set-Location $projectRoot

# Load .env file
$envFile = "$projectRoot\.env"
if (Test-Path $envFile) {
    Write-Host "Loading environment from .env..." -ForegroundColor Yellow
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        # Skip comments and empty lines
        if ($line -and !$line.StartsWith('#')) {
            # Handle KEY="VALUE" format
            if ($line -match '^([^=]+)="?([^"]*)"?$') {
                $key = $matches[1].Trim()
                $value = $matches[2].Trim()
                [Environment]::SetEnvironmentVariable($key, $value, "Process")
                Write-Host "  Set $key" -ForegroundColor Gray
            }
        }
    }
} else {
    Write-Warning ".env file not found at $envFile"
}

# Verify critical environment variables
$criticalVars = @('DATABASE_URL', 'JWT_SECRET', 'KIMI_API_KEY', 'LARK_APP_ID', 'LARK_APP_SECRET')
$missing = @()
foreach ($var in $criticalVars) {
    $value = [Environment]::GetEnvironmentVariable($var, "Process")
    if (-not $value) {
        $missing += $var
    }
}

if ($missing.Count -gt 0) {
    Write-Warning "Missing environment variables: $($missing -join ', ')"
}

if ($CheckOnly) {
    Write-Host "`nChecking compilation..." -ForegroundColor Yellow
    cargo check -p beebotos-gateway --lib
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`n✅ Compilation check passed!" -ForegroundColor Green
    } else {
        Write-Host "`n❌ Compilation check failed!" -ForegroundColor Red
    }
} else {
    Write-Host "`nStarting BeeBotOS Gateway..." -ForegroundColor Green
    Write-Host "Press Ctrl+C to stop`n" -ForegroundColor Gray
    cargo run -p beebotos-gateway
}
