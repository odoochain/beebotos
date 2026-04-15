//! BeeBotOS Peer-to-Peer Networking (Stub)

use libp2p::PeerId;

/// P2P result type
pub type Result<T> = anyhow::Result<T>;

/// P2P message
#[derive(Debug, Clone)]
pub struct P2PMessage {
    pub from: PeerId,
    pub to: Option<PeerId>,
    pub payload: Vec<u8>,
}

/// P2P network node
pub struct P2PNode {
    local_peer_id: PeerId,
}

impl P2PNode {
    /// Create new P2P node
    pub fn new() -> Result<Self> {
        let local_peer_id = PeerId::random();
        Ok(Self { local_peer_id })
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    /// Start the P2P network
    pub async fn start(&self) -> Result<()> {
        Ok(())
    }

    /// Stop the P2P network
    pub async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for P2PNode {
    fn default() -> Self {
        Self::new().expect("Failed to create P2P node")
    }
}
