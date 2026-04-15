//! Round Table Collaboration Mechanism
//!
//! Implements a peer-to-peer multi-agent collaboration system with equal voting rights,
//! shared fact sources, and conflict resolution. Unlike hub-spoke models, all agents
//! in the round table have equal authority.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Round Table Council                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │    ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
//! │    │  Scout  │  │  Quill  │  │ Observer│  │ Analyst │         │
//! │    │(Research│  │ (Write) │  │(Review) │  │ (Data)  │         │
//! │    └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘         │
//! │         │            │            │            │               │
//! │         └────────────┴─────┬──────┴────────────┘               │
//! │                            │                                    │
//! │                    ┌───────▼────────┐                          │
//! │                    │  Shared Facts  │  ← AGENTS.md              │
//! │                    │   & Consensus  │                          │
//! │                    └───────┬────────┘                          │
//! │                            │                                    │
//! │         ┌──────────────────┼──────────────────┐                │
//! │         │                  │                  │                │
//! │    ┌────▼────┐       ┌────▼────┐       ┌────▼────┐           │
//! │    │Proposal │       │ Voting  │       │Conflict │           │
//! │    │ Phase   │──────▶│ Phase   │──────▶│Resolution│          │
//! │    └─────────┘       └─────────┘       └─────────┘           │
//! │                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//! - Equal voting rights for all agents
//! - Proposal-based decision making
//! - Multiple conflict resolution strategies
//! - Shared fact source (AGENTS.md)
//! - Consensus tracking
//! - Human arbitration support

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{timeout, Duration};
use tracing::{info, warn, debug, error};
use uuid::Uuid;

/// Default voting timeout in seconds
pub const DEFAULT_VOTING_TIMEOUT_SECS: u64 = 60;
/// Default consensus threshold (percentage)
pub const DEFAULT_CONSENSUS_THRESHOLD: f32 = 0.67; // 2/3 majority
/// Default round table size limit
pub const DEFAULT_MAX_PARTICIPANTS: usize = 10;

/// Agent role in round table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoundTableRole {
    /// Research and information gathering
    Scout,
    /// Content creation and writing
    Quill,
    /// Review and verification
    Observer,
    /// Data analysis
    Analyst,
    /// Creative ideation
    Muse,
    /// Strategic planning
    Strategist,
    /// Coordinator (facilitates but has equal vote)
    Coordinator,
    /// Custom role
    Custom(String),
}

impl RoundTableRole {
    /// Get role description
    pub fn description(&self) -> &str {
        match self {
            RoundTableRole::Scout => "Researches and gathers information",
            RoundTableRole::Quill => "Creates and refines content",
            RoundTableRole::Observer => "Reviews and verifies accuracy",
            RoundTableRole::Analyst => "Analyzes data and patterns",
            RoundTableRole::Muse => "Generates creative ideas",
            RoundTableRole::Strategist => "Plans and coordinates strategy",
            RoundTableRole::Coordinator => "Facilitates discussion without bias",
            RoundTableRole::Custom(_) => "Specialized custom role",
        }
    }

    /// Get default capabilities for role
    pub fn default_capabilities(&self) -> Vec<String> {
        match self {
            RoundTableRole::Scout => vec!["research".to_string(), "search".to_string()],
            RoundTableRole::Quill => vec!["write".to_string(), "edit".to_string()],
            RoundTableRole::Observer => vec!["review".to_string(), "verify".to_string()],
            RoundTableRole::Analyst => vec!["analyze".to_string(), "calculate".to_string()],
            RoundTableRole::Muse => vec!["create".to_string(), "brainstorm".to_string()],
            RoundTableRole::Strategist => vec!["plan".to_string(), "coordinate".to_string()],
            RoundTableRole::Coordinator => vec!["facilitate".to_string(), "mediate".to_string()],
            RoundTableRole::Custom(_) => vec!["specialize".to_string()],
        }
    }
}

/// Round table participant (agent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundTableParticipant {
    /// Agent ID
    pub id: Uuid,
    /// Agent name
    pub name: String,
    /// Role in round table
    pub role: RoundTableRole,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Current status
    pub status: ParticipantStatus,
    /// Join time
    pub joined_at: chrono::DateTime<chrono::Utc>,
    /// Voting weight (default: 1.0, equal for all)
    pub voting_weight: f32,
}

