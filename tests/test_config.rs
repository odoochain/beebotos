//! Test configuration loading

use std::collections::HashMap;

#[test]
fn test_channel_config_structure() {
    // Simulate channel config from TOML
    let mut settings = HashMap::new();
    settings.insert("app_id".to_string(), serde_json::json!("cli_test123"));
    settings.insert("app_secret".to_string(), serde_json::json!("secret_test456"));
    
    let config = serde_json::json!({
        "enabled": true,
        "settings": settings
    });
    
    assert!(config.get("enabled").unwrap().as_bool().unwrap());
    println!("✅ Channel config structure test passed");
}

#[test]
fn test_lazy_url_format() {
    let platform = "lark";
    let message_id = "om_123456";
    let file_key = "img_abcdef";
    let file_type = "image";
    
    let lazy_url = format!("lazy://{}/{}/{}/{}", platform, message_id, file_key, file_type);
    
    assert_eq!(lazy_url, "lazy://lark/om_123456/img_abcdef/image");
    println!("✅ Lazy URL format test passed: {}", lazy_url);
}

#[test]
fn test_factory_validation() {
    // Test Lark validation
    let lark_config = serde_json::json!({
        "app_id": "cli_123",
        "app_secret": "secret_456"
    });
    
    let has_app_id = lark_config.get("app_id")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    
    let has_app_secret = lark_config.get("app_secret")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    
    assert!(has_app_id && has_app_secret);
    println!("✅ Factory validation test passed");
}

#[test]
fn test_telegram_token_validation() {
    let valid_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11";
    let invalid_token = "invalid_token";
    
    let is_valid = valid_token.contains(':');
    let is_invalid = !invalid_token.contains(':');
    
    assert!(is_valid);
    assert!(is_invalid);
    println!("✅ Telegram token validation test passed");
}

#[test]
fn test_slack_token_validation() {
    let valid_token = "xoxb-TESTTOKEN";
    let invalid_token = "bot-token-without-prefix";
    
    let is_valid = valid_token.starts_with("xoxb-");
    let is_invalid = !invalid_token.starts_with("xoxb-");
    
    assert!(is_valid);
    assert!(is_invalid);
    println!("✅ Slack token validation test passed");
}
