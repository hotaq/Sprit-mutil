//! Project root detection and navigation utilities for Sprite.
//!
//! This module provides functions to automatically detect the Sprite project
//! root directory and handle directory navigation for commands.

use crate::error::SpriteError;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Walk up from a starting point looking for `agents/agents.yaml` and return the
/// directory that contains it.
fn search_candidate_for_root(start: &Path) -> Option<PathBuf> {
    // If we're given a file path (e.g. an executable), start from its parent.
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        let config_path = current.join("agents").join("agents.yaml");
        if config_path.is_file() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

/// Find the Sprite project root directory by looking for agents/agents.yaml
pub fn find_project_root() -> Result<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Highest priority: explicit override via environment variable
    if let Some(env_root) = std::env::var_os("SPRITE_PROJECT_ROOT") {
        if !env_root.is_empty() {
            candidates.push(PathBuf::from(env_root));
        }
    }

    // Current working directory (typical invocation path)
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir);
    }

    // Directory containing the running executable (supports invoking via absolute path)
    let disable_exe_search = std::env::var_os("SPRITE_DISABLE_EXE_DISCOVERY")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    if !disable_exe_search {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                candidates.push(parent.to_path_buf());
            }
        }
    }

    // Deduplicate candidates after canonicalisation while preserving order
    let mut unique_candidates: Vec<PathBuf> = Vec::new();
    for candidate in candidates {
        let canonical = std::fs::canonicalize(&candidate).unwrap_or_else(|_| candidate.clone());
        if unique_candidates
            .iter()
            .any(|existing| existing == &canonical)
        {
            continue;
        }
        unique_candidates.push(canonical);
    }

    for candidate in &unique_candidates {
        if let Some(root) = search_candidate_for_root(candidate.as_path()) {
            return Ok(root);
        }
    }

    Err(SpriteError::config(
        "Not in a Sprite project. Could not find agents/agents.yaml in current or parent directories.",
    )
    .into())
}

/// Change to the project root directory and return the original directory
pub fn change_to_project_root() -> Result<PathBuf> {
    let original_dir = std::env::current_dir().context("Failed to get current directory")?;
    let project_root = find_project_root()?;

    // Only change directories if we're not already in the project root
    if original_dir != project_root {
        std::env::set_current_dir(&project_root)
            .with_context(|| format!("Failed to change directory to {}", project_root.display()))?;

        println!("📁 Changed to project root: {}", project_root.display());
    }

    Ok(original_dir)
}

/// Restore the original directory after command execution
pub fn restore_original_directory(original_dir: &Path) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    if current_dir != original_dir {
        std::env::set_current_dir(original_dir).with_context(|| {
            format!("Failed to restore directory to {}", original_dir.display())
        })?;
    }

    Ok(())
}

