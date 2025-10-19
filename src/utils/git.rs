//! Git utility functions for the Sprite multi-agent workflow toolkit.
//!
//! This module provides functions for managing git repositories, worktrees,
//! branches, and other git operations needed for agent workspace management.

use crate::error::SpriteError;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the current git repository root directory.
#[allow(dead_code)]
pub fn get_git_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .with_context(|| "Failed to execute git command. Is git installed?")?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            "Failed to get git repository root",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let root_str = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(root_str.trim()))
}

/// Get the current git branch name.
#[allow(dead_code)]
pub fn get_current_branch() -> Result<String> {
    get_current_branch_at(&std::env::current_dir()?)
}

/// Get the current git branch name at a specific path.
pub fn get_current_branch_at(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .with_context(|| format!("Failed to get current git branch at {}", path.display()))?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!("Failed to get current branch at {}", path.display()),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

/// Check if a git branch exists.
pub fn branch_exists(branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
        .output()
        .with_context(|| "Failed to check if branch exists")?;

    Ok(output.status.success())
}

/// Create a new git branch.
pub fn create_branch(branch: &str) -> Result<()> {
    create_branch_from(branch, "HEAD")
}

/// Create a new git branch from a specific starting point.
pub fn create_branch_from(branch: &str, from: &str) -> Result<()> {
    if branch_exists(branch)? {
        return Err(SpriteError::git(format!("Branch '{}' already exists", branch)).into());
    }

    let output = Command::new("git")
        .args(["checkout", "-b", branch, from])
        .output()
        .with_context(|| format!("Failed to create branch '{}' from '{}'", branch, from))?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!("Failed to create branch '{}' from '{}'", branch, from),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Switch to an existing git branch.
#[allow(dead_code)]
pub fn switch_branch(branch: &str) -> Result<()> {
    switch_branch_at(&std::env::current_dir()?, branch)
}

/// Switch to an existing git branch at a specific path.
pub fn switch_branch_at(path: &Path, branch: &str) -> Result<()> {
    if !branch_exists(branch)? {
        return Err(SpriteError::git(format!("Branch '{}' does not exist", branch)).into());
    }

    let output = Command::new("git")
        .current_dir(path)
        .args(["checkout", branch])
        .output()
        .with_context(|| {
            format!(
                "Failed to switch to branch '{}' at {}",
                branch,
                path.display()
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!(
                "Failed to switch to branch '{}' at {}",
                branch,
                path.display()
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Checkout a branch in a specific path.
pub fn checkout_branch(path: &Path, branch: &str) -> Result<()> {
    switch_branch_at(path, branch)
}

/// Create a git worktree for an agent.
pub fn create_worktree(path: &Path, branch: &str) -> Result<()> {
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create parent directory for worktree: {}",
                parent.display()
            )
        })?;
    }

    // Check if branch exists first
    if !branch_exists(branch)? {
        create_branch(branch)?;
    }

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            branch,
            path.to_string_lossy().as_ref(),
        ])
        .output()
        .with_context(|| format!("Failed to create worktree at '{}'", path.display()))?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!("Failed to create worktree at '{}'", path.display()),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Remove a git worktree.
pub fn remove_worktree(path: &Path) -> Result<()> {
    // Check if path exists and is a worktree
    if !path.exists() {
        return Err(
            SpriteError::git(format!("Worktree path does not exist: {}", path.display())).into(),
        );
    }

    let output = Command::new("git")
        .args(["worktree", "remove", path.to_string_lossy().as_ref()])
        .output()
        .with_context(|| format!("Failed to remove worktree at '{}'", path.display()))?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!("Failed to remove worktree at '{}'", path.display()),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    // Try to remove the directory if it still exists
    if path.exists() {
        std::fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove worktree directory: {}", path.display()))?;
    }

    Ok(())
}

/// List all git worktrees in the repository.
pub fn list_worktrees() -> Result<Vec<WorktreeInfo>> {
    let output = Command::new("git")
        .args(["worktree", "list"])
        .output()
        .with_context(|| "Failed to list worktrees")?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            "Failed to list worktrees",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let worktrees_str = String::from_utf8_lossy(&output.stdout);
    parse_worktree_list(&worktrees_str)
}

/// Information about a git worktree.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// Path to the worktree
    pub path: PathBuf,
    /// Branch name
    #[allow(dead_code)]
    pub branch: String,
    /// Whether the worktree is bare
    #[allow(dead_code)]
    pub bare: bool,
    /// Worktree ID (for detached HEAD worktrees)
    #[allow(dead_code)]
    pub id: Option<String>,
}

/// Parse the output of `git worktree list`.
fn parse_worktree_list(output: &str) -> Result<Vec<WorktreeInfo>> {
    let mut worktrees = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Parse worktree list format
        // Format: <path> [<branch> (<id>)]
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let path = PathBuf::from(parts[0]);
        let (branch, id, bare) = if parts.len() > 1 {
            // Check if this is a bare worktree
            if parts.len() == 2 && parts[1].starts_with('(') && parts[1].ends_with(')') {
                (String::new(), None, true)
            } else if parts.len() >= 3 {
                // Format: path branch id or path branch (something)
                if parts[2].starts_with('(') {
                    // This is a special marker like (bare), not an ID
                    (parts[1].to_string(), None, parts[2].contains("bare"))
                } else {
                    // Format: path branch id
                    (parts[1].to_string(), Some(parts[2].to_string()), false)
                }
            } else {
                // Format: path branch or path branch [id]
                let branch_part = parts[1];
                let (branch, id_part) = branch_part.split_once(' ').unwrap_or((branch_part, ""));

                let id = if !id_part.is_empty() {
                    Some(id_part.to_string())
                } else {
                    None
                };

                (branch.trim().to_string(), id, false)
            }
        } else {
            (String::new(), None, false)
        };

        worktrees.push(WorktreeInfo {
            path,
            branch,
            bare,
            id,
        });
    }

    Ok(worktrees)
}

