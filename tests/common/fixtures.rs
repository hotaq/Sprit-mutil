//! Test fixtures and test data setup

use assert_cmd::prelude::*;
use assert_fs::TempDir;
use std::fs;
use std::path::Path;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub project_name: String,
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let project_name = format!(
            "test-project-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs()
        );

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
        fs::write(
            self.temp_dir.path().join("README.md"),
            format!("# {}", self.project_name),
        )?;

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

    #[allow(dead_code)]
    pub fn setup_sprite_config(&self, agent_count: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize sprite environment
        std::process::Command::cargo_bin("sprite")?
            .args(["init", "--agents", &agent_count.to_string()])
            .current_dir(self.temp_dir.path())
            .assert()
            .success();

        Ok(())
    }

    #[allow(dead_code)]
    pub fn create_mock_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create mock spec, plan, and tasks files
        fs::write(
            self.temp_dir.path().join("spec.md"),
            "# Mock Specification\n\nThis is a test specification.",
        )?;

        fs::write(
            self.temp_dir.path().join("plan.md"),
            "# Mock Implementation Plan\n\nThis is a test plan.",
        )?;

        fs::write(
            self.temp_dir.path().join("tasks.md"),
            "# Mock Tasks\n\n- [ ] Task 1\n- [ ] Task 2",
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}
