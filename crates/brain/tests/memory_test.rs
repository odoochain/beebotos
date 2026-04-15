//! Memory System Tests
//!
//! Comprehensive tests for multi-modal memory system.

use beebotos_brain::{
    EpisodicMemory, Location, MemoryQuery, MemoryType, Priority, ProceduralMemory, RelationType,
    SemanticMemory, ShortTermMemory, Step, UnifiedMemory,
};

// ============================================================================
// Short-Term Memory Tests
// ============================================================================

#[test]
fn test_short_term_memory_creation() {
    let stm = ShortTermMemory::new();
    assert_eq!(stm.len(), 0);
    assert!(stm.is_empty());
}

#[test]
fn test_short_term_memory_capacity() {
    let mut stm = ShortTermMemory::with_capacity(3);

    // Fill to capacity
    stm.push("item1");
    stm.push("item2");
    stm.push("item3");

    assert_eq!(stm.len(), 3);

    // Adding another should evict
    let evicted = stm.push("item4");
    assert!(evicted.is_some());
    assert_eq!(stm.len(), 3); // Still at capacity
}

#[test]
fn test_short_term_memory_priority() {
    let mut stm = ShortTermMemory::with_capacity(3);

    stm.push_with_priority("low", Priority::Low);
    stm.push_with_priority("critical", Priority::Critical);
    stm.push_with_priority("medium", Priority::Medium);

    // Add another low priority - should evict the existing low
    let evicted = stm.push_with_priority("new_low", Priority::Low);

    assert!(evicted.is_some());
    // Critical should still be there
    let items: Vec<_> = stm.items().iter().map(|i| i.content.clone()).collect();
    assert!(items.contains(&"critical".to_string()));
}

#[test]
fn test_short_term_memory_rehearsal() {
    let mut stm = ShortTermMemory::new();
    stm.push("test content");

    let id = stm.items()[0].id.clone();

    // Rehearse should succeed
    assert!(stm.rehearse(&id).is_ok());
    assert_eq!(stm.rehearsal_count(&id), 1);

    // Rehearse again
    assert!(stm.rehearse(&id).is_ok());
    assert_eq!(stm.rehearsal_count(&id), 2);
}

#[test]
fn test_short_term_memory_retrieve() {
    let mut stm = ShortTermMemory::new();
    stm.push("hello world");
    stm.push("foo bar");
    stm.push("hello rust");

    let results = stm.retrieve("hello");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_short_term_memory_ready_for_consolidation() {
    let mut stm = ShortTermMemory::new();
    stm.push("item1");
    stm.push("item2");

    let id1 = stm.items()[0].id.clone();

    // Rehearse multiple times
    for _ in 0..5 {
        let _ = stm.rehearse(&id1);
    }

    let ready = stm.ready_for_consolidation(3);
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].content, "item1");
}

// ============================================================================
// Episodic Memory Tests
// ============================================================================

#[test]
fn test_episodic_memory_creation() {
    let em = EpisodicMemory::new();
    assert!(em.is_empty());
    assert_eq!(em.len(), 0);
}

#[test]
fn test_episodic_memory_encode() {
    let mut em = EpisodicMemory::new();

    let id = em.encode(
        "Met Alice at the coffee shop",
        1000,
        Some(Location {
            name: "Coffee Shop".to_string(),
            coordinates: Some((40.7, -74.0)),
        }),
    );

    assert!(!id.is_empty());
    assert_eq!(em.len(), 1);
}

#[test]
fn test_episodic_memory_time_range_query() {
    let mut em = EpisodicMemory::new();

    em.encode("Morning meeting", 100, None);
    em.encode("Lunch break", 500, None);
    em.encode("Evening workout", 1000, None);

    let results = em.query_time_range(200, 800);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].what, "Lunch break");
}

#[test]
fn test_episodic_memory_location_query() {
    let mut em = EpisodicMemory::new();

    em.encode(
        "At office",
        100,
        Some(Location {
            name: "Office".to_string(),
            coordinates: None,
        }),
    );

    em.encode(
        "At home",
        200,
        Some(Location {
            name: "Home".to_string(),
            coordinates: None,
        }),
    );

    let results = em.query_location("Office");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].what, "At office");
}

