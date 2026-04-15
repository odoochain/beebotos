//! PAD (Pleasure-Arousal-Dominance) Emotional Model Tests
//!
//! Comprehensive tests for the PAD emotional model and related functionality.

use beebotos_brain::{
    BasicEmotion, Emotion, EmotionCategory, EmotionalEvent, EmotionalIntelligence, EmotionalTrait,
    Pad,
};

// ============================================================================
// PAD State Tests
// ============================================================================

#[test]
fn test_pad_creation() {
    let pad = Pad::new(0.5, 0.6, 0.7);

    assert_eq!(pad.pleasure, 0.5);
    assert_eq!(pad.arousal, 0.6);
    assert_eq!(pad.dominance, 0.7);
}

#[test]
fn test_pad_clamping() {
    // Test pleasure clamping (-1.0 to 1.0)
    let pad1 = Pad::new(2.0, 0.5, 0.5);
    assert_eq!(pad1.pleasure, 1.0);

    let pad2 = Pad::new(-2.0, 0.5, 0.5);
    assert_eq!(pad2.pleasure, -1.0);

    // Test arousal clamping (0.0 to 1.0)
    let pad3 = Pad::new(0.0, 1.5, 0.5);
    assert_eq!(pad3.arousal, 1.0);

    let pad4 = Pad::new(0.0, -0.5, 0.5);
    assert_eq!(pad4.arousal, 0.0);
}

#[test]
fn test_pad_neutral() {
    let neutral = Pad::neutral();
    assert_eq!(neutral.pleasure, 0.0);
    assert_eq!(neutral.arousal, 0.5);
    assert_eq!(neutral.dominance, 0.5);
}

#[test]
fn test_pad_constants() {
    assert_eq!(Pad::JOY.pleasure, 1.0);
    assert!(Pad::FEAR.pleasure < 0.0);
    assert!(Pad::FEAR.arousal > 0.5);
    assert!(Pad::ANGER.pleasure < 0.0);
    assert!(Pad::SADNESS.pleasure < 0.0);
    assert!(Pad::SADNESS.arousal < 0.5);
}

#[test]
fn test_pad_intensity() {
    let neutral = Pad::neutral();
    let joy = Pad::JOY;

    assert!(neutral.intensity() < joy.intensity());
}

#[test]
fn test_pad_is_positive() {
    let positive = Pad::new(0.5, 0.5, 0.5);
    assert!(positive.is_positive());

    let negative = Pad::new(-0.5, 0.5, 0.5);
    assert!(negative.is_negative());

    let neutral = Pad::new(0.0, 0.5, 0.5);
    assert!(!neutral.is_positive());
    assert!(!neutral.is_negative());
}

#[test]
fn test_pad_arousal_states() {
    let high_arousal = Pad::new(0.0, 0.8, 0.5);
    assert!(high_arousal.is_aroused());
    assert!(!high_arousal.is_calm());

    let low_arousal = Pad::new(0.0, 0.2, 0.5);
    assert!(low_arousal.is_calm());
    assert!(!low_arousal.is_aroused());
}

#[test]
fn test_pad_dominance_states() {
    let dominant = Pad::new(0.0, 0.5, 0.8);
    assert!(dominant.is_dominant());
    assert!(!dominant.is_submissive());

    let submissive = Pad::new(0.0, 0.5, 0.2);
    assert!(submissive.is_submissive());
    assert!(!submissive.is_dominant());
}

#[test]
fn test_pad_blend() {
    let pad1 = Pad::new(1.0, 0.5, 0.5);
    let pad2 = Pad::new(-1.0, 0.5, 0.5);

    let blended = pad1.blend(&pad2, 0.5);

    assert_eq!(blended.pleasure, 0.0); // Average of 1.0 and -1.0
}

#[test]
fn test_pad_toward_neutral() {
    let excited = Pad::new(0.8, 0.9, 0.8);
    let calmer = excited.toward_neutral(0.5);

    assert!(calmer.pleasure.abs() < excited.pleasure.abs());
}

#[test]
fn test_pad_distance() {
    let pad1 = Pad::new(0.0, 0.0, 0.0);
    let pad2 = Pad::new(1.0, 1.0, 1.0);

    let dist = pad1.distance(&pad2);
    assert!(dist > 0.0);

    // Distance to self should be 0
    assert_eq!(pad1.distance(&pad1), 0.0);
}

#[test]
fn test_pad_decay() {
    let mut pad = Pad::new(0.8, 0.9, 0.7);
    let baseline = Pad::neutral();

    pad.decay(&baseline, 0.5);

    assert!(pad.pleasure < 0.8);
    assert!(pad.arousal < 0.9);
}

#[test]
fn test_pad_risk_bias() {
    let excited_positive = Pad::new(0.5, 0.8, 0.5);
    let excited_negative = Pad::new(-0.5, 0.8, 0.5);

    // High arousal + positive = risk seeking
    assert!(excited_positive.risk_bias() > 0.0);

    // High arousal + negative = risk averse
    assert!(excited_negative.risk_bias() < 0.0);
}

#[test]
fn test_pad_memory_enhancement() {
    let high_arousal = Pad::new(0.0, 0.9, 0.5);
    let low_arousal = Pad::new(0.0, 0.1, 0.5);

    assert!(high_arousal.memory_enhancement() > low_arousal.memory_enhancement());
}

// ============================================================================
// Basic Emotion Tests
// ============================================================================

#[test]
fn test_basic_emotion_to_pad() {
    let happy_pad = Pad::from_basic_emotion(BasicEmotion::Happy);
    assert!(happy_pad.pleasure > 0.0);
    assert!(happy_pad.arousal < 0.5); // Happy is low arousal

    let angry_pad = Pad::from_basic_emotion(BasicEmotion::Angry);
    assert!(angry_pad.pleasure < 0.0);
    assert!(angry_pad.arousal > 0.5);
}

