//! A2A Commerce
//!
//! Agent-to-agent commerce and payments.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Commerce deal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: Uuid,
    pub buyer: Uuid,
    pub seller: Uuid,
    pub service: String,
    pub price: u64,
    pub status: DealStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DealStatus {
    Proposed,
    Accepted,
    InProgress,
    Completed,
    Disputed,
    Cancelled,
}

/// Commerce manager
pub struct CommerceManager {
    deals: Vec<Deal>,
}

impl CommerceManager {
    pub fn new() -> Self {
        Self {
            deals: Vec::new(),
        }
    }

    pub fn create_deal(&mut self, buyer: Uuid, seller: Uuid, service: impl Into<String>, price: u64) -> Uuid {
        let deal = Deal {
            id: Uuid::new_v4(),
            buyer,
            seller,
            service: service.into(),
            price,
            status: DealStatus::Proposed,
        };
        
        let id = deal.id;
        self.deals.push(deal);
        id
    }

    pub fn get_deal(&self, id: Uuid) -> Option<&Deal> {
        self.deals.iter().find(|d| d.id == id)
    }

    pub fn accept_deal(&mut self, id: Uuid) -> Result<()> {
        if let Some(deal) = self.deals.iter_mut().find(|d| d.id == id) {
            deal.status = DealStatus::Accepted;
            Ok(())
        } else {
            Err(crate::error::AgentError::not_found(format!("Deal {}", id)))
        }
    }

    pub fn complete_deal(&mut self, id: Uuid) -> Result<()> {
        if let Some(deal) = self.deals.iter_mut().find(|d| d.id == id) {
            deal.status = DealStatus::Completed;
            Ok(())
        } else {
            Err(crate::error::AgentError::not_found(format!("Deal {}", id)))
        }
    }
}

impl Default for CommerceManager {
    fn default() -> Self {
        Self::new()
    }
}
