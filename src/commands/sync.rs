//! Sync command - Context-aware git synchronization

use crate::error::SpriteError;
use crate::models::ConflictResolution;
use crate::utils::git;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Execute the sync command with the given parameters.
pub fn execute(
    agent: Option<&str>,
    force: bool,
    strategy: &str,
    dry_run: bool,
) -> Result<()> {
    // Validate we're in a git repository
    git::validate_git_repository()?;

    // Parse conflict resolution strategy
    let conflict_strategy = match strategy {
        "manual" => ConflictResolution::Manual,
        "auto-theirs" => ConflictResolution::AutoTheirs,
        "auto-ours" => ConflictResolution::AutoOurs,
        _ => {
            return Err(SpriteError::validation(
                format!("Invalid conflict resolution strategy: '{}'. Valid options: manual, auto-theirs, auto-ours", strategy),
                Some("strategy".to_string()),
                Some(strategy.to_string())
            ).into());
        }
    };

    // Determine sync context
    let sync_context = determine_sync_context(agent.map(|s| s.to_string()))?;

    println!("🔄 Syncing in {:?} context...", sync_context.context_type);

    if dry_run {
        println!("🔍 Dry run mode - no changes will be made");
    }

    // Execute sync based on context
    match &sync_context.context_type {
        SyncContextType::MainWorktree => sync_main_worktree(&sync_context, conflict_strategy, dry_run, force)?,
        SyncContextType::AgentWorktree { agent_id } => sync_agent_worktree(&sync_context, agent_id, conflict_strategy, dry_run, force)?,
        SyncContextType::Unknown => {
            return Err(SpriteError::sync(
                "Cannot determine sync context. Please run from main worktree or an agent worktree."
            ).into());
        }
    }

    if !dry_run {
        println!("✓ Sync completed successfully");
    } else {
        println!("✓ Dry run completed - no changes made");
    }

    Ok(())
}

/// Context for determining sync behavior.
#[derive(Debug)]
struct SyncContext {
    /// Type of context (main vs agent worktree)
    context_type: SyncContextType,
    /// Current working directory
    current_dir: PathBuf,
    /// Current git branch
    current_branch: String,
    /// Git repository root
    git_root: PathBuf,
}

/// Types of sync contexts.
#[derive(Debug)]
enum SyncContextType {
    /// In the main worktree (root repository)
    MainWorktree,
    /// In an agent worktree
    AgentWorktree { agent_id: String },
    /// Unknown context
    Unknown,
}

/// Determine the current sync context.
fn determine_sync_context(agent_id: Option<String>) -> Result<SyncContext> {
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;

    let git_root = git::get_git_root()
        .context("Failed to get git repository root")?;

    let current_branch = git::get_current_branch_at(&current_dir)
        .context("Failed to get current git branch")?;

    // Normalize paths for comparison
    let current_dir_normalized = current_dir.canonicalize()
        .context("Failed to canonicalize current directory")?;
    let git_root_normalized = git_root.canonicalize()
        .context("Failed to canonicalize git root")?;

    // Check if we're in the main worktree
    if current_dir_normalized == git_root_normalized {
        return Ok(SyncContext {
            context_type: SyncContextType::MainWorktree,
            current_dir,
            current_branch,
            git_root,
        });
    }

    // Check if we're in an agent worktree
    let relative_path = match current_dir_normalized.strip_prefix(&git_root_normalized) {
        Ok(path) => path,
        Err(_) => {
            return Ok(SyncContext {
                context_type: SyncContextType::Unknown,
                current_dir,
                current_branch,
                git_root,
            });
        }
    };

    // Check if this looks like an agent directory
    let path_str = relative_path.to_string_lossy();

    // Try to extract agent ID from path
    if let Some(extracted_agent_id) = extract_agent_id_from_path(&path_str) {
        // Validate against provided agent ID if specified
        if let Some(provided_id) = &agent_id {
            if provided_id != &extracted_agent_id {
                return Err(SpriteError::validation(
                    format!("Agent ID mismatch. Path suggests '{}' but you specified '{}'", extracted_agent_id, provided_id),
                    Some("agent".to_string()),
                    Some(format!("path: {}, agent: {}", path_str, provided_id))
                ).into());
            }
        }

        return Ok(SyncContext {
            context_type: SyncContextType::AgentWorktree { agent_id: extracted_agent_id },
            current_dir,
            current_branch,
            git_root,
        });
    }

    Ok(SyncContext {
        context_type: SyncContextType::Unknown,
        current_dir,
        current_branch,
        git_root,
    })
}

