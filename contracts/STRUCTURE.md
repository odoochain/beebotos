# BeeBotOS Contracts Structure

## Overview
Optimized file structure for BeeBotOS smart contracts following Foundry best practices.

## Directory Layout

```
contracts/
├── src/                          # Source contracts
│   ├── core/                     # Core identity and registry
│   │   ├── AgentIdentity.sol
│   │   ├── AgentRegistry.sol
│   │   └── ReputationSystem.sol
│   │
│   ├── dao/                      # DAO governance (flattened)
│   │   ├── AgentDAO.sol
│   │   ├── AgentDAOFactory.sol
│   │   ├── AgentGovernor.sol
│   │   ├── DAOTimelock.sol
│   │   ├── DelegationManager.sol
│   │   ├── ProposalEngine.sol
│   │   ├── QuorumManager.sol
│   │   ├── VotingStrategy.sol
│   │   ├── VotingSystem.sol
│   │   ├── BeeToken.sol
│   │   ├── ReputationPoints.sol
│   │   ├── VeBeeToken.sol
│   │   ├── VestingWallet.sol
│   │   ├── BudgetController.sol
│   │   └── TreasuryManager.sol
│   │
│   ├── a2a/                      # A2A Commerce
│   │   ├── A2ACommerce.sol
│   │   ├── DealEscrow.sol
│   │   ├── DisputeResolution.sol
│   │   └── IntentMatcher.sol
│   │
│   ├── payment/                  # Payment system
│   │   ├── AgentPayment.sol
│   │   ├── CrossChainBridge.sol
│   │   └── LiquidityPool.sol
│   │
│   ├── skills/                   # Skills NFT
│   │   ├── SkillLicensing.sol
│   │   ├── SkillNFT.sol
│   │   └── SkillRegistry.sol
│   │
│   ├── libraries/                # Utility libraries
│   │   ├── ECDSAUtils.sol
│   │   ├── MerkleProof.sol
│   │   ├── SafeMath.sol
│   │   └── ZKVerifier.sol
│   │
│   └── interfaces/               # Interface definitions
│       ├── IA2ACommerce.sol
│       ├── IAgentDAO.sol
│       ├── IAgentPayment.sol
│       ├── IDelegationManager.sol
│       ├── IERC8004.sol
│       ├── IProposalEngine.sol
│       ├── IReputationSystem.sol
│       ├── ISkillNFT.sol
│       ├── ITreasuryManager.sol
│       └── IVotingSystem.sol
│
├── test/                         # Test files
│   ├── unit/                     # Unit tests
│   │   ├── AgentRegistry.t.sol
│   │   ├── AgentIdentity.t.sol
│   │   ├── ReputationSystem.t.sol
│   │   ├── A2ACommerce.t.sol
│   │   ├── DealEscrow.t.sol
│   │   ├── SkillNFT.t.sol
│   │   ├── AgentPayment.t.sol
│   │   ├── TreasuryManager.t.sol
│   │   ├── TreasuryManager.supplement.t.sol
│   │   ├── AgentDAO.t.sol
│   │   ├── BeeToken.t.sol
│   │   ├── VestingWallet.t.sol
│   │   └── CoverageTest.t.sol
│   │
│   ├── integration/              # Integration tests
│   │   ├── dao-workflow.t.sol
│   │   └── (Integration.t.sol - to be added)
│   │
│   └── invariant/                # Fuzz/invariant tests
│       └── Invariant.t.sol
│
├── script/                       # Deployment scripts
│   └── deploy/
│       └── DeployDAO.s.sol
│
└── mocks/                        # Mock contracts for testing
    ├── MockERC20.sol
    └── MockOracle.sol
```

## Import Path Conventions

### Within `src/` directory:
```solidity
// From src/dao/ to src/interfaces/
import "../interfaces/ITreasuryManager.sol";

// From src/core/ to src/core/
import "./AgentIdentity.sol";
```

### Within `test/` directory:
```solidity
// From test/unit/ to src/core/
import "../../src/core/AgentRegistry.sol";

// From test/integration/ to src/dao/
import "../../src/dao/AgentDAO.sol";
```

### From `script/` directory:
```solidity
// From script/deploy/ to src/dao/
import "../../src/dao/AgentDAO.sol";
```

## Configuration

### foundry.toml
```toml
[profile.default]
src = "contracts/src"
test = "contracts/test"
out = "out"
libs = ["lib"]
```

## File Count Summary

| Directory | Files | Description |
|-----------|-------|-------------|
| `src/core` | 3 | Core identity contracts |
| `src/dao` | 15 | DAO governance contracts |
| `src/a2a` | 4 | A2A commerce contracts |
| `src/payment` | 3 | Payment system contracts |
| `src/skills` | 3 | Skills NFT contracts |
| `src/libraries` | 4 | Utility libraries |
| `src/interfaces` | 10 | Interface definitions |
| `test/unit` | 13 | Unit test files |
| `test/integration` | 1 | Integration tests |
| `test/invariant` | 1 | Fuzz tests |
| `script/deploy` | 1 | Deployment scripts |
| `mocks` | 2 | Mock contracts |
| **Total** | **60** | **All Solidity files** |

## Migration Notes

### From old structure:
```
contracts/solidity/
  ├── core/
  ├── dao/
  │   ├── core/
  │   ├── delegation/
  │   ├── governance/
  │   ├── token/
  │   └── treasury/
  ├── a2a/
  ├── payment/
  ├── skills/
  ├── interfaces/
  ├── libraries/
  ├── mocks/
  ├── script/
  └── test/
    └── integration/
```

### To new structure:
1. Flattened `dao/` subdirectory structure
2. Separated `test/` into `unit/`, `integration/`, `invariant/`
3. Moved `mocks/` and `script/` to top level
4. Updated all import paths accordingly

## Benefits

1. **Clearer organization**: Separated concerns with dedicated directories
2. **Foundry compatible**: Follows standard Foundry project layout
3. **Easier navigation**: Flatter structure in `src/dao/`
4. **Better testing**: Organized tests by type (unit, integration, invariant)
5. **Maintainable**: Consistent import patterns throughout
