//! Final integration test for T059: Communication commands validation
//! Tests hey, status, and agents commands working together

use assert_fs::prelude::*;
use std::process::Command;
use std::thread;
use std::time::Duration;

mod common;
use common::{cleanup_tmux_sessions, wait_for_condition, TestFixture};

/// Test complete communication workflow - agents, status, hey
#[test]
#[ignore] // Requires tmux environment
fn test_complete_communication_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Clean up any existing sessions first
    cleanup_tmux_sessions("sprite-");

    // Setup test environment
    fixture.setup_git_repo()?;
    setup_comprehensive_agent_config(&fixture)?;

    println!("🧪 Setting up communication workflow test...");

    // Step 1: Initialize sprite environment with 3 agents
    let mut sprite_init = assert_cmd::Command::cargo_bin("sprite")?;
    sprite_init
        .args(["init", "--force", "--agents", "3"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    println!("✅ Sprite init successful");

    // Step 2: Test agents command - list agents
    let mut agents_list = assert_cmd::Command::cargo_bin("sprite")?;
    let agents_result = agents_list
        .args(["agents", "list"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let agents_output = String::from_utf8_lossy(&agents_result.get_output().stdout);
    assert!(agents_output.contains("frontend"));
    assert!(agents_output.contains("backend"));
    assert!(agents_output.contains("devops"));
    println!("✅ Agents list command successful");

    // Step 3: Test agents command - show specific agent
    let mut agents_show = assert_cmd::Command::cargo_bin("sprite")?;
    let show_result = agents_show
        .args(["agents", "show", "frontend"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let show_output = String::from_utf8_lossy(&show_result.get_output().stdout);
    assert!(show_output.contains("frontend"));
    println!("✅ Agents show command successful");

    // Step 4 Test status command - before session
    let mut status_before = assert_cmd::Command::cargo_bin("sprite")?;
    status_before
        .args(["status"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    println!("✅ Status command (before session) successful");

    // Step 5: Start sprite session
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["start"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Wait for session initialization
    let session_ready = wait_for_condition(
        || check_tmux_session_exists("sprite-test-session").unwrap_or(false),
        Duration::from_secs(10),
        Duration::from_millis(500),
    );

    if !session_ready {
        cleanup_tmux_sessions("sprite-");
        return Err("Failed to initialize tmux session".into());
    }
    thread::sleep(Duration::from_secs(3));

    // Step 6: Test status command - after session
    let mut status_after = assert_cmd::Command::cargo_bin("sprite")?;
    let after_result = status_after
        .args(["status"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let after_output = String::from_utf8_lossy(&after_result.get_output().stdout);
    println!("Status output: {}", after_output);
    println!("✅ Status command (after session) successful");

    // Step 7: Test hey command - single agent
    let mut hey_single = assert_cmd::Command::cargo_bin("sprite")?;
    let start_time = std::time::Instant::now();
    hey_single
        .args(["hey", "1", "echo", "\"Hello from agent 1\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let single_time = start_time.elapsed();
    assert!(
        single_time < Duration::from_secs(3),
        "Single agent hey took {:?}",
        single_time
    );
    println!(
        "✅ Single agent hey command successful (took {:?})",
        single_time
    );

    thread::sleep(Duration::from_secs(2));

    // Step 8: Test hey command - multiple agents
    let mut hey_multiple = assert_cmd::Command::cargo_bin("sprite")?;
    let start_time = std::time::Instant::now();
    hey_multiple
        .args(["hey", "1,2,3", "echo", "\"Hello from multiple agents\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let multiple_time = start_time.elapsed();
    assert!(
        multiple_time < Duration::from_secs(5),
        "Multiple agent hey took {:?}",
        multiple_time
    );
    println!(
        "✅ Multiple agent hey command successful (took {:?})",
        multiple_time
    );

    thread::sleep(Duration::from_secs(2));

    // Step 9: Test hey command - broadcast to all
    let mut hey_all = assert_cmd::Command::cargo_bin("sprite")?;
    let start_time = std::time::Instant::now();
    hey_all
        .args(["hey", "all", "echo", "\"Broadcast to all agents\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let broadcast_time = start_time.elapsed();
    assert!(
        broadcast_time < Duration::from_secs(5),
        "Broadcast hey took {:?}",
        broadcast_time
    );
    println!(
        "✅ Broadcast hey command successful (took {:?})",
        broadcast_time
    );

    thread::sleep(Duration::from_secs(2));

    // Step 10: Test hey command - with flags
    let mut hey_flags = assert_cmd::Command::cargo_bin("sprite")?;
    hey_flags
        .args([
            "hey",
            "1",
            "echo",
            "\"Command with flags\"",
            "--timeout",
            "10",
            "--work-dir",
            ".",
        ])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    println!("✅ Hey command with flags successful");

    thread::sleep(Duration::from_secs(2));

    // Step 11: Test status command with --detailed flag
    let mut status_detailed = assert_cmd::Command::cargo_bin("sprite")?;
    status_detailed
        .args(["status", "--detailed"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    println!("✅ Detailed status command successful");

    // Step 12: Test agents command with validate
    let mut agents_validate = assert_cmd::Command::cargo_bin("sprite")?;
    agents_validate
        .args(["agents", "validate"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    println!("✅ Agents validate command successful");

    // Step 13: Test agents command with provision
    let mut agents_provision = assert_cmd::Command::cargo_bin("sprite")?;
    agents_provision
        .args(["agents", "provision"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    println!("✅ Agents provision command successful");

    // Cleanup
    cleanup_tmux_sessions("sprite-");
    cleanup_tmux_sessions("sprite-test-session");

    println!("🎉 Complete communication workflow test PASSED!");
    Ok(())
}

/// Test error handling and edge cases
#[test]
fn test_communication_commands_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    println!("🧪 Setting up error handling test...");

    // Test 1: hey command without session
    let mut hey_no_session = assert_cmd::Command::cargo_bin("sprite")?;
    hey_no_session
        .args(["hey", "1", "echo", "test"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .failure();
    println!("✅ Hey without session correctly fails");

    // Test 2: hey command with invalid agent
    let mut hey_invalid = assert_cmd::Command::cargo_bin("sprite")?;
    hey_invalid
        .args(["hey", "99", "echo", "test"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .failure();
    println!("✅ Hey with invalid agent correctly fails");

    // Test 3: agents command with invalid agent_id
    let mut agents_invalid = assert_cmd::Command::cargo_bin("sprite")?;
    fixture.init_sprite_env()?;

    agents_invalid
        .args(["agents", "show", "nonexistent"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .failure();
    println!("✅ Agents with invalid ID correctly fails");

    // Test 4: status command with invalid scope (graceful error handling)
    let mut status_invalid = assert_cmd::Command::cargo_bin("sprite")?;
    let status_result = status_invalid
        .args(["status", "invalid-scope"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let status_output = String::from_utf8_lossy(&status_result.get_output().stdout);
    assert!(
        status_output.contains("not found")
            || status_output.contains("error")
            || status_output.contains("invalid")
            || status_output.contains("No tmux sessions found")
    );
    println!("✅ Status with invalid scope handles gracefully");

    println!("🎉 Error handling test PASSED!");
    Ok(())
}

/// Test performance requirements for communication commands
#[test]
fn test_communication_commands_performance() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Clean up any existing sessions first
    cleanup_tmux_sessions("sprite-");

    println!("🧪 Setting up performance test...");

    // Setup
    fixture.setup_git_repo()?;
    setup_comprehensive_agent_config(&fixture)?;

    // Initialize sprite with 2 agents
    let mut sprite_init = assert_cmd::Command::cargo_bin("sprite")?;
    sprite_init
        .args(["init", "--force", "--agents", "2"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Start session (force to replace any existing session)
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["start", "--force"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    thread::sleep(Duration::from_secs(3));

    // Performance test 1: Simple hey command < 3 seconds
    let start_time = std::time::Instant::now();
    let mut hey_perf = assert_cmd::Command::cargo_bin("sprite")?;
    hey_perf
        .args(["hey", "1", "echo", "\"Performance test\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let hey_time = start_time.elapsed();
    assert!(
        hey_time < Duration::from_secs(3),
        "Hey command took {:?}, expected < 3s",
        hey_time
    );
    println!("✅ Hey command performance: {:?} (< 3s)", hey_time);

    // Performance test 2: Status command < 2 seconds
    let start_time = std::time::Instant::now();
    let mut status_perf = assert_cmd::Command::cargo_bin("sprite")?;
    status_perf
        .args(["status"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let status_time = start_time.elapsed();
    assert!(
        status_time < Duration::from_secs(2),
        "Status command took {:?}, expected < 2s",
        status_time
    );
    println!("✅ Status command performance: {:?} (< 2s)", status_time);

    // Performance test 3: Agents command < 2 seconds
    let start_time = std::time::Instant::now();
    let mut agents_perf = assert_cmd::Command::cargo_bin("sprite")?;
    agents_perf
        .args(["agents", "list"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let agents_time = start_time.elapsed();
    assert!(
        agents_time < Duration::from_secs(2),
        "Agents command took {:?}, expected < 2s",
        agents_time
    );
    println!("✅ Agents command performance: {:?} (< 2s)", agents_time);

    cleanup_tmux_sessions("sprite-");
    cleanup_tmux_sessions("sprite-test-session");

    println!("🎉 Performance test PASSED!");
    Ok(())
}

/// Test real-time scenarios and reliability
#[test]
#[ignore] // Requires tmux environment
fn test_communication_commands_real_time() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    println!("🧪 Setting up real-time test...");

    // Setup
    fixture.setup_git_repo()?;
    setup_comprehensive_agent_config(&fixture)?;

    // Initialize sprite with 2 agents
    let mut sprite_init = assert_cmd::Command::cargo_bin("sprite")?;
    sprite_init
        .args(["init", "--force", "--agents", "2"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Start session (force to replace any existing session)
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["start", "--force"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    thread::sleep(Duration::from_secs(3));

    // Real-time test: Rapid succession commands
    let mut times = Vec::new();
    for i in 1..=5 {
        let start_time = std::time::Instant::now();
        let mut hey_rapid = assert_cmd::Command::cargo_bin("sprite")?;
        hey_rapid
            .args(["hey", "1", "echo", &format!("\"Rapid test {}\"", i)])
            .current_dir(fixture.temp_dir.path())
            .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
            .env_remove("SPRITE_PROJECT_ROOT")
            .assert()
            .success();
        let command_time = start_time.elapsed();
        times.push(command_time);

        // Small delay between commands
        thread::sleep(Duration::from_millis(500));
    }

    // Verify consistent performance
    let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
    assert!(
        avg_time < Duration::from_secs(2),
        "Average rapid command time: {:?}, expected < 2s",
        avg_time
    );
    println!("✅ Rapid commands average time: {:?} (< 2s)", avg_time);

    // Real-time test: Check system remains responsive
    let start_time = std::time::Instant::now();
    let mut status_under_load = assert_cmd::Command::cargo_bin("sprite")?;
    status_under_load
        .args(["status", "--health"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let load_time = start_time.elapsed();
    assert!(
        load_time < Duration::from_secs(3),
        "Status under load took {:?}, expected < 3s",
        load_time
    );
    println!("✅ Status under load: {:?} (< 3s)", load_time);

    cleanup_tmux_sessions("sprite-");
    cleanup_tmux_sessions("sprite-test-session");

    println!("🎉 Real-time test PASSED!");
    Ok(())
}

// Helper functions

fn setup_comprehensive_agent_config(
    fixture: &TestFixture,
) -> Result<(), Box<dyn std::error::Error>> {
    let agents_config = r#"
agents:
  1:
    id: "1"
    name: "Frontend Agent"
    description: "Frontend development agent"
    profile: "profile0.sh"
    workspace: "frontend"
    worktree_path: "frontend"
    status: "active"
    commands: ["npm", "node", "yarn", "echo"]
    model: "claude-sonnet-4"
  
  2:
    id: "2" 
    name: "Backend Agent"
    description: "Backend development agent"
    profile: "profile1.sh"
    workspace: "backend"
    worktree_path: "backend"
    status: "active"
    commands: ["cargo", "rustc", "pytest", "echo"]
    model: "claude-sonnet-4"
  
  3:
    id: "3"
    name: "DevOps Agent" 
    description: "DevOps and infrastructure agent"
    profile: "profile2.sh"
    workspace: "infrastructure"
    worktree_path: "infrastructure"
    status: "active"
    commands: ["docker", "kubectl", "terraform", "echo"]
    model: "claude-sonnet-4"
"#;

    // Create agents directory and config
    fixture.temp_dir.child("agents").create_dir_all()?;
    fixture
        .temp_dir
        .child("agents/agents.yaml")
        .write_str(agents_config)?;

    // Create workspaces
    fixture.temp_dir.child("frontend").create_dir_all()?;
    fixture.temp_dir.child("backend").create_dir_all()?;
    fixture.temp_dir.child("infrastructure").create_dir_all()?;

    // Create basic package.json for frontend
    fixture.temp_dir.child("frontend/package.json").write_str(
        r#"
{
  "name": "test-frontend",
  "version": "1.0.0",
  "scripts": {
    "test": "echo 'Frontend tests passing...'"
  }
}
"#,
    )?;

    // Create basic Cargo.toml for backend
    fixture.temp_dir.child("backend/Cargo.toml").write_str(
        r#"
[package]
name = "test-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )?;

    Ok(())
}

fn check_tmux_session_exists(session_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    Ok(output)
}

impl TestFixture {
    fn init_sprite_env(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sprite_init = assert_cmd::Command::cargo_bin("sprite")?;
        sprite_init
            .args(["init", "--force"])
            .current_dir(self.temp_dir.path())
            .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
            .env_remove("SPRITE_PROJECT_ROOT")
            .assert()
            .success();
        Ok(())
    }
}
