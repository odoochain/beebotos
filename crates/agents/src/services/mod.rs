//! Service layer for user-channel and agent-channel lifecycle management.

pub mod agent_channel_service;
pub mod agent_channel_store;
pub mod agent_channel_store_sqlite;
pub mod credential_crypto;
pub mod user_channel_service;
pub mod user_channel_store;
pub mod user_channel_store_sqlite;

pub use agent_channel_service::AgentChannelService;
pub use agent_channel_store::AgentChannelBindingStore;
pub use agent_channel_store_sqlite::SqliteAgentChannelBindingStore;
pub use credential_crypto::{plaintext_encryptor, ChannelConfigEncryptor};
pub use user_channel_service::UserChannelService;
pub use user_channel_store::UserChannelStore;
pub use user_channel_store_sqlite::SqliteUserChannelStore;