/// Execute a function from the project root directory, then restore original directory
pub fn execute_from_project_root<F, R>(func: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    let original_dir = change_to_project_root()?;

    // Execute the function from project root
    let result = func();

    // Always try to restore the original directory, even if the function failed
    if let Err(e) = restore_original_directory(&original_dir) {
        eprintln!("⚠️  Warning: Failed to restore original directory: {}", e);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to serialize tests that modify current directory
    // This prevents parallel tests from interfering with each other
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_find_project_root() {
        let _lock = TEST_MUTEX.lock().unwrap();
        // Create a temporary directory structure
        let original_dir = std::env::current_dir().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let agents_dir = project_root.join("agents");
        let config_file = agents_dir.join("agents.yaml");

        // Create the directory structure
        fs::create_dir_all(&agents_dir).unwrap();
        fs::write(&config_file, "test: config").unwrap();

        // Save and clear environment variables to isolate test
        let previous_disable_exe = std::env::var_os("SPRITE_DISABLE_EXE_DISCOVERY");
        let previous_project_root = std::env::var_os("SPRITE_PROJECT_ROOT");
        std::env::set_var("SPRITE_DISABLE_EXE_DISCOVERY", "1");
        std::env::remove_var("SPRITE_PROJECT_ROOT");

        // Test from project root
        std::env::set_current_dir(project_root).unwrap();
        let found_root = find_project_root().unwrap();
        let expected_root = project_root.canonicalize().unwrap();
        assert_eq!(found_root, expected_root);

        // Test from subdirectory
        let sub_dir = project_root.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        std::env::set_current_dir(&sub_dir).unwrap();
        let found_root = find_project_root().unwrap();
        assert_eq!(found_root, expected_root);

        // Restore environment and directory
        match previous_disable_exe {
            Some(value) => std::env::set_var("SPRITE_DISABLE_EXE_DISCOVERY", value),
            None => std::env::remove_var("SPRITE_DISABLE_EXE_DISCOVERY"),
        }
        match previous_project_root {
            Some(value) => std::env::set_var("SPRITE_PROJECT_ROOT", value),
            None => std::env::remove_var("SPRITE_PROJECT_ROOT"),
        }
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_find_project_root_not_found() {
        let _lock = TEST_MUTEX.lock().unwrap();
        // Test in a directory without agents.yaml
        let original_dir = std::env::current_dir().unwrap();
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        std::env::remove_var("SPRITE_PROJECT_ROOT");

        let previous = std::env::var_os("SPRITE_DISABLE_EXE_DISCOVERY");
        std::env::set_var("SPRITE_DISABLE_EXE_DISCOVERY", "1");

        let result = find_project_root();

        match previous {
            Some(value) => std::env::set_var("SPRITE_DISABLE_EXE_DISCOVERY", value),
            None => std::env::remove_var("SPRITE_DISABLE_EXE_DISCOVERY"),
        }

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_change_to_project_root() {
        let _lock = TEST_MUTEX.lock().unwrap();
        // Create a temporary directory structure
        let global_original_dir = std::env::current_dir().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let agents_dir = project_root.join("agents");
        let config_file = agents_dir.join("agents.yaml");
        let sub_dir = project_root.join("subdir");

        // Create the directory structure
        fs::create_dir_all(&agents_dir).unwrap();
        fs::write(&config_file, "test: config").unwrap();
        fs::create_dir(&sub_dir).unwrap();

        // Save and clear environment variables to isolate test
        let previous_disable_exe = std::env::var_os("SPRITE_DISABLE_EXE_DISCOVERY");
        let previous_project_root = std::env::var_os("SPRITE_PROJECT_ROOT");
        std::env::set_var("SPRITE_DISABLE_EXE_DISCOVERY", "1");
        std::env::remove_var("SPRITE_PROJECT_ROOT");

        std::env::set_current_dir(&sub_dir).unwrap();
        let original_dir = std::env::current_dir().unwrap();
        assert_eq!(
            original_dir.canonicalize().unwrap(),
            sub_dir.canonicalize().unwrap()
        );

        // Change to project root
        let returned_original = change_to_project_root().unwrap();
        let expected_subdir = sub_dir.canonicalize().unwrap();
        assert_eq!(returned_original.canonicalize().unwrap(), expected_subdir);

        // Verify we're now in project root
        let current_dir = std::env::current_dir().unwrap();
        let expected_root = project_root.canonicalize().unwrap();
        assert_eq!(current_dir.canonicalize().unwrap(), expected_root);

        // Restore environment and directory
        match previous_disable_exe {
            Some(value) => std::env::set_var("SPRITE_DISABLE_EXE_DISCOVERY", value),
            None => std::env::remove_var("SPRITE_DISABLE_EXE_DISCOVERY"),
        }
        match previous_project_root {
            Some(value) => std::env::set_var("SPRITE_PROJECT_ROOT", value),
            None => std::env::remove_var("SPRITE_PROJECT_ROOT"),
        }
        std::env::set_current_dir(global_original_dir).unwrap();
    }
}
