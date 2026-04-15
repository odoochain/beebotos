//! Gmail Pub/Sub Scheduling

use crate::error::Result;

/// Gmail Pub/Sub scheduler
pub struct GmailPubSubScheduler {
    project_id: String,
    subscription: String,
}

impl GmailPubSubScheduler {
    pub fn new(project_id: impl Into<String>, subscription: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            subscription: subscription.into(),
        }
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting Gmail Pub/Sub scheduler");
        // TODO: Implement Gmail Pub/Sub integration
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping Gmail Pub/Sub scheduler");
        Ok(())
    }
}
