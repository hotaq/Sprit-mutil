use anyhow::{Context, Result};
use crate::error::SpriteError;
use crate::cli::ConfigCommands;
use crate::utils::git;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Simple agent configuration matching the YAML format from init command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimpleAgentConfig {
    pub branch: String,
    pub worktree_path: String,
    pub model: String,
    pub description: String,
}

/// Configuration file structure matching the init command output
/// Configuration change detection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub last_modified: u64,
    pub content_hash: String,
    pub version: u64,
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        Self {
            last_modified: 0,
            content_hash: String::new(),
            version: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteConfig {
    pub agents: HashMap<String, SimpleAgentConfig>,
    pub session: SessionConfig,
    pub sync: SyncConfig,
    #[serde(skip)]
    metadata: ConfigMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionConfig {
    pub name: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncConfig {
    pub auto_sync: bool,
    pub conflict_resolution: String,
    pub exclude_branches: Vec<String>,
}

impl SpriteConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            session: SessionConfig {
                name: "sprite-session".to_string(),
                profile: "profile0".to_string(),
            },
            sync: SyncConfig {
                auto_sync: false,
                conflict_resolution: "manual".to_string(),
                exclude_branches: Vec::new(),
            },
            metadata: ConfigMetadata::default(),
        }
    }
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

        let mut config: SpriteConfig = serde_yaml::from_str(&content)
            .map_err(|e| SpriteError::config(format!(
                "Failed to parse configuration file {}: {}",
                path.display(),
                e
            )))?;

        // Update metadata
        config.update_metadata(path, &content)?;

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

    /// Update metadata with current file information
    fn update_metadata(&mut self, path: &PathBuf, content: &str) -> Result<()> {
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to read file metadata: {}", path.display()))?;

        let last_modified = metadata
            .modified()
            .with_context(|| format!("Failed to get modification time: {}", path.display()))?
            .duration_since(UNIX_EPOCH)
            .with_context(|| "System time before UNIX epoch")?
            .as_secs();

        // Simple content hash using SHA-256 (via std)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = format!("{:x}", hasher.finish());

        self.metadata = ConfigMetadata {
            last_modified,
            content_hash,
            version: self.metadata.version + 1,
        };

        Ok(())
    }

    /// Check if the configuration file has changed since last load
    pub fn has_changed(&self) -> Result<bool> {
        let config_path = PathBuf::from("agents/agents.yaml");

        if !config_path.exists() {
            return Ok(false);
        }

        let current_metadata = std::fs::metadata(&config_path)
            .with_context(|| format!("Failed to read file metadata: {}", config_path.display()))?;

        let current_modified = current_metadata
            .modified()
            .with_context(|| format!("Failed to get modification time: {}", config_path.display()))?
            .duration_since(UNIX_EPOCH)
            .with_context(|| "System time before UNIX epoch")?
            .as_secs();

        Ok(current_modified != self.metadata.last_modified)
    }

    /// Get the current configuration version
    pub fn version(&self) -> u64 {
        self.metadata.version
    }

    /// Get the last modification time
    pub fn last_modified(&self) -> u64 {
        self.metadata.last_modified
    }

    /// Get the content hash
    pub fn content_hash(&self) -> &str {
        &self.metadata.content_hash
    }

    /// Reload configuration if it has changed
    pub fn reload_if_changed(&mut self) -> Result<bool> {
        if !self.has_changed()? {
            return Ok(false);
        }

        println!("🔄 Configuration file changed, reloading...");

        // Store old version for comparison
        let old_version = self.metadata.version;

        // Reload from disk
        let config_path = PathBuf::from("agents/agents.yaml");
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read configuration file: {}", config_path.display()))?;

        let new_config: SpriteConfig = serde_yaml::from_str(&content)
            .map_err(|e| SpriteError::config(format!(
                "Failed to parse configuration file {}: {}",
                config_path.display(),
                e
            )))?;

        // Update current config
        self.agents = new_config.agents;
        self.session = new_config.session;
        self.sync = new_config.sync;

        // Update metadata
        self.update_metadata(&config_path, &content)?;

        println!("✅ Configuration reloaded (version {} → {})", old_version, self.metadata.version);
        Ok(true)
    }

    /// Detect and report configuration changes
    pub fn detect_changes(&self) -> Result<ConfigChanges> {
        let config_path = PathBuf::from("agents/agents.yaml");

        if !config_path.exists() {
            return Ok(ConfigChanges::new());
        }

        let current_metadata = std::fs::metadata(&config_path)
            .with_context(|| format!("Failed to read file metadata: {}", config_path.display()))?;

        let current_modified = current_metadata
            .modified()
            .with_context(|| format!("Failed to get modification time: {}", config_path.display()))?
            .duration_since(UNIX_EPOCH)
            .with_context(|| "System time before UNIX epoch")?
            .as_secs();

        if current_modified == self.metadata.last_modified {
            return Ok(ConfigChanges::new());
        }

        // Load current config to compare
        let current_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read configuration file: {}", config_path.display()))?;

        let current_config: SpriteConfig = serde_yaml::from_str(&current_content)
            .map_err(|e| SpriteError::config(format!(
                "Failed to parse configuration file {}: {}",
                config_path.display(),
                e
            )))?;

        // Detect changes
        let mut changes = ConfigChanges::new();
        changes.file_modified = true;
        changes.old_version = self.metadata.version;
        changes.new_version = self.metadata.version + 1;

        // Compare agents
        for (agent_id, old_agent) in &self.agents {
            if let Some(new_agent) = current_config.agents.get(agent_id) {
                if old_agent != new_agent {
                    changes.modified_agents.push(agent_id.clone());
                }
            } else {
                changes.removed_agents.push(agent_id.clone());
            }
        }

        for (agent_id, _) in &current_config.agents {
            if !self.agents.contains_key(agent_id) {
                changes.added_agents.push(agent_id.clone());
            }
        }

        // Compare session
        if self.session != current_config.session {
            changes.session_changed = true;
        }

        // Compare sync
        if self.sync != current_config.sync {
            changes.sync_changed = true;
        }

        Ok(changes)
    }
}

