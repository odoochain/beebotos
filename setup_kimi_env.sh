#!/bin/bash
# Kimi API 环境变量配置

export KIMI_API_KEY="sk-LDJf4mZVHeM7fAHyLEPppx3fvkEM2vEezv4HnjbBRZWywKry"
export KIMI_DEFAULT_MODEL="kimi-k2.5"
export KIMI_BASE_URL="https://api.moonshot.cn/v1"

# 可选: 设置默认LLM provider为kimi
export LLM_PROVIDER="kimi"
export LLM_MODEL="kimi-k2.5"

echo "✅ Kimi 环境变量已设置"
echo "  API Key: ${KIMI_API_KEY:0:15}..."
echo "  Model: $KIMI_DEFAULT_MODEL"
echo "  Base URL: $KIMI_BASE_URL"
