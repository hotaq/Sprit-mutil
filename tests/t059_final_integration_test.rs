//! T059 Final integration test for communication commands
//! Focus on core functionality validation

use std::thread;
use std::time::Duration;

mod common;
use common::{cleanup_tmux_sessions, TestFixture};

/// T059 Core communication commands validation
#[test]
#[ignore] // Requires tmux environment
fn test_t059_final_validation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    println!("🧪 T059: Final communication commands validation...");

    // Clean up any existing sessions first
    cleanup_tmux_sessions("sprite-");

    // Setup basic test environment
    fixture.setup_git_repo()?;

    println!("✅ Test environment setup complete");

    // Test 1: Initialize sprite environment
    let mut sprite_init = assert_cmd::Command::cargo_bin("sprite")?;
    sprite_init
        .args(["init", "--force", "--agents", "2"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    println!("✅ Sprite init successful");

    // Test 2: Start sprite session
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["start", "--force"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    println!("✅ Sprite start successful");

    thread::sleep(Duration::from_secs(3));

    // Test 3: Status command basic functionality
    let start_time = std::time::Instant::now();
    let mut status_check = assert_cmd::Command::cargo_bin("sprite")?;
    status_check
        .args(["status"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let status_time = start_time.elapsed();
    assert!(
        status_time < Duration::from_secs(5),
        "Status command took {:?}",
        status_time
    );
    println!("✅ Status command successful {:?} (< 5s)", status_time);

    // Test 4: Agents command basic functionality
    let start_time = std::time::Instant::now();
    let mut agents_check = assert_cmd::Command::cargo_bin("sprite")?;
    let agents_result = agents_check
        .args(["agents", "list"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let agents_time = start_time.elapsed();
    assert!(
        agents_time < Duration::from_secs(3),
        "Agents command took {:?}",
        agents_time
    );

    let agents_output = String::from_utf8_lossy(&agents_result.get_output().stdout);
    // Should mention agents or configuration
    assert!(
        !agents_output.is_empty(),
        "Agents command should output something"
    );
    println!("✅ Agents command successful {:?} (< 3s)", agents_time);

    // Test 5: Hey command basic functionality
    let start_time = std::time::Instant::now();
    let mut hey_single = assert_cmd::Command::cargo_bin("sprite")?;
    hey_single
        .args(["hey", "1", "echo", "\"Hello from agent 1\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let hey_time = start_time.elapsed();
    assert!(
        hey_time < Duration::from_secs(5),
        "Hey command took {:?}",
        hey_time
    );
    println!("✅ Hey command successful {:?} (< 5s)", hey_time);

    thread::sleep(Duration::from_secs(2));

    // Test 6: Hey command with multiple agents
    let start_time = std::time::Instant::now();
    let mut hey_multiple = assert_cmd::Command::cargo_bin("sprite")?;
    hey_multiple
        .args(["hey", "1,2", "echo", "\"Hello from multiple agents\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let hey_multi_time = start_time.elapsed();
    assert!(
        hey_multi_time < Duration::from_secs(5),
        "Hey multi command took {:?}",
        hey_multi_time
    );
    println!(
        "✅ Hey multiple agents successful {:?} (< 5s)",
        hey_multi_time
    );

    thread::sleep(Duration::from_secs(2));

    // Test 7: Hey command broadcast
    let start_time = std::time::Instant::now();
    let mut hey_broadcast = assert_cmd::Command::cargo_bin("sprite")?;
    hey_broadcast
        .args(["hey", "all", "echo", "\"Broadcast to all agents\""])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let broadcast_time = start_time.elapsed();
    assert!(
        broadcast_time < Duration::from_secs(5),
        "Hey broadcast took {:?}",
        broadcast_time
    );
    println!("✅ Hey broadcast successful {:?} (< 5s)", broadcast_time);

    // Test 8: Status with detailed flag
    let start_time = std::time::Instant::now();
    let mut status_detailed = assert_cmd::Command::cargo_bin("sprite")?;
    status_detailed
        .args(["status", "--detailed"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let detailed_time = start_time.elapsed();
    assert!(
        detailed_time < Duration::from_secs(5),
        "Detailed status took {:?}",
        detailed_time
    );
    println!("✅ Status detailed successful {:?} (< 5s)", detailed_time);

    // Test 9: Agents validate command
    let start_time = std::time::Instant::now();
    let mut agents_validate = assert_cmd::Command::cargo_bin("sprite")?;
    agents_validate
        .args(["agents", "validate"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();
    let validate_time = start_time.elapsed();
    assert!(
        validate_time < Duration::from_secs(5),
        "Agents validate took {:?}",
        validate_time
    );
    println!("✅ Agents validate successful {:?} (< 5s)", validate_time);

    // Cleanup
    cleanup_tmux_sessions("sprite-");

    println!("\n🎉 T059 FINAL VALIDATION COMPLETE!");
    println!("✅ All communication commands working: status, agents, hey");
    println!("✅ Performance requirements met: all < 5 seconds");
    println!("✅ Basic error handling: commands succeed gracefully");
    println!("✅ Single, multiple, and broadcast functionality verified");

    Ok(())
}

/// T059 Error scenarios validation
#[test]
fn test_t059_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    println!("🧪 T059: Error scenarios validation...");

    // Test 1: Hey command without sprite session
    let mut hey_no_session = assert_cmd::Command::cargo_bin("sprite")?;
    hey_no_session
        .args(["hey", "1", "echo", "test"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .failure();
    println!("✅ Hey without session correctly fails");

    // Test 2: Status command without sprite session (graceful)
    let mut status_no_session = assert_cmd::Command::cargo_bin("sprite")?;
    let status_result = status_no_session
        .args(["status"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    let status_output = String::from_utf8_lossy(&status_result.get_output().stdout);
    // Should handle missing session gracefully
    assert!(!status_output.is_empty());
    println!("✅ Status without session handles gracefully");

    println!("🎉 T059 Error scenarios validation PASSED!");
    Ok(())
}

/// T059 Performance benchmark
#[test]
fn test_t059_performance_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    println!("🧪 T059: Performance benchmark...");

    // Clean up any existing sessions first
    cleanup_tmux_sessions("sprite-");

    // Setup
    fixture.setup_git_repo()?;

    // Initialize sprite with 2 agents
    let mut sprite_init = assert_cmd::Command::cargo_bin("sprite")?;
    sprite_init
        .args(["init", "--force", "--agents", "2"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    // Start session
    assert_cmd::Command::cargo_bin("sprite")?
        .args(["start", "--force"])
        .current_dir(fixture.temp_dir.path())
        .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
        .env_remove("SPRITE_PROJECT_ROOT")
        .assert()
        .success();

    thread::sleep(Duration::from_secs(2));

    // Performance benchmark: 10 rapid hey commands
    let mut times = Vec::new();
    for i in 1..=10 {
        let start_time = std::time::Instant::now();
        assert_cmd::Command::cargo_bin("sprite")?
            .args(["hey", "1", "echo", &format!("\"Rapid test {}\"", i)])
            .current_dir(fixture.temp_dir.path())
            .env("SPRITE_DISABLE_EXE_DISCOVERY", "1")
            .env_remove("SPRITE_PROJECT_ROOT")
            .assert()
            .success();
        let command_time = start_time.elapsed();
        times.push(command_time);

        // Small delay between commands
        thread::sleep(Duration::from_millis(200));
    }

    // Calculate metrics
    let total_time: Duration = times.iter().sum();
    let avg_time = total_time / times.len() as u32;
    let max_time = *times.iter().max().unwrap();
    let min_time = *times.iter().min().unwrap();

    println!("Performance metrics for 10 hey commands:");
    println!("  Total time: {:?}", total_time);
    println!("  Average time: {:?}", avg_time);
    println!("  Min time: {:?}", min_time);
    println!("  Max time: {:?}", max_time);

    // Verify performance requirements
    assert!(
        avg_time < Duration::from_secs(2),
        "Average time should be < 2s, got {:?}",
        avg_time
    );
    assert!(
        max_time < Duration::from_secs(3),
        "Max time should be < 3s, got {:?}",
        max_time
    );

    cleanup_tmux_sessions("sprite-");

    println!("🎉 T059 Performance benchmark PASSED!");
    Ok(())
}
