use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Negotiation {
    pub id: String,
    pub initiator: String,
    pub responder: String,
    pub status: NegotiationStatus,
    pub proposals: Vec<Proposal>,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NegotiationStatus {
    Pending,
    InProgress,
    Accepted,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub proposer: String,
    pub terms: Terms,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terms {
    pub service_type: String,
    pub price: Price,
    pub duration: Duration,
    pub quality_of_service: QoSLevel,
    pub penalties: Vec<Penalty>,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
    pub payment_schedule: PaymentSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentSchedule {
    Upfront,
    Milestone(Vec<MilestonePayment>),
    OnCompletion,
    Recurring { interval: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestonePayment {
    pub description: String,
    pub percentage: f32,
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    pub value: u32,
    pub unit: TimeUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeUnit {
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QoSLevel {
    Basic,
    Standard,
    Premium,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Penalty {
    pub condition: String,
    pub amount: f64,
    pub max_penalty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    NonDisclosure,
    Exclusivity,
    MinimumCommitment,
    PerformanceGuarantee,
    Custom(String),
}

pub struct NegotiationEngine {
    negotiations: HashMap<String, Negotiation>,
}

impl NegotiationEngine {
    pub fn new() -> Self {
        Self {
            negotiations: HashMap::new(),
        }
    }

    pub fn create_negotiation(
        &mut self,
        initiator: String,
        responder: String,
        initial_proposal: Proposal,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp() as u64;
        let negotiation = Negotiation {
            id: id.clone(),
            initiator,
            responder,
            status: NegotiationStatus::Pending,
            proposals: vec![initial_proposal],
            created_at: now,
            expires_at: now + 86400,
        };
        self.negotiations.insert(id.clone(), negotiation);
        id
    }

    pub fn counter_proposal(
        &mut self,
        negotiation_id: &str,
        proposal: Proposal,
    ) -> Result<(), String> {
        if let Some(neg) = self.negotiations.get_mut(negotiation_id) {
            neg.proposals.push(proposal);
            neg.status = NegotiationStatus::InProgress;
            Ok(())
        } else {
            Err("Negotiation not found".to_string())
        }
    }

    pub fn accept(&mut self, negotiation_id: &str) -> Result<Terms, String> {
        if let Some(neg) = self.negotiations.get_mut(negotiation_id) {
            neg.status = NegotiationStatus::Accepted;
            neg.proposals
                .last()
                .map(|p| p.terms.clone())
                .ok_or("No proposals".to_string())
        } else {
            Err("Negotiation not found".to_string())
        }
    }

    pub fn evaluate_proposal(&self, proposal: &Proposal) -> f64 {
        let mut score = 100.0;

        match proposal.terms.quality_of_service {
            QoSLevel::Basic => score += 10.0,
            QoSLevel::Standard => score += 20.0,
            QoSLevel::Premium => score += 30.0,
            QoSLevel::Enterprise => score += 40.0,
        }

        let price_factor = 1000.0 / (proposal.terms.price.amount + 1.0);
        score += price_factor.min(50.0);

        score
    }
}
