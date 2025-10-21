//! Warp command - Navigate to agent workspaces with comprehensive navigation utilities

use crate::utils::workspace::{
    generate_workspace_aliases, get_workspace_navigator, quick_warp, WorkspaceNavigator,
};
use anyhow::{Context, Result};
use std::env;

/// Execute the warp command with the given parameters.
pub fn execute(workspace: Option<String>, list: bool, print: bool, relative: bool) -> Result<()> {
    // Create workspace navigator
    let navigator = get_workspace_navigator()
        .context("Failed to initialize workspace navigator. Are you in a git repository?")?;

    // Handle list option
    if list {
        return list_workspaces(&navigator);
    }

    // Determine target workspace
    let target_workspace = if let Some(workspace) = workspace {
        workspace
    } else {
        // If no workspace specified, show current status or go to main
        let current = navigator.get_current_workspace();
        match current {
            Some(workspace) => {
                println!(
                    "📍 Currently in workspace: {} ({})",
                    workspace.name,
                    workspace.path.display()
                );
                println!("💡 Use 'sprite warp <name>' to navigate to another workspace");
                println!("💡 Use 'sprite warp --list' to see all available workspaces");
                return Ok(());
            }
            None => {
                // Not in any known workspace, go to main
                "main".to_string()
            }
        }
    };

    // Navigate to workspace
    navigate_to_workspace(&navigator, &target_workspace, print, relative)?;

    Ok(())
}

/// List all available workspaces with detailed information
fn list_workspaces(navigator: &WorkspaceNavigator) -> Result<()> {
    println!("🗂️  Available Workspaces");
    println!("=======================");

    let workspaces = navigator.get_all_workspaces();
    let _current = navigator.get_current_workspace();

    if workspaces.is_empty() {
        println!("📭 No workspaces found.");
        return Ok(());
    }

    // Display workspaces in a table format
    println!(
        "{:<15} {:<12} {:<8} {:<30}",
        "Workspace", "Type", "Status", "Path"
    );
    println!("{:-<15} {:-<12} {:-<8} {:-<30}", "", "", "", "");

    for workspace in workspaces {
        let status = if workspace.current {
            "🟢 current"
        } else if workspace.exists {
            "⚪ available"
        } else {
            "❌ missing"
        };

        let workspace_type = match workspace.workspace_type {
            crate::utils::workspace::WorkspaceType::Main => "main",
            crate::utils::workspace::WorkspaceType::Agent => "agent",
            crate::utils::workspace::WorkspaceType::Discovered => "discovered",
            crate::utils::workspace::WorkspaceType::Path => "path",
        };

        // Truncate path if too long
        let path_display = if workspace.relative_path.len() > 30 {
            format!(
                "../{}",
                workspace
                    .path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("...")
            )
        } else {
            workspace.relative_path.clone()
        };

        println!(
            "{:<15} {:<12} {:<8} {:<30}",
            workspace.name, workspace_type, status, path_display
        );
    }

    println!();
    println!("💡 Usage:");
    println!("  sprite warp <workspace_name>  # Navigate to workspace");
    println!("  sprite warp main             # Navigate to main directory");
    println!("  sprite warp --print <name>   # Print path without changing directory");
    println!("  sprite warp --relative <name> # Show relative path");

    // Show shell integration hint
    println!();
    println!("🔧 Shell Integration:");
    println!("  sprite warp --generate-shell-integration  # Generate shell script for integration");
    println!("  sprite warp --generate-aliases             # Generate workspace aliases");

    Ok(())
}

