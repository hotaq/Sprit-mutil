//! Configuration validation utilities for the Sprite multi-agent workflow toolkit.
//!
//! This module provides comprehensive validation functions that produce clear,
//! actionable error messages for configuration issues.

use crate::error::SpriteError;
use crate::models::{
    config::SyncHook, Agent, AgentStatus, ConflictResolution, LoggingConfig, PerformanceSettings,
    ProjectConfig, ProjectSettings, SecuritySettings, SyncConfig,
};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Validate configuration with detailed error messages.
///
/// This function provides comprehensive validation of the entire configuration
/// with specific, actionable error messages that help users understand
/// what needs to be fixed.
pub fn validate_config(config: &ProjectConfig) -> Result<()> {
    validate_version(&config.version)?;
    validate_agents(&config.agents)?;
    validate_session_name(&config.session_name)?;
    validate_sync_config(&config.sync)?;
    validate_project_settings(&config.settings)?;

    Ok(())
}

/// Validate configuration version.
fn validate_version(version: &str) -> Result<()> {
    if version.is_empty() {
        return Err(SpriteError::validation(
            "Configuration version is required",
            Some("version".to_string()),
            Some("empty".to_string()),
        )
        .into());
    }

    // Check if version follows semantic versioning pattern
    if !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err(SpriteError::validation(
            "Configuration version must be in semantic versioning format (e.g., '1.0', '1.2.3')",
            Some("version".to_string()),
            Some(version.to_string()),
        )
        .into());
    }

    Ok(())
}

/// Validate agent configurations.
fn validate_agents(agents: &[Agent]) -> Result<()> {
    if agents.is_empty() {
        return Err(SpriteError::validation(
            "At least one agent must be configured",
            Some("agents".to_string()),
            Some("empty list".to_string()),
        )
        .into());
    }

    // Check for duplicate agent IDs
    let mut agent_ids = std::collections::HashSet::new();
    for (index, agent) in agents.iter().enumerate() {
        if !agent_ids.insert(&agent.id) {
            return Err(SpriteError::validation(
                format!("Duplicate agent ID '{}' found at index {}", agent.id, index),
                Some("agents".to_string()),
                Some(agent.id.clone()),
            )
            .into());
        }

        validate_agent(agent).with_context(|| format!("Agent '{}' validation failed", agent.id))?;
    }

    Ok(())
}

/// Validate individual agent configuration.
pub fn validate_agent(agent: &Agent) -> Result<()> {
    // Validate agent ID
    if agent.id.is_empty() {
        return Err(SpriteError::validation(
            "Agent ID cannot be empty",
            Some("agent.id".to_string()),
            Some("empty".to_string()),
        )
        .into());
    }

    if !agent
        .id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(SpriteError::validation(
            "Agent ID can only contain alphanumeric characters, hyphens, and underscores",
            Some("agent.id".to_string()),
            Some(agent.id.clone()),
        )
        .into());
    }

    if agent.id.len() > 50 {
        return Err(SpriteError::validation(
            "Agent ID cannot exceed 50 characters",
            Some("agent.id".to_string()),
            Some(format!("{} characters", agent.id.len())),
        )
        .into());
    }

    // Validate branch name
    if agent.branch.is_empty() {
        return Err(SpriteError::validation(
            "Agent branch cannot be empty",
            Some("agent.branch".to_string()),
            Some("empty".to_string()),
        )
        .into());
    }

    validate_branch_name(&agent.branch)?;

    // Validate workspace path
    let workspace_path = agent.workspace_path();
    validate_workspace_path(workspace_path.as_path())?;

    // Validate agent status consistency
    validate_agent_status(agent)?;

    Ok(())
}

/// Validate branch name format.
fn validate_branch_name(branch: &str) -> Result<()> {
    // Basic branch name validation (git branch naming conventions)
    if branch.contains("..") {
        return Err(SpriteError::validation(
            "Branch name cannot contain '..' (path traversal)",
            Some("branch".to_string()),
            Some(branch.to_string()),
        )
        .into());
    }

    if branch.starts_with('/') || branch.ends_with('/') {
        return Err(SpriteError::validation(
            "Branch name cannot start or end with '/'",
            Some("branch".to_string()),
            Some(branch.to_string()),
        )
        .into());
    }

    if branch.chars().any(|c| c.is_control()) {
        return Err(SpriteError::validation(
            "Branch name cannot contain control characters",
            Some("branch".to_string()),
            Some(branch.to_string()),
        )
        .into());
    }

    Ok(())
}

