//! Network behaviour stub

use libp2p::PeerId;
use crate::Result;

/// Agent network behaviour
pub struct AgentBehaviour;

/// Behaviour event
#[derive(Debug, Clone)]
pub enum AgentBehaviourEvent {
    MessageReceived { from: PeerId, data: Vec<u8> },
}

/// P2P configuration
pub struct P2PConfig {
    pub listen_addrs: Vec<String>,
    pub bootstrap_peers: Vec<String>,
    pub enable_mdns: bool,
    pub enable_kademlia: bool,
    pub capabilities: Vec<String>,
}

impl AgentBehaviour {
    /// Create new behaviour
    pub fn new(_config: &P2PConfig, _local_peer_id: PeerId) -> Result<Self> {
        Ok(Self)
    }
}
