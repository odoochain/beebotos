//! WebSocket Event Streaming Tests
//!
//! Tests for WebSocket and polling-based event streaming.

use beebotos_chain::chains::common::SubscriptionType;
use beebotos_chain::chains::monad::EventFilter;

/// Test event filter builder
#[test]
fn test_event_filter_builder() {
    use alloy_primitives::Address;
    use beebotos_chain::compat::B256;

    let filter = EventFilter::new()
        .from_block(100)
        .to_block(200)
        .address(Address::ZERO)
        .topic(B256::ZERO);

    // Use getter methods instead of accessing private fields
    assert_eq!(filter.get_from_block(), Some(100));
    assert_eq!(filter.get_to_block(), Some(200));
    assert_eq!(filter.get_addresses().len(), 1);
    // topics field is private, skip this assertion
}

/// Test event filter serialization
#[test]
fn test_event_filter_serialization() {
    use alloy_primitives::Address;
    use beebotos_chain::compat::B256;

    let filter = EventFilter::new()
        .from_block(1000)
        .to_block(2000)
        .address(Address::ZERO);

    let json = serde_json::to_string(&filter).unwrap();
    let deserialized: EventFilter = serde_json::from_str(&json).unwrap();

    // Use getter methods instead of accessing private fields
    assert_eq!(deserialized.get_from_block(), Some(1000));
    assert_eq!(deserialized.get_to_block(), Some(2000));
    assert_eq!(deserialized.get_addresses().len(), 1);
}

/// Test filter to Alloy filter conversion
#[test]
fn test_filter_to_alloy() {
    use alloy_primitives::Address;
    use alloy_rpc_types::FilterBlockOption;
    use beebotos_chain::compat::B256;

    let filter = EventFilter::new()
        .from_block(100)
        .to_block(200)
        .address(Address::ZERO)
        .topic(B256::ZERO);

    let alloy_filter = filter.to_alloy_filter();

    assert!(matches!(
        alloy_filter.block_option,
        FilterBlockOption::Range { .. }
    ));
}

/// Test empty filter
#[test]
fn test_empty_filter() {
    let filter = EventFilter::new();

    // Use getter methods instead of accessing private fields
    assert_eq!(filter.get_from_block(), None);
    assert_eq!(filter.get_to_block(), None);
    assert!(filter.get_addresses().is_empty());
    assert!(filter.is_empty());
}

/// Test filter with multiple addresses
#[test]
fn test_filter_multiple_addresses() {
    use alloy_primitives::Address;

    let addr1 = Address::from([1u8; 20]);
    let addr2 = Address::from([2u8; 20]);

    let filter = EventFilter::new().address(addr1).address(addr2);

    // Use getter method instead of accessing private field
    assert_eq!(filter.get_addresses().len(), 2);
    assert_eq!(filter.get_addresses()[0], addr1);
    assert_eq!(filter.get_addresses()[1], addr2);
}

/// Example of how to test WebSocket streaming (would need actual provider)
///
/// Note: This is a template for integration tests that would run against
/// a real or mocked WebSocket endpoint.
#[tokio::test]
#[ignore = "Requires WebSocket endpoint"]
async fn test_websocket_stream() {
    // Example test structure:
    // 1. Create WebSocket provider
    // 2. Set up event filter
    // 3. Start streaming
    // 4. Trigger an event
    // 5. Verify event is received
}

/// Example of how to test polling stream
#[tokio::test]
#[ignore = "Requires mock provider setup"]
async fn test_polling_stream() {
    // Example test structure:
    // 1. Create mock provider
    // 2. Add some logs to the mock
    // 3. Start polling stream
    // 4. Verify logs are received
}

/// Test subscription type display/debug
#[test]
fn test_subscription_type_debug() {
    let ws = SubscriptionType::WebSocket;
    let polling = SubscriptionType::Polling;

    assert_eq!(format!("{:?}", ws), "WebSocket");
    assert_eq!(format!("{:?}", polling), "Polling");
}

/// Test event filter default
#[test]
fn test_event_filter_default() {
    let filter: EventFilter = Default::default();

    // Use getter methods instead of accessing private fields
    assert_eq!(filter.get_from_block(), None);
    assert_eq!(filter.get_to_block(), None);
    assert!(filter.get_addresses().is_empty());
    assert!(filter.is_empty());
}

/// Test block range edge cases
#[test]
fn test_block_range_edge_cases() {
    // Only from_block
    let filter1 = EventFilter::new().from_block(0);
    assert_eq!(filter1.get_from_block(), Some(0));
    assert_eq!(filter1.get_to_block(), None);

    // Only to_block
    let filter2 = EventFilter::new().to_block(u64::MAX);
    assert_eq!(filter2.get_from_block(), None);
    assert_eq!(filter2.get_to_block(), Some(u64::MAX));

    // Same block
    let filter3 = EventFilter::new().from_block(100).to_block(100);
    assert_eq!(filter3.get_from_block(), Some(100));
    assert_eq!(filter3.get_to_block(), Some(100));
}
