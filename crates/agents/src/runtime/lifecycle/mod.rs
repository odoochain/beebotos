//! Agent lifecycle management

use serde::{Deserialize, Serialize};

/// Agent lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    /// Initial state
    Created,
    /// Setting up environment
    Initializing,
    /// Ready to run
    Ready,
    /// Actively running
    Running,
    /// Temporarily paused
    Paused,
    /// Shutting down
    Stopping,
    /// Stopped but can restart
    Stopped,
    /// Cleanup completed
    Terminated,
    /// Error occurred
    Error,
}

impl LifecycleState {
    /// Check if state is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }

    /// Check if can transition to target state
    pub fn can_transition(&self, target: LifecycleState) -> bool {
        use LifecycleState::*;
        match (self, target) {
            (Created, Initializing) => true,
            (Initializing, Ready) => true,
            (Initializing, Error) => true,
            (Ready, Running) => true,
            (Ready, Stopped) => true,
            (Running, Paused) => true,
            (Running, Stopping) => true,
            (Paused, Running) => true,
            (Paused, Stopping) => true,
            (Stopping, Stopped) => true,
            (Stopped, Running) => true,
            (Stopped, Terminated) => true,
            (Error, Stopped) => true,
            (Error, Terminated) => true,
            _ => false,
        }
    }

    /// Get state description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Created => "Agent created, waiting for initialization",
            Self::Initializing => "Setting up agent environment",
            Self::Ready => "Agent ready to start",
            Self::Running => "Agent actively executing tasks",
            Self::Paused => "Agent temporarily paused",
            Self::Stopping => "Agent shutting down",
            Self::Stopped => "Agent stopped, can be restarted",
            Self::Terminated => "Agent terminated and cleaned up",
            Self::Error => "Error occurred during execution",
        }
    }
}

/// Lifecycle manager
pub struct LifecycleManager {
    current: LifecycleState,
    history: Vec<StateTransition>,
}

impl LifecycleManager {
    /// Create new lifecycle manager
    pub fn new() -> Self {
        Self {
            current: LifecycleState::Created,
            history: vec![StateTransition {
                from: None,
                to: LifecycleState::Created,
                timestamp: now(),
                reason: "Agent created".to_string(),
            }],
        }
    }

    /// Get current state
    pub fn current(&self) -> LifecycleState {
        self.current
    }

    /// Attempt state transition
    pub fn transition(
        &mut self,
        to: LifecycleState,
        reason: impl Into<String>,
    ) -> Result<(), TransitionError> {
        if !self.current.can_transition(to) {
            return Err(TransitionError::InvalidTransition {
                from: self.current,
                to,
            });
        }

        let transition = StateTransition {
            from: Some(self.current),
            to,
            timestamp: now(),
            reason: reason.into(),
        };

        self.current = to;
        self.history.push(transition);
        Ok(())
    }

    /// Get state history
    pub fn history(&self) -> &[StateTransition] {
        &self.history
    }

    /// Time in current state
    pub fn time_in_state(&self) -> u64 {
        self.history
            .last()
            .map(|t| now() - t.timestamp)
            .unwrap_or(0)
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// State transition record
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: Option<LifecycleState>,
    pub to: LifecycleState,
    pub timestamp: u64,
    pub reason: String,
}

/// Transition error
#[derive(Debug, Clone)]
pub enum TransitionError {
    InvalidTransition {
        from: LifecycleState,
        to: LifecycleState,
    },
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => {
                write!(f, "Invalid transition from {:?} to {:?}", from, to)
            }
        }
    }
}

impl std::error::Error for TransitionError {}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