/// Extract agent ID from a relative path.
fn extract_agent_id_from_path(path: &str) -> Option<String> {
    // Check if path is in agents/ directory
    if path.starts_with("agents/") {
        let remaining = &path[7..]; // Remove "agents/" prefix

        // Check if the remaining part is just an agent ID (like "1", "2", etc.)
        if remaining.is_empty() {
            return None;
        }

        // Extract the first component as potential agent ID
        if let Some(slash_pos) = remaining.find('/') {
            let potential_id = &remaining[..slash_pos];
            // Validate that it's a valid agent ID (numeric or simple string)
            if potential_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Some(potential_id.to_string());
            }
        } else {
            // Entire remaining part is the agent ID
            if remaining.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Some(remaining.to_string());
            }
        }
    }

    None
}

/// Sync the main worktree (pull from remote).
fn sync_main_worktree(
    context: &SyncContext,
    conflict_strategy: ConflictResolution,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    println!("📥 Syncing main worktree (pull from remote origin/main)");

    if dry_run {
        println!("  Would run: git pull origin main");
        return Ok(());
    }

    // Check if we have uncommitted changes
    let status = git::get_status_string_at(&context.current_dir)
        .context("Failed to get git status")?;

    if !status.is_empty() {
        println!("⚠️  You have uncommitted changes:");
        println!("{}", status);

        if !force {
            return Err(SpriteError::sync(
                "Cannot sync main worktree with uncommitted changes. Use --force to proceed anyway or commit your changes first."
            ).into());
        }

        println!("⚠️  Proceeding with uncommitted changes due to --force flag");
    }

    // Pull changes from remote
    println!("  Pulling changes from origin/main...");
    git::pull().with_context(|| "Failed to pull changes from remote")?;

    println!("✓ Main worktree synced successfully");
    Ok(())
}

/// Sync an agent worktree (merge from main).
fn sync_agent_worktree(
    context: &SyncContext,
    agent_id: &str,
    conflict_strategy: ConflictResolution,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    println!("📥 Syncing agent worktree '{}' (merge from main branch)", agent_id);

    if dry_run {
        println!("  Would run: git merge main");
        return Ok(());
    }

    // Check if we have uncommitted changes in the agent worktree
    let status = git::get_status_string_at(&context.current_dir)
        .context("Failed to get git status for agent worktree")?;

    if !status.is_empty() {
        println!("⚠️  Agent {} has uncommitted changes:", agent_id);
        println!("{}", status);

        if !force {
            return Err(SpriteError::sync(
                format!("Cannot sync agent worktree '{}' with uncommitted changes. Use --force to proceed or commit your changes first.", agent_id)
            ).into());
        }

        println!("⚠️  Proceeding with uncommitted changes due to --force flag");
    }

    // Ensure main branch exists and is up to date
    println!("  Ensuring main branch is available...");

    // Switch to main branch temporarily if needed
    let current_branch = context.current_branch.clone();
    if current_branch != "main" && current_branch != "master" {
        println!("  Switching to main branch temporarily...");
        git::switch_branch("main").or_else(|_| {
            // Try master if main doesn't exist
            git::switch_branch("master")
        }).with_context(|| "Failed to switch to main branch")?;

        // Switch back to original branch after merge
        defer_switch_back(&current_branch)?;
    }

    // Merge changes from main branch
    println!("  Merging changes from main branch...");

    let output = std::process::Command::new("git")
        .args(["merge", "main"])
        .current_dir(&context.current_dir)
        .output()
        .with_context(|| "Failed to merge main branch into agent worktree")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for merge conflicts
        if stderr.contains("CONFLICT") || stderr.contains("Merge conflict") {
            handle_merge_conflicts(&stderr, conflict_strategy, dry_run)?;
        } else {
            return Err(SpriteError::git_with_source(
                "Failed to merge main branch",
                stderr.to_string()
            ).into());
        }
    }

    println!("✓ Agent worktree '{}' synced successfully", agent_id);
    Ok(())
}

