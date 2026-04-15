//! Cognition Module Integration Tests
//!
//! Tests for the integrated cognitive system functionality.

use beebotos_brain::{
    DecisionContext, DecisionOption, RiskLevel, SensoryModality, SocialBrainApi, TimeHorizon,
};

// =============================================================================
// Perception Tests
// =============================================================================

#[test]
fn test_perceptual_input_creation() {
    let api = SocialBrainApi::new();
    let input = api.create_text_input("Hello world", "user");

    assert_eq!(input.modality, SensoryModality::Text);
    assert_eq!(input.source, "user");
    assert!(input.confidence > 0.0);
}

#[test]
fn test_process_perception() {
    let api = SocialBrainApi::new();
    let input = api.create_text_input("Test input", "test");

    let _percept = api.process_perception(input);
    // May be None if confidence is below threshold
    // Just verify it doesn't panic
}

// =============================================================================
// Working Memory Tests
// =============================================================================

#[test]
fn test_working_memory_operations() {
    let mut api = SocialBrainApi::new();

    // Add item
    api.add_to_working_memory("key1", serde_json::json!("value1"), 0.8);

    // Retrieve
    let item = api.get_from_working_memory("key1");
    assert!(item.is_some());
    assert_eq!(item.unwrap().key, "key1");

    // Non-existent key
    let missing = api.get_from_working_memory("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_working_memory_multiple_items() {
    let mut api = SocialBrainApi::new();

    api.add_to_working_memory("key1", serde_json::json!(1), 0.5);
    api.add_to_working_memory("key2", serde_json::json!(2), 0.7);
    api.add_to_working_memory("key3", serde_json::json!(3), 0.9);

    let stats = api.stats();
    assert!(stats.working_memory_items >= 3);
}

// =============================================================================
// Belief Tests
// =============================================================================

#[test]
fn test_belief_management() {
    let mut api = SocialBrainApi::new();

    // Add beliefs
    api.add_belief("The sky is blue", 0.95);
    api.add_belief("Water is wet", 0.99);

    // Check beliefs count
    let beliefs = api.get_beliefs();
    assert_eq!(beliefs.len(), 2);

    // Check stats
    let stats = api.stats();
    assert_eq!(stats.beliefs_count, 2);
}

#[test]
fn test_belief_confidence() {
    let mut api = SocialBrainApi::new();

    api.add_belief("High confidence belief", 0.9);
    api.add_belief("Low confidence belief", 0.3);

    let beliefs = api.get_beliefs();
    assert_eq!(beliefs.len(), 2);

    // Check that beliefs have appropriate activation (confidence)
    // Both beliefs should have activation >= 0.3 (the lower confidence)
    assert!(beliefs.iter().all(|b| b.activation >= 0.3));
}

// =============================================================================
// Decision Making Tests
// =============================================================================

#[test]
fn test_decision_engine_creation() {
    let api = SocialBrainApi::new();

    let stats = api.stats();
    // Should have default strategies
    assert!(stats.decision_strategies > 0);
}

#[test]
fn test_decision_making() {
    let api = SocialBrainApi::new();

    let option1 = DecisionOption {
        id: "opt1".to_string(),
        description: "Safe option".to_string(),
        expected_outcomes: vec![],
        resource_requirements: Default::default(),
        risk_level: RiskLevel::Low,
        time_horizon: TimeHorizon::ShortTerm,
    };

    let option2 = DecisionOption {
        id: "opt2".to_string(),
        description: "Risky option".to_string(),
        expected_outcomes: vec![],
        resource_requirements: Default::default(),
        risk_level: RiskLevel::High,
        time_horizon: TimeHorizon::ShortTerm,
    };

    let context = DecisionContext {
        situation: "Test situation".to_string(),
        available_options: vec![option1, option2],
        constraints: vec![],
        objectives: vec![],
        time_pressure: 0.5,
    };

    let decision = api.decide(&context);
    assert!(decision.is_some());
}

#[test]
fn test_decision_empty_options() {
    let api = SocialBrainApi::new();

    let context = DecisionContext {
        situation: "No options".to_string(),
        available_options: vec![],
        constraints: vec![],
        objectives: vec![],
        time_pressure: 0.0,
    };

    let decision = api.decide(&context);
    assert!(decision.is_none());
}

#[test]
fn test_compare_strategies() {
    let api = SocialBrainApi::new();

    let option = DecisionOption {
        id: "opt1".to_string(),
        description: "Test".to_string(),
        expected_outcomes: vec![],
        resource_requirements: Default::default(),
        risk_level: RiskLevel::Medium,
        time_horizon: TimeHorizon::ShortTerm,
    };

    let context = DecisionContext {
        situation: "Compare test".to_string(),
        available_options: vec![option],
        constraints: vec![],
        objectives: vec![],
        time_pressure: 0.5,
    };

    let comparisons = api.compare_strategies(&context);
    assert!(!comparisons.is_empty());
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_cognitive_pipeline() {
    let mut api = SocialBrainApi::new();

    // 1. Process perception
    let input = api.create_text_input("Important information", "sensor");
    let _percept = api.process_perception(input);

    // 2. Add to working memory
    api.add_to_working_memory("important_info", serde_json::json!("data"), 0.9);

    // 3. Form belief
    api.add_belief("Information is valuable", 0.8);

    // 4. Check stats
    let stats = api.stats();
    assert!(stats.working_memory_items > 0);
    assert!(stats.beliefs_count > 0);
}

#[test]
fn test_stats_comprehensive() {
    let mut api = SocialBrainApi::new();

    // Initial state
    let initial_stats = api.stats();

    // Add various cognitive elements
    api.add_to_working_memory("wm1", serde_json::json!(1), 0.5);
    api.add_belief("belief1", 0.9);
    api.set_goal("Test goal", 0.8).unwrap();

    // Process stimulus to add memory
    let _ = api.process_stimulus("Test stimulus");

    // Check updated stats
    let stats = api.stats();
    assert!(stats.working_memory_items >= initial_stats.working_memory_items + 1);
    assert!(stats.beliefs_count >= initial_stats.beliefs_count + 1);
    assert!(stats.active_goals >= initial_stats.active_goals + 1);
    assert!(stats.memory_items >= initial_stats.memory_items);
    assert!(stats.decision_strategies > 0);
}
