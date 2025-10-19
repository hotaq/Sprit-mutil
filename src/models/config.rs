//! Configuration data structures for the Sprite multi-agent workflow toolkit.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use super::{Agent, LogLevel, ConflictResolution};

/// Top-level configuration that defines the entire multi-agent setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Configuration version for migration support
    pub version: String,

    /// List of configured agents
    pub agents: Vec<Agent>,

    /// Session configuration name (references a tmux profile)
    pub session_name: String,

    /// Synchronization settings
    pub sync: SyncConfig,

    /// Global project settings
    pub settings: ProjectSettings,
}

impl ProjectConfig {
    /// Create a new project configuration with default values.
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            agents: Vec::new(),
            session_name: "default".to_string(),
            sync: SyncConfig::default(),
            settings: ProjectSettings::default(),
        }
    }

    /// Add an agent to the configuration.
    pub fn add_agent(&mut self, agent: Agent) -> Result<(), String> {
        // Check for duplicate agent IDs
        if self.agents.iter().any(|a| a.id == agent.id) {
            return Err(format!("Agent with ID '{}' already exists", agent.id));
        }

        // Validate the agent
        agent.validate()?;
        self.agents.push(agent);
        Ok(())
    }

    /// Remove an agent by ID.
    pub fn remove_agent(&mut self, agent_id: &str) -> Option<Agent> {
        let index = self.agents.iter().position(|a| a.id == agent_id)?;
        Some(self.agents.remove(index))
    }

    /// Get an agent by ID.
    pub fn get_agent(&self, agent_id: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == agent_id)
    }

    /// Get an agent by ID (mutable).
    pub fn get_agent_mut(&mut self, agent_id: &str) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id == agent_id)
    }

    /// Get the number of configured agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Get active agents only.
    pub fn active_agents(&self) -> Vec<&Agent> {
        self.agents.iter().filter(|a| a.is_active()).collect()
    }

    /// Validate the entire configuration.
    pub fn validate(&self) -> Result<(), String> {
        // Validate version
        if self.version.is_empty() {
            return Err("Configuration version cannot be empty".to_string());
        }

        // Validate agents
        if self.agents.is_empty() {
            return Err("At least one agent must be configured".to_string());
        }

        // Check for duplicate agent IDs
        let mut agent_ids = std::collections::HashSet::new();
        for agent in &self.agents {
            if !agent_ids.insert(&agent.id) {
                return Err(format!("Duplicate agent ID: {}", agent.id));
            }
            agent.validate()?;
        }

        // Validate session name
        if self.session_name.is_empty() {
            return Err("Session name cannot be empty".to_string());
        }

        // Validate sync config
        self.sync.validate()?;

        // Validate settings
        self.settings.validate()?;

        Ok(())
    }

    /// Get the configuration file path.
    pub fn config_path() -> PathBuf {
        PathBuf::from("agents/agents.yaml")
    }

    /// Load configuration from a file.
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read configuration file: {}", e))?;

        let config: ProjectConfig = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse configuration: {}", e))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a file.
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        // Validate before saving
        self.validate()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize configuration: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write configuration file: {}", e))?;

        Ok(())
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Synchronization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Whether to automatically sync agents
    pub auto_sync: bool,

    /// Default sync interval in seconds
    pub default_interval_secs: u64,

    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,

    /// Branches to exclude from auto-sync
    pub exclude_branches: Vec<String>,

    /// Pre-sync hooks
    pub pre_sync_hooks: Vec<SyncHook>,

    /// Post-sync hooks
    pub post_sync_hooks: Vec<SyncHook>,
}

impl SyncConfig {
    /// Validate the sync configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.default_interval_secs == 0 {
            return Err("Sync interval must be greater than 0".to_string());
        }

        // Validate hooks
        for (i, hook) in self.pre_sync_hooks.iter().enumerate() {
            hook.validate().map_err(|e| format!("Pre-sync hook {}: {}", i + 1, e))?;
        }

        for (i, hook) in self.post_sync_hooks.iter().enumerate() {
            hook.validate().map_err(|e| format!("Post-sync hook {}: {}", i + 1, e))?;
        }

        Ok(())
    }

    /// Check if a branch should be excluded from sync.
    pub fn should_exclude_branch(&self, branch: &str) -> bool {
        self.exclude_branches.iter().any(|excluded| {
            if excluded.ends_with('*') {
                branch.starts_with(&excluded[..excluded.len() - 1])
            } else {
                excluded == branch
            }
        })
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_sync: false,
            default_interval_secs: 300, // 5 minutes
            conflict_resolution: ConflictResolution::default(),
            exclude_branches: vec!["main".to_string(), "master".to_string()],
            pre_sync_hooks: Vec::new(),
            post_sync_hooks: Vec::new(),
        }
    }
}

