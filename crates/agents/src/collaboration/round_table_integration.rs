//! Round Table Integration with A2A/MCP Protocols and Workflow Engine
//!
//! Integrates the Round Table collaboration mechanism with:
//! - A2A (Agent-to-Agent) protocol for message transport
//! - MCP (Model Context Protocol) for context sharing
//! - Collaboration Workflow engine for structured processes
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │              Round Table Integration Layer                      │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
//! │  │  Round Table │  │     A2A      │  │     MCP      │         │
//! │  │   Council    │◄─┤   Protocol   │◄─┤   Protocol   │         │
//! │  │              │  │  (Transport) │  │   (Context)  │         │
//! │  └──────┬───────┘  └──────────────┘  └──────────────┘         │
//! │         │                                                      │
//! │         ▼                                                      │
//! │  ┌──────────────────────────────────────────────────────┐    │
//! │  │           Collaboration Workflow Engine              │    │
//! │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │    │
//! │  │  │ Proposal│ │ Debate  │ │  Vote   │ │ Execute │   │    │
//! │  │  │  Step   │ │  Step   │ │  Step   │ │  Step   │   │    │
//! │  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘   │    │
//! │  └──────────────────────────────────────────────────────┘    │
//! │                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use crate::a2a::{
    message::{A2AMessage, MessagePayload, MessageType, NegotiationOffer},
    A2AClient, A2AError,
};
use crate::collaboration::{
    round_table::{
        Conflict, ConflictResolution, ConflictType, Proposal, ProposalType, ResolutionMethod,
        RoundTableCouncil, RoundTableEvent, RoundTableParticipant, RoundTableRole, Vote, VoteRecord,
        VotingResult, VotingStatus,
    },
    workflow::{CollaborationWorkflow, StepStatus, WorkflowStep},
};
use crate::error::Result;
use crate::mcp::{
    client::MCPClient,
    context::ContextManager,
    types::{ContextEntry, ContextId},
};
use crate::types::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Round table message types for A2A transport
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundTableMessage {
    /// New proposal announcement
    ProposalCreated { proposal: Proposal },
    /// Vote cast
    VoteCast { proposal_id: Uuid, vote: VoteRecord },
    /// Voting completed
    VotingCompleted { result: VotingResult },
    /// Conflict detected
    ConflictDetected { conflict: Conflict },
    /// Conflict resolved
    ConflictResolved { conflict_id: Uuid, resolution: ConflictResolution },
    /// Consensus reached
    ConsensusReached { topic: String, participants: Vec<Uuid> },
    /// Debate argument
    DebateArgument { debate_id: Uuid, participant_id: Uuid, argument: String },
    /// Shared fact update
    SharedFactUpdate { key: String, value: String },
}

/// Round table workflow step types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundTableStepType {
    /// Proposal phase
    Proposal,
    /// Debate/discussion phase
    Debate,
    /// Voting phase
    Voting,
    /// Conflict resolution phase
    ConflictResolution,
    /// Execution phase
    Execution,
}

/// Integrated round table system with A2A and Workflow
pub struct IntegratedRoundTable {
    /// Underlying round table council
    council: Arc<RwLock<RoundTableCouncil>>,
    /// A2A client for transport
    a2a_client: Arc<A2AClient>,
    /// MCP context manager
    context_manager: Arc<RwLock<ContextManager>>,
    /// Active workflows
    workflows: Arc<RwLock<HashMap<Uuid, RoundTableWorkflow>>>,
    /// Event sender
    event_sender: mpsc::Sender<RoundTableIntegrationEvent>,
    /// Agent ID mappings
    agent_mappings: Arc<RwLock<HashMap<Uuid, String>>>, // participant_id -> agent_id
}

/// Integration events
#[derive(Debug, Clone)]
pub enum RoundTableIntegrationEvent {
    /// Workflow started
    WorkflowStarted { workflow_id: Uuid, topic: String },
    /// Workflow step completed
    StepCompleted { workflow_id: Uuid, step_type: RoundTableStepType },
    /// Workflow completed
    WorkflowCompleted { workflow_id: Uuid, success: bool },
    /// A2A message received
    A2AMessageReceived { from: String, message: RoundTableMessage },
    /// MCP context updated
    ContextUpdated { key: String },
}

/// Round table workflow wrapper
#[derive(Debug)]
pub struct RoundTableWorkflow {
    /// Underlying workflow
    pub inner: CollaborationWorkflow,
    /// Topic being discussed
    pub topic: String,
    /// Current proposal
    pub current_proposal: Option<Proposal>,
    /// Step type mapping
    pub step_types: Vec<RoundTableStepType>,
    /// Debate arguments
    pub debate_arguments: HashMap<Uuid, Vec<DebateArgument>>,
}

