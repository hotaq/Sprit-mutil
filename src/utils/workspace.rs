//! Workspace navigation utilities for the Sprite multi-agent workflow toolkit.
//!
//! This module provides comprehensive workspace navigation functionality including
//! path resolution, directory traversal, agent workspace discovery, and navigation
//! helpers for efficient movement between workspaces.

use crate::config::{load_default_config, DEFAULT_CONFIG_PATH};
use crate::error::SpriteError;
use crate::utils::git;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// Workspace navigation utilities for managing and navigating between agent workspaces.
#[derive(Debug, Clone)]
pub struct WorkspaceNavigator {
    /// Git repository root
    pub git_root: PathBuf,
    /// Current working directory
    pub current_dir: PathBuf,
    /// Available agent workspaces
    pub agent_workspaces: HashMap<String, PathBuf>,
    /// Main workspace (git root)
    pub main_workspace: PathBuf,
    /// Additional discovered workspaces
    pub discovered_workspaces: HashMap<String, PathBuf>,
}

impl WorkspaceNavigator {
    /// Create a new workspace navigator.
    pub fn new() -> Result<Self> {
        let git_root = git::get_git_root()
            .context("Failed to get git repository root. Are you in a git repository?")?;

        let current_dir = env::current_dir().context("Failed to get current working directory")?;

        let mut navigator = Self {
            main_workspace: git_root.clone(),
            git_root,
            current_dir,
            agent_workspaces: HashMap::new(),
            discovered_workspaces: HashMap::new(),
        };

        navigator.load_agent_workspaces()?;
        navigator.discover_workspaces()?;

        Ok(navigator)
    }

    /// Load agent workspaces from configuration.
    fn load_agent_workspaces(&mut self) -> Result<()> {
        let config_path = PathBuf::from(DEFAULT_CONFIG_PATH);

        if config_path.exists() {
            match load_default_config() {
                Ok(config) => {
                    for agent in &config.agents {
                        if let Some(workspace) = &agent.worktree_path {
                            self.agent_workspaces
                                .insert(agent.id.clone(), workspace.clone());
                        }
                    }
                }
                Err(_) => {
                    // Configuration exists but can't be loaded - continue without agent workspaces
                    eprintln!("⚠️  Warning: Failed to load configuration, agent workspaces may not be available");
                }
            }
        }

        Ok(())
    }

