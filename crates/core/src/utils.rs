//! Core Utilities

/// Generate a unique ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Get current timestamp
pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

/// Retry a function
pub async fn retry<F, Fut, T, E>(f: F, max_retries: u32) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(t) => return Ok(t),
            Err(e) if retries < max_retries => {
                retries += 1;
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
