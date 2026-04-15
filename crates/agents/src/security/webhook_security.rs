//! Webhook Security Module
//!
//! Provides secure webhook signature verification and replay attack prevention.

use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{debug, error, warn};

use crate::error::{AgentError, Result};

type HmacSha256 = Hmac<Sha256>;

/// Webhook signature verifier
pub struct WebhookSignatureVerifier {
    secret: String,
    timestamp_tolerance: u64,
    validate_timestamp: bool,
}

impl WebhookSignatureVerifier {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            timestamp_tolerance: 300,
            validate_timestamp: true,
        }
    }

    pub fn with_timestamp_tolerance(mut self, tolerance_secs: u64) -> Self {
        self.timestamp_tolerance = tolerance_secs;
        self
    }

    pub fn without_timestamp_validation(mut self) -> Self {
        self.validate_timestamp = false;
        self
    }

    /// Verify webhook signature
    pub fn verify(&self, body: &[u8], signature: &str, timestamp: &str) -> Result<()> {
        if self.validate_timestamp {
            self.validate_timestamp(timestamp)?;
        }

        let expected = self.compute_signature(body, timestamp)?;

        if !constant_time_eq(signature.as_bytes(), expected.as_bytes()) {
            error!("Webhook signature mismatch");
            return Err(AgentError::authentication("Invalid signature"));
        }

        debug!("Webhook signature verified successfully");
        Ok(())
    }

    /// Compute HMAC-SHA256 signature
    fn compute_signature(&self, body: &[u8], timestamp: &str) -> Result<String> {
        let base_string = format!("{}.{}", timestamp, std::str::from_utf8(body).unwrap_or(""));

        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .map_err(|e| AgentError::Internal(format!("Failed to create HMAC: {}", e)))?;

        mac.update(base_string.as_bytes());

        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }

    /// Verify Slack-style signature
    pub fn verify_slack(&self, body: &[u8], signature: &str, timestamp: &str) -> Result<()> {
        if self.validate_timestamp {
            self.validate_timestamp(timestamp)?;
        }

        let expected = self.compute_slack_signature(body, timestamp)?;

        if !constant_time_eq(signature.as_bytes(), expected.as_bytes()) {
            error!("Slack webhook signature mismatch");
            return Err(AgentError::authentication("Invalid signature"));
        }

        Ok(())
    }

    /// Compute Slack-style signature
    fn compute_slack_signature(&self, body: &[u8], timestamp: &str) -> Result<String> {
        let base_string = format!("v0:{}:", timestamp);

        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .map_err(|e| AgentError::Internal(format!("Failed to create HMAC: {}", e)))?;

        mac.update(base_string.as_bytes());
        mac.update(body);

        let result = mac.finalize();
        Ok(format!("v0={}", hex::encode(result.into_bytes())))
    }

    /// Validate timestamp
    fn validate_timestamp(&self, timestamp: &str) -> Result<()> {
        let ts = timestamp
            .parse::<u64>()
            .map_err(|_| AgentError::authentication("Invalid timestamp format"))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AgentError::Internal("System time error".to_string()))?
            .as_secs();

        let diff = if now > ts { now - ts } else { ts - now };

        if diff > self.timestamp_tolerance {
            error!("Webhook timestamp out of tolerance: {}s", diff);
            return Err(AgentError::authentication("Timestamp out of tolerance"));
        }

        Ok(())
    }
}

/// Constant-time comparison
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

/// Replay attack prevention
pub struct ReplayProtection {
    window_size: u64,
    seen_nonces: std::collections::HashSet<String>,
    window_start: u64,
}

impl ReplayProtection {
    pub fn new(window_size_secs: u64) -> Self {
        Self {
            window_size: window_size_secs,
            seen_nonces: std::collections::HashSet::new(),
            window_start: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn check_and_record(&mut self, nonce: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now - self.window_start > self.window_size {
            self.seen_nonces.clear();
            self.window_start = now;
        }

        if self.seen_nonces.contains(nonce) {
            warn!("Replay attack detected: {}", nonce);
            return false;
        }

        self.seen_nonces.insert(nonce.to_string());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
    }

    #[test]
    fn test_replay_protection() {
        let mut protection = ReplayProtection::new(60);
        assert!(protection.check_and_record("nonce1"));
        assert!(!protection.check_and_record("nonce1"));
    }
}
