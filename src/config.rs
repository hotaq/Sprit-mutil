//! Configuration management for the Sprite multi-agent workflow toolkit.
//!
//! This module provides functions for loading, saving, and validating YAML
//! configuration files using the data models defined in the models module.

use crate::error::SpriteError;
use crate::models::{Agent, ProjectConfig};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Default configuration file path
#[allow(dead_code)]
pub const DEFAULT_CONFIG_PATH: &str = "agents/agents.yaml";

/// Load configuration from a file.
///
/// This function loads a YAML configuration file, validates it, and returns
/// a ProjectConfig instance. It provides clear error messages for common
/// configuration issues.
#[allow(dead_code)]
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ProjectConfig> {
    let path = path.as_ref();

    // Check if file exists
    if !path.exists() {
        return Err(SpriteError::config_with_source(
            format!("Configuration file not found: {}", path.display()),
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        )
        .into());
    }

    // Read file content
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;

    // Parse YAML
    let config: ProjectConfig = serde_yaml::from_str(&content).map_err(|e| {
        SpriteError::yaml(
            format!("Failed to parse YAML configuration: {}", e),
            e.location().map(|loc| loc.line()),
            e.location().map(|loc| loc.column()),
        )
    })?;

    // Validate configuration
    config
        .validate()
        .map_err(|e| SpriteError::config(format!("Configuration validation failed: {}", e)))?;

    Ok(config)
}

/// Save configuration to a file.
///
/// This function saves a ProjectConfig instance to a YAML file, creating
/// the parent directory if necessary.
#[allow(dead_code)]
pub fn save_config<P: AsRef<Path>>(config: &ProjectConfig, path: P) -> Result<()> {
    let path = path.as_ref();

    // Validate configuration before saving
    config
        .validate()
        .map_err(|e| SpriteError::config(format!("Cannot save invalid configuration: {}", e)))?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create configuration directory: {}",
                parent.display()
            )
        })?;
    }

    // Serialize to YAML
    let content = serde_yaml::to_string(config)
        .map_err(|e| SpriteError::config(format!("Failed to serialize configuration: {}", e)))?;

    // Write to file
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write configuration file: {}", path.display()))?;

    Ok(())
}

/// Create a default configuration with example agents.
///
/// This function creates a sample configuration with three agents that
/// users can modify for their own needs.
#[allow(dead_code)]
pub fn create_default_config() -> ProjectConfig {
    let mut agents = Vec::new();

    // Agent 1: Frontend development
    agents.push(Agent::new("1", "feature/frontend-work"));
    if let Some(agent) = agents.last_mut() {
        agent.model = Some("claude-sonnet-4".to_string());
        agent.description = Some("Frontend development specialist".to_string());
        agent.config.custom_settings.insert(
            "role".to_string(),
            serde_yaml::Value::String("frontend".to_string()),
        );
        agent
            .config
            .startup_commands
            .push("npm install".to_string());
        agent
            .config
            .startup_commands
            .push("npm run dev".to_string());
    }

    // Agent 2: Backend development
    agents.push(Agent::new("2", "feature/backend-work"));
    if let Some(agent) = agents.last_mut() {
        agent.model = Some("claude-sonnet-4".to_string());
        agent.description = Some("Backend development specialist".to_string());
        agent.config.custom_settings.insert(
            "role".to_string(),
            serde_yaml::Value::String("backend".to_string()),
        );
        agent
            .config
            .startup_commands
            .push("cargo build".to_string());
        agent.config.startup_commands.push("cargo run".to_string());
    }

    // Agent 3: Testing and QA
    agents.push(Agent::new("3", "feature/testing-work"));
    if let Some(agent) = agents.last_mut() {
        agent.model = Some("claude-sonnet-4".to_string());
        agent.description = Some("Testing and QA specialist".to_string());
        agent.config.custom_settings.insert(
            "role".to_string(),
            serde_yaml::Value::String("testing".to_string()),
        );
        agent.config.startup_commands.push("cargo test".to_string());
    }

    ProjectConfig {
        version: "1.0".to_string(),
        agents,
        session_name: "sprite-session".to_string(),
        sync: crate::models::SyncConfig::default(),
        settings: crate::models::ProjectSettings::default(),
    }
}

/// Initialize a new configuration file.
///
/// This function creates a new default configuration and saves it to the
/// specified path. It will fail if the file already exists unless force is true.
#[allow(dead_code)]
pub fn init_config<P: AsRef<Path>>(path: P, force: bool) -> Result<ProjectConfig> {
    let path = path.as_ref();

    // Check if file already exists
    if path.exists() && !force {
        return Err(SpriteError::config(format!(
            "Configuration file already exists: {}. Use --force to overwrite.",
            path.display()
        ))
        .into());
    }

    let config = create_default_config();
    save_config(&config, path)?;

    Ok(config)
}

/// Load configuration from the default location.
///
/// This is a convenience function that loads configuration from the
/// standard agents/agents.yaml location.
#[allow(dead_code)]
pub fn load_default_config() -> Result<ProjectConfig> {
    load_config(DEFAULT_CONFIG_PATH)
}

/// Save configuration to the default location.
///
/// This is a convenience function that saves configuration to the
/// standard agents/agents.yaml location.
#[allow(dead_code)]
pub fn save_default_config(config: &ProjectConfig) -> Result<()> {
    save_config(config, DEFAULT_CONFIG_PATH)
}

/// Validate that we're in a git repository.
///
/// This function checks if the current directory is a git repository,
/// which is required for Sprite to function properly.
#[allow(dead_code)]
pub fn validate_git_repository() -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .with_context(|| "Failed to execute git command. Is git installed?")?;

    if !output.status.success() {
        return Err(SpriteError::config(
            "Not a git repository. Sprite must be run from within a git repository.",
        )
        .into());
    }

    Ok(())
}

