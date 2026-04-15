//! Collaboration Workflow

use crate::error::Result;
use std::collections::HashMap;
use uuid::Uuid;

/// Multi-agent workflow
#[derive(Debug)]
pub struct CollaborationWorkflow {
    pub id: Uuid,
    pub steps: Vec<WorkflowStep>,
    pub current_step: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub task: String,
    pub dependencies: Vec<Uuid>,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl CollaborationWorkflow {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            steps: Vec::new(),
            current_step: 0,
        }
    }

    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.push(step);
    }

    pub fn current_step(&self) -> Option<&WorkflowStep> {
        self.steps.get(self.current_step)
    }

    pub fn advance(&mut self) -> bool {
        if self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
            true
        } else {
            false
        }
    }

    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|s| s.status == StepStatus::Completed)
    }
}

impl Default for CollaborationWorkflow {
    fn default() -> Self {
        Self::new()
    }
}
