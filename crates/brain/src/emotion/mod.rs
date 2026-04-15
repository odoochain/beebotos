//! Emotion Module - High-Level Emotional System
//!
//! This module provides **high-level emotional processing** capabilities
//! built on top of the foundational PAD model.
//!
//! ## Features
//! - Emotion dynamics (decay toward baseline over time)
//! - Emotional contagion (spread of emotions between agents)
//! - Emotion-based memory enhancement
//! - Emotional computation and blending
//!
//! ## Architecture
//! - `state` - Emotional state representation (re-exports from `pad`)
//! - `dynamics` - Temporal evolution of emotions
//! - `contagion` - Inter-agent emotional influence
//! - `computing` - Algorithms for emotion calculation
//!
//! ## Relation to `pad` module
//! This module depends on `pad` for the core PAD representation.
//! Use `pad` directly when you need:
//! - Low-level numerical operations
//! - Direct PAD space manipulation
//!
//! Use this `emotion` module when you need:
//! - Temporal dynamics
//! - Social/emotional contagion
//! - Integration with memory and cognition

pub mod computing;
pub mod contagion;
pub mod dynamics;
pub mod memory;
pub mod state;
pub mod transitions;

pub use dynamics::EmotionDynamics;
pub use state::{EmotionState, EmotionType};

/// Emotion engine configuration
#[derive(Debug, Clone)]
pub struct EmotionConfig {
    pub decay_rate: f64,
    pub contagion_rate: f64,
    pub enable_contagion: bool,
}

impl Default for EmotionConfig {
    fn default() -> Self {
        Self {
            decay_rate: 0.01,
            contagion_rate: 0.3,
            enable_contagion: true,
        }
    }
}

/// Main emotion engine
pub struct EmotionEngine {
    #[allow(dead_code)]
    config: EmotionConfig,
    current_state: EmotionState,
    dynamics: EmotionDynamics,
}

impl EmotionEngine {
    pub fn new(config: EmotionConfig) -> Self {
        let dynamics = EmotionDynamics::new(config.decay_rate);
        Self {
            config,
            current_state: EmotionState::neutral(),
            dynamics,
        }
    }

    /// Update emotion state
    pub fn update(&mut self, dt: f64) {
        self.dynamics.update(&mut self.current_state, dt);
    }

    /// Apply emotional stimulus
    pub fn apply_stimulus(&mut self, stimulus: EmotionState, intensity: f64) {
        self.current_state = self.current_state.lerp(&stimulus, intensity);
    }

    /// Get current emotion
    pub fn current(&self) -> &EmotionState {
        &self.current_state
    }

    /// Set emotion directly
    pub fn set_emotion(&mut self, emotion: EmotionState) {
        self.current_state = emotion;
    }
}

impl Default for EmotionEngine {
    fn default() -> Self {
        Self::new(EmotionConfig::default())
    }
}
