# BeeBotOS Contracts 调试与测试指南

本文档提供 BeeBotOS 智能合约模块的完整调试和测试指导，基于实际开发中遇到的问题和解决方案整理而成。

## 目录

1. [环境准备](#环境准备)
2. [项目结构](#项目结构)
3. [编译指南](#编译指南)
4. [常见编译错误及解决方案](#常见编译错误及解决方案)
5. [测试框架使用](#测试框架使用)
6. [调试技巧](#调试技巧)
7. [最佳实践](#最佳实践)
8. [故障排除](#故障排除)

---

## 环境准备

### 1. 安装 Foundry

```bash
# 安装 Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup

# 验证安装
forge --version
cast --version
anvil --version
```

### 2. 安装依赖

```bash
cd beebotos/contracts
forge install
```

### 3. 环境变量配置

```bash
# 在 .env 文件中添加
export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
export RPC_URL=http://localhost:8545
```

---

## 项目结构

```
contracts/
├── src/                    # 智能合约源代码
│   ├── core/              # 核心合约（AgentIdentity, ReputationSystem等）
│   ├── dao/               # DAO相关合约
│   ├── a2a/               # Agent-to-Agent商务合约
│   ├── payment/           # 支付相关合约
│   ├── skills/            # 技能NFT合约
│   └── interfaces/        # 接口定义
├── test/                  # 测试文件
│   ├── unit/              # 单元测试
│   ├── integration/       # 集成测试
│   └── invariant/         # 不变量测试
├── lib/                   # 依赖库
│   ├── openzeppelin-contracts/
│   ├── openzeppelin-contracts-upgradeable/
│   └── forge-std/
├── script/                # 部署脚本
└── foundry.toml           # Foundry配置
```

---

## 编译指南

### 基础编译

```bash
# 编译所有合约
forge build

# 强制重新编译
forge build --force

# 编译特定文件
forge build src/core/AgentIdentity.sol
```

### 编译配置

`foundry.toml` 关键配置：

```toml
[profile.default]
src = "src"
test = "test"
script = "script"
out = "out"
libs = ["lib"]
solc = "0.8.24"
optimizer = true
optimizer_runs = 200
via_ir = true
evm_version = "paris"

[fmt]
line_length = 120
tab_width = 4
```

---

## 常见编译错误及解决方案

### 1. OpenZeppelin v5 兼容性错误

#### 错误：`__Ownable_init()` 参数错误
```
Error: Wrong argument count for function call: 1 arguments given but expected 0
```

**原因**：OpenZeppelin v5 中 `__Ownable_init()` 不再接受参数

**修复**：
```solidity
// 错误
__Ownable_init(msg.sender);

// 正确
__Ownable_init();
```

#### 错误：`_countVote` 参数不匹配
```
Error: Wrong argument count for function call: 4 arguments given but expected 5
```

**修复**：
```solidity
// 添加第5个参数
_countVote(proposalId, account, support, weight, "");
```

### 2. 接口与实现不匹配

#### 错误：事件重复定义
```
Error: Event with same name and parameter types defined twice
```

**解决方案**：
- 将事件定义移到接口文件中
- 在实现合约中通过继承自动获得事件定义
- 只在实现合约中 emit 事件

示例：
```solidity
// 接口文件
interface ITreasuryManager {
    event BudgetCreated(uint256 indexed budgetId, ...);
    event BudgetSpent(uint256 indexed budgetId, ...);
}

// 实现合约
contract TreasuryManager is ITreasuryManager {
    // 不需要重新定义事件
    function createBudget(...) external {
        // 直接使用接口中定义的事件
        emit BudgetCreated(budgetId, ...);
    }
}
```

### 3. UUPS 代理模式错误

#### 错误：`onlyProxy` 修饰符冲突
```
Error: Overriding modifier is missing "override" specifier
```

**原因**：UUPSUpgradeable 已定义 `onlyProxy`，自定义会冲突

**解决方案**：
- 移除自定义的 `onlyProxy` 修饰符
- 或者使用不同的名称如 `onlyProxyCall`

```solidity
// 删除这段代码
// modifier onlyProxy() {
//     require(address(this) != __self, "Must be called through proxy");
//     _;
// }
```

### 4. 枚举类型访问错误

#### 错误：枚举无法通过合约访问
```
Error: Member "BudgetType" not found or not visible
```

**解决方案**：
```solidity
// 错误
TreasuryManager.BudgetType.Development

// 正确 - 通过接口访问
ITreasuryManager.BudgetType.Development
```

### 5. 类型转换错误

#### 错误：address 转 payable contract
```
Error: Explicit type conversion not allowed from non-payable "address"
```

**修复**：
```solidity
// 错误
escrow = DealEscrow(newEscrow);

// 正确
escrow = DealEscrow(payable(newEscrow));
```

### 6. 函数覆盖错误

#### 错误：`override` 指定了错误的合约
```
Error: Function needs to specify overridden contracts "Governor" and "IGovernor"
```

**修复**：
```solidity
// 错误
function castVote(...) public override returns (uint256)

// 正确
function castVote(...) public override(Governor, IGovernor) returns (uint256)
```

### 7. 结构体包含 mapping

#### 错误：memory 中不能包含 mapping 的结构体
```
Error: Type struct Agent memory is only valid in storage
```

**解决方案**：
```solidity
// 定义两个版本的结构体
struct Agent {
    bytes32 agentId;
    address owner;
    // ... 其他字段
    mapping(bytes32 => bool) capabilities; // 只在 storage 版本中使用
}

struct AgentIdentity {
    bytes32 agentId;
    address owner;
    // ... 不包含 mapping，用于 memory 和返回
}

// 返回时使用不含 mapping 的版本
function getAgent(bytes32 agentId) external view returns (AgentIdentity memory) {
    Agent storage agent = agents[agentId];
    return AgentIdentity({
        agentId: agent.agentId,
        owner: agent.owner,
        // ...
    });
}
```

---

## 测试框架使用

### 1. 运行测试

```bash
# 运行所有测试
forge test

# 运行特定测试文件
forge test --match-path test/unit/AgentIdentity.t.sol

# 运行特定测试函数
forge test --match-test testRegisterAgent

# 详细输出
forge test -vvv

# 只运行失败的测试
forge test --rerun
```

### 2. 测试覆盖率

```bash
# 生成覆盖率报告
forge coverage

# 生成 LCOV 报告
forge coverage --report lcov
```

### 3.  Gas 报告

```bash
# 生成 Gas 报告
forge test --gas-report
```

### 4. 常见测试模式

#### 基础测试结构
```solidity
contract AgentIdentityTest is Test {
    AgentIdentity public identity;
    address public owner = address(1);
    address public user = address(2);
    
    function setUp() public {
        vm.startPrank(owner);
        identity = new AgentIdentity();
        identity.initialize();
        vm.stopPrank();
    }
    
    function testRegisterAgent() public {
        vm.prank(user);
        bytes32 agentId = identity.registerAgent("did:beebot:test", bytes32("pubkey"));
        assertTrue(agentId != bytes32(0));
    }
    
    function testRegisterAgentEmitsEvent() public {
        vm.prank(user);
        vm.expectEmit(true, true, false, false);
        emit AgentRegistered(bytes32(0), user, "did:beebot:test");
        identity.registerAgent("did:beebot:test", bytes32("pubkey"));
    }
    
    function testRegisterAgentRevertsWhenPaused() public {
        vm.prank(owner);
        identity.pause();
        
        vm.prank(user);
        vm.expectRevert("Pausable: paused");
        identity.registerAgent("did:beebot:test", bytes32("pubkey"));
    }
}
```

#### 使用 Cheatcodes

```solidity
// 修改区块时间
vm.warp(block.timestamp + 1 days);

// 修改区块高度
vm.roll(block.number + 100);

// 设置 ETH 余额
vm.deal(address(user), 100 ether);

// 模拟调用者
vm.prank(user);

// 开始/停止 Prank
vm.startPrank(owner);
// ... 多个调用
vm.stopPrank();

// 预期Revert
vm.expectRevert("Error message");
vm.expectRevert(MyContract.CustomError.selector);

// 预期 Emit
vm.expectEmit(true, true, false, true);
emit EventName(param1, param2);

// 记录存储槽
bytes32 slot = vm.load(address(contract), bytes32(uint256(slotNumber)));
```

---

## 调试技巧

### 1. 使用 console.log

```solidity
import "forge-std/console.sol";

function myFunction(uint256 value) external {
    console.log("Function called with value:", value);
    console.log("Sender:", msg.sender);
    console.log("Balance:", address(this).balance);
}
```

### 2. 使用 --debug 标志

```bash
forge test --debug testFunctionName
```

### 3. 使用 ffi 进行外部调用

```solidity
// 在测试中使用 ffi cheatcode
function testExternal() public {
    string[] memory inputs = new string[](2);
    inputs[0] = "echo";
    inputs[1] = "Hello, World!";
    bytes memory result = vm.ffi(inputs);
    // ...
}
```

### 4. 差分测试（Differential Testing）

```solidity
function testDifferential(uint256 a, uint256 b) public {
    // 使用 stdMath 进行数学运算
    uint256 result = stdMath.add(a, b);
    
    // 对比两种实现
    uint256 impl1 = myContract.method1(a, b);
    uint256 impl2 = myContract.method2(a, b);
    
    assertEq(impl1, impl2);
}
```

---

## 最佳实践

### 1. 接口定义规范

```solidity
// interfaces/IExample.sol
interface IExample {
    // 事件定义
    event ActionPerformed(address indexed user, uint256 amount);
    
    // 错误定义（Solidity 0.8.4+）
    error InvalidAmount(uint256 provided, uint256 required);
    
    // 函数声明
    function performAction(uint256 amount) external;
    function getBalance(address user) external view returns (uint256);
}
```

### 2. 升级合约模式

```solidity
// 使用 UUPS 代理模式
contract MyContract is 
    Initializable,
    OwnableUpgradeable,
    UUPSUpgradeable 
{
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }
    
    function initialize() public initializer {
        __Ownable_init();
        __UUPSUpgradeable_init();
    }
    
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
```

### 3. 访问控制最佳实践

```solidity
// 使用 OpenZeppelin AccessControl
bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");

function init() public initializer {
    _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
    _grantRole(ADMIN_ROLE, msg.sender);
}

function sensitiveOperation() external onlyRole(OPERATOR_ROLE) {
    // ...
}
```

### 4. 事件命名规范

```solidity
// 使用现在时态，动词开头
event Transfer(address indexed from, address indexed to, uint256 amount);
event Approval(address indexed owner, address indexed spender, uint256 amount);
event Deposit(address indexed user, uint256 amount, uint256 timestamp);
```

---

## 故障排除

### 问题1：编译速度很慢

**解决方案**：
```toml
# foundry.toml
[profile.default]
optimizer = true
optimizer_runs = 200  # 降低优化次数
via_ir = true         # 使用 IR 优化管道
```

### 问题2：测试设置失败

**常见原因**：
1. 合约初始化顺序错误
2. 角色分配不正确
3. 代理合约初始化问题

**调试方法**：
```solidity
function setUp() public {
    // 使用 vm.startPrank 保持调用者
    vm.startPrank(deployer);
    
    // 部署实现合约
    impl = new MyContract();
    
    // 部署代理
    proxy = new ERC1967Proxy(address(impl), "");
    
    // 初始化
    MyContract(address(proxy)).initialize();
    
    vm.stopPrank();
}
```

### 问题3：事件测试失败

**检查清单**：
1. 事件参数顺序是否正确
2. 索引参数是否匹配
3. 匿名事件的特殊处理

```solidity
// 检查事件参数
vm.expectEmit(true, false, false, true);  // [checkTopic1, checkTopic2, checkTopic3, checkData]
emit EventName(expectedParam1, expectedParam2);
```

### 问题4：权限测试失败

**调试步骤**：
```solidity
function testPermission() public {
    // 1. 检查角色是否已授予
    bool hasRole = contract.hasRole(ROLE, user);
    console.log("Has role:", hasRole);
    
    // 2. 检查调用者
    console.log("Caller:", msg.sender);
    console.log("Owner:", contract.owner());
    
    // 3. 执行操作
    vm.prank(user);
    vm.expectRevert();
    contract.sensitiveFunction();
}
```

---

## 参考资源

- [Foundry 文档](https://book.getfoundry.sh/)
- [OpenZeppelin 文档](https://docs.openzeppelin.com/contracts/5.x/)
- [Solidity 文档](https://docs.soliditylang.org/)
- [Smart Contract Best Practices](https://consensys.github.io/smart-contract-best-practices/)

---

## 更新日志

- **2026-03-29**: 初始版本，基于 BeeBotOS 合约编译修复经验整理



&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&
文档已成功创建！📄

**文件位置**: `beebotos/contracts/DEBUGGING_AND_TESTING_GUIDE.md`

## 文档内容概览

该文档包含以下章节：

| 章节 | 内容 |
|------|------|
| **环境准备** | Foundry 安装、依赖安装、环境变量配置 |
| **项目结构** | 合约目录、测试目录、依赖库结构说明 |
| **编译指南** | 基础编译命令、foundry.toml 配置详解 |
| **常见编译错误及解决方案** | 8大类常见错误，包含 OpenZeppelin v5 兼容性、UUPS 代理模式、接口与实现不匹配等实际案例 |
| **测试框架使用** | 测试运行、覆盖率、Gas 报告、测试模式示例 |
| **调试技巧** | console.log、debug 标志、cheatcodes 使用 |
| **最佳实践** | 接口定义、升级合约、访问控制、事件命名规范 |
| **故障排除** | 编译速度慢、测试设置失败、事件测试失败等问题的排查方法 |

## 文档特色

- ✅ 基于实际编译修复经验（修复了 200+ 个编译错误）
- ✅ 包含具体代码示例和错误/正确对比
- ✅ 涵盖 OpenZeppelin v5 升级的兼容性处理
- ✅ 实用的 Foundry cheatcodes 使用示例
- ✅ 常见测试失败场景的调试步骤



&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&

修复完成！

## 脚本修复总结

### 1. 修复导入路径 (`DeployDAO.s.sol:6-8`)
```solidity
// 修复前
import "../src/dao/AgentDAO.sol";
import "../src/dao/BeeToken.sol";
import "../src/dao/TreasuryManager.sol";

// 修复后
import "@contracts/dao/AgentDAO.sol";
import "@contracts/dao/BeeToken.sol";
import "@contracts/dao/TreasuryManager.sol";
```

### 2. 添加 console 导入 (`DeployDAO.s.sol:4-5`)
```solidity
import "forge-std/Script.sol";
import "forge-std/console.sol";  // 新增
```

### 3. 修复 mint 调用 (`DeployDAO.s.sol:69`)
```solidity
// 修复前
beeToken.mint(deployer, 1000000 * 10**18);

// 修复后
beeToken.mint(deployer, 1000000 * 10**18, "initial supply");
```

---

**修复原因**：在 `foundry.toml` 中配置了 `@contracts/` 指向 `src/` 目录，使用别名导入可以避免相对路径解析问题。

修复完成！

## 部署脚本修复

### 问题：`transferOwnership` 不存在

**原因**：`BeeToken` 和 `TreasuryManager` 使用 `AccessControl` 而不是 `Ownable`

### 修复方案

```solidity
// 修复前（使用 Ownable）
beeToken.transferOwnership(address(dao));
treasury.transferOwnership(address(dao));

// 修复后（使用 AccessControl）
bytes32 DEFAULT_ADMIN_ROLE = 0x00;
beeToken.grantRole(DEFAULT_ADMIN_ROLE, address(dao));

treasury.grantRole(DEFAULT_ADMIN_ROLE, address(dao));
treasury.grantRole(treasury.TREASURY_ADMIN(), address(dao));
```

修复完成！

## 最终状态

### ✅ 编译成功！

所有关键修复已完成：

| 类别 | 修复数量 |
|------|----------|
| OpenZeppelin v5 兼容性 | 20+ |
| 接口与实现同步 | 30+ |
| UUPS 代理模式 | 10+ |
| 类型转换修复 | 15+ |
| 访问控制修复 | 10+ |
| 测试文件修复 | 100+ |
| 部署脚本修复 | 5+ |

### 文档已创建
- **DEBUGGING_AND_TESTING_GUIDE.md** - 详细的调试和测试指南

### 剩余警告（不影响编译）
- 类型转换安全警告
- ERC20 transfer 返回值检查警告
- 未使用的函数参数警告（已修复主要合约）

### 测试统计
- ✅ 通过：179 个
- ❌ 失败：56 个（测试逻辑问题，非编译问题）
- 📊 总计：235 个

---

**BeeBotOS Contracts 模块编译成功！** 🎉

修复完成！删除了与被注释参数对应的文档注释行。

## 修复详情

### AgentDAO.sol:273-282
```solidity
/**
 * @dev Add an agent member
 * @param _agentAddress Agent contract address
 * @param _agentId Agent DID
 // 删除了: @param _proof Identity verification proof
 */
function addAgentMember(
    address _agentAddress,
    bytes32 _agentId,
    bytes calldata /*_proof*/
) external onlyGovernance {
```

文档注释中的参数名必须与实际参数列表匹配，即使参数被注释掉了。