#[test]
fn test_episodic_memory_search() {
    let mut em = EpisodicMemory::new();

    em.encode("Learned about Rust programming", 100, None);
    em.encode("Had lunch with team", 200, None);
    em.encode("Rust meetup in evening", 300, None);

    let results = em.search("rust");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_episodic_memory_importance() {
    let mut em = EpisodicMemory::new();

    let id = em.encode("Important event", 100, None);
    em.set_importance(100, 0.9);

    // The importance should be set (verify through consolidation)
    let episodes = em.query_time_range(100, 100);
    assert_eq!(episodes[0].importance, 0.9);
}

#[test]
fn test_episodic_memory_consolidation() {
    let mut em = EpisodicMemory::new();

    em.encode("Event 1", 100, None);
    em.encode("Event 2", 200, None);
    em.encode("Event 3", 300, None);

    let summary = em.consolidate((100, 300));

    assert!(summary.is_some());
    assert!(summary.unwrap().what.contains("3 events"));
}

// ============================================================================
// Semantic Memory Tests
// ============================================================================

#[test]
fn test_semantic_memory_creation() {
    let sm = SemanticMemory::new();
    assert!(sm.is_empty());
}

#[test]
fn test_semantic_memory_learn_concept() {
    let mut sm = SemanticMemory::new();

    let id = sm.learn_concept(
        "Rust",
        "A systems programming language",
        "Programming Language",
    );

    assert!(!id.is_empty());
    assert_eq!(sm.len(), 1);
}

#[test]
fn test_semantic_memory_get() {
    let mut sm = SemanticMemory::new();

    let id = sm.learn_concept(
        "Python",
        "A high-level programming language",
        "Programming Language",
    );

    let concept = sm.get(&id);
    assert!(concept.is_some());
    assert_eq!(concept.unwrap().name, "Python");
}

#[test]
fn test_semantic_memory_find_by_name() {
    let mut sm = SemanticMemory::new();

    sm.learn_concept("Java", "Object-oriented language", "Language");

    let found = sm.find_by_name("Java");
    assert!(found.is_some());
    assert_eq!(found.unwrap().definition, "Object-oriented language");
}

#[test]
fn test_semantic_memory_add_relation() {
    let mut sm = SemanticMemory::new();

    let id1 = sm.learn_concept("Animal", "A living organism", "Biology");
    let id2 = sm.learn_concept("Dog", "A domesticated mammal", "Biology");

    let result = sm.add_relation(&id1, &id2, RelationType::IsA, 1.0);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_memory_find_similar() {
    let mut sm = SemanticMemory::new();

    let id1 = sm.learn_concept("Cat", "A feline", "Animal");
    sm.learn_concept("Dog", "A canine", "Animal");
    sm.learn_concept("Car", "A vehicle", "Machine");

    let similar = sm.find_similar(&id1, 0.5);
    // Should find Dog (same category)
    assert_eq!(similar.len(), 1);
    assert_eq!(similar[0].name, "Dog");
}

#[test]
fn test_semantic_memory_by_category() {
    let mut sm = SemanticMemory::new();

    sm.learn_concept("Lion", "Big cat", "Animal");
    sm.learn_concept("Tiger", "Big cat", "Animal");
    sm.learn_concept("Python", "Programming language", "Technology");

    let animals = sm.by_category("Animal");
    assert_eq!(animals.len(), 2);
}

// ============================================================================
// Memory Query Tests
// ============================================================================

#[test]
fn test_memory_query_builder() {
    let query = MemoryQuery::new("blockchain")
        .with_limit(10)
        .with_min_importance(0.5)
        .with_location("database");

    assert_eq!(query.query, "blockchain");
    assert_eq!(query.limit, 10);
    assert_eq!(query.min_importance, 0.5);
    assert_eq!(query.location, Some("database".to_string()));
}

#[test]
fn test_memory_query_with_types() {
    let query =
        MemoryQuery::new("test").with_types(vec![MemoryType::Episodic, MemoryType::Semantic]);

    assert_eq!(query.memory_types.len(), 2);
    assert!(query.memory_types.contains(&MemoryType::Episodic));
    assert!(query.memory_types.contains(&MemoryType::Semantic));
}

#[test]
fn test_memory_query_with_emotion() {
    let query = MemoryQuery::new("happy memory").with_emotion("joy", 0.5, 1.0);

    assert!(query.emotional_filter.is_some());
    let filter = query.emotional_filter.unwrap();
    assert_eq!(filter.emotion, "joy");
    assert_eq!(filter.min_intensity, 0.5);
}

// ============================================================================
// Unified Memory Tests
// ============================================================================

#[test]
fn test_unified_memory_creation() {
    let memory = UnifiedMemory::new();
    assert!(memory.short_term.is_empty());
    assert!(memory.episodic.is_empty());
    assert!(memory.semantic.is_empty());
}

#[test]
fn test_unified_memory_consolidate() {
    let mut memory = UnifiedMemory::new();

    // Add items to STM with rehearsals
    memory
        .short_term
        .push_with_priority("important fact", Priority::High);
    let id = memory.short_term.items()[0].id.clone();

    for _ in 0..5 {
        let _ = memory.short_term.rehearse(&id);
    }

    let count = memory.consolidate().unwrap();
    assert_eq!(count, 1);
}

// ============================================================================
// Procedural Memory Tests
// ============================================================================

#[test]
fn test_procedural_memory_creation() {
    let pm = ProceduralMemory::new();
    assert!(pm.is_empty());
}

#[test]
fn test_procedural_memory_learn() {
    let mut pm = ProceduralMemory::new();

    let steps = vec![Step {
        id: "1".to_string(),
        action: "Gather ingredients".to_string(),
        expected_outcome: "Have all items ready".to_string(),
        on_success: Some("2".to_string()),
        on_failure: None,
    }];

    let id = pm.learn("Make coffee", steps);
    assert!(!id.is_empty());
}

#[test]
fn test_procedural_memory_record_execution() {
    let mut pm = ProceduralMemory::new();

    let steps = vec![Step {
        id: "1".to_string(),
        action: "Test".to_string(),
        expected_outcome: "Success".to_string(),
        on_success: None,
        on_failure: None,
    }];

    let id = pm.learn("Test procedure", steps);

    pm.record_execution(&id, true);
    pm.record_execution(&id, true);
    pm.record_execution(&id, false);

    let proc = pm.get(&id).unwrap();
    assert_eq!(proc.execution_count, 3);
    // Success rate should be around 2/3
    assert!(proc.success_rate > 0.6 && proc.success_rate < 0.7);
}

#[test]
fn test_procedural_memory_search() {
    let mut pm = ProceduralMemory::new();

    let steps = vec![Step {
        id: "1".to_string(),
        action: "Brew".to_string(),
        expected_outcome: "Coffee ready".to_string(),
        on_success: None,
        on_failure: None,
    }];

    pm.learn("Make coffee", steps);

    let results = pm.search("coffee");
    assert_eq!(results.len(), 1);
}