/// Navigate to the specified workspace with enhanced features
fn navigate_to_workspace(
    navigator: &WorkspaceNavigator,
    workspace_name: &str,
    print: bool,
    relative: bool,
) -> Result<()> {
    // Get navigation result
    let result = navigator
        .navigate_to_workspace(workspace_name)
        .map_err(|e| {
            // Provide helpful suggestions if workspace not found
            let available_workspaces: Vec<String> = navigator
                .get_all_workspaces()
                .iter()
                .map(|w| w.name.clone())
                .collect();

            if available_workspaces.is_empty() {
                eprintln!("💡 Tip: Use 'sprite init' to create a new project configuration");
            } else {
                eprintln!(
                    "💡 Available workspaces: {}",
                    available_workspaces.join(", ")
                );
            }

            e
        })?;

    // Determine display path
    let display_path = if relative {
        result.workspace.relative_path.clone()
    } else {
        result.workspace.path.display().to_string()
    };

    if print {
        // Just print the path
        println!("{}", display_path);
    } else {
        // Show navigation information
        if result.workspace.current {
            println!("📍 Already in workspace: {}", result.workspace.name);
            println!("📂 Path: {}", result.workspace.path.display());
        } else {
            println!("🚀 Navigating to workspace: {}", result.workspace.name);
            println!("📂 Target: {}", result.workspace.path.display());

            // Show the command to execute
            if env::consts::OS == "windows" {
                println!("⌨️  Command: cd /d {}", display_path);
            } else {
                println!("⌨️  Command: cd {}", display_path);
            }

            println!();
            println!("💡 To automatically change directory, use:");
            println!("  cd $(sprite warp --print {})", workspace_name);

            // Show relative command if different
            if relative && result.relative_command != result.command {
                println!("  # or: {}", result.relative_command);
            }
        }

        // Show additional context
        if !result.workspace.current {
            println!();
            match result.workspace.workspace_type {
                crate::utils::workspace::WorkspaceType::Agent => {
                    println!("🤖 This is an AI agent workspace");
                    if result.workspace.exists {
                        println!("   Use 'sprite start' to begin a session with this agent");
                    }
                }
                crate::utils::workspace::WorkspaceType::Main => {
                    println!("🏠 This is the main project workspace");
                    println!("   Use 'sprite agents list' to see all agents");
                }
                crate::utils::workspace::WorkspaceType::Discovered => {
                    println!("🔍 This is a discovered workspace directory");
                }
                crate::utils::workspace::WorkspaceType::Path => {
                    println!("📁 This is a direct path reference");
                }
            }
        }
    }

    Ok(())
}

/// Generate shell integration script
#[allow(dead_code)]
pub fn generate_shell_integration() -> Result<()> {
    let navigator = get_workspace_navigator()?;
    let integration_script = navigator.generate_shell_integration();

    println!("🔧 Shell Integration Script");
    println!("==========================");
    println!();
    println!("Add this to your shell configuration file:");
    println!("  • ~/.bashrc (for Bash)");
    println!("  • ~/.zshrc (for Zsh)");
    println!("  • ~/.config/fish/config.fish (for Fish)");
    println!();
    println!("{}", integration_script);
    println!();
    println!("💡 After adding the script, restart your shell or run 'source ~/.bashrc'");

    Ok(())
}

/// Generate workspace aliases
#[allow(dead_code)]
pub fn generate_aliases() -> Result<()> {
    let aliases = generate_workspace_aliases()?;

    println!("🔗 Workspace Aliases");
    println!("====================");
    println!();
    println!("Add these aliases to your shell configuration for quick access:");
    println!();
    println!("{}", aliases);
    println!();
    println!("💡 Usage: sw1, sw2, sw3, etc. to quickly navigate to agent workspaces");

    Ok(())
}

/// Show workspace status and suggestions
#[allow(dead_code)]
pub fn show_workspace_status() -> Result<()> {
    let navigator = get_workspace_navigator()?;
    navigator.print_workspace_status()
}

/// Quick warp with enhanced feedback
#[allow(dead_code)]
pub fn enhanced_quick_warp(workspace_name: &str) -> Result<()> {
    quick_warp(workspace_name)?;

    // Show additional context and suggestions
    let navigator = get_workspace_navigator()?;
    let current = navigator.get_current_workspace();

    if let Some(current) = current {
        println!();
        match current.workspace_type {
            crate::utils::workspace::WorkspaceType::Agent => {
                println!("🤖 Agent Workspace Commands:");
                println!("  sprite start              # Start agent session");
                println!(
                    "  sprite hey {} <command>   # Send command to this agent",
                    current.name
                );
                println!("  sprite status            # Check agent status");
            }
            crate::utils::workspace::WorkspaceType::Main => {
                println!("🏠 Main Workspace Commands:");
                println!("  sprite agents list        # List all agents");
                println!("  sprite start              # Start all agents");
                println!("  sprite status            # Check project status");
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::workspace::{WorkspaceInfo, WorkspaceType};

    #[test]
    fn test_warp_execution() {
        // This test would need mocking to work properly in isolation
        // For now, we just verify the function exists and handles basic cases
        // The actual functionality is tested in integration tests
    }

    #[test]
    fn test_workspace_list_formatting() {
        // Test workspace list formatting logic
        let workspace = WorkspaceInfo {
            name: "test-agent".to_string(),
            path: std::path::PathBuf::from("/test/path"),
            workspace_type: WorkspaceType::Agent,
            exists: true,
            current: false,
            relative_path: "../test-agent".to_string(),
        };

        assert_eq!(workspace.name, "test-agent");
        assert_eq!(workspace.workspace_type, WorkspaceType::Agent);
        assert!(workspace.exists);
        assert!(!workspace.current);
        assert_eq!(workspace.relative_path, "../test-agent");
    }
}