/// Configuration change detection results
#[derive(Debug, Clone)]
pub struct ConfigChanges {
    pub file_modified: bool,
    pub old_version: u64,
    pub new_version: u64,
    pub added_agents: Vec<String>,
    pub removed_agents: Vec<String>,
    pub modified_agents: Vec<String>,
    pub session_changed: bool,
    pub sync_changed: bool,
}

impl ConfigChanges {
    pub fn new() -> Self {
        Self {
            file_modified: false,
            old_version: 0,
            new_version: 0,
            added_agents: Vec::new(),
            removed_agents: Vec::new(),
            modified_agents: Vec::new(),
            session_changed: false,
            sync_changed: false,
        }
    }

    pub fn has_changes(&self) -> bool {
        self.file_modified ||
        !self.added_agents.is_empty() ||
        !self.removed_agents.is_empty() ||
        !self.modified_agents.is_empty() ||
        self.session_changed ||
        self.sync_changed
    }

    pub fn print_summary(&self) {
        if !self.has_changes() {
            println!("✅ No configuration changes detected.");
            return;
        }

        println!("🔄 Configuration changes detected (v{} → v{}):", self.old_version, self.new_version);

        if !self.added_agents.is_empty() {
            println!("  ➕ Added agents: {}", self.added_agents.join(", "));
        }

        if !self.removed_agents.is_empty() {
            println!("  ➖ Removed agents: {}", self.removed_agents.join(", "));
        }

        if !self.modified_agents.is_empty() {
            println!("  ✏️  Modified agents: {}", self.modified_agents.join(", "));
        }

        if self.session_changed {
            println!("  🎮 Session configuration changed");
        }

        if self.sync_changed {
            println!("  🔄 Sync configuration changed");
        }
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
        ConfigCommands::Status => check_config_status(),
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

/// Check configuration status and detect changes
fn check_config_status() -> Result<()> {
    println!("🔍 Checking configuration status...");

    let config = SpriteConfig::load()
        .context("Failed to load configuration")?;

    println!("📋 Configuration Status:");
    println!("  Version: {}", config.version());
    println!("  Last Modified: {}", {
        use std::time::{UNIX_EPOCH, Duration};
        let datetime = UNIX_EPOCH + Duration::from_secs(config.last_modified());
        let time_str = datetime.duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Simple formatting of timestamp
        let hours = time_str / 3600;
        let minutes = (time_str % 3600) / 60;
        let seconds = time_str % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    });
    println!("  Content Hash: {}...", &config.content_hash()[..8.min(config.content_hash().len())]);

    // Check for changes
    let changes = config.detect_changes()
        .context("Failed to detect configuration changes")?;

    if changes.has_changes() {
        println!();
        changes.print_summary();

        // If agents were added or modified, suggest provisioning
        if !changes.added_agents.is_empty() || !changes.modified_agents.is_empty() {
            println!();
            println!("💡 Suggestion: Run 'sprite agents provision' to create/update workspaces");
        }

        // If agents were removed, suggest cleanup
        if !changes.removed_agents.is_empty() {
            println!();
            println!("💡 Suggestion: Check if workspaces for removed agents need cleanup");
        }
    } else {
        println!("  ✅ Configuration is up to date");
    }

    // Show workspace status
    println!();
    println!("🔧 Workspace Status:");
    match config.validate_workspaces() {
        Ok(_) => println!("  ✅ All workspaces are valid"),
        Err(e) => println!("  ⚠️  Workspace validation failed: {}", e),
    }

    Ok(())
}