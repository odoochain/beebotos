#!/bin/bash
# 企业微信环境变量配置

export WECHAT_CORP_ID="ww812fdb0ce621af12"
export WECHAT_AGENT_ID="1000004"
export WECHAT_CORP_SECRET="9jwOXjryfBxlFb0wIUeSi3YkZa5RnUujZb1u12qGeEo"

echo "✓ 企业微信环境变量已设置"
echo "  CORP_ID: $WECHAT_CORP_ID"
echo "  AGENT_ID: $WECHAT_AGENT_ID"
echo "  SECRET: ${WECHAT_CORP_SECRET:0:10}..."
