//! Personality Genetics

use super::ocean::OceanScores;

/// Genetic basis for personality
#[derive(Debug, Clone)]
pub struct PersonalityGenome {
    pub openness_gene: f64,
    pub conscientiousness_gene: f64,
    pub extraversion_gene: f64,
    pub agreeableness_gene: f64,
    pub neuroticism_gene: f64,
}

impl PersonalityGenome {
    pub fn express(&self, environment: f64) -> OceanScores {
        OceanScores::new(
            self.openness_gene * environment,
            self.conscientiousness_gene * environment,
            self.extraversion_gene * environment,
            self.agreeableness_gene * environment,
            self.neuroticism_gene * environment,
        )
    }

    pub fn crossover(parent1: &Self, parent2: &Self) -> Self {
        Self {
            openness_gene: (parent1.openness_gene + parent2.openness_gene) / 2.0,
            conscientiousness_gene: (parent1.conscientiousness_gene + parent2.conscientiousness_gene) / 2.0,
            extraversion_gene: (parent1.extraversion_gene + parent2.extraversion_gene) / 2.0,
            agreeableness_gene: (parent1.agreeableness_gene + parent2.agreeableness_gene) / 2.0,
            neuroticism_gene: (parent1.neuroticism_gene + parent2.neuroticism_gene) / 2.0,
        }
    }
}
