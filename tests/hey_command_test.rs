//! Integration tests for the `/hey` command functionality

use assert_fs::prelude::*;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Common test utilities
mod common;
use common::{cleanup_tmux_sessions, TestFixture};

/// Test the `/hey` command functionality with multiple agents
#[test]
#[ignore = "Integration test requires tmux and complex setup - run manually"]
fn test_hey_command_agent_communication() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Setup test environment
    fixture.setup_git_repo()?;

    // Initialize sprite environment with test agents
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["init", "--force"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Create test agent configuration
    let agents_config = r#"
agents:
  frontend:
    name: "Frontend Agent"
    profile: "profile0.sh"
    workspace: "frontend"
    active: true
    commands: ["npm", "node", "yarn"]
  
  backend:
    name: "Backend Agent" 
    profile: "profile1.sh"
    workspace: "backend"
    active: true
    commands: ["cargo", "rustc", "pytest"]
"#;

    fixture
        .temp_dir
        .child("agents/agents.yaml")
        .write_str(agents_config)?;

    // Create workspaces
    fixture.temp_dir.child("frontend").create_dir_all()?;
    fixture.temp_dir.child("backend").create_dir_all()?;

    // Start the sprite session (using std::process::Command for spawn support)
    let sprite_bin = assert_cmd::cargo::cargo_bin("sprite");
    let mut _sprite_start = Command::new(sprite_bin)
        .args(["start", "--agents", "2"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .spawn()?;

    // Wait for session to initialize
    thread::sleep(Duration::from_secs(5));

    // Test 1: Basic command to frontend agent
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["hey", "frontend", "echo", "\"Hello frontend\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Wait for command execution
    thread::sleep(Duration::from_secs(2));

    // Test 2: Command to backend agent
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["hey", "backend", "echo", "\"Hello backend\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Wait for command execution
    thread::sleep(Duration::from_secs(2));

    // Cleanup
    cleanup_tmux_sessions("sprite-");

    println!("✅ Hey command agent communication test passed!");
    Ok(())
}

/// Test error scenarios for the `/hey` command
#[test]
fn test_hey_command_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.setup_git_repo()?;

    // Test 1: Command without sprite session
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["hey", "nonexistent", "echo", "test"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .failure(); // Should fail without sprite session

    println!("✅ Hey command error scenario test passed!");
    Ok(())
}

/// Test `/hey` command performance requirements
#[test]
#[ignore = "Integration test requires tmux and complex setup - run manually"]
fn test_hey_command_performance() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.setup_git_repo()?;

    // Initialize sprite for performance test
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["init", "--force"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Create simple agent config for performance test
    let agents_config = r#"
agents:
  frontend:
    name: "Frontend Agent"
    profile: "profile0.sh"
    workspace: "frontend"
    active: true
    commands: ["echo"]
"#;

    fixture
        .temp_dir
        .child("agents/agents.yaml")
        .write_str(agents_config)?;

    fixture.temp_dir.child("frontend").create_dir_all()?;

    // Performance test: Simple command should complete in < 3 seconds
    let start_time = std::time::Instant::now();

    assert_cmd::Command::cargo_bin("sprite")?
        .args(["hey", "frontend", "echo", "\"performance test\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let execution_time = start_time.elapsed();

    println!("Hey command execution time: {:?}", execution_time);

    // Verify performance requirement (< 3 seconds for message delivery)
    assert!(
        execution_time < Duration::from_secs(3),
        "Hey command took {:?}, expected < 3 seconds",
        execution_time
    );

    println!("✅ Hey command performance test passed!");
    Ok(())
}

/// Test `/hey` command with complex agent workflows
#[test]
#[ignore = "Integration test requires tmux and complex setup - run manually"]
fn test_hey_command_complex_workflows() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.setup_git_repo()?;

    // Initialize sprite with test project
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["init", "--force"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Create test project structure
    let agents_config = r#"
agents:
  frontend:
    name: "Frontend Agent"
    profile: "profile0.sh"
    workspace: "frontend"
    active: true
    commands: ["echo", "npm"]
  
  backend:
    name: "Backend Agent"
    profile: "profile1.sh"
    workspace: "backend"
    active: true
    commands: ["echo", "cargo"]
"#;

    fixture
        .temp_dir
        .child("agents/agents.yaml")
        .write_str(agents_config)?;

    fixture.temp_dir.child("frontend").create_dir_all()?;
    fixture.temp_dir.child("backend").create_dir_all()?;

    // Test complex workflow with different flags
    assert_cmd::Command::cargo_bin("sprite")?
        .args([
            "hey",
            "frontend",
            "echo",
            "\"Complex workflow test...\"",
            "--work-dir",
            "frontend",
            "--timeout",
            "10",
        ])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    thread::sleep(Duration::from_secs(2));

    println!("✅ Hey command complex workflow test passed!");
    Ok(())
}
