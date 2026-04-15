//! Emotion Computing
//!
//! Algorithms for computing emotional responses.

#![allow(dead_code)]

use super::state::{EmotionState, EmotionType};

/// Emotion computing engine
pub struct EmotionComputing;

impl EmotionComputing {
    /// Compute emotion from text sentiment
    pub fn from_sentiment(positive: f64, negative: f64, intensity: f64) -> EmotionState {
        let pleasure = positive - negative;
        let arousal = intensity * 2.0 - 1.0;
        let dominance = if pleasure > 0.0 { 0.3 } else { -0.3 };

        EmotionState::new(pleasure, arousal, dominance)
    }

    /// Compute emotion from event impact
    pub fn from_event(impact: f64, unexpectedness: f64, control: f64) -> EmotionState {
        let pleasure = if impact > 0.0 { impact } else { -impact };
        let arousal = unexpectedness;
        let dominance = control;

        EmotionState::new(pleasure, arousal, dominance)
    }

    /// Blend multiple emotions
    pub fn blend(emotions: &[(EmotionState, f64)]) -> EmotionState {
        let total_weight: f64 = emotions.iter().map(|(_, w)| w).sum();

        if total_weight == 0.0 {
            return EmotionState::neutral();
        }

        let mut pleasure = 0.0;
        let mut arousal = 0.0;
        let mut dominance = 0.0;

        for (emotion, weight) in emotions {
            let normalized_weight = weight / total_weight;
            pleasure += emotion.pleasure * normalized_weight;
            arousal += emotion.arousal * normalized_weight;
            dominance += emotion.dominance * normalized_weight;
        }

        EmotionState::new(pleasure, arousal, dominance)
    }

    /// Find nearest named emotion
    pub fn nearest_named(state: &EmotionState) -> EmotionType {
        let emotions = [
            EmotionType::Neutral,
            EmotionType::Happy,
            EmotionType::Sad,
            EmotionType::Angry,
            EmotionType::Afraid,
            EmotionType::Excited,
            EmotionType::Bored,
        ];

        emotions
            .iter()
            .min_by_key(|e| {
                let state2 = e.to_state();
                let dist = state.distance(&state2);
                (dist * 1000.0) as i32
            })
            .copied()
            .unwrap_or(EmotionType::Neutral)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_sentiment() {
        let emotion = EmotionComputing::from_sentiment(0.8, 0.2, 0.7);
        assert!(emotion.pleasure > 0.0);
    }

    #[test]
    fn test_blend() {
        let e1 = EmotionState::happy();
        let e2 = EmotionState::sad();
        let blended = EmotionComputing::blend(&[(e1, 0.7), (e2, 0.3)]);
        assert!(blended.pleasure > 0.0);
    }
}
