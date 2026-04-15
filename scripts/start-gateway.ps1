#!/usr/bin/env pwsh
# BeeBotOS Gateway 启动脚本

param(
    [string]$EnvFile = ".env",
    [int]$Port = 8080,
    [switch]$DevMode
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  BeeBotOS Gateway 启动脚本" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# 检查 .env 文件
if (-not (Test-Path $EnvFile)) {
    Write-Error "错误: 找不到环境配置文件 $EnvFile"
    Write-Host "请复制 .env.example 为 .env 并配置你的 API 密钥" -ForegroundColor Yellow
    exit 1
}

# 加载环境变量
Write-Host "[1/5] 加载环境变量..." -ForegroundColor Green
$envContent = Get-Content $EnvFile -Raw
$envLines = $envContent -split "`n"

foreach ($line in $envLines) {
    $line = $line.Trim()
    if ($line -match '^\s*#' -or $line -match '^\s*$') { continue }
    if ($line -match '^([^=]+)=(.*)$') {
        $key = $matches[1].Trim()
        $value = $matches[2].Trim() -replace '^["\'']|["\'']$', ''
        [Environment]::SetEnvironmentVariable($key, $value, "Process")
    }
}

# 验证必需配置
Write-Host "[2/5] 验证配置..." -ForegroundColor Green
$requiredVars = @(
    "LARK_APP_ID",
    "LARK_APP_SECRET", 
    "LARK_VERIFICATION_TOKEN",
    "KIMI_API_KEY"
)

$missing = @()
foreach ($var in $requiredVars) {
    $val = [Environment]::GetEnvironmentVariable($var, "Process")
    if (-not $val -or $val -match "your-|-key-here|xxxxxxxx") {
        $missing += $var
    }
}

if ($missing.Count -gt 0) {
    Write-Error "错误: 以下必需环境变量未配置或仍为默认值:"
    foreach ($var in $missing) {
        Write-Host "  - $var" -ForegroundColor Red
    }
    Write-Host "`n请编辑 $EnvFile 文件配置这些变量" -ForegroundColor Yellow
    exit 1
}

# 检查数据库连接
Write-Host "[3/5] 检查数据库..." -ForegroundColor Green
$databaseUrl = [Environment]::GetEnvironmentVariable("DATABASE_URL", "Process")
Write-Host "  数据库: $databaseUrl" -ForegroundColor Gray

# 检查可执行文件
Write-Host "[4/5] 检查可执行文件..." -ForegroundColor Green
$exePaths = @(
    "target/release/beebot.exe",
    "target/release/beebotos-gateway.exe"
)

$exePath = $null
foreach ($path in $exePaths) {
    if (Test-Path $path) {
        $exePath = $path
        break
    }
}

if (-not $exePath) {
    Write-Error "错误: 找不到网关可执行文件"
    Write-Host "请先运行: cargo build --release" -ForegroundColor Yellow
    exit 1
}

Write-Host "  使用: $exePath" -ForegroundColor Gray

# 启动服务
Write-Host "[5/5] 启动 BeeBotOS Gateway..." -ForegroundColor Green
Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "  服务启动成功!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host "  HTTP 端口: $Port" -ForegroundColor Cyan
Write-Host "  Webhook 地址: http://localhost:$Port/webhook/lark" -ForegroundColor Cyan
Write-Host "  健康检查: http://localhost:$Port/health" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Green
Write-Host ""

if ($DevMode) {
    Write-Host "开发模式: 启用详细日志" -ForegroundColor Yellow
    $env:RUST_LOG = "debug"
} else {
    $env:RUST_LOG = "info"
}

# 启动程序
& $exePath