/// Validate workspace path security and format.
fn validate_workspace_path(path: &std::path::Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Path must be within agents/ directory
    if !path_str.starts_with("agents/") {
        return Err(SpriteError::validation(
            "Agent workspace must be within the 'agents/' directory",
            Some("worktree_path".to_string()),
            Some(path_str.to_string()),
        )
        .into());
    }

    // Check for path traversal attempts
    if path_str.contains("..") {
        return Err(SpriteError::validation(
            "Agent workspace path cannot contain parent directory references",
            Some("worktree_path".to_string()),
            Some(path_str.to_string()),
        )
        .into());
    }

    // Check for absolute paths
    if path.is_absolute() {
        return Err(SpriteError::validation(
            "Agent workspace path must be relative to project root",
            Some("worktree_path".to_string()),
            Some(path_str.to_string()),
        )
        .into());
    }

    Ok(())
}

/// Validate agent status consistency.
fn validate_agent_status(agent: &Agent) -> Result<()> {
    match &agent.status {
        AgentStatus::Error(msg) => {
            if msg.is_empty() {
                return Err(SpriteError::validation(
                    "Agent error status must include an error message",
                    Some("agent.status".to_string()),
                    Some("empty error message".to_string()),
                )
                .into());
            }
        }
        AgentStatus::Initializing => {
            // Check if last_activity should be set during initialization
            if agent.last_activity.is_none() {
                return Err(SpriteError::validation(
                    "Agent in initializing state should have a last_activity timestamp",
                    Some("agent.last_activity".to_string()),
                    Some("not set".to_string()),
                )
                .into());
            }
        }
        _ => {} // Other statuses are valid without additional checks
    }

    Ok(())
}

/// Validate session configuration.
fn validate_session_name(session_name: &str) -> Result<()> {
    if session_name.is_empty() {
        return Err(SpriteError::validation(
            "Session name cannot be empty",
            Some("session_name".to_string()),
            Some("empty".to_string()),
        )
        .into());
    }

    if !session_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(SpriteError::validation(
            "Session name can only contain alphanumeric characters, hyphens, and underscores",
            Some("session_name".to_string()),
            Some(session_name.to_string()),
        )
        .into());
    }

    if session_name.len() > 50 {
        return Err(SpriteError::validation(
            "Session name cannot exceed 50 characters",
            Some("session_name".to_string()),
            Some(format!("{} characters", session_name.len())),
        )
        .into());
    }

    Ok(())
}

/// Validate sync configuration.
fn validate_sync_config(sync: &SyncConfig) -> Result<()> {
    if sync.default_interval_secs == 0 {
        return Err(SpriteError::validation(
            "Sync interval must be greater than 0 seconds",
            Some("sync.default_interval_secs".to_string()),
            Some("0".to_string()),
        )
        .into());
    }

    if sync.default_interval_secs > 86400 {
        return Err(SpriteError::validation(
            "Sync interval cannot exceed 24 hours (86400 seconds)",
            Some("sync.default_interval_secs".to_string()),
            Some(sync.default_interval_secs.to_string()),
        )
        .into());
    }

    // Validate conflict resolution strategy
    validate_conflict_resolution(&sync.conflict_resolution)?;

    // Validate exclude branches
    for (index, branch) in sync.exclude_branches.iter().enumerate() {
        validate_branch_name(branch)
            .with_context(|| format!("Invalid exclude branch at index {}", index))?;
    }

    // Validate hooks
    for (index, hook) in sync.pre_sync_hooks.iter().enumerate() {
        validate_sync_hook(hook)
            .with_context(|| format!("Pre-sync hook {} validation failed", index + 1))?;
    }

    for (index, hook) in sync.post_sync_hooks.iter().enumerate() {
        validate_sync_hook(hook)
            .with_context(|| format!("Post-sync hook {} validation failed", index + 1))?;
    }

    Ok(())
}

/// Validate conflict resolution strategy.
fn validate_conflict_resolution(resolution: &ConflictResolution) -> Result<()> {
    match resolution {
        ConflictResolution::Manual => Ok(()),
        ConflictResolution::AutoTheirs => Ok(()),
        ConflictResolution::AutoOurs => Ok(()),
    }
}

