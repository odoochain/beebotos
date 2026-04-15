#!/usr/bin/env pwsh
# BeeBotOS 数据库初始化脚本

param(
    [string]$DatabaseUrl = $env:DATABASE_URL,
    [switch]$SkipMigrations,
    [switch]$Reset
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  BeeBotOS 数据库初始化" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# 检查数据库连接字符串
if (-not $DatabaseUrl) {
    # 尝试从 .env 文件加载
    if (Test-Path ".env") {
        Get-Content ".env" | ForEach-Object {
            if ($_ -match '^DATABASE_URL=(.+)$') {
                $DatabaseUrl = $matches[1].Trim() -replace '^["\'']|["\'']$', ''
            }
        }
    }
    
    if (-not $DatabaseUrl) {
        Write-Error "错误: 未提供 DATABASE_URL"
        Write-Host "请设置环境变量或在 .env 文件中配置 DATABASE_URL" -ForegroundColor Yellow
        exit 1
    }
}

Write-Host "数据库 URL: $DatabaseUrl" -ForegroundColor Gray

# 解析数据库名称
if ($DatabaseUrl -match '/([^/]+?)(\?|$)') {
    $dbName = $matches[1]
    Write-Host "数据库名: $dbName" -ForegroundColor Gray
}

# 检查 psql 是否可用
$psql = Get-Command psql -ErrorAction SilentlyContinue
if (-not $psql) {
    Write-Error "错误: 找不到 psql 命令"
    Write-Host "请安装 PostgreSQL 客户端工具" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "[1/5] 检查数据库连接..." -ForegroundColor Green

try {
    $result = psql $DatabaseUrl -c "SELECT version();" 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "连接失败"
    }
    Write-Host "  ✓ 数据库连接成功" -ForegroundColor Green
} catch {
    Write-Error "错误: 无法连接到数据库"
    Write-Host "请检查:" -ForegroundColor Yellow
    Write-Host "  1. PostgreSQL 服务是否运行" -ForegroundColor Yellow
    Write-Host "  2. 数据库是否存在" -ForegroundColor Yellow
    Write-Host "  3. 连接字符串是否正确" -ForegroundColor Yellow
    exit 1
}

# 检查并创建扩展
Write-Host ""
Write-Host "[2/5] 检查 PostgreSQL 扩展..." -ForegroundColor Green

$extensions = @("uuid-ossp", "pgvector")
foreach ($ext in $extensions) {
    try {
        psql $DatabaseUrl -c "CREATE EXTENSION IF NOT EXISTS \"$ext\";" 2>&1 | Out-Null
        Write-Host "  ✓ 扩展 $ext 已就绪" -ForegroundColor Green
    } catch {
        Write-Warning "  ⚠ 无法创建扩展 $ext`: $_"
    }
}

# 如果需要重置，删除所有表
if ($Reset) {
    Write-Host ""
    Write-Host "[3/5] 重置数据库 (删除所有数据)..." -ForegroundColor Yellow
    
    $dropSql = @"
DO \$\$
DECLARE
    r RECORD;
BEGIN
    FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public') LOOP
        EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(r.tablename) || ' CASCADE';
    END LOOP;
END \$\$;
"@
    
    psql $DatabaseUrl -c $dropSql 2>&1 | Out-Null
    Write-Host "  ✓ 所有表已删除" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[3/5] 跳过重置 (使用 -Reset 参数可清空数据库)" -ForegroundColor Gray
}

# 运行迁移
if (-not $SkipMigrations) {
    Write-Host ""
    Write-Host "[4/5] 运行数据库迁移..." -ForegroundColor Green
    
    # 检查是否有 sqlx 工具
    $sqlx = Get-Command sqlx -ErrorAction SilentlyContinue
    
    if ($sqlx) {
        # 使用 sqlx-cli 运行迁移
        try {
            sqlx migrate run --database-url $DatabaseUrl
            Write-Host "  ✓ 迁移完成" -ForegroundColor Green
        } catch {
            Write-Warning "  ⚠ sqlx migrate 失败，尝试手动执行 SQL 文件"
        }
    } else {
        Write-Host "  未找到 sqlx-cli，手动执行 SQL 迁移文件..." -ForegroundColor Yellow
        
        # 手动执行迁移文件
        $migrationFiles = Get-ChildItem "migrations/*.sql" | Sort-Object Name
        foreach ($file in $migrationFiles) {
            Write-Host "    执行: $($file.Name)" -ForegroundColor Gray
            try {
                psql $DatabaseUrl -f $file.FullName 2>&1 | Out-Null
                Write-Host "    ✓ 成功" -ForegroundColor Green
            } catch {
                Write-Warning "    ⚠ 失败: $_"
            }
        }
    }
} else {
    Write-Host ""
    Write-Host "[4/5] 跳过迁移 (使用 -SkipMigrations 参数)" -ForegroundColor Gray
}

# 验证表结构
Write-Host ""
Write-Host "[5/5] 验证数据库结构..." -ForegroundColor Green

$tables = psql $DatabaseUrl -t -c "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;" 2>$null
$tables = $tables | Where-Object { $_.Trim() }

if ($tables.Count -eq 0) {
    Write-Warning "  ⚠ 没有找到任何表，迁移可能未成功"
} else {
    Write-Host "  ✓ 找到 $($tables.Count) 个表:" -ForegroundColor Green
    foreach ($table in $tables) {
        Write-Host "    - $($table.Trim())" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "  数据库初始化完成!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host ""
Write-Host "常用命令:" -ForegroundColor Cyan
Write-Host "  查看表结构: psql $DatabaseUrl -c '\dt'" -ForegroundColor White
Write-Host "  查看 agents: psql $DatabaseUrl -c 'SELECT * FROM agents;'" -ForegroundColor White
Write-Host "  查看设置: psql $DatabaseUrl -c 'SELECT * FROM system_settings;'" -ForegroundColor White
Write-Host ""
