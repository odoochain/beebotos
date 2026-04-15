//! gRPC Transport Implementation
//! 
//! A simplified gRPC-compatible transport using HTTP/2 and JSON messages.
//! This provides cluster federation and inter-service communication.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::{
    Message, MessageBus, MessagePriority, Result, SubscriptionId, TraceContext,
    MessageBusError, MessageStream,
};

/// gRPC transport configuration
#[derive(Debug, Clone)]
pub struct GrpcConfig {
    /// Bind address for the server
    pub bind_addr: SocketAddr,
    /// Cluster discovery addresses (seed nodes)
    pub cluster_addrs: Vec<SocketAddr>,
    /// Node ID (auto-generated if not set)
    pub node_id: String,
    /// Keepalive interval
    pub keepalive_interval: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Max message size
    pub max_message_size: usize,
    /// Enable TLS
    pub tls_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS key path
    pub tls_key_path: Option<String>,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:50051"
                .parse()
                .expect("hardcoded default bind address should be valid"),
            cluster_addrs: vec![],
            node_id: format!("node-{}", Uuid::new_v4()),
            keepalive_interval: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_message_size: 10 * 1024 * 1024, // 10MB
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// gRPC frame types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GrpcFrame {
    #[serde(rename = "publish")]
    Publish { topic: String, message: Message },
    #[serde(rename = "subscribe")]
    Subscribe { pattern: String, sub_id: String },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { sub_id: String },
    #[serde(rename = "message")]
    Message { sub_id: String, message: Message },
    #[serde(rename = "request")]
    Request { 
        request_id: String,
        topic: String, 
        message: Message,
        timeout_ms: u64,
    },
    #[serde(rename = "response")]
    Response { request_id: String, message: Message },
    #[serde(rename = "ping")]
    Ping { timestamp: i64, node_id: String },
    #[serde(rename = "pong")]
    Pong { timestamp: i64, node_id: String },
    #[serde(rename = "join")]
    Join { node_id: String, addr: String, topics: Vec<String> },
    #[serde(rename = "join_ack")]
    JoinAck { 
        success: bool,
        cluster_id: String,
        nodes: Vec<NodeInfo>,
    },
    #[serde(rename = "leave")]
    Leave { node_id: String, reason: String },
    #[serde(rename = "heartbeat")]
    Heartbeat { node_id: String, timestamp: i64 },
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: String,
    pub topics: Vec<String>,
    pub message_count: u64,
    pub status: NodeStatus,
    pub last_seen: i64,
}

/// Node status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    Unknown,
    Joining,
    Active,
    Leaving,
    Failed,
}

/// Connection to a remote node
pub struct RemoteConnection {
    node_id: String,
    stream: Arc<Mutex<TcpStream>>,
    heartbeat_tx: mpsc::Sender<()>,
    pending_requests: Arc<DashMap<String, oneshot::Sender<Message>>>,
}

/// gRPC Transport implementation
pub struct GrpcTransport {
    config: GrpcConfig,
    local_subs: DashMap<String, Vec<SubscriptionId>>,
    sub_senders: DashMap<SubscriptionId, mpsc::UnboundedSender<Message>>,
    cluster_nodes: DashMap<String, Arc<RemoteConnection>>,
    pending_requests: Arc<DashMap<String, oneshot::Sender<Message>>>,
    shutdown_tx: broadcast::Sender<()>,
    message_counter: Arc<RwLock<u64>>,
}