/// Debate argument
#[derive(Debug, Clone)]
pub struct DebateArgument {
    pub participant_id: Uuid,
    pub argument: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl IntegratedRoundTable {
    /// Create new integrated round table
    pub fn new(
        a2a_client: Arc<A2AClient>,
        context_manager: Arc<RwLock<ContextManager>>,
    ) -> Result<(Self, mpsc::Receiver<RoundTableIntegrationEvent>)> {
        let (council, council_events) = RoundTableCouncil::default();
        let (event_sender, event_receiver) = mpsc::channel(100);

        let integrated = Self {
            council: Arc::new(RwLock::new(council)),
            a2a_client,
            context_manager,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            agent_mappings: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start event processing
        integrated.start_event_processor(council_events);

        Ok((integrated, event_receiver))
    }

    /// Register a participant with A2A agent mapping
    pub async fn register_participant(
        &self,
        participant: RoundTableParticipant,
        a2a_agent_id: String,
    ) -> Result<()> {
        // Register with council
        let council = self.council.read().await;
        council.join(participant.clone()).await?;

        // Store mapping
        let mut mappings = self.agent_mappings.write().await;
        mappings.insert(participant.id, a2a_agent_id);

        info!(
            "Registered participant {} with A2A agent {}",
            participant.id, a2a_agent_id
        );
        Ok(())
    }

    /// Create a proposal with workflow
    pub async fn create_proposal_workflow(
        &self,
        topic: String,
        description: String,
        proposer_id: Uuid,
    ) -> Result<Uuid> {
        let workflow_id = Uuid::new_v4();

        // Create proposal
        let proposal = Proposal {
            id: Uuid::new_v4(),
            title: topic.clone(),
            description,
            proposer_id,
            proposal_type: ProposalType::Strategy,
            content: String::new(),
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::minutes(30)),
        };

        // Create workflow with steps
        let mut workflow = CollaborationWorkflow::new();
        let step_types = vec![
            RoundTableStepType::Proposal,
            RoundTableStepType::Debate,
            RoundTableStepType::Voting,
            RoundTableStepType::Execution,
        ];

        for (i, step_type) in step_types.iter().enumerate() {
            workflow.add_step(WorkflowStep {
                id: Uuid::new_v4(),
                agent_id: proposer_id, // Initial agent, can be changed
                task: format!("{:?} phase for {}", step_type, topic),
                dependencies: if i == 0 {
                    vec![]
                } else {
                    vec![workflow.steps[i - 1].id]
                },
                status: StepStatus::Pending,
            });
        }

        // Store workflow
        let rt_workflow = RoundTableWorkflow {
            inner: workflow,
            topic: topic.clone(),
            current_proposal: Some(proposal.clone()),
            step_types,
            debate_arguments: HashMap::new(),
        };

        {
            let mut workflows = self.workflows.write().await;
            workflows.insert(workflow_id, rt_workflow);
        }

        // Submit proposal to council
        let council = self.council.read().await;
        council.propose(proposal.clone()).await?;

        // Broadcast via A2A
        self.broadcast_message(RoundTableMessage::ProposalCreated { proposal })
            .await?;

        // Store in MCP context
        self.store_in_context("current_proposal", &serde_json::to_string(&proposal)?)
            .await?;

        // Notify
        let _ = self
            .event_sender
            .send(RoundTableIntegrationEvent::WorkflowStarted {
                workflow_id,
                topic,
            })
            .await;

        info!("Created proposal workflow: {}", workflow_id);
        Ok(workflow_id)
    }

    /// Start debate phase
    pub async fn start_debate(&self, workflow_id: Uuid) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        let workflow = workflows
            .get_mut(&workflow_id)
            .ok_or_else(|| crate::error::AgentError::not_found("Workflow not found".to_string()))?;

        // Find debate step
        let debate_step_idx = workflow
            .step_types
            .iter()
            .position(|t| *t == RoundTableStepType::Debate)
            .ok_or_else(|| {
                crate::error::AgentError::configuration("No debate step in workflow".to_string())
            })?;

        // Mark previous steps as completed
        for i in 0..debate_step_idx {
            workflow.inner.steps[i].status = StepStatus::Completed;
        }

        // Set current step to running
        workflow.inner.steps[debate_step_idx].status = StepStatus::Running;

        let _ = self
            .event_sender
            .send(RoundTableIntegrationEvent::StepCompleted {
                workflow_id,
                step_type: RoundTableStepType::Debate,
            })
            .await;

        info!("Started debate phase for workflow: {}", workflow_id);
        Ok(())
    }

