//! Session Unit Tests
//!
//! Tests for agent session management.

use agents::session::{SessionKey, SessionType};

#[test]
fn test_session_key_parsing() {
    let key = SessionKey::parse("agent:test123:session:a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
    
    assert_eq!(key.agent_id, "test123");
    assert!(matches!(key.session_type, SessionType::Session));
    assert_eq!(key.depth, 0);
}

#[test]
fn test_session_key_subagent() {
    let key = SessionKey::parse("agent:parent:subagent:child-uuid").unwrap();
    
    assert_eq!(key.agent_id, "parent");
    assert!(matches!(key.session_type, SessionType::Subagent));
}

#[test]
fn test_session_key_cron() {
    let key = SessionKey::parse("agent:scheduler:cron:task-uuid").unwrap();
    
    assert_eq!(key.agent_id, "scheduler");
    assert!(matches!(key.session_type, SessionType::Cron));
}

#[test]
fn test_session_key_webhook() {
    let key = SessionKey::parse("agent:api:webhook:hook-uuid").unwrap();
    
    assert_eq!(key.agent_id, "api");
    assert!(matches!(key.session_type, SessionType::Webhook));
}

#[test]
fn test_invalid_session_key() {
    assert!(SessionKey::parse("invalid").is_err());
    assert!(SessionKey::parse("agent:only").is_err());
    assert!(SessionKey::parse("agent:id:unknown:uuid").is_err());
}

#[test]
fn test_spawn_child() {
    let parent = SessionKey::parse("agent:parent:session:uuid1").unwrap();
    let child = parent.spawn_child().unwrap();
    
    assert_eq!(child.agent_id, "parent");
    assert!(matches!(child.session_type, SessionType::Subagent));
    assert_eq!(child.depth, 1);
}

#[test]
fn test_max_depth() {
    let mut key = SessionKey::parse("agent:root:session:uuid1").unwrap();
    
    // Simulate reaching max depth
    for _ in 0..5 {
        key = key.spawn_child().unwrap();
    }
    
    assert_eq!(key.depth, 5);
    
    // Should fail at depth 6
    assert!(key.spawn_child().is_err());
}

#[test]
fn test_to_path() {
    let key = SessionKey::parse("agent:test:session:uuid-1234").unwrap();
    let path = key.to_path();
    
    assert!(path.contains("test"));
    assert!(path.contains("uuid-1234"));
}
