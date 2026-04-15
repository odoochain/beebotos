//! HTTP Client
//!
//! API client for BeeBotOS Gateway.

use reqwest::{Client, Method, Response};
use serde::{de::DeserializeOwned, Serialize};

use crate::{Result, SdkError};

/// HTTP API client
pub struct HttpClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl HttpClient {
    /// Create new client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            api_key: None,
        }
    }

    /// With API key
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// With custom client
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(Method::GET, path, None::<&()>).await
    }

    /// POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        self.request(Method::POST, path, Some(body)).await
    }

    /// PUT request
    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        self.request(Method::PUT, path, Some(body)).await
    }

    /// DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(Method::DELETE, path, None::<&()>).await
    }

    /// Raw request
    async fn request<T: DeserializeOwned, B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        
        let mut req = self.client.request(method, &url);
        
        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        
        if let Some(body) = body {
            req = req.json(body);
        }
        
        let resp = req.send().await?;
        
        Self::handle_response(resp).await
    }

    async fn handle_response<T: DeserializeOwned>(resp: Response) -> Result<T> {
        let status = resp.status();
        
        if status.is_success() {
            let data = resp.json::<T>().await?;
            Ok(data)
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(SdkError::Api {
                code: status.as_u16(),
                message: text,
            })
        }
    }
}

/// Paginated request builder
pub struct PaginatedRequest<'a> {
    client: &'a HttpClient,
    path: String,
    page: u32,
    per_page: u32,
}

impl<'a> PaginatedRequest<'a> {
    pub fn new(client: &'a HttpClient, path: impl Into<String>) -> Self {
        Self {
            client,
            path: path.into(),
            page: 1,
            per_page: 20,
        }
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = per_page;
        self
    }

    pub async fn fetch<T: DeserializeOwned>(self) -> Result<Vec<T>> {
        let path = format!("{}?page={}&per_page={}", self.path, self.page, self.per_page);
        self.client.get(&path).await
    }
}

/// WebSocket client for real-time updates
pub struct WebSocketClient {
    url: String,
}

impl WebSocketClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }

    /// Connect and handle messages
    pub async fn connect<F>(&self, _handler: F) -> Result<()>
    where
        F: Fn(crate::Message),
    {
        // In production, use tokio-tungstenite
        tracing::info!("Connecting to WebSocket: {}", self.url);
        Ok(())
    }
}
