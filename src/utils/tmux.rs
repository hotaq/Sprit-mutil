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
        .args(["new-session", "-d", "-s", name])
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
        .args(["has-session", "-t", name])
        .output()
        .with_context(|| "Failed to check tmux session existence")?;

    Ok(output.status.success())
}

/// Kill a tmux session.
#[allow(dead_code)]
pub fn kill_session(name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["kill-session", "-t", name])
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
#[allow(dead_code)]
pub fn attach_session(name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["attach", "-t", name])
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
#[allow(dead_code)]
pub fn list_sessions() -> Result<Vec<SessionInfo>> {
    let output = Command::new("tmux")
        .args(["list-sessions"])
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
    #[allow(dead_code)]
    pub name: String,
    /// Number of windows
    #[allow(dead_code)]
    pub windows: usize,
    /// Number of panes
    #[allow(dead_code)]
    pub panes: usize,
    /// Session creation time
    #[allow(dead_code)]
    pub created: String,
    /// Session attachments
    #[allow(dead_code)]
    pub attached: bool,
}

/// Parse the output of `tmux list-sessions`.
#[allow(dead_code)]
fn parse_sessions_list(output: &str) -> Result<Vec<SessionInfo>> {
    let mut sessions = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Parse tmux list-sessions format
        // Format: <session_name>: <windows> windows (created <date>) [<attached>]
        if let Some((name, info_part)) = line.split_once(':') {
            let name = name.to_string();

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
    }

    Ok(sessions)
}

