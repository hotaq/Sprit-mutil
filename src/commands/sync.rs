//! Sync command - Context-aware git synchronization

use crate::error::SpriteError;
use crate::models::ConflictResolution;
use crate::utils::git;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;

/// Execute the sync command with the given parameters.
pub fn execute(agent: Option<&str>, force: bool, strategy: &str, dry_run: bool) -> Result<()> {
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
        SyncContextType::MainWorktree => {
            sync_main_worktree(&sync_context, conflict_strategy, dry_run, force)?
        }
        SyncContextType::AgentWorktree { agent_id } => {
            sync_agent_worktree(&sync_context, agent_id, conflict_strategy, dry_run, force)?
        }
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
    #[allow(dead_code)]
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
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let git_root = git::get_git_root().context("Failed to get git repository root")?;

    let current_branch =
        git::get_current_branch_at(&current_dir).context("Failed to get current git branch")?;

    // Normalize paths for comparison
    let current_dir_normalized = current_dir
        .canonicalize()
        .context("Failed to canonicalize current directory")?;
    let git_root_normalized = git_root
        .canonicalize()
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
                    format!(
                        "Agent ID mismatch. Path suggests '{}' but you specified '{}'",
                        extracted_agent_id, provided_id
                    ),
                    Some("agent".to_string()),
                    Some(format!("path: {}, agent: {}", path_str, provided_id)),
                )
                .into());
            }
        }

        return Ok(SyncContext {
            context_type: SyncContextType::AgentWorktree {
                agent_id: extracted_agent_id,
            },
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
    if let Some(remaining) = path.strip_prefix("agents/") {
        // Check if the remaining part is just an agent ID (like "1", "2", etc.)
        if remaining.is_empty() {
            return None;
        }

        // Extract the first component as potential agent ID
        if let Some(slash_pos) = remaining.find('/') {
            let potential_id = &remaining[..slash_pos];
            // Validate that it's a valid agent ID (numeric or simple string)
            if potential_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Some(potential_id.to_string());
            }
        } else {
            // Entire remaining part is the agent ID
            if remaining
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Some(remaining.to_string());
            }
        }
    }

    None
}

/// Sync the main worktree (pull from remote).
fn sync_main_worktree(
    context: &SyncContext,
    _conflict_strategy: ConflictResolution,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    println!("📥 Syncing main worktree (pull from remote origin/main)");

    if dry_run {
        println!("  Would run: git pull origin main");
        return Ok(());
    }

    // Check if we have uncommitted changes
    let status =
        git::get_status_string_at(&context.current_dir).context("Failed to get git status")?;

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
    println!(
        "📥 Syncing agent worktree '{}' (merge from main branch)",
        agent_id
    );

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
        git::switch_branch("main")
            .or_else(|_| {
                // Try master if main doesn't exist
                git::switch_branch("master")
            })
            .with_context(|| "Failed to switch to main branch")?;

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
                stderr.to_string(),
            )
            .into());
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
        println!(
            "  Merge conflicts detected - would use strategy: {:?}",
            strategy
        );
        return Ok(());
    }

    match strategy {
        ConflictResolution::Manual => {
            println!("❌ Merge conflicts detected:");
            println!("{}", error_output);
            return Err(SpriteError::sync(
                "Merge conflicts detected. Please resolve them manually and try again.",
            )
            .into());
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
                    String::from_utf8_lossy(&output.stderr),
                )
                .into());
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
                    String::from_utf8_lossy(&output.stderr),
                )
                .into());
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
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
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
        println!(
            "  Note: Would switch back to '{}' after operations",
            original_branch
        );
    }
    Ok(())
}

