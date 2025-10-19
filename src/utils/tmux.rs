//! Tmux utility functions for the Sprite multi-agent workflow toolkit.
//!
//! This module provides functions for managing tmux sessions, panes,
//! and layouts needed for multi-agent supervision.

use crate::error::SpriteError;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Command;

/// Create a new tmux session.
pub fn create_session(name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(&["new-session", "-d", "-s", name])
        .output()
        .with_context(|| "Failed to create tmux session. Is tmux installed?")?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!("Failed to create tmux session '{}'", name),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Check if a tmux session exists.
pub fn session_exists(name: &str) -> Result<bool> {
    let output = Command::new("tmux")
        .args(&["has-session", "-t", name])
        .output()
        .with_context(|| "Failed to check tmux session existence")?;

    Ok(output.status.success())
}

/// Kill a tmux session.
pub fn kill_session(name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(&["kill-session", "-t", name])
        .output()
        .with_context(|| format!("Failed to kill tmux session '{}'", name))?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!("Failed to kill tmux session '{}'", name),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Attach to a tmux session.
pub fn attach_session(name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(&["attach", "-t", name])
        .output()
        .with_context(|| format!("Failed to attach to tmux session '{}'", name))?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!("Failed to attach to tmux session '{}'", name),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// List all tmux sessions.
pub fn list_sessions() -> Result<Vec<SessionInfo>> {
    let output = Command::new("tmux")
        .args(&["list-sessions"])
        .output()
        .with_context(|| "Failed to list tmux sessions")?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            "Failed to list tmux sessions",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let sessions_str = String::from_utf8_lossy(&output.stdout);
    parse_sessions_list(&sessions_str)
}

/// Information about a tmux session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Session name
    pub name: String,
    /// Number of windows
    pub windows: usize,
    /// Number of panes
    pub panes: usize,
    /// Session creation time
    pub created: String,
    /// Session attachments
    pub attached: bool,
}

/// Parse the output of `tmux list-sessions`.
fn parse_sessions_list(output: &str) -> Result<Vec<SessionInfo>> {
    let mut sessions = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Parse tmux list-sessions format
        // Format: <session_name>: <windows> windows (created <date>) [<attached>]
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 2 {
            continue;
        }

        let name = parts[0].to_string();
        let info_part = parts[1];

        let windows = info_part
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let created = info_part
            .find("(created ")
            .and_then(|i| {
                let start = i + 9;
                info_part[start..]
                    .find(')')
                    .map(|j| info_part[start..j].to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let attached = info_part.contains("[attached]");

        // TODO: Count panes more accurately by examining each window
        let panes = windows;

        sessions.push(SessionInfo {
            name,
            windows,
            panes,
            created,
            attached,
        });
    }

    Ok(sessions)
}

/// Create a new window in a tmux session.
pub fn create_window(session: &str, window_name: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(&["new-window", "-t", session, "-n", window_name])
        .output()
        .with_context(|| {
            format!(
                "Failed to create window '{}' in session '{}'",
                window_name, session
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to create window '{}' in session '{}'",
                window_name, session
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    // The window index is in stdout
    let window_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(window_id)
}

/// Split a window vertically.
pub fn split_window_vertical(session: &str, target: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(&["split-window", "-t", session, "-v", "-P", target])
        .output()
        .with_context(|| format!("Failed to split window vertically in session '{}'", session))?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!("Failed to split window vertically in session '{}'", session),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Split a window horizontally.
pub fn split_window_horizontal(session: &str, target: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(&["split-window", "-t", session, "-h", "-P", target])
        .output()
        .with_context(|| {
            format!(
                "Failed to split window horizontally in session '{}'",
                session
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to split window horizontally in session '{}'",
                session
            ),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
        .into());
    }

    Ok(())
}

/// Send a command to a specific pane.
pub fn send_keys(session: &str, target: &str, command: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(&["send-keys", "-t", session, target, command, "C-m"])
        .output()
        .with_context(|| {
            format!(
                "Failed to send keys to pane '{}' in session '{}'",
                target, session
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to send keys to pane '{}' in session '{}'",
                target, session
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Capture output from a pane.
pub fn capture_pane(session: &str, target: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(&["capture-pane", "-p", "-t", session, target])
        .output()
        .with_context(|| {
            format!(
                "Failed to capture pane '{}' in session '{}'",
                target, session
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to capture pane '{}' in session '{}'",
                target, session
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Set the layout for a window.
pub fn select_layout(session: &str, target: &str, layout: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(&["select-layout", "-t", session, "-P", target, layout])
        .output()
        .with_context(|| {
            format!(
                "Failed to select layout '{}' for pane '{}' in session '{}'",
                layout, target, session
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to select layout '{}' for pane '{}' in session '{}'",
                layout, target, session
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Get the layout for a window.
pub fn get_layout(session: &str, target: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(&[
            "display-message",
            "-p",
            "-t",
            session,
            "-F '#{window_layout}'",
            target,
        ])
        .output()
        .with_context(|| {
            format!(
                "Failed to get layout for pane '{}' in session '{}'",
                target, session
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to get layout for pane '{}' in session '{}'",
                target, session
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Check if tmux is installed and available.
pub fn is_tmux_available() -> Result<bool> {
    let output = Command::new("tmux")
        .arg("-V")
        .output()
        .with_context(|| "Failed to check tmux availability")?;

    Ok(output.status.success())
}

/// Get tmux version information.
pub fn get_tmux_version() -> Result<String> {
    let output = Command::new("tmux")
        .arg("-V")
        .output()
        .with_context(|| "Failed to get tmux version")?;

    if !output.status.success() {
        return Err(SpriteError::tmux("tmux not found or not executable").into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Set environment variables for a session.
pub fn set_environment(session: &str, env_vars: &HashMap<String, String>) -> Result<()> {
    for (key, value) in env_vars {
        let output = Command::new("tmux")
            .args(&[
                "set-environment",
                "-t",
                session,
                &format!("{}={}", key, value),
            ])
            .output()
            .with_context(|| {
                format!(
                    "Failed to set environment variable '{}' for session '{}'",
                    key, session
                )
            })?;

        if !output.status.success() {
            return Err(SpriteError::tmux_with_source(
                format!(
                    "Failed to set environment variable '{}' for session '{}'",
                    key, session
                ),
                String::from_utf8_lossy(&output.stderr),
            )
            .into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sessions_list() {
        let input = r#"
sprite-session: 3 windows (created Wed Oct 18 10:30:00 2023) [attached]
another-session: 1 windows (created Wed Oct 18 09:15:00 2023)
"#;

        let sessions = parse_sessions_list(input).unwrap();
        assert_eq!(sessions.len(), 2);

        assert_eq!(sessions[0].name, "sprite-session");
        assert_eq!(sessions[0].windows, 3);
        assert!(sessions[0].attached);

        assert_eq!(sessions[1].name, "another-session");
        assert_eq!(sessions[1].windows, 1);
        assert!(!sessions[1].attached);
    }

    #[test]
    fn test_is_tmux_available() {
        // This test depends on tmux being installed
        let _result = is_tmux_available();
        // We don't assert the result since it depends on the test environment
    }
}
