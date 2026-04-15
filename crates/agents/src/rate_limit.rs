//! Rate Limiting Module
//!
//! 🟡 MEDIUM FIX: Token bucket rate limiter for agent operations

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Token bucket rate limiter
pub struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();

        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        self.last_refill = now;
    }
}

/// Rate limiter for multiple keys
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, Arc<Mutex<TokenBucket>>>>,
    default_capacity: u32,
    default_refill_rate: f64,
}

impl RateLimiter {
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            default_capacity: capacity,
            default_refill_rate: refill_rate,
        }
    }

    /// Check if operation is allowed
    pub fn check(&self, key: &str) -> bool {
        let Ok(mut buckets) = self.buckets.lock() else {
            tracing::error!("Rate limiter buckets lock poisoned");
            return true; // Allow on error to avoid blocking
        };

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| {
            Arc::new(Mutex::new(TokenBucket::new(
                self.default_capacity,
                self.default_refill_rate,
            )))
        });

        let Ok(mut bucket) = bucket.lock() else {
            tracing::error!("Rate limiter bucket lock poisoned for key: {}", key);
            return true; // Allow on error to avoid blocking
        };
        bucket.try_consume(1)
    }

    /// Check with custom cost
    pub fn check_with_cost(&self, key: &str, cost: u32) -> bool {
        let Ok(mut buckets) = self.buckets.lock() else {
            tracing::error!("Rate limiter buckets lock poisoned");
            return true; // Allow on error to avoid blocking
        };

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| {
            Arc::new(Mutex::new(TokenBucket::new(
                self.default_capacity,
                self.default_refill_rate,
            )))
        });

        let Ok(mut bucket) = bucket.lock() else {
            tracing::error!("Rate limiter bucket lock poisoned for key: {}", key);
            return true; // Allow on error to avoid blocking
        };
        bucket.try_consume(cost)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Default: 100 requests per second
        Self::new(100, 100.0)
    }
}
