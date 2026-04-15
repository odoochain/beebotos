//! Complete Channel Example
//!
//! This example demonstrates the complete channel system including:
//! - Channel Manager
//! - Multiple platforms
//! - Template engine
//! - Webhook server
//! - Event handling

use std::collections::HashMap;
use std::sync::Arc;

use beebotos_agents::communication::channel::*;
use beebotos_agents::communication::{Message, MessageType, PlatformType};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Complete Channel System Example ===\n");

    // Step 1: Create Channel Manager
    println!("1. Creating Channel Manager...");
    let manager_config = ChannelManagerConfig {
        auto_connect: false, // We'll connect manually
        auto_listen: false,
        event_buffer_size: 1000,
        enable_templates: true,
        template_directory: None,
    };

    let (manager, mut event_rx) = ChannelManager::new(manager_config).await?;
    println!("   ✓ Channel Manager created\n");

    // Step 2: Register channels
    println!("2. Registering channels...");

    // Telegram
    if let Ok(telegram) = TelegramChannel::from_env() {
        manager.register_channel(
            "telegram-1",
            "Main Telegram Bot",
            Arc::new(telegram),
        ).await?;
        println!("   ✓ Registered Telegram channel");
    }

    // Discord
    if let Ok(discord) = DiscordChannel::from_env() {
        manager.register_channel(
            "discord-1",
            "Main Discord Bot",
            Arc::new(discord),
        ).await?;
        println!("   ✓ Registered Discord channel");
    }

    // Slack
    if let Ok(slack) = SlackChannel::from_env() {
        manager.register_channel(
            "slack-1",
            "Main Slack Bot",
            Arc::new(slack),
        ).await?;
        println!("   ✓ Registered Slack channel");
    }

    // Twitter
    if let Ok(twitter) = TwitterChannel::from_env() {
        manager.register_channel(
            "twitter-1",
            "Main Twitter Bot",
            Arc::new(twitter),
        ).await?;
        println!("   ✓ Registered Twitter channel");
    }

    println!();

    // Step 3: Display registered channels
    println!("3. Registered channels:");
    let statuses = manager.get_all_statuses().await;
    for status in &statuses {
        println!(
            "   - {} ({:?}): connected={}, listening={}",
            status.id, status.platform, status.connected, status.listening
        );
    }
    println!();

    // Step 4: Spawn event handler
    println!("4. Starting event handler...");
    let event_handler = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                ChannelEvent::MessageReceived { platform, channel_id, message } => {
                    println!(
                        "   📨 [{:?}] {}: {}",
                        platform, channel_id, message.content
                    );
                }
                ChannelEvent::ConnectionStateChanged { platform, connected, reason } => {
                    println!(
                        "   🔌 [{:?}] Connected: {} (reason: {:?})",
                        platform, connected, reason
                    );
                }
                ChannelEvent::Error { platform, error } => {
                    eprintln!("   ❌ [{:?}] Error: {}", platform, error);
                }
            }
        }
    });
    println!();

    // Step 5: Connect all channels
    println!("5. Connecting all channels...");
    let connect_results = manager.connect_all().await;
    for (id, result) in connect_results {
        match result {
            Ok(_) => println!("   ✓ Connected {}", id),
            Err(e) => println!("   ✗ Failed to connect {}: {}", id, e),
        }
    }
    println!();

    // Step 6: Use template engine
    println!("6. Using template engine...");
    if let Some(template_engine) = manager.get_template_engine().await {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("service".to_string(), "BeeBotOS".to_string());
        vars.insert("is_new".to_string(), "true".to_string());

        // Render template for different platforms
        let platforms = vec![
            PlatformType::Telegram,
            PlatformType::Discord,
            PlatformType::Slack,
        ];

        for platform in platforms {
            match template_engine.render("welcome", &vars, Some(platform)) {
                Ok(rendered) => {
                    println!("   [{:?}] {}", platform, rendered.lines().next().unwrap_or(""));
                }
                Err(e) => {
                    println!("   [{:?}] Error: {}", platform, e);
                }
            }
        }
    }
    println!();

    // Step 7: Send template messages
    println!("7. Sending template messages...");
    for status in &statuses {
        if status.connected {
            let mut vars = HashMap::new();
            vars.insert("name".to_string(), "User".to_string());
            vars.insert("title".to_string(), "System Status".to_string());
            vars.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

            // Note: This would actually send messages in a real scenario
            println!(
                "   Would send to {} using template",
                status.id
            );
        }
    }
    println!();

    // Step 8: Channel routing example
    println!("8. Channel routing example...");
    let mut router = ChannelRouter::new();
    router.set_default(PlatformType::Telegram, "telegram-1");
    router.set_default(PlatformType::Discord, "discord-1");
    router.set_default(PlatformType::Slack, "slack-1");
    router.set_default(PlatformType::Twitter, "twitter-1");

    let test_message = Message {
        id: uuid::Uuid::new_v4(),
        thread_id: uuid::Uuid::new_v4(),
        platform: PlatformType::Telegram,
        message_type: MessageType::Text,
        content: "Test broadcast message".to_string(),
        metadata: HashMap::new(),
        timestamp: chrono::Utc::now(),
    };

    for platform in vec![
        PlatformType::Telegram,
        PlatformType::Discord,
        PlatformType::Slack,
        PlatformType::Twitter,
    ] {
        if let Some(channel_id) = router.route(platform, &test_message) {
            println!("   [{:?}] -> {}", platform, channel_id);
        }
    }
    println!();

    // Step 9: Health monitoring
    println!("9. Health monitoring...");
    let health_monitor = ChannelHealthMonitor::new(Duration::from_secs(30), 5);
    health_monitor.start(&manager).await;
    
    // Check health
    for status in &statuses {
        let healthy = health_monitor.is_healthy(&status.id).await;
        println!("   {}: {}", status.id, if healthy { "✓ healthy" } else { "✗ unhealthy" });
    }
    println!();

    // Step 10: Webhook server example
    println!("10. Webhook server example...");
    let webhook_config = WebhookServerConfig {
        bind_address: "0.0.0.0".to_string(),
        port: 8080,
        path_prefix: "/webhook".to_string(),
        secret: None,
        tls_cert_path: None,
        tls_key_path: None,
    };

    // Create webhook server
    let (webhook_server, webhook_state) = create_webhook_server(
        webhook_config,
        manager.event_tx.clone(),
    ).await?;

    // Register webhook handlers
    let telegram_handler = Arc::new(TelegramWebhookHandler::new(None));
    webhook_state.register_handler(
        PlatformType::Telegram,
        "/telegram",
        telegram_handler,
    ).await;

    let slack_handler = Arc::new(SlackWebhookHandler::new(None));
    webhook_state.register_handler(
        PlatformType::Slack,
        "/slack",
        slack_handler,
    ).await;

    let discord_handler = Arc::new(DiscordWebhookHandler::new(None));
    webhook_state.register_handler(
        PlatformType::Discord,
        "/discord",
        discord_handler,
    ).await;

    println!("   ✓ Webhook server started on http://0.0.0.0:8080");
    println!("     - /webhook/telegram - Telegram webhooks");
    println!("     - /webhook/slack - Slack webhooks");
    println!("     - /webhook/discord - Discord webhooks");
    println!();

    // Step 11: Channel statistics
    println!("11. Channel statistics:");
    let updated_statuses = manager.get_all_statuses().await;
    for status in updated_statuses {
        println!(
            "   {}: sent={}, received={}, errors={}",
            status.id, status.messages_sent, status.messages_received, status.error_count
        );
    }
    println!();

    // Step 12: Graceful shutdown
    println!("12. Shutting down...");
    
    // Disconnect all channels
    manager.shutdown().await;
    
    // Stop event handler
    event_handler.abort();
    
    println!("   ✓ Shutdown complete");
    println!();

    println!("=== Example completed ===");
    println!();
    println!("Summary:");
    println!("  - Registered {} channels", statuses.len());
    println!("  - Template engine with built-in templates");
    println!("  - Webhook server for receiving events");
    println!("  - Health monitoring");
    println!("  - Channel routing");
    println!();
    println!("Environment variables needed for full functionality:");
    println!("  - TELEGRAM_BOT_TOKEN");
    println!("  - DISCORD_BOT_TOKEN");
    println!("  - SLACK_BOT_TOKEN, SLACK_APP_TOKEN");
    println!("  - TWITTER_BEARER_TOKEN or TWITTER_API_KEY/SECRET");

    Ok(())
}
