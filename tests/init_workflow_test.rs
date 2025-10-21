//! Integration tests for the complete init workflow
//!
//! This module tests the entire init command workflow from start to finish,
//! ensuring all components work together correctly.

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

/// Test the complete init workflow with default settings
#[test]
fn test_complete_init_workflow_default() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize a git repository first (required by init)
    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    // Run sprite init with default settings
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .arg("init")
        .assert()
        .success()
        .stdout(contains("🚀 Initializing Sprite multi-agent environment"))
        .stdout(contains("✅ Sprite environment initialized successfully"))
        .stdout(contains("Created directories:"));

    // Verify the complete directory structure was created
    let agents_dir = temp_path.join("agents");
    assert!(agents_dir.exists(), "agents directory should exist");

    let scripts_dir = agents_dir.join("scripts");
    assert!(scripts_dir.exists(), "scripts directory should exist");

    let profiles_dir = agents_dir.join("profiles");
    assert!(profiles_dir.exists(), "profiles directory should exist");

    // Verify configuration file was created
    let config_file = agents_dir.join("agents.yaml");
    assert!(config_file.exists(), "agents.yaml should exist");

    // Verify config content
    let config_content = fs::read_to_string(&config_file).unwrap();
    assert!(config_content.contains("version:"));
    assert!(config_content.contains("agents:"));
    assert!(config_content.contains("session_name:"));

    // Verify agent directories were created (default is 3 agents)
    for i in 1..=3 {
        let agent_dir = agents_dir.join(i.to_string());
        assert!(agent_dir.exists(), "agent {} directory should exist", i);
    }

    // Verify shell script templates exist
    let setup_script = scripts_dir.join("setup.sh");
    let start_script = scripts_dir.join("start-agents.sh");
    let attach_script = scripts_dir.join("attach-session.sh");

    assert!(setup_script.exists(), "setup.sh should exist");
    assert!(start_script.exists(), "start-agents.sh should exist");
    assert!(attach_script.exists(), "attach-session.sh should exist");

    // Verify tmux profile templates exist
    for i in 0..=5 {
        let profile_file = profiles_dir.join(format!("profile{}.sh", i));
        assert!(profile_file.exists(), "profile{}.sh should exist", i);
    }

    // Check scripts are executable (on Unix systems)
    #[cfg(unix)]
    {
        let setup_perms = fs::metadata(&setup_script).unwrap().permissions();
        assert_eq!(
            setup_perms.mode() & 0o111,
            0o111,
            "setup.sh should be executable"
        );

        let start_perms = fs::metadata(&start_script).unwrap().permissions();
        assert_eq!(
            start_perms.mode() & 0o111,
            0o111,
            "start-agents.sh should be executable"
        );

        let attach_perms = fs::metadata(&attach_script).unwrap().permissions();
        assert_eq!(
            attach_perms.mode() & 0o111,
            0o111,
            "attach-session.sh should be executable"
        );
    }
}

/// Test init workflow with custom agent count
#[test]
fn test_init_workflow_custom_agent_count() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    // Run sprite init with 5 agents
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .args(["init", "--agents", "5"])
        .assert()
        .success()
        .stdout(contains("🤖 Configured 5 agents:"));

    // Verify all 5 agent directories were created
    let agents_dir = temp_path.join("agents");
    for i in 1..=5 {
        let agent_dir = agents_dir.join(i.to_string());
        assert!(agent_dir.exists(), "agent {} directory should exist", i);
    }

    // Verify config contains all 5 agents
    let config_file = agents_dir.join("agents.yaml");
    let config_content = fs::read_to_string(&config_file).unwrap();

    for i in 1..=5 {
        assert!(
            config_content.contains(&format!("- id: \"{}\"", i)),
            "config should contain agent {}",
            i
        );
    }

    // Should not contain agent 6
    assert!(
        !config_content.contains("- id: \"6\""),
        "config should not contain agent 6"
    );
}

