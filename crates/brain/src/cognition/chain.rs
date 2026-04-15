//! Chain of Thought

use super::thought::Thought;

/// Chain of thoughts
#[derive(Debug, Default)]
pub struct ThoughtChain {
    thoughts: Vec<Thought>,
}

impl ThoughtChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, thought: Thought) {
        self.thoughts.push(thought);
    }

    pub fn steps(&self) -> &[Thought] {
        &self.thoughts
    }

    pub fn conclusion(&self) -> Option<&Thought> {
        self.thoughts.last()
    }
}
