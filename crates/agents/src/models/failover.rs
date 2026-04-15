//! Model Failover

use super::CompletionResponse;
use crate::error::Result;

/// Failover manager for model providers
pub struct FailoverManager {
    providers: Vec<String>,
    current_index: usize,
}

impl FailoverManager {
    pub fn new(providers: Vec<String>) -> Self {
        Self {
            providers,
            current_index: 0,
        }
    }

    pub async fn execute<F>(&mut self, f: F) -> Result<CompletionResponse>
    where
        F: Fn(&str) -> Result<CompletionResponse>,
    {
        let start_index = self.current_index;

        loop {
            let provider = &self.providers[self.current_index];

            match f(provider) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    tracing::warn!("Provider {} failed: {}", provider, e);
                    self.current_index = (self.current_index + 1) % self.providers.len();

                    if self.current_index == start_index {
                        return Err(crate::error::AgentError::Execution(
                            "All providers failed".to_string(),
                        ));
                    }
                }
            }
        }
    }

    pub fn current_provider(&self) -> &str {
        &self.providers[self.current_index]
    }
}
