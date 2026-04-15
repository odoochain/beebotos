//! Bridge adapters for different chains

pub mod cosmos;
pub mod ethereum;
pub mod polkadot;
pub mod solana;

pub use cosmos::CosmosBridgeAdapter;
pub use ethereum::EthereumBridgeAdapter;
pub use polkadot::PolkadotBridgeAdapter;
pub use solana::SolanaBridgeAdapter;
