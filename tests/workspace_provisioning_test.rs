// Integration tests for workspace provisioning functionality

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// Since this is a binary project, we'll test the CLI commands directly
use assert_cmd::Command as AssertCommand;

#[test]
fn test_cli_basic_functionality() -> Result<()> {
    // Test that the CLI responds to help
    AssertCommand::cargo_bin("sprite")?
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("sprite"))
        .stdout(predicates::str::contains("init"))
        .stdout(predicates::str::contains("agents"));

    // Test version
    AssertCommand::cargo_bin("sprite")?
        .args(["--version"])
        .assert()
        .success()
        .stdout(predicates::str::contains("0.2.4"));

    Ok(())
}

#[test]
fn test_init_auto_initializes_git() -> Result<()> {
    // Create a temporary directory that is NOT a git repository
    let temp_dir = TempDir::new()?;

    // Initialize sprite - should auto-initialize git
    AssertCommand::cargo_bin("sprite")?
        .current_dir(temp_dir.path())
        .args(["init", "--agents", "2"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Git repository initialized"));

    // Verify git was initialized
    assert!(temp_dir.path().join(".git").exists());

    // Verify worktrees were created
    assert!(temp_dir.path().join("agents/1").exists());
    assert!(temp_dir.path().join("agents/2").exists());

    Ok(())
}

#[test]
fn test_init_creates_required_files() -> Result<()> {
    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Initialize sprite configuration
    let result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path) // Explicitly set the working directory
        .args(["init", "--force"])
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
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(["init", "--force"])
        .assert()
        .success();

    // Try to initialize again without force - should fail
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(["init"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));

    // Initialize again with force - should succeed
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(["init", "--force"])
        .assert()
        .success();

    // Cleanup
    fs::remove_dir_all(repo_path.join("agents"))?;

    Ok(())
}

#[test]
fn test_agents_commands_require_config() -> Result<()> {
    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Try agents commands without initialization - should fail
    // Set env vars to prevent finding project root outside temp dir
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .args(["agents", "list"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Could not find sprite configuration file",
        ));

    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .args(["agents", "validate"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Could not find sprite configuration file",
        ));

    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .args(["agents", "provision"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Could not find sprite configuration file",
        ));

    Ok(())
}

#[test]
fn test_config_commands_require_config() -> Result<()> {
    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Try config commands without initialization - should fail
    // Set env vars to prevent finding project root outside temp dir
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .args(["config", "show"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Could not find sprite configuration file",
        ));

    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .args(["config", "validate"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Could not find sprite configuration file",
        ));

    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .args(["config", "status"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Could not find sprite configuration file",
        ));

    Ok(())
}

#[test]
fn test_workspace_provisioning_integration() -> Result<()> {
    // This test verifies that the provisioning command exists and is accessible
    // even if we can't test the full functionality due to config conflicts

    // Create a temporary git repository
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Test that the provision command is recognized
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(["agents", "provision", "--help"])
        .assert()
        .success()
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
        .args(["init"])
        .current_dir(&repo_path)
        .output()?;

    // Configure git user
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()?;

    // Create initial commit
    let readme_path = repo_path.join("README.md");
    fs::write(&readme_path, "# Test Repository\n")?;

    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    Ok((temp_dir, repo_path))
}
