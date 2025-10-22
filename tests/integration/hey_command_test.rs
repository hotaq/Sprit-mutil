//! Integration tests for the `/hey` command functionality

use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use assert_fs::prelude::*;
use crate::common::{TestFixture, wait_for_condition, cleanup_tmux_sessions};

/// Test the `/hey` command functionality with multiple agents
#[test]
fn test_hey_command_agent_communication() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    
    // Setup test environment
    fixture.setup_git_repo()?;
    
    // Initialize sprite environment with test agents
    let mut sprite_init = Command::new("cargo");
    sprite_init
        .args(["run", "init", "--force"])
        .current_dir(fixture.temp_dir.path())
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
  
  devops:
    name: "DevOps Agent"
    profile: "profile2.sh" 
    workspace: "infrastructure"
    active: true
    commands: ["docker", "kubectl", "terraform"]
"#;

    fixture.temp_dir
        .child("agents/agents.yaml")
        .write_str(agents_config)?;

    // Create workspaces
    fixture.temp_dir.child("frontend").create_dir_all()?;
    fixture.temp_dir.child("backend").create_dir_all()?;
    fixture.temp_dir.child("infrastructure").create_dir_all()?;

    // Start the sprite session
    let mut sprite_start = Command::new("cargo");
    sprite_start
        .args(["run", "start", "--agents", "3"])
        .current_dir(fixture.temp_dir.path())
        .spawn()?;

    // Wait for session to initialize
    thread::sleep(Duration::from_secs(5));

    // Test 1: Basic command to frontend agent
    let mut hey_frontend = Command::new("cargo");
    hey_frontend
        .args(["run", "hey", "frontend", "npm", "--version"])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    // Wait for command execution
    thread::sleep(Duration::from_secs(2));

    // Test 2: Command to backend agent
    let mut hey_backend = Command::new("cargo");
    hey_backend
        .args(["run", "hey", "backend", "cargo", "--version"])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    // Wait for command execution
    thread::sleep(Duration::from_secs(2));

    // Test 3: Command with working directory flag
    let mut hey_with_dir = Command::new("cargo");
    hey_with_dir
        .args([
            "run", "hey", 
            "frontend", 
            "ls", 
            "--work-dir", "frontend"
        ])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    // Wait for command execution
    thread::sleep(Duration::from_secs(2));

    // Test 4: Command with timeout
    let mut hey_with_timeout = Command::new("cargo");
    hey_with_timeout
        .args([
            "run", "hey",
            "backend",
            "echo",
            "\"timeout test\"",
            "--timeout", "10"
        ])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    // Test 5: Interactive mode test (non-blocking)
    let mut hey_interactive = Command::new("cargo");
    hey_interactive
        .args([
            "run", "hey",
            "devops",
            "echo",
            "\"interactive test\"",
            "--interactive"
        ])
        .current_dir(fixture.temp_dir.path())
        .timeout(Duration::from_secs(5))
        .assert()
        .success();

    // Cleanup
    cleanup_tmux_sessions("sprite-");
    
    Ok(())
}

/// Test error scenarios for the `/hey` command
#[test]
fn test_hey_command_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.setup_git_repo()?;

    // Test 1: Command without sprite session
    let mut hey_no_session = Command::new("cargo");
    hey_no_session
        .args(["run", "hey", "nonexistent", "echo", "test"])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .failure();

    // Initialize sprite for next tests
    fixture.init_sprite_environment()?;

    // Test 2: Command to non-existent agent
    let mut hey_no_agent = Command::new("cargo");
    hey_no_agent
        .args(["run", "hey", "nonexistent-agent", "echo", "test"])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .failure();

    // Test 3: Empty command
    let mut hey_empty = Command::new("cargo");
    hey_empty
        .args(["run", "hey", ""])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .failure();

    Ok(())
}

/// Test `/hey` command performance requirements
#[test]
fn test_hey_command_performance() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.setup_git_repo()?;
    fixture.init_sprite_environment()?;

    // Performance test: Simple command should complete in < 3 seconds
    let start_time = std::time::Instant::now();

    let mut hey_perf = Command::new("cargo");
    hey_perf
        .args(["run", "hey", "frontend", "echo", "\"performance test\""])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    let execution_time = start_time.elapsed();
    
    // Verify performance requirement (< 3 seconds for message delivery)
    assert!(
        execution_time < Duration::from_secs(3),
        "Hey command took {:?}, expected < 3 seconds",
        execution_time
    );

    Ok(())
}

