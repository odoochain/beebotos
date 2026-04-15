//! Capability system tests

use beebotos_kernel::capabilities::levels::{DecayRate, DecayingCapability};
use beebotos_kernel::capabilities::{CapabilityLevel, CapabilitySet};

#[test]
fn test_capability_levels() {
    assert!(CapabilityLevel::L10SystemAdmin > CapabilityLevel::L0LocalCompute);
    assert!(CapabilityLevel::L5SpawnLimited > CapabilityLevel::L4NetworkIn);
}

#[test]
fn test_capability_set_standard() {
    let set = CapabilitySet::standard();

    assert!(set.has(CapabilityLevel::L1FileRead));
    assert!(set.has(CapabilityLevel::L3NetworkOut));
    // Standard set includes up to L5SpawnLimited
    assert!(set.has(CapabilityLevel::L5SpawnLimited));
    assert!(set.has_permission("compute"));
}

#[test]
fn test_capability_set_full() {
    let set = CapabilitySet::full();

    assert!(set.has(CapabilityLevel::L10SystemAdmin));
    assert!(set.has_permission("*"));
}

#[test]
fn test_capability_set_empty() {
    let set = CapabilitySet::empty();

    assert!(!set.has(CapabilityLevel::L1FileRead));
    assert!(!set.has_permission("compute"));
}

#[test]
fn test_capability_with_permission() {
    let caps = CapabilitySet::standard().with_permission("custom:action");

    assert!(caps.has(CapabilityLevel::L3NetworkOut));
    assert!(caps.has_permission("compute"));
    assert!(caps.has_permission("custom:action"));
}

#[test]
fn test_capability_expiration() {
    // Test empty with expiration
    let caps = CapabilitySet::empty().with_expiration(0); // Expired

    assert!(caps.is_expired());
    assert!(!caps.has(CapabilityLevel::L0LocalCompute));

    // Test standard with expiration
    let caps2 = CapabilitySet::standard().with_expiration(0); // Expired

    assert!(caps2.is_expired());
    assert!(!caps2.has(CapabilityLevel::L1FileRead));
}

#[test]
fn test_capability_intersection() {
    let caps1 = CapabilitySet::standard();
    let caps2 = CapabilitySet::full();

    let intersection = caps1.intersect(&caps2);
    assert_eq!(intersection.max_level, caps1.max_level);
}

#[test]
fn test_capability_verify() {
    let caps = CapabilitySet::standard();

    assert!(caps.verify(CapabilityLevel::L1FileRead).is_ok());
    assert!(caps.verify(CapabilityLevel::L10SystemAdmin).is_err());
}

#[test]
fn test_decaying_capability() {
    let cap = DecayingCapability::new(CapabilityLevel::L5SpawnLimited, DecayRate::Fast);

    assert_eq!(cap.current_level(), CapabilityLevel::L5SpawnLimited);
}