impl RoundTableParticipant {
    /// Create new participant
    pub fn new(id: Uuid, name: impl Into<String>, role: RoundTableRole) -> Self {
        let capabilities = role.default_capabilities();
        
        Self {
            id,
            name: name.into(),
            role,
            capabilities,
            status: ParticipantStatus::Active,
            joined_at: chrono::Utc::now(),
            voting_weight: 1.0,
        }
    }
}

/// Participant status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantStatus {
    Active,
    Away,
    Busy,
    Offline,
}

/// Proposal for round table decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Proposal ID
    pub id: Uuid,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Proposing agent
    pub proposer_id: Uuid,
    /// Proposal type
    pub proposal_type: ProposalType,
    /// Proposed solution/content
    pub content: String,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expires at
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Type of proposal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    /// Content creation or modification
    Content,
    /// Strategic decision
    Strategy,
    /// Resource allocation
    Resource,
    /// Process change
    Process,
    /// Conflict resolution
    ConflictResolution,
}

/// Vote on a proposal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Approve,
    Reject,
    Abstain,
}

impl Vote {
    /// Get numeric weight
    pub fn weight(&self) -> f32 {
        match self {
            Vote::Approve => 1.0,
            Vote::Reject => -1.0,
            Vote::Abstain => 0.0,
        }
    }
}

/// Vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRecord {
    pub participant_id: Uuid,
    pub vote: Vote,
    pub reason: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Voting result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    pub proposal_id: Uuid,
    pub status: VotingStatus,
    pub approve_count: usize,
    pub reject_count: usize,
    pub abstain_count: usize,
    pub total_weight: f32,
    pub approval_ratio: f32,
    pub votes: Vec<VoteRecord>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Voting status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VotingStatus {
    Pending,
    Approved,
    Rejected,
    Tied,
    Expired,
    Cancelled,
}

/// Conflict record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Conflict ID
    pub id: Uuid,
    /// Participating agent IDs
    pub participant_ids: Vec<Uuid>,
    /// Conflict description
    pub description: String,
    /// Proposed solutions
    pub proposals: Vec<Proposal>,
    /// Conflict type
    pub conflict_type: ConflictType,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Resolved at
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Resolution
    pub resolution: Option<ConflictResolution>,
}

/// Type of conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Disagreement on approach
    Approach,
    /// Resource contention
    Resource,
    /// Priority conflict
    Priority,
    /// Data/interpretation disagreement
    Interpretation,
    /// Goal misalignment
    Goal,
}

/// Conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub method: ResolutionMethod,
    pub winning_proposal_id: Option<Uuid>,
    pub explanation: String,
    pub human_arbitrated: bool,
}

/// Resolution method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionMethod {
    /// Simple majority vote
    MajorityVote,
    /// Weighted voting
    WeightedVote,
    /// Consensus reached
    Consensus,
    /// Compromise solution
    Compromise,
    /// Rotating priority
    RotatingPriority,
    /// Human arbitration
    HumanArbitration,
    /// Delegation to expert
    ExpertDelegation,
}

/// Round table configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundTableConfig {
    /// Maximum participants
    pub max_participants: usize,
    /// Voting timeout
    pub voting_timeout_secs: u64,
    /// Consensus threshold
    pub consensus_threshold: f32,
    /// Enable human arbitration
    pub enable_human_arbitration: bool,
    /// Auto-resolve conflicts
    pub auto_resolve_conflicts: bool,
    /// Shared fact source file (AGENTS.md)
    pub shared_fact_source: String,
}

impl Default for RoundTableConfig {
    fn default() -> Self {
        Self {
            max_participants: DEFAULT_MAX_PARTICIPANTS,
            voting_timeout_secs: DEFAULT_VOTING_TIMEOUT_SECS,
            consensus_threshold: DEFAULT_CONSENSUS_THRESHOLD,
            enable_human_arbitration: true,
            auto_resolve_conflicts: true,
            shared_fact_source: "AGENTS.md".to_string(),
        }
    }
}

