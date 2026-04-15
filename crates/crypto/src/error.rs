//! Crypto Errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Signature verification failed")]
    SignatureVerification,
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
}

pub type CryptoResult<T> = Result<T, CryptoError>;
