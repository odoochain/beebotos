# Channel Factory Verification Script
# This script verifies the channel factory implementations

Write-Host "=== BeeBotOS Channel Factory Verification ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: Check factory files exist
$factories = @(
    "crates/agents/src/channels/lark_factory.rs",
    "crates/agents/src/channels/dingtalk_factory.rs",
    "crates/agents/src/channels/telegram_factory.rs",
    "crates/agents/src/channels/discord_factory.rs",
    "crates/agents/src/channels/slack_factory.rs"
)

Write-Host "1. Checking factory files..." -ForegroundColor Yellow
$allExist = $true
foreach ($factory in $factories) {
    if (Test-Path $factory) {
        Write-Host "   ✓ $factory" -ForegroundColor Green
    } else {
        Write-Host "   ✗ $factory (MISSING)" -ForegroundColor Red
        $allExist = $false
    }
}
Write-Host ""

# Test 2: Check mod.rs exports
Write-Host "2. Checking mod.rs exports..." -ForegroundColor Yellow
$modContent = Get-Content "crates/agents/src/channels/mod.rs" -Raw
$exports = @("LarkChannelFactory", "DingTalkChannelFactory", "TelegramChannelFactory", "DiscordChannelFactory", "SlackChannelFactory")
foreach ($export in $exports) {
    if ($modContent -match $export) {
        Write-Host "   ✓ $export exported" -ForegroundColor Green
    } else {
        Write-Host "   ✗ $export (NOT EXPORTED)" -ForegroundColor Red
    }
}
Write-Host ""

# Test 3: Check lib.rs exports
Write-Host "3. Checking lib.rs exports..." -ForegroundColor Yellow
$libContent = Get-Content "crates/agents/src/lib.rs" -Raw
foreach ($export in $exports) {
    if ($libContent -match $export) {
        Write-Host "   ✓ $export exported" -ForegroundColor Green
    } else {
        Write-Host "   ✗ $export (NOT EXPORTED)" -ForegroundColor Red
    }
}
Write-Host ""

# Test 4: Check Gateway integration
Write-Host "4. Checking Gateway integration..." -ForegroundColor Yellow
$mainContent = Get-Content "apps/gateway/src/main.rs" -Raw
$integrations = @(
    "SlackChannelFactory",
    "registry.register(Box::new(SlackChannelFactory",
    "channels.slack"
)
foreach ($integration in $integrations) {
    if ($mainContent -match [regex]::Escape($integration)) {
        Write-Host "   ✓ $integration found" -ForegroundColor Green
    } else {
        Write-Host "   ✗ $integration (NOT FOUND)" -ForegroundColor Red
    }
}
Write-Host ""

# Test 5: Check downloader_v2.rs
Write-Host "5. Checking Media Downloader V2..." -ForegroundColor Yellow
$downloaderPath = "crates/agents/src/media/downloader_v2.rs"
if (Test-Path $downloaderPath) {
    $content = Get-Content $downloaderPath -Raw
    $features = @("LazyUrl", "MediaCache", "MediaDownloaderV2", "PlatformMediaDownloader")
    foreach ($feature in $features) {
        if ($content -match $feature) {
            Write-Host "   ✓ $feature implemented" -ForegroundColor Green
        } else {
            Write-Host "   ✗ $feature (NOT FOUND)" -ForegroundColor Red
        }
    }
} else {
    Write-Host "   ✗ downloader_v2.rs not found" -ForegroundColor Red
}
Write-Host ""

# Test 6: Check config supports all channels
Write-Host "6. Checking configuration support..." -ForegroundColor Yellow
$configContent = Get-Content "apps/gateway/src/config.rs" -Raw
$channels = @("lark", "dingtalk", "telegram", "discord", "slack")
foreach ($channel in $channels) {
    if ($configContent -match "channels\.$channel") {
        Write-Host "   ✓ $channel config supported" -ForegroundColor Green
    } else {
        Write-Host "   ✗ $channel config (NOT SUPPORTED)" -ForegroundColor Red
    }
}
Write-Host ""

Write-Host "=== Verification Complete ===" -ForegroundColor Cyan