/// Round table council
pub struct RoundTableCouncil {
    config: RoundTableConfig,
    /// Participants
    participants: Arc<RwLock<HashMap<Uuid, RoundTableParticipant>>>,
    /// Active proposals
    proposals: Arc<RwLock<HashMap<Uuid, Proposal>>>,
    /// Votes per proposal
    votes: Arc<RwLock<HashMap<Uuid, Vec<VoteRecord>>>>,
    /// Active conflicts
    conflicts: Arc<RwLock<HashMap<Uuid, Conflict>>>,
    /// Shared facts (in-memory cache of AGENTS.md)
    shared_facts: Arc<RwLock<HashMap<String, String>>>,
    /// Event sender
    event_sender: mpsc::Sender<RoundTableEvent>,
    /// Event receiver
    _event_receiver: Arc<RwLock<mpsc::Receiver<RoundTableEvent>>>,
}

/// Round table events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoundTableEvent {
    ParticipantJoined { participant: RoundTableParticipant },
    ParticipantLeft { participant_id: Uuid },
    ProposalCreated { proposal: Proposal },
    VoteCast { proposal_id: Uuid, vote: VoteRecord },
    VotingCompleted { result: VotingResult },
    ConflictDetected { conflict: Conflict },
    ConflictResolved { conflict_id: Uuid, resolution: ConflictResolution },
    ConsensusReached { topic: String, participants: Vec<Uuid> },
}

impl RoundTableCouncil {
    /// Create new round table council
    pub fn new(config: RoundTableConfig) -> (Self, mpsc::Receiver<RoundTableEvent>) {
        let (event_sender, event_receiver) = mpsc::channel(100);
        
        let council = Self {
            config,
            participants: Arc::new(RwLock::new(HashMap::new())),
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            conflicts: Arc::new(RwLock::new(HashMap::new())),
            shared_facts: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            _event_receiver: Arc::new(RwLock::new(event_receiver)),
        };

        (council, event_receiver)
    }

    /// Create with default configuration
    pub fn default() -> (Self, mpsc::Receiver<RoundTableEvent>) {
        Self::new(RoundTableConfig::default())
    }

    /// Join the round table
    pub async fn join(&self, participant: RoundTableParticipant) -> Result<()> {
        let mut participants = self.participants.write().await;
        
        if participants.len() >= self.config.max_participants {
            return Err(crate::error::AgentError::platform(
                "Round table is full".to_string()
            ));
        }

        let id = participant.id;
        participants.insert(id, participant.clone());

        let _ = self.event_sender.send(RoundTableEvent::ParticipantJoined { participant }).await;
        
        info!("Agent {} joined round table as {:?}", id, participant.role);
        Ok(())
    }

    /// Leave the round table
    pub async fn leave(&self, participant_id: Uuid) -> Result<()> {
        let mut participants = self.participants.write().await;
        
        if participants.remove(&participant_id).is_some() {
            let _ = self.event_sender.send(RoundTableEvent::ParticipantLeft { participant_id }).await;
            info!("Agent {} left round table", participant_id);
        }

        Ok(())
    }

    /// Create a proposal
    pub async fn propose(&self, mut proposal: Proposal) -> Result<Uuid> {
        let mut proposals = self.proposals.write().await;
        
        // Set expiration if not set
        if proposal.expires_at.is_none() {
            proposal.expires_at = Some(
                chrono::Utc::now() + chrono::Duration::seconds(self.config.voting_timeout_secs as i64)
            );
        }

        let id = proposal.id;
        proposals.insert(id, proposal.clone());

        // Initialize vote storage
        let mut votes = self.votes.write().await;
        votes.insert(id, Vec::new());

        let _ = self.event_sender.send(RoundTableEvent::ProposalCreated { proposal }).await;
        
        info!("Proposal {} created by {}", id, proposal.proposer_id);
        Ok(id)
    }

