//! Unified Channels Example
//!
//! This example demonstrates how to use the unified Channel trait
//! with different messaging platforms.

use std::collections::HashMap;

use beebotos_agents::communication::channel::*;
use beebotos_agents::communication::{Message, MessageType, PlatformType};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Unified Channels Example ===\n");

    // Example 1: Telegram Channel (Polling mode)
    println!("1. Telegram Channel (Polling mode)");
    if let Ok(telegram) = TelegramChannel::from_env() {
        println!("   ✓ Telegram channel created from environment");
        println!("   Platform: {:?}", telegram.platform());
        println!("   Connection mode: {:?}", telegram.connection_mode());
    } else {
        println!("   ✗ TELEGRAM_BOT_TOKEN not set, skipping");
    }
    println!();

    // Example 2: DingTalk Channel (WebSocket mode)
    println!("2. DingTalk Channel (WebSocket mode)");
    if let Ok(dingtalk) = DingTalkChannel::from_env() {
        println!("   ✓ DingTalk channel created from environment");
        println!("   Platform: {:?}", dingtalk.platform());
        println!("   Connection mode: {:?}", dingtalk.connection_mode());
    } else {
        println!("   ✗ DINGTALK_APP_KEY or DINGTALK_APP_SECRET not set, skipping");
    }
    println!();

    // Example 3: Discord Channel (WebSocket Gateway mode)
    println!("3. Discord Channel (WebSocket Gateway mode)");
    if let Ok(discord) = DiscordChannel::from_env() {
        println!("   ✓ Discord channel created from environment");
        println!("   Platform: {:?}", discord.platform());
        println!("   Connection mode: {:?}", discord.connection_mode());
    } else {
        println!("   ✗ DISCORD_BOT_TOKEN not set, skipping");
    }
    println!();

    // Example 4: Slack Channel (Socket Mode)
    println!("4. Slack Channel (Socket Mode / WebSocket)");
    if let Ok(slack) = SlackChannel::from_env() {
        println!("   ✓ Slack channel created from environment");
        println!("   Platform: {:?}", slack.platform());
        println!("   Connection mode: {:?}", slack.connection_mode());
    } else {
        println!("   ✗ SLACK_BOT_TOKEN not set, skipping");
    }
    println!();

    // Example 5: Matrix Channel (Polling mode)
    println!("5. Matrix Channel (Polling mode)");
    if let Ok(matrix) = MatrixChannel::from_env() {
        println!("   ✓ Matrix channel created from environment");
        println!("   Platform: {:?}", matrix.platform());
        println!("   Connection mode: {:?}", matrix.connection_mode());
    } else {
        println!("   ✗ MATRIX_USERNAME or MATRIX_PASSWORD not set, skipping");
    }
    println!();

    // Example 6: WeChat Work Channel (Webhook mode)
    println!("6. WeChat Work Channel (Webhook mode)");
    if let Ok(wechat) = WeChatChannel::from_env() {
        println!("   ✓ WeChat channel created from environment");
        println!("   Platform: {:?}", wechat.platform());
        println!("   Connection mode: {:?}", wechat.connection_mode());
    } else {
        println!("   ✗ WECHAT_CORP_ID, WECHAT_CORP_SECRET, or WECHAT_AGENT_ID not set, skipping");
    }
    println!();

    // Example 7: WhatsApp Channel (WebSocket via Baileys Bridge)
    println!("7. WhatsApp Channel (WebSocket via Baileys Bridge)");
    if let Ok(whatsapp) = WhatsAppChannel::from_env() {
        println!("   ✓ WhatsApp channel created from environment");
        println!("   Platform: {:?}", whatsapp.platform());
        println!("   Connection mode: {:?}", whatsapp.connection_mode());
    } else {
        println!("   ✗ Failed to load WhatsApp configuration, skipping");
    }
    println!();

    // Example 8: Using the Channel trait polymorphically
    println!("8. Using Channel trait polymorphically");
    
    // Create a vector of channels
    let mut channels: Vec<Box<dyn Channel>> = vec![];
    
    // Add channels if they can be created
    if let Ok(telegram) = TelegramChannel::from_env() {
        channels.push(Box::new(telegram));
    }
    if let Ok(discord) = DiscordChannel::from_env() {
        channels.push(Box::new(discord));
    }
    if let Ok(slack) = SlackChannel::from_env() {
        channels.push(Box::new(slack));
    }

    println!("   Created {} channels", channels.len());
    
    for channel in &channels {
        println!(
            "   - {} ({:?}): {:?}",
            channel.name(),
            channel.platform(),
            channel.connection_mode()
        );
        println!(
            "     Supported content types: {:?}",
            channel.supported_content_types()
        );
    }
    println!();

    // Example 9: Manual configuration
    println!("9. Manual configuration example");
    
    let telegram_config = TelegramChannelConfig {
        bot_token: "your_bot_token".to_string(),
        connection_mode: ConnectionMode::Polling,
        auto_reconnect: true,
        max_reconnect_attempts: 10,
        polling_interval_secs: 1,
        webhook_url: None,
        webhook_port: 8080,
        allowed_updates: None,
    };
    
    let telegram = TelegramChannel::new(telegram_config);
    println!("   ✓ Telegram channel created with manual config");
    println!("   Connection mode: {:?}", telegram.connection_mode());
    println!();

    // Example 10: Event handling
    println!("10. Event handling example");
    println!("   Creating event bus...");
    
    let (event_tx, mut event_rx) = mpsc::channel::<ChannelEvent>(100);
    
    // Spawn event handler
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                ChannelEvent::MessageReceived { platform, channel_id, message } => {
                    println!(
                        "   📨 Received message from {:?} in {}: {}",
                        platform, channel_id, message.content
                    );
                }
                ChannelEvent::ConnectionStateChanged { platform, connected, reason } => {
                    println!(
                        "   🔌 {:?} connection state: {} (reason: {:?})",
                        platform, connected, reason
                    );
                }
                ChannelEvent::Error { platform, error } => {
                    println!("   ❌ Error on {:?}: {}", platform, error);
                }
            }
        }
    });
    
    println!("   ✓ Event handler spawned");
    println!();

    println!("=== Example completed ===");
    println!();
    println!("Environment variables needed:");
    println!("  - TELEGRAM_BOT_TOKEN");
    println!("  - DINGTALK_APP_KEY, DINGTALK_APP_SECRET");
    println!("  - DISCORD_BOT_TOKEN");
    println!("  - SLACK_BOT_TOKEN, SLACK_APP_TOKEN (for Socket Mode)");
    println!("  - MATRIX_USERNAME, MATRIX_PASSWORD (or MATRIX_ACCESS_TOKEN)");
    println!("  - WECHAT_CORP_ID, WECHAT_CORP_SECRET, WECHAT_AGENT_ID");
    println!("  - WHATSAPP_BRIDGE_PATH (optional)");

    Ok(())
}

/// Example: Generic function that works with any Channel
async fn send_greeting<C: Channel>(
    channel: &C,
    channel_id: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = Message {
        id: uuid::Uuid::new_v4(),
        thread_id: uuid::Uuid::new_v4(),
        platform: channel.platform(),
        message_type: MessageType::Text,
        content: format!("Hello, {}! Welcome to BeeBotOS.", name),
        metadata: HashMap::new(),
        timestamp: chrono::Utc::now(),
    };
    
    channel.send(channel_id, &message).await?;
    Ok(())
}

/// Example: Connect and start listening on any Channel
async fn setup_channel<C: Channel>(
    channel: &mut C,
    event_bus: mpsc::Sender<ChannelEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the platform
    channel.connect().await?;
    
    // Start listening for events
    channel.start_listener(event_bus).await?;
    
    println!(
        "Connected to {} ({:?}) via {:?}",
        channel.name(),
        channel.platform(),
        channel.connection_mode()
    );
    
    Ok(())
}
