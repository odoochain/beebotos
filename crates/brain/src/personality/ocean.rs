//! OCEAN Personality Engine
//!
//! Implementation of the Big Five personality model.

use serde::{Deserialize, Serialize};

/// OCEAN personality profile
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct OceanProfile {
    /// Openness to experience
    pub openness: f32,
    /// Conscientiousness
    pub conscientiousness: f32,
    /// Extraversion
    pub extraversion: f32,
    /// Agreeableness
    pub agreeableness: f32,
    /// Neuroticism (emotional stability inverse)
    pub neuroticism: f32,
}

impl OceanProfile {
    /// Create new profile with all traits
    pub fn new(
        openness: f32,
        conscientiousness: f32,
        extraversion: f32,
        agreeableness: f32,
        neuroticism: f32,
    ) -> Self {
        Self {
            openness: openness.clamp(0.0, 1.0),
            conscientiousness: conscientiousness.clamp(0.0, 1.0),
            extraversion: extraversion.clamp(0.0, 1.0),
            agreeableness: agreeableness.clamp(0.0, 1.0),
            neuroticism: neuroticism.clamp(0.0, 1.0),
        }
    }

    /// Balanced profile (all 0.5)
    pub fn balanced() -> Self {
        Self::new(0.5, 0.5, 0.5, 0.5, 0.5)
    }

    /// Creative personality (high openness)
    pub fn creative() -> Self {
        Self::new(0.85, 0.5, 0.65, 0.6, 0.4)
    }

    /// Analytical personality (high conscientiousness)
    pub fn analytical() -> Self {
        Self::new(0.7, 0.85, 0.35, 0.45, 0.25)
    }

    /// Social personality (high extraversion, agreeableness)
    pub fn social() -> Self {
        Self::new(0.6, 0.45, 0.85, 0.85, 0.35)
    }

    /// Leader personality
    pub fn leader() -> Self {
        Self::new(0.75, 0.8, 0.7, 0.65, 0.3)
    }

    /// Cautious personality
    pub fn cautious() -> Self {
        Self::new(0.4, 0.75, 0.3, 0.6, 0.6)
    }

    /// Calculate Euclidean distance to another profile
    pub fn distance(&self, other: &Self) -> f32 {
        let d_o = self.openness - other.openness;
        let d_c = self.conscientiousness - other.conscientiousness;
        let d_e = self.extraversion - other.extraversion;
        let d_a = self.agreeableness - other.agreeableness;
        let d_n = self.neuroticism - other.neuroticism;

        (d_o * d_o + d_c * d_c + d_e * d_e + d_a * d_a + d_n * d_n).sqrt()
    }

    /// Calculate similarity (0.0 - 1.0)
    pub fn similarity(&self, other: &Self) -> f32 {
        let max_dist = (5.0f32).sqrt(); // Maximum possible distance
        1.0 - self.distance(other) / max_dist
    }

    /// Get dominant trait name and value
    pub fn dominant_trait(&self) -> (&'static str, f32) {
        let traits = [
            ("openness", self.openness),
            ("conscientiousness", self.conscientiousness),
            ("extraversion", self.extraversion),
            ("agreeableness", self.agreeableness),
            ("neuroticism", self.neuroticism),
        ];

        traits.into_iter()
            .max_by(|a, b| crate::utils::compare_f32(&a.1, &b.1))
            .unwrap_or(("unknown", 0.0))
    }

    /// Get weakest trait
    pub fn weakest_trait(&self) -> (&'static str, f32) {
        let traits = [
            ("openness", self.openness),
            ("conscientiousness", self.conscientiousness),
            ("extraversion", self.extraversion),
            ("agreeableness", self.agreeableness),
            ("neuroticism", self.neuroticism),
        ];

        traits.into_iter()
            .min_by(|a, b| crate::utils::compare_f32(&a.1, &b.1))
            .unwrap_or(("unknown", 0.0))
    }

    /// Blend with another profile (weighted average)
    pub fn blend(&self, other: &Self, weight: f32) -> Self {
        let w = weight.clamp(0.0, 1.0);
        Self::new(
            self.openness * (1.0 - w) + other.openness * w,
            self.conscientiousness * (1.0 - w) + other.conscientiousness * w,
            self.extraversion * (1.0 - w) + other.extraversion * w,
            self.agreeableness * (1.0 - w) + other.agreeableness * w,
            self.neuroticism * (1.0 - w) + other.neuroticism * w,
        )
    }
}

impl Default for OceanProfile {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Personality engine for adaptation
pub struct OceanEngine {
    base_profile: OceanProfile,
    current_profile: OceanProfile,
    learning_rate: f32,
}

impl OceanEngine {
    pub fn new(profile: OceanProfile) -> Self {
        Self {
            base_profile: profile,
            current_profile: profile,
            learning_rate: 0.05,
        }
    }

