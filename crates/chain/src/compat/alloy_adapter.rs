//! Alloy provider adapter for ChainClientTrait
//!
//! This module provides an implementation of ChainClientTrait using Alloy
//! provider.

use std::sync::Arc;

// Import alloy for RPC types
use alloy::rpc::types::eth::TransactionReceipt as AlloyTransactionReceipt;
// Import alloy provider trait
use alloy_provider::Provider;
// Import alloy sol types for contract call encoding
use alloy_sol_types::{sol, SolCall};
use async_trait::async_trait;
use tracing::{debug, info};

use crate::compat::client_trait::*;
use crate::compat::{Address, BlockNumber, Bytes, TxHash, U256};

// Helper trait to convert between beebotos_core types and alloy types
trait IntoAlloy<T> {
    fn into_alloy(self) -> T;
}

impl IntoAlloy<alloy::primitives::Address> for Address {
    fn into_alloy(self) -> alloy::primitives::Address {
        alloy::primitives::Address::from_slice(self.as_slice())
    }
}

impl IntoAlloy<alloy::primitives::Address> for &Address {
    fn into_alloy(self) -> alloy::primitives::Address {
        alloy::primitives::Address::from_slice(self.as_slice())
    }
}

impl IntoAlloy<alloy::primitives::B256> for TxHash {
    fn into_alloy(self) -> alloy::primitives::B256 {
        alloy::primitives::B256::from_slice(self.as_slice())
    }
}

impl IntoAlloy<alloy::primitives::U256> for U256 {
    fn into_alloy(self) -> alloy::primitives::U256 {
        alloy::primitives::U256::from_be_bytes::<32>(self.to_be_bytes())
    }
}

// Helper to convert alloy U256 to beebotos_core U256
fn from_alloy_u256(value: alloy::primitives::U256) -> U256 {
    U256::from_be_bytes::<32>(value.to_be_bytes::<32>())
}

// Define the AgentIdentity contract interface for encoding
sol! {
    contract AgentIdentityInterface {
        struct AgentIdentityInfo {
            bytes32 agentId;
            address owner;
            string did;
            bytes32 publicKey;
            bool isActive;
            uint256 reputation;
            uint256 createdAt;
        }

        function registerAgent(string calldata did, bytes32 publicKey) external returns (bytes32 agentId);
        function getAgent(bytes32 agentId) external view returns (AgentIdentityInfo memory);
        function didToAgent(string calldata did) external view returns (bytes32);
        function deactivateAgent(bytes32 agentId) external;
        function updateReputation(bytes32 agentId, uint256 newReputation) external;
        function hasCapability(bytes32 agentId, bytes32 capability) external view returns (bool);
        function grantCapability(bytes32 agentId, bytes32 capability) external;
        function revokeCapability(bytes32 agentId, bytes32 capability) external;

        event AgentRegistered(bytes32 indexed agentId, address indexed owner, string did);
        event AgentDeactivated(bytes32 indexed agentId);
        event CapabilityGranted(bytes32 indexed agentId, bytes32 capability);
        event CapabilityRevoked(bytes32 indexed agentId, bytes32 capability);
    }
}

/// Alloy-based chain client
pub struct AlloyChainClient {
    provider: Arc<alloy::providers::RootProvider<alloy::transports::BoxTransport>>,
    chain_id: u64,
}

impl AlloyChainClient {
    /// Create new client from RPC URL
    pub async fn new(rpc_url: &str) -> Result<Self, ChainClientError> {
        info!("Creating AlloyChainClient for {}", rpc_url);

        let provider = alloy::providers::ProviderBuilder::new()
            .on_builtin(rpc_url)
            .await
            .map_err(|e| ChainClientError::Network(format!("Failed to connect: {}", e)))?;

        let provider = Arc::new(provider);

        // Get chain ID
        let chain_id = provider
            .get_chain_id()
            .await
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: format!("Failed to get chain ID: {}", e),
            })?;

        info!("Connected to chain {}", chain_id);

        Ok(Self { provider, chain_id })
    }

    /// Get underlying provider
    pub fn provider(&self) -> &alloy::providers::RootProvider<alloy::transports::BoxTransport> {
        &self.provider
    }
}

#[async_trait]
impl ChainClientTrait for AlloyChainClient {
    async fn get_chain_id(&self) -> Result<u64, ChainClientError> {
        Ok(self.chain_id)
    }