/// Check if the current directory is a git repository.
pub fn is_git_repository() -> Result<bool> {
    is_git_repository_at(&std::env::current_dir()?)
}

/// Check if a specific path is a git repository.
pub fn is_git_repository_at(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "--git-dir"])
        .output()
        .with_context(|| format!("Failed to execute git command at {}", path.display()))?;

    Ok(output.status.success())
}

/// Validate that we're in a git repository.
pub fn validate_git_repository() -> Result<()> {
    if !is_git_repository()? {
        return Err(SpriteError::config(
            "Not a git repository. Sprite must be run from within a git repository.",
        )
        .into());
    }
    Ok(())
}

/// Get the git repository status.
#[allow(dead_code)]
pub fn get_status() -> Result<GitStatus> {
    get_status_at(&std::env::current_dir()?)
}

/// Get the git repository status as a string.
#[allow(dead_code)]
pub fn get_status_string() -> Result<String> {
    get_status_string_at(&std::env::current_dir()?)
}

/// Get the git repository status at a specific path.
#[allow(dead_code)]
pub fn get_status_at(path: &Path) -> Result<GitStatus> {
    let output = Command::new("git")
        .current_dir(path)
        .args(["status", "--porcelain"])
        .output()
        .with_context(|| format!("Failed to get git status at {}", path.display()))?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!("Failed to get git status at {}", path.display()),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let status_str = String::from_utf8_lossy(&output.stdout);
    parse_git_status(&status_str)
}

/// Get the git repository status as a string at a specific path.
pub fn get_status_string_at(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(path)
        .args(["status", "--porcelain"])
        .output()
        .with_context(|| format!("Failed to get git status string at {}", path.display()))?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!("Failed to get git status string at {}", path.display()),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(status_str)
}

/// Git repository status information.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GitStatus {
    /// Number of modified files
    pub modified: usize,
    /// Number of added files
    pub added: usize,
    /// Number of deleted files
    pub deleted: usize,
    /// Number of untracked files
    pub untracked: usize,
    /// Whether there are any changes
    pub has_changes: bool,
}

