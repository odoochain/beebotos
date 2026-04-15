//! Emotion Dynamics
//!
//! Handles emotion evolution over time.

#![allow(dead_code)]

use super::state::EmotionState;

/// Emotion dynamics engine
pub struct EmotionDynamics {
    decay_rate: f64,
    baseline: EmotionState,
}

impl EmotionDynamics {
    pub fn new(decay_rate: f64) -> Self {
        Self {
            decay_rate,
            baseline: EmotionState::neutral(),
        }
    }

    /// Update emotion state over time
    pub fn update(&self, emotion: &mut EmotionState, dt: f64) {
        // Decay towards baseline
        *emotion = emotion.lerp(&self.baseline, self.decay_rate * dt);
    }

    /// Set baseline emotion
    pub fn set_baseline(&mut self, baseline: EmotionState) {
        self.baseline = baseline;
    }

    /// Get baseline
    pub fn baseline(&self) -> &EmotionState {
        &self.baseline
    }
}

impl Default for EmotionDynamics {
    fn default() -> Self {
        Self::new(0.01)
    }
}