/// Hook configuration for sync operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHook {
    /// Hook command to execute
    pub command: String,

    /// Arguments for the command
    pub args: Vec<String>,

    /// Working directory for the hook
    pub work_dir: Option<PathBuf>,

    /// Whether hook failure should stop sync
    pub required: bool,

    /// Timeout for the hook in seconds
    pub timeout_secs: Option<u64>,
}

impl SyncHook {
    /// Validate the hook configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.command.is_empty() {
            return Err("Hook command cannot be empty".to_string());
        }

        if let Some(ref work_dir) = self.work_dir {
            if work_dir.as_os_str().is_empty() {
                return Err("Hook working directory cannot be empty".to_string());
            }
        }

        Ok(())
    }

    /// Execute the hook.
    pub fn execute(&self) -> Result<std::process::Output, String> {
        use std::process::Command;

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref work_dir) = self.work_dir {
            cmd.current_dir(work_dir);
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute hook command '{}': {}", self.command, e))?;

        Ok(output)
    }
}

/// Global project settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Default shell for agent operations
    pub default_shell: String,

    /// Global environment variables
    pub global_env_vars: HashMap<String, String>,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Performance settings
    pub performance: PerformanceSettings,

    /// Security settings
    pub security: SecuritySettings,
}

impl ProjectSettings {
    /// Validate the project settings.
    pub fn validate(&self) -> Result<(), String> {
        if self.default_shell.is_empty() {
            return Err("Default shell cannot be empty".to_string());
        }

        self.logging.validate()?;
        self.performance.validate()?;
        self.security.validate()?;

        Ok(())
    }

    /// Get effective environment variables (global + agent-specific).
    pub fn get_effective_env(&self, agent_env: &HashMap<String, String>) -> HashMap<String, String> {
        let mut env = self.global_env_vars.clone();
        for (key, value) in agent_env {
            env.insert(key.clone(), value.clone());
        }
        env
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            default_shell: "bash".to_string(),
            global_env_vars: HashMap::new(),
            logging: LoggingConfig::default(),
            performance: PerformanceSettings::default(),
            security: SecuritySettings::default(),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log file location
    pub log_file: Option<PathBuf>,

    /// Log level
    pub level: LogLevel,

    /// Whether to log to stdout
    pub log_to_stdout: bool,

    /// Log rotation settings
    pub rotation: Option<LogRotation>,
}

impl LoggingConfig {
    /// Validate the logging configuration.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref log_file) = self.log_file {
            if log_file.as_os_str().is_empty() {
                return Err("Log file path cannot be empty".to_string());
            }
        }

        if let Some(ref rotation) = self.rotation {
            rotation.validate()?;
        }

        Ok(())
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_file: Some(PathBuf::from("agents/logs/sprite.log")),
            level: LogLevel::default(),
            log_to_stdout: true,
            rotation: Some(LogRotation::default()),
        }
    }
}

/// Log rotation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    /// Maximum log file size in MB
    pub max_size_mb: u64,

    /// Maximum number of log files to keep
    pub max_files: u32,
}

impl LogRotation {
    /// Validate the log rotation settings.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_size_mb == 0 {
            return Err("Maximum log file size must be greater than 0".to_string());
        }

        if self.max_files == 0 {
            return Err("Maximum number of log files must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl Default for LogRotation {
    fn default() -> Self {
        Self {
            max_size_mb: 100, // 100MB
            max_files: 5,
        }
    }
}

/// Performance settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Maximum concurrent operations
    pub max_concurrent_ops: u32,

    /// Timeout for operations in seconds
    pub default_timeout_secs: u64,

    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,

    /// Memory limit in MB
    pub memory_limit_mb: Option<u64>,

    /// CPU limit as percentage
    pub cpu_limit_percent: Option<u8>,
}

impl PerformanceSettings {
    /// Validate the performance settings.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_concurrent_ops == 0 {
            return Err("Maximum concurrent operations must be greater than 0".to_string());
        }

        if self.default_timeout_secs == 0 {
            return Err("Default timeout must be greater than 0".to_string());
        }

        if let Some(cpu_limit) = self.cpu_limit_percent {
            if cpu_limit == 0 || cpu_limit > 100 {
                return Err("CPU limit must be between 1 and 100".to_string());
            }
        }

        Ok(())
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            max_concurrent_ops: 10,
            default_timeout_secs: 300, // 5 minutes
            enable_monitoring: true,
            memory_limit_mb: Some(1024), // 1GB
            cpu_limit_percent: Some(80),  // 80%
        }
    }
}