/// Test init workflow with zero agents (empty configuration)
#[test]
fn test_init_workflow_zero_agents() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    // Run sprite init with 0 agents
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .args(["init", "--agents", "0"])
        .assert()
        .success();

    // Verify agents directory exists but no agent directories
    let agents_dir = temp_path.join("agents");
    assert!(agents_dir.exists(), "agents directory should exist");

    // Should not create individual agent directories
    for i in 1..=5 {
        let agent_dir = agents_dir.join(i.to_string());
        assert!(
            !agent_dir.exists(),
            "agent {} directory should not exist for 0 agents",
            i
        );
    }

    // Verify empty configuration was used
    let config_file = agents_dir.join("agents.yaml");
    let config_content = fs::read_to_string(&config_file).unwrap();
    assert!(config_content.contains("agents:"));
    assert!(!config_content.contains("- id: \"1\""));
}

/// Test init workflow with force flag when config already exists
#[test]
fn test_init_workflow_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    // Run init once to create initial setup
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .args(["init", "--agents", "2"])
        .assert()
        .success();

    // Modify the config file to detect changes
    let config_file = temp_path.join("agents").join("agents.yaml");
    let original_content = fs::read_to_string(&config_file).unwrap();

    // Run init again without force (should fail)
    let mut cmd_fail = Command::cargo_bin("sprit-mutil").unwrap();
    cmd_fail
        .current_dir(&temp_path)
        .args(["init", "--agents", "3"])
        .assert()
        .failure()
        .stderr(contains("already exists"));

    // Run init again with force (should succeed)
    let mut cmd_force = Command::cargo_bin("sprit-mutil").unwrap();
    cmd_force
        .current_dir(&temp_path)
        .args(["init", "--agents", "3", "--force"])
        .assert()
        .success();

    // Verify config was updated
    let new_content = fs::read_to_string(&config_file).unwrap();
    assert_ne!(
        original_content, new_content,
        "config should be updated with force flag"
    );

    // Should now have 3 agents
    assert!(
        new_content.contains("- id: \"3\""),
        "config should contain agent 3"
    );
}

/// Test init workflow error handling when not in git repository
#[test]
fn test_init_workflow_not_git_repository() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Don't initialize git repository

    // Run sprite init (should fail)
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .arg("init")
        .assert()
        .failure()
        .stderr(contains("Failed to validate git repository"));
}

/// Test init workflow edge case with very large agent count
#[test]
fn test_init_workflow_large_agent_count() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    // Run sprite init with large number of agents
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .args(["init", "--agents", "50"])
        .assert()
        .success();

    // Verify all 50 agent directories were created
    let agents_dir = temp_path.join("agents");
    for i in 1..=50 {
        let agent_dir = agents_dir.join(i.to_string());
        assert!(agent_dir.exists(), "agent {} directory should exist", i);
    }

    // Verify config contains all agents
    let config_file = agents_dir.join("agents.yaml");
    let config_content = fs::read_to_string(&config_file).unwrap();

    // Check a few agents from the beginning, middle, and end
    assert!(
        config_content.contains("- id: \"1\""),
        "config should contain agent 1"
    );
    assert!(
        config_content.contains("- id: \"25\""),
        "config should contain agent 25"
    );
    assert!(
        config_content.contains("- id: \"50\""),
        "config should contain agent 50"
    );
}

/// Test init workflow performance requirement (under 60 seconds)
#[test]
fn test_init_workflow_performance_requirement() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    // Measure time for init command
    let start = std::time::Instant::now();

    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .args(["init", "--agents", "5"])
        .assert()
        .success();

    let duration = start.elapsed();

    // Should complete in under 60 seconds (requirement is much less, but 60 is a reasonable test timeout)
    assert!(
        duration.as_secs() < 60,
        "init should complete in under 60 seconds, took {:?}",
        duration
    );

    // Should complete very quickly (actual requirement is much less)
    assert!(
        duration.as_secs() < 5,
        "init should complete very quickly, took {:?}",
        duration
    );
}

