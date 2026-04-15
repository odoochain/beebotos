//! Personal WeChat Channel Example
//!
//! This example demonstrates how to use the Personal WeChat channel
//! to receive and send messages via the OpenClaw/iLink protocol.
//!
//! ## Prerequisites
//!
//! 1. Deploy ClawBot service (https://github.com/SiverKing/weixin-ClawBot-API)
//! 2. Get API key from the service
//! 3. Run this example
//!
//! ## Usage
//!
//! ```bash
//! export PERSONAL_WECHAT_API_KEY="your_api_key_here"
//! export PERSONAL_WECHAT_API_URL="http://localhost:3000"
//! cargo run --example personal_wechat_example
//! ```

use beebotos_agents::communication::channel::{
    ChannelRegistry, PersonalWeChatFactory, PersonalWeChatConfig, ChannelEvent,
};
use beebotos_agents::communication::{Message, MessageType, PlatformType};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Personal WeChat Channel Example");

    // Create event bus
    let (event_bus_tx, mut event_bus_rx) = mpsc::channel(100);

    // Create channel registry
    let mut registry = ChannelRegistry::new(event_bus_tx);

    // Register Personal WeChat factory
    registry.register(Box::new(PersonalWeChatFactory::new())).await;
    info!("✅ Registered PersonalWeChatFactory");

    // Create configuration
    let api_key = std::env::var("PERSONAL_WECHAT_API_KEY")
        .unwrap_or_else(|_| {
            warn!("PERSONAL_WECHAT_API_KEY not set, using demo key");
            "demo_key_123".to_string()
        });

    let api_url = std::env::var("PERSONAL_WECHAT_API_URL")
        .unwrap_or_else(|_| {
            warn!("PERSONAL_WECHAT_API_URL not set, using default");
            "http://localhost:3000".to_string()
        });

    let config = json!({
        "api_key": api_key,
        "api_url": api_url,
        "auto_reconnect": true,
        "reconnect_interval_secs": 300,
        "connection_mode": "polling"
    });

    info!("📝 Configuration:");
    info!("   API URL: {}", api_url);
    info!("   Auto Reconnect: true");

    // Create the channel
    let channel_name = registry
        .create_channel("personal_wechat", &config, Some("my_wechat"))
        .await?;

    info!("✅ Created channel: {}", channel_name);
    info!("");
    info!("========================================");
    info!("请查看日志中的二维码并扫码登录...");
    info!("========================================");
    info!("");

    // Spawn message handler task
    let registry_clone = registry.clone();
    let channel_name_clone = channel_name.clone();

    tokio::spawn(async move {
        while let Some(event) = event_bus_rx.recv().await {
            match event {
                ChannelEvent::MessageReceived { platform, channel_id, message } => {
                    info!("");
                    info!("📨 New Message:");
                    info!("   Platform: {:?}", platform);
                    info!("   From: {}", channel_id);
                    info!("   Content: {}", message.content);

                    // Extract metadata
                    if let Some(is_group) = message.metadata.get("is_group") {
                        info!("   Is Group: {}", is_group);
                    }
                    if let Some(group_name) = message.metadata.get("group_name") {
                        info!("   Group: {}", group_name);
                    }
                    if let Some(sender) = message.metadata.get("sender_name") {
                        info!("   Sender: {}", sender);
                    }

                    // Auto-reply logic
                    let reply_content = format!("收到你的消息: {}", message.content);

                    let reply = Message {
                        id: uuid::Uuid::new_v4(),
                        thread_id: message.thread_id,
                        platform: PlatformType::WeChat,
                        message_type: MessageType::Text,
                        content: reply_content,
                        metadata: std::collections::HashMap::new(),
                        timestamp: chrono::Utc::now(),
                    };

                    // Send reply
                    match registry_clone.get_channel(&channel_name_clone).await {
                        Ok(channel) => {
                            match channel.write().await.send(&channel_id, &reply).await {
                                Ok(_) => info!("✅ Reply sent successfully"),
                                Err(e) => error!("❌ Failed to send reply: {}", e),
                            }
                        }
                        Err(e) => error!("❌ Failed to get channel: {}", e),
                    }

                    info!("");
                }
                ChannelEvent::ConnectionStateChanged { platform, connected, reason } => {
                    info!("📡 Connection State Changed:");
                    info!("   Platform: {:?}", platform);
                    info!("   Connected: {}", connected);
                    if let Some(r) = reason {
                        info!("   Reason: {}", r);
                    }
                }
                ChannelEvent::Error { platform, error } => {
                    error!("❌ Channel Error:");
                    error!("   Platform: {:?}", platform);
                    error!("   Error: {}", error);
                }
                _ => {}
            }
        }
    });

    // Keep main task running
    info!("🤖 Bot is running. Press Ctrl+C to exit.");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    info!("");
    info!("🛑 Shutting down...");

    // Cleanup
    let channel = registry.get_channel(&channel_name).await?;
    channel.write().await.disconnect().await?;

    info!("✅ Goodbye!");

    Ok(())
}

/// Example: Manual login flow
#[allow(dead_code)]
async fn manual_login_example() -> Result<(), Box<dyn std::error::Error>> {
    use beebotos_agents::communication::channel::PersonalWeChatChannel;

    // Create config
    let config = PersonalWeChatConfig {
        api_key: "your_api_key".to_string(),
        bot_id: None,
        api_url: "http://localhost:3000".to_string(),
        reconnect_interval_secs: 300,
        use_ilink_official: false,
        base: Default::default(),
    };

    // Create channel
    let channel = PersonalWeChatChannel::new(config);

    // Get QR code
    let login_resp = channel.get_qr_code().await?;
    info!("QR Code URL: {}", login_resp.qr_code_url);

    // Wait for login
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let status = channel.check_login_status().await?;
        if status.logged_in {
            info!("Logged in as: {:?}", status.nickname);
            break;
        }
    }

    Ok(())
}

/// Example: Send message
#[allow(dead_code)]
async fn send_message_example(
    registry: &ChannelRegistry,
    channel_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel = registry.get_channel(channel_name).await?;

    let message = Message {
        id: uuid::Uuid::new_v4(),
        thread_id: uuid::Uuid::new_v4(),
        platform: PlatformType::WeChat,
        message_type: MessageType::Text,
        content: "Hello from BeeBotOS!".to_string(),
        metadata: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
    };

    // Send to a specific user (wxid)
    channel.write().await.send("wxid_xxxxxxxx", &message).await?;
    info!("Message sent!");

    Ok(())
}

/// Example: Check session status
#[allow(dead_code)]
async fn check_session_example(
    registry: &ChannelRegistry,
    channel_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let channel = registry.get_channel(channel_name).await?;
    let channel_guard = channel.read().await;

    // Check if connected
    let is_connected = channel_guard.is_connected();
    info!("Connected: {}", is_connected);

    // Check session validity
    let is_valid = channel_guard.is_session_valid().await;
    info!("Session Valid: {}", is_valid);

    // Get session info
    if let Some(session) = channel_guard.get_session_info().await {
        info!("Bot ID: {}", session.bot_id);
        info!("Expires At: {}", session.expires_at);
        if let Some(nickname) = session.nickname {
            info!("Nickname: {}", nickname);
        }
        if let Some(wxid) = session.wxid {
            info!("WXID: {}", wxid);
        }
    }

    Ok(())
}