    /// Cast a vote
    pub async fn vote(
        &self,
        participant_id: Uuid,
        proposal_id: Uuid,
        vote: Vote,
        reason: Option<String>,
    ) -> Result<()> {
        // Verify participant exists
        let participants = self.participants.read().await;
        if !participants.contains_key(&participant_id) {
            return Err(crate::error::AgentError::not_found(
                "Participant not in round table".to_string()
            ));
        }

        // Verify proposal exists and is active
        let proposals = self.proposals.read().await;
        let proposal = proposals.get(&proposal_id)
            .ok_or_else(|| crate::error::AgentError::not_found("Proposal not found".to_string()))?;

        // Check if expired
        if let Some(expires_at) = proposal.expires_at {
            if chrono::Utc::now() > expires_at {
                return Err(crate::error::AgentError::platform(
                    "Voting period has expired".to_string()
                ));
            }
        }

        drop(proposals);
        drop(participants);

        // Record vote
        let vote_record = VoteRecord {
            participant_id,
            vote,
            reason,
            timestamp: chrono::Utc::now(),
        };

        let mut votes = self.votes.write().await;
        if let Some(vote_list) = votes.get_mut(&proposal_id) {
            // Remove existing vote if any
            vote_list.retain(|v| v.participant_id != participant_id);
            vote_list.push(vote_record.clone());
        }

        let _ = self.event_sender.send(RoundTableEvent::VoteCast { 
            proposal_id, 
            vote: vote_record 
        }).await;

        debug!("Vote cast by {} on proposal {}: {:?}", participant_id, proposal_id, vote);
        Ok(())
    }

    /// Close voting and get result
    pub async fn close_voting(&self, proposal_id: Uuid) -> Result<VotingResult> {
        let proposals = self.proposals.read().await;
        let proposal = proposals.get(&proposal_id)
            .ok_or_else(|| crate::error::AgentError::not_found("Proposal not found".to_string()))?;

        let votes = self.votes.read().await;
        let vote_list = votes.get(&proposal_id).cloned().unwrap_or_default();

        let participants = self.participants.read().await;
        let total_participants = participants.len();

        // Calculate results
        let mut approve_weight = 0.0;
        let mut reject_weight = 0.0;
        let mut abstain_weight = 0.0;

        for vote_record in &vote_list {
            if let Some(participant) = participants.get(&vote_record.participant_id) {
                let weight = participant.voting_weight * vote_record.vote.weight().abs();
                match vote_record.vote {
                    Vote::Approve => approve_weight += weight,
                    Vote::Reject => reject_weight += weight,
                    Vote::Abstain => abstain_weight += weight,
                }
            }
        }

        let total_votes = vote_list.len();
        let approve_count = vote_list.iter().filter(|v| v.vote == Vote::Approve).count();
        let reject_count = vote_list.iter().filter(|v| v.vote == Vote::Reject).count();
        let abstain_count = vote_list.iter().filter(|v| v.vote == Vote::Abstain).count();

        let total_weight = approve_weight + reject_weight;
        let approval_ratio = if total_weight > 0.0 {
            approve_weight / total_weight
        } else {
            0.0
        };

        // Determine status
        let status = if approve_weight > reject_weight && 
                      approval_ratio >= self.config.consensus_threshold {
            VotingStatus::Approved
        } else if reject_weight > approve_weight {
            VotingStatus::Rejected
        } else if approve_weight == reject_weight && total_votes > 0 {
            VotingStatus::Tied
        } else if total_votes == 0 {
            VotingStatus::Expired
        } else {
            VotingStatus::Pending
        };

        let result = VotingResult {
            proposal_id,
            status,
            approve_count,
            reject_count,
            abstain_count,
            total_weight,
            approval_ratio,
            votes: vote_list,
            completed_at: chrono::Utc::now(),
        };

        let _ = self.event_sender.send(RoundTableEvent::VotingCompleted { 
            result: result.clone() 
        }).await;

        // If tied, create a conflict for resolution
        if status == VotingStatus::Tied {
            self.create_conflict(
                proposal.proposer_id,
                format!("Tied vote on proposal: {}", proposal.title),
                ConflictType::Approach,
            ).await?;
        }

        info!("Voting closed for proposal {}: {:?}", proposal_id, status);
        Ok(result)
    }

