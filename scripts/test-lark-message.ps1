#!/usr/bin/env pwsh
# Lark Message Test Script for Windows
# This script tests the Lark message handling flow without running the full Gateway

param(
    [switch]$RunUnitTests,
    [switch]$SimulateWebhook,
    [switch]$Help
)

if ($Help) {
    Write-Host "Usage: .\test-lark-message.ps1 [-RunUnitTests] [-SimulateWebhook] [-Help]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -RunUnitTests     Run Rust unit tests for Lark message handling"
    Write-Host "  -SimulateWebhook  Simulate a Lark webhook message"
    Write-Host "  -Help             Show this help message"
    exit 0
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Resolve-Path "$scriptDir\.."
Set-Location $projectRoot

Write-Host "=== Lark Message Test ===" -ForegroundColor Cyan
Write-Host ""

# Load environment variables
$envFile = "$projectRoot\.env"
if (Test-Path $envFile) {
    Write-Host "Loading environment from .env..." -ForegroundColor Yellow
    Get-Content $envFile | ForEach-Object {
        $line = $_.Trim()
        if ($line -and !$line.StartsWith('#') -and $line -match '^([^=]+)="?([^"]*)"?$') {
            $key = $matches[1].Trim()
            $value = $matches[2].Trim()
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }
}

# Check Lark configuration
$larkAppId = [Environment]::GetEnvironmentVariable("LARK_APP_ID", "Process")
$larkAppSecret = [Environment]::GetEnvironmentVariable("LARK_APP_SECRET", "Process")

Write-Host "Lark Configuration:" -ForegroundColor Yellow
Write-Host "  App ID: $($larkAppId.Substring(0, [Math]::Min(10, $larkAppId.Length)))..." -ForegroundColor Gray
Write-Host "  App Secret: $($larkAppSecret.Substring(0, [Math]::Min(5, $larkAppSecret.Length)))..." -ForegroundColor Gray
Write-Host ""

if ($RunUnitTests) {
    Write-Host "Running unit tests..." -ForegroundColor Yellow
    Write-Host "Note: This requires dependencies to compile." -ForegroundColor Gray
    Write-Host ""
    
    cargo test -p beebotos-agents test_lark_message -- --nocapture 2>&1 | ForEach-Object {
        if ($_ -match "✓|✅|passed") {
            Write-Host $_ -ForegroundColor Green
        } elseif ($_ -match "✗|❌|failed") {
            Write-Host $_ -ForegroundColor Red
        } else {
            Write-Host $_
        }
    }
}

if ($SimulateWebhook) {
    Write-Host "Simulating Lark webhook message..." -ForegroundColor Yellow
    Write-Host ""
    
    # Create a test message
    $testMessage = @{
        schema = "2.0"
        header = @{
            event_id = "test_event_$(Get-Random)"
            token = "test_token"
            create_time = ([DateTimeOffset]::UtcNow.ToUnixTimeSeconds()).ToString()
            event_type = "im.message.receive_v1"
            app_id = $larkAppId
            tenant_key = "tenant_key"
        }
        event = @{
            message = @{
                message_id = "om_$(-join ((65..90) + (97..122) | Get-Random -Count 16 | ForEach-Object { [char]$_ }))"
                create_time = ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()).ToString()
                chat_id = "oc_test_chat_123"
                chat_type = "p2p"
                message_type = "text"
                content = '{"text":"Hello from Lark test message"}'
            }
            sender = @{
                sender_id = @{
                    union_id = "on_test_user"
                    open_id = "ou_test_user_123"
                }
                sender_type = "user"
                tenant_key = "tenant_key"
            }
        }
    } | ConvertTo-Json -Depth 10
    
    Write-Host "Test Message:" -ForegroundColor Cyan
    Write-Host $testMessage -ForegroundColor Gray
    Write-Host ""
    
    # Try to send to local Gateway if running
    $gatewayUrl = "http://localhost:8080/webhook/lark"
    
    try {
        Write-Host "Sending to Gateway at $gatewayUrl..." -ForegroundColor Yellow
        $response = Invoke-RestMethod -Uri $gatewayUrl -Method POST -Body $testMessage -ContentType "application/json" -TimeoutSec 5
        Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Green
    } catch {
        if ($_.Exception.Response) {
            $statusCode = $_.Exception.Response.StatusCode.value__
            Write-Host "Gateway returned HTTP $statusCode" -ForegroundColor Yellow
            
            if ($statusCode -eq 404) {
                Write-Host "Gateway is not running. Start it first with: cargo run -p beebotos-gateway" -ForegroundColor Red
            }
        } else {
            Write-Host "Cannot connect to Gateway. Is it running?" -ForegroundColor Red
            Write-Host "Start Gateway with: cargo run -p beebotos-gateway" -ForegroundColor Gray
        }
    }
}

# Always show message flow diagram
Write-Host ""
Write-Host "=== Lark Message Flow ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "  1. User sends message in Lark" -ForegroundColor White
Write-Host "     ↓" -ForegroundColor Gray
Write-Host "  2. Lark WebSocket/Event pushes to Gateway" -ForegroundColor White
Write-Host "     ↓" -ForegroundColor Gray
Write-Host "  3. Gateway receives via /webhook/lark" -ForegroundColor White
Write-Host "     ↓" -ForegroundColor Gray
Write-Host "  4. Message parsed and deduplicated" -ForegroundColor White
Write-Host "     ↓" -ForegroundColor Gray
Write-Host "  5. ChannelEvent sent to processing loop" -ForegroundColor White
Write-Host "     ↓" -ForegroundColor Gray
Write-Host "  6. AI processes message and generates reply" -ForegroundColor White
Write-Host "     ↓" -ForegroundColor Gray
Write-Host "  7. Reply sent back to Lark via API" -ForegroundColor White
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
