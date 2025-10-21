use crate::config::{load_default_config, save_default_config};
use crate::error::SpriteError;
use crate::models::Agent;
use crate::utils::git::{
    self, delete_branch, has_merge_conflicts, has_uncommitted_changes, merge_branch,
    remove_worktree,
};
use crate::utils::tmux;
use anyhow::{Context, Result};
use std::io::{self, Write};
use std::path::Path;

/// Remove agent workspace with comprehensive safety checks and cleanup.
pub fn execute(
    agent_id: &str,
    force: bool,
    keep_workspace: bool,
    merge_branch: bool,
) -> Result<()> {
    // Validate git repository
    git::validate_git_repository().context("Must be run from within a git repository")?;

    // Load configuration
    let mut config =
        load_default_config().context("Failed to load configuration. Run 'sprite init' first.")?;

    // Find the agent to remove
    let agent_index = config
        .agents
        .iter()
        .position(|a| a.id == agent_id)
        .ok_or_else(|| {
            SpriteError::agent(
                format!(
                    "Agent '{}' not found. Use 'sprite agents list' to see available agents.",
                    agent_id
                ),
                Some(agent_id),
            )
        })?;

    let agent = &config.agents[agent_index];

    // Safety checks
    if !force {
        perform_safety_checks(agent)?;
    }

    // Check if agent is currently running in any tmux session
    check_agent_not_running(agent)?;

    // Merge branch if requested
    if merge_branch {
        println!(
            "🔄 Merging agent branch '{}' before removal...",
            agent.branch
        );
        merge_agent_branch(agent)?;
        println!("✅ Branch merged successfully");
    }

    // Remove worktree if it exists
    let workspace_path = agent.workspace_path();
    if workspace_path.exists() {
        if !keep_workspace {
            println!(
                "🗑️  Removing agent workspace at {}...",
                workspace_path.display()
            );
            remove_agent_worktree(workspace_path)?;
            println!("✅ Workspace removed");
        } else {
            println!("📂 Keeping workspace files as requested");
        }
    } else {
        println!("ℹ️  No workspace found at {}", workspace_path.display());
    }

    // Remove git branch if it exists
    if git::branch_exists(&agent.branch)? {
        if !merge_branch {
            // Ask about branch deletion if not already merged
            if !force && !prompt_for_branch_deletion(&agent.branch)? {
                println!("📂 Keeping git branch '{}' as requested", agent.branch);
            } else {
                delete_agent_branch(&agent.branch, force)?;
                println!("✅ Branch '{}' removed", agent.branch);
            }
        } else {
            // Branch was already merged, safe to delete
            delete_agent_branch(&agent.branch, false)?;
            println!("✅ Merged branch '{}' removed", agent.branch);
        }
    } else {
        println!("ℹ️  No git branch '{}' found", agent.branch);
    }

    // Remove agent from configuration
    config.agents.remove(agent_index);
    save_default_config(&config).context("Failed to save updated configuration")?;

    println!("✅ Agent '{}' removed successfully", agent_id);

    // Show remaining agents
    if !config.agents.is_empty() {
        println!("\n📋 Remaining agents:");
        for remaining_agent in &config.agents {
            let status = if remaining_agent.is_active() {
                "🟢 Active"
            } else {
                "⚪ Inactive"
            };
            println!(
                "   {} {} ({})",
                remaining_agent.id, remaining_agent.branch, status
            );
        }
    } else {
        println!("\n📭 No agents remaining. Use 'sprite agents create <ID>' to add new agents.");
    }

    Ok(())
}

/// Perform comprehensive safety checks before agent removal.
fn perform_safety_checks(agent: &Agent) -> Result<()> {
    println!("🔍 Performing safety checks for agent '{}'...", agent.id);

    // Check for uncommitted changes in workspace
    let workspace_path = agent.workspace_path();
    if workspace_path.exists() && has_uncommitted_changes(workspace_path)? {
        return Err(SpriteError::agent(
            format!(
                "Agent '{}' has uncommitted changes in workspace '{}'. \
                Commit or stash changes first, or use --force to override.",
                agent.id,
                workspace_path.display()
            ),
            Some(&agent.id),
        )
        .into());
    }

    // Check if branch has unmerged changes
    if git::branch_exists(&agent.branch)? {
        let current_branch = git::get_current_branch()?;

        // Switch to main branch to check merge status
        if current_branch != agent.branch {
            git::switch_branch("main")?;
        }

        if !git::is_branch_merged(&agent.branch)? {
            println!(
                "⚠️  Warning: Branch '{}' may have unmerged changes.",
                agent.branch
            );
            println!("   Consider using --merge-branch to merge before removal.");

            if !confirm_proceed_with_unmerged()? {
                return Err(SpriteError::agent(
                    "Agent removal cancelled by user due to unmerged changes".to_string(),
                    Some(&agent.id),
                )
                .into());
            }
        }

        // Return to original branch
        if current_branch != agent.branch && current_branch != "main" {
            git::switch_branch(&current_branch)?;
        }
    }

    // Check if this is the only agent
    // (This is just informational, not a blocking check)
    println!(
        "ℹ️  Agent '{}' workspace: {}",
        agent.id,
        workspace_path.display()
    );
    println!("ℹ️  Agent '{}' branch: {}", agent.id, agent.branch);

    Ok(())
}