/// Test init workflow with nested directory structure
#[test]
fn test_init_workflow_nested_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create nested directory structure
    let nested_path = temp_path.join("deep").join("nested").join("project");
    fs::create_dir_all(&nested_path).unwrap();

    // Initialize git repository in nested directory
    Command::new("git")
        .arg("init")
        .current_dir(&nested_path)
        .assert()
        .success();

    // Run sprite init from nested directory
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&nested_path)
        .args(["init", "--agents", "2"])
        .assert()
        .success()
        .stdout(contains("✅ Sprite environment initialized successfully"));

    // Verify directory structure was created in nested location
    let agents_dir = nested_path.join("agents");
    assert!(
        agents_dir.exists(),
        "agents directory should exist in nested path"
    );

    // Verify agent directories
    for i in 1..=2 {
        let agent_dir = agents_dir.join(i.to_string());
        assert!(
            agent_dir.exists(),
            "agent {} directory should exist in nested path",
            i
        );
    }
}

/// Test init workflow creates valid YAML configuration
#[test]
fn test_init_workflow_valid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    // Run sprite init
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .args(["init", "--agents", "3"])
        .assert()
        .success();

    // Verify the generated YAML is valid by parsing it with serde_yaml
    let config_file = temp_path.join("agents").join("agents.yaml");
    let config_content = fs::read_to_string(&config_file).unwrap();

    // Try to parse the YAML - this will panic if invalid
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&config_content).expect("Generated config should be valid YAML");

    // Verify basic structure
    assert!(
        parsed.get("agents").is_some(),
        "Config should have agents field"
    );
    assert!(
        parsed.get("version").is_some(),
        "Config should have version field"
    );
    assert!(
        parsed.get("session_name").is_some(),
        "Config should have session_name field"
    );
}

/// Test init workflow creates all required file types
#[test]
fn test_init_workflow_all_file_types() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .arg("init")
        .current_dir(&temp_path)
        .assert()
        .success();

    // Run sprite init
    let mut cmd = Command::cargo_bin("sprit-mutil").unwrap();
    cmd.current_dir(&temp_path)
        .args(["init", "--agents", "2"])
        .assert()
        .success();

    // Verify expected files were created by checking specific files
    let agents_dir = temp_path.join("agents");

    // Check main config file
    let config_file = agents_dir.join("agents.yaml");
    assert!(config_file.exists(), "agents.yaml should exist");
    let config_content = fs::read_to_string(&config_file).unwrap();
    assert!(
        config_content.contains("agents:"),
        "Config should contain agents section"
    );

    // Check script files
    let scripts_dir = agents_dir.join("scripts");
    assert!(scripts_dir.exists(), "scripts directory should exist");

    let setup_script = scripts_dir.join("setup.sh");
    let start_script = scripts_dir.join("start-agents.sh");
    let attach_script = scripts_dir.join("attach-session.sh");

    assert!(setup_script.exists(), "setup.sh should exist");
    assert!(start_script.exists(), "start-agents.sh should exist");
    assert!(attach_script.exists(), "attach-session.sh should exist");

    // Check profile files
    let profiles_dir = agents_dir.join("profiles");
    assert!(profiles_dir.exists(), "profiles directory should exist");

    for i in 0..=5 {
        let profile_file = profiles_dir.join(format!("profile{}.sh", i));
        assert!(profile_file.exists(), "profile{}.sh should exist", i);
    }

    // Check agent directories
    for i in 1..=2 {
        let agent_dir = agents_dir.join(i.to_string());
        assert!(agent_dir.exists(), "agent {} directory should exist", i);
    }

    // Check scripts are executable (on Unix systems)
    #[cfg(unix)]
    {
        for script in ["setup.sh", "start-agents.sh", "attach-session.sh"] {
            let script_path = scripts_dir.join(script);
            let metadata = fs::metadata(&script_path).unwrap();
            assert_eq!(
                metadata.permissions().mode() & 0o111,
                0o111,
                "Script {} should be executable",
                script
            );
        }
    }
}
