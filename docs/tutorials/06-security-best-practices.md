# 安全最佳实践

> **保护您的 Agent 和资产安全**

本教程将介绍 BeeBotOS 的安全最佳实践，帮助您构建安全可靠的 Agent 系统。

---

## 目录

1. [安全概述](#安全概述)
2. [密钥管理](#密钥管理)
3. [Agent 安全配置](#agent-安全配置)
4. [网络安全](#网络安全)
5. [智能合约安全](#智能合约安全)
6. [监控和审计](#监控和审计)
7. [应急响应](#应急响应)

---

## 安全概述

### 威胁模型

BeeBotOS 面临的主要安全威胁：

| 威胁 | 风险等级 | 影响 |
|------|---------|------|
| 私钥泄露 | 严重 | 资金损失 |
| Agent 权限过大 | 高 | 未授权操作 |
| 恶意 Skill | 高 | 系统入侵 |
| 网络攻击 | 中 | 服务中断 |
| 智能合约漏洞 | 严重 | 资金损失 |

### 安全原则

1. **最小权限原则** - 只授予必要的权限
2. **纵深防御** - 多层安全防护
3. **零信任** - 不信任任何外部输入
4. **持续监控** - 实时检测异常

---

## 密钥管理

### 1. 使用硬件钱包

```bash
# 连接 Ledgereebotos-cli wallet connect --type ledger

# 连接 Trezor
beebotos-cli wallet connect --type trezor

# 验证连接
beebotos-cli wallet list
```

### 2. 多签钱包配置

```bash
# 创建 2/3 多签钱包
beebotos-cli wallet create-multisig \
  --owners 0xADDR1,0xADDR2,0xADDR3 \
  --threshold 2 \
  --name "Treasury"

# 所有交易需要至少 2 个签名
```

### 3. 密钥分片存储

```bash
# 使用 Shamir Secret Sharing
beebotos-cli key shard \
  --secret "0xPRIVATE_KEY" \
  --shares 5 \
  --threshold 3

# 分发给 5 个保管人，需要至少 3 个才能恢复
```

### 4. 定期轮换密钥

```bash
# 生成新密钥
beebotos-cli key generate --name "agent-key-v2"

# 迁移权限
beebotos-cli agent rotate-key \
  --agent agent_xxx \
  --new-key "agent-key-v2"

# 撤销旧密钥
beebotos-cli key revoke "agent-key-v1"
```

---

## Agent 安全配置

### 1. 最小权限配置

```yaml
# secure-agent.yaml
name: "SecureTrader"

# 只授予必要的 Capability
capabilities:
  - L3_NetworkOut          # 需要调用 API
  - L7_ChainRead           # 需要查询余额
  # - L8_ChainWriteLow     # 不要自动授权支付

# 资源限制
resources:
  memory_mb: 256            # 限制内存
  cpu_quota: 500            # 限制 CPU
  max_tasks: 5              # 限制并发任务

# 白名单配置
security:
  # 允许访问的域名
  allowed_domains:
    - "api.exchange.com"
    - "price.feed.com"
  
  # 禁止的操作
  forbidden_operations:
    - "transfer_eth"
    - "approve_token"
  
  # 需要确认的高风险操作阈值
  confirm_threshold:
    eth_transfer: "0.1"     # 超过 0.1 ETH 需要确认
    token_approval: "1000"  # 超过 1000 代币需要确认
```

### 2. 沙箱隔离

```bash
# 启用严格的 WASM 沙箱
beebotos-cli agent config agent_xxx \
  --set sandbox.strict=true

# 限制 WASM 内存
beebotos-cli agent config agent_xxx \
  --set wasm.memory_limit=128

# 限制 WASM 执行时间
beebotos-cli agent config agent_xxx \
  --set wasm.timeout=30000
```

### 3. 输入验证

```rust
// 验证所有外部输入
fn validate_input(input: &str) -> Result<(), Error> {
    // 检查长度
    if input.len() > MAX_INPUT_LENGTH {
        return Err(Error::InputTooLong);
    }
    
    // 检查特殊字符
    if contains_dangerous_chars(input) {
        return Err(Error::InvalidCharacters);
    }
    
    // 检查注入攻击
    if contains_sql_injection(input) || contains_command_injection(input) {
        return Err(Error::PotentialAttack);
    }
    
    Ok(())
}
```

---

## 网络安全

### 1. 使用 TLS

```yaml
server:
  tls:
    enabled: true
    cert: "data/certs/server.crt"
    key: "data/certs/server.key"
    min_version: "1.3"
    cipher_suites:
      - "TLS_AES_256_GCM_SHA384"
      - "TLS_CHACHA20_POLY1305_SHA256"
```

### 2. 防火墙配置

```bash
# 使用 UFW (Ubuntu)
sudo ufw default deny incoming
sudo ufw default allow outgoing

# 允许 BeeBotOS 端口
sudo ufw allow 8080/tcp
sudo ufw allow 4001/tcp  # P2P

# 限制 SSH 访问
sudo ufw allow from YOUR_IP to any port 22

sudo ufw enable
```

### 3. 速率限制

```yaml
security:
  rate_limit:
    enabled: true
    
    # 按 IP 限制
    per_ip:
      requests: 100
      window: 60  # 每秒请求数
    
    # 按 Agent 限制
    per_agent:
      requests: 1000
      window: 60
    
    # 异常检测
    anomaly_detection:
      enabled: true
      threshold: 0.95
      action: "block"  # block | captcha | log
```

### 4. DDoS 防护

```bash
# 使用 Cloudflare
# 1. 将 DNS 切换到 Cloudflare
# 2. 启用 DDoS 防护
# 3. 配置速率限制规则

# 或使用 AWS Shield (如果在 AWS 上)
```

---

## 智能合约安全

### 1. 合约审计检查清单

部署前确认：

- [ ] 通过专业安全审计 (Certik, Trail of Bits, OpenZeppelin)
- [ ] 使用最新版本的 Solidity (>= 0.8.24)
- [ ] 启用编译器优化但保留调试信息
- [ ] 所有外部调用都有重入保护
- [ ] 整数溢出保护 (使用 SafeMath 或 0.8+ 内置检查)
- [ ] 访问控制完善 (Ownable, AccessControl)
- [ ] 紧急暂停功能已测试
- [ ] 事件日志完整

### 2. 安全交互模式

```solidity
// 正确的 Checks-Effects-Interactions 模式
function withdraw(uint256 amount) external {
    // 1. Checks
    require(balances[msg.sender] >= amount, "Insufficient balance");
    
    // 2. Effects
    balances[msg.sender] -= amount;
    totalSupply -= amount;
    
    // 3. Interactions (最后)
    (bool success, ) = msg.sender.call{value: amount}("");
    require(success, "Transfer failed");
    
    emit Withdrawal(msg.sender, amount);
}
```

### 3. 重入保护

```solidity
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract SecureContract is ReentrancyGuard {
    function withdraw() external nonReentrant {
        // 安全的外部调用
    }
}
```

### 4. 代理合约安全

```solidity
// 使用 OpenZeppelin 代理
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

// 实现合约
contract AgentRegistryV1 is Initializable, UUPSUpgradeable {
    function initialize() public initializer {
        __UUPSUpgradeable_init();
        // ...
    }
    
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
```

---

## 监控和审计

### 1. 日志配置

```yaml
logging:
  level: "info"
  
  # 安全相关事件
  security_events:
    - "capability_violation"
    - "unauthorized_access"
    - "rate_limit_exceeded"
    - "suspicious_transaction"
  
  # 敏感操作日志
  audit_log:
    enabled: true
    retention_days: 90
    
    operations:
      - "agent_created"
      - "agent_deleted"
      - "capability_changed"
      - "payment_executed"
      - "dao_vote_cast"
```

### 2. 实时监控

```bash
# 使用 Prometheus + Grafana
# 导入 BeeBotOS 监控仪表盘

# 关键告警规则
groups:
  - name: security_alerts
    rules:
      # 异常交易检测
      - alert: SuspiciousTransaction
        expr: increase(beebotos_transactions_total[5m]) > 100
        
      # 权限提升尝试
      - alert: CapabilityEscalationAttempt
        expr: beebotos_security_violations{type="capability"} > 0
        
      # 资金异常流动
      - alert: LargeTransfer
        expr: beebotos_transfer_amount > 1000000000000000000  # > 1 ETH
```

### 3. 定期审计

```bash
# 生成审计报告
beebotos-cli audit report \
  --agent agent_xxx \
  --since 7d \
  --format json

# 检查 Agent 权限
beebotos-cli agent audit agent_xxx

# 输出
Agent: agent_xxx
Capabilities: [L0, L1, L3, L7]
Last capability change: 2026-03-10
Total transactions: 156
Suspicious activities: 0
Risk score: Low (23/100)
```

---

## 应急响应

### 1. 紧急暂停

```bash
# 暂停 Agent
beebotos-cli agent pause agent_xxx --reason "Security incident"

# 暂停合约 (多签)
beebotos-cli contract pause \
  --contract "AgentRegistry" \
  --multisig "Treasury"
```

### 2. 事件响应流程

```
1. 检测异常
   └─> 监控告警 / 用户报告

2. 初步评估
   └─> 影响范围 / 严重程度

3. 紧急响应
   ├─> 暂停相关服务
   ├─> 冻结可疑账户
   └─> 通知核心团队

4. 调查取证
   └─> 收集日志 / 分析交易

5. 修复漏洞
   └─> 开发补丁 / 测试验证

6. 恢复服务
   └─> 逐步恢复 / 持续监控

7. 事后复盘
   └─> 报告撰写 / 改进措施
```

### 3. 安全事件报告

```bash
# 生成事件报告
beebotos-cli incident create \
  --type "suspicious_activity" \
  --severity "high" \
  --agent agent_xxx \
  --description "检测到异常的交易模式"

# 自动收集证据
beebotos-cli incident collect \
  --id incident_123 \
  --logs \
  --transactions \
  --memory_dump
```

### 4. 备份和恢复

```bash
# 定期备份
beebotos-cli backup create \
  --type full \
  --destination s3://beebotos-backups/

# 测试恢复
beebotos-cli backup restore \
  --backup-id backup_20260313 \
  --test

# 实际恢复
beebotos-cli backup restore \
  --backup-id backup_20260313 \
  --confirm
```

---

## 安全检查清单

### 部署前检查

- [ ] 所有密钥存储在硬件钱包或安全密钥管理系统
- [ ] Agent 只拥有最小必要的 Capability
- [ ] WASM 沙箱启用且配置正确
- [ ] 网络防火墙配置完成
- [ ] TLS 证书有效且配置正确
- [ ] 审计日志启用
- [ ] 告警规则配置完成
- [ ] 应急响应计划已制定

### 日常维护

- [ ] 定期审查 Agent 权限
- [ ] 检查异常日志
- [ ] 更新依赖到最新安全版本
- [ ] 测试备份恢复流程
- [ ] 轮换长期使用的密钥
- [ ] 审查 Skill 代码安全性

---

## 总结

安全是一个持续的过程，而非一次性任务。遵循本指南的最佳实践，可以显著降低安全风险。

记住：
- **永远不要** 将私钥硬编码在代码中
- **永远不要** 授予不必要的权限
- **永远不要** 信任外部输入
- **永远要** 监控和审计系统活动

---

**预计时间**: 15 分钟  
**难度**: ⭐⭐⭐⭐ 高级  
**前置教程**: [DAO 治理参与](05-dao-governance.md)