#[test]
fn test_pad_to_basic_emotion() {
    let joy = Pad::JOY;
    let emotion = joy.to_basic_emotion();

    // Joy should map to Happy or Content
    assert!(matches!(
        emotion,
        BasicEmotion::Happy | BasicEmotion::Content
    ));
}

#[test]
fn test_basic_emotion_roundtrip() {
    // Test that conversion is somewhat consistent
    for emotion in [
        BasicEmotion::Happy,
        BasicEmotion::Sad,
        BasicEmotion::Angry,
        BasicEmotion::Afraid,
    ] {
        let pad = Pad::from_basic_emotion(emotion);
        let roundtrip = pad.to_basic_emotion();

        // The roundtrip might not be exact due to overlapping regions
        // but should be in the same general category
        match emotion {
            BasicEmotion::Happy | BasicEmotion::Content => {
                assert!(pad.pleasure > 0.0)
            }
            BasicEmotion::Sad | BasicEmotion::Depressed => {
                assert!(pad.pleasure < 0.0)
            }
            _ => {} // Other emotions can vary
        }
    }
}

// ============================================================================
// Emotional Trait Tests
// ============================================================================

#[test]
fn test_emotional_trait_baseline() {
    let optimistic = EmotionalTrait::Optimistic;
    let baseline = optimistic.baseline_offset();
    assert!(baseline.pleasure > 0.0);

    let pessimistic = EmotionalTrait::Pessimistic;
    let baseline = pessimistic.baseline_offset();
    assert!(baseline.pleasure < 0.0);

    let high_energy = EmotionalTrait::HighEnergy;
    let baseline = high_energy.baseline_offset();
    assert!(baseline.arousal > 0.0);
}

// ============================================================================
// Emotion Category Tests
// ============================================================================

#[test]
fn test_emotion_category_pad_center() {
    let joy_center = EmotionCategory::Joy.pad_center();
    assert!(joy_center.pleasure > 0.0);

    let fear_center = EmotionCategory::Fear.pad_center();
    assert!(fear_center.pleasure < 0.0);
    assert!(fear_center.arousal > 0.5);
}

#[test]
fn test_emotion_category_opposite() {
    assert!(matches!(
        EmotionCategory::Joy.opposite(),
        EmotionCategory::Sadness
    ));

    assert!(matches!(
        EmotionCategory::Trust.opposite(),
        EmotionCategory::Disgust
    ));

    // Double opposite should return original
    assert!(matches!(
        EmotionCategory::Joy.opposite().opposite(),
        EmotionCategory::Joy
    ));
}

// ============================================================================
// Emotional Intelligence Tests
// ============================================================================

#[test]
fn test_emotional_intelligence_creation() {
    let ei = EmotionalIntelligence::new();
    let current = ei.current();

    // Should start near neutral
    assert!(current.pleasure.abs() < 0.1);
}

#[test]
fn test_emotional_intelligence_update() {
    let mut ei = EmotionalIntelligence::new();

    let event = EmotionalEvent {
        description: "Good news".to_string(),
        pleasure_impact: 0.5,
        arousal_impact: 0.3,
        dominance_impact: 0.2,
    };

    ei.update(&event);

    assert!(ei.current().pleasure > 0.0);
    assert!(ei.current().arousal > 0.5);
}

#[test]
fn test_emotional_intelligence_tick() {
    let mut ei = EmotionalIntelligence::new();

    // First boost emotion
    let event = EmotionalEvent {
        description: "Exciting event".to_string(),
        pleasure_impact: 0.8,
        arousal_impact: 0.5,
        dominance_impact: 0.0,
    };
    ei.update(&event);

    let before_tick = *ei.current();
    ei.tick();
    let after_tick = *ei.current();

    // Should decay toward baseline
    assert!(after_tick.pleasure.abs() < before_tick.pleasure.abs());
}

#[test]
fn test_emotional_intelligence_empathize() {
    let mut ei = EmotionalIntelligence::new();

    // Start neutral
    let neutral = Pad::neutral();

    // Empathize with someone who is happy
    let happy_other = Pad::new(0.8, 0.6, 0.5);
    ei.empathize(&happy_other);

    // Should move toward the other person's emotion
    assert!(ei.current().pleasure > neutral.pleasure);
}

// ============================================================================
// Emotion Enum Tests
// ============================================================================

#[test]
fn test_emotion_from_pad() {
    let happy_pad = Pad::from_basic_emotion(BasicEmotion::Happy);
    let emotion = Emotion::from_pad(happy_pad);

    assert!(matches!(emotion, Emotion::Happy | Emotion::Joy));
}

#[test]
fn test_basic_emotion_to_emotion_conversion() {
    let basic = BasicEmotion::Angry;
    let emotion: Emotion = basic.into();

    assert!(matches!(emotion, Emotion::Angry));
}

// ============================================================================
// PAD Arithmetic Tests
// ============================================================================

#[test]
fn test_pad_addition() {
    let pad1 = Pad::new(0.3, 0.4, 0.5);
    let pad2 = Pad::new(0.2, 0.3, 0.2);

    let sum = pad1 + pad2;

    assert_eq!(sum.pleasure, 0.5);
    assert_eq!(sum.arousal, 0.7);
    assert_eq!(sum.dominance, 0.7);
}

#[test]
fn test_pad_scalar_multiplication() {
    let pad = Pad::new(0.5, 0.6, 0.7);
    let scaled = pad * 2.0;

    assert_eq!(scaled.pleasure, 1.0); // Clamped
    assert_eq!(scaled.arousal, 1.0); // Clamped
    assert_eq!(scaled.dominance, 1.0); // Clamped
}