/// Create a new window in a tmux session.
pub fn create_window(session: &str, window_name: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(["new-window", "-t", session, "-n", window_name])
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
#[allow(dead_code)]
pub fn split_window_vertical(session: &str, target: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["split-window", "-t", session, "-v", "-P", target])
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
#[allow(dead_code)]
pub fn split_window_horizontal(session: &str, target: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["split-window", "-t", session, "-h", "-P", target])
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
#[allow(dead_code)]
pub fn send_keys(session: &str, target: &str, command: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["send-keys", "-t", session, target, command, "C-m"])
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
#[allow(dead_code)]
pub fn capture_pane(session: &str, target: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(["capture-pane", "-p", "-t", session, target])
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
#[allow(dead_code)]
pub fn select_layout(session: &str, target: &str, layout: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(["select-layout", "-t", session, "-P", target, layout])
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
#[allow(dead_code)]
pub fn get_layout(session: &str, target: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args([
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
#[allow(dead_code)]
pub fn is_tmux_available() -> Result<bool> {
    let output = Command::new("tmux")
        .arg("-V")
        .output()
        .with_context(|| "Failed to check tmux availability")?;

    Ok(output.status.success())
}

/// Get tmux version information.
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn set_environment(session: &str, env_vars: &HashMap<String, String>) -> Result<()> {
    for (key, value) in env_vars {
        let output = Command::new("tmux")
            .args([
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

/// Execute a tmux profile script in a session.
pub fn execute_profile_script(session: &str, script_path: &std::path::Path) -> Result<()> {
    if !script_path.exists() {
        return Err(SpriteError::filesystem(
            format!("Profile script not found: {}", script_path.display()),
            script_path.display().to_string(),
        )
        .into());
    }

    println!("📜 Executing profile script: {}", script_path.display());

    // Clear the session first
    let output = Command::new("tmux")
        .args(["send-keys", "-t", session, "C-l", "C-c"])
        .output()
        .context("Failed to clear tmux session")?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            "Failed to clear tmux session",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    // Read and execute the profile script
    let script_content = std::fs::read_to_string(script_path)
        .with_context(|| format!("Failed to read profile script: {}", script_path.display()))?;

    // Split script into lines and execute each command
    for line in script_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Execute the command in the tmux session
        let output = Command::new("tmux")
            .args(["send-keys", "-t", session, line, "C-m"])
            .output()
            .with_context(|| format!("Failed to execute tmux command: {}", line))?;

        if !output.status.success() {
            eprintln!("⚠️  Warning: Failed to execute tmux command: {}", line);
            eprintln!("   Error: {}", String::from_utf8_lossy(&output.stderr));
        }

        // Small delay to allow commands to execute
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}

/// Get information about panes in a session.
pub fn get_session_panes(session: &str) -> Result<Vec<PaneInfo>> {
    let output = Command::new("tmux")
        .args(["list-panes", "-t", session])
        .output()
        .with_context(|| format!("Failed to list panes for session '{}'", session))?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!("Failed to list panes for session '{}'", session),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let panes_str = String::from_utf8_lossy(&output.stdout);
    parse_panes_list(&panes_str)
}

/// Information about a tmux pane.
#[derive(Debug, Clone)]
pub struct PaneInfo {
    /// Pane index
    pub index: usize,
    /// Current working directory
    #[allow(dead_code)]
    pub current_path: Option<String>,
    /// Running command
    #[allow(dead_code)]
    pub current_command: Option<String>,
    /// Pane size (lines x columns)
    #[allow(dead_code)]
    pub size: Option<(usize, usize)>,
    /// Pane layout
    #[allow(dead_code)]
    pub layout: Option<String>,
}

/// Parse the output of `tmux list-panes`.
fn parse_panes_list(output: &str) -> Result<Vec<PaneInfo>> {
    let mut panes = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Parse tmux list-panes format
        // Format: <index>: [<size>] [<command>] [<path>] [<layout>]
        let parts: Vec<&str> = line.split(':').collect();
        if parts.is_empty() {
            continue;
        }

        let index = parts[0].trim().parse::<usize>().unwrap_or(0);
        let info_part = if parts.len() > 1 { parts[1] } else { "" };

        // Extract current command (in brackets)
        let current_command = info_part
            .find('[')
            .and_then(|start| {
                info_part[start + 1..]
                    .find(']')
                    .map(|end| info_part[start + 1..start + end].to_string())
            })
            .filter(|cmd| !cmd.trim().is_empty());

        // Extract current path (usually at the end)
        let current_path = info_part
            .rfind(' ')
            .map(|start| info_part[start + 1..].trim().to_string())
            .filter(|path| !path.is_empty() && path.starts_with('/'));

        panes.push(PaneInfo {
            index,
            current_path,
            current_command,
            size: None,   // Would need additional tmux commands to get this
            layout: None, // Would need additional tmux commands to get this
        });
    }

    Ok(panes)
}

/// Switch to a specific pane.
#[allow(dead_code)]
pub fn select_pane(session: &str, pane_index: usize) -> Result<()> {
    let output = Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}.{}", session, pane_index)])
        .output()
        .with_context(|| {
            format!(
                "Failed to select pane {} in session '{}'",
                pane_index, session
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to select pane {} in session '{}'",
                pane_index, session
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(())
}

/// Get the current working directory of a pane.
#[allow(dead_code)]
pub fn get_pane_cwd(session: &str, pane_index: usize) -> Result<String> {
    let output = Command::new("tmux")
        .args([
            "display-message",
            "-p",
            "-t",
            &format!("{}.{}", session, pane_index),
            "#{pane_current_path}",
        ])
        .output()
        .with_context(|| {
            format!(
                "Failed to get current path for pane {} in session '{}'",
                pane_index, session
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to get current path for pane {} in session '{}'",
                pane_index, session
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Send a command to a specific pane with optional delay.
pub fn send_keys_with_delay(
    session: &str,
    target: &str,
    command: &str,
    delay_ms: u64,
) -> Result<()> {
    let output = Command::new("tmux")
        .args(["send-keys", "-t", session, target, command, "C-m"])
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

    // Add delay if requested
    if delay_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    }

    Ok(())
}

/// Create a new window with specific working directory.
#[allow(dead_code)]
pub fn create_window_with_path(
    session: &str,
    window_name: &str,
    working_dir: &str,
) -> Result<String> {
    let output = Command::new("tmux")
        .args([
            "new-window",
            "-t",
            session,
            "-n",
            window_name,
            "-c",
            working_dir,
        ])
        .output()
        .with_context(|| {
            format!(
                "Failed to create window '{}' in session '{}' with path '{}'",
                window_name, session, working_dir
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to create window '{}' in session '{}' with path '{}'",
                window_name, session, working_dir
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    // The window index is in stdout
    let window_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(window_id)
}

/// Rename an existing window.
#[allow(dead_code)]
pub fn rename_window(session: &str, window_index: &str, new_name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args([
            "rename-window",
            "-t",
            &format!("{}.{}", session, window_index),
            new_name,
        ])
        .output()
        .with_context(|| {
            format!(
                "Failed to rename window '{}.{}' to '{}'",
                session, window_index, new_name
            )
        })?;

    if !output.status.success() {
        return Err(SpriteError::tmux_with_source(
            format!(
                "Failed to rename window '{}.{}' to '{}'",
                session, window_index, new_name
            ),
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
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
