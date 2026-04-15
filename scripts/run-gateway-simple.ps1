#!/usr/bin/env pwsh
# BeeBotOS Gateway 简化启动脚本
# 使用最小配置文件，通过环境变量覆盖

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  BeeBotOS Gateway 启动脚本" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# 检查可执行文件
$exePath = "target/release/beebotos-gateway.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "错误: 找不到 $exePath"
    exit 1
}

# 检查最小配置文件
$configPath = "config/beebotos.minimal.toml"
if (-not (Test-Path $configPath)) {
    Write-Error "错误: 找不到 $configPath"
    exit 1
}

# 设置必需的环境变量
Write-Host "检查必需配置..." -ForegroundColor Green

# JWT Secret
if (-not $env:JWT_SECRET) {
    $env:JWT_SECRET = "your-super-secret-jwt-key-change-this-in-production-min-32-chars"
    Write-Host "  JWT_SECRET: 使用默认值（生产环境请修改）" -ForegroundColor Yellow
} else {
    if ($env:JWT_SECRET.Length -lt 32) {
        Write-Error "错误: JWT_SECRET 必须至少 32 字符"
        exit 1
    }
    Write-Host "  JWT_SECRET: 已设置 ($( $env:JWT_SECRET.Length ) 字符)" -ForegroundColor Green
}

# 数据库
if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://postgres:A111222@localhost:5432/beebotos"
    Write-Host "  DATABASE_URL: 使用默认值" -ForegroundColor Yellow
} else {
    Write-Host "  DATABASE_URL: 已设置" -ForegroundColor Green
}

# Kimi API Key
if (-not $env:KIMI_API_KEY) {
    Write-Host "  KIMI_API_KEY: 未设置（Kimi 功能将不可用）" -ForegroundColor Yellow
} else {
    Write-Host "  KIMI_API_KEY: 已设置" -ForegroundColor Green
}

# 使用临时配置文件（替换占位符）
Write-Host ""
Write-Host "生成临时配置文件..." -ForegroundColor Green

$tempConfig = Get-Content $configPath -Raw

# 替换占位符
if ($env:KIMI_API_KEY) {
    $tempConfig = $tempConfig.Replace('api_key = ""  # 必需：设置 KIMI_API_KEY 环境变量', "api_key = `"$env:KIMI_API_KEY`"")
}

# 保存临时配置
$tempPath = "config/beebotos.temp.toml"
$tempConfig | Set-Content $tempPath -Encoding UTF8

Write-Host "  临时配置: $tempPath" -ForegroundColor Gray

# 备份原配置
$backupPath = "config/beebotos.toml.backup"
if (Test-Path "config/beebotos.toml") {
    Copy-Item "config/beebotos.toml" $backupPath -Force
}

# 使用临时配置替换原配置
Copy-Item $tempPath "config/beebotos.toml" -Force

Write-Host ""
Write-Host "启动 BeeBotOS Gateway..." -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan

# 启动 Gateway
try {
    & $exePath
} finally {
    # 恢复原始配置
    if (Test-Path $backupPath) {
        Copy-Item $backupPath "config/beebotos.toml" -Force
        Remove-Item $backupPath -Force
        Write-Host ""
        Write-Host "已恢复原始配置文件" -ForegroundColor Gray
    }
    if (Test-Path $tempPath) {
        Remove-Item $tempPath -Force
    }
}