/// Test concurrent `/hey` commands to multiple agents
#[test]
fn test_hey_command_concurrent() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.setup_git_repo()?;
    fixture.setup_multi_agent_config()?;

    // Start sprite session
    let mut sprite_start = Command::new("cargo");
    sprite_start
        .args(["run", "start", "--agents", "3"])
        .current_dir(fixture.temp_dir.path())
        .spawn()?;

    thread::sleep(Duration::from_secs(5));

    // Send concurrent commands to different agents
    let start_time = std::time::Instant::now();

    let mut handles = Vec::new();

    for agent in ["frontend", "backend", "devops"] {
        let mut cmd = Command::new("cargo");
        cmd.args([
            "run", "hey", agent, "echo", 
            &format!("\"Hello from {}\"", agent)
        ])
        .current_dir(fixture.temp_dir.path());
        
        let handle = std::thread::spawn(move || {
            cmd.assert().success();
        });
        handles.push(handle);
    }

    // Wait for all commands to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let total_execution_time = start_time.elapsed();
    
    // Concurrent commands should complete efficiently
    assert!(
        total_execution_time < Duration::from_secs(10),
        "Concurrent hey commands took {:?}, expected < 10 seconds",
        total_execution_time
    );

    cleanup_tmux_sessions("sprite-");
    Ok(())
}

/// Test `/hey` command with complex agent workflows
#[test]
fn test_hey_command_complex_workflows() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.setup_git_repo()?;
    fixture.setup_complex_project_structure()?;

    // Initialize sprite with complex project
    let mut sprite_init = Command::new("cargo");
    sprite_init
        .args(["run", "init", "--force"])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    // Start sprite session
    let mut sprite_start = Command::new("cargo");
    sprite_start
        .args(["run", "start", "--agents", "2"])
        .current_dir(fixture.temp_dir.path())
        .spawn()?;

    thread::sleep(Duration::from_secs(5));

    // Test 1: Build workflow - send build command to frontend
    let mut build_frontend = Command::new("cargo");
    build_frontend
        .args([
            "run", "hey", "frontend", 
            "npm", "run", "build",
            "--work-dir", "frontend"
        ])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    thread::sleep(Duration::from_secs(3));

    // Test 2: Test workflow - send test command to backend  
    let mut test_backend = Command::new("cargo");
    test_backend
        .args([
            "run", "hey", "backend",
            "cargo", "test", 
            "--work-dir", "backend"
        ])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    thread::sleep(Duration::from_secs(3));

    // Test 3: Integration workflow - chain commands
    let mut integration_test = Command::new("cargo");
    integration_test
        .args([
            "run", "hey", "frontend",
            "echo", "\"Starting integration test...\" && npm run test:e2e",
            "--work-dir", "frontend"
        ])
        .current_dir(fixture.temp_dir.path())
        .assert()
        .success();

    cleanup_tmux_sessions("sprite-");
    Ok(())
}

impl TestFixture {
    fn init_sprite_environment(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sprite_init = Command::new("cargo");
        sprite_init
            .args(["run", "init", "--force"])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();
        Ok(())
    }

    fn setup_multi_agent_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.init_sprite_environment()?;

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
  
  devops:
    name: "DevOps Agent"
    profile: "profile2.sh"
    workspace: "infrastructure"
    active: true
    commands: ["docker", "kubectl", "terraform"]
"#;

        self.temp_dir
            .child("agents/agents.yaml")
            .write_str(agents_config)?;

        self.temp_dir.child("frontend").create_dir_all()?;
        self.temp_dir.child("backend").create_dir_all()?;
        self.temp_dir.child("infrastructure").create_dir_all()?;

        Ok(())
    }

    fn setup_complex_project_structure(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create frontend project structure
        let frontend_dir = self.temp_dir.child("frontend");
        frontend_dir.create_dir_all()?;
        frontend_dir.child("package.json").write_str(r#"
{
  "name": "test-frontend",
  "version": "1.0.0",
  "scripts": {
    "build": "echo 'Building frontend...'",
    "test": "echo 'Running frontend tests...'",
    "test:e2e": "echo 'Running E2E tests...'"
  }
}
"#)?;

        // Create backend project structure  
        let backend_dir = self.temp_dir.child("backend");
        backend_dir.create_dir_all()?;
        backend_dir.child("Cargo.toml").write_str(r#"
[package]
name = "test-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
"#)?;

        backend_dir.child("src").create_dir_all()?;
        backend_dir.child("src/main.rs").write_str("fn main() { println!(\"Hello backend!\"); }")?;

        Ok(())
    }
}
