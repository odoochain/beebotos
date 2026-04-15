//! Transport stub

use libp2p::{PeerId, Transport};
use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport::Boxed;

/// P2P transport type
pub type P2PTransport = Boxed<(PeerId, StreamMuxerBox)>;

/// Create transport
#[allow(dead_code)]
pub fn create_transport() -> anyhow::Result<P2PTransport> {
    unimplemented!("Transport creation not implemented in stub")
}
