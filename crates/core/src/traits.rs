//! Core Traits

/// Identifiable trait
pub trait Identifiable {
    fn id(&self) -> &str;
}

/// Configurable trait
pub trait Configurable {
    type Config;
    fn configure(&mut self, config: Self::Config);
}

/// Lifecycle trait
#[async_trait::async_trait]
pub trait Lifecycle {
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
