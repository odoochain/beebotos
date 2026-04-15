//! Subagent Spawning Module
//!
//! Implements non-blocking subagent creation (OpenClaw feature).
//! - Async spawn with immediate return
//! - Resource quota management
//! - Cross-agent spawning support

pub mod announce;
pub mod commands;
pub mod engine;
pub mod nonblocking;
pub mod workspace;

pub use announce::Announcement;
pub use engine::SpawnEngine;
pub use nonblocking::{SpawnConfig, SpawnResult, SpawnStatus};