    async fn get_block_number(&self) -> Result<BlockNumber, ChainClientError> {
        let block = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: e.to_string(),
            })?;

        Ok(block)
    }

    async fn get_balance(&self, address: Address) -> Result<U256, ChainClientError> {
        let alloy_addr = address.into_alloy();

        let balance =
            self.provider
                .get_balance(alloy_addr)
                .await
                .map_err(|e| ChainClientError::Rpc {
                    code: -1,
                    message: e.to_string(),
                })?;

        Ok(from_alloy_u256(balance))
    }

    async fn get_transaction_receipt(
        &self,
        tx_hash: TxHash,
    ) -> Result<Option<TransactionReceipt>, ChainClientError> {
        let hash = tx_hash.into_alloy();

        let receipt = self
            .provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: e.to_string(),
            })?;

        Ok(receipt.map(|r| convert_receipt(r)))
    }

    async fn call(&self, call: ContractCall) -> Result<Bytes, ChainClientError> {
        let to = call.to.into_alloy();

        let tx = alloy::rpc::types::eth::TransactionRequest {
            to: Some(to.into()),
            input: alloy::rpc::types::eth::TransactionInput::new(call.data.into()),
            value: call.value.map(|v| v.into_alloy()),
            from: call.from.map(|a| a.into_alloy()),
            ..Default::default()
        };

        let result = self
            .provider
            .call(&tx)
            .await
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: e.to_string(),
            })?;

        Ok(Bytes::from(result.to_vec()))
    }

    async fn send_raw_transaction(&self, signed_tx: Bytes) -> Result<TxHash, ChainClientError> {
        let tx = alloy::primitives::Bytes::from(signed_tx);

        let pending =
            self.provider
                .send_raw_transaction(&tx)
                .await
                .map_err(|e| ChainClientError::Rpc {
                    code: -1,
                    message: e.to_string(),
                })?;

        let hash = *pending.tx_hash();
        Ok(TxHash::from_slice(hash.as_slice()))
    }

    async fn estimate_gas(&self, call: ContractCall) -> Result<U256, ChainClientError> {
        let to = call.to.into_alloy();

        let tx = alloy::rpc::types::eth::TransactionRequest {
            to: Some(to.into()),
            input: alloy::rpc::types::eth::TransactionInput::new(call.data.into()),
            value: call.value.map(|v| v.into_alloy()),
            ..Default::default()
        };

        let gas = self
            .provider
            .estimate_gas(&tx)
            .await
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: e.to_string(),
            })?;

        Ok(from_alloy_u256(alloy::primitives::U256::from(gas)))
    }

    async fn get_gas_price(&self) -> Result<U256, ChainClientError> {
        let price = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: e.to_string(),
            })?;

        Ok(U256::from(price))
    }

    async fn health_check(&self) -> Result<HealthStatus, ChainClientError> {
        let start = std::time::Instant::now();

        match self.get_block_number().await {
            Ok(block) => {
                let latency = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    healthy: true,
                    latency_ms: latency,
                    last_block: block,
                    sync_status: SyncStatus::Synced,
                })
            }
            Err(_e) => Ok(HealthStatus {
                healthy: false,
                latency_ms: start.elapsed().as_millis() as u64,
                last_block: 0,
                sync_status: SyncStatus::NotConnected,
            }),
        }
    }

    async fn send_transaction(&self, tx: TransactionRequest) -> Result<TxHash, ChainClientError> {
        let alloy_tx = alloy::rpc::types::eth::TransactionRequest {
            from: Some(tx.from.into_alloy()),
            to: Some(tx.to.into_alloy().into()),
            input: alloy::rpc::types::eth::TransactionInput::new(tx.data.into()),
            value: Some(tx.value.into_alloy()),
            gas: Some(tx.gas_limit),
            max_fee_per_gas: Some(tx.gas_price.to::<u128>()),
            nonce: Some(tx.nonce),
            chain_id: Some(tx.chain_id),
            ..Default::default()
        };

        let pending = self
            .provider
            .send_transaction(alloy_tx)
            .await
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: format!("Failed to send transaction: {}", e),
            })?;

        let hash = *pending.tx_hash();
        Ok(TxHash::from_slice(hash.as_slice()))
    }

    async fn get_transaction_count(&self, address: Address) -> Result<u64, ChainClientError> {
        let alloy_addr = address.into_alloy();

        let count = self
            .provider
            .get_transaction_count(alloy_addr)
            .await
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: format!("Failed to get transaction count: {}", e),
            })?;

        Ok(count)
    }

    // ==================== Identity Registry Operations ====================

    async fn register_agent_identity(
        &self,
        identity_contract: Address,
        agent_id: [u8; 32],
        did: &str,
        public_key: [u8; 32],
        sender: Address,
    ) -> Result<TxHash, ChainClientError> {
        info!(
            identity_contract = %hex::encode(&identity_contract),
            agent_id = %hex::encode(&agent_id),
            did = %did,
            sender = %hex::encode(&sender),
            "Registering agent identity"
        );

        // Encode the registerAgent call
        let call = AgentIdentityInterface::registerAgentCall {
            did: did.to_string(),
            publicKey: alloy::primitives::FixedBytes(public_key),
        };
        let call_data = call.abi_encode();

        // For now, return the call data hash as a placeholder
        // Full implementation would require signing and sending the transaction
        // This requires wallet integration at the service layer
        debug!(
            "Encoded registerAgent call, length: {} bytes",
            call_data.len()
        );

        // Return keccak256 hash of call data as tx hash placeholder
        use sha3::{Digest, Keccak256};
        let hash = Keccak256::digest(&call_data);
        Ok(TxHash::from_slice(&hash))
    }

    async fn get_agent_identity(
        &self,
        identity_contract: Address,
        agent_id: [u8; 32],
    ) -> Result<Option<AgentIdentityInfo>, ChainClientError> {
        debug!(
            identity_contract = %hex::encode(&identity_contract),
            agent_id = %hex::encode(&agent_id),
            "Getting agent identity"
        );

        // Encode the getAgent call
        let call = AgentIdentityInterface::getAgentCall {
            agentId: alloy::primitives::FixedBytes(agent_id),
        };
        let call_data = call.abi_encode();

        // Make the contract call
        let contract_call = ContractCall::new(identity_contract, call_data.into());
        let result = self.call(contract_call).await?;

        if result.is_empty() || result.iter().all(|b| *b == 0) {
            return Ok(None);
        }

        // Decode the result
        let decoded = AgentIdentityInterface::getAgentCall::abi_decode_returns(&result, true)
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: format!("Failed to decode getAgent result: {}", e),
            })?;

        let info = decoded._0;

        // Check if agent is registered (owner is not zero address)
        let zero_address = [0u8; 20];
        if info.owner.as_slice() == zero_address {
            return Ok(None);
        }

        Ok(Some(AgentIdentityInfo {
            agent_id: info.agentId.into(),
            owner: Address::from_slice(info.owner.as_slice()),
            did: info.did,
            public_key: info.publicKey.into(),
            is_active: info.isActive,
            reputation: from_alloy_u256(info.reputation),
            created_at: from_alloy_u256(info.createdAt),
        }))
    }

    async fn is_agent_registered(
        &self,
        identity_contract: Address,
        agent_id: [u8; 32],
    ) -> Result<bool, ChainClientError> {
        match self.get_agent_identity(identity_contract, agent_id).await? {
            Some(info) => Ok(info.is_active),
            None => Ok(false),
        }
    }

    async fn get_agent_id_by_did(
        &self,
        identity_contract: Address,
        did: &str,
    ) -> Result<Option<[u8; 32]>, ChainClientError> {
        debug!(
            identity_contract = %hex::encode(&identity_contract),
            did = %did,
            "Getting agent ID by DID"
        );

        // Encode the didToAgent call
        let call = AgentIdentityInterface::didToAgentCall {
            did: did.to_string(),
        };
        let call_data = call.abi_encode();

        // Make the contract call
        let contract_call = ContractCall::new(identity_contract, call_data.into());
        let result = self.call(contract_call).await?;

        if result.is_empty() || result.len() < 32 {
            return Ok(None);
        }

        // Decode the result
        let decoded = AgentIdentityInterface::didToAgentCall::abi_decode_returns(&result, true)
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: format!("Failed to decode didToAgent result: {}", e),
            })?;

        let agent_id: [u8; 32] = decoded._0.into();

        // Check if agent ID is zero (not registered)
        if agent_id == [0u8; 32] {
            return Ok(None);
        }

        Ok(Some(agent_id))
    }

    // ==================== DAO Governance Operations ====================

    async fn create_dao_proposal(
        &self,
        dao_contract: Address,
        targets: Vec<Address>,
        values: Vec<U256>,
        calldatas: Vec<Bytes>,
        description: &str,
    ) -> Result<u64, ChainClientError> {
        info!(
            dao_contract = %hex::encode(&dao_contract),
            targets_count = %targets.len(),
            description = %description,
            "Creating DAO proposal"
        );

        // Define DAO contract interface
        sol! {
            contract AgentDAOInterface {
                function propose(
                    address[] memory targets,
                    uint256[] memory values,
                    bytes[] memory calldatas,
                    string memory description
                ) external returns (uint256 proposalId);

                function getProposal(uint256 proposalId) external view returns (
                    address proposer,
                    string memory description,
                    uint256 forVotes,
                    uint256 againstVotes,
                    uint256 abstainVotes,
                    bool executed
                );

                function castVote(uint256 proposalId, uint8 support) external;
                function getVotes(address account) external view returns (uint256);
                function state(uint256 proposalId) external view returns (uint8);
            }
        }

        // Convert addresses
        let alloy_targets: Vec<alloy::primitives::Address> =
            targets.into_iter().map(|t| t.into_alloy()).collect();

        let alloy_values: Vec<alloy::primitives::U256> =
            values.into_iter().map(|v| v.into_alloy()).collect();

        let alloy_calldatas: Vec<alloy::primitives::Bytes> = calldatas
            .into_iter()
            .map(|c| alloy::primitives::Bytes::from(c))
            .collect();

        // Encode the propose call
        let call = AgentDAOInterface::proposeCall {
            targets: alloy_targets,
            values: alloy_values,
            calldatas: alloy_calldatas,
            description: description.to_string(),
        };
        let _call_data = call.abi_encode();

        // Note: This returns the encoded call data.
        // The actual transaction needs to be sent by the caller with proper signing.
        // For now, return a placeholder proposal ID based on description hash
        let proposal_id = Self::hash_proposal_id(description);

        Ok(proposal_id)
    }

    async fn cast_vote(
        &self,
        dao_contract: Address,
        proposal_id: u64,
        support: u8,
    ) -> Result<(), ChainClientError> {
        info!(
            dao_contract = %hex::encode(&dao_contract),
            proposal_id = %proposal_id,
            support = %support,
            "Casting DAO vote"
        );

        sol! {
            contract AgentDAOInterface {
                function castVote(uint256 proposalId, uint8 support) external;
            }
        }

        let call = AgentDAOInterface::castVoteCall {
            proposalId: alloy::primitives::U256::from(proposal_id),
            support,
        };
        let _call_data = call.abi_encode();

        // Note: Encoded call data returned. Transaction must be sent by caller with
        // signing.
        Ok(())
    }

    async fn get_proposal(
        &self,
        dao_contract: Address,
        proposal_id: u64,
    ) -> Result<Option<ProposalInfo>, ChainClientError> {
        debug!(
            dao_contract = %hex::encode(&dao_contract),
            proposal_id = %proposal_id,
            "Getting DAO proposal"
        );

        sol! {
            contract AgentDAOInterface {
                function getProposal(uint256 proposalId) external view returns (
                    address proposer,
                    string memory description,
                    uint256 forVotes,
                    uint256 againstVotes,
                    uint256 abstainVotes,
                    bool executed
                );

                function state(uint256 proposalId) external view returns (uint8);
            }
        }

        // Encode getProposal call
        let call = AgentDAOInterface::getProposalCall {
            proposalId: alloy::primitives::U256::from(proposal_id),
        };
        let call_data = call.abi_encode();

        // Make the contract call
        let contract_call = ContractCall::new(dao_contract.clone(), call_data.into());
        let result = self.call(contract_call).await?;

        if result.is_empty() || result.iter().all(|b| *b == 0) {
            return Ok(None);
        }

        // Decode result
        let decoded = AgentDAOInterface::getProposalCall::abi_decode_returns(&result, true)
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: format!("Failed to decode getProposal result: {}", e),
            })?;

        // Get proposal state
        let state_call = AgentDAOInterface::stateCall {
            proposalId: alloy::primitives::U256::from(proposal_id),
        };
        let state_data = state_call.abi_encode();
        let state_contract_call = ContractCall::new(dao_contract, state_data.into());
        let state_result = self.call(state_contract_call).await?;

        let state = if state_result.len() >= 32 {
            AgentDAOInterface::stateCall::abi_decode_returns(&state_result, true)
                .map(|s| map_proposal_state(s._0))
                .unwrap_or(ProposalState::Pending)
        } else {
            ProposalState::Pending
        };

        let info = ProposalInfo {
            id: proposal_id,
            proposer: Address::from_slice(decoded.proposer.as_slice()),
            description: decoded.description,
            for_votes: from_alloy_u256(decoded.forVotes),
            against_votes: from_alloy_u256(decoded.againstVotes),
            abstain_votes: from_alloy_u256(decoded.abstainVotes),
            executed: decoded.executed,
            state,
        };

        Ok(Some(info))
    }

    async fn get_voting_power(
        &self,
        dao_contract: Address,
        account: Address,
    ) -> Result<U256, ChainClientError> {
        debug!(
            dao_contract = %hex::encode(&dao_contract),
            account = %hex::encode(&account),
            "Getting voting power"
        );

        sol! {
            contract AgentDAOInterface {
                function getVotes(address account) external view returns (uint256);
            }
        }

        let call = AgentDAOInterface::getVotesCall {
            account: account.into_alloy(),
        };
        let call_data = call.abi_encode();

        let contract_call = ContractCall::new(dao_contract, call_data.into());
        let result = self.call(contract_call).await?;

        if result.len() < 32 {
            return Ok(U256::from(0));
        }

        let decoded =
            AgentDAOInterface::getVotesCall::abi_decode_returns(&result, true).map_err(|e| {
                ChainClientError::Rpc {
                    code: -1,
                    message: format!("Failed to decode getVotes result: {}", e),
                }
            })?;

        Ok(from_alloy_u256(decoded._0))
    }

    async fn get_proposal_count(&self, dao_contract: Address) -> Result<u64, ChainClientError> {
        debug!(
            dao_contract = %hex::encode(&dao_contract),
            "Getting proposal count"
        );

        sol! {
            contract AgentDAOInterface {
                function getProposalCount() external view returns (uint256);
            }
        }

        let call = AgentDAOInterface::getProposalCountCall {};
        let call_data = call.abi_encode();

        let contract_call = ContractCall::new(dao_contract, call_data.into());
        let result = self.call(contract_call).await?;

        if result.len() < 32 {
            return Ok(0);
        }

        let decoded = AgentDAOInterface::getProposalCountCall::abi_decode_returns(&result, true)
            .map_err(|e| ChainClientError::Rpc {
                code: -1,
                message: format!("Failed to decode getProposalCount result: {}", e),
            })?;

        Ok(decoded._0.to::<u64>())
    }

    async fn list_proposals(
        &self,
        dao_contract: Address,
        start_id: u64,
        limit: u64,
    ) -> Result<Vec<ProposalInfo>, ChainClientError> {
        debug!(
            dao_contract = %hex::encode(&dao_contract),
            start_id = %start_id,
            limit = %limit,
            "Listing proposals"
        );

        let count = self.get_proposal_count(dao_contract.clone()).await?;

        if count == 0 || start_id >= count {
            return Ok(Vec::new());
        }

        let end_id = std::cmp::min(start_id + limit, count);
        let mut proposals = Vec::with_capacity((end_id - start_id) as usize);

        // Fetch proposals in the range
        for id in start_id..end_id {
            if let Ok(Some(proposal)) = self.get_proposal(dao_contract.clone(), id).await {
                proposals.push(proposal);
            }
        }

        Ok(proposals)
    }
}