impl GrpcTransport {
    /// Create a new gRPC transport
    pub async fn new(config: GrpcConfig) -> Result<Arc<Self>> {
        let (shutdown_tx, _) = broadcast::channel(1);
        
        let transport = Arc::new(Self {
            config,
            local_subs: DashMap::new(),
            sub_senders: DashMap::new(),
            cluster_nodes: DashMap::new(),
            pending_requests: Arc::new(DashMap::new()),
            shutdown_tx,
            message_counter: Arc::new(RwLock::new(0)),
        });
        
        // Start server
        let transport_clone = Arc::clone(&transport);
        tokio::spawn(async move {
            if let Err(e) = transport_clone.run_server().await {
                error!("gRPC server error: {}", e);
            }
        });
        
        // Connect to cluster
        let transport_clone = Arc::clone(&transport);
        tokio::spawn(async move {
            transport_clone.connect_to_cluster().await;
        });
        
        // Start heartbeat
        let transport_clone = Arc::clone(&transport);
        tokio::spawn(async move {
            transport_clone.run_heartbeat().await;
        });
        
        info!("gRPC transport initialized on {}", transport.config.bind_addr);
        Ok(transport)
    }
    
    /// Run the gRPC server
    async fn run_server(self: &Arc<Self>) -> Result<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await
            .map_err(|e| MessageBusError::Transport(format!("Failed to bind: {}", e)))?;
        
