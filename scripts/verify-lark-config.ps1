#!/usr/bin/env pwsh
# Verify Lark Configuration

param(
    [switch]$TestConnection,
    [switch]$Help
)

if ($Help) {
    Write-Host "Usage: .\verify-lark-config.ps1 [-TestConnection] [-Help]"
    exit 0
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Resolve-Path "$scriptDir\.."
Set-Location $projectRoot

Write-Host "=== Lark Configuration Verification ===" -ForegroundColor Cyan
Write-Host ""

# Load .env
$envFile = "$projectRoot\.env"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        if ($line -and !$line.StartsWith('#') -and $line -match '^([^=]+)="?([^"]*)"?$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim()
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }
}

# Check required variables
$required = @{
    "LARK_APP_ID" = "飞书 App ID"
    "LARK_APP_SECRET" = "飞书 App Secret"
    "KIMI_API_KEY" = "Kimi API Key"
    "DATABASE_URL" = "数据库 URL"
    "JWT_SECRET" = "JWT Secret"
}

Write-Host "1. Checking Environment Variables:" -ForegroundColor Yellow
$allOk = $true
foreach ($var in $required.GetEnumerator()) {
    $value = [Environment]::GetEnvironmentVariable($var.Key, "Process")
    if ($value) {
        $masked = if ($value.Length -gt 10) { $value.Substring(0, 10) + "..." } else { $value }
        Write-Host "   ✓ $($var.Value): $masked" -ForegroundColor Green
    } else {
        Write-Host "   ✗ $($var.Value): NOT SET" -ForegroundColor Red
        $allOk = $false
    }
}
Write-Host ""

# Check config file
Write-Host "2. Checking Configuration File:" -ForegroundColor Yellow
$configFile = "$projectRoot\config\beebotos.toml"
if (Test-Path $configFile) {
    $config = Get-Content $configFile -Raw
    if ($config -match 'enabled\s*=\s*true' -and $config -match '\[channels\.lark\]') {
        Write-Host "   ✓ Lark channel is enabled in config" -ForegroundColor Green
    } else {
        Write-Host "   ✗ Lark channel not enabled in config" -ForegroundColor Red
        $allOk = $false
    }
} else {
    Write-Host "   ✗ Config file not found" -ForegroundColor Red
    $allOk = $false
}
Write-Host ""

# Check code compilation
Write-Host "3. Checking Code Compilation:" -ForegroundColor Yellow
cargo check -p beebotos-agents --lib 2>&1 | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✓ beebotos-agents compiles" -ForegroundColor Green
} else {
    Write-Host "   ✗ beebotos-agents has errors" -ForegroundColor Red
    $allOk = $false
}

cargo check -p beebotos-gateway --lib 2>&1 | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✓ beebotos-gateway compiles" -ForegroundColor Green
} else {
    Write-Host "   ✗ beebotos-gateway has errors" -ForegroundColor Red
    $allOk = $false
}
Write-Host ""

# Test connection to Lark API
if ($TestConnection) {
    Write-Host "4. Testing Lark API Connection:" -ForegroundColor Yellow
    
    $appId = [Environment]::GetEnvironmentVariable("LARK_APP_ID", "Process")
    $appSecret = [Environment]::GetEnvironmentVariable("LARK_APP_SECRET", "Process")
    
    try {
        $body = @{
            app_id = $appId
            app_secret = $appSecret
        } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri "https://open.feishu.cn/open-apis/auth/v3/app_access_token/internal" `
            -Method POST -Body $body -ContentType "application/json" -TimeoutSec 10
        
        if ($response.code -eq 0) {
            Write-Host "   ✓ Lark API connection successful" -ForegroundColor Green
            Write-Host "     App Access Token: $($response.app_access_token.Substring(0, 10))..." -ForegroundColor Gray
        } else {
            Write-Host "   ✗ Lark API error: $($response.msg)" -ForegroundColor Red
            $allOk = $false
        }
    } catch {
        Write-Host "   ✗ Failed to connect to Lark API: $_" -ForegroundColor Red
        $allOk = $false
    }
    Write-Host ""
}

# Summary
Write-Host "=== Verification Summary ===" -ForegroundColor Cyan
if ($allOk) {
    Write-Host "✅ All checks passed! Ready to test Lark messages." -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "  1. Run: docker-compose -f docker-compose.test.yml up --build" -ForegroundColor White
    Write-Host "  2. Or use WSL2: wsl cargo run -p beebotos-gateway" -ForegroundColor White
    Write-Host "  3. Test webhook: .\scripts\test-lark-message.ps1 -SimulateWebhook" -ForegroundColor White
} else {
    Write-Host "❌ Some checks failed. Please fix the issues above." -ForegroundColor Red
}