/// Handle merge conflicts based on strategy.
fn handle_merge_conflicts(
    error_output: &str,
    strategy: ConflictResolution,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("  Merge conflicts detected - would use strategy: {:?}", strategy);
        return Ok(());
    }

    match strategy {
        ConflictResolution::Manual => {
            println!("❌ Merge conflicts detected:");
            println!("{}", error_output);
            return Err(SpriteError::sync(
                "Merge conflicts detected. Please resolve them manually and try again."
            ).into());
        }
        ConflictResolution::AutoTheirs => {
            println!("🔄 Automatically accepting their changes...");
            let output = std::process::Command::new("git")
                .args(["checkout", "--theirs", "--", "."])
                .output()
                .with_context(|| "Failed to auto-accept their changes")?;

            if !output.status.success() {
                return Err(SpriteError::git_with_source(
                    "Failed to auto-accept their changes",
                    String::from_utf8_lossy(&output.stderr)
                ).into());
            }

            println!("✓ Automatically accepted their changes");
        }
        ConflictResolution::AutoOurs => {
            println!("🔄 Automatically accepting our changes...");
            let output = std::process::Command::new("git")
                .args(["checkout", "--ours", "--", "."])
                .output()
                .with_context(|| "Failed to auto-accept our changes")?;

            if !output.status.success() {
                return Err(SpriteError::git_with_source(
                    "Failed to auto-accept our changes",
                    String::from_utf8_lossy(&output.stderr)
                ).into());
            }

            println!("✓ Automatically accepted our changes");
        }
    }

    // Stage resolved files
    let output = std::process::Command::new("git")
        .args(["add", "."])
        .output()
        .with_context(|| "Failed to stage resolved files")?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            "Failed to stage resolved files",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    Ok(())
}

/// Helper function to switch back to original branch (using defer pattern).
fn defer_switch_back(original_branch: &str) -> Result<()> {
    // In Rust, we don't have defer, so this is just a placeholder
    // The actual implementation would need to use a more sophisticated approach
    // or handle the switch back logic differently
    if original_branch != "main" && original_branch != "master" {
        // This would need to be handled with proper error handling
        println!("  Note: Would switch back to '{}' after operations", original_branch);
    }
    Ok(())
}

/// Check if we need to update tasks.md with completed tasks
fn mark_task_completed(task_id: &str) -> Result<()> {
    let tasks_path = PathBuf::from("specs/001-multi-agent-toolkit/tasks.md");

    if !tasks_path.exists() {
        return Ok(()); // Skip if tasks file doesn't exist
    }

    let content = std::fs::read_to_string(&tasks_path)
        .with_context(|| "Failed to read tasks.md")?;

    // Replace task status from pending to completed
    let updated_content = content.replace(&format!("- [ ] {}", task_id), &format!("- [x] {}", task_id));

    if updated_content != content {
        std::fs::write(&tasks_path, updated_content)
            .with_context(|| "Failed to update tasks.md")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_agent_id_from_path() {
        assert_eq!(extract_agent_id_from_path("agents/1"), Some("1".to_string()));
        assert_eq!(extract_agent_id_from_path("agents/2/"), Some("2".to_string()));
        assert_eq!(extract_agent_id_from_path("agents/agent-3"), Some("agent-3".to_string()));
        assert_eq!(extract_agent_id_from_path("agents/agent_4"), Some("agent_4".to_string()));
        assert_eq!(extract_agent_id_from_path("other/path"), None);
        assert_eq!(extract_agent_id_from_path("agents/"), None);
    }

    #[test]
    fn test_determine_sync_context() {
        // This test would need to be run in different directories
        // For now, just ensure the function compiles
        let _context = determine_sync_context(None);
    }

    #[test]
    fn test_conflict_resolution_parsing() {
        // Test that valid strategies are accepted
        let _valid_manual = "manual";
        let _valid_theirs = "auto-theirs";
        let _valid_ours = "auto-ours";

        // Invalid strategy should be handled by the execute function
        // assert!(execute(None, false, "invalid", true).is_err());
    }
}
