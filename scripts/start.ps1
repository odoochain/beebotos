# BeeBotOS Gateway 启动脚本（统一 TOML 配置）
# 使用 config/beebotos.toml 作为单一配置文件

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "BeeBotOS Gateway" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 检查配置文件
$configPath = "config/beebotos.toml"
if (-not (Test-Path $configPath)) {
    Write-Host "错误: 配置文件 $configPath 不存在" -ForegroundColor Red
    Write-Host ""
    Write-Host "请创建配置文件:"
    Write-Host "  1. 复制示例配置: Copy-Item config/beebotos.example.toml $configPath" -ForegroundColor Yellow
    Write-Host "  2. 编辑配置: notepad $configPath" -ForegroundColor Yellow
    Write-Host "  3. 设置环境变量（可选）: notepad .env" -ForegroundColor Yellow
    exit 1
}

# 加载 .env 文件（如果存在）
$envFile = ".env"
if (Test-Path $envFile) {
    Write-Host "加载环境变量: $envFile" -ForegroundColor Green
    Get-Content $envFile | ForEach-Object {
        if ($_ -match '^([^#][^=]*)=(.*)$') {
            $name = $matches[1].Trim()
            $value = $matches[2].Trim()
            [Environment]::SetEnvironmentVariable($name, $value, "Process")
        }
    }
}

# 显示配置信息
Write-Host ""
Write-Host "配置信息:" -ForegroundColor Green
Write-Host "  - 配置文件: $configPath" -ForegroundColor White

# 检查必需的环境变量
$requiredVars = @("JWT_SECRET")
$missingVars = @()

foreach ($var in $requiredVars) {
    $value = [Environment]::GetEnvironmentVariable($var)
    if (-not $value) {
        $missingVars += $var
    }
}

if ($missingVars.Count -gt 0) {
    Write-Host ""
    Write-Host "警告: 以下必需的环境变量未设置:" -ForegroundColor Yellow
    foreach ($var in $missingVars) {
        Write-Host "  - $var" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "请设置这些环境变量，或在 .env 文件中定义" -ForegroundColor Yellow
}

# 确定可执行文件路径
$exePaths = @(
    "target/release/beebotos-gateway.exe",
    "target/debug/beebotos-gateway.exe"
)

$exePath = $null
foreach ($path in $exePaths) {
    if (Test-Path $path) {
        $exePath = $path
        break
    }
}

if (-not $exePath) {
    Write-Host ""
    Write-Host "错误: 找不到 beebotos-gateway.exe" -ForegroundColor Red
    Write-Host "请先编译项目:" -ForegroundColor Yellow
    Write-Host "  cargo build --release -p beebotos-gateway" -ForegroundColor Cyan
    exit 1
}

Write-Host "  - 可执行文件: $exePath" -ForegroundColor White
Write-Host ""

# 启动 Gateway
Write-Host "启动 Gateway..." -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan

& $exePath