    /// Discover additional workspaces in the repository.
    fn discover_workspaces(&mut self) -> Result<()> {
        let agents_dir = self.git_root.join("agents");

        if agents_dir.exists() && agents_dir.is_dir() {
            let entries = std::fs::read_dir(&agents_dir).with_context(|| {
                format!("Failed to read agents directory: {}", agents_dir.display())
            })?;

            for entry in entries {
                let entry = entry.with_context(|| "Failed to read directory entry")?;
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Only add if not already in agent workspaces
                        if !self.agent_workspaces.contains_key(name) {
                            self.discovered_workspaces.insert(name.to_string(), path);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all available workspaces.
    pub fn get_all_workspaces(&self) -> Vec<WorkspaceInfo> {
        let mut workspaces = Vec::new();

        // Main workspace
        workspaces.push(WorkspaceInfo {
            name: "main".to_string(),
            path: self.main_workspace.clone(),
            workspace_type: WorkspaceType::Main,
            exists: true,
            current: self.current_dir == self.main_workspace,
            relative_path: self.get_relative_path(&self.main_workspace),
        });

        // Agent workspaces
        for (id, path) in &self.agent_workspaces {
            workspaces.push(WorkspaceInfo {
                name: id.clone(),
                path: path.clone(),
                workspace_type: WorkspaceType::Agent,
                exists: path.exists(),
                current: self.current_dir == *path,
                relative_path: self.get_relative_path(path),
            });
        }

        // Discovered workspaces
        for (name, path) in &self.discovered_workspaces {
            workspaces.push(WorkspaceInfo {
                name: name.clone(),
                path: path.clone(),
                workspace_type: WorkspaceType::Discovered,
                exists: true,
                current: self.current_dir == *path,
                relative_path: self.get_relative_path(path),
            });
        }

        // Sort by name for consistent display
        workspaces.sort_by(|a, b| a.name.cmp(&b.name));

        workspaces
    }

    /// Find workspace by name or path.
    pub fn find_workspace(&self, name: &str) -> Option<WorkspaceInfo> {
        // Direct name lookup
        if let Some(path) = self.agent_workspaces.get(name) {
            return Some(WorkspaceInfo {
                name: name.to_string(),
                path: path.clone(),
                workspace_type: WorkspaceType::Agent,
                exists: path.exists(),
                current: self.current_dir == *path,
                relative_path: self.get_relative_path(path),
            });
        }

        // Check discovered workspaces
        if let Some(path) = self.discovered_workspaces.get(name) {
            return Some(WorkspaceInfo {
                name: name.to_string(),
                path: path.clone(),
                workspace_type: WorkspaceType::Discovered,
                exists: true,
                current: self.current_dir == *path,
                relative_path: self.get_relative_path(path),
            });
        }

        // Special case for main
        if name == "main" || name == "root" {
            return Some(WorkspaceInfo {
                name: "main".to_string(),
                path: self.main_workspace.clone(),
                workspace_type: WorkspaceType::Main,
                exists: true,
                current: self.current_dir == self.main_workspace,
                relative_path: self.get_relative_path(&self.main_workspace),
            });
        }

        // Try as a path relative to git root
        let relative_path = self.git_root.join(name);
        if relative_path.exists() {
            return Some(WorkspaceInfo {
                name: name.to_string(),
                path: relative_path.clone(),
                workspace_type: WorkspaceType::Path,
                exists: true,
                current: self.current_dir == relative_path,
                relative_path: self.get_relative_path(&relative_path),
            });
        }

        None
    }

    /// Get the current workspace information.
    pub fn get_current_workspace(&self) -> Option<WorkspaceInfo> {
        // Check if we're in the main workspace
        if self.current_dir == self.main_workspace {
            return Some(WorkspaceInfo {
                name: "main".to_string(),
                path: self.main_workspace.clone(),
                workspace_type: WorkspaceType::Main,
                exists: true,
                current: true,
                relative_path: ".".to_string(),
            });
        }

        // Check agent workspaces
        for (id, path) in &self.agent_workspaces {
            if self.current_dir == *path {
                return Some(WorkspaceInfo {
                    name: id.clone(),
                    path: path.clone(),
                    workspace_type: WorkspaceType::Agent,
                    exists: true,
                    current: true,
                    relative_path: self.get_relative_path(path),
                });
            }
        }

        // Check discovered workspaces
        for (name, path) in &self.discovered_workspaces {
            if self.current_dir == *path {
                return Some(WorkspaceInfo {
                    name: name.clone(),
                    path: path.clone(),
                    workspace_type: WorkspaceType::Discovered,
                    exists: true,
                    current: true,
                    relative_path: self.get_relative_path(path),
                });
            }
        }

        // Check if we're in a subdirectory of any workspace
        if let Some(workspace) = self.find_parent_workspace() {
            return Some(workspace);
        }

        None
    }

    /// Find the parent workspace if current directory is a subdirectory.
    fn find_parent_workspace(&self) -> Option<WorkspaceInfo> {
        // Check main workspace
        if self.current_dir.starts_with(&self.main_workspace) {
            return Some(WorkspaceInfo {
                name: "main".to_string(),
                path: self.main_workspace.clone(),
                workspace_type: WorkspaceType::Main,
                exists: true,
                current: false,
                relative_path: self.get_relative_path(&self.main_workspace),
            });
        }

        // Check agent workspaces
        for (id, path) in &self.agent_workspaces {
            if self.current_dir.starts_with(path) {
                return Some(WorkspaceInfo {
                    name: id.clone(),
                    path: path.clone(),
                    workspace_type: WorkspaceType::Agent,
                    exists: true,
                    current: false,
                    relative_path: self.get_relative_path(path),
                });
            }
        }

        None
    }

    /// Get relative path from current directory to target.
    pub fn get_relative_path(&self, target: &Path) -> String {
        match pathdiff::diff_paths(target, &self.current_dir) {
            Some(relative) => relative.to_string_lossy().to_string(),
            None => target.display().to_string(),
        }
    }

    /// Check if a workspace exists and is accessible.
    #[allow(dead_code)]
    pub fn workspace_exists(&self, name: &str) -> bool {
        self.find_workspace(name).is_some_and(|w| w.exists)
    }

    /// Get navigation command for the specified workspace.
    #[allow(dead_code)]
    pub fn get_navigation_command(&self, workspace_name: &str) -> Result<String> {
        let workspace = self.find_workspace(workspace_name).ok_or_else(|| {
            SpriteError::validation(
                format!("Workspace '{}' not found", workspace_name),
                Some("workspace".to_string()),
                Some(workspace_name.to_string()),
            )
        })?;

        if !workspace.exists {
            return Err(SpriteError::filesystem(
                format!(
                    "Workspace path does not exist: {}",
                    workspace.path.display()
                ),
                workspace_name.to_string(),
            )
            .into());
        }

        Ok(format!("cd {}", workspace.path.display()))
    }

    /// Navigate to workspace (returns command to execute).
    pub fn navigate_to_workspace(&self, workspace_name: &str) -> Result<NavigationResult> {
        let workspace = self.find_workspace(workspace_name).ok_or_else(|| {
            SpriteError::validation(
                format!("Workspace '{}' not found", workspace_name),
                Some("workspace".to_string()),
                Some(workspace_name.to_string()),
            )
        })?;

        if !workspace.exists {
            return Err(SpriteError::filesystem(
                format!(
                    "Workspace path does not exist: {}",
                    workspace.path.display()
                ),
                workspace_name.to_string(),
            )
            .into());
        }

        let current = self.get_current_workspace();

        Ok(NavigationResult {
            workspace: workspace.clone(),
            current_workspace: current,
            command: format!("cd {}", workspace.path.display()),
            relative_command: format!("cd {}", workspace.relative_path),
        })
    }

    /// Generate shell integration script for workspace navigation.
    #[allow(dead_code)]
    pub fn generate_shell_integration(&self) -> String {
        r#"# Sprite workspace navigation integration
# Add this to your shell configuration (.bashrc, .zshrc, etc.)

# Function to navigate to sprite workspaces
sprite_warp() {
    local workspace="$1"
    local cmd
    cmd=$(sprite warp --print "$workspace" 2>/dev/null)
    if [ $? -eq 0 ]; then
        cd "$cmd"
    else
        echo "Error: $cmd"
        return 1
    fi
}

# Tab completion for sprite warp
_sprite_warp_completion() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local workspaces
    workspaces=$(sprite warp --list 2>/dev/null | grep "📁" | awk '{print $3}' | sed 's/://')

    COMPREPLY=($(compgen -W "$workspaces" -- "$cur"))
}

# Register completion
complete -F _sprite_warp_completion sprite_warp
complete -F _sprite_warp_completion sprite warp

# Alias for convenience
alias sw='sprite_warp'
"#
        .to_string()
    }

    /// Print workspace status and navigation suggestions.
    #[allow(dead_code)]
    pub fn print_workspace_status(&self) -> Result<()> {
        let current = self.get_current_workspace();
        let all_workspaces = self.get_all_workspaces();

        println!("🗂️  Workspace Status");
        println!("====================");

        if let Some(current) = &current {
            println!("📍 Current: {} ({})", current.name, current.path.display());
            if current.workspace_type != WorkspaceType::Main {
                println!("   Type: {:?}", current.workspace_type);
            }
        } else {
            println!(
                "📍 Current: {} (outside any workspace)",
                self.current_dir.display()
            );
        }

        println!();
        println!("📋 Available Workspaces:");

        for workspace in &all_workspaces {
            let status = if workspace.current {
                "🟢 current"
            } else if workspace.exists {
                "⚪ available"
            } else {
                "❌ missing"
            };

            let workspace_type = match workspace.workspace_type {
                WorkspaceType::Main => "main",
                WorkspaceType::Agent => "agent",
                WorkspaceType::Discovered => "discovered",
                WorkspaceType::Path => "path",
            };

            println!(
                "   {} {} ({}) - {}",
                status, workspace.name, workspace_type, workspace.relative_path
            );
        }

        println!();
        println!("💡 Navigation:");
        println!("  sprite warp <name>     # Navigate to workspace");
        println!("  sprite warp --list     # List all workspaces");
        println!("  sprite warp --print <name>  # Print path without changing");

        Ok(())
    }
}

/// Information about a workspace.
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// Workspace name
    pub name: String,
    /// Workspace path
    pub path: PathBuf,
    /// Type of workspace
    pub workspace_type: WorkspaceType,
    /// Whether the workspace exists
    pub exists: bool,
    /// Whether this is the current workspace
    pub current: bool,
    /// Relative path from current directory
    pub relative_path: String,
}

/// Type of workspace.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceType {
    /// Main git repository workspace
    Main,
    /// Configured agent workspace
    Agent,
    /// Discovered workspace directory
    Discovered,
    /// Direct path reference
    Path,
}

/// Result of a navigation operation.
#[derive(Debug, Clone)]
pub struct NavigationResult {
    /// Target workspace
    pub workspace: WorkspaceInfo,
    /// Current workspace (if any)
    #[allow(dead_code)]
    pub current_workspace: Option<WorkspaceInfo>,
    /// Absolute navigation command
    pub command: String,
    /// Relative navigation command
    pub relative_command: String,
}

/// Get workspace navigator with current context.
pub fn get_workspace_navigator() -> Result<WorkspaceNavigator> {
    WorkspaceNavigator::new()
}

/// Quick navigation helper - change to workspace and print status.
#[allow(dead_code)]
pub fn quick_warp(workspace_name: &str) -> Result<()> {
    let navigator = WorkspaceNavigator::new()?;
    let result = navigator.navigate_to_workspace(workspace_name)?;

    println!("🚀 Navigating to workspace: {}", result.workspace.name);
    println!("📍 Target: {}", result.workspace.path.display());
    println!("⌨️  Command: {}", result.command);

    if result.workspace.current {
        println!("ℹ️  Already in this workspace");
    } else {
        println!("💡 Execute: cd $(sprite warp --print {})", workspace_name);
    }

    Ok(())
}

/// List all available workspaces in a formatted table.
#[allow(dead_code)]
pub fn list_workspaces_detailed() -> Result<()> {
    let navigator = WorkspaceNavigator::new()?;
    navigator.print_workspace_status()
}

/// Generate workspace aliases for shell integration.
#[allow(dead_code)]
pub fn generate_workspace_aliases() -> Result<String> {
    let navigator = WorkspaceNavigator::new()?;
    let workspaces = navigator.get_all_workspaces();

    let mut aliases = String::new();
    aliases.push_str("# Sprite workspace aliases\n");
    aliases.push_str("# Generated automatically - add to your shell config\n\n");

    for workspace in workspaces {
        if workspace.exists {
            let alias_name = workspace.name.replace('-', "_");
            aliases.push_str(&format!(
                "alias sw{}='cd {}'\n",
                alias_name,
                workspace.path.display()
            ));
        }
    }

    Ok(aliases)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_workspace_navigator_creation() {
        // This test would need a proper git repository setup
        // For now, we just verify the basic structure exists
        // The actual functionality is tested in integration tests
    }

    #[test]
    fn test_workspace_info_creation() {
        let workspace = WorkspaceInfo {
            name: "test".to_string(),
            path: PathBuf::from("/test/path"),
            workspace_type: WorkspaceType::Agent,
            exists: true,
            current: false,
            relative_path: "../test".to_string(),
        };

        assert_eq!(workspace.name, "test");
        assert_eq!(workspace.workspace_type, WorkspaceType::Agent);
        assert!(workspace.exists);
        assert!(!workspace.current);
    }

    #[test]
    fn test_navigation_result() {
        let workspace = WorkspaceInfo {
            name: "test".to_string(),
            path: PathBuf::from("/test/path"),
            workspace_type: WorkspaceType::Agent,
            exists: true,
            current: false,
            relative_path: "../test".to_string(),
        };

        let result = NavigationResult {
            workspace: workspace.clone(),
            current_workspace: None,
            command: "cd /test/path".to_string(),
            relative_command: "cd ../test".to_string(),
        };

        assert_eq!(result.workspace.name, "test");
        assert_eq!(result.command, "cd /test/path");
        assert_eq!(result.relative_command, "cd ../test");
    }
}