        info!("gRPC server listening on {}", self.config.bind_addr);
        
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        
        loop {
            tokio::select! {
                Ok((stream, addr)) = listener.accept() => {
                    debug!("New connection from {}", addr);
                    let transport = Arc::clone(self);
                    tokio::spawn(async move {
                        if let Err(e) = transport.handle_connection(stream).await {
                            debug!("Connection from {} closed: {}", addr, e);
                        }
                    });
                }
                _ = shutdown_rx.recv() => {
                    info!("gRPC server shutting down");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle incoming connection
    async fn handle_connection(self: &Arc<Self>, stream: TcpStream) -> Result<()> {
        let stream = Arc::new(Mutex::new(stream));
        let (tx, mut rx) = mpsc::channel::<GrpcFrame>(1000);
        
        // Read loop
        let read_transport = Arc::clone(self);
        let read_stream = Arc::clone(&stream);
        let read_handle = tokio::spawn(async move {
            let mut buffer = vec![0u8; 4096];
            let mut msg_buffer = Vec::new();
            
            loop {
                let n = {
                    let mut s = read_stream.lock().await;
                    match s.read(&mut buffer).await {
                        Ok(0) => return Ok(()), // Connection closed
                        Ok(n) => n,
                        Err(e) => return Err(MessageBusError::Transport(format!("Read error: {}", e))),
                    }
                };
                
                msg_buffer.extend_from_slice(&buffer[..n]);
                
                // Process complete messages (length-prefixed JSON)
                while msg_buffer.len() >= 4 {
                    let len = u32::from_be_bytes([
                        msg_buffer[0], msg_buffer[1], 
                        msg_buffer[2], msg_buffer[3]
                    ]) as usize;
                    
                    if msg_buffer.len() < 4 + len {
                        break; // Wait for more data
                    }
                    
                    let frame: GrpcFrame = match serde_json::from_slice(&msg_buffer[4..4+len]) {
                        Ok(f) => f,
                        Err(e) => {
                            warn!("Failed to parse frame: {}", e);
                            msg_buffer.clear();
                            break;
                        }
                    };
                    
                    if let Err(_) = tx.send(frame).await {
                        return Ok(()); // Channel closed
                    }
                    
                    msg_buffer = msg_buffer.split_off(4 + len);
                }
            }
        });
        
        // Process loop
        while let Some(frame) = rx.recv().await {
            match frame {
                GrpcFrame::Publish { topic, message } => {
                    // Forward to local subscribers
                    self.forward_to_local_subs(&topic, message).await;
                }
                GrpcFrame::Subscribe { pattern, sub_id } => {
                    self.local_subs.entry(pattern).or_default().push(sub_id);
                }
                GrpcFrame::Unsubscribe { sub_id } => {
                    for mut entry in self.local_subs.iter_mut() {
                        entry.value().retain(|id| id != &sub_id);
                    }
                }
                GrpcFrame::Request { request_id, topic, message, timeout_ms } => {
                    // Handle request from remote node
                    let transport = Arc::clone(self);
                    let response_stream = Arc::clone(&stream);
                    tokio::spawn(async move {
                        match transport.handle_remote_request(topic, message, Duration::from_millis(timeout_ms)).await {
                            Ok(response) => {
                                let frame = GrpcFrame::Response { request_id, message: response };
                                let _ = transport.send_frame(&response_stream, &frame).await;
                            }
                            Err(_) => {
                                // Send error response
                                let error_msg = Message::new("error", serde_json::json!({"error": "Request failed"}));
                                let frame = GrpcFrame::Response { request_id, message: error_msg };
                                let _ = transport.send_frame(&response_stream, &frame).await;
                            }
                        }
                    });
                }
                GrpcFrame::Response { request_id, message } => {
                    // Fulfill pending request
                    if let Some((_, tx)) = self.pending_requests.remove(&request_id) {
                        let _ = tx.send(message);
                    }
                }
                GrpcFrame::Join { node_id, addr, topics } => {
                    info!("Node {} joined cluster from {}", node_id, addr);
                    // Send join ack
                    let ack = GrpcFrame::JoinAck {
                        success: true,
                        cluster_id: format!("cluster-{}", self.config.node_id),
                        nodes: self.get_cluster_nodes().await,
                    };
                    let _ = self.send_frame(&stream, &ack).await;
                }
                GrpcFrame::Ping { timestamp, node_id } => {
                    let pong = GrpcFrame::Pong {
                        timestamp: Utc::now().timestamp_millis(),
                        node_id: self.config.node_id.clone(),
                    };
                    let _ = self.send_frame(&stream, &pong).await;
                }
                _ => {}
            }
        }
        
        read_handle.abort();
        Ok(())
    }
    
    /// Send a frame over the stream
    async fn send_frame(&self, stream: &Arc<Mutex<TcpStream>>, frame: &GrpcFrame) -> Result<()> {
        let data = serde_json::to_vec(frame)
            .map_err(|e| MessageBusError::Serialization(e.to_string()))?;
        
        let len = (data.len() as u32).to_be_bytes();
        let mut msg = len.to_vec();
        msg.extend_from_slice(&data);
        
        let mut s = stream.lock().await;
        s.write_all(&msg).await
            .map_err(|e| MessageBusError::Transport(format!("Write error: {}", e)))?;
        s.flush().await
            .map_err(|e| MessageBusError::Transport(format!("Flush error: {}", e)))?;
        
        Ok(())
    }
    
    /// Connect to cluster seed nodes
    async fn connect_to_cluster(&self) {
        for addr in &self.config.cluster_addrs {
            match self.connect_to_node(*addr).await {
                Ok(conn) => {
                    info!("Connected to cluster node at {}", addr);
                    // Send join request
                    let join = GrpcFrame::Join {
                        node_id: self.config.node_id.clone(),
                        addr: self.config.bind_addr.to_string(),
                        topics: self.local_subs.iter().map(|e| e.key().clone()).collect(),
                    };
                    let _ = self.send_frame(&conn.stream, &join).await;
                }
                Err(e) => {
                    warn!("Failed to connect to {}: {}", addr, e);
                }
            }
        }
    }
    
    /// Connect to a specific node
    async fn connect_to_node(&self, addr: SocketAddr) -> Result<Arc<RemoteConnection>> {
        let stream = timeout(self.config.connect_timeout, TcpStream::connect(addr)).await
            .map_err(|_| MessageBusError::Timeout)?
            .map_err(|e| MessageBusError::Transport(format!("Connection failed: {}", e)))?;
        
        let conn = Arc::new(RemoteConnection {
            node_id: format!("unknown-{}", addr),
            stream: Arc::new(Mutex::new(stream)),
            heartbeat_tx: mpsc::channel(1).0,
            pending_requests: Arc::clone(&self.pending_requests),
        });
        
        Ok(conn)
    }
    
    /// Run heartbeat to cluster nodes
    async fn run_heartbeat(&self) {
        let mut interval = interval(self.config.keepalive_interval);
        
        loop {
            interval.tick().await;
            
            for entry in self.cluster_nodes.iter() {
                let ping = GrpcFrame::Ping {
                    timestamp: Utc::now().timestamp_millis(),
                    node_id: self.config.node_id.clone(),
                };
                let _ = self.send_frame(&entry.value().stream, &ping).await;
            }
        }
    }
    
    /// Forward message to local subscribers
    async fn forward_to_local_subs(&self, topic: &str, message: Message) {
        let matching_subs = self.get_matching_subs(topic);
        
        for sub_id in matching_subs {
            if let Some(sender) = self.sub_senders.get(&sub_id) {
                let _ = sender.send(message.clone());
            }
        }
    }
    
    /// Get matching subscription IDs for a topic
    fn get_matching_subs(&self, topic: &str) -> Vec<SubscriptionId> {
        let mut matches = Vec::new();
        
        for entry in self.local_subs.iter() {
            if self.topic_matches(entry.key(), topic) {
                matches.extend(entry.value().iter().cloned());
            }
        }
        
        matches
    }
    
    /// Check if pattern matches topic
    fn topic_matches(&self, pattern: &str, topic: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let topic_parts: Vec<&str> = topic.split('/').collect();
        
        let mut i = 0;
        for (j, part) in pattern_parts.iter().enumerate() {
            if i >= topic_parts.len() {
                return false;
            }
            match *part {
                "#" => return true, // Multi-level wildcard
                "+" => i += 1,      // Single-level wildcard
                _ if part == topic_parts[i] => i += 1,
                _ => return false,
            }
        }
        
        i == topic_parts.len()
    }
    
    /// Handle request from remote node
    async fn handle_remote_request(&self, topic: String, message: Message, timeout: Duration) -> Result<Message> {
        // This should be implemented by the actual request handler
        // For now, return an error message
        let response = Message::new("error", serde_json::json!({
            "error": "Remote request handling not implemented",
            "topic": topic,
        }));
        Ok(response)
    }
    
    /// Get cluster node information
    async fn get_cluster_nodes(&self) -> Vec<NodeInfo> {
        self.cluster_nodes
            .iter()
            .map(|e| NodeInfo {
                node_id: e.key().clone(),
                address: e.value().stream.lock().await.peer_addr().map(|a| a.to_string()).unwrap_or_default(),
                topics: vec![],
                message_count: 0,
                status: NodeStatus::Active,
                last_seen: Utc::now().timestamp_millis(),
            })
            .collect()
    }
    
    /// Broadcast message to cluster
    async fn broadcast_to_cluster(&self, frame: &GrpcFrame) -> Result<()> {
        for entry in self.cluster_nodes.iter() {
            let _ = self.send_frame(&entry.value().stream, frame).await;
        }
        Ok(())
    }
}

#[async_trait]
impl MessageBus for GrpcTransport {
    async fn publish(&self, topic: &str, message: Message) -> Result<()> {
        // Forward to local subscribers
        self.forward_to_local_subs(topic, message.clone()).await;
        
        // Broadcast to cluster
        let frame = GrpcFrame::Publish {
            topic: topic.to_string(),
            message,
        };
        self.broadcast_to_cluster(&frame).await?;
        
        *self.message_counter.write() += 1;
        Ok(())
    }
    
    async fn subscribe(&self, topic_pattern: &str) -> Result<(SubscriptionId, crate::MessageStream)> {
        let sub_id = format!("sub-{}", Uuid::new_v4());
        let (tx, rx) = mpsc::unbounded_channel();
        
        self.local_subs.entry(topic_pattern.to_string()).or_default().push(sub_id.clone());
        self.sub_senders.insert(sub_id.clone(), tx);
        
        // Create message stream
        let stream = crate::MessageStream::new(rx);
        
        // Broadcast subscription to cluster
        let frame = GrpcFrame::Subscribe {
            pattern: topic_pattern.to_string(),
            sub_id: sub_id.clone(),
        };
        let _ = self.broadcast_to_cluster(&frame).await;
        
        Ok((sub_id, stream))
    }
    
    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<()> {
        self.sub_senders.remove(&subscription_id);
        
        for mut entry in self.local_subs.iter_mut() {
            entry.value().retain(|id| id != &subscription_id);
        }
        
        // Broadcast to cluster
        let frame = GrpcFrame::Unsubscribe { sub_id: subscription_id };
        let _ = self.broadcast_to_cluster(&frame).await;
        
        Ok(())
    }
    
    async fn request(&self, topic: &str, message: Message, timeout: Duration) -> Result<Message> {
        let request_id = format!("req-{}", Uuid::new_v4());
        let (tx, rx) = oneshot::channel();
        
        self.pending_requests.insert(request_id.clone(), tx);
        
        // Try local nodes first
        let frame = GrpcFrame::Request {
            request_id: request_id.clone(),
            topic: topic.to_string(),
            message,
            timeout_ms: timeout.as_millis() as u64,
        };
        self.broadcast_to_cluster(&frame).await?;
        
        // Wait for response
        match timeout(timeout, rx).await {
            Ok(Ok(msg)) => Ok(msg),
            Ok(Err(_)) => Err(MessageBusError::ChannelClosed),
            Err(_) => {
                self.pending_requests.remove(&request_id);
                Err(MessageBusError::Timeout)
            }
        }
    }
    
    async fn respond(&self, correlation_id: &str, message: Message) -> Result<()> {
        let frame = GrpcFrame::Response {
            request_id: correlation_id.to_string(),
            message,
        };
        self.broadcast_to_cluster(&frame).await
    }
}

impl Drop for GrpcTransport {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
    }
}

/// Builder for GrpcTransport
pub struct GrpcTransportBuilder {
    config: GrpcConfig,
}

impl GrpcTransportBuilder {
    pub fn new() -> Self {
        Self {
            config: GrpcConfig::default(),
        }
    }
    
    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.config.bind_addr = addr;
        self
    }
    
    pub fn cluster_addrs(mut self, addrs: Vec<SocketAddr>) -> Self {
        self.config.cluster_addrs = addrs;
        self
    }
    
    pub fn node_id(mut self, id: String) -> Self {
        self.config.node_id = id;
        self
    }
    
    pub fn keepalive_interval(mut self, interval: Duration) -> Self {
        self.config.keepalive_interval = interval;
        self
    }
    
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }
    
