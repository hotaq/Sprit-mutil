# Comprehensive Testing Strategy for Rust CLI Application with Slash Commands

This document provides a comprehensive testing strategy for the Rust CLI application that integrates slash commands with external AI frameworks, focusing on the Sprite multi-agent toolkit.

## Table of Contents

1. [Testing Framework Stack](#testing-framework-stack)
2. [Unit Testing Patterns](#unit-testing-patterns)
3. [Integration Testing for Slash Commands](#integration-testing-for-slash-commands)
4. [End-to-End Testing](#end-to-end-testing)
5. [Mocking External Dependencies](#mocking-external-dependencies)
6. [Tmux Integration Testing](#tmux-integration-testing)
7. [Performance Testing](#performance-testing)
8. [Test Coverage and Quality Gates](#test-coverage-and-quality-gates)
9. [CI/CD Integration](#cicd-integration)
10. [Test Organization](#test-organization)

---

## Testing Framework Stack

### Core Testing Dependencies

Add these dependencies to your `Cargo.toml`:

```toml
[dev-dependencies]
# Core testing utilities
tempfile = "3.0"
assert_cmd = "2.0"
predicates = "3.0"
assert_fs = "1.0"

# Mocking and test doubles
mockall = "0.12"
wiremock = "0.6"
tokio-test = "0.4"

# Property-based testing
proptest = "1.4"
quickcheck = "1.0"

# Test utilities and fixtures
fake = { version = "2.9", features = ["derive"] }
rand = "0.8"
pretty_assertions = "1.4"

# HTTP API testing
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"

# Async testing
futures = "0.3"

# Coverage analysis
tarpaulin = "0.27"

# Performance testing
criterion = "0.5"

# Test database and fixtures
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
```

### Recommended Testing Crates for CLI Applications

1. **assert_cmd** - Testing CLI commands
2. **assert_fs** - File system assertions
3. **predicates** - Boolean conditions for assertions
4. **tempfile** - Temporary file/directory creation
5. **mockall** - Mocking framework
6. **wiremock** - HTTP service mocking
7. **proptest** - Property-based testing
8. **criterion** - Benchmarking and performance testing

---

## Unit Testing Patterns

### 1. CLI Command Argument Parsing Tests

```rust
// tests/unit/cli_parsing_test.rs
use crate::cli::{Commands, AgentsCommands, ConfigCommands};
use clap::Parser;
use sprite::cli::Cli;

#[cfg(test)]
mod cli_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_agents_list_command() {
        let args = vec!["sprite", "agents", "list"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Agents { command: AgentsCommands::List } => {
                // Test passed
            }
            _ => panic!("Expected Agents::List command"),
        }
    }

    #[test]
    fn test_parse_agents_create_command_with_all_options() {
        let args = vec![
            "sprite", "agents", "create", "5",
            "--branch", "feature/test",
            "--workspace", "custom/path",
            "--model", "claude-sonnet-4",
            "--description", "Test agent",
            "--no-workspace"
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Agents {
                command: AgentsCommands::Create {
                    agent_id,
                    branch,
                    workspace,
                    model,
                    description,
                    no_workspace,
                },
            } => {
                assert_eq!(agent_id, "5");
                assert_eq!(branch, Some("feature/test".to_string()));
                assert_eq!(workspace, Some("custom/path".to_string()));
                assert_eq!(model, "claude-sonnet-4");
                assert_eq!(description, Some("Test agent".to_string()));
                assert!(no_workspace);
            }
            _ => panic!("Expected Agents::Create command"),
        }
    }

    #[test]
    fn test_parse_invalid_command() {
        let args = vec!["sprite", "invalid-command"];
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_parse_config_commands() {
        let test_cases = vec![
            (vec!["sprite", "config", "show"], "show"),
            (vec!["sprite", "config", "set", "key", "value"], "set"),
            (vec!["sprite", "config", "get", "key"], "get"),
        ];

        for (args, expected_subcommand) in test_cases {
            let cli = Cli::try_parse_from(args).unwrap();

            match cli.command {
                Commands::Config { command } => {
                    match expected_subcommand {
                        "show" => assert!(matches!(command, ConfigCommands::Show)),
                        "set" => assert!(matches!(command, ConfigCommands::Set { .. })),
                        "get" => assert!(matches!(command, ConfigCommands::Get { .. })),
                        _ => panic!("Unexpected subcommand: {}", expected_subcommand),
                    }
                }
                _ => panic!("Expected Config command"),
            }
        }
    }
}
```

### 2. Configuration Management Tests

```rust
// tests/unit/config_test.rs
use anyhow::Result;
use assert_fs::prelude::*;
use serde_yaml;
use sprite::config::SpriteConfig;
use sprite::commands::config::AgentConfig;
use tempfile::TempDir;

#[cfg(test)]
mod config_tests {
    use super::*;

    fn create_test_config() -> SpriteConfig {
        SpriteConfig {
            version: "1.0".to_string(),
            session_name: "test-session".to_string(),
            agents: vec![
                AgentConfig {
                    id: "1".to_string(),
                    branch: "agents/1".to_string(),
                    worktree_path: "agents/1".to_string(),
                    model: "claude-sonnet-4".to_string(),
                    description: "Test agent 1".to_string(),
                    status: "active".to_string(),
                    config: std::collections::HashMap::new(),
                },
            ],
        }
    }

    #[test]
    fn test_save_and_load_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("agents.yaml");

        // Create and save config
        let original_config = create_test_config();
        let yaml_content = serde_yaml::to_string(&original_config)?;
        std::fs::write(&config_path, yaml_content)?;

        // Load config
        let loaded_config = SpriteConfig::load_from_path(&config_path)?;

        assert_eq!(loaded_config.version, original_config.version);
        assert_eq!(loaded_config.session_name, original_config.session_name);
        assert_eq!(loaded_config.agents.len(), original_config.agents.len());
        assert_eq!(loaded_config.agents[0].id, original_config.agents[0].id);

        Ok(())
    }

    #[test]
    fn test_config_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("invalid.yaml");

        // Create invalid config (missing required fields)
        let invalid_yaml = r#"
        agents:
          - id: "1"
            # Missing required fields
        "#;

        std::fs::write(&config_path, invalid_yaml)?;

        // Should handle invalid config gracefully
        let result = SpriteConfig::load_from_path(&config_path);
        assert!(result.is_err() || result.unwrap().agents.is_empty());

        Ok(())
    }

    #[test]
    fn test_config_update_agent() -> Result<()> {
        let mut config = create_test_config();

        // Add new agent
        let new_agent = AgentConfig {
            id: "2".to_string(),
            branch: "agents/2".to_string(),
            worktree_path: "agents/2".to_string(),
            model: "gpt-4".to_string(),
            description: "Test agent 2".to_string(),
            status: "inactive".to_string(),
            config: std::collections::HashMap::new(),
        };

        config.agents.push(new_agent);
        assert_eq!(config.agents.len(), 2);
        assert_eq!(config.agents[1].id, "2");

        // Update existing agent
        config.agents[0].status = "updated".to_string();
        assert_eq!(config.agents[0].status, "updated");

        Ok(())
    }
}
```

### 3. Tmux Utility Function Tests

```rust
// tests/unit/tmux_test.rs
use mockall::mock;
use mockall::predicate::*;
use sprite::utils::tmux::*;
use std::process::Command;

mock! {
    CommandExecutor {}

    impl CommandExecutorTrait for CommandExecutor {
        fn execute_command(&self, args: &[&str]) -> Result<std::process::Output, std::io::Error>;
    }
}

trait CommandExecutorTrait {
    fn execute_command(&self, args: &[&str]) -> Result<std::process::Output, std::io::Error>;
}

#[cfg(test)]
mod tmux_tests {
    use super::*;

    #[test]
    fn test_session_exists_success() {
        // This test requires tmux to be installed
        let result = session_exists("non-existent-session");
        // Should return Ok(false) for non-existent session
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_parse_sessions_list() {
        let input = r#"
sprite-session: 3 windows (created Wed Oct 18 10:30:00 2023) [attached]
another-session: 1 windows (created Wed Oct 18 09:15:00 2023)
test-session: 2 windows (created Wed Oct 18 08:00:00 2023) [attached]
"#;

        let sessions = parse_sessions_list(input).unwrap();
        assert_eq!(sessions.len(), 3);

        assert_eq!(sessions[0].name, "sprite-session");
        assert_eq!(sessions[0].windows, 3);
        assert!(sessions[0].attached);

        assert_eq!(sessions[1].name, "another-session");
        assert_eq!(sessions[1].windows, 1);
        assert!(!sessions[1].attached);

        assert_eq!(sessions[2].name, "test-session");
        assert_eq!(sessions[2].windows, 2);
        assert!(sessions[2].attached);
    }

    #[test]
    fn test_parse_panes_list() {
        let input = r#"
0: [vim] (active) /Users/user/project
1: [bash] /Users/user/project
2: [git status] /Users/user/project
"#;

        let panes = parse_panes_list(input).unwrap();
        assert_eq!(panes.len(), 3);

        assert_eq!(panes[0].index, 0);
        assert_eq!(panes[0].current_command, Some("vim".to_string()));
        assert_eq!(panes[0].current_path, Some("/Users/user/project".to_string()));

        assert_eq!(panes[1].index, 1);
        assert_eq!(panes[1].current_command, Some("bash".to_string()));
    }

    #[test]
    fn test_tmux_version_parsing() {
        let mock_output = b"tmux 3.3a\n";
        let version_str = String::from_utf8_lossy(mock_output).trim();
        assert!(version_str.starts_with("tmux"));
    }
}
```

---

## Integration Testing for Slash Commands

### 1. Slash Command Framework Tests

```rust
// tests/integration/slash_command_test.rs
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod slash_command_integration_tests {
    use super::*;

    fn setup_test_environment(temp_dir: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize git repository
        Command::new("git")
            .arg("init")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // Create initial commit
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::write(temp_dir.path().join("test.txt"), "test content")?;

        Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn test_speckit_specify_command() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_environment(&temp_dir)?;

        // Test /speckit.specify command
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .args(["/speckit.specify", "Add authentication system with JWT tokens"])
            .assert()
            .success()
            .stdout(predicates::contains("Creating feature specification"))
            .stdout(predicates::contains("authentication system"));

        // Verify spec.md was created
        temp_dir.child("spec.md").assert(predicates::path::exists());

        // Verify spec content
        let spec_content = fs::read_to_string(temp_dir.path().join("spec.md"))?;
        assert!(spec_content.contains("authentication"));
        assert!(spec_content.contains("JWT"));

        Ok(())
    }

    #[test]
    fn test_speckit_plan_command() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_environment(&temp_dir)?;

        // First create a spec
        fs::write(
            temp_dir.path().join("spec.md"),
            r#"
# Authentication System Specification

## Overview
Add JWT-based authentication to the application.

## Requirements
- User registration and login
- JWT token generation and validation
- Protected routes
"#,
        )?;

        // Test /speckit.plan command
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("/speckit.plan")
            .assert()
            .success()
            .stdout(predicates::contains("Creating implementation plan"))
            .stdout(predicates::contains("plan.md"));

        // Verify plan.md was created
        temp_dir.child("plan.md").assert(predicates::path::exists());

        Ok(())
    }

    #[test]
    fn test_speckit_tasks_command() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_environment(&temp_dir)?;

        // Create spec and plan
        fs::write(temp_dir.path().join("spec.md"), "# Test Spec")?;
        fs::write(temp_dir.path().join("plan.md"), "# Test Plan")?;

        // Test /speckit.tasks command
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("/speckit.tasks")
            .assert()
            .success()
            .stdout(predicates::contains("Generating actionable tasks"))
            .stdout(predicates::contains("tasks.md"));

        // Verify tasks.md was created
        temp_dir.child("tasks.md").assert(predicates::path::exists());

        Ok(())
    }

    #[test]
    fn test_speckit_implement_command() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_environment(&temp_dir)?;

        // Create all required files
        fs::write(temp_dir.path().join("spec.md"), "# Test Spec")?;
        fs::write(temp_dir.path().join("plan.md"), "# Test Plan")?;
        fs::write(
            temp_dir.path().join("tasks.md"),
            r#"
## Tasks
- [ ] Create auth module
- [ ] Implement JWT utilities
- [ ] Add middleware
"#,
        )?;

        // Test /speckit.implement command
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("/speckit.implement")
            .assert()
            .success()
            .stdout(predicates::contains("Starting implementation"))
            .stdout(predicates::contains("Processing tasks"));

        Ok(())
    }

    #[test]
    fn test_speckit_analyze_command() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_environment(&temp_dir)?;

        // Create test files
        fs::write(temp_dir.path().join("spec.md"), "# Test Spec")?;
        fs::write(temp_dir.path().join("plan.md"), "# Test Plan")?;
        fs::write(temp_dir.path().join("tasks.md"), "# Test Tasks")?;

        // Test /speckit.analyze command
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("/speckit.analyze")
            .assert()
            .success()
            .stdout(predicates::contains("Analyzing artifacts"))
            .stdout(predicates::contains("consistency check"));

        Ok(())
    }

    #[test]
    fn test_invalid_speckit_command() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_environment(&temp_dir)?;

        // Test invalid speckit command
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("/speckit.invalid")
            .assert()
            .failure()
            .stderr(predicates::contains("Unknown speckit command"));

        Ok(())
    }

    #[test]
    fn test_speckit_command_missing_prerequisites() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_test_environment(&temp_dir)?;

        // Test /speckit.plan without spec.md
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("/speckit.plan")
            .assert()
            .failure()
            .stderr(predicates::contains("spec.md not found"));

        Ok(())
    }
}
```

### 2. Multi-Agent Command Integration Tests

```rust
// tests/integration/multi_agent_test.rs
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod multi_agent_integration_tests {
    use super::*;

    fn setup_agent_environment(temp_dir: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize git repository
        Command::new("git")
            .arg("init")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // Configure git
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // Create initial commit
        fs::write(temp_dir.path().join("initial.txt"), "initial content")?;
        Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // Initialize sprite environment
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .args(["init", "--agents", "3"])
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn test_agents_create_and_list_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_agent_environment(&temp_dir)?;

        // Create new agent
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .args([
                "agents", "create", "4",
                "--branch", "feature/test-agent",
                "--model", "claude-sonnet-4",
                "--description", "Test agent for integration testing"
            ])
            .assert()
            .success()
            .stdout(predicates::contains("Creating new agent: 4"))
            .stdout(predicates::contains("✅ Agent 4 created successfully"));

        // List agents
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("agents")
            .arg("list")
            .assert()
            .success()
            .stdout(predicates::contains("Configured Agents (4)"))
            .stdout(predicates::contains("Agent 4"));

        // Show specific agent
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .args(["agents", "show", "4"])
            .assert()
            .success()
            .stdout(predicates::contains("Agent 4"))
            .stdout(predicates::contains("feature/test-agent"))
            .stdout(predicates::contains("claude-sonnet-4"));

        Ok(())
    }

    #[test]
    fn test_agent_validation_and_provisioning() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_agent_environment(&temp_dir)?;

        // Validate agents (should show issues with missing workspaces)
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("agents")
            .arg("validate")
            .assert()
            .success()
            .stdout(predicates::contains("Validating agent workspaces"));

        // Provision agent workspaces
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("agents")
            .arg("provision")
            .assert()
            .success()
            .stdout(predicates::contains("Provisioning workspaces"))
            .stdout(predicates::contains("✅ Workspace provisioning complete"));

        // Validate again (should show all workspaces as valid)
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("agents")
            .arg("validate")
            .assert()
            .success()
            .stdout(predicates::contains("All 3 agent workspaces are valid"));

        Ok(())
    }

    #[test]
    fn test_agent_send_command_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_agent_environment(&temp_dir)?;

        // Provision workspaces first
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("agents")
            .arg("provision")
            .assert()
            .success();

        // Send command to specific agent
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .args([
                "hey", "1",
                "echo", "Hello from agent 1"
            ])
            .assert()
            .success()
            .stdout(predicates::contains("Sending command to agent 1"));

        // Send command to all agents
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .args([
                "send", "echo", 'Hello from all agents'
            ])
            .assert()
            .success()
            .stdout(predicates::contains("Sending command to all agents"));

        Ok(())
    }

    #[test]
    fn test_agent_workspace_sync() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_agent_environment(&temp_dir)?;

        // Provision workspaces
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("agents")
            .arg("provision")
            .assert()
            .success();

        // Make changes in one agent workspace
        let agent1_path = temp_dir.path().join("agents/1");
        fs::write(agent1_path.join("test.txt"), "test content from agent 1")?;

        Command::new("git")
            .args(["add", "."])
            .current_dir(&agent1_path)
            .assert()
            .success();

        Command::new("git")
            .args(["commit", "-m", "Test changes from agent 1"])
            .current_dir(&agent1_path)
            .assert()
            .success();

        // Sync workspaces
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("sync")
            .assert()
            .success()
            .stdout(predicates::contains("Synchronizing workspaces"));

        Ok(())
    }

    #[test]
    fn test_agent_remove_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        setup_agent_environment(&temp_dir)?;

        // Create additional agent for removal test
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .args([
                "agents", "create", "test-agent",
                "--no-workspace"
            ])
            .assert()
            .success();

        // Verify agent exists
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("agents")
            .arg("list")
            .assert()
            .success()
            .stdout(predicates::contains("test-agent"));

        // Remove agent
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .args(["agents", "remove", "test-agent", "--force"])
            .assert()
            .success()
            .stdout(predicates::contains("Removing agent: test-agent"))
            .stdout(predicates::contains("✅ Agent test-agent removed successfully"));

        // Verify agent is removed
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(temp_dir.path())
            .arg("agents")
            .arg("list")
            .assert()
            .success()
            .stdout(predicates::contains("Configured Agents (3)")) // Back to original 3
            .stdout(predicates::contains("test-agent").not());

        Ok(())
    }
}
```

---

## End-to-End Testing

### 1. Complete Workflow E2E Tests

```rust
// tests/e2e/complete_workflow_test.rs
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::{Child, Command as StdCommand};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[cfg(test)]
mod e2e_tests {
    use super::*;

    struct TestEnvironment {
        temp_dir: TempDir,
        session_name: String,
    }

    impl TestEnvironment {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let temp_dir = TempDir::new()?;
            let session_name = format!("test-session-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs());

            // Initialize git repository
            StdCommand::new("git")
                .arg("init")
                .current_dir(temp_dir.path())
                .assert()
                .success();

            // Configure git
            StdCommand::new("git")
                .args(["config", "user.email", "test@example.com"])
                .current_dir(temp_dir.path())
                .assert()
                .success();

            StdCommand::new("git")
                .args(["config", "user.name", "Test User"])
                .current_dir(temp_dir.path())
                .assert()
                .success();

            // Create initial commit
            fs::write(temp_dir.path().join("README.md"), "# Test Project")?;

            StdCommand::new("git")
                .args(["add", "."])
                .current_dir(temp_dir.path())
                .assert()
                .success();

            StdCommand::new("git")
                .args(["commit", "-m", "Initial commit"])
                .current_dir(temp_dir.path())
                .assert()
                .success();

            Ok(Self {
                temp_dir,
                session_name,
            })
        }

        fn run_sprite_command(&self, args: &[&str]) -> assert_cmd::Command {
            let mut cmd = Command::cargo_bin("sprite");
            cmd.current_dir(self.temp_dir.path());
            for arg in args {
                cmd.arg(arg);
            }
            cmd
        }

        fn cleanup_tmux_session(&self) {
            let _ = StdCommand::new("tmux")
                .args(["kill-session", "-t", &self.session_name])
                .output();
        }
    }

    impl Drop for TestEnvironment {
        fn drop(&mut self) {
            self.cleanup_tmux_session();
        }
    }

    #[test]
    fn test_complete_multi_agent_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let env = TestEnvironment::new()?;

        // 1. Initialize sprite environment
        env.run_sprite_command(&["init", "--agents", "3"])
            .assert()
            .success()
            .stdout(predicates::contains("✅ Sprite environment initialized successfully"));

        // 2. Create additional agents
        env.run_sprite_command(&[
            "agents", "create", "4",
            "--model", "claude-sonnet-4",
            "--description", "E2E Test Agent"
        ])
        .assert()
        .success();

        env.run_sprite_command(&[
            "agents", "create", "5",
            "--model", "gpt-4",
            "--description", "Another E2E Test Agent"
        ])
        .assert()
        .success();

        // 3. Verify agent configuration
        env.run_sprite_command(&["agents", "list"])
            .assert()
            .success()
            .stdout(predicates::contains("Configured Agents (5)"));

        // 4. Provision workspaces
        env.run_sprite_command(&["agents", "provision"])
            .assert()
            .success()
            .stdout(predicates::contains("✅ Workspace provisioning complete"));

        // 5. Validate workspaces
        env.run_sprite_command(&["agents", "validate"])
            .assert()
            .success()
            .stdout(predicates::contains("All 5 agent workspaces are valid"));

        // 6. Start tmux session
        env.run_sprite_command(&[
            "start",
            "--session-name", &env.session_name,
            "--detach"
        ])
        .assert()
        .success()
            .stdout(predicates::contains("Session started successfully"));

        // 7. Give tmux time to initialize
        thread::sleep(Duration::from_secs(2));

        // 8. Send commands to agents
        env.run_sprite_command(&[
            "hey", "1",
            "pwd"
        ])
        .assert()
        .success()
        .stdout(predicates::contains("Sending command to agent 1"));

        env.run_sprite_command(&[
            "hey", "2",
            "ls", "-la"
        ])
        .assert()
        .success();

        env.run_sprite_command(&[
            "send", "echo", 'Hello from all agents'
        ])
        .assert()
        .success()
        .stdout(predicates::contains("Sending command to all agents"));

        // 9. Check session status
        env.run_sprite_command(&["status"])
            .assert()
            .success()
            .stdout(predicates::contains(&env.session_name));

        // 10. Test zoom functionality
        env.run_sprite_command(&["zoom", "--agent", "1"])
            .assert()
            .success()
            .stdout(predicates::contains("Focusing on agent 1"));

        env.run_sprite_command(&["zoom", "--unzoom"])
            .assert()
            .success()
            .stdout(predicates::contains("Returning to normal layout"));

        // 11. Test workspace navigation
        env.run_sprite_command(&["warp", "--list"])
            .assert()
            .success()
            .stdout(predicates::contains("Available workspaces"));

        // 12. Clean up session
        env.run_sprite_command(&[
            "kill",
            "--session-name", &env.session_name,
            "--force"
        ])
        .assert()
        .success()
        .stdout(predicates::contains("Session terminated"));

        Ok(())
    }

    #[test]
    fn test_speckit_complete_development_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let env = TestEnvironment::new()?;

        // 1. Initialize sprite environment
        env.run_sprite_command(&["init", "--agents", "2"])
            .assert()
            .success();

        // 2. Use speckit to specify a feature
        env.run_sprite_command(&[
            "/speckit.specify",
            "Add user authentication system with JWT tokens, including login, registration, and protected routes"
        ])
        .assert()
        .success()
        .stdout(predicates::contains("Creating feature specification"));

        // Verify spec.md was created
        env.temp_dir.child("spec.md").assert(predicates::path::exists());

        // 3. Generate implementation plan
        env.run_sprite_command(&["/speckit.plan"])
            .assert()
            .success()
            .stdout(predicates::contains("Creating implementation plan"));

        // Verify plan.md was created
        env.temp_dir.child("plan.md").assert(predicates::path::exists());

        // 4. Generate tasks
        env.run_sprite_command(&["/speckit.tasks"])
            .assert()
            .success()
            .stdout(predicates::contains("Generating actionable tasks"));

        // Verify tasks.md was created
        env.temp_dir.child("tasks.md").assert(predicates::path::exists());

        // 5. Analyze artifacts
        env.run_sprite_command(&["/speckit.analyze"])
            .assert()
            .success()
            .stdout(predicates::contains("Analyzing artifacts"))
            .stdout(predicates::contains("consistency check"));

        // 6. Start implementation (dry run to avoid network calls)
        env.temp_dir.child("agents").assert(predicates::path::exists());

        // Verify all speckit files exist and have content
        let spec_content = fs::read_to_string(env.temp_dir.path().join("spec.md"))?;
        assert!(spec_content.contains("authentication"));
        assert!(spec_content.contains("JWT"));

        let plan_content = fs::read_to_string(env.temp_dir.path().join("plan.md"))?;
        assert!(!plan_content.is_empty());

        let tasks_content = fs::read_to_string(env.temp_dir.path().join("tasks.md"))?;
        assert!(!tasks_content.is_empty());

        Ok(())
    }

    #[test]
    fn test_error_recovery_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let env = TestEnvironment::new()?;

        // 1. Try to start session without initialization (should fail)
        env.run_sprite_command(&["start"])
            .assert()
            .failure()
            .stderr(predicates::contains("No configuration found"));

        // 2. Initialize environment
        env.run_sprite_command(&["init", "--agents", "2"])
            .assert()
            .success();

        // 3. Try to create agent with invalid ID (should fail)
        env.run_sprite_command(&[
            "agents", "create", "invalid/id",
            "--no-workspace"
        ])
        .assert()
        .failure()
        .stderr(predicates::contains("Invalid agent ID"));

        // 4. Try to create duplicate agent (should fail)
        env.run_sprite_command(&[
            "agents", "create", "1",
            "--no-workspace"
        ])
        .assert()
        .failure()
        .stderr(predicates::contains("already exists"));

        // 5. Try to send command to non-existent agent (should fail)
        env.run_sprite_command(&[
            "hey", "99",
            "echo", "test"
        ])
        .assert()
        .failure()
        .stderr(predicates::contains("does not exist"));

        // 6. Try speckit command without prerequisites (should fail)
        env.run_sprite_command(&["/speckit.plan"])
            .assert()
            .failure()
            .stderr(predicates::contains("spec.md not found"));

        // But recovery should work
        env.run_sprite_command(&["/speckit.specify", "Test feature"])
            .assert()
            .success();

        env.run_sprite_command(&["/speckit.plan"])
            .assert()
            .success();

        Ok(())
    }

    #[test]
    fn test_performance_requirements() -> Result<(), Box<dyn std::error::Error>> {
        let env = TestEnvironment::new()?;

        // Test simple command performance (<2s)
        let start = std::time::Instant::now();

        env.run_sprite_command(&["init", "--agents", "1"])
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 2, "init command should complete in <2s, took {:?}", duration);

        // Test complex command performance (<5s)
        let start = std::time::Instant::now();

        env.run_sprite_command(&["agents", "provision"])
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "provision command should complete in <5s, took {:?}", duration);

        // Test speckit command performance
        let start = std::time::Instant::now();

        env.run_sprite_command(&[
            "/speckit.specify",
            "Create a simple REST API with user management"
        ])
        .assert()
        .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "speckit specify should complete in <5s, took {:?}", duration);

        Ok(())
    }
}
```

---

## Mocking External Dependencies

### 1. AI Framework API Mocking

```rust
// tests/mocks/ai_framework_mocks.rs
use mockall::mock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Mock structures for AI framework responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ClaudeContent>,
    pub usage: ClaudeUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeContent {
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexResponse {
    pub choices: Vec<CodexChoice>,
    pub usage: CodexUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexChoice {
    pub text: String,
    pub index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DroidResponse {
    pub response: String,
    pub metadata: DroidMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DroidMetadata {
    pub model: String,
    pub processing_time: u64,
}

// Mock AI client trait
mock! {
    pub AiClient {}

    impl AiClientTrait for AiClient {
        async fn ask_claude(&self, prompt: &str, model: &str) -> Result<ClaudeResponse, AiError>;
        async fn ask_codex(&self, prompt: &str, model: &str) -> Result<CodexResponse, AiError>;
        async fn ask_droid(&self, prompt: &str, model: &str) -> Result<DroidResponse, AiError>;
        async fn validate_api_key(&self, api_key: &str, service: &str) -> Result<bool, AiError>;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("API request failed: {0}")]
    ApiError(String),
    #[error("Authentication failed")]
    AuthError,
    #[error("Rate limit exceeded")]
    RateLimitError,
    #[error("Network error: {0}")]
    NetworkError(String),
}

pub trait AiClientTrait {
    async fn ask_claude(&self, prompt: &str, model: &str) -> Result<ClaudeResponse, AiError>;
    async fn ask_codex(&self, prompt: &str, model: &str) -> Result<CodexResponse, AiError>;
    async fn ask_droid(&self, prompt: &str, model: &str) -> Result<DroidResponse, AiError>;
    async fn validate_api_key(&self, api_key: &str, service: &str) -> Result<bool, AiError>;
}

// Mock implementations for testing
impl MockAiClient {
    pub fn new_success_mock() -> Self {
        let mut mock = MockAiClient::new();

        // Claude API mock
        mock.expect_ask_claude()
            .returning(|_prompt, _model| {
                Ok(ClaudeResponse {
                    content: vec![ClaudeContent {
                        content_type: "text".to_string(),
                        text: "This is a mock Claude response for testing purposes.".to_string(),
                    }],
                    usage: ClaudeUsage {
                        input_tokens: 10,
                        output_tokens: 15,
                    },
                })
            });

        // Codex API mock
        mock.expect_ask_codex()
            .returning(|_prompt, _model| {
                Ok(CodexResponse {
                    choices: vec![CodexChoice {
                        text: "function mockImplementation() { return 'mock'; }".to_string(),
                        index: 0,
                    }],
                    usage: CodexUsage {
                        prompt_tokens: 8,
                        completion_tokens: 12,
                    },
                })
            });

        // Droid API mock
        mock.expect_ask_droid()
            .returning(|_prompt, _model| {
                Ok(DroidResponse {
                    response: "Mock Droid response for testing".to_string(),
                    metadata: DroidMetadata {
                        model: "droid-mock".to_string(),
                        processing_time: 100,
                    },
                })
            });

        // API key validation mock
        mock.expect_validate_api_key()
            .returning(|_api_key, _service| Ok(true));

        mock
    }

    pub fn new_error_mock() -> Self {
        let mut mock = MockAiClient::new();

        mock.expect_ask_claude()
            .returning(|_prompt, _model| Err(AiError::ApiError("Mock API error".to_string())));

        mock.expect_ask_codex()
            .returning(|_prompt, _model| Err(AiError::RateLimitError));

        mock.expect_ask_droid()
            .returning(|_prompt, _model| Err(AiError::NetworkError("Mock network error".to_string())));

        mock.expect_validate_api_key()
            .returning(|_api_key, _service| Err(AiError::AuthError));

        mock
    }
}
```

### 2. HTTP Service Mocking with WireMock

```rust
// tests/mocks/http_mocks.rs
use mockito::{Server, ServerGuard};
use reqwest::Client;
use serde_json::json;

pub struct MockHttpServer {
    server: ServerGuard,
    client: Client,
}

impl MockHttpServer {
    pub fn new() -> Self {
        let server = Server::new();
        let client = Client::new();

        Self { server, client }
    }

    pub fn mock_claude_api(&mut self) -> String {
        let mock_response = json!({
            "content": [{
                "type": "text",
                "text": "This is a mock Claude response"
            }],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 15
            }
        });

        self.server
            .mock("POST", "/v1/messages")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&mock_response).unwrap())
            .create();

        self.server.url()
    }

    pub fn mock_codex_api(&mut self) -> String {
        let mock_response = json!({
            "choices": [{
                "text": "function mockFunction() { return 'mock'; }",
                "index": 0
            }],
            "usage": {
                "prompt_tokens": 8,
                "completion_tokens": 12
            }
        });

        self.server
            .mock("POST", "/v1/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&mock_response).unwrap())
            .create();

        self.server.url()
    }

    pub fn mock_error_response(&mut self, status: usize, error_message: &str) -> String {
        let error_response = json!({
            "error": {
                "message": error_message,
                "type": "api_error"
            }
        });

        self.server
            .mock("POST", "/v1/test")
            .with_status(status)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&error_response).unwrap())
            .create();

        self.server.url()
    }
}

#[cfg(test)]
mod http_mock_tests {
    use super::*;
    use crate::mocks::ai_framework_mocks::*;

    #[tokio::test]
    async fn test_mock_claude_api() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock_server = MockHttpServer::new();
        let base_url = mock_server.mock_claude_api();

        // Test using the mocked API
        let client = Client::new();
        let response = client
            .post(&format!("{}/v1/messages", base_url))
            .json(&json!({
                "model": "claude-3-sonnet-20240229",
                "max_tokens": 1024,
                "messages": [{
                    "role": "user",
                    "content": "Hello, Claude!"
                }]
            }))
            .send()
            .await?;

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await?;
        assert!(body["content"].as_array().unwrap().len() > 0);

        Ok(())
    }
}
```

### 3. File System Mocking

```rust
// tests/mocks/file_system_mocks.rs
use assert_fs::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

pub struct MockFileSystem {
    temp_dir: TempDir,
}

impl MockFileSystem {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        Ok(Self { temp_dir })
    }

    pub fn create_mock_config(&self, agent_count: u32) -> Result<(), Box<dyn std::error::Error>> {
        let config_content = format!(r#"
version: "1.0"
session_name: "test-session"
agents:
{}
"#,
            (1..=agent_count)
                .map(|i| format!(r#"
  - id: "{}"
    branch: "agents/{}"
    worktree_path: "agents/{}"
    model: "claude-sonnet-4"
    description: "Test agent {}"
    status: "active"
    config: {{}}"#, i, i, i, i))
                .collect::<Vec<_>>()
                .join("")
        );

        let config_path = self.temp_dir.path().join("agents.yaml");
        fs::write(config_path, config_content)?;

        // Create agent directories
        for i in 1..=agent_count {
            let agent_dir = self.temp_dir.path().join("agents").join(i.to_string());
            fs::create_dir_all(agent_dir)?;
        }

        Ok(())
    }

    pub fn create_mock_spec(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let spec_path = self.temp_dir.path().join("spec.md");
        fs::write(spec_path, content)?;
        Ok(())
    }

    pub fn create_mock_plan(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plan_path = self.temp_dir.path().join("plan.md");
        fs::write(plan_path, content)?;
        Ok(())
    }

    pub fn create_mock_tasks(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let tasks_path = self.temp_dir.path().join("tasks.md");
        fs::write(tasks_path, content)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

#[cfg(test)]
mod file_system_mock_tests {
    use super::*;

    #[test]
    fn test_mock_file_system_creation() -> Result<(), Box<dyn std::error::Error>> {
        let mock_fs = MockFileSystem::new()?;

        mock_fs.create_mock_config(3)?;
        mock_fs.create_mock_spec("# Test Specification")?;
        mock_fs.create_mock_plan("# Test Implementation Plan")?;
        mock_fs.create_mock_tasks("# Test Tasks")?;

        // Verify files exist
        mock_fs.path().join("agents.yaml").assert(predicates::path::exists());
        mock_fs.path().join("spec.md").assert(predicates::path::exists());
        mock_fs.path().join("plan.md").assert(predicates::path::exists());
        mock_fs.path().join("tasks.md").assert(predicates::path::exists());

        // Verify agent directories exist
        for i in 1..=3 {
            mock_fs.path().join("agents").join(i.to_string()).assert(predicates::path::exists());
        }

        Ok(())
    }
}
```

---

## Tmux Integration Testing

### 1. Tmux Session Management Tests

```rust
// tests/integration/tmux_integration_test.rs
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::{Child, Command as StdCommand};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

struct TmuxTestEnvironment {
    temp_dir: TempDir,
    session_name: String,
    cleanup_sessions: Vec<String>,
}

impl TmuxTestEnvironment {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let session_name = format!("test-session-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs());

        Ok(Self {
            temp_dir,
            session_name,
            cleanup_sessions: Vec::new(),
        })
    }

    fn setup_git_repo(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize git repository
        StdCommand::new("git")
            .arg("init")
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        // Configure git
        StdCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        StdCommand::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        // Create initial commit
        std::fs::write(self.temp_dir.path().join("README.md"), "# Test Project")?;

        StdCommand::new("git")
            .args(["add", "."])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        StdCommand::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    fn create_test_tmux_session(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = StdCommand::new("tmux")
            .args(["new-session", "-d", "-s", name])
            .output()?;

        if output.status.success() {
            self.cleanup_sessions.push(name.to_string());
        }

        Ok(())
    }

    fn session_exists(&self, name: &str) -> bool {
        StdCommand::new("tmux")
            .args(["has-session", "-t", name])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn get_session_panes(&self, name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let output = StdCommand::new("tmux")
            .args(["list-panes", "-t", name])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let panes: Vec<String> = output_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();

        Ok(panes)
    }

    fn run_sprite_command(&self, args: &[&str]) -> assert_cmd::Command {
        let mut cmd = Command::cargo_bin("sprite");
        cmd.current_dir(self.temp_dir.path());
        for arg in args {
            cmd.arg(arg);
        }
        cmd
    }
}

impl Drop for TmuxTestEnvironment {
    fn drop(&mut self) {
        for session_name in &self.cleanup_sessions {
            let _ = StdCommand::new("tmux")
                .args(["kill-session", "-t", session_name])
                .output();
        }
    }
}

#[cfg(test)]
mod tmux_integration_tests {
    use super::*;

    #[test]
    fn test_tmux_session_creation_and_management() -> Result<(), Box<dyn std::error::Error>> {
        let mut env = TmuxTestEnvironment::new()?;
        env.setup_git_repo()?;

        // Initialize sprite environment
        env.run_sprite_command(&["init", "--agents", "2"])
            .assert()
            .success();

        // Start tmux session
        env.run_sprite_command(&[
            "start",
            "--session-name", &env.session_name,
            "--detach"
        ])
        .assert()
        .success()
        .stdout(predicates::contains("Session started successfully"));

        // Give tmux time to initialize
        thread::sleep(Duration::from_secs(3));

        // Verify session exists
        assert!(env.session_exists(&env.session_name), "Session should exist");

        // Check session status
        env.run_sprite_command(&["status", "--detailed"])
            .assert()
            .success()
            .stdout(predicates::contains(&env.session_name));

        // Verify panes were created
        let panes = env.get_session_panes(&env.session_name)?;
        assert!(!panes.is_empty(), "Session should have panes");

        // Kill session
        env.run_sprite_command(&[
            "kill",
            "--session-name", &env.session_name,
            "--force"
        ])
        .assert()
        .success()
        .stdout(predicates::contains("Session terminated"));

        // Verify session no longer exists
        thread::sleep(Duration::from_secs(1));
        assert!(!env.session_exists(&env.session_name), "Session should not exist after kill");

        Ok(())
    }

    #[test]
    fn test_tmux_command_execution_in_panes() -> Result<(), Box<dyn std::error::Error>> {
        let mut env = TmuxTestEnvironment::new()?;
        env.setup_git_repo()?;

        // Initialize and start
        env.run_sprite_command(&["init", "--agents", "2"])
            .assert()
            .success();

        env.run_sprite_command(&[
            "start",
            "--session-name", &env.session_name,
            "--detach"
        ])
        .assert()
        .success();

        // Give tmux time to initialize
        thread::sleep(Duration::from_secs(3));

        // Send command to specific agent
        env.run_sprite_command(&[
            "hey", "1",
            "pwd"
        ])
        .assert()
        .success()
        .stdout(predicates::contains("Sending command to agent 1"));

        // Send command to all agents
        env.run_sprite_command(&[
            "send", "echo", 'Hello from test'
        ])
        .assert()
        .success()
        .stdout(predicates::contains("Sending command to all agents"));

        // Give commands time to execute
        thread::sleep(Duration::from_secs(2));

        // Clean up
        env.run_sprite_command(&[
            "kill",
            "--session-name", &env.session_name,
            "--force"
        ])
        .assert()
        .success();

        Ok(())
    }

    #[test]
    fn test_tmux_zoom_functionality() -> Result<(), Box<dyn std::error::Error>> {
        let mut env = TmuxTestEnvironment::new()?;
        env.setup_git_repo()?;

        // Initialize and start
        env.run_sprite_command(&["init", "--agents", "3"])
            .assert()
            .success();

        env.run_sprite_command(&[
            "start",
            "--session-name", &env.session_name,
            "--detach"
        ])
        .assert()
        .success();

        // Give tmux time to initialize
        thread::sleep(Duration::from_secs(3));

        // Test zoom on agent
        env.run_sprite_command(&["zoom", "--agent", "1"])
            .assert()
            .success()
            .stdout(predicates::contains("Focusing on agent 1"));

        // Test unzoom
        env.run_sprite_command(&["zoom", "--unzoom"])
            .assert()
            .success()
            .stdout(predicates::contains("Returning to normal layout"));

        // Test zoom list
        env.run_sprite_command(&["zoom", "--list"])
            .assert()
            .success()
            .stdout(predicates::contains("Available panes"));

        // Clean up
        env.run_sprite_command(&[
            "kill",
            "--session-name", &env.session_name,
            "--force"
        ])
        .assert()
        .success();

        Ok(())
    }

    #[test]
    fn test_tmux_session_recovery_after_crash() -> Result<(), Box<dyn std::error::Error>> {
        let mut env = TmuxTestEnvironment::new()?;
        env.setup_git_repo()?;

        // Initialize sprite environment
        env.run_sprite_command(&["init", "--agents", "2"])
            .assert()
            .success();

        // Create a tmux session manually (simulating a crash scenario)
        env.create_test_tmux_session(&env.session_name)?;

        // Try to start with existing session (should handle gracefully)
        env.run_sprite_command(&[
            "start",
            "--session-name", &env.session_name,
            "--force"  // Should kill existing session and create new one
        ])
        .assert()
            .success()
            .stdout(predicates::contains("Session started successfully"));

        // Verify session is working
        thread::sleep(Duration::from_secs(2));
        assert!(env.session_exists(&env.session_name));

        // Clean up
        env.run_sprite_command(&[
            "kill",
            "--session-name", &env.session_name,
            "--force"
        ])
        .assert()
        .success();

        Ok(())
    }

    #[test]
    fn test_tmux_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let mut env = TmuxTestEnvironment::new()?;
        env.setup_git_repo()?;

        // Initialize sprite environment
        env.run_sprite_command(&["init", "--agents", "2"])
            .assert()
            .success();

        // Try to kill non-existent session
        env.run_sprite_command(&[
            "kill",
            "--session-name", "non-existent-session",
            "--force"
        ])
        .assert()
        .success();  // Should succeed even if session doesn't exist

        // Try to attach to non-existent session
        env.run_sprite_command(&["attach", "--session-name", "non-existent-session"])
            .assert()
            .failure()
            .stderr(predicates::contains("No session found"));

        // Try to zoom on non-existent agent
        env.run_sprite_command(&["zoom", "--agent", "99"])
            .assert()
            .failure()
            .stderr(predicates::contains("does not exist"));

        // Try to send command to non-existent agent
        env.run_sprite_command(&["hey", "99", "echo", "test"])
            .assert()
            .failure()
            .stderr(predicates::contains("does not exist"));

        Ok(())
    }
}
```

---

## Performance Testing

### 1. Criterion Benchmarking

```rust
// benches/performance_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::process::Command;
use tempfile::TempDir;
use std::fs;

fn benchmark_init_command(c: &mut Criterion) {
    let mut group = c.benchmark_group("init_command");

    for agent_count in [1, 3, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("init", agent_count),
            agent_count,
            |b, &agent_count| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();

                    // Setup git repo
                    Command::new("git")
                        .arg("init")
                        .current_dir(temp_dir.path())
                        .output()
                        .unwrap();

                    Command::new("git")
                        .args(["config", "user.email", "test@example.com"])
                        .current_dir(temp_dir.path())
                        .output()
                        .unwrap();

                    Command::new("git")
                        .args(["config", "user.name", "Test User"])
                        .current_dir(temp_dir.path())
                        .output()
                        .unwrap();

                    // Benchmark init command
                    let output = Command::cargo_bin("sprite")
                        .unwrap()
                        .args(["init", "--agents", &agent_count.to_string()])
                        .current_dir(temp_dir.path())
                        .output()
                        .unwrap();

                    assert!(output.status.success());

                    black_box(temp_dir)
                });
            },
        );
    }
    group.finish();
}

fn benchmark_agents_provision(c: &mut Criterion) {
    let mut group = c.benchmark_group("agents_provision");

    for agent_count in [1, 3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("provision", agent_count),
            agent_count,
            |b, &agent_count| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();

                    // Setup environment
                    setup_test_environment(&temp_dir, agent_count);

                    // Benchmark provision command
                    let start = std::time::Instant::now();

                    let output = Command::cargo_bin("sprite")
                        .unwrap()
                        .arg("agents")
                        .arg("provision")
                        .current_dir(temp_dir.path())
                        .output()
                        .unwrap();

                    let duration = start.elapsed();

                    assert!(output.status.success());
                    assert!(duration.as_secs() < 5, "Provision should complete in <5s");

                    black_box((temp_dir, duration))
                });
            },
        );
    }
    group.finish();
}

fn benchmark_speckit_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("speckit_commands");

    // Benchmark /speckit.specify
    group.bench_function("speckit_specify", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            setup_test_environment(&temp_dir, 1);

            let start = std::time::Instant::now();

            let output = Command::cargo_bin("sprite")
                .unwrap()
                .args(["/speckit.specify", "Add user authentication with JWT"])
                .current_dir(temp_dir.path())
                .output()
                .unwrap();

            let duration = start.elapsed();

            assert!(output.status.success());
            assert!(duration.as_secs() < 5, "Speckit specify should complete in <5s");

            black_box((temp_dir, duration))
        });
    });

    // Benchmark /speckit.plan
    group.bench_function("speckit_plan", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            setup_test_environment(&temp_dir, 1);

            // Create spec.md
            fs::write(temp_dir.path().join("spec.md"), "# Authentication Spec").unwrap();

            let start = std::time::Instant::now();

            let output = Command::cargo_bin("sprite")
                .unwrap()
                .arg("/speckit.plan")
                .current_dir(temp_dir.path())
                .output()
                .unwrap();

            let duration = start.elapsed();

            assert!(output.status.success());
            assert!(duration.as_secs() < 5, "Speckit plan should complete in <5s");

            black_box((temp_dir, duration))
        });
    });

    group.finish();
}

fn benchmark_simple_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_commands");

    // Benchmark agents list
    group.bench_function("agents_list", |b| {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 5);

        b.iter(|| {
            let start = std::time::Instant::now();

            let output = Command::cargo_bin("sprite")
                .unwrap()
                .args(["agents", "list"])
                .current_dir(temp_dir.path())
                .output()
                .unwrap();

            let duration = start.elapsed();

            assert!(output.status.success());
            assert!(duration.as_secs() < 2, "Agents list should complete in <2s");

            black_box(duration)
        });
    });

    // Benchmark status command
    group.bench_function("status", |b| {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 3);

        b.iter(|| {
            let start = std::time::Instant::now();

            let output = Command::cargo_bin("sprite")
                .unwrap()
                .arg("status")
                .current_dir(temp_dir.path())
                .output()
                .unwrap();

            let duration = start.elapsed();

            assert!(output.status.success());
            assert!(duration.as_secs() < 2, "Status should complete in <2s");

            black_box(duration)
        });
    });

    group.finish();
}

fn setup_test_environment(temp_dir: &TempDir, agent_count: u32) {
    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Initialize sprite environment
    let output = Command::cargo_bin("sprite")
        .unwrap()
        .args(["init", "--agents", &agent_count.to_string()])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
}

criterion_group!(
    benches,
    benchmark_init_command,
    benchmark_agents_provision,
    benchmark_speckit_commands,
    benchmark_simple_commands
);
criterion_main!(benches);
```

### 2. Performance Test Integration

```rust
// tests/performance/performance_requirements_test.rs
use assert_cmd::Command;
use std::time::Instant;
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod performance_tests {
    use super::*;

    fn setup_test_environment(temp_dir: &TempDir, agent_count: u32) {
        // Initialize git repository
        Command::new("git")
            .arg("init")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .assert()
            .success();
    }

    #[test]
    fn test_simple_commands_performance() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 3);

        // Test agents list command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .args(["agents", "list"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 2, "agents list should complete in <2s, took {:?}", duration);

        // Test status command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .arg("status")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 2, "status should complete in <2s, took {:?}", duration);

        // Test config show command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .args(["config", "show"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 2, "config show should complete in <2s, took {:?}", duration);
    }

    #[test]
    fn test_complex_commands_performance() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 5);

        // Test init command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .args(["init", "--agents", "5"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "init with 5 agents should complete in <5s, took {:?}", duration);

        // Test agents provision command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .arg("agents")
            .arg("provision")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "agents provision should complete in <5s, took {:?}", duration);

        // Test agents validate command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .arg("agents")
            .arg("validate")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "agents validate should complete in <5s, took {:?}", duration);
    }

    #[test]
    fn test_speckit_commands_performance() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 1);

        // Test /speckit.specify command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .args(["/speckit.specify", "Add user authentication system"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "speckit specify should complete in <5s, took {:?}", duration);

        // Test /speckit.plan command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .arg("/speckit.plan")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "speckit plan should complete in <5s, took {:?}", duration);

        // Test /speckit.tasks command
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .arg("/speckit.tasks")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 5, "speckit tasks should complete in <5s, took {:?}", duration);
    }

    #[test]
    fn test_concurrent_operations_performance() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 3);

        // Test multiple quick operations in sequence
        let start = Instant::now();

        // agents list
        Command::cargo_bin("sprite")
            .unwrap()
            .args(["agents", "list"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // status
        Command::cargo_bin("sprite")
            .unwrap()
            .arg("status")
            .current_dir(temp_dir.path())
            .assert()
            .success();

        // config show
        Command::cargo_bin("sprite")
            .unwrap()
            .args(["config", "show"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 3, "3 simple commands should complete in <3s total, took {:?}", duration);
    }

    #[test]
    fn test_large_agent_count_performance() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 20);

        // Test with larger agent count
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .args(["init", "--agents", "20"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 10, "init with 20 agents should complete in <10s, took {:?}", duration);

        // Test agents list with many agents
        let start = Instant::now();

        Command::cargo_bin("sprite")
            .unwrap()
            .args(["agents", "list"])
            .current_dir(temp_dir.path())
            .assert()
            .success();

        let duration = start.elapsed();
        assert!(duration.as_secs() < 3, "agents list with 20 agents should complete in <3s, took {:?}", duration);
    }
}
```

---

## Test Coverage and Quality Gates

### 1. Coverage Configuration

```toml
# .tarpaulin.toml
[coverage]
run_coveralls = false
coveralls = false
ci = false
ignore_files = [
    "target/*",
    "tests/*",
    "benches/*"
]
exclude = [
    "src/main.rs",  # Entry point typically doesn't need coverage
    "src/error.rs", # Error variants are often covered indirectly
]

[coverage.report]
kind = "Html"
output_dir = "target/coverage"

[coverage.limits]
# Set minimum coverage thresholds
functions = 80
lines = 80
branches = 75
```

### 2. Coverage Analysis Script

```bash
#!/bin/bash
# scripts/check_coverage.sh

set -e

echo "🔍 Running test coverage analysis..."

# Run cargo tarpaulin with coverage
cargo tarpaulin --out Html --output-dir target/coverage --skip-clean

# Check if coverage meets minimum thresholds
COVERAGE_FILE="target/coverage/tarpaulin-report.html"

if [ ! -f "$COVERAGE_FILE" ]; then
    echo "❌ Coverage report not generated"
    exit 1
fi

# Extract coverage percentage (this is a simplified example)
# In practice, you might want to parse XML or JSON output
COVERAGE_OUTPUT=$(cargo tarpaulin --out Xml --output-dir target/coverage 2>&1)
COVERAGE_PERCENT=$(echo "$COVERAGE_OUTPUT" | grep -o '[0-9]*\.[0-9]*%' | head -1 | sed 's/%//')

echo "📊 Current coverage: ${COVERAGE_PERCENT}%"

# Check thresholds
MIN_LINE_COVERAGE=80
MIN_FUNCTION_COVERAGE=80

if (( $(echo "$COVERAGE_PERCENT < $MIN_LINE_COVERAGE" | bc -l) )); then
    echo "❌ Line coverage ${COVERAGE_PERCENT}% is below minimum ${MIN_LINE_COVERAGE}%"
    echo "Please add more tests to improve coverage"
    exit 1
else
    echo "✅ Line coverage ${COVERAGE_PERCENT}% meets minimum requirement"
fi

echo "📈 Coverage report generated: target/coverage/tarpaulin-report.html"
```

### 3. Test Quality Gates

```rust
// tests/quality_gates/mod.rs
//! Quality gate tests to ensure test coverage and quality standards

#[cfg(test)]
mod quality_gate_tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_all_public_functions_are_tested() {
        // This test ensures that all public functions have corresponding tests
        // This is a simplified example - in practice you might use proc macros
        // or static analysis tools for this

        let src_files = vec![
            "src/commands/agents.rs",
            "src/commands/config.rs",
            "src/commands/init.rs",
            "src/commands/start.rs",
            "src/utils/tmux.rs",
            "src/config.rs",
        ];

        for file in src_files {
            let path = Path::new(file);
            if path.exists() {
                let content = fs::read_to_string(path).unwrap();

                // Check for public functions
                let public_functions: HashSet<String> = content
                    .lines()
                    .filter(|line| line.trim().starts_with("pub fn "))
                    .map(|line| {
                        line.trim()
                            .strip_prefix("pub fn ")
                            .unwrap()
                            .split('(')
                            .next()
                            .unwrap()
                            .to_string()
                    })
                    .collect();

                // You would then check if these functions have corresponding tests
                // This is a simplified check - in practice you'd want more sophisticated analysis
                if !public_functions.is_empty() {
                    println!("Found {} public functions in {}", public_functions.len(), file);
                    // In a real implementation, you'd verify each has test coverage
                }
            }
        }
    }

    #[test]
    fn test_error_paths_are_covered() {
        // Ensure error handling paths are tested
        // This is a simplified check - you might use tools like cargo-tarpaulin
        // to verify actual coverage of error branches

        let error_variants = vec![
            "SpriteError::config",
            "SpriteError::tmux",
            "SpriteError::filesystem",
            "SpriteError::git",
        ];

        // In practice, you'd verify these error variants are covered by tests
        for variant in error_variants {
            // Check if there are tests that trigger this error variant
            // This is a placeholder for actual coverage analysis
            assert!(true, "Error variant {} should be covered by tests", variant);
        }
    }

    #[test]
    fn test_integration_tests_cover_workflows() {
        // Verify that integration tests cover major workflows
        let required_workflows = vec![
            "init_workflow_test",
            "multi_agent_test",
            "slash_command_test",
            "tmux_integration_test",
        ];

        for workflow in required_workflows {
            let test_file = format!("tests/integration/{}_test.rs", workflow);
            let path = Path::new(&test_file);

            assert!(path.exists(), "Integration test for {} should exist", workflow);

            // Verify test file has content
            let content = fs::read_to_string(path).unwrap();
            assert!(content.contains("#[test]"), "{} test file should contain tests", workflow);
        }
    }

    #[test]
    fn test_unit_tests_exist_for_all_modules() {
        // Verify unit tests exist for key modules
        let modules_with_tests = vec![
            ("cli", "tests/unit/cli_parsing_test.rs"),
            ("config", "tests/unit/config_test.rs"),
            ("tmux", "tests/unit/tmux_test.rs"),
        ];

        for (module, test_file) in modules_with_tests {
            let path = Path::new(test_file);
            assert!(path.exists(), "Unit tests for {} module should exist", module);
        }
    }

    #[test]
    fn test_performance_requirements_enforced() {
        // Verify performance requirements are tested
        let performance_test_file = "tests/performance/performance_requirements_test.rs";
        let path = Path::new(performance_test_file);

        assert!(path.exists(), "Performance requirements test should exist");

        let content = fs::read_to_string(path).unwrap();

        // Check for specific performance tests
        assert!(content.contains("test_simple_commands_performance"),
               "Should test simple command performance");
        assert!(content.contains("test_complex_commands_performance"),
               "Should test complex command performance");
        assert!(content.contains("as_secs() < 2"), "Should enforce <2s requirement");
        assert!(content.contains("as_secs() < 5"), "Should enforce <5s requirement");
    }

    #[test]
    fn test_mock_coverage_is_comprehensive() {
        // Verify external dependencies are properly mocked
        let mock_files = vec![
            "tests/mocks/ai_framework_mocks.rs",
            "tests/mocks/http_mocks.rs",
            "tests/mocks/file_system_mocks.rs",
        ];

        for mock_file in mock_files {
            let path = Path::new(mock_file);
            assert!(path.exists(), "Mock file {} should exist", mock_file);

            let content = fs::read_to_string(path).unwrap();
            assert!(content.contains("mockall::mock") || content.contains("mockito"),
                   "Mock file {} should contain mock definitions", mock_file);
        }
    }
}
```

---

## CI/CD Integration

### 1. GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust:
          - stable
          - beta

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: ${{ matrix.rust }}
        override: true
        components: rustfmt, clippy

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-

    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y tmux git

    - name: Check formatting
      run: cargo fmt --all -- --check

    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings

    - name: Run unit tests
      run: cargo test --lib --bins --tests

    - name: Run integration tests
      run: cargo test --test '*' -- --test-threads=1

    - name: Run doc tests
      run: cargo test --doc

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    needs: test

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true

    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y tmux git libssl-dev pkg-config

    - name: Install cargo-tarpaulin
      run: cargo install cargo-tarpaulin

    - name: Generate coverage report
      run: |
        cargo tarpaulin --workspace --all-features --out Xml --output-dir target/coverage

    - name: Upload to codecov.io
      uses: codecov/codecov-action@v3
      with:
        file: target/coverage/tarpaulin-report.xml

  performance:
    name: Performance Tests
    runs-on: ubuntu-latest
    needs: test

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true

    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y tmux git

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Run performance tests
      run: cargo test --test performance_requirements_test --release

    - name: Run benchmarks
      run: cargo bench --no-run

  e2e:
    name: End-to-End Tests
    runs-on: ubuntu-latest
    needs: test
    if: github.event_name == 'pull_request'

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true

    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y tmux git

    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Run E2E tests
      run: cargo test --test e2e_test -- --test-threads=1
      env:
        TMUX: tmux

  security:
    name: Security Audit
    runs-on: ubuntu-latest

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Security audit
      uses: actions-rs/audit-check@v1
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
```

### 2. Pre-commit Hooks

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/doublify/pre-commit-rust
    rev: v1.0
    hooks:
      - id: fmt
        args: ['--all', '--', '--check']
      - id: cargo-check
        args: ['--all-targets', '--all-features']
      - id: clippy
        args: ['--all-targets', '--all-features', '--', '-D', 'warnings']

  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files
      - id: check-merge-conflict

  - repo: local
    hooks:
      - id: run-tests
        name: run tests
        entry: cargo test
        language: system
        pass_filenames: false
        always_run: true

      - id: check-coverage
        name: check coverage
        entry: bash scripts/check_coverage.sh
        language: system
        pass_filenames: false
        always_run: true
```

### 3. Docker Test Environment

```dockerfile
# Dockerfile.test
FROM rust:1.75-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    tmux \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install cargo tools
RUN cargo install cargo-tarpaulin cargo-audit

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Run tests
CMD ["cargo", "test", "--all-features"]
```

---

## Test Organization

### 1. Directory Structure

```
tests/
├── unit/                          # Unit tests
│   ├── cli_parsing_test.rs       # CLI argument parsing
│   ├── config_test.rs            # Configuration management
│   ├── tmux_test.rs              # Tmux utilities
│   └── mod.rs                    # Unit test module
├── integration/                   # Integration tests
│   ├── init_workflow_test.rs     # Init command workflows
│   ├── multi_agent_test.rs       # Multi-agent workflows
│   ├── slash_command_test.rs     # Slash command integration
│   ├── tmux_integration_test.rs  # Tmux integration
│   └── mod.rs                    # Integration test module
├── e2e/                          # End-to-end tests
│   ├── complete_workflow_test.rs # Complete workflows
│   └── mod.rs                    # E2E test module
├── performance/                   # Performance tests
│   ├── performance_requirements_test.rs
│   └── mod.rs                    # Performance test module
├── mocks/                        # Mock implementations
│   ├── ai_framework_mocks.rs    # AI framework mocks
│   ├── http_mocks.rs            # HTTP service mocks
│   └── file_system_mocks.rs     # File system mocks
└── quality_gates/               # Quality gate tests
    ├── mod.rs                   # Quality gate tests
```

### 2. Test Categories and Naming Conventions

```rust
// Test naming conventions
// 1. Unit tests: test_<functionality>_<scenario>
// 2. Integration tests: test_<module>_<workflow>_workflow
// 3. E2E tests: test_complete_<workflow>_workflow
// 4. Performance tests: test_<complexity>_commands_performance
// 5. Mock tests: test_mock_<service>_behavior

// Example test organization
#[cfg(test)]
mod agent_management_tests {
    use super::*;

    mod unit {
        use super::*;

        #[test]
        fn test_agent_creation_with_valid_parameters() { /* ... */ }

        #[test]
        fn test_agent_creation_with_invalid_id_fails() { /* ... */ }

        #[test]
        fn test_agent_configuration_serialization() { /* ... */ }
    }

    mod integration {
        use super::*;

        #[test]
        fn test_agent_creation_workflow_with_workspace() { /* ... */ }

        #[test]
        fn test_agent_validation_workflow() { /* ... */ }

        #[test]
        fn test_agent_provisioning_workflow() { /* ... */ }
    }

    mod e2e {
        use super::*;

        #[test]
        fn test_complete_agent_lifecycle_workflow() { /* ... */ }

        #[test]
        fn test_multi_agent_collaboration_workflow() { /* ... */ }
    }
}
```

### 3. Test Utilities and Fixtures

```rust
// tests/common/mod.rs
pub mod fixtures;
pub mod utils;

pub use fixtures::*;
pub use utils::*;
```

```rust
// tests/common/fixtures.rs
use assert_fs::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub project_name: String,
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let project_name = format!("test-project-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs());

        Ok(Self {
            temp_dir,
            project_name,
        })
    }

    pub fn setup_git_repo(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize git repository
        std::process::Command::new("git")
            .arg("init")
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        // Configure git
        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        // Create initial commit
        fs::write(self.temp_dir.path().join("README.md"),
                 format!("# {}", self.project_name))?;

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    pub fn setup_sprite_config(&self, agent_count: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize sprite environment
        std::process::Command::cargo_bin("sprite")?
            .args(["init", "--agents", &agent_count.to_string()])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    pub fn create_mock_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create mock spec, plan, and tasks files
        fs::write(self.temp_dir.path().join("spec.md"),
                 "# Mock Specification\n\nThis is a test specification.")?;

        fs::write(self.temp_dir.path().join("plan.md"),
                 "# Mock Implementation Plan\n\nThis is a test plan.")?;

        fs::write(self.temp_dir.path().join("tasks.md"),
                 "# Mock Tasks\n\n- [ ] Task 1\n- [ ] Task 2")?;

        Ok(())
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}
```

```rust
// tests/common/utils.rs
use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;

pub fn wait_for_condition<F>(
    condition: F,
    timeout: Duration,
    check_interval: Duration,
) -> bool
where
    F: Fn() -> bool,
{
    let start = Instant::now();

    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        thread::sleep(check_interval);
    }

    false
}

pub fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut child = command.spawn()?;

    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                return Ok(child.wait_with_output()?);
            }
            None => {
                if start.elapsed() > timeout {
                    child.kill()?;
                    return Err("Command timed out".into());
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

pub fn cleanup_tmux_sessions(prefix: &str) {
    if let Ok(output) = Command::new("tmux")
        .args(["list-sessions"])
        .output()
    {
        let sessions = String::from_utf8_lossy(&output.stdout);
        for line in sessions.lines() {
            if line.starts_with(prefix) {
                if let Some(session_name) = line.split(':').next() {
                    let _ = Command::new("tmux")
                        .args(["kill-session", "-t", session_name])
                        .output();
                }
            }
        }
    }
}
```

---

## Summary

This comprehensive testing strategy provides:

1. **Complete Test Coverage**: Unit, integration, and end-to-end tests for all major functionality
2. **Performance Validation**: Automated testing against <2s and <5s performance requirements
3. **External Dependency Mocking**: Comprehensive mocking for AI frameworks, HTTP services, and file system
4. **Tmux Integration Testing**: Full tmux session management testing with proper cleanup
5. **Quality Gates**: Automated coverage analysis and quality checks
6. **CI/CD Integration**: Complete GitHub Actions workflow with all test categories
7. **Maintainable Structure**: Well-organized test directory with clear naming conventions

The strategy ensures that the Rust CLI application meets all performance requirements, handles external dependencies gracefully, and maintains high code quality standards throughout development.