/// Validate sync hook configuration.
fn validate_sync_hook(hook: &SyncHook) -> Result<()> {
    if hook.command.is_empty() {
        return Err(SpriteError::validation(
            "Hook command cannot be empty",
            Some("hook.command".to_string()),
            Some("empty".to_string()),
        )
        .into());
    }

    // Validate command doesn't contain suspicious characters
    if hook.command.contains("..") || hook.command.contains(";") || hook.command.contains("&") {
        return Err(SpriteError::validation(
            "Hook command contains potentially unsafe characters",
            Some("hook.command".to_string()),
            Some(hook.command.clone()),
        )
        .into());
    }

    // Validate working directory if specified
    if let Some(ref work_dir) = hook.work_dir {
        if work_dir.as_os_str().is_empty() {
            return Err(SpriteError::validation(
                "Hook working directory cannot be empty",
                Some("hook.work_dir".to_string()),
                Some("empty path".to_string()),
            )
            .into());
        }

        validate_workspace_path(work_dir.as_path())?;
    }

    // Validate timeout if specified
    if let Some(timeout) = hook.timeout_secs {
        if timeout == 0 {
            return Err(SpriteError::validation(
                "Hook timeout must be greater than 0 seconds",
                Some("hook.timeout_secs".to_string()),
                Some("0".to_string()),
            )
            .into());
        }

        if timeout > 3600 {
            return Err(SpriteError::validation(
                "Hook timeout cannot exceed 1 hour (3600 seconds)",
                Some("hook.timeout_secs".to_string()),
                Some(timeout.to_string()),
            )
            .into());
        }
    }

    Ok(())
}

/// Validate project settings.
fn validate_project_settings(settings: &ProjectSettings) -> Result<()> {
    validate_logging_config(&settings.logging)?;
    validate_performance_settings(&settings.performance)?;
    validate_security_settings(&settings.security)?;

    Ok(())
}

/// Validate logging configuration.
fn validate_logging_config(logging: &LoggingConfig) -> Result<()> {
    if let Some(ref log_file) = logging.log_file {
        let path_str = log_file.to_string_lossy();

        if log_file.is_absolute() {
            return Err(SpriteError::validation(
                "Log file path should be relative to project root",
                Some("logging.log_file".to_string()),
                Some(path_str.to_string()),
            )
            .into());
        }

        if path_str.contains("..") {
            return Err(SpriteError::validation(
                "Log file path cannot contain parent directory references",
                Some("logging.log_file".to_string()),
                Some(path_str.to_string()),
            )
            .into());
        }
    }

    if let Some(ref rotation) = logging.rotation {
        if rotation.max_size_mb == 0 {
            return Err(SpriteError::validation(
                "Maximum log file size must be greater than 0 MB",
                Some("logging.rotation.max_size_mb".to_string()),
                Some("0".to_string()),
            )
            .into());
        }

        if rotation.max_files == 0 {
            return Err(SpriteError::validation(
                "Maximum number of log files must be greater than 0",
                Some("logging.rotation.max_files".to_string()),
                Some("0".to_string()),
            )
            .into());
        }

        if rotation.max_files > 100 {
            return Err(SpriteError::validation(
                "Maximum number of log files cannot exceed 100",
                Some("logging.rotation.max_files".to_string()),
                Some(rotation.max_files.to_string()),
            )
            .into());
        }
    }

    Ok(())
}

/// Validate performance settings.
fn validate_performance_settings(performance: &PerformanceSettings) -> Result<()> {
    if performance.max_concurrent_ops == 0 {
        return Err(SpriteError::validation(
            "Maximum concurrent operations must be greater than 0",
            Some("performance.max_concurrent_ops".to_string()),
            Some("0".to_string()),
        )
        .into());
    }

    if performance.max_concurrent_ops > 100 {
        return Err(SpriteError::validation(
            "Maximum concurrent operations cannot exceed 100",
            Some("performance.max_concurrent_ops".to_string()),
            Some(performance.max_concurrent_ops.to_string()),
        )
        .into());
    }

    if performance.default_timeout_secs == 0 {
        return Err(SpriteError::validation(
            "Default timeout must be greater than 0 seconds",
            Some("performance.default_timeout_secs".to_string()),
            Some("0".to_string()),
        )
        .into());
    }

    if performance.default_timeout_secs > 3600 {
        return Err(SpriteError::validation(
            "Default timeout cannot exceed 1 hour (3600 seconds)",
            Some("performance.default_timeout_secs".to_string()),
            Some(performance.default_timeout_secs.to_string()),
        )
        .into());
    }

    if let Some(memory_limit) = performance.memory_limit_mb {
        if memory_limit == 0 {
            return Err(SpriteError::validation(
                "Memory limit must be greater than 0 MB",
                Some("performance.memory_limit_mb".to_string()),
                Some("0".to_string()),
            )
            .into());
        }

        if memory_limit > 16384 {
            return Err(SpriteError::validation(
                "Memory limit cannot exceed 16 GB (16384 MB)",
                Some("performance.memory_limit_mb".to_string()),
                Some(memory_limit.to_string()),
            )
            .into());
        }
    }

    if let Some(cpu_limit) = performance.cpu_limit_percent {
        if cpu_limit == 0 || cpu_limit > 100 {
            return Err(SpriteError::validation(
                "CPU limit must be between 1 and 100 percent",
                Some("performance.cpu_limit_percent".to_string()),
                Some(cpu_limit.to_string()),
            )
            .into());
        }
    }

    Ok(())
}