    /// Submit debate argument
    pub async fn submit_argument(
        &self,
        workflow_id: Uuid,
        participant_id: Uuid,
        argument: String,
    ) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        let workflow = workflows
            .get_mut(&workflow_id)
            .ok_or_else(|| crate::error::AgentError::not_found("Workflow not found".to_string()))?;

        let debate_arg = DebateArgument {
            participant_id,
            argument: argument.clone(),
            timestamp: chrono::Utc::now(),
        };

        workflow
            .debate_arguments
            .entry(participant_id)
            .or_default()
            .push(debate_arg);

        // Broadcast via A2A
        self.broadcast_message(RoundTableMessage::DebateArgument {
            debate_id: workflow_id,
            participant_id,
            argument,
        })
        .await?;

        info!("Received argument from {} in workflow {}", participant_id, workflow_id);
        Ok(())
    }

    /// Cast vote via A2A
    pub async fn cast_vote_a2a(
        &self,
        participant_id: Uuid,
        proposal_id: Uuid,
        vote: Vote,
        reason: Option<String>,
    ) -> Result<()> {
        // Cast vote in council
        let council = self.council.read().await;
        council.vote(participant_id, proposal_id, vote, reason.clone()).await?;

        // Create vote record
        let vote_record = VoteRecord {
            participant_id,
            vote,
            reason,
            timestamp: chrono::Utc::now(),
        };

        // Broadcast via A2A
        self.broadcast_message(RoundTableMessage::VoteCast {
            proposal_id,
            vote: vote_record,
        })
        .await?;

        info!("Vote cast by {} via A2A", participant_id);
        Ok(())
    }

    /// Close voting and process results
    pub async fn close_voting(&self, workflow_id: Uuid, proposal_id: Uuid) -> Result<VotingResult> {
        let council = self.council.read().await;
        let result = council.close_voting(proposal_id).await?;

        // Broadcast result
        self.broadcast_message(RoundTableMessage::VotingCompleted {
            result: result.clone(),
        })
        .await?;

        // Update workflow
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(&workflow_id) {
            // Find voting step and mark complete
            if let Some(idx) = workflow
                .step_types
                .iter()
                .position(|t| *t == RoundTableStepType::Voting)
            {
                workflow.inner.steps[idx].status = StepStatus::Completed;

                // Mark execution step as running if approved
                if result.status == VotingStatus::Approved {
                    if let Some(exec_idx) = workflow
                        .step_types
                        .iter()
                        .position(|t| *t == RoundTableStepType::Execution)
                    {
                        workflow.inner.steps[exec_idx].status = StepStatus::Running;
                    }
                }
            }
        }

        let _ = self
            .event_sender
            .send(RoundTableIntegrationEvent::StepCompleted {
                workflow_id,
                step_type: RoundTableStepType::Voting,
            })
            .await;

        // Store result in MCP context
        self.store_in_context(
            &format!("vote_result_{}", proposal_id),
            &serde_json::to_string(&result)?,
        )
        .await?;

        info!("Voting closed for proposal {}: {:?}", proposal_id, result.status);
        Ok(result)
    }

    /// Handle conflict resolution
    pub async fn resolve_conflict_workflow(
        &self,
        workflow_id: Uuid,
        conflict_id: Uuid,
        method: ResolutionMethod,
    ) -> Result<ConflictResolution> {
        let resolution = ConflictResolution {
            method,
            winning_proposal_id: None,
            explanation: "Resolved through workflow".to_string(),
            human_arbitrated: method == ResolutionMethod::HumanArbitration,
        };

        // Resolve in council
        let council = self.council.read().await;
        council.resolve_conflict(conflict_id, resolution.clone()).await?;

        // Broadcast
        self.broadcast_message(RoundTableMessage::ConflictResolved {
            conflict_id,
            resolution: resolution.clone(),
        })
        .await?;

        info!("Conflict {} resolved using {:?}", conflict_id, method);
        Ok(resolution)
    }

    /// Broadcast message to all participants via A2A
    async fn broadcast_message(&self, message: RoundTableMessage) -> Result<()> {
        let council = self.council.read().await;
        let participants = council.get_participants().await;
        let mappings = self.agent_mappings.read().await;

        for participant in participants {
            if let Some(agent_id) = mappings.get(&participant.id) {
                let a2a_message = A2AMessage::new(
                    MessageType::Event,
                    AgentId::from_string("round_table"),
                    Some(AgentId::from_string(agent_id)),
                    MessagePayload::Event {
                        event_type: "round_table".to_string(),
                        data: serde_json::to_value(&message).unwrap_or_default(),
                    },
                );

                match self.a2a_client.send_message(a2a_message, agent_id).await {
                    Ok(_) => debug!("Sent A2A message to {}", agent_id),
                    Err(e) => warn!("Failed to send A2A message to {}: {}", agent_id, e),
                }
            }
        }

        Ok(())
    }

    /// Store data in MCP context
    async fn store_in_context(&self, key: &str, value: &str) -> Result<()> {
        let context_manager = self.context_manager.read().await;
        // Note: Actual MCP context storage would depend on MCP implementation
        debug!("Storing in MCP context: {} = {}", key, value);
        Ok(())
    }

    /// Start event processor
    fn start_event_processor(&self, mut council_events: mpsc::Receiver<RoundTableEvent>) {
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            while let Some(event) = council_events.recv().await {
                match event {
                    RoundTableEvent::ConsensusReached { topic, participants } => {
                        let _ = event_sender
                            .send(RoundTableIntegrationEvent::WorkflowCompleted {
                                workflow_id: Uuid::new_v4(), // Would need to track properly
                                success: true,
                            })
                            .await;
                        info!("Consensus reached on: {}", topic);
                    }
                    _ => {}
                }
            }
        });
    }

    /// Get workflow status
    pub async fn get_workflow_status(&self, workflow_id: Uuid) -> Option<RoundTableWorkflowStatus> {
        let workflows = self.workflows.read().await;
        workflows.get(&workflow_id).map(|w| RoundTableWorkflowStatus {
            workflow_id,
            topic: w.topic.clone(),
            current_step: w.inner.current_step,
            total_steps: w.inner.steps.len(),
            is_complete: w.inner.is_complete(),
            debate_argument_count: w.debate_arguments.values().map(|v| v.len()).sum(),
        })
    }

    /// List active workflows
    pub async fn list_active_workflows(&self) -> Vec<RoundTableWorkflowStatus> {
        let workflows = self.workflows.read().await;
        workflows
            .iter()
            .filter(|(_, w)| !w.inner.is_complete())
            .map(|(id, w)| RoundTableWorkflowStatus {
                workflow_id: *id,
                topic: w.topic.clone(),
                current_step: w.inner.current_step,
                total_steps: w.inner.steps.len(),
                is_complete: false,
                debate_argument_count: w.debate_arguments.values().map(|v| v.len()).sum(),
            })
            .collect()
    }
}