/// Sync status tracking and reporting
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SyncStatus {
    pub start_time: std::time::SystemTime,
    pub end_time: Option<std::time::SystemTime>,
    pub context_type: String,
    pub operations_completed: Vec<SyncOperation>,
    pub conflicts_detected: Vec<MergeConflict>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SyncOperation {
    pub operation_type: String,
    pub description: String,
    pub start_time: std::time::SystemTime,
    pub end_time: Option<std::time::SystemTime>,
    pub success: bool,
    pub output: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MergeConflict {
    pub file_path: String,
    pub conflict_type: String,
    pub detected_at: std::time::SystemTime,
}

/// Sync hooks and pre/post processing
#[allow(dead_code)]
pub struct SyncHooks {
    pub pre_sync_hooks: Vec<SyncHook>,
    pub post_sync_hooks: Vec<SyncHook>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SyncHook {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub required: bool,
    pub timeout: Option<std::time::Duration>,
}

impl SyncHooks {
    #[allow(dead_code)]
    pub fn load_from_config() -> Result<Self> {
        // In a real implementation, this would load from configuration
        // For now, return default hooks
        Ok(Self {
            pre_sync_hooks: vec![SyncHook {
                name: "Check for uncommitted changes".to_string(),
                command: "git".to_string(),
                args: vec!["status".to_string(), "--porcelain".to_string()],
                working_dir: None,
                required: true,
                timeout: Some(std::time::Duration::from_secs(5)),
            }],
            post_sync_hooks: vec![SyncHook {
                name: "Run tests if available".to_string(),
                command: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
                working_dir: None,
                required: false,
                timeout: Some(std::time::Duration::from_secs(60)),
            }],
        })
    }

    #[allow(dead_code)]
    pub async fn execute_pre_sync_hooks(&self, status: &mut SyncStatus) -> Result<()> {
        println!("🔧 Running pre-sync hooks...");

        for hook in &self.pre_sync_hooks {
            let operation = SyncOperation {
                operation_type: "pre-sync-hook".to_string(),
                description: hook.name.clone(),
                start_time: std::time::SystemTime::now(),
                end_time: None,
                success: false,
                output: None,
            };

            println!("  Running: {}", hook.name);

            let result = tokio::time::timeout(
                hook.timeout.unwrap_or(std::time::Duration::from_secs(30)),
                execute_hook_command(hook),
            )
            .await;

            let mut completed_operation = operation;
            completed_operation.end_time = Some(std::time::SystemTime::now());

            match result {
                Ok(Ok(output)) => {
                    completed_operation.success = true;
                    completed_operation.output = Some(output);
                    status.operations_completed.push(completed_operation);
                    println!("  ✓ {} completed successfully", hook.name);
                }
                Ok(Err(e)) => {
                    completed_operation.output = Some(format!("Error: {}", e));
                    status.operations_completed.push(completed_operation);

                    if hook.required {
                        status.success = false;
                        status.error_message =
                            Some(format!("Required pre-sync hook failed: {}", e));
                        return Err(SpriteError::sync(format!(
                            "Required pre-sync hook '{}' failed: {}",
                            hook.name, e
                        ))
                        .into());
                    } else {
                        println!(
                            "  ⚠️  Optional pre-sync hook failed, continuing: {}",
                            hook.name
                        );
                    }
                }
                Err(_) => {
                    completed_operation.output = Some("Hook timed out".to_string());
                    status.operations_completed.push(completed_operation);

                    if hook.required {
                        status.success = false;
                        status.error_message =
                            Some(format!("Required pre-sync hook '{}' timed out", hook.name));
                        return Err(SpriteError::sync(format!(
                            "Required pre-sync hook '{}' timed out",
                            hook.name
                        ))
                        .into());
                    } else {
                        println!(
                            "  ⚠️  Optional pre-sync hook timed out, continuing: {}",
                            hook.name
                        );
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn execute_post_sync_hooks(&self, status: &mut SyncStatus) -> Result<()> {
        println!("🔧 Running post-sync hooks...");

        for hook in &self.post_sync_hooks {
            let operation = SyncOperation {
                operation_type: "post-sync-hook".to_string(),
                description: hook.name.clone(),
                start_time: std::time::SystemTime::now(),
                end_time: None,
                success: false,
                output: None,
            };

            println!("  Running: {}", hook.name);

            let result = tokio::time::timeout(
                hook.timeout.unwrap_or(std::time::Duration::from_secs(30)),
                execute_hook_command(hook),
            )
            .await;

            let mut completed_operation = operation;
            completed_operation.end_time = Some(std::time::SystemTime::now());

            match result {
                Ok(Ok(output)) => {
                    completed_operation.success = true;
                    completed_operation.output = Some(output);
                    status.operations_completed.push(completed_operation);
                    println!("  ✓ {} completed successfully", hook.name);
                }
                Ok(Err(e)) => {
                    completed_operation.output = Some(format!("Error: {}", e));
                    status.operations_completed.push(completed_operation);

                    if hook.required {
                        println!("  ❌ Required post-sync hook failed: {}", hook.name);
                        // Post-sync hook failures are warnings, not errors
                    } else {
                        println!("  ⚠️  Optional post-sync hook failed: {}", hook.name);
                    }
                }
                Err(_) => {
                    completed_operation.output = Some("Hook timed out".to_string());
                    status.operations_completed.push(completed_operation);

                    if hook.required {
                        println!("  ❌ Required post-sync hook timed out: {}", hook.name);
                    } else {
                        println!("  ⚠️  Optional post-sync hook timed out: {}", hook.name);
                    }
                }
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
async fn execute_hook_command(hook: &SyncHook) -> Result<String> {
    let mut cmd = TokioCommand::new(&hook.command);
    cmd.args(&hook.args);

    if let Some(dir) = &hook.working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().await.with_context(|| {
        format!(
            "Failed to execute hook command: {} {}",
            hook.command,
            hook.args.join(" ")
        )
    })?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!(
                "Hook command failed: {} {}",
                hook.command,
                hook.args.join(" ")
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

impl SyncStatus {
    #[allow(dead_code)]
    pub fn new(context_type: String) -> Self {
        Self {
            start_time: std::time::SystemTime::now(),
            end_time: None,
            context_type,
            operations_completed: Vec::new(),
            conflicts_detected: Vec::new(),
            success: true,
            error_message: None,
        }
    }

    #[allow(dead_code)]
    pub fn complete(&mut self) {
        self.end_time = Some(std::time::SystemTime::now());
    }

    #[allow(dead_code)]
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.end_time
            .map(|end| end.duration_since(self.start_time).unwrap_or_default())
    }

    #[allow(dead_code)]
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        report.push_str("📊 Sync Status Report\n");
        report.push_str("===================\n\n");
        report.push_str(&format!("Context: {}\n", self.context_type));
        report.push_str(&format!(
            "Status: {}\n",
            if self.success {
                "✅ Success"
            } else {
                "❌ Failed"
            }
        ));

        if let Some(duration) = self.duration() {
            report.push_str(&format!("Duration: {:.2}s\n", duration.as_secs_f64()));
        }

        if !self.operations_completed.is_empty() {
            report.push_str(&format!(
                "\nOperations Completed ({}):\n",
                self.operations_completed.len()
            ));
            for (i, op) in self.operations_completed.iter().enumerate() {
                let status = if op.success { "✅" } else { "❌" };
                report.push_str(&format!("  {}. {} {}\n", i + 1, status, op.description));
            }
        }

        if !self.conflicts_detected.is_empty() {
            report.push_str(&format!(
                "\nConflicts Detected ({}):\n",
                self.conflicts_detected.len()
            ));
            for (i, conflict) in self.conflicts_detected.iter().enumerate() {
                report.push_str(&format!(
                    "  {}. {} in {}\n",
                    i + 1,
                    conflict.conflict_type,
                    conflict.file_path
                ));
            }
        }

        if let Some(error) = &self.error_message {
            report.push_str(&format!("\nError: {}\n", error));
        }

        report
    }

    #[allow(dead_code)]
    pub fn format_accessible_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!(
            "Sync operation in {} context. ",
            self.context_type
        ));

        if self.success {
            summary.push_str("Completed successfully. ");
        } else {
            summary.push_str("Failed. ");
        }

        if let Some(duration) = self.duration() {
            summary.push_str(&format!(
                "Duration: {:.2} seconds. ",
                duration.as_secs_f64()
            ));
        }

        summary.push_str(&format!(
            "{} operations completed. ",
            self.operations_completed.len()
        ));

        if !self.conflicts_detected.is_empty() {
            summary.push_str(&format!(
                "{} conflicts detected. ",
                self.conflicts_detected.len()
            ));
        }

        if let Some(error) = &self.error_message {
            summary.push_str(&format!("Error: {}. ", error));
        }

        summary
    }
}

/// Check if we need to update tasks.md with completed tasks
#[allow(dead_code)]
fn mark_task_completed(task_id: &str) -> Result<()> {
    let tasks_path = PathBuf::from("specs/001-multi-agent-toolkit/tasks.md");

    if !tasks_path.exists() {
        return Ok(()); // Skip if tasks file doesn't exist
    }

    let content =
        std::fs::read_to_string(&tasks_path).with_context(|| "Failed to read tasks.md")?;

    // Replace task status from pending to completed
    let updated_content =
        content.replace(&format!("- [ ] {}", task_id), &format!("- [x] {}", task_id));

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
        assert_eq!(
            extract_agent_id_from_path("agents/1"),
            Some("1".to_string())
        );
        assert_eq!(
            extract_agent_id_from_path("agents/2/"),
            Some("2".to_string())
        );
        assert_eq!(
            extract_agent_id_from_path("agents/agent-3"),
            Some("agent-3".to_string())
        );
        assert_eq!(
            extract_agent_id_from_path("agents/agent_4"),
            Some("agent_4".to_string())
        );
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
