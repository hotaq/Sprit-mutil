//! Session recovery and cleanup utilities for the Sprite multi-agent workflow toolkit.
//!
//! This module provides functions for detecting orphaned sessions, cleaning up
//! dead resources, and recovering from session failures.

#![allow(dead_code)]

use crate::utils::tmux::{kill_session, list_sessions, SessionInfo};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

/// Information about a session health status
#[derive(Debug, Clone)]
pub struct SessionHealth {
    /// Session name
    pub name: String,
    /// Health status
    pub status: SessionStatus,
    /// Number of windows
    pub windows: usize,
    /// Number of active panes
    pub active_panes: usize,
    /// Session age in seconds
    pub age_seconds: u64,
    /// Last activity timestamp
    pub last_activity: Option<SystemTime>,
    /// Issues found
    pub issues: Vec<SessionIssue>,
}

/// Session health status
#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    /// Session is healthy and responsive
    Healthy,
    /// Session exists but has issues
    Degraded,
    /// Session is unresponsive or dead
    Dead,
    /// Session is orphaned (no matching workspace)
    Orphaned,
}

/// Types of session issues
#[derive(Debug, Clone)]
pub enum SessionIssue {
    /// No active panes
    NoActivePanes,
    /// Session has been idle too long
    IdleTooLong(Duration),
    /// Session has zombie processes
    ZombieProcesses(usize),
    /// Workspace directory missing
    WorkspaceMissing(String),
    /// Git repository issues
    GitIssues(String),
    /// Tmux socket issues
    TmuxSocketIssues(String),
    /// Memory usage too high
    HighMemoryUsage(u64),
}

/// Recovery options for sessions
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// No action needed
    None,
    /// Restart the session
    Restart,
    /// Clean up and kill session
    Cleanup,
    /// Attempt to repair session
    Repair,
    /// Prompt user for action
    Prompt,
}

/// Session recovery configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum idle time before considering session stale
    pub max_idle_time: Duration,
    /// Maximum session age before cleanup
    pub max_session_age: Duration,
    /// Enable automatic cleanup
    pub auto_cleanup: bool,
    /// Directories to monitor for cleanup
    pub cleanup_dirs: Vec<String>,
    /// Processes to monitor
    pub monitored_processes: Vec<String>,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_idle_time: Duration::from_secs(3600),    // 1 hour
            max_session_age: Duration::from_secs(86400), // 24 hours
            auto_cleanup: false,
            cleanup_dirs: vec!["/tmp".to_string(), "/var/tmp".to_string()],
            monitored_processes: vec!["tmux".to_string(), "git".to_string(), "sprite".to_string()],
        }
    }
}

/// Analyze the health of all tmux sessions
pub fn analyze_session_health(config: &RecoveryConfig) -> Result<Vec<SessionHealth>> {
    let sessions = match list_sessions() {
        Ok(sessions) => sessions,
        Err(_) => {
            // Handle case where tmux server is not running or no sessions exist
            return Ok(Vec::new());
        }
    };

    let mut health_reports = Vec::new();

    for session in sessions {
        let health = check_session_health(&session, config)?;
        health_reports.push(health);
    }

    Ok(health_reports)
}

/// Check the health of a specific session
pub fn check_session_health(
    session: &SessionInfo,
    config: &RecoveryConfig,
) -> Result<SessionHealth> {
    let mut issues = Vec::new();
    let mut active_panes = 0;

    // Check if session actually responds
    let is_responsive = check_session_responsiveness(&session.name)?;

    // Check for active panes
    match crate::utils::tmux::get_session_panes(&session.name) {
        Ok(panes) => {
            active_panes = panes.len();
            if active_panes == 0 {
                issues.push(SessionIssue::NoActivePanes);
            }
        }
        Err(_) => {
            issues.push(SessionIssue::TmuxSocketIssues(
                "Failed to list session panes".to_string(),
            ));
        }
    }

    // Check session age
    let age_seconds = parse_session_age(&session.created)?;
    if age_seconds > config.max_session_age.as_secs() {
        issues.push(SessionIssue::IdleTooLong(Duration::from_secs(age_seconds)));
    }

    // Check workspace integrity
    if let Some(workspace_issue) = check_workspace_integrity(&session.name)? {
        issues.push(workspace_issue);
    }

    // Check for zombie processes
    if let Ok(zombie_count) = count_zombie_processes(&session.name) {
        if zombie_count > 0 {
            issues.push(SessionIssue::ZombieProcesses(zombie_count));
        }
    }

    // Determine overall status
    let status = if !is_responsive
        || issues
            .iter()
            .any(|i| matches!(i, SessionIssue::TmuxSocketIssues(_)))
    {
        SessionStatus::Dead
    } else if issues
        .iter()
        .any(|i| matches!(i, SessionIssue::WorkspaceMissing(_)))
    {
        SessionStatus::Orphaned
    } else if !issues.is_empty() {
        SessionStatus::Degraded
    } else {
        SessionStatus::Healthy
    };

    Ok(SessionHealth {
        name: session.name.clone(),
        status,
        windows: session.windows,
        active_panes,
        age_seconds,
        last_activity: None, // Would need additional tmux commands to get this
        issues,
    })
}

