//! Personality Dynamics

use super::ocean::OceanScores;

pub struct PersonalityDynamics {
    baseline: OceanScores,
    current: OceanScores,
}

impl PersonalityDynamics {
    pub fn new(scores: OceanScores) -> Self {
        Self {
            baseline: scores,
            current: scores,
        }
    }

    pub fn adapt(&mut self, situation: &str, intensity: f64) {
        // Temporary personality adaptations
    }

    pub fn revert(&mut self, rate: f64) {
        // Return to baseline
        self.current = self.current.lerp(&self.baseline, rate);
    }
}