    /// Create a conflict record
    pub async fn create_conflict(
        &self,
        initiator_id: Uuid,
        description: String,
        conflict_type: ConflictType,
    ) -> Result<Uuid> {
        let conflict = Conflict {
            id: Uuid::new_v4(),
            participant_ids: vec![initiator_id],
            description,
            proposals: Vec::new(),
            conflict_type,
            created_at: chrono::Utc::now(),
            resolved_at: None,
            resolution: None,
        };

        let id = conflict.id;
        let mut conflicts = self.conflicts.write().await;
        conflicts.insert(id, conflict.clone());

        let _ = self.event_sender.send(RoundTableEvent::ConflictDetected { conflict }).await;

        info!("Conflict {} created", id);
        Ok(id)
    }

    /// Resolve a conflict
    pub async fn resolve_conflict(
        &self,
        conflict_id: Uuid,
        resolution: ConflictResolution,
    ) -> Result<()> {
        let mut conflicts = self.conflicts.write().await;
        
        if let Some(conflict) = conflicts.get_mut(&conflict_id) {
            conflict.resolved_at = Some(chrono::Utc::now());
            conflict.resolution = Some(resolution.clone());

            let _ = self.event_sender.send(RoundTableEvent::ConflictResolved { 
                conflict_id, 
                resolution 
            }).await;

            info!("Conflict {} resolved using {:?}", conflict_id, resolution.method);
        }

        Ok(())
    }

    /// Add participant to conflict
    pub async fn join_conflict(&self, conflict_id: Uuid, participant_id: Uuid) -> Result<()> {
        let mut conflicts = self.conflicts.write().await;
        
        if let Some(conflict) = conflicts.get_mut(&conflict_id) {
            if !conflict.participant_ids.contains(&participant_id) {
                conflict.participant_ids.push(participant_id);
            }
        }

        Ok(())
    }

    /// Add proposal to conflict
    pub async fn propose_resolution(
        &self,
        conflict_id: Uuid,
        proposal: Proposal,
    ) -> Result<()> {
        let mut conflicts = self.conflicts.write().await;
        
        if let Some(conflict) = conflicts.get_mut(&conflict_id) {
            conflict.proposals.push(proposal);
        }

        Ok(())
    }

    /// Update shared fact
    pub async fn update_shared_fact(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut facts = self.shared_facts.write().await;
        facts.insert(key.into(), value.into());
    }

    /// Get shared fact
    pub async fn get_shared_fact(&self, key: &str) -> Option<String> {
        let facts = self.shared_facts.read().await;
        facts.get(key).cloned()
    }

    /// Get all participants
    pub async fn get_participants(&self) -> Vec<RoundTableParticipant> {
        let participants = self.participants.read().await;
        participants.values().cloned().collect()
    }