/// Check if a session is responsive
fn check_session_responsiveness(session_name: &str) -> Result<bool> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "-t", session_name, "#S"])
        .output()
        .context("Failed to check session responsiveness")?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

/// Parse session creation time to seconds
fn parse_session_age(_created_str: &str) -> Result<u64> {
    // For now, return a reasonable default. In a real implementation,
    // we would parse the tmux timestamp format
    Ok(3600) // 1 hour default
}

/// Check workspace integrity for a session
fn check_workspace_integrity(session_name: &str) -> Result<Option<SessionIssue>> {
    // Check if we're in a valid sprite workspace (current directory should have agents.yaml)
    let agents_config = Path::new("agents/agents.yaml");

    if agents_config.exists() {
        // Check if it's a valid git repository
        if Path::new(".git").exists() {
            return Ok(None);
        } else {
            return Ok(Some(SessionIssue::GitIssues(
                "Current directory is not a git repository".to_string(),
            )));
        }
    }

    // If we can't find the agents configuration, consider it orphaned
    Ok(Some(SessionIssue::WorkspaceMissing(format!(
        "No workspace found for session {}",
        session_name
    ))))
}

/// Count zombie processes associated with a session
fn count_zombie_processes(_session_name: &str) -> Result<usize> {
    // This is a placeholder implementation
    // In practice, we would parse /proc or use ps to find zombies
    Ok(0)
}

/// Get recommended recovery action for a session
pub fn get_recovery_action(health: &SessionHealth, config: &RecoveryConfig) -> RecoveryAction {
    match health.status {
        SessionStatus::Healthy => RecoveryAction::None,
        SessionStatus::Dead => {
            if config.auto_cleanup {
                RecoveryAction::Cleanup
            } else {
                RecoveryAction::Prompt
            }
        }
        SessionStatus::Orphaned => {
            if config.auto_cleanup {
                RecoveryAction::Cleanup
            } else {
                RecoveryAction::Prompt
            }
        }
        SessionStatus::Degraded => {
            if health
                .issues
                .iter()
                .any(|i| matches!(i, SessionIssue::NoActivePanes))
            {
                RecoveryAction::Repair
            } else if health.age_seconds > config.max_session_age.as_secs() {
                if config.auto_cleanup {
                    RecoveryAction::Cleanup
                } else {
                    RecoveryAction::Prompt
                }
            } else {
                RecoveryAction::Repair
            }
        }
    }
}

/// Perform recovery action on a session
pub fn perform_recovery_action(session_name: &str, action: RecoveryAction) -> Result<()> {
    match action {
        RecoveryAction::None => Ok(()),
        RecoveryAction::Restart => {
            println!("🔄 Restarting session: {}", session_name);
            // Kill existing session and recreate it
            kill_session(session_name)?;
            // In practice, we'd need to recreate the session with proper layout
            Ok(())
        }
        RecoveryAction::Cleanup => {
            println!("🧹 Cleaning up session: {}", session_name);
            cleanup_session_resources(session_name)?;
            kill_session(session_name)?;
            Ok(())
        }
        RecoveryAction::Repair => {
            println!("🔧 Repairing session: {}", session_name);
            repair_session(session_name)?;
            Ok(())
        }
        RecoveryAction::Prompt => {
            println!("❓ Action required for session: {}", session_name);
            // In practice, this would prompt the user
            Ok(())
        }
    }
}

