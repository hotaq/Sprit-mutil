//! Performance requirements tests

use assert_cmd::Command;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

fn setup_test_environment(temp_dir: &TempDir, _agent_count: u32) {
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
    assert!(
        duration.as_secs() < 2,
        "agents list should complete in <2s, took {:?}",
        duration
    );

    // Test status command
    let start = Instant::now();

    Command::cargo_bin("sprite")
        .unwrap()
        .arg("status")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let duration = start.elapsed();
    assert!(
        duration.as_secs() < 2,
        "status should complete in <2s, took {:?}",
        duration
    );

    // Test config show command
    let start = Instant::now();

    Command::cargo_bin("sprite")
        .unwrap()
        .args(["config", "show"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let duration = start.elapsed();
    assert!(
        duration.as_secs() < 2,
        "config show should complete in <2s, took {:?}",
        duration
    );
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
    assert!(
        duration.as_secs() < 5,
        "init with 5 agents should complete in <5s, took {:?}",
        duration
    );

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
    assert!(
        duration.as_secs() < 5,
        "agents validate should complete in <5s, took {:?}",
        duration
    );
}

#[test]
#[ignore] // Commands not yet implemented
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
    assert!(
        duration.as_secs() < 5,
        "speckit specify should complete in <5s, took {:?}",
        duration
    );

    // Test /speckit.plan command
    let start = Instant::now();

    Command::cargo_bin("sprite")
        .unwrap()
        .arg("/speckit.plan")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let duration = start.elapsed();
    assert!(
        duration.as_secs() < 5,
        "speckit plan should complete in <5s, took {:?}",
        duration
    );

    // Test /speckit.tasks command
    let start = Instant::now();

    Command::cargo_bin("sprite")
        .unwrap()
        .arg("/speckit.tasks")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let duration = start.elapsed();
    assert!(
        duration.as_secs() < 5,
        "speckit tasks should complete in <5s, took {:?}",
        duration
    );
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
    assert!(
        duration.as_secs() < 3,
        "3 simple commands should complete in <3s total, took {:?}",
        duration
    );
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
    assert!(
        duration.as_secs() < 10,
        "init with 20 agents should complete in <10s, took {:?}",
        duration
    );

    // Test agents list with many agents
    let start = Instant::now();

    Command::cargo_bin("sprite")
        .unwrap()
        .args(["agents", "list"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let duration = start.elapsed();
    assert!(
        duration.as_secs() < 3,
        "agents list with 20 agents should complete in <3s, took {:?}",
        duration
    );
}
