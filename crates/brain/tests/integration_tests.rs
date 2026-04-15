//! Integration Tests
//!
//! Tests for complete workflows and component interactions.

use beebotos_brain::{
    Action, ApiConfig, BrainConfig, ConfigBuilder, ConfigProfile, DecisionContext, DecisionOption,
    Goal, MemoryQuery, MemoryType, OceanProfile, Pad, RiskLevel, SocialBrainApi, TimeHorizon,
};

// =============================================================================
// API Integration Tests
// =============================================================================

#[test]
fn test_api_full_workflow() {
    let mut api = SocialBrainApi::new();

    // 1. Process initial stimulus
    let response = api
        .process_stimulus("Hello, I'm excited to start working!")
        .unwrap();
    assert!(!response.memory_id.is_empty());
    assert!(response.emotional_change.pleasure > 0.0); // Positive sentiment

    // 2. Set goals
    let goal_id = api.set_goal("Complete project", 0.9).unwrap();
    assert!(!goal_id.is_empty());

    // 3. Process more stimuli
    api.process_stimulus("Urgent: deadline approaching!")
        .unwrap();

    // 4. Check emotional state changed
    let emotion = api.current_emotion();
    assert!(emotion.arousal > 0.0); // High arousal from urgency

    // 5. Query memory
    let query = MemoryQuery::new("project").with_types(vec![MemoryType::ShortTerm]);
    let results = api.query_memory(&query).unwrap();
    assert!(!results.is_empty());

    // 6. Check stats
    let stats = api.stats();
    assert!(stats.active_goals >= 1);
    assert!(stats.memory_items >= 2);
}

#[test]
fn test_api_with_disabled_features() {
    let config = ApiConfig {
        memory_enabled: false,
        emotion_enabled: false,
        learning_enabled: false,
        personality_influence: 0.0,
    };

    let mut api = SocialBrainApi::with_config(config);

    // Processing should still work but without those features
    let response = api.process_stimulus("Test").unwrap();
    assert!(response.memory_id.is_empty()); // Memory disabled

    let emotion = api.current_emotion();
    assert!(emotion.pleasure.abs() < 0.001); // Should be neutral
}

#[test]
fn test_api_emotion_personality_interaction() {
    let mut api = SocialBrainApi::new();

    // Set personality to high neuroticism (amplifies negative emotions)
    let mut personality = OceanProfile::balanced();
    personality.neuroticism = 0.9;
    api.set_personality(personality);

    // Apply negative stimulus
    let initial_emotion = api.current_pad();
    api.apply_emotional_stimulus(Pad::new(-0.5, 0.5, 0.0), 0.8);
    let after_emotion = api.current_pad();

    // Negative emotion should be amplified
    assert!(after_emotion.pleasure < initial_emotion.pleasure);
}

// =============================================================================
// Memory Integration Tests
// =============================================================================

#[test]
fn test_memory_consolidation_workflow() {
    let mut api = SocialBrainApi::new();

    // Store multiple memories
    for i in 0..10 {
        api.store_memory(&format!("Memory item {}", i), 0.7)
            .unwrap();
    }

    let stats_before = api.stats();

    // Consolidate memories
    let consolidated = api.consolidate_memories().unwrap();

    let stats_after = api.stats();

    // Stats should reflect changes
    assert!(stats_after.memory_items >= stats_before.memory_items);
}

#[test]
fn test_memory_query_with_filters() {
    let mut api = SocialBrainApi::new();

    // Store memories with different content
    api.store_memory("Important meeting notes", 0.9).unwrap();
    api.store_memory("Shopping list", 0.3).unwrap();
    api.store_memory("Project deadline tomorrow", 0.8).unwrap();

    // Query for important items
    let query = MemoryQuery::new("important").with_min_importance(0.8);
    let results = api.query_memory(&query).unwrap();

    // Should find high importance items
    assert!(results.total_count() > 0);
}

// =============================================================================
// Goal Integration Tests
// =============================================================================

#[test]
fn test_goal_priority_management() {
    let mut api = SocialBrainApi::new();

    // Set multiple goals with different priorities
    api.set_goal("Low priority task", 0.3).unwrap();
    api.set_goal("Critical task", 0.9).unwrap();
    api.set_goal("Medium priority task", 0.6).unwrap();

    // Check goals are sorted by priority
    let goals = api.current_goals();
    assert_eq!(goals.len(), 3);
    assert!(goals[0].priority >= goals[1].priority);
    assert!(goals[1].priority >= goals[2].priority);

    // Form intention should use highest priority goal
    let intention = api.form_intention();
    assert!(intention.is_some());
}