/// Get the current git branch.
///
/// This function returns the name of the current git branch.
#[allow(dead_code)]
pub fn get_current_branch() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .with_context(|| "Failed to get current git branch")?;

    if !output.status.success() {
        return Err(SpriteError::git("Failed to get current branch").into());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

/// Check if a git branch exists.
///
/// This function checks if a branch with the given name exists in the repository.
#[allow(dead_code)]
pub fn branch_exists(branch: &str) -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
        .output()
        .with_context(|| "Failed to check if branch exists")?;

    Ok(output.status.success())
}

/// Get the git repository root directory.
///
/// This function returns the absolute path to the git repository root.
#[allow(dead_code)]
pub fn get_git_root() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .with_context(|| "Failed to get git repository root")?;

    if !output.status.success() {
        return Err(SpriteError::git("Failed to get git repository root").into());
    }

    let root_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(root_str))
}

/// Validate that an agent configuration is compatible with the current git repository.
#[allow(dead_code)]
pub fn validate_agent_config(agent: &Agent) -> Result<()> {
    // Check if the agent's branch exists
    if !branch_exists(&agent.branch)? {
        return Err(SpriteError::agent(
            format!(
                "Branch '{}' does not exist. Create it first with 'git checkout -b {}'",
                agent.branch, agent.branch
            ),
            Some(&agent.id),
        )
        .into());
    }

    // Check if the workspace path is within the git repository
    let git_root = get_git_root()?;
    let workspace_path = agent.workspace_path();

    if !workspace_path.starts_with(&git_root) {
        return Err(SpriteError::agent(
            format!(
                "Agent workspace '{}' must be within the git repository",
                workspace_path.display()
            ),
            Some(&agent.id),
        )
        .into());
    }

    Ok(())
}

/// Migrate configuration from an older version.
///
/// This function handles configuration migrations between versions.
#[allow(dead_code)]
pub fn migrate_config(
    _config: &mut ProjectConfig,
    from_version: &str,
    to_version: &str,
) -> Result<()> {
    if from_version == to_version {
        return Ok(());
    }

    match (from_version, to_version) {
        ("1.0", "1.0") => Ok(()),
        _ => Err(SpriteError::config(format!(
            "Unsupported migration from version {} to {}",
            from_version, to_version
        ))
        .into()),
    }
}

/// Get configuration statistics.
///
/// This function returns useful statistics about the configuration.
#[allow(dead_code)]
pub fn get_config_stats(config: &ProjectConfig) -> HashMap<String, serde_yaml::Value> {
    let mut stats = HashMap::new();

    stats.insert(
        "version".to_string(),
        serde_yaml::Value::String(config.version.clone()),
    );
    stats.insert(
        "agent_count".to_string(),
        serde_yaml::Value::Number(config.agents.len().into()),
    );
    stats.insert(
        "session_name".to_string(),
        serde_yaml::Value::String(config.session_name.clone()),
    );
    stats.insert(
        "auto_sync".to_string(),
        serde_yaml::Value::Bool(config.sync.auto_sync),
    );

    let active_agents = config.agents.iter().filter(|a| a.is_active()).count();
    stats.insert(
        "active_agents".to_string(),
        serde_yaml::Value::Number(active_agents.into()),
    );

    let error_agents = config.agents.iter().filter(|a| a.has_error()).count();
    stats.insert(
        "error_agents".to_string(),
        serde_yaml::Value::Number(error_agents.into()),
    );

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_default_config() {
        let config = create_default_config();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.agents.len(), 3);
        assert_eq!(config.session_name, "sprite-session");
        assert!(!config.sync.auto_sync);
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        let original_config = create_default_config();
        save_config(&original_config, &config_path).unwrap();

        let loaded_config = load_config(&config_path).unwrap();
        assert_eq!(original_config.version, loaded_config.version);
        assert_eq!(original_config.agents.len(), loaded_config.agents.len());
        assert_eq!(original_config.session_name, loaded_config.session_name);
    }

    #[test]
    fn test_load_nonexistent_config() {
        let result = load_config("nonexistent.yaml");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_init_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("new_config.yaml");

        let config = init_config(&config_path, false).unwrap();
        assert!(config_path.exists());
        assert_eq!(config.agents.len(), 3);

        // Test that it fails when file already exists
        let result = init_config(&config_path, false);
        assert!(result.is_err());

        // Test that it succeeds with force
        let config2 = init_config(&config_path, true).unwrap();
        assert_eq!(config2.agents.len(), 3);
    }

    #[test]
    fn test_validate_git_repository() {
        // This test will fail if not run in a git repository
        // In CI, it should be run from within a git repo
        let _result = validate_git_repository();
        // We don't assert success here since it depends on the test environment
    }

    #[test]
    fn test_get_config_stats() {
        let config = create_default_config();
        let stats = get_config_stats(&config);

        assert_eq!(stats.get("version").unwrap().as_str().unwrap(), "1.0");
        assert_eq!(stats.get("agent_count").unwrap().as_i64().unwrap(), 3);
        assert_eq!(stats.get("active_agents").unwrap().as_i64().unwrap(), 0);
        assert_eq!(stats.get("error_agents").unwrap().as_i64().unwrap(), 0);
    }

    #[test]
    fn test_invalid_config() {
        let invalid_yaml = "invalid: yaml: content:";
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.yaml");
        fs::write(&config_path, invalid_yaml).unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to parse YAML"));
    }
}
