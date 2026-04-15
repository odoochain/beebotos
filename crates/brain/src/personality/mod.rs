//! Personality System
//!
//! OCEAN (Big Five) personality model implementation.

use serde::{Deserialize, Serialize};

/// OCEAN personality profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OceanProfile {
    pub openness: f32,
    pub conscientiousness: f32,
    pub extraversion: f32,
    pub agreeableness: f32,
    pub neuroticism: f32,
}

impl OceanProfile {
    /// Create new profile
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

    /// Creative profile
    pub fn creative() -> Self {
        Self::new(0.8, 0.5, 0.6, 0.5, 0.4)
    }

    /// Analytical profile
    pub fn analytical() -> Self {
        Self::new(0.7, 0.8, 0.3, 0.4, 0.3)
    }

    /// Social profile
    pub fn social() -> Self {
        Self::new(0.6, 0.4, 0.8, 0.8, 0.4)
    }

    /// Leader profile
    pub fn leader() -> Self {
        Self::new(0.7, 0.8, 0.7, 0.6, 0.3)
    }

    /// Distance from another profile
    pub fn distance(&self, other: &OceanProfile) -> f32 {
        let d_openness = self.openness - other.openness;
        let d_conscientiousness = self.conscientiousness - other.conscientiousness;
        let d_extraversion = self.extraversion - other.extraversion;
        let d_agreeableness = self.agreeableness - other.agreeableness;
        let d_neuroticism = self.neuroticism - other.neuroticism;

        (d_openness * d_openness
            + d_conscientiousness * d_conscientiousness
            + d_extraversion * d_extraversion
            + d_agreeableness * d_agreeableness
            + d_neuroticism * d_neuroticism)
            .sqrt()
    }

    /// Similarity score (0.0 - 1.0)
    pub fn similarity(&self, other: &OceanProfile) -> f32 {
        1.0 - self.distance(other) / 5.0f32.sqrt()
    }

    /// Get dominant trait
    pub fn dominant_trait(&self) -> (&'static str, f32) {
        let traits = [
            ("openness", self.openness),
            ("conscientiousness", self.conscientiousness),
            ("extraversion", self.extraversion),
            ("agreeableness", self.agreeableness),
            ("neuroticism", self.neuroticism),
        ];

        traits
            .iter()
            .copied()
            .max_by(|a, b| crate::utils::compare_f32(&a.1, &b.1))
            .unwrap_or(("unknown", 0.0))
    }

    /// Influence behavior
    pub fn influence(&self, behavior: &mut Behavior) {
        // Openness affects creativity
        behavior.creativity = self.openness;

        // Conscientiousness affects organization
        behavior.organization = self.conscientiousness;

        // Extraversion affects social engagement
        behavior.social_engagement = self.extraversion;

        // Agreeableness affects cooperation
        behavior.cooperation = self.agreeableness;

        // Neuroticism affects risk tolerance (inverted)
        behavior.risk_tolerance = 1.0 - self.neuroticism;
    }
}

impl Default for OceanProfile {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Behavior influenced by personality
#[derive(Debug, Clone, Default)]
pub struct Behavior {
    pub creativity: f32,
    pub organization: f32,
    pub social_engagement: f32,
    pub cooperation: f32,
    pub risk_tolerance: f32,
}

/// Personality engine
pub struct OceanEngine {
    profile: OceanProfile,
    learning_rate: f32,
}

impl OceanEngine {
    pub fn new(profile: OceanProfile) -> Self {
        Self {
            profile,
            learning_rate: 0.01,
        }
    }

    /// Adapt personality based on experience
    pub fn adapt(&mut self, experience: &Experience) {
        // Adjust traits based on outcome
        match experience.outcome {
            Outcome::Positive => {
                self.profile.openness += self.learning_rate * experience.openness_relevance;
            }
            Outcome::Negative => {
                self.profile.neuroticism += self.learning_rate * 0.5;
            }
            _ => {}
        }

        // Clamp values
        self.profile = OceanProfile::new(
            self.profile.openness,
            self.profile.conscientiousness,
            self.profile.extraversion,
            self.profile.agreeableness,
            self.profile.neuroticism,
        );
    }

    /// Get current profile
    pub fn profile(&self) -> &OceanProfile {
        &self.profile
    }

    /// Calculate decision style
    pub fn decision_style(&self) -> DecisionStyle {
        let o = self.profile.openness;
        let c = self.profile.conscientiousness;
        let e = self.profile.extraversion;
        let a = self.profile.agreeableness;
        let n = self.profile.neuroticism;

        if c > 0.7 && n < 0.4 {
            DecisionStyle::Analytical
        } else if e > 0.7 && a > 0.6 {
            DecisionStyle::Collaborative
        } else if o > 0.7 && n < 0.5 {
            DecisionStyle::Innovative
        } else if c > 0.6 && n < 0.3 {
            DecisionStyle::Cautious
        } else {
            DecisionStyle::Balanced
        }
    }

    /// Get learning strategy
    pub fn learning_strategy(&self) -> LearningStrategy {
        if self.profile.openness > 0.7 {
            LearningStrategy::Exploratory
        } else if self.profile.conscientiousness > 0.7 {
            LearningStrategy::Structured
        } else if self.profile.extraversion > 0.7 {
            LearningStrategy::Social
        } else {
            LearningStrategy::Adaptive
        }
    }
}

/// Experience for personality adaptation
#[derive(Debug, Clone)]
pub struct Experience {
    pub outcome: Outcome,
    pub openness_relevance: f32,
    pub context: String,
}

/// Outcome of experience
#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Positive,
    Negative,
    Neutral,
}

/// Decision style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionStyle {
    Analytical,
    Collaborative,
    Innovative,
    Cautious,
    Balanced,
}

/// Learning strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningStrategy {
    Exploratory,
    Structured,
    Social,
    Adaptive,
}
