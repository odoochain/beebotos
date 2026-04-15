//! Test Channel Registry initialization
//! Run with: cargo test -p beebotos-agents test_channel_registry -- --nocapture

use beebotos_agents::{
    ChannelRegistry, 
    LarkChannelFactory, DingTalkChannelFactory,
    TelegramChannelFactory, DiscordChannelFactory, SlackChannelFactory,
    communication::channel::ChannelEvent,
};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_channel_registry_creation() {
    let (tx, _rx) = mpsc::channel::<ChannelEvent>(100);
    let registry = ChannelRegistry::new(tx);
    
    // Check initial state
    let factory_count = registry.factory_count().await;
    let channel_count = registry.channel_count().await;
    
    assert_eq!(factory_count, 0, "Initial factory count should be 0");
    assert_eq!(channel_count, 0, "Initial channel count should be 0");
    
    println!("✅ ChannelRegistry created successfully");
}

#[tokio::test]
async fn test_factory_registration() {
    let (tx, _rx) = mpsc::channel::<ChannelEvent>(100);
    let registry = ChannelRegistry::new(tx);
    
    // Register all factories
    registry.register(Box::new(LarkChannelFactory::new())).await;
    registry.register(Box::new(DingTalkChannelFactory::new())).await;
    registry.register(Box::new(TelegramChannelFactory::new())).await;
    registry.register(Box::new(DiscordChannelFactory::new())).await;
    registry.register(Box::new(SlackChannelFactory::new())).await;
    
    let factory_count = registry.factory_count().await;
    assert_eq!(factory_count, 5, "Should have 5 factories registered");
    
    println!("✅ All 5 factories registered successfully");
}

#[tokio::test]
async fn test_lark_factory_validation() {
    let factory = LarkChannelFactory::new();
    
    // Valid config
    let valid_config = serde_json::json!({
        "app_id": "cli_123456",
        "app_secret": "secret_789012"
    });
    assert!(factory.validate_config(&valid_config), "Valid config should pass");
    
    // Invalid config - missing app_secret
    let invalid_config = serde_json::json!({
        "app_id": "cli_123456"
    });
    assert!(!factory.validate_config(&invalid_config), "Invalid config should fail");
    
    // Invalid config - empty values
    let empty_config = serde_json::json!({
        "app_id": "",
        "app_secret": ""
    });
    assert!(!factory.validate_config(&empty_config), "Empty config should fail");
    
    println!("✅ Lark factory validation works correctly");
}

#[tokio::test]
async fn test_telegram_factory_validation() {
    let factory = TelegramChannelFactory::new();
    
    // Valid config
    let valid_config = serde_json::json!({
        "bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
    });
    assert!(factory.validate_config(&valid_config), "Valid token should pass");
    
    // Invalid config - no colon
    let invalid_config = serde_json::json!({
        "bot_token": "invalid_token_without_colon"
    });
    assert!(!factory.validate_config(&invalid_config), "Token without colon should fail");
    
    println!("✅ Telegram factory validation works correctly");
}

#[tokio::test]
async fn test_slack_factory_validation() {
    let factory = SlackChannelFactory::new();
    
    // Valid config
    let valid_config = serde_json::json!({
        "bot_token": "xoxb-TESTTOKEN"
    });
    assert!(factory.validate_config(&valid_config), "Valid xoxb token should pass");
    
    // Invalid config - wrong prefix
    let invalid_config = serde_json::json!({
        "bot_token": "bot-token-without-xoxb-prefix"
    });
    assert!(!factory.validate_config(&invalid_config), "Token without xoxb- prefix should fail");
    
    println!("✅ Slack factory validation works correctly");
}
