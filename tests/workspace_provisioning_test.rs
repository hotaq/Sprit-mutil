// Integration tests for workspace provisioning functionality

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use anyhow::Result;

// Since this is a binary project, we'll test the CLI commands directly
use assert_cmd::Command as AssertCommand;

#[test]
fn test_cli_basic_functionality() -> Result<()> {
    // Test that the CLI responds to help
    AssertCommand::cargo_bin("sprit-mutil")?
        .args(&["--help"])
        .assert().success()
        .stdout(predicates::str::contains("sprit-mutil"))
        .stdout(predicates::str::contains("init"))
        .stdout(predicates::str::contains("agents"));

    // Test version
    AssertCommand::cargo_bin("sprit-mutil")?
        .args(&["--version"])
        .assert().success()
        .stdout(predicates::str::contains("0.1.0"));

    Ok(())
}

#[test]
fn test_init_requires_git_repo() -> Result<()> {
    // Create a temporary directory that is NOT a git repository
    let temp_dir = TempDir::new()?;

    // Try to initialize sprite configuration - should fail without git
    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(temp_dir.path())
        .args(&["init"])
        .assert().failure()
        .stderr(predicates::str::contains("git"));

    Ok(())
}

#[test]
fn test_init_creates_required_files() -> Result<()> {
    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Initialize sprite configuration
    let result = AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)  // Explicitly set the working directory
        .args(&["init", "--force"])
        .assert();

    // Check output contains expected messages
    let stdout = std::str::from_utf8(&result.get_output().stdout)?;
    assert!(stdout.contains("initialized successfully"));
    assert!(stdout.contains("agents/agents.yaml"));

    // Verify basic directory structure was created
    assert!(Path::new(&repo_path.join("agents")).exists());
    assert!(Path::new(&repo_path.join("agents/scripts")).exists());
    assert!(Path::new(&repo_path.join("agents/profiles")).exists());

    // Cleanup
    fs::remove_dir_all(repo_path.join("agents"))?;

    Ok(())
}

#[test]
fn test_init_error_handling() -> Result<()> {
    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Initialize sprite configuration
    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["init", "--force"])
        .assert().success();

    // Try to initialize again without force - should fail
    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["init"])
        .assert().failure()
        .stderr(predicates::str::contains("already exists"));

    // Initialize again with force - should succeed
    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["init", "--force"])
        .assert().success();

    // Cleanup
    fs::remove_dir_all(repo_path.join("agents"))?;

    Ok(())
}

#[test]
fn test_agents_commands_require_config() -> Result<()> {
    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Try agents commands without initialization - should fail
    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["agents", "list"])
        .assert().failure()
        .stderr(predicates::str::contains("Configuration file not found"));

    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["agents", "validate"])
        .assert().failure()
        .stderr(predicates::str::contains("Configuration file not found"));

    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["agents", "provision"])
        .assert().failure()
        .stderr(predicates::str::contains("Configuration file not found"));

    Ok(())
}

#[test]
fn test_config_commands_require_config() -> Result<()> {
    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Try config commands without initialization - should fail
    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["config", "show"])
        .assert().failure()
        .stderr(predicates::str::contains("Configuration file not found"));

    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["config", "validate"])
        .assert().failure()
        .stderr(predicates::str::contains("Configuration file not found"));

    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["config", "status"])
        .assert().failure()
        .stderr(predicates::str::contains("Configuration file not found"));

    Ok(())
}

#[test]
fn test_workspace_provisioning_integration() -> Result<()> {
    // This test verifies that the provisioning command exists and is accessible
    // even if we can't test the full functionality due to config conflicts

    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Test that the provision command is recognized
    AssertCommand::cargo_bin("sprit-mutil")?
        .current_dir(&repo_path)
        .args(&["agents", "provision", "--help"])
        .assert().success()
        .stdout(predicates::str::contains("provision"))
        .stdout(predicates::str::contains("workspaces"));

    Ok(())
}

/// Test helper to create a temporary git repository
fn create_test_git_repo() -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repository
    Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()?;

    // Configure git user
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;

    // Create initial commit
    let readme_path = repo_path.join("README.md");
    fs::write(&readme_path, "# Test Repository\n")?;

    Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    Ok((temp_dir, repo_path))
}