use anyhow::{Context, Result};
use crate::error::SpriteError;
use crate::cli::ConfigCommands;
use crate::utils::git;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple agent configuration matching the YAML format from init command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleAgentConfig {
    pub branch: String,
    pub worktree_path: String,
    pub model: String,
    pub description: String,
}

/// Configuration file structure matching the init command output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteConfig {
    pub agents: HashMap<String, SimpleAgentConfig>,
    pub session: SessionConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub name: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub auto_sync: bool,
    pub conflict_resolution: String,
    pub exclude_branches: Vec<String>,
}

impl SpriteConfig {
    /// Load configuration from the default path
    pub fn load() -> Result<Self> {
        let config_path = PathBuf::from("agents/agents.yaml");
        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(SpriteError::config(format!(
                "Configuration file not found: {}",
                path.display()
            )).into());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;

        let config: SpriteConfig = serde_yaml::from_str(&content)
            .map_err(|e| SpriteError::config(format!(
                "Failed to parse configuration file {}: {}",
                path.display(),
                e
            )))?;

        Ok(config)
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        let config_path = PathBuf::from("agents/agents.yaml");
        self.save_to_path(&config_path)
    }

    /// Save configuration to a specific path
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        // Validate before saving
        self.validate()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|e| SpriteError::config(format!("Failed to serialize configuration: {}", e)))?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write configuration file: {}", path.display()))?;

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.agents.is_empty() {
            return Err(SpriteError::config("At least one agent must be configured").into());
        }

        for (agent_id, agent_config) in &self.agents {
            if agent_id.is_empty() {
                return Err(SpriteError::config("Agent ID cannot be empty").into());
            }

            if agent_config.branch.is_empty() {
                return Err(SpriteError::config(format!(
                    "Agent {} branch cannot be empty", agent_id
                )).into());
            }

            if agent_config.worktree_path.is_empty() {
                return Err(SpriteError::config(format!(
                    "Agent {} worktree_path cannot be empty", agent_id
                )).into());
            }

            if agent_config.model.is_empty() {
                return Err(SpriteError::config(format!(
                    "Agent {} model cannot be empty", agent_id
                )).into());
            }
        }

        if self.session.name.is_empty() {
            return Err(SpriteError::config("Session name cannot be empty").into());
        }

        if self.session.profile.is_empty() {
            return Err(SpriteError::config("Session profile cannot be empty").into());
        }

        Ok(())
    }

    /// Get all agent IDs
    pub fn agent_ids(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Get agent configuration by ID
    pub fn get_agent(&self, agent_id: &str) -> Option<&SimpleAgentConfig> {
        self.agents.get(agent_id)
    }

    /// Add or update an agent
    pub fn set_agent(&mut self, agent_id: String, config: SimpleAgentConfig) {
        self.agents.insert(agent_id, config);
    }

    /// Remove an agent
    pub fn remove_agent(&mut self, agent_id: &str) -> Option<SimpleAgentConfig> {
        self.agents.remove(agent_id)
    }

    /// Provision worktrees for all configured agents
    pub fn provision_worktrees(&self) -> Result<()> {
        println!("🔧 Provisioning worktrees for {} agents...", self.agents.len());

        // Validate git repository first
        git::validate_git_repository()?;

        for (agent_id, agent_config) in &self.agents {
            println!("  📁 Setting up Agent {}...", agent_id);

            let worktree_path = PathBuf::from(&agent_config.worktree_path);

            // Create worktree
            git::create_worktree(&worktree_path, &agent_config.branch)
                .with_context(|| format!("Failed to create worktree for agent {}", agent_id))?;

            println!("    ✅ Created worktree: {}", worktree_path.display());
            println!("    🌿 Branch: {}", agent_config.branch);
        }

        println!("✅ Worktree provisioning complete!");
        Ok(())
    }

    /// Validate that all agent worktrees exist and are properly configured
    pub fn validate_workspaces(&self) -> Result<()> {
        println!("🔍 Validating agent workspaces...");

        let worktrees = git::list_worktrees()
            .context("Failed to list existing worktrees")?;

        let worktree_paths: std::collections::HashSet<_> = worktrees
            .iter()
            .map(|w| w.path.clone())
            .collect();

        for (agent_id, agent_config) in &self.agents {
            let worktree_path = PathBuf::from(&agent_config.worktree_path);

            if !worktree_paths.contains(&worktree_path) {
                return Err(SpriteError::config(format!(
                    "Agent {} worktree does not exist: {}",
                    agent_id,
                    worktree_path.display()
                )).into());
            }

            // Check if branch exists
            if !git::branch_exists(&agent_config.branch)? {
                return Err(SpriteError::config(format!(
                    "Agent {} branch does not exist: {}",
                    agent_id,
                    agent_config.branch
                )).into());
            }

            println!("  ✅ Agent {}: {} ({})", agent_id, worktree_path.display(), agent_config.branch);
        }

        println!("✅ All workspaces are valid!");
        Ok(())
    }
}

/// Execute config command with subcommands
pub fn execute(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => show_config(),
        ConfigCommands::Validate => validate_config(),
        ConfigCommands::Set { key, value } => set_config_value(key, value),
        ConfigCommands::Get { key } => get_config_value(key),
        ConfigCommands::Edit => edit_config(),
    }
}