/// Workflow status summary
#[derive(Debug, Clone)]
pub struct RoundTableWorkflowStatus {
    pub workflow_id: Uuid,
    pub topic: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub is_complete: bool,
    pub debate_argument_count: usize,
}

/// Structured debate session
pub struct DebateSession {
    pub topic: String,
    pub proposition: String,
    pub opposition: String,
    pub rounds: Vec<DebateRound>,
    pub current_round: usize,
}

/// Debate round
pub struct DebateRound {
    pub round_number: usize,
    pub proposition_argument: Option<String>,
    pub opposition_argument: Option<String>,
    pub cross_examination: Vec<(Uuid, String)>,
}

impl DebateSession {
    /// Create new debate session
    pub fn new(topic: String, proposition: String, opposition: String) -> Self {
        Self {
            topic,
            proposition,
            opposition,
            rounds: vec![DebateRound {
                round_number: 1,
                proposition_argument: None,
                opposition_argument: None,
                cross_examination: vec![],
            }],
            current_round: 0,
        }
    }

    /// Submit argument for a side
    pub fn submit_argument(&mut self, side: DebateSide, argument: String) {
        let round = &mut self.rounds[self.current_round];
        match side {
            DebateSide::Proposition => round.proposition_argument = Some(argument),
            DebateSide::Opposition => round.opposition_argument = Some(argument),
        }
    }

    /// Add cross-examination question/answer
    pub fn add_cross_examination(&mut self, participant_id: Uuid, content: String) {
        let round = &mut self.rounds[self.current_round];
        round.cross_examination.push((participant_id, content));
    }

    /// Advance to next round
    pub fn next_round(&mut self) {
        self.current_round += 1;
        self.rounds.push(DebateRound {
            round_number: self.current_round + 1,
            proposition_argument: None,
            opposition_argument: None,
            cross_examination: vec![],
        });
    }
}

/// Debate side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebateSide {
    Proposition,
    Opposition,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would need actual A2A and MCP clients to run properly
    // For unit testing, we'd use mock implementations

    #[test]
    fn test_debate_session() {
        let mut session = DebateSession::new(
            "AI Safety".to_string(),
            "AI should be regulated".to_string(),
            "AI should not be regulated".to_string(),
        );

        session.submit_argument(DebateSide::Proposition, "Regulation ensures safety".to_string());
        session.submit_argument(DebateSide::Opposition, "Regulation stifles innovation".to_string());

        assert!(session.rounds[0].proposition_argument.is_some());
        assert!(session.rounds[0].opposition_argument.is_some());
    }
}