/// Clean up resources associated with a session
pub fn cleanup_session_resources(session_name: &str) -> Result<()> {
    // Clean up temporary files
    let temp_patterns = vec![
        format!("/tmp/sprite-{}-*", session_name),
        format!("/var/tmp/sprite-{}-*", session_name),
        format!("/tmp/tmux-{}*", session_name),
    ];

    for pattern in temp_patterns {
        if let Ok(output) = Command::new("find")
            .args(["/tmp", "-name", &pattern, "-delete"])
            .output()
        {
            if !output.status.success() {
                eprintln!(
                    "Warning: Failed to cleanup pattern {}: {}",
                    pattern,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    // Clean up any sprite-specific state files
    let state_dir = Path::new(".sprites");
    if state_dir.exists() {
        let session_state = state_dir.join(format!("{}.json", session_name));
        if session_state.exists() {
            fs::remove_file(&session_state).with_context(|| {
                format!("Failed to remove session state file: {:?}", session_state)
            })?;
        }
    }

    Ok(())
}

/// Attempt to repair a degraded session
fn repair_session(session_name: &str) -> Result<()> {
    // Check if session has no active panes
    match crate::utils::tmux::get_session_panes(session_name) {
        Ok(panes) if panes.is_empty() => {
            // Try to create a new window
            crate::utils::tmux::create_window(session_name, "recovery")?;
            println!("✅ Created recovery window for session: {}", session_name);
        }
        Ok(_) => {
            println!("✅ Session {} appears to be functioning", session_name);
        }
        Err(e) => {
            eprintln!("❌ Failed to repair session {}: {}", session_name, e);
            return Err(e);
        }
    }

    Ok(())
}

/// Clean up old sessions based on configuration
pub fn cleanup_old_sessions(config: &RecoveryConfig) -> Result<Vec<String>> {
    let health_reports = analyze_session_health(config)?;
    let mut cleaned_sessions = Vec::new();

    for health in health_reports {
        let action = get_recovery_action(&health, config);

        if matches!(action, RecoveryAction::Cleanup) {
            if let Err(e) = perform_recovery_action(&health.name, action) {
                eprintln!("Failed to cleanup session {}: {}", health.name, e);
            } else {
                cleaned_sessions.push(health.name);
            }
        }
    }

    Ok(cleaned_sessions)
}

/// Clean up temporary files and directories
pub fn cleanup_temp_files(config: &RecoveryConfig) -> Result<()> {
    for temp_dir in &config.cleanup_dirs {
        if Path::new(temp_dir).exists() {
            cleanup_directory(temp_dir, "sprite-*")?;
            cleanup_directory(temp_dir, "tmux-*")?;
        }
    }
    Ok(())
}

/// Clean up files matching a pattern in a directory
fn cleanup_directory(dir: &str, pattern: &str) -> Result<()> {
    let output = Command::new("find")
        .args([dir, "-name", pattern, "-mtime", "+1", "-delete"])
        .output()
        .context("Failed to cleanup temporary files")?;

    if !output.status.success() {
        eprintln!(
            "Warning: Failed to cleanup {} in {}: {}",
            pattern,
            dir,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Generate a session health report
pub fn generate_health_report(health_reports: &[SessionHealth]) -> String {
    let mut report = String::new();
    report.push_str("🏥 Session Health Report\n");
    report.push_str("========================\n\n");

    let mut healthy_count = 0;
    let mut degraded_count = 0;
    let mut dead_count = 0;
    let mut orphaned_count = 0;

    for health in health_reports {
        match health.status {
            SessionStatus::Healthy => healthy_count += 1,
            SessionStatus::Degraded => degraded_count += 1,
            SessionStatus::Dead => dead_count += 1,
            SessionStatus::Orphaned => orphaned_count += 1,
        }

        let status_icon = match health.status {
            SessionStatus::Healthy => "🟢",
            SessionStatus::Degraded => "🟡",
            SessionStatus::Dead => "🔴",
            SessionStatus::Orphaned => "⚪",
        };

        report.push_str(&format!(
            "{} {} ({} windows, {} panes, {}s old)\n",
            status_icon, health.name, health.windows, health.active_panes, health.age_seconds
        ));

        if !health.issues.is_empty() {
            for issue in &health.issues {
                let issue_desc = match issue {
                    SessionIssue::NoActivePanes => "No active panes".to_string(),
                    SessionIssue::IdleTooLong(duration) => format!("Idle for {:?}", duration),
                    SessionIssue::ZombieProcesses(count) => format!("{} zombie processes", count),
                    SessionIssue::WorkspaceMissing(path) => format!("Workspace missing: {}", path),
                    SessionIssue::GitIssues(msg) => format!("Git issues: {}", msg),
                    SessionIssue::TmuxSocketIssues(msg) => format!("Tmux socket issues: {}", msg),
                    SessionIssue::HighMemoryUsage(bytes) => {
                        format!("High memory usage: {} MB", bytes / 1024 / 1024)
                    }
                };
                report.push_str(&format!("    ⚠️  {}\n", issue_desc));
            }
        }
        report.push('\n');
    }

    report.push_str("--- Summary ---\n");
    report.push_str(&format!("🟢 Healthy: {}\n", healthy_count));
    report.push_str(&format!("🟡 Degraded: {}\n", degraded_count));
    report.push_str(&format!("🔴 Dead: {}\n", dead_count));
    report.push_str(&format!("⚪ Orphaned: {}\n", orphaned_count));

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_config_default() {
        let config = RecoveryConfig::default();
        assert_eq!(config.max_idle_time, Duration::from_secs(3600));
        assert_eq!(config.max_session_age, Duration::from_secs(86400));
        assert!(!config.auto_cleanup);
    }

    #[test]
    fn test_session_age_parsing() {
        let age = parse_session_age("created Wed Oct 18 10:30:00 2023").unwrap();
        assert_eq!(age, 3600); // Default fallback value
    }

    #[test]
    fn test_health_report_generation() {
        let health = SessionHealth {
            name: "test-session".to_string(),
            status: SessionStatus::Healthy,
            windows: 3,
            active_panes: 3,
            age_seconds: 1800,
            last_activity: None,
            issues: vec![],
        };

        let report = generate_health_report(&[health]);
        assert!(report.contains("Session Health Report"));
        assert!(report.contains("test-session"));
        assert!(report.contains("🟢"));
    }

    #[test]
    fn test_recovery_action_selection() {
        let config = RecoveryConfig::default();

        let healthy_session = SessionHealth {
            name: "healthy".to_string(),
            status: SessionStatus::Healthy,
            windows: 1,
            active_panes: 1,
            age_seconds: 100,
            last_activity: None,
            issues: vec![],
        };

        let action = get_recovery_action(&healthy_session, &config);
        assert!(matches!(action, RecoveryAction::None));
    }
}
