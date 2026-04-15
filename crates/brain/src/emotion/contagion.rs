//! Emotion Contagion
//!
//! Implements emotional contagion between agents.

#![allow(dead_code)]

use super::state::EmotionState;

/// Contagion configuration
#[derive(Debug, Clone)]
pub struct ContagionConfig {
    /// How easily this agent catches emotions
    pub receptivity: f64,
    /// How strongly this agent spreads emotions
    pub expressiveness: f64,
    /// Minimum arousal for contagion
    pub arousal_threshold: f64,
}

impl Default for ContagionConfig {
    fn default() -> Self {
        Self {
            receptivity: 0.5,
            expressiveness: 0.5,
            arousal_threshold: 0.3,
        }
    }
}

/// Emotion contagion engine
pub struct EmotionContagion {
    config: ContagionConfig,
}

impl EmotionContagion {
    pub fn new(config: ContagionConfig) -> Self {
        Self { config }
    }

    /// Calculate emotional contagion
    pub fn contagion(
        &self,
        receiver: &EmotionState,
        sender: &EmotionState,
        relationship_strength: f64,
    ) -> Option<EmotionState> {
        // Only high arousal emotions are contagious
        if sender.arousal < self.config.arousal_threshold {
            return None;
        }

        // Calculate transfer intensity
        let intensity = self.config.receptivity
            * self.config.expressiveness
            * relationship_strength
            * sender.arousal;

        if intensity < 0.1 {
            return None;
        }

        // Create transferred emotion (reduce arousal slightly)
        let transferred = EmotionState::new(
            sender.pleasure,
            sender.arousal * 0.7,
            sender.dominance * 0.8,
        );

        // Blend with receiver's current emotion
        Some(receiver.lerp(&transferred, intensity))
    }

    /// Apply group emotion convergence
    pub fn group_convergence(
        &self,
        current: &EmotionState,
        group_average: &EmotionState,
        group_size: usize,
    ) -> EmotionState {
        let convergence_rate = 0.1 / (group_size as f64).sqrt();
        current.lerp(group_average, convergence_rate)
    }
}

impl Default for EmotionContagion {
    fn default() -> Self {
        Self::new(ContagionConfig::default())
    }
}