/// Show current configuration
fn show_config() -> Result<()> {
    let config = SpriteConfig::load()
        .context("Failed to load configuration")?;

    println!("📋 Sprite Configuration");
    println!();

    // Show agents
    println!("🤖 Agents ({} total):", config.agents.len());
    for (agent_id, agent_config) in &config.agents {
        println!("  {}:", agent_id);
        println!("    Branch: {}", agent_config.branch);
        println!("    Workspace: {}", agent_config.worktree_path);
        println!("    Model: {}", agent_config.model);
        println!("    Description: {}", agent_config.description);
        println!();
    }

    // Show session
    println!("🎮 Session:");
    println!("  Name: {}", config.session.name);
    println!("  Profile: {}", config.session.profile);
    println!();

    // Show sync settings
    println!("🔄 Sync:");
    println!("  Auto-sync: {}", config.sync.auto_sync);
    println!("  Conflict resolution: {}", config.sync.conflict_resolution);
    println!("  Exclude branches: {:?}", config.sync.exclude_branches);

    Ok(())
}

/// Validate configuration
fn validate_config() -> Result<()> {
    println!("🔍 Validating Sprite configuration...");

    let config = SpriteConfig::load()
        .context("Failed to load configuration")?;

    // Validate configuration structure
    config.validate()
        .context("Configuration validation failed")?;

    println!("✅ Configuration structure is valid!");

    // Validate workspaces
    config.validate_workspaces()
        .context("Workspace validation failed")?;

    println!("✅ Configuration is fully valid!");
    Ok(())
}

/// Set configuration value
fn set_config_value(key: String, value: String) -> Result<()> {
    let mut config = SpriteConfig::load()
        .context("Failed to load configuration")?;

    // Parse key in format: section.field or section.item.field
    let parts: Vec<&str> = key.split('.').collect();
    let value_clone = value.clone();

    match parts.as_slice() {
        ["session", "name"] => {
            config.session.name = value;
        }
        ["session", "profile"] => {
            config.session.profile = value;
        }
        ["sync", "auto_sync"] => {
            config.sync.auto_sync = value.parse::<bool>()
                .map_err(|_| SpriteError::config("auto_sync must be true or false"))?;
        }
        ["sync", "conflict_resolution"] => {
            config.sync.conflict_resolution = value;
        }
        ["agents", agent_id, field] => {
            let agent_config = config.agents.entry(agent_id.to_string())
                .or_insert_with(|| SimpleAgentConfig {
                    branch: format!("agents/{}", agent_id),
                    worktree_path: format!("agents/{}", agent_id),
                    model: "claude-sonnet-4".to_string(),
                    description: format!("Agent {} workspace", agent_id),
                });

            match *field {
                "branch" => agent_config.branch = value,
                "worktree_path" => agent_config.worktree_path = value,
                "model" => agent_config.model = value,
                "description" => agent_config.description = value,
                _ => return Err(SpriteError::config(format!(
                    "Unknown agent field: {}", field
                )).into()),
            }
        }
        _ => return Err(SpriteError::config(format!(
            "Unknown configuration key: {}", key
        )).into()),
    }

    config.save()
        .context("Failed to save configuration")?;

    println!("✅ Configuration updated: {} = {}", key, value_clone);
    Ok(())
}

/// Get configuration value
fn get_config_value(key: String) -> Result<()> {
    let config = SpriteConfig::load()
        .context("Failed to load configuration")?;

    let parts: Vec<&str> = key.split('.').collect();

    let value = match parts.as_slice() {
        ["session", "name"] => Some(config.session.name.clone()),
        ["session", "profile"] => Some(config.session.profile.clone()),
        ["sync", "auto_sync"] => Some(config.sync.auto_sync.to_string()),
        ["sync", "conflict_resolution"] => Some(config.sync.conflict_resolution.clone()),
        ["agents", agent_id, field] => {
            config.agents.get(&**agent_id).and_then(|agent| {
                match *field {
                    "branch" => Some(agent.branch.clone()),
                    "worktree_path" => Some(agent.worktree_path.clone()),
                    "model" => Some(agent.model.clone()),
                    "description" => Some(agent.description.clone()),
                    _ => None,
                }
            })
        }
        _ => None,
    };

    match value {
        Some(v) => println!("{}: {}", key, v),
        None => return Err(SpriteError::config(format!(
            "Unknown configuration key: {}", key
        )).into()),
    }

    Ok(())
}

/// Edit configuration file
fn edit_config() -> Result<()> {
    let config_path = PathBuf::from("agents/agents.yaml");

    if !config_path.exists() {
        return Err(SpriteError::config(format!(
            "Configuration file not found: {}",
            config_path.display()
        )).into());
    }

    // Try to find a suitable editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Try common editors
            if std::path::Path::new("/usr/bin/nano").exists() {
                "nano".to_string()
            } else if std::path::Path::new("/usr/bin/vim").exists() {
                "vim".to_string()
            } else if std::path::Path::new("/usr/bin/vi").exists() {
                "vi".to_string()
            } else {
                "emacs".to_string()
            }
        });

    println!("📝 Opening configuration file in {}...", editor);

    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()
        .with_context(|| format!("Failed to open editor: {}", editor))?;

    if !status.success() {
        return Err(SpriteError::process(
            "editor".to_string(),
            status.code(),
            "".to_string(),
            format!("Editor exited with non-zero status: {}", status)
        ).into());
    }

    // Validate after editing
    println!("🔍 Validating updated configuration...");
    let config = SpriteConfig::load()
        .context("Failed to load updated configuration")?;

    config.validate()
        .context("Updated configuration is invalid")?;

    println!("✅ Configuration updated and validated successfully!");
    Ok(())
}