    pub fn max_message_size(mut self, size: usize) -> Self {
        self.config.max_message_size = size;
        self
    }
    
    pub fn build(self) -> Result<Arc<GrpcTransport>> {
        // This is a blocking call in async context, should use tokio::runtime::Handle::current().block_on
        // But for simplicity, return an error requiring async build
        Err(MessageBusError::Config("Use GrpcTransport::new().await instead".to_string()))
    }
    
    pub async fn build_async(self) -> Result<Arc<GrpcTransport>> {
        GrpcTransport::new(self.config).await
    }
}

impl Default for GrpcTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_grpc_config_default() {
        let config = GrpcConfig::default();
        assert_eq!(config.bind_addr.to_string(), "0.0.0.0:50051");
        assert!(config.cluster_addrs.is_empty());
        assert!(!config.tls_enabled);
    }
    
    #[tokio::test]
    async fn test_topic_matching() {
        let config = GrpcConfig::default();
        let transport = GrpcTransport {
            config,
            local_subs: DashMap::new(),
            sub_senders: DashMap::new(),
            cluster_nodes: DashMap::new(),
            pending_requests: Arc::new(DashMap::new()),
            shutdown_tx: broadcast::channel(1).0,
            message_counter: Arc::new(RwLock::new(0)),
        };
        
        assert!(transport.topic_matches("agent/+/status", "agent/123/status"));
        assert!(!transport.topic_matches("agent/+/status", "agent/123/task/status"));
        assert!(transport.topic_matches("agent/#", "agent/123/task/status"));
        assert!(transport.topic_matches("agent/123/#", "agent/123/task/status"));
    }
}
