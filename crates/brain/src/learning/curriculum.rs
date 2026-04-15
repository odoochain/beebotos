//! Curriculum Learning
//!
//! Structured learning progression.

/// Curriculum designer
pub struct Curriculum {
    stages: Vec<LearningStage>,
    current_stage: usize,
}

#[derive(Debug, Clone)]
pub struct LearningStage {
    pub name: String,
    pub difficulty: f32,
    pub objectives: Vec<String>,
}

impl Curriculum {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            current_stage: 0,
        }
    }

    pub fn add_stage(&mut self, stage: LearningStage) {
        self.stages.push(stage);
    }

    pub fn current(&self) -> Option<&LearningStage> {
        self.stages.get(self.current_stage)
    }

    pub fn advance(&mut self) -> bool {
        if self.current_stage < self.stages.len() - 1 {
            self.current_stage += 1;
            true
        } else {
            false
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_stage >= self.stages.len() - 1
    }
}

impl Default for Curriculum {
    fn default() -> Self {
        Self::new()
    }
}
