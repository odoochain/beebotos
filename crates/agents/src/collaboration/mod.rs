//! Collaboration Module
//!
//! Multi-agent collaboration support.
//!
//! ## High Priority Features (OpenClaw Compatible)
//! - **Round Table**: Peer-to-peer collaboration with equal voting rights

pub mod hub;
pub mod spoke;
pub mod routing;
pub mod workflow;
pub mod round_table;
pub mod round_table_integration;

pub use hub::CollaborationHub;
pub use spoke::SpokeAgent;
pub use routing::CollaborationRouter;
pub use workflow::CollaborationWorkflow;
pub use round_table::{
    RoundTableCouncil, RoundTableConfig, RoundTableParticipant, RoundTableRole,
    RoundTableEvent, Proposal, ProposalType, Vote, VoteRecord, VotingResult,
    VotingStatus, Conflict, ConflictType, ConflictResolution, ResolutionMethod,
    ParticipantStatus, DEFAULT_CONSENSUS_THRESHOLD, DEFAULT_VOTING_TIMEOUT_SECS,
};
pub use round_table_integration::{
    IntegratedRoundTable, RoundTableIntegrationEvent, RoundTableWorkflow,
    RoundTableWorkflowStatus, RoundTableMessage, DebateSession, DebateSide,
    DebateArgument, RoundTableStepType,
};

use crate::error::Result;
use uuid::Uuid;

/// Collaboration message
#[derive(Debug, Clone)]
pub struct CollabMessage {
    pub from: Uuid,
    pub to: Option<Uuid>,
    pub content: String,
    pub message_type: CollabMessageType,
}

#[derive(Debug, Clone)]
pub enum CollabMessageType {
    Request,
    Response,
    Broadcast,
    Coordination,
}
