//! Collaboration Routing

use super::CollabMessage;
use crate::error::Result;
use uuid::Uuid;

/// Routes collaboration messages between agents
pub struct CollaborationRouter {
    routes: Vec<Route>,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub from: Uuid,
    pub to: Uuid,
    pub priority: u8,
}

impl CollaborationRouter {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
        }
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn find_route(&self, from: Uuid, to: Uuid) -> Option<&Route> {
        self.routes
            .iter()
            .find(|r| r.from == from && r.to == to)
    }

    pub async fn route(&self, message: CollabMessage) -> Result<()> {
        if let Some(to) = message.to {
            tracing::info!("Routing message from {} to {}", message.from, to);
        }
        Ok(())
    }
}

impl Default for CollaborationRouter {
    fn default() -> Self {
        Self::new()
    }
}