    /// Get active proposals
    pub async fn get_active_proposals(&self) -> Vec<Proposal> {
        let proposals = self.proposals.read().await;
        let now = chrono::Utc::now();
        
        proposals.values()
            .filter(|p| {
                if let Some(expires_at) = p.expires_at {
                    now < expires_at
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    /// Get active conflicts
    pub async fn get_active_conflicts(&self) -> Vec<Conflict> {
        let conflicts = self.conflicts.read().await;
        
        conflicts.values()
            .filter(|c| c.resolved_at.is_none())
            .cloned()
            .collect()
    }

    /// Request human arbitration
    pub async fn request_human_arbitration(&self, conflict_id: Uuid) -> Result<()> {
        if !self.config.enable_human_arbitration {
            return Err(crate::error::AgentError::platform(
                "Human arbitration is disabled".to_string()
            ));
        }

        warn!("Human arbitration requested for conflict {}", conflict_id);
        // In real implementation, this would notify the user through appropriate channels
        
        Ok(())
    }

    /// Check for consensus on a topic
    pub async fn check_consensus(&self, topic: &str, proposal_ids: &[Uuid]) -> Option<Vec<Uuid>> {
        let votes = self.votes.read().await;
        let participants = self.participants.read().await;
        
        let mut supporters = HashSet::new();
        
        for proposal_id in proposal_ids {
            if let Some(vote_list) = votes.get(proposal_id) {
                for vote in vote_list {
                    if vote.vote == Vote::Approve {
                        supporters.insert(vote.participant_id);
                    }
                }
            }
        }

        let total = participants.len();
        let supporting = supporters.len();
        
        if total > 0 && (supporting as f32 / total as f32) >= self.config.consensus_threshold {
            let consensus_participants = supporters.into_iter().collect();
            let _ = self.event_sender.send(RoundTableEvent::ConsensusReached {
                topic: topic.to_string(),
                participants: consensus_participants.clone(),
            }).await;
            Some(consensus_participants)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_table_role_description() {
        assert!(!RoundTableRole::Scout.description().is_empty());
        assert!(!RoundTableRole::Quill.description().is_empty());
    }

    #[test]
    fn test_round_table_role_capabilities() {
        let scout_caps = RoundTableRole::Scout.default_capabilities();
        assert!(scout_caps.contains(&"research".to_string()));
        
        let quill_caps = RoundTableRole::Quill.default_capabilities();
        assert!(quill_caps.contains(&"write".to_string()));
    }

    #[test]
    fn test_vote_weight() {
        assert_eq!(Vote::Approve.weight(), 1.0);
        assert_eq!(Vote::Reject.weight(), -1.0);
        assert_eq!(Vote::Abstain.weight(), 0.0);
    }

    #[test]
    fn test_round_table_config_default() {
        let config = RoundTableConfig::default();
        assert_eq!(config.max_participants, 10);
        assert_eq!(config.consensus_threshold, 0.67);
        assert!(config.enable_human_arbitration);
    }

    #[tokio::test]
    async fn test_round_table_join_leave() {
        let (council, _receiver) = RoundTableCouncil::default();

        let participant = RoundTableParticipant::new(
            Uuid::new_v4(),
            "Test Agent",
            RoundTableRole::Analyst,
        );

        council.join(participant.clone()).await.unwrap();
        let participants = council.get_participants().await;
        assert_eq!(participants.len(), 1);

        council.leave(participant.id).await.unwrap();
        let participants = council.get_participants().await;
        assert_eq!(participants.len(), 0);
    }

    #[tokio::test]
    async fn test_round_table_voting() {
        let (council, _receiver) = RoundTableCouncil::default();

        // Add participants
        let p1 = RoundTableParticipant::new(Uuid::new_v4(), "Agent 1", RoundTableRole::Scout);
        let p2 = RoundTableParticipant::new(Uuid::new_v4(), "Agent 2", RoundTableRole::Quill);
        let p3 = RoundTableParticipant::new(Uuid::new_v4(), "Agent 3", RoundTableRole::Observer);

        council.join(p1.clone()).await.unwrap();
        council.join(p2.clone()).await.unwrap();
        council.join(p3.clone()).await.unwrap();

        // Create proposal
        let proposal = Proposal {
            id: Uuid::new_v4(),
            title: "Test Proposal".to_string(),
            description: "A test proposal".to_string(),
            proposer_id: p1.id,
            proposal_type: ProposalType::Content,
            content: "Test content".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        let proposal_id = council.propose(proposal).await.unwrap();

        // Cast votes
        council.vote(p1.id, proposal_id, Vote::Approve, None).await.unwrap();
        council.vote(p2.id, proposal_id, Vote::Approve, None).await.unwrap();
        council.vote(p3.id, proposal_id, Vote::Reject, None).await.unwrap();

        // Close voting
        let result = council.close_voting(proposal_id).await.unwrap();
        
        assert_eq!(result.approve_count, 2);
        assert_eq!(result.reject_count, 1);
    }

    #[tokio::test]
    async fn test_shared_facts() {
        let (council, _receiver) = RoundTableCouncil::default();

        council.update_shared_fact("key1", "value1").await;
        council.update_shared_fact("key2", "value2").await;

        assert_eq!(council.get_shared_fact("key1").await, Some("value1".to_string()));
        assert_eq!(council.get_shared_fact("key2").await, Some("value2".to_string()));
        assert_eq!(council.get_shared_fact("missing").await, None);
    }
}
