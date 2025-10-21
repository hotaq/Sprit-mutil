// Integration tests for session management functionality
//
// This test suite covers the complete session management workflow:
// - Session creation (start command)
// - Session listing and attachment (attach command)
// - Session health monitoring (status command)
// - Session termination (kill command)
// - Session recovery and cleanup

use anyhow::Result;
use std::process::Command;
use tempfile::TempDir;
use uuid::Uuid;

// Since this is a binary project, we'll test the CLI commands directly
use assert_cmd::Command as AssertCommand;

/// Generate a unique session name for test isolation
fn generate_unique_session_name(prefix: &str) -> String {
    format!(
        "{}-{}",
        prefix,
        Uuid::new_v4().to_string().split('-').next().unwrap()
    )
}

/// Helper to ensure session cleanup even on test failure
struct SessionGuard {
    session_name: String,
}

impl SessionGuard {
    fn new(session_name: String) -> Self {
        Self { session_name }
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        // Best effort cleanup - ignore errors
        let _ = Command::new("tmux")
            .args(&["kill-session", "-t", &self.session_name])
            .output();
    }
}

#[test]
fn test_session_start_list_attach_kill_workflow() -> Result<()> {
    // Create a temporary git repository for testing
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Generate unique session name for test isolation
    let session_name = generate_unique_session_name("test-session");
    let _guard = SessionGuard::new(session_name.clone());

    // Initialize sprite configuration
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["init", "--force"])
        .assert()
        .success();

    // Create worktrees for the agents
    create_test_worktrees(&repo_path, 3)?;

    // 1. Test start command
    let start_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["start", "--session-name", &session_name, "--detach"])
        .assert()
        .success();

    let start_stdout = std::str::from_utf8(&start_result.get_output().stdout)?;
    assert!(start_stdout.contains("tmux session") || start_stdout.contains("Created"));

    // 2. Test attach --list to see the session
    let list_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", "--list"])
        .assert()
        .success();

    let list_stdout = std::str::from_utf8(&list_result.get_output().stdout)?;
    assert!(list_stdout.contains(&session_name));

    // 3. Test status command to check session health
    let status_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["status"])
        .assert()
        .success();

    let status_stdout = std::str::from_utf8(&status_result.get_output().stdout)?;
    assert!(status_stdout.contains("Session Health Report"));

    // 4. Test kill command to clean up the session
    let kill_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["kill", "--force", &session_name])
        .assert()
        .success();

    let kill_stdout = std::str::from_utf8(&kill_result.get_output().stdout)?;
    assert!(kill_stdout.contains("killed successfully") || kill_stdout.contains("killed session"));

    // 5. Verify session is gone
    let final_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", "--list"])
        .assert()
        .success();

    let final_stdout = std::str::from_utf8(&final_result.get_output().stdout)?;
    assert!(!final_stdout.contains(&session_name));

    // Cleanup
    cleanup_sprite_config(&repo_path)?;

    Ok(())
}

#[test]
fn test_attach_command_list_functionality() -> Result<()> {
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Generate unique session name for test isolation
    let session_name = generate_unique_session_name("list-test");
    let _guard = SessionGuard::new(session_name.clone());

    // Initialize sprite
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["init", "--force"])
        .assert()
        .success();

    // Create worktrees for the agents
    create_test_worktrees(&repo_path, 3)?;

    // Test attach --list with no sessions
    let result1 = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", "--list"])
        .assert()
        .success();

    let stdout1 = std::str::from_utf8(&result1.get_output().stdout)?;
    assert!(stdout1.contains("No tmux sessions") || stdout1.contains("Available tmux sessions"));

    // Create a test session
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["start", "--session-name", &session_name, "--detach"])
        .assert()
        .success();

    // Test attach --list with one session
    let result2 = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", "--list"])
        .assert()
        .success();

    let stdout2 = std::str::from_utf8(&result2.get_output().stdout)?;
    assert!(stdout2.contains(&session_name));
    assert!(stdout2.contains("windows"));

    // Cleanup
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["kill", "--force", &session_name])
        .assert()
        .success();

    cleanup_sprite_config(&repo_path)?;

    Ok(())
}