/// Validate security settings.
fn validate_security_settings(security: &SecuritySettings) -> Result<()> {
    if let Some(max_length) = security.max_command_length {
        if max_length == 0 {
            return Err(SpriteError::validation(
                "Maximum command length must be greater than 0",
                Some("security.max_command_length".to_string()),
                Some("0".to_string()),
            )
            .into());
        }

        if max_length > 10000 {
            return Err(SpriteError::validation(
                "Maximum command length cannot exceed 10000 characters",
                Some("security.max_command_length".to_string()),
                Some(max_length.to_string()),
            )
            .into());
        }
    }

    // Validate allowed paths
    for (index, path) in security.allowed_paths.iter().enumerate() {
        if path.as_os_str().is_empty() {
            return Err(SpriteError::validation(
                "Allowed path cannot be empty",
                Some(format!("security.allowed_paths[{}]", index)),
                Some("empty path".to_string()),
            )
            .into());
        }

        if !path.starts_with("agents/") {
            return Err(SpriteError::validation(
                "Allowed paths should be within the agents/ directory",
                Some(format!("security.allowed_paths[{}]", index)),
                Some(path.to_string_lossy().to_string()),
            )
            .into());
        }
    }

    // Validate blocked commands
    for (index, command) in security.blocked_commands.iter().enumerate() {
        if command.is_empty() {
            return Err(SpriteError::validation(
                "Blocked command cannot be empty",
                Some(format!("security.blocked_commands[{}]", index)),
                Some("empty command".to_string()),
            )
            .into());
        }

        // Check for obviously dangerous commands in blocked list
        let dangerous_commands = ["rm", "sudo", "su", "chmod", "chown", "dd"];
        for dangerous in dangerous_commands.iter() {
            if command.starts_with(dangerous) || command.ends_with(dangerous) {
                break; // Good, this is appropriately blocked
            } else if index == security.blocked_commands.len() - 1 {
                // If we're at the last command and none are dangerous, warn
                return Err(SpriteError::validation(
                    "Consider adding dangerous commands to blocked list (e.g., 'rm', 'sudo', 'su')",
                    Some("security.blocked_commands".to_string()),
                    Some("insufficient protection".to_string()),
                )
                .into());
            }
        }
    }

    Ok(())
}

/// Validate configuration against current git repository state.
pub fn validate_config_with_git(config: &ProjectConfig) -> Result<()> {
    // First, do basic validation
    validate_config(config)?;

    // Then validate against git state
    crate::config::validate_git_repository()
        .context("Configuration validation requires a valid git repository")?;

    // Check that all agent branches exist
    for agent in &config.agents {
        if !crate::config::branch_exists(&agent.branch)? {
            return Err(SpriteError::agent(
                format!(
                    "Agent branch '{}' does not exist. Create it with 'git checkout -b {}'",
                    agent.branch, agent.branch
                ),
                Some(&agent.id),
            )
            .into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_validate_version() {
        assert!(validate_version("1.0").is_ok());
        assert!(validate_version("2.1.3").is_ok());
        assert!(validate_version("").is_err());
        assert!(validate_version("1.0-beta").is_err());
    }

    #[test]
    fn test_validate_agent() {
        let mut agent = Agent::new("agent-1", "feature/test");
        assert!(validate_agent(&agent).is_ok());

        // Test invalid ID
        agent.id = "invalid agent!".to_string();
        assert!(validate_agent(&agent).is_err());

        // Reset and test invalid branch
        agent = Agent::new("valid-agent", "");
        assert!(validate_agent(&agent).is_err());
    }

    #[test]
    fn test_validate_branch_name() {
        assert!(validate_branch_name("feature/test").is_ok());
        assert!(validate_branch_name("hotfix/issue-123").is_ok());
        assert!(validate_branch_name("../dangerous").is_err());
        assert!(validate_branch_name("/absolute").is_err());
        assert!(validate_branch_name("trailing/").is_err());
    }

    #[test]
    fn test_validate_workspace_path() {
        assert!(validate_workspace_path(PathBuf::from("agents/agent-1").as_path()).is_ok());
        assert!(validate_workspace_path(PathBuf::from("agents/workspace").as_path()).is_ok());
        assert!(validate_workspace_path(PathBuf::from("outside").as_path()).is_err());
        assert!(validate_workspace_path(PathBuf::from("agents/../dangerous").as_path()).is_err());
        assert!(validate_workspace_path(PathBuf::from("/absolute/path").as_path()).is_err());
    }

    #[test]
    fn test_validate_security_settings() {
        let security = SecuritySettings::default();
        assert!(validate_security_settings(&security).is_ok());
    }
}
