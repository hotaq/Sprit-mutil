//! Agent data structures for the Sprite multi-agent workflow toolkit.

use super::{CommandMessage, ExecutionResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents an AI coding agent with its workspace configuration and operational parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier for the agent (number or name)
    pub id: String,

    /// Git branch name for this agent's work
    pub branch: String,

    /// AI model to use (e.g., "claude-sonnet-4", "gpt-4")
    pub model: Option<String>,

    /// Role or specialization description
    pub description: Option<String>,

    /// Local workspace path (defaults to agents/{id})
    pub worktree_path: Option<PathBuf>,

    /// Current status of the agent
    pub status: AgentStatus,

    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,

    /// tmux pane identifier
    pub tmux_pane: Option<String>,

    /// Agent-specific configuration
    pub config: AgentConfig,
}

#[allow(dead_code)]
impl Agent {
    /// Create a new agent with basic configuration.
    pub fn new(id: impl Into<String>, branch: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            id: id_str.clone(),
            branch: branch.into(),
            model: None,
            description: None,
            worktree_path: Some(PathBuf::from(format!("agents/{}", id_str))),
            status: AgentStatus::Inactive,
            last_activity: None,
            tmux_pane: None,
            config: AgentConfig::default(),
        }
    }

    /// Get the workspace path for this agent.
    pub fn workspace_path(&self) -> &PathBuf {
        self.worktree_path.as_ref().unwrap()
    }

    /// Update the agent's last activity timestamp.
    pub fn update_activity(&mut self) {
        self.last_activity = Some(Utc::now());
    }

    /// Check if the agent is currently active.
    pub fn is_active(&self) -> bool {
        matches!(self.status, AgentStatus::Active)
    }

    /// Check if the agent is in an error state.
    pub fn has_error(&self) -> bool {
        matches!(self.status, AgentStatus::Error(_))
    }

    /// Get the error message if the agent is in an error state.
    pub fn error_message(&self) -> Option<&str> {
        match &self.status {
            AgentStatus::Error(msg) => Some(msg),
            _ => None,
        }
    }

    /// Validate the agent configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Agent ID cannot be empty".to_string());
        }

        if self.branch.is_empty() {
            return Err("Agent branch cannot be empty".to_string());
        }

        // Validate ID format (alphanumeric with hyphens/underscores)
        if !self
            .id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "Agent ID can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            );
        }

        // Validate workspace path
        let workspace_path = self.workspace_path();
        if !workspace_path.starts_with("agents/") {
            return Err("Agent workspace must be within the agents/ directory".to_string());
        }

        // Check for path traversal
        if workspace_path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(
                "Agent workspace path cannot contain parent directory references".to_string(),
            );
        }

        Ok(())
    }

    /// Add a command to the agent's command history.
    pub fn add_command(&mut self, _command: CommandMessage) {
        self.update_activity();
        // TODO: Implement command history storage
    }

    /// Add an execution result to the agent's history.
    pub fn add_result(&mut self, _result: ExecutionResult) {
        self.update_activity();
        // TODO: Implement result history storage
    }
}

/// Current status of an agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    /// Agent is configured but not running
    Inactive,
    /// Agent is running and active
    Active,
    /// Agent is paused/suspended
    Paused,
    /// Agent encountered an error
    Error(String),
    /// Agent is being setup/initialized
    Initializing,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Inactive
    }
}

/// Agent-specific configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Shell environment variables
    pub env_vars: HashMap<String, String>,

    /// Default working directory within workspace
    pub work_dir: Option<PathBuf>,

    /// Startup commands to run when agent starts
    pub startup_commands: Vec<String>,

    /// Resource limits
    pub resource_limits: ResourceLimits,

    /// Preferred shell for this agent
    pub shell: Option<String>,

    /// Timeout for commands in seconds
    pub default_timeout_secs: Option<u64>,

    /// Whether to automatically sync changes
    pub auto_sync: bool,

    /// Custom settings for the agent
    pub custom_settings: HashMap<String, serde_yaml::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            env_vars: HashMap::new(),
            work_dir: None,
            startup_commands: Vec::new(),
            resource_limits: ResourceLimits::default(),
            shell: None,
            default_timeout_secs: Some(300), // 5 minutes default
            auto_sync: false,
            custom_settings: HashMap::new(),
        }
    }
}

#[allow(dead_code)]
impl AgentConfig {
    /// Get the default workspace path for an agent with the given ID.
    pub fn default_workspace(&self) -> PathBuf {
        PathBuf::from("agents/default")
    }

    /// Get the effective timeout value.
    pub fn timeout(&self) -> u64 {
        self.default_timeout_secs
            .or(self.resource_limits.operation_timeout_secs)
            .unwrap_or(300)
    }

    /// Get the effective shell command.
    pub fn shell_command(&self) -> &str {
        self.shell.as_deref().unwrap_or("bash")
    }

