//! Skills Module
//!
//! Skill system for agent capabilities (ClawHub integration).
//!
//! 🔧 P0 FIX: Added link_handler and command_handler for enhanced functionality

pub mod command_handler;
pub mod executor;
pub mod link_handler;
pub mod loader;
pub mod registry;

pub use command_handler::{CommandContext, CommandHandler, CommandResult, RuntimeInfo, RuntimeStatus};
pub use executor::{SkillContext, SkillExecutionError, SkillExecutionResult, SkillExecutor};
pub use link_handler::{format_summary_for_display, LinkHandler, LinkSummary, ContentType};
pub use loader::SkillLoader;
pub use registry::{SkillDefinition, SkillRegistry};