/// Security settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Allowed workspace paths
    pub allowed_paths: Vec<PathBuf>,

    /// Blocked commands
    pub blocked_commands: Vec<String>,

    /// Whether to validate paths strictly
    pub strict_path_validation: bool,

    /// Maximum command length
    pub max_command_length: Option<usize>,

    /// Whether to allow shell command execution
    pub allow_shell_execution: bool,
}

impl SecuritySettings {
    /// Validate the security settings.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(max_length) = self.max_command_length {
            if max_length == 0 {
                return Err("Maximum command length must be greater than 0".to_string());
            }
        }

        for path in &self.allowed_paths {
            if path.as_os_str().is_empty() {
                return Err("Allowed paths cannot contain empty paths".to_string());
            }
        }

        Ok(())
    }

    /// Check if a command is blocked.
    pub fn is_command_blocked(&self, command: &str) -> bool {
        self.blocked_commands.iter().any(|blocked| {
            if blocked.ends_with('*') {
                command.starts_with(&blocked[..blocked.len() - 1])
            } else {
                // Check exact match or if command starts with blocked command
                blocked == command || command.starts_with(&format!("{} ", blocked))
            }
        })
    }

    /// Check if a path is allowed.
    pub fn is_path_allowed(&self, path: &PathBuf) -> bool {
        if !self.strict_path_validation {
            return true;
        }

        if self.allowed_paths.is_empty() {
            return true; // If no restrictions, allow all
        }

        self.allowed_paths.iter().any(|allowed| {
            path.starts_with(allowed) || path == allowed
        })
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            allowed_paths: vec![PathBuf::from("agents/")],
            blocked_commands: vec![
                "rm -rf".to_string(),
                "sudo".to_string(),
                "su".to_string(),
                "chmod 777".to_string(),
            ],
            strict_path_validation: true,
            max_command_length: Some(1000),
            allow_shell_execution: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AgentStatus;

    #[test]
    fn test_project_config_creation() {
        let config = ProjectConfig::new();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.agents.len(), 0);
        assert_eq!(config.settings.default_shell, "bash");
    }

    #[test]
    fn test_add_agent() {
        let mut config = ProjectConfig::new();
        let agent = Agent::new("agent-1", "feature/test");

        assert!(config.add_agent(agent.clone()).is_ok());
        assert_eq!(config.agent_count(), 1);

        // Test duplicate agent
        assert!(config.add_agent(agent).is_err());
    }

    #[test]
    fn test_remove_agent() {
        let mut config = ProjectConfig::new();
        let agent = Agent::new("agent-1", "feature/test");

        config.add_agent(agent).unwrap();
        assert_eq!(config.agent_count(), 1);

        let removed = config.remove_agent("agent-1");
        assert!(removed.is_some());
        assert_eq!(config.agent_count(), 0);

        // Test removing non-existent agent
        let removed = config.remove_agent("non-existent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_config_validation() {
        let mut config = ProjectConfig::new();

        // Empty config should fail (no agents)
        assert!(config.validate().is_err());

        // Add an agent
        let agent = Agent::new("agent-1", "feature/test");
        config.add_agent(agent).unwrap();

        // Now should be valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_sync_config() {
        let sync = SyncConfig::default();
        assert!(!sync.auto_sync);
        assert_eq!(sync.default_interval_secs, 300);
        assert!(sync.should_exclude_branch("main"));
        assert!(!sync.should_exclude_branch("feature/test"));
    }

    #[test]
    fn test_security_settings() {
        let security = SecuritySettings::default();
        assert!(security.is_command_blocked("rm -rf"));
        assert!(security.is_command_blocked("sudo ls"));
        assert!(!security.is_command_blocked("git status"));
        assert!(security.is_path_allowed(&PathBuf::from("agents/agent-1")));
        assert!(!security.is_path_allowed(&PathBuf::from("/etc/passwd")));
    }

    #[test]
    fn test_sync_hook() {
        let hook = SyncHook {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            work_dir: None,
            required: false,
            timeout_secs: Some(30),
        };

        assert!(hook.validate().is_ok());

        let invalid_hook = SyncHook {
            command: "".to_string(),
            args: vec![],
            work_dir: None,
            required: false,
            timeout_secs: None,
        };

        assert!(invalid_hook.validate().is_err());
    }
}