#[test]
fn test_goal_and_memory_integration() {
    let mut api = SocialBrainApi::new();

    // Store information related to goal
    api.store_memory("Goal context: Complete project X by Friday", 0.8)
        .unwrap();

    // Set goal
    api.set_goal("Complete project X", 0.9).unwrap();

    // Query memory for goal-related information
    let query = MemoryQuery::new("project X");
    let results = api.query_memory(&query).unwrap();

    assert!(results.total_count() > 0);
}

// =============================================================================
// Configuration Integration Tests
// =============================================================================

#[test]
fn test_config_builder_workflow() {
    let config = ConfigBuilder::new().build().unwrap();

    // Config should be valid
    assert!(config.memory.enabled);
}

#[test]
fn test_config_profile_lightweight() {
    let config = ConfigProfile::Lightweight.apply().build().unwrap();

    // Lightweight profile should have reduced features
    assert!(!config.features.learning);
    assert!(!config.features.social);
    assert!(!config.parallel.enabled);
}

#[test]
fn test_config_profile_high_performance() {
    let config = ConfigProfile::HighPerformance.apply().build().unwrap();

    // High performance should enable parallel processing
    assert!(config.parallel.enabled);
    assert!(config.memory.stm_capacity > 7);
}

#[test]
fn test_config_validation() {
    let result = ConfigBuilder::new().build();

    assert!(result.is_ok());
}

// =============================================================================
// Cognitive Integration Tests
// =============================================================================

#[test]
fn test_perception_and_memory_integration() {
    let mut api = SocialBrainApi::new();

    // Create perceptual input
    let input = api.create_text_input("Test input data", "sensor_1");

    // Process perception
    let percept = api.process_perception(input);

    // Perception may or may not be processed depending on attention
    // Just verify it doesn't panic
    let _ = percept;
}

#[test]
fn test_decision_engine_integration() {
    let api = SocialBrainApi::new();

    // Create decision context
    let context = DecisionContext {
        available_options: vec![
            DecisionOption {
                id: "option_1".to_string(),
                description: "Safe option".to_string(),
                expected_outcomes: vec![],
                risk_level: RiskLevel::Low,
                time_horizon: TimeHorizon::ShortTerm,
                resource_requirements: Default::default(),
            },
            DecisionOption {
                id: "option_2".to_string(),
                description: "Risky option".to_string(),
                expected_outcomes: vec![],
                risk_level: RiskLevel::High,
                time_horizon: TimeHorizon::Immediate,
                resource_requirements: Default::default(),
            },
        ],
        situation: "Test decision".to_string(),
        constraints: vec![],
        objectives: vec![],
        time_pressure: 0.5,
    };

    // Make decision
    let decision = api.decide(&context);
    assert!(decision.is_some());

    // Compare strategies
    let comparisons = api.compare_strategies(&context);
    assert!(!comparisons.is_empty());
}

#[test]
fn test_working_memory_and_beliefs() {
    let mut api = SocialBrainApi::new();

    // Add to working memory
    use serde_json::json;
    api.add_to_working_memory("fact_1", json!({"value": 42}), 0.9);
    api.add_to_working_memory("fact_2", json!({"status": "active"}), 0.8);

    // Retrieve from working memory
    let item1 = api.get_from_working_memory("fact_1");
    assert!(item1.is_some());

    // Add beliefs
    api.add_belief("It is daytime", 0.95);
    api.add_belief("Temperature is warm", 0.7);

    // Get beliefs
    let beliefs = api.get_beliefs();
    assert_eq!(beliefs.len(), 2);

    // Check stats reflect working memory
    let stats = api.stats();
    assert_eq!(stats.working_memory_items, 4); // 2 memory items + 2 beliefs
    assert_eq!(stats.beliefs_count, 2);
}

// =============================================================================
// Cross-Module Integration Tests
// =============================================================================

#[test]
fn test_emotion_memory_personality_interaction() {
    let mut api = SocialBrainApi::new();

    // Set personality
    let personality = OceanProfile {
        openness: 0.8,
        conscientiousness: 0.7,
        extraversion: 0.6,
        agreeableness: 0.5,
        neuroticism: 0.3, // Low neuroticism = stable emotions
    };
    api.set_personality(personality);

    // Store emotional memory
    api.apply_emotional_stimulus(Pad::new(0.8, 0.5, 0.6), 0.9); // Positive event
    api.store_memory("Had a great success today!", 0.9).unwrap();

    // Process negative stimulus (should be less impactful due to personality)
    let emotion_before = api.current_emotion();
    api.apply_emotional_stimulus(Pad::new(-0.3, 0.2, 0.1), 0.5);
    let emotion_after = api.current_emotion();

    // Personality should moderate emotional swings
    let change = (emotion_after.pleasure - emotion_before.pleasure).abs();
    assert!(change < 0.5); // Change should be moderate
}

#[test]
fn test_stimulus_processing_emotional_response() {
    let mut api = SocialBrainApi::new();

    // Positive stimulus
    let response1 = api
        .process_stimulus("Great news! Everything is working perfectly!")
        .unwrap();
    assert!(response1.emotional_change.pleasure > 0.0);

    // Negative stimulus
    let response2 = api
        .process_stimulus("Terrible failure, everything is broken!")
        .unwrap();
    assert!(response2.emotional_change.pleasure < 0.0);

    // Urgent stimulus
    let response3 = api
        .process_stimulus("URGENT: Critical issue needs immediate attention!")
        .unwrap();
    assert!(response3.emotional_change.arousal > 0.5); // High arousal
}

#[test]
fn test_api_stats_consistency() {
    let mut api = SocialBrainApi::new();

    let initial_stats = api.stats();

    // Add some data
    api.store_memory("Test memory", 0.5).unwrap();
    api.set_goal("Test goal", 0.7).unwrap();
    api.apply_emotional_stimulus(Pad::new(0.5, 0.3, 0.2), 0.6);

    let final_stats = api.stats();

    // Stats should reflect changes
    assert!(final_stats.memory_items >= initial_stats.memory_items);
    assert!(final_stats.active_goals > initial_stats.active_goals);
    assert!(final_stats.has_network == initial_stats.has_network); // Network status unchanged
}

// =============================================================================
// Error Handling Integration Tests
// =============================================================================

#[test]
fn test_api_error_handling() {
    let mut api = SocialBrainApi::new();

    // Invalid priority should return error
    let result = api.set_goal("Test", 1.5);
    assert!(result.is_err());

    // Invalid priority (negative)
    let result = api.set_goal("Test", -0.1);
    assert!(result.is_err());

    // Valid priority should succeed
    let result = api.set_goal("Test", 0.5);
    assert!(result.is_ok());
}

#[test]
fn test_empty_input_handling() {
    let mut api = SocialBrainApi::new();

    // Empty stimulus should return error
    let result = api.process_stimulus("");
    assert!(result.is_err());
}

// =============================================================================
// Long-Running Workflow Tests
// =============================================================================

#[test]
fn test_extended_interaction_session() {
    let mut api = SocialBrainApi::new();

    // Simulate a conversation/session
    let inputs = vec![
        "Hello there!",
        "I'm working on an important project",
        "The deadline is tomorrow",
        "I'm feeling stressed about it",
        "But I think I can make it",
        "Thanks for listening",
    ];

    for (i, input) in inputs.iter().enumerate() {
        let response = api.process_stimulus(input).unwrap();

        // Every response should have a memory ID
        assert!(!response.memory_id.is_empty(), "Failed at input {}", i);
    }

    // Should have accumulated memories
    let stats = api.stats();
    assert!(stats.memory_items >= inputs.len());

    // Should have goals from context
    assert!(stats.active_goals > 0);
}

#[test]
fn test_memory_query_after_many_stores() {
    let mut api = SocialBrainApi::new();

    // Store many memories
    for i in 0..20 {
        let content = format!("Memory number {} with some keywords", i);
        api.store_memory(&content, 0.6).unwrap();
    }

    // Query should still work efficiently
    let query = MemoryQuery::new("keywords").with_limit(10);
    let results = api.query_memory(&query).unwrap();

    // Should find matches
    assert!(results.total_count() > 0);
}
