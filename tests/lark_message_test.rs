//! Lark Message Flow Test
//! 
//! This test simulates the message flow from Lark to the Gateway.
//! Run with: cargo test -p beebotos-agents test_lark_message -- --nocapture

use beebotos_agents::{
    ChannelRegistry, LarkChannelFactory,
    communication::{PlatformType, Message, MessageType},
    communication::channel::{ChannelEvent, ContentType},
    media::downloader_v2::{LazyUrl, MediaDownloaderV2},
};
use tokio::sync::mpsc;
use serde_json::json;

/// Simulate a Lark message received via WebSocket
fn create_test_lark_message() -> serde_json::Value {
    json!({
        "schema": "2.0",
        "header": {
            "event_id": "test_event_123",
            "token": "test_token",
            "create_time": "1234567890",
            "event_type": "im.message.receive_v1",
            "app_id": "cli_a92ba86d9978dbcd",
            "tenant_key": "tenant_key"
        },
        "event": {
            "message": {
                "message_id": "om_1234567890abcdef",
                "root_id": null,
                "parent_id": null,
                "create_time": "1234567890000",
                "chat_id": "oc_1234567890abcdef",
                "chat_type": "p2p",
                "message_type": "text",
                "content": "{\"text\":\"Hello from Lark test\"}",
                "mentions": null
            },
            "sender": {
                "sender_id": {
                    "union_id": "on_1234567890abcdef",
                    "user_id": "user_123",
                    "open_id": "ou_1234567890abcdef"
                },
                "sender_type": "user",
                "tenant_key": "tenant_key"
            }
        }
    })
}

/// Simulate a Lark image message
fn create_test_lark_image_message() -> serde_json::Value {
    json!({
        "schema": "2.0",
        "header": {
            "event_id": "test_event_456",
            "token": "test_token",
            "create_time": "1234567890",
            "event_type": "im.message.receive_v1",
            "app_id": "cli_a92ba86d9978dbcd",
            "tenant_key": "tenant_key"
        },
        "event": {
            "message": {
                "message_id": "om_abcdef1234567890",
                "create_time": "1234567890000",
                "chat_id": "oc_1234567890abcdef",
                "chat_type": "p2p",
                "message_type": "image",
                "content": "{\"image_key\":\"img_abcdef1234567890\"}",
            },
            "sender": {
                "sender_id": {
                    "union_id": "on_1234567890abcdef",
                    "open_id": "ou_1234567890abcdef"
                },
                "sender_type": "user",
                "tenant_key": "tenant_key"
            }
        }
    })
}

#[tokio::test]
async fn test_lark_message_parsing() {
    println!("\n=== Testing Lark Message Parsing ===\n");
    
    let message = create_test_lark_message();
    
    // Extract message details
    let event_type = message["header"]["event_type"].as_str().unwrap();
    let message_id = message["event"]["message"]["message_id"].as_str().unwrap();
    let chat_id = message["event"]["message"]["chat_id"].as_str().unwrap();
    let sender_id = message["event"]["sender"]["sender_id"]["open_id"].as_str().unwrap();
    let content = message["event"]["message"]["content"].as_str().unwrap();
    
    println!("Event Type: {}", event_type);
    println!("Message ID: {}", message_id);
    println!("Chat ID: {}", chat_id);
    println!("Sender ID: {}", sender_id);
    println!("Content: {}", content);
    
    // Parse content JSON
    let content_json: serde_json::Value = serde_json::from_str(content).unwrap();
    let text = content_json["text"].as_str().unwrap();
    
    assert_eq!(event_type, "im.message.receive_v1");
    assert_eq!(text, "Hello from Lark test");
    
    println!("\n✅ Lark text message parsing test passed");
}

#[tokio::test]
async fn test_lark_image_message_lazy_url() {
    println!("\n=== Testing Lark Image Lazy URL ===\n");
    
    let message = create_test_lark_image_message();
    
    let message_id = message["event"]["message"]["message_id"].as_str().unwrap();
    let image_key = message["event"]["message"]["content"]
        .as_str()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v["image_key"].as_str().map(|s| s.to_string()))
        .unwrap();
    
    // Create lazy download URL
    let lazy_url = LazyUrl::build(
        PlatformType::Lark,
        message_id,
        &image_key,
        "image"
    );
    
    println!("Message ID: {}", message_id);
    println!("Image Key: {}", image_key);
    println!("Lazy URL: {}", lazy_url);
    
    // Verify URL format
    assert!(lazy_url.starts_with("lazy://lark/"));
    assert!(lazy_url.contains(message_id));
    assert!(lazy_url.contains(&image_key));
    
    // Parse the URL back
    let (platform, parsed_msg_id, parsed_file_key, file_type) = LazyUrl::parse(&lazy_url).unwrap();
    
    assert!(matches!(platform, PlatformType::Lark));
    assert_eq!(parsed_msg_id, message_id);
    assert_eq!(parsed_file_key, image_key);
    assert_eq!(file_type, "image");
    
    println!("\n✅ Lazy URL generation and parsing test passed");
}