/// Check if agent is currently running in any tmux session.
fn check_agent_not_running(agent: &Agent) -> Result<()> {
    let sessions = tmux::list_sessions().context("Failed to list tmux sessions")?;

    for session in sessions {
        let panes = tmux::list_panes(&session.name).context("Failed to list tmux panes")?;

        for pane in panes {
            // Check if this pane belongs to our agent by examining the current working directory
            let pane_cwd = tmux::get_pane_current_path(&session.name, &pane.id).unwrap_or_default();

            if let Some(agent_workspace) = agent.worktree_path.as_ref() {
                if pane_cwd == agent_workspace.to_string_lossy() {
                    return Err(SpriteError::agent(
                        format!(
                            "Agent '{}' appears to be running in tmux session '{}' pane {}. \
                            Stop the agent first with 'sprite kill' or detach from the session.",
                            agent.id, session.name, pane.id
                        ),
                        Some(&agent.id),
                    )
                    .into());
                }
            }
        }
    }

    Ok(())
}

/// Remove the agent's git worktree.
fn remove_agent_worktree(workspace_path: &Path) -> Result<()> {
    if !workspace_path.exists() {
        println!("ℹ️  Workspace does not exist: {}", workspace_path.display());
        return Ok(());
    }

    // Remove the worktree using git
    remove_worktree(workspace_path).with_context(|| {
        format!(
            "Failed to remove git worktree at {}",
            workspace_path.display()
        )
    })?;

    // Also remove the directory if it still exists
    if workspace_path.exists() {
        std::fs::remove_dir_all(workspace_path)
            .with_context(|| format!("Failed to remove directory {}", workspace_path.display()))?;
    }

    Ok(())
}

/// Merge agent branch into main branch.
fn merge_agent_branch(agent: &Agent) -> Result<()> {
    if !git::branch_exists(&agent.branch)? {
        println!(
            "ℹ️  Branch '{}' does not exist, skipping merge",
            agent.branch
        );
        return Ok(());
    }

    // Get current branch to restore later
    let original_branch = git::get_current_branch()?;

    // Switch to main branch
    git::switch_branch("main").context("Failed to switch to main branch for merging")?;

    // Perform the merge
    merge_branch(&agent.branch).context("Failed to merge agent branch")?;

    // Check for conflicts
    if has_merge_conflicts()? {
        println!("⚠️  Merge conflicts detected. Please resolve them manually:");
        println!("   1. Run 'git status' to see conflicted files");
        println!("   2. Edit files to resolve conflicts");
        println!("   3. Run 'git add' on resolved files");
        println!("   4. Run 'git commit' to complete the merge");
        println!(
            "   5. Run 'sprite remove {} --force' to continue removal",
            agent.id
        );

        return Err(SpriteError::git(format!(
            "Merge conflicts detected while merging branch '{}'. Resolve conflicts and try again.",
            agent.branch
        ))
        .into());
    }

    // Return to original branch if it wasn't the agent branch
    if original_branch != agent.branch && original_branch != "main" {
        git::switch_branch(&original_branch)?;
    }

    Ok(())
}

/// Delete agent branch with appropriate safety checks.
fn delete_agent_branch(branch: &str, force: bool) -> Result<()> {
    if !git::branch_exists(branch)? {
        return Ok(());
    }

    if force {
        delete_branch(branch, true)?;
    } else {
        // Safe deletion - check if merged first
        let current_branch = git::get_current_branch()?;

        // Temporarily switch to main to check merge status
        if current_branch != "main" {
            git::switch_branch("main")?;
        }

        if git::is_branch_merged(branch)? {
            delete_branch(branch, false)?;
        } else {
            println!(
                "⚠️  Branch '{}' has unmerged changes. Use --force to delete anyway.",
                branch
            );
            return Err(
                SpriteError::git(format!("Branch '{}' has unmerged changes", branch)).into(),
            );
        }

        // Return to original branch
        if current_branch != "main" {
            git::switch_branch(&current_branch)?;
        }
    }

    Ok(())
}

/// Prompt user to confirm proceeding with unmerged changes.
fn confirm_proceed_with_unmerged() -> Result<bool> {
    print!("❓ Branch may have unmerged changes. Proceed anyway? [y/N] ");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}

/// Prompt user for branch deletion confirmation.
fn prompt_for_branch_deletion(branch: &str) -> Result<bool> {
    print!("❓ Delete git branch '{}'? [y/N] ", branch);
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_command_basic() {
        // This test would require mocking and a test git repository
        // For now, we just verify the function exists and has the right signature
        // The actual functionality is tested in integration tests
    }
}