    /// Merge this configuration with another, with the other taking precedence.
    pub fn merge(&self, other: &AgentConfig) -> AgentConfig {
        let mut merged = self.clone();

        // Merge environment variables
        for (key, value) in &other.env_vars {
            merged.env_vars.insert(key.clone(), value.clone());
        }

        // Override fields if present in other
        if other.work_dir.is_some() {
            merged.work_dir = other.work_dir.clone();
        }
        merged
            .startup_commands
            .extend(other.startup_commands.clone());
        merged.resource_limits = merged.resource_limits.merge(&other.resource_limits);

        if other.shell.is_some() {
            merged.shell = other.shell.clone();
        }
        if other.default_timeout_secs.is_some() {
            merged.default_timeout_secs = other.default_timeout_secs;
        }
        merged.auto_sync = other.auto_sync;

        // Merge custom settings
        for (key, value) in &other.custom_settings {
            merged.custom_settings.insert(key.clone(), value.clone());
        }

        merged
    }
}

/// Resource limits for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,

    /// Maximum CPU percentage
    pub max_cpu_percent: Option<u8>,

    /// Timeout for operations in seconds
    pub operation_timeout_secs: Option<u64>,

    /// Maximum number of concurrent operations
    pub max_concurrent_ops: Option<u32>,

    /// Maximum disk usage in MB
    pub max_disk_mb: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(1024),         // 1GB default
            max_cpu_percent: Some(80),         // 80% default
            operation_timeout_secs: Some(300), // 5 minutes default
            max_concurrent_ops: Some(3),
            max_disk_mb: Some(5120), // 5GB default
        }
    }
}

#[allow(dead_code)]
impl ResourceLimits {
    /// Merge this resource limits with another, with the other taking precedence.
    pub fn merge(&self, other: &ResourceLimits) -> ResourceLimits {
        Self {
            max_memory_mb: other.max_memory_mb.or(self.max_memory_mb),
            max_cpu_percent: other.max_cpu_percent.or(self.max_cpu_percent),
            operation_timeout_secs: other.operation_timeout_secs.or(self.operation_timeout_secs),
            max_concurrent_ops: other.max_concurrent_ops.or(self.max_concurrent_ops),
            max_disk_mb: other.max_disk_mb.or(self.max_disk_mb),
        }
    }

    /// Check if a memory usage value exceeds the limit.
    pub fn exceeds_memory_limit(&self, memory_mb: u64) -> bool {
        self.max_memory_mb
            .map(|limit| memory_mb > limit)
            .unwrap_or(false)
    }

    /// Check if a CPU usage value exceeds the limit.
    pub fn exceeds_cpu_limit(&self, cpu_percent: f64) -> bool {
        self.max_cpu_percent
            .map(|limit| cpu_percent > limit as f64)
            .unwrap_or(false)
    }

    /// Check if a disk usage value exceeds the limit.
    pub fn exceeds_disk_limit(&self, disk_mb: u64) -> bool {
        self.max_disk_mb
            .map(|limit| disk_mb > limit)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("agent-1", "feature/test");
        assert_eq!(agent.id, "agent-1");
        assert_eq!(agent.branch, "feature/test");
        assert_eq!(agent.status, AgentStatus::Inactive);
        assert_eq!(agent.workspace_path(), &PathBuf::from("agents/agent-1"));
    }

    #[test]
    fn test_agent_validation() {
        let mut agent = Agent::new("valid-agent-1", "feature/test");
        assert!(agent.validate().is_ok());

        // Test empty ID
        agent.id = "".to_string();
        assert!(agent.validate().is_err());

        // Test invalid ID format
        agent.id = "invalid agent!".to_string();
        assert!(agent.validate().is_err());

        // Test empty branch
        agent.id = "valid-agent".to_string();
        agent.branch = "".to_string();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_status() {
        let mut agent = Agent::new("agent-1", "feature/test");

        assert!(!agent.is_active());
        assert!(!agent.has_error());

        agent.status = AgentStatus::Active;
        assert!(agent.is_active());
        assert!(!agent.has_error());

        agent.status = AgentStatus::Error("Test error".to_string());
        assert!(!agent.is_active());
        assert!(agent.has_error());
        assert_eq!(agent.error_message(), Some("Test error"));
    }

    #[test]
    fn test_agent_config_merge() {
        let mut config1 = AgentConfig::default();
        config1
            .env_vars
            .insert("VAR1".to_string(), "value1".to_string());
        config1.auto_sync = false;

        let mut config2 = AgentConfig::default();
        config2
            .env_vars
            .insert("VAR2".to_string(), "value2".to_string());
        config2.auto_sync = true;

        let merged = config1.merge(&config2);

        assert_eq!(merged.env_vars.get("VAR1"), Some(&"value1".to_string()));
        assert_eq!(merged.env_vars.get("VAR2"), Some(&"value2".to_string()));
        assert!(merged.auto_sync);
    }

    #[test]
    fn test_resource_limits() {
        let limits = ResourceLimits::default();

        assert!(!limits.exceeds_memory_limit(512)); // 512MB < 1GB limit
        assert!(limits.exceeds_memory_limit(2048)); // 2GB > 1GB limit

        assert!(!limits.exceeds_cpu_limit(50.0)); // 50% < 80% limit
        assert!(limits.exceeds_cpu_limit(90.0)); // 90% > 80% limit
    }
}
