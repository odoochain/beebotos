//! Cognitive Process

use crate::error::BrainResult;

/// Cognitive process types
#[derive(Debug, Clone)]
pub enum CognitiveProcess {
    Perception,
    Attention,
    Memory,
    Language,
    Learning,
    Decision,
}

/// Process manager
pub struct ProcessManager {
    active_processes: Vec<CognitiveProcess>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            active_processes: Vec::new(),
        }
    }

    pub fn start(&mut self, process: CognitiveProcess) {
        self.active_processes.push(process);
    }

    pub fn stop(&mut self, process: &CognitiveProcess) {
        self.active_processes.retain(|p| std::mem::discriminant(p) != std::mem::discriminant(process));
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
