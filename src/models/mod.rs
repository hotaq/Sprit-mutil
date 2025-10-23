//! Data models for the Sprite multi-agent workflow toolkit.
//!
//! This module contains the core data structures that define agents, sessions,
//! configurations, and other entities used throughout the application.

pub mod agent;
pub mod config;
pub mod session;

// Re-export commonly used types
pub use agent::{Agent, AgentStatus};
pub use config::{
    LoggingConfig, PerformanceSettings, ProjectConfig, ProjectSettings, SecuritySettings,
    SyncConfig,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Log levels for configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

/// Conflict resolution strategies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ConflictResolution {
    #[default]
    Manual,
    AutoTheirs,
    AutoOurs,
}

/// Command message for agent communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMessage {
    /// Unique message identifier
    pub id: String,

    /// Message timestamp
    pub timestamp: DateTime<Utc>,

    /// Target agent name (empty for broadcast)
    pub target_agent: Option<String>,

    /// Command to execute
    pub command: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Working directory for command
    pub work_dir: Option<String>,

    /// Environment variables for this command
    pub env_vars: HashMap<String, String>,

    /// Timeout for this command
    pub timeout_secs: Option<u64>,

    /// Message priority
    pub priority: MessagePriority,
}

/// Message priority levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MessagePriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

/// Execution result for commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Corresponding command message ID
    pub command_id: String,

    /// Agent that executed the command
    pub agent_name: String,

    /// Execution status
    pub status: ExecutionStatus,

    /// Command output (stdout)
    pub output: String,

    /// Error output (stderr)
    pub error_output: String,

    /// Exit code
    pub exit_code: i32,

    /// Execution start time
    pub start_time: DateTime<Utc>,

    /// Execution end time
    pub end_time: DateTime<Utc>,

    /// Resource usage during execution
    pub resource_usage: ResourceUsage,
}

/// Execution status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Timeout,
    Cancelled,
}

/// Resource usage metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,

    /// Memory usage in MB
    pub memory_mb: u64,

    /// Disk usage in MB
    pub disk_mb: u64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0,
            disk_mb: 0,
        }
    }
}
