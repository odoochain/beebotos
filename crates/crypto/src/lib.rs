//! BeeBotOS Crypto Layer

pub mod encryption;
pub mod hashing;
pub mod signatures;

pub type Result<T> = std::result::Result<T, CryptoError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Signature error: {0}")]
    Signature(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
}