/// Parse the output of `git status --porcelain`.
#[allow(dead_code)]
fn parse_git_status(output: &str) -> Result<GitStatus> {
    let mut status = GitStatus {
        modified: 0,
        added: 0,
        deleted: 0,
        untracked: 0,
        has_changes: false,
    };

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let first_char = line.chars().next().unwrap_or(' ');
        let second_char = line.chars().nth(1).unwrap_or(' ');

        match (first_char, second_char) {
            // Modified files (staged or unstaged)
            ('M', _) | ('A', 'M') | ('R', 'M') => status.modified += 1,
            (' ', 'M') => status.modified += 1, // Unstaged modifications
            // Added files
            ('A', _) | ('R', 'A') | ('C', 'A') => status.added += 1,
            // Deleted files
            ('D', _) | ('C', 'D') => status.deleted += 1,
            (' ', 'D') => status.deleted += 1, // Unstaged deletions
            // Untracked files
            ('?', _) => status.untracked += 1,
            _ => {}
        }
    }

    status.has_changes = status.modified + status.added + status.deleted + status.untracked > 0;

    Ok(status)
}

/// Pull changes from the remote repository.
#[allow(dead_code)]
pub fn pull() -> Result<()> {
    let output = Command::new("git")
        .args(["pull"])
        .output()
        .with_context(|| "Failed to pull changes")?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            "Failed to pull changes",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Push changes to the remote repository.
#[allow(dead_code)]
pub fn push(branch: Option<&str>) -> Result<()> {
    let mut args = vec!["push"];
    if let Some(b) = branch {
        args.extend(&["origin", b]);
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .with_context(|| "Failed to push changes")?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            "Failed to push changes",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Commit changes with a message.
#[allow(dead_code)]
pub fn commit(message: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .with_context(|| "Failed to commit changes")?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            "Failed to commit changes",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Add files to git staging area.
#[allow(dead_code)]
pub fn add(files: &[&Path]) -> Result<()> {
    let mut args: Vec<String> = vec!["add".to_string()];
    for file in files {
        args.push(file.to_string_lossy().into_owned());
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .with_context(|| "Failed to add files to git")?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            "Failed to add files to git",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Check if a file has uncommitted changes.
#[allow(dead_code)]
pub fn has_uncommitted_changes(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args([
            "diff",
            "--quiet",
            "--exit-code",
            path.to_string_lossy().as_ref(),
        ])
        .output()
        .with_context(|| "Failed to check for uncommitted changes")?;

    // Exit code 1 means there are changes, 0 means no changes
    Ok(output.status.code() == Some(1))
}

/// Get the current git commit hash.
#[allow(dead_code)]
pub fn get_current_commit() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .with_context(|| "Failed to get current commit hash")?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            "Failed to get current commit hash",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(commit)
}

/// Get the git remote URL.
#[allow(dead_code)]
pub fn get_remote_url(remote: Option<&str>) -> Result<String> {
    let remote_name = remote.unwrap_or("origin");

    let output = Command::new("git")
        .args(["remote", "get-url", remote_name])
        .output()
        .with_context(|| format!("Failed to get remote URL for '{}'", remote_name))?;

    if !output.status.success() {
        return Err(SpriteError::git_with_source(
            format!("Failed to get remote URL for '{}'", remote_name),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list() {
        let input = r#"
/path/to/worktree feature/branch
/path/to/another main abc123
/path/to/bare (bare)
"#;

        let worktrees = parse_worktree_list(input).unwrap();
        assert_eq!(worktrees.len(), 3);

        assert_eq!(worktrees[0].path, PathBuf::from("/path/to/worktree"));
        assert_eq!(worktrees[0].branch, "feature/branch");
        assert!(!worktrees[0].bare);
        assert_eq!(worktrees[0].id, None);

        assert_eq!(worktrees[1].path, PathBuf::from("/path/to/another"));
        assert_eq!(worktrees[1].branch, "main");
        assert!(!worktrees[1].bare);
        assert_eq!(worktrees[1].id, Some("abc123".to_string()));

        assert_eq!(worktrees[2].path, PathBuf::from("/path/to/bare"));
        assert!(worktrees[2].bare);
    }

    #[test]
    fn test_parse_git_status() {
        let input = r#"
 M src/main.rs
A src/new_file.rs
 D src/old_file.rs
?? untracked.txt
"#;

        let status = parse_git_status(input).unwrap();
        assert_eq!(status.modified, 1);
        assert_eq!(status.added, 1);
        assert_eq!(status.deleted, 1);
        assert_eq!(status.untracked, 1);
        assert!(status.has_changes);
    }

    #[test]
    fn test_validate_git_repository() {
        // This test depends on being run in a git repository
        // In actual usage, this would be tested in a proper git repository
        let _result = is_git_repository();
        // We don't assert the result since it depends on the test environment
    }
}
