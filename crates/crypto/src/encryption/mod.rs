pub mod aes;
pub mod chacha;
pub mod envelope;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
    XSalsa20Poly1305,
}

pub trait EncryptionScheme: Send + Sync {
    fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&[u8]>,
    ) -> Result<EncryptedData, EncryptionError>;
    fn decrypt(
        &self,
        data: &EncryptedData,
        associated_data: Option<&[u8]>,
    ) -> Result<Vec<u8>, EncryptionError>;
    fn algorithm(&self) -> EncryptionAlgorithm;
}

#[derive(Debug, Clone)]
pub enum EncryptionError {
    InvalidKey,
    InvalidNonce,
    EncryptionFailed,
    DecryptionFailed,
    AuthenticationFailed,
    UnsupportedAlgorithm,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::InvalidKey => write!(f, "Invalid encryption key"),
            EncryptionError::InvalidNonce => write!(f, "Invalid nonce"),
            EncryptionError::EncryptionFailed => write!(f, "Encryption failed"),
            EncryptionError::DecryptionFailed => write!(f, "Decryption failed"),
            EncryptionError::AuthenticationFailed => write!(f, "Authentication failed"),
            EncryptionError::UnsupportedAlgorithm => write!(f, "Unsupported algorithm"),
        }
    }
}

impl std::error::Error for EncryptionError {}

pub struct KeyDerivation {
    iterations: u32,
    salt: Vec<u8>,
}

impl KeyDerivation {
    pub fn new(iterations: u32) -> Self {
        Self {
            iterations,
            salt: Self::generate_salt(),
        }
    }

    fn generate_salt() -> Vec<u8> {
        use rand::RngCore;
        let mut salt = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }

    pub fn derive_key(&self, password: &[u8]) -> Vec<u8> {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        let mut key = vec![0u8; 32];
        pbkdf2_hmac::<Sha256>(password, &self.salt, self.iterations, &mut key);
        key
    }
}

pub struct SecureVault {
    master_key: Vec<u8>,
    data: std::collections::HashMap<String, EncryptedData>,
}

impl SecureVault {
    pub fn new(master_key: Vec<u8>) -> Self {
        Self {
            master_key,
            data: std::collections::HashMap::new(),
        }
    }

    pub fn store(
        &mut self,
        key: String,
        plaintext: &[u8],
        scheme: &dyn EncryptionScheme,
    ) -> Result<(), EncryptionError> {
        let encrypted = scheme.encrypt(plaintext, Some(&self.master_key))?;
        self.data.insert(key, encrypted);
        Ok(())
    }

    pub fn retrieve(
        &self,
        key: &str,
        scheme: &dyn EncryptionScheme,
    ) -> Result<Vec<u8>, EncryptionError> {
        let encrypted = self
            .data
            .get(key)
            .ok_or(EncryptionError::DecryptionFailed)?;
        scheme.decrypt(encrypted, Some(&self.master_key))
    }
}
