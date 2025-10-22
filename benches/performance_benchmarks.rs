//! Performance benchmarks for Sprite CLI commands

use assert_cmd::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

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

fn benchmark_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_operations");

    // Benchmark config show
    group.bench_function("config_show", |b| {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 3);

        b.iter(|| {
            let output = Command::cargo_bin("sprite")
                .unwrap()
                .args(["config", "show"])
                .current_dir(temp_dir.path())
                .output()
                .unwrap();

            assert!(output.status.success());
            black_box(output)
        });
    });

    // Benchmark config validate
    group.bench_function("config_validate", |b| {
        let temp_dir = TempDir::new().unwrap();
        setup_test_environment(&temp_dir, 3);

        b.iter(|| {
            let output = Command::cargo_bin("sprite")
                .unwrap()
                .args(["config", "validate"])
                .current_dir(temp_dir.path())
                .output()
                .unwrap();

            assert!(output.status.success());
            black_box(output)
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
    benchmark_simple_commands,
    benchmark_config_operations
);
criterion_main!(benches);