#[test]
fn test_kill_command_session_selection() -> Result<()> {
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Generate unique session names for test isolation
    let session_alpha = generate_unique_session_name("session-alpha");
    let session_beta = generate_unique_session_name("session-beta");
    let _guard_alpha = SessionGuard::new(session_alpha.clone());
    let _guard_beta = SessionGuard::new(session_beta.clone());

    // Initialize sprite
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["init", "--force"])
        .assert()
        .success();

    // Create worktrees for the agents
    create_test_worktrees(&repo_path, 3)?;

    // Create two test sessions
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["start", "--session-name", &session_alpha, "--detach"])
        .assert()
        .success();

    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["start", "--session-name", &session_beta, "--detach"])
        .assert()
        .success();

    // Test killing specific session
    let kill_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["kill", "--force", &session_alpha])
        .assert()
        .success();

    let kill_stdout = std::str::from_utf8(&kill_result.get_output().stdout)?;
    assert!(kill_stdout.contains(&session_alpha));

    // Verify only alpha is killed
    let list_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", "--list"])
        .assert()
        .success();

    let list_stdout = std::str::from_utf8(&list_result.get_output().stdout)?;
    assert!(!list_stdout.contains(&session_alpha));
    assert!(list_stdout.contains(&session_beta));

    // Test kill --all
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["kill", "--all", "--force"])
        .assert()
        .success();

    // Verify all sessions are killed
    let final_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", "--list"])
        .assert()
        .success();

    let final_stdout = std::str::from_utf8(&final_result.get_output().stdout)?;
    assert!(!final_stdout.contains(&session_alpha));
    assert!(!final_stdout.contains(&session_beta));

    cleanup_sprite_config(&repo_path)?;

    Ok(())
}

#[test]
fn test_status_command_functionality() -> Result<()> {
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Generate unique session name for test isolation
    let session_name = generate_unique_session_name("status-test");
    let _guard = SessionGuard::new(session_name.clone());

    // Initialize sprite
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["init", "--force"])
        .assert()
        .success();

    // Create worktrees for the agents
    create_test_worktrees(&repo_path, 3)?;

    // Test status with no sessions
    let result1 = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["status"])
        .assert()
        .success();

    let stdout1 = std::str::from_utf8(&result1.get_output().stdout)?;
    assert!(stdout1.contains("No tmux sessions") || stdout1.contains("Session Health Report"));

    // Test status --detailed with no sessions
    let result2 = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["status", "--detailed"])
        .assert()
        .success();

    let stdout2 = std::str::from_utf8(&result2.get_output().stdout)?;
    assert!(stdout2.contains("No tmux sessions") || stdout2.contains("Session Health Report"));

    // Create a test session
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["start", "--session-name", &session_name, "--detach"])
        .assert()
        .success();

    // Test status with one session
    let result3 = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["status"])
        .assert()
        .success();

    let stdout3 = std::str::from_utf8(&result3.get_output().stdout)?;
    assert!(stdout3.contains("Session Health Report"));
    assert!(stdout3.contains(&session_name));

    // Test status --detailed
    let result4 = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["status", "--detailed"])
        .assert()
        .success();

    let stdout4 = std::str::from_utf8(&result4.get_output().stdout)?;
    assert!(stdout4.contains("Detailed Session Information"));
    assert!(stdout4.contains(&session_name));

    // Test status for specific session
    let result5 = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["status", &session_name])
        .assert()
        .success();

    let stdout5 = std::str::from_utf8(&result5.get_output().stdout)?;
    assert!(stdout5.contains(&session_name));

    // Cleanup
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["kill", "--force", &session_name])
        .assert()
        .success();

    cleanup_sprite_config(&repo_path)?;

    Ok(())
}

#[test]
fn test_session_error_handling() -> Result<()> {
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Generate unique session name for test isolation
    let non_existent_session = generate_unique_session_name("non-existent");

    // Initialize sprite
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["init", "--force"])
        .assert()
        .success();

    // Test attaching to non-existent session
    let attach_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", &non_existent_session])
        .assert()
        .failure();

    let attach_stderr = std::str::from_utf8(&attach_result.get_output().stderr)?;
    assert!(attach_stderr.contains("does not exist") || attach_stderr.contains("Session"));

    // Test killing non-existent session
    let kill_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["kill", "--force", &non_existent_session])
        .assert();

    // Command might succeed (exit 0) with "No sessions found" or fail with error message
    let kill_output = kill_result.get_output();
    let kill_stderr = std::str::from_utf8(&kill_output.stderr)?;
    let kill_stdout = std::str::from_utf8(&kill_output.stdout)?;

    // Accept either: error about non-existent session, or message saying no sessions found
    assert!(
        kill_stderr.contains("does not exist")
            || kill_stderr.contains("Session")
            || kill_stderr.contains("not found")
            || kill_stdout.contains("No tmux sessions")
            || kill_stdout.contains("No sessions"),
        "Expected error or info about non-existent session, stderr: {}, stdout: {}",
        kill_stderr,
        kill_stdout
    );

    // Test status for non-existent session
    let status_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["status", &non_existent_session])
        .assert()
        .success();

    let status_stdout = std::str::from_utf8(&status_result.get_output().stdout)?;
    assert!(status_stdout.contains("not found") || status_stdout.contains("No tmux sessions"));

    cleanup_sprite_config(&repo_path)?;

    Ok(())
}