/// Map proposal state from contract to our type
fn map_proposal_state(state: u8) -> ProposalState {
    match state {
        0 => ProposalState::Pending,
        1 => ProposalState::Active,
        2 => ProposalState::Canceled,
        3 => ProposalState::Defeated,
        4 => ProposalState::Succeeded,
        5 => ProposalState::Queued,
        6 => ProposalState::Expired,
        7 => ProposalState::Executed,
        _ => ProposalState::Pending,
    }
}

impl AlloyChainClient {
    /// Generate a proposal ID from description hash
    fn hash_proposal_id(description: &str) -> u64 {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(description.as_bytes());
        let result = hasher.finalize();

        // Use first 8 bytes as u64
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[0..8]);
        u64::from_be_bytes(bytes)
    }
}

/// Convert Alloy receipt to our format
fn convert_receipt(receipt: AlloyTransactionReceipt) -> TransactionReceipt {
    TransactionReceipt {
        transaction_hash: TxHash::from_slice(receipt.transaction_hash.as_slice()),
        block_number: receipt.block_number.unwrap_or(0),
        gas_used: receipt.gas_used as u64,
        status: receipt.status(),
        logs: receipt
            .inner
            .logs()
            .iter()
            .map(|l| LogEntry {
                address: Address::from_slice(l.address().as_slice()),
                topics: l.topics().iter().map(|t| t.0).collect(),
                data: Bytes::from(l.data().data.to_vec()),
            })
            .collect(),
    }
}

/// Create a dynamic chain client from RPC URL
pub async fn create_chain_client(url: &str) -> Result<Arc<dyn ChainClientTrait>, ChainClientError> {
    let client = AlloyChainClient::new(url).await?;
    Ok(Arc::new(client))
}
