//! Emotional transition rules for PAD model

use super::emotion::{Emotion, Pad};

/// Emotional transition
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EmotionalTransition {
    pub from: Emotion,
    pub to: Emotion,
    pub trigger: String,
    pub probability: f32,
}

/// Calculate emotional drift
#[allow(dead_code)]
pub fn drift(current: &Pad, target: &Pad, rate: f32) -> Pad {
    Pad::new(
        lerp(current.pleasure, target.pleasure, rate),
        lerp(current.arousal, target.arousal, rate),
        lerp(current.dominance, target.dominance, rate),
    )
}

#[allow(dead_code)]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
