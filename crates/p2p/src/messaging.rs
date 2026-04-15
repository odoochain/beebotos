//! Messaging service stub

use crate::{P2PMessage, Result};
use libp2p::PeerId;

/// Message handler trait
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, msg: P2PMessage) -> Result<()>;
}

/// Simple message handler
pub struct DefaultMessageHandler;

#[async_trait::async_trait]
impl MessageHandler for DefaultMessageHandler {
    async fn handle_message(&self, _msg: P2PMessage) -> Result<()> {
        Ok(())
    }
}
