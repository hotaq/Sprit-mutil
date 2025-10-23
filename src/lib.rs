//! Sprite Multi-Agent Workflow Toolkit Library
//!
//! This library provides the core functionality for the Sprite multi-agent workflow toolkit,
//! including configuration management, workspace provisioning, git operations, and tmux integration.

pub mod cli;
pub mod commands;
pub mod communication;
pub mod config;
pub mod error;
pub mod models;
pub mod utils;
pub mod validation;

// Re-export commonly used types for easier access in tests
pub use commands::config::{ConfigChanges, SimpleAgentConfig, SpriteConfig};
pub use error::SpriteError;
pub use utils::git;