#[tokio::test]
async fn test_channel_event_creation() {
    println!("\n=== Testing Channel Event Creation ===\n");
    
    let lark_message = create_test_lark_message();
    
    // Create a ChannelEvent as would be done in the Lark channel
    let event = ChannelEvent::MessageReceived {
        channel_name: "lark_main".to_string(),
        message: Message {
            id: lark_message["event"]["message"]["message_id"].as_str().unwrap().to_string(),
            platform: PlatformType::Lark,
            message_type: MessageType::Text,
            content: lark_message["event"]["message"]["content"]
                .as_str()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|v| v["text"].as_str().map(|s| s.to_string()))
                .unwrap(),
            sender_id: lark_message["event"]["sender"]["sender_id"]["open_id"]
                .as_str()
                .unwrap()
                .to_string(),
            sender_name: None,
            channel_id: lark_message["event"]["message"]["chat_id"]
                .as_str()
                .unwrap()
                .to_string(),
            timestamp: chrono::Utc::now(),
            reply_to: None,
            metadata: serde_json::json!({
                "event_type": "im.message.receive_v1",
                "chat_type": lark_message["event"]["message"]["chat_type"].as_str(),
            }),
        },
    };
    
    match &event {
        ChannelEvent::MessageReceived { channel_name, message } => {
            println!("Channel: {}", channel_name);
            println!("Message ID: {}", message.id);
            println!("Platform: {:?}", message.platform);
            println!("Sender: {}", message.sender_id);
            println!("Content: {}", message.content);
        }
        _ => panic!("Unexpected event type"),
    }
    
    println!("\n✅ Channel event creation test passed");
}

#[tokio::test]
async fn test_message_deduplication() {
    println!("\n=== Testing Message Deduplication ===\n");
    
    use beebotos_agents::deduplicator::MessageDeduplicator;
    
    let dedup = MessageDeduplicator::new(3600);
    
    let message_id = "om_test_123";
    let platform = PlatformType::Lark;
    
    // First occurrence - should not be duplicate
    let is_dup1 = dedup.is_duplicate(platform, message_id).await;
    assert!(!is_dup1, "First message should not be duplicate");
    println!("First message: not duplicate ✓");
    
    // Second occurrence - should be duplicate
    let is_dup2 = dedup.is_duplicate(platform, message_id).await;
    assert!(is_dup2, "Second message should be duplicate");
    println!("Second message: duplicate detected ✓");
    
    // Different platform, same ID - should not be duplicate
    let is_dup3 = dedup.is_duplicate(PlatformType::Telegram, message_id).await;
    assert!(!is_dup3, "Different platform should not be duplicate");
    println!("Different platform: not duplicate ✓");
    
    println!("\n✅ Message deduplication test passed");
}

#[tokio::test]
async fn test_lark_factory_config_validation() {
    println!("\n=== Testing Lark Factory Config Validation ===\n");
    
    let factory = LarkChannelFactory::new();
    
    // Valid config
    let valid_config = json!({
        "app_id": "cli_a92ba86d9978dbcd",
        "app_secret": "yUxrnnPdqgtJJAJJqi3KChn2v7PQuJo5"
    });
    
    assert!(factory.validate_config(&valid_config), "Valid config should pass");
    println!("Valid config: ✓");
    
    // Invalid - missing app_secret
    let invalid_config = json!({
        "app_id": "cli_a92ba86d9978dbcd"
    });
    
    assert!(!factory.validate_config(&invalid_config), "Missing app_secret should fail");
    println!("Missing app_secret: correctly rejected ✓");
    
    // Invalid - empty values
    let empty_config = json!({
        "app_id": "",
        "app_secret": ""
    });
    
    assert!(!factory.validate_config(&empty_config), "Empty values should fail");
    println!("Empty values: correctly rejected ✓");
    
    println!("\n✅ Lark factory config validation test passed");
}
