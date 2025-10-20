//! Utility modules for the Sprite multi-agent workflow toolkit.
//!
//! This directory contains utility functions for various operations:
//! - git: Git repository and worktree management
//! - tmux: Terminal multiplexer session management
//! - logging: Application logging infrastructure
//! - security: Path validation and security checks
//! - session_recovery: Session health monitoring and recovery
//! - accessibility: Accessibility features and WCAG compliance

pub mod accessibility;
pub mod git;
pub mod logging;
pub mod security;
pub mod session_recovery;
pub mod tmux;