    /// Adapt personality based on experience
    pub fn adapt(&mut self, outcome: &ExperienceOutcome) {
        let mut profile = self.current_profile;

        match outcome.result {
            ExperienceResult::Success => {
                // Reinforce traits that led to success
                profile.openness = self.move_toward(profile.openness, self.base_profile.openness * 1.1);
                profile.conscientiousness = self.move_toward(
                    profile.conscientiousness,
                    self.base_profile.conscientiousness * 1.1,
                );
            }
            ExperienceResult::Failure => {
                // Adapt to avoid future failures
                profile.neuroticism = (profile.neuroticism + 0.05).min(1.0);
            }
            ExperienceResult::Mixed => {
                // Small adjustments
            }
        }

        // Ensure bounds
        self.current_profile = OceanProfile::new(
            profile.openness,
            profile.conscientiousness,
            profile.extraversion,
            profile.agreeableness,
            profile.neuroticism,
        );
    }

    fn move_toward(&self, current: f32, target: f32) -> f32 {
        current + (target - current) * self.learning_rate
    }

    /// Reset to base profile
    pub fn reset(&mut self) {
        self.current_profile = self.base_profile;
    }

    /// Get current profile
    pub fn profile(&self) -> &OceanProfile {
        &self.current_profile
    }

    /// Get base profile
    pub fn base(&self) -> &OceanProfile {
        &self.base_profile
    }

    /// Calculate decision style based on personality
    pub fn decision_style(&self) -> DecisionStyle {
        let p = &self.current_profile;
        
        if p.conscientiousness > 0.7 && p.neuroticism < 0.4 {
            DecisionStyle::Analytical
        } else if p.extraversion > 0.7 && p.agreeableness > 0.6 {
            DecisionStyle::Collaborative
        } else if p.openness > 0.7 && p.neuroticism < 0.5 {
            DecisionStyle::Innovative
        } else if p.conscientiousness > 0.6 && p.openness < 0.4 {
            DecisionStyle::Cautious
        } else {
            DecisionStyle::Balanced
        }
    }

    /// Determine learning strategy
    pub fn learning_strategy(&self) -> LearningStrategy {
        let p = &self.current_profile;
        
        if p.openness > 0.75 {
            LearningStrategy::Exploratory
        } else if p.conscientiousness > 0.7 {
            LearningStrategy::Structured
        } else if p.extraversion > 0.7 {
            LearningStrategy::Social
        } else if p.neuroticism < 0.3 {
            LearningStrategy::Experimental
        } else {
            LearningStrategy::Adaptive
        }
    }

    /// Calculate communication style
    pub fn communication_style(&self) -> CommunicationStyle {
        let p = &self.current_profile;
        
        if p.extraversion > 0.7 {
            CommunicationStyle::Expressive
        } else if p.agreeableness > 0.7 {
            CommunicationStyle::Empathetic
        } else if p.openness > 0.7 {
            CommunicationStyle::Intellectual
        } else if p.conscientiousness > 0.7 {
            CommunicationStyle::Precise
        } else {
            CommunicationStyle::Balanced
        }
    }
}

/// Experience outcome for adaptation
#[derive(Debug, Clone, Copy)]
pub struct ExperienceOutcome {
    pub result: ExperienceResult,
    pub effort: f32,
    pub satisfaction: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ExperienceResult {
    Success,
    Failure,
    Mixed,
}

/// Decision style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStyle {
    Analytical,
    Collaborative,
    Innovative,
    Cautious,
    Balanced,
}

/// Learning strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningStrategy {
    Exploratory,
    Structured,
    Social,
    Experimental,
    Adaptive,
}

/// Communication style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationStyle {
    Expressive,
    Empathetic,
    Intellectual,
    Precise,
    Balanced,
}

/// Personality presets for common agent types
pub mod presets {
    use super::OceanProfile;

    /// Helper agent
    pub fn helper() -> OceanProfile {
        OceanProfile::new(0.6, 0.75, 0.7, 0.85, 0.35)
    }

    /// Researcher agent
    pub fn researcher() -> OceanProfile {
        OceanProfile::new(0.85, 0.75, 0.4, 0.5, 0.3)
    }

    /// Creative agent
    pub fn creative() -> OceanProfile {
        OceanProfile::new(0.9, 0.4, 0.7, 0.6, 0.45)
    }

    /// Manager agent
    pub fn manager() -> OceanProfile {
        OceanProfile::new(0.7, 0.8, 0.75, 0.7, 0.35)
    }

    /// Coder agent
    pub fn coder() -> OceanProfile {
        OceanProfile::new(0.65, 0.85, 0.4, 0.5, 0.3)
    }
}