#[test]
fn test_session_help_commands() -> Result<()> {
    // Test start help
    AssertCommand::cargo_bin("sprite")?
        .args(&["start", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Start supervision session"))
        .stdout(predicates::str::contains("session-name"));

    // Test attach help
    AssertCommand::cargo_bin("sprite")?
        .args(&["attach", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Attach to existing session"))
        .stdout(predicates::str::contains("list"));

    // Test kill help
    AssertCommand::cargo_bin("sprite")?
        .args(&["kill", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Terminate session"))
        .stdout(predicates::str::contains("force"))
        .stdout(predicates::str::contains("all"));

    // Test status help
    AssertCommand::cargo_bin("sprite")?
        .args(&["status", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Check session status and health"))
        .stdout(predicates::str::contains("cleanup"))
        .stdout(predicates::str::contains("detailed"));

    Ok(())
}

#[test]
fn test_concurrent_session_operations() -> Result<()> {
    let (_temp_dir, repo_path) = create_test_git_repo()?;

    // Initialize sprite
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["init", "--force"])
        .assert()
        .success();

    // Create worktrees for the agents
    create_test_worktrees(&repo_path, 3)?;

    // Create multiple sessions rapidly with unique names
    let session_names = vec![
        generate_unique_session_name("concurrent-1"),
        generate_unique_session_name("concurrent-2"),
        generate_unique_session_name("concurrent-3"),
    ];

    // Create guards for cleanup
    let _guards: Vec<SessionGuard> = session_names
        .iter()
        .map(|name| SessionGuard::new(name.clone()))
        .collect();

    for session_name in &session_names {
        AssertCommand::cargo_bin("sprite")?
            .current_dir(&repo_path)
            .args(&["start", "--session-name", session_name, "--detach"])
            .assert()
            .success();
    }

    // Verify all sessions exist
    let list_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", "--list"])
        .assert()
        .success();

    let list_stdout = std::str::from_utf8(&list_result.get_output().stdout)?;
    for session_name in &session_names {
        assert!(list_stdout.contains(session_name));
    }

    // Check status for all sessions
    let status_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["status"])
        .assert()
        .success();

    let status_stdout = std::str::from_utf8(&status_result.get_output().stdout)?;
    assert!(status_stdout.contains("Session Health Report"));

    // Clean up all sessions
    AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["kill", "--all", "--force"])
        .assert()
        .success();

    // Verify cleanup
    let final_result = AssertCommand::cargo_bin("sprite")?
        .current_dir(&repo_path)
        .args(&["attach", "--list"])
        .assert()
        .success();

    let final_stdout = std::str::from_utf8(&final_result.get_output().stdout)?;
    for session_name in &session_names {
        assert!(!final_stdout.contains(session_name));
    }

    cleanup_sprite_config(&repo_path)?;

    Ok(())
}

/// Test helper to create a temporary git repository
fn create_test_git_repo() -> Result<(TempDir, std::path::PathBuf)> {
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
    std::fs::write(repo_path.join("README.md"), "# Test Repository\n")?;

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

/// Helper to create worktrees for agents after init
fn create_test_worktrees(repo_path: &std::path::Path, agent_count: u32) -> Result<()> {
    for i in 1..=agent_count {
        let worktree_path = repo_path.join("agents").join(i.to_string());
        let branch_name = format!("agents/{}", i);

        // Create branch and worktree
        Command::new("git")
            .args(&[
                "worktree",
                "add",
                "-b",
                &branch_name,
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(repo_path)
            .output()?;
    }
    Ok(())
}

/// Test helper to clean up sprite configuration
fn cleanup_sprite_config(repo_path: &std::path::Path) -> Result<()> {
    let agents_dir = repo_path.join("agents");
    if agents_dir.exists() {
        std::fs::remove_dir_all(&agents_dir)?;
    }
    Ok(())
}
