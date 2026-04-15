# RFC-001: BeeBotOS Capability 权限系统

> **状态**: 已接受 (Accepted)  
> **作者**: BeeBotOS Core Team  
> **创建日期**: 2026-01-15  
> **最后更新**: 2026-03-13  
> **版本**: v1.0

---

## 目录

1. [摘要](#摘要)
2. [动机](#动机)
3. [设计目标](#设计目标)
4. [详细设计](#详细设计)
5. [10层 Capability 模型](#10层-capability-模型)
6. [实现细节](#实现细节)
7. [安全考量](#安全考量)
8. [向后兼容性](#向后兼容性)
9. [替代方案](#替代方案)
10. [参考实现](#参考实现)
11. [未解决问题](#未解决问题)

---

## 摘要

本 RFC 定义了 BeeBotOS 的 Capability-based 权限系统。该系统采用 10 层分级模型，为 AI Agent 提供细粒度的资源访问控制。相比传统的基于身份的权限模型，Capability 模型更符合最小权限原则，更适合自主运行的 AI Agent 场景。

**核心要点**:
- 10 层 Capability 权限级别 (L0-L9)
- 可委托、可撤销的权限令牌
- 与 WASM 沙箱深度集成
- 支持权限的动态升级和降级

---

## 动机

### 当前问题

现有的 AI Agent 系统通常使用基于身份的权限模型 (RBAC/ABAC)，存在以下问题：

1. **粗粒度控制** - 无法精确控制单个操作
2. **权限扩散** - 长期运行容易积累过多权限
3. **难以审计** - 无法追踪权限使用路径
4. **不适合自主代理** - Agent 需要人类持续授权

### 为什么 Capability 模型更适合

| 特性 | 基于身份 (RBAC) | 基于能力 (Capability) |
|------|----------------|---------------------|
| 权限粒度 | 角色级别 | 操作级别 |
| 委托 | 困难 | 原生支持 |
| 撤销 | 复杂 | 简单 |
| 审计 | 间接 | 直接 |
| 自主代理 | 不适合 | 理想 |

---

## 设计目标

### 必须实现 (MUST)

- **M1**: 支持至少 10 个权限级别
- **M2**: 支持权限委托给子 Agent
- **M3**: 支持权限动态撤销
- **M4**: 与 WASM 沙箱集成
- **M5**: 完整的审计日志

### 应该实现 (SHOULD)

- **S1**: 权限使用时自动降级
- **S2**: 基于风险的动态权限调整
- **S3**: 权限使用统计分析

### 可以实现 (MAY)

- **C1**: 零知识证明验证权限
- **C2**: 跨链权限验证

---

## 详细设计

### 核心概念

#### Capability

Capability 是一个不可伪造的令牌，授予持有者执行特定操作的权限。

```rust
/// Capability 令牌
pub struct CapabilityToken {
    /// 权限级别 (0-9)
    pub level: CapabilityLevel,
    
    /// 权限标识符
    pub permission: Permission,
    
    /// 签发者
    pub issuer: AgentId,
    
    /// 持有者
    pub holder: AgentId,
    
    /// 签发时间
    pub issued_at: Timestamp,
    
    /// 过期时间 (None 表示永不过期)
    pub expires_at: Option<Timestamp>,
    
    /// 是否可委托
    pub delegable: bool,
    
    /// 数字签名
    pub signature: Signature,
}
```

#### Capability Set

Agent 拥有的所有 Capability 的集合。

```rust
/// Capability 集合
pub struct CapabilitySet {
    /// 位图表示，每个 bit 代表一个级别
    bits: u64,
    
    /// 具体的权限令牌
    tokens: Vec<CapabilityToken>,
}

impl CapabilitySet {
    /// 检查是否拥有某级别权限
    pub fn has(&self, level: CapabilityLevel) -> bool {
        (self.bits >> (level as u8)) & 1 == 1
    }
    
    /// 添加权限
    pub fn add(&mut self, token: CapabilityToken) {
        self.bits |= 1 << (token.level as u8);
        self.tokens.push(token);
    }
    
    /// 移除权限
    pub fn remove(&mut self, level: CapabilityLevel) {
        self.bits &= !(1 << (level as u8));
        self.tokens.retain(|t| t.level != level);
    }
}
```

### 权限检查流程

```
操作请求
    │
    ▼
┌─────────────────┐
│ 1. 提取所需权限  │
│    (从系统调用)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 2. 检查 Capability│
│    - 级别是否足够 │
│    - 是否过期     │
│    - 签名是否有效 │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
 通过       拒绝
    │         │
    ▼         ▼
执行操作   返回错误
    │         │
    ▼         ▼
记录审计日志
```

---

## 10层 Capability 模型

### 层级定义

| 级别 | 名称 | 权限范围 | 风险等级 | 典型操作 |
|------|------|---------|---------|---------|
| L0 | LocalCompute | 仅本地计算 | 极低 | 数据处理、逻辑运算 |
| L1 | FileRead | 文件系统读 | 低 | 读取配置、日志 |
| L2 | FileWrite | 文件系统写 | 低 | 写入日志、缓存 |
| L3 | NetworkOut | 网络出站 | 中 | HTTP 请求、API 调用 |
| L4 | NetworkIn | 网络入站 | 中 | 监听端口、接收请求 |
| L5 | SpawnLimited | 有限创建子 Agent | 中 | 创建 ≤5 个子 Agent |
| L6 | SpawnUnlimited | 无限创建子 Agent | 高 | 无限制创建子 Agent |
| L7 | ChainRead | 区块链读 | 中 | 查询余额、事件 |
| L8 | ChainWriteLow | 区块链写 (低价值) | 高 | 转账 < 0.1 ETH |
| L9 | ChainWriteHigh | 区块链写 (高价值) | 极高 | 转账 ≥ 0.1 ETH |

### 权限继承关系

```
L9 (ChainWriteHigh)
└── 包含 L8
    └── 包含 L7
        └── 包含 L6
            └── 包含 L5
                └── 包含 L4
                    └── 包含 L3
                        └── 包含 L2
                            └── 包含 L1
                                └── 包含 L0
```

**规则**: 拥有高级别权限自动包含所有低级别权限。

### 权限升级路径

```
L0 → L1 → L2 → L3 → L5 → L7 → L8
                    ↓
                   L4 (特殊分支)
                    ↓
                   L6 (需要额外验证)
                    ↓
                   L9 (需要多重签名)
```

---

## 实现细节

### 1. 权限委托

```rust
impl Agent {
    /// 委托权限给子 Agent
    pub fn delegate_capability(
        &self,
        to: &AgentId,
        level: CapabilityLevel,
        constraints: DelegationConstraints,
    ) -> Result<CapabilityToken, DelegationError> {
        // 1. 检查自己是否有该权限
        if !self.capabilities.has(level) {
            return Err(DelegationError::InsufficientCapability);
        }
        
        // 2. 检查权限是否可委托
        let token = self.capabilities.get(level).ok_or(
            DelegationError::TokenNotFound
        )?;
        
        if !token.delegable {
            return Err(DelegationError::NotDelegable);
        }
        
        // 3. 创建子权限
        let delegated_token = CapabilityToken {
            level,
            permission: token.permission.clone(),
            issuer: self.id.clone(),
            holder: to.clone(),
            issued_at: now(),
            expires_at: constraints.expires_at,
            delegable: constraints.allow_redelagation,
            signature: self.sign(&payload),
        };
        
        // 4. 记录委托关系
        self.record_delegation(&delegated_token)?;
        
        Ok(delegated_token)
    }
}
```

### 2. 权限撤销

```rust
impl Agent {
    /// 撤销已委托的权限
    pub fn revoke_capability(
        &self,
        token_id: &TokenId,
    ) -> Result<(), RevocationError> {
        // 1. 查找令牌
        let token = self.find_token(token_id)?;
        
        // 2. 验证撤销权限
        if token.issuer != self.id {
            return Err(RevocationError::NotAuthorized);
        }
        
        // 3. 撤销令牌
        self.capabilities.remove(token.level);
        
        // 4. 级联撤销 (如果该令牌被进一步委托)
        self.cascade_revoke(token_id)?;
        
        // 5. 通知持有者
        self.notify_revocation(&token.holder, token_id)?;
        
        Ok(())
    }
}
```

### 3. 权限验证

```rust
pub struct CapabilityVerifier {
    policy: SecurityPolicy,
    audit_log: AuditLog,
}

impl CapabilityVerifier {
    pub async fn verify(
        &self,
        caller: &AgentId,
        required: CapabilityLevel,
        context: &OperationContext,
    ) -> Result<VerificationResult, VerificationError> {
        // 1. 获取 caller 的 Capability
        let caps = self.get_capabilities(caller).await?;
        
        // 2. 检查级别
        if !caps.has(required) {
            self.audit_log.record_denied(caller, required, context).await;
            return Err(VerificationError::InsufficientCapability);
        }
        
        // 3. 检查令牌有效性
        let token = caps.get(required).unwrap();
        if self.is_expired(token) {
            return Err(VerificationError::TokenExpired);
        }
        
        // 4. 验证签名
        if !self.verify_signature(token) {
            return Err(VerificationError::InvalidSignature);
        }
        
        // 5. 检查策略限制
        if let Some(limit) = self.policy.get_limit(required) {
            if context.amount > limit {
                return Err(VerificationError::ExceedsLimit);
            }
        }
        
        // 6. 记录审计
        self.audit_log.record_allowed(caller, required, context).await;
        
        Ok(VerificationResult::Allowed)
    }
}
```

### 4. 与系统调用集成

```rust
/// 系统调用分发器
pub struct SyscallDispatcher {
    handlers: HashMap<SyscallNumber, Box<dyn SyscallHandler>>,
    verifier: CapabilityVerifier,
}

impl SyscallDispatcher {
    pub async fn dispatch(
        &self,
        num: SyscallNumber,
        args: SyscallArgs,
        caller: &AgentId,
    ) -> SyscallResult {
        // 1. 获取所需权限
        let required = self.get_required_capability(num);
        
        // 2. 验证权限
        let context = OperationContext::from_args(&args);
        match self.verifier.verify(caller, required, &context).await {
            Ok(_) => {
                // 3. 执行系统调用
                let handler = self.handlers.get(&num).unwrap();
                handler.handle(args).await
            }
            Err(e) => {
                SyscallResult::Error(SyscallError::PermissionDenied(e.to_string()))
            }
        }
    }
    
    fn get_required_capability(&self, syscall: SyscallNumber) -> CapabilityLevel {
        match syscall {
            // L0: 本地计算
            Syscall::QueryMemory | Syscall::LocalCompute => L0,
            
            // L1: 文件读
            Syscall::ReadFile | Syscall::ListFiles => L1,
            
            // L2: 文件写
            Syscall::WriteFile | Syscall::DeleteFile => L2,
            
            // L3: 网络
            Syscall::SendMessage | Syscall::HttpRequest => L3,
            
            // L5: 创建 Agent
            Syscall::SpawnAgent => L5,
            
            // L7: 链上读
            Syscall::QueryBalance | Syscall::QueryContract => L7,
            
            // L8: 链上写 (低价值)
            Syscall::ExecutePaymentLow => L8,
            
            // L9: 链上写 (高价值)
            Syscall::ExecutePaymentHigh | Syscall::ContractUpgrade => L9,
            
            _ => L9, // 默认最高权限
        }
    }
}
```

---

## 安全考量

### 1. 令牌安全

- **不可伪造**: 使用 Ed25519 数字签名
- **防重放**: 包含时间戳和 nonce
- **过期机制**: 支持临时权限

### 2. 委托限制

- **委托深度**: 默认最多 3 层委托
- **委托范围**: 可限制子权限的范围
- **时间限制**: 委托可设置过期时间

### 3. 撤销安全

- **即时生效**: 撤销后立即失效
- **级联撤销**: 自动撤销所有派生权限
- **通知机制**: 通知所有相关方

### 4. 审计要求

所有权限操作必须记录：
- 时间戳
- 操作类型 (授予/使用/委托/撤销)
- 涉及 Agent
- 权限级别
- 操作结果

---

## 向后兼容性

### 版本兼容性

| RFC 版本 | 内核版本 | 兼容性 |
|----------|---------|--------|
| v1.0 | 1.x | 已弃用 |
| v1.0 | 1.x | 当前版本 |

### 迁移指南

从 v0.9 升级到 v1.0:

1. 更新 Capability 存储格式
2. 重新签发所有令牌
3. 更新权限检查代码

---

## 替代方案

### 方案 A: 基于角色的访问控制 (RBAC)

**缺点**:
- 权限粒度粗
- 不支持委托
- 难以审计

**结论**: 不适合自主 Agent 场景

### 方案 B: 基于属性的访问控制 (ABAC)

**优点**:
- 灵活的规则定义

**缺点**:
- 复杂度高
- 性能开销大

**结论**: 可作为未来扩展

### 方案 C: 当前方案 - Capability-based

**优点**:
- 天然支持委托
- 细粒度控制
- 可审计
- 适合自主 Agent

**结论**: 采用此方案

---

## 参考实现

### Rust 实现

```rust
// crates/kernel/src/capabilities/mod.rs
pub mod token;
pub mod set;
pub mod verifier;

pub use token::CapabilityToken;
pub use set::CapabilitySet;
pub use verifier::CapabilityVerifier;
```

### 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_check() {
        let mut caps = CapabilitySet::new();
        caps.add(create_token(L3));
        
        assert!(caps.has(L0)); // L3 包含 L0
        assert!(caps.has(L3));
        assert!(!caps.has(L5));
    }
    
    #[test]
    fn test_delegation() {
        let parent = create_agent_with_cap(L5);
        let child = create_agent();
        
        let token = parent.delegate(&child.id, L3, constraints).unwrap();
        
        assert_eq!(token.level, L3);
        assert_eq!(token.issuer, parent.id);
        assert_eq!(token.holder, child.id);
    }
    
    #[test]
    fn test_revocation() {
        let parent = create_agent_with_cap(L5);
        let child = create_agent();
        
        let token = parent.delegate(&child.id, L3, constraints).unwrap();
        parent.revoke(&token.id).unwrap();
        
        assert!(!child.capabilities.has(L3));
    }
}
```

---

## 未解决问题

### 开放问题

1. **跨链权限**: 如何在不同区块链间验证权限？
2. **权限市场**: 是否允许权限的交易？
3. **AI 自主权限申请**: Agent 如何自主申请需要的权限？

### 研究项目

- 零知识证明在权限验证中的应用
- 基于机器学习的异常权限使用检测

---

## 更新历史

| 日期 | 版本 | 更新内容 |
|------|------|---------|
| 2026-01-15 | v1.0 | 初始版本 |
| 2026-03-13 | v1.0 | 完善委托和撤销机制，增加审计要求 |

---

## 参考文档

- [Capability-based security - Wikipedia](https://en.wikipedia.org/wiki/Capability-based_security)
- [Hydra: Federated Access Control](https://www.ory.sh/hydra/)
- [Macaroons: Cookies with Contextual Caveats](https://research.google/pubs/pub41892/)
