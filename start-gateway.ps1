# BeeBotOS Gateway 启动脚本
# 此脚本确保从正确的目录运行

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

Write-Host "Starting BeeBotOS Gateway..." -ForegroundColor Green
Write-Host "Working directory: $(Get-Location)" -ForegroundColor Gray
Write-Host "Config file: config/beebotos.toml" -ForegroundColor Gray

# 检查配置文件是否存在
if (-not (Test-Path "config/beebotos.toml")) {
    Write-Error "Config file not found: config/beebotos.toml"
    exit 1
}

# 显示当前 LLM 配置
$config = Get-Content "config/beebotos.toml" | Select-String -Pattern "default_provider|fallback_chain"
Write-Host "LLM Configuration:" -ForegroundColor Cyan
$config | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }

# 启动 Gateway
.\target\release\beebotos-gateway.exe
