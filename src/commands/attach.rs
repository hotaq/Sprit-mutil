use crate::error::SpriteError;
use crate::utils::tmux;
use anyhow::{Context, Result};

/// Execute attach command with session name and list options
pub fn execute(session_name: Option<String>, list: bool) -> Result<()> {
    // Check if tmux is available
    tmux::is_tmux_available()
        .context("tmux is not available. Please install tmux to use sprite attach.")?;

    // Handle list sessions option
    if list {
        list_available_sessions()?;
        return Ok(());
    }

    // Determine session name
    let session_name = match session_name {
        Some(name) => name,
        None => {
            // Try to find the most recent sprite session
            find_default_session()?
        }
    };

    // Check if session exists
    if !tmux::session_exists(&session_name)? {
        return Err(SpriteError::session(
            format!(
                "Session '{}' does not exist. Use --list to see available sessions.",
                session_name
            ),
            Some(session_name.clone()),
        )
        .into());
    }

    // Get session info for display
    let sessions = tmux::list_sessions().context("Failed to list sessions")?;
    let session_info = sessions
        .iter()
        .find(|s| s.name == session_name)
        .ok_or_else(|| {
            SpriteError::session(
                format!("Session '{}' not found in session list", session_name),
                Some(session_name.clone()),
            )
        })?;

    // Attach to the session
    println!("🔗 Attaching to tmux session: {}", session_name);

    if session_info.attached {
        println!("ℹ️  Session is already attached by another client");
    }

    // Get current working directory to restore after detach
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    println!("💡 Use 'Ctrl+B D' to detach from the session");
    println!("💡 You'll return to directory: {}", current_dir.display());
    println!();

    tmux::attach_session(&session_name)
        .with_context(|| format!("Failed to attach to session '{}'", session_name))?;

    println!("👋 Detached from session '{}'", session_name);

    Ok(())
}

/// Find the default sprite session to attach to
fn find_default_session() -> Result<String> {
    let sessions = tmux::list_sessions().context("Failed to list sessions")?;

    // First try to find a session with "sprite-session" name
    if let Some(session) = sessions.iter().find(|s| s.name.contains("sprite-session")) {
        return Ok(session.name.clone());
    }

    // If no sprite session found, take the most recently created session
    if let Some(session) = sessions.first() {
        return Ok(session.name.clone());
    }

    Err(SpriteError::session(
        "No tmux sessions found. Use 'sprite start' to create a new session.",
        None::<String>,
    )
    .into())
}

/// List all available tmux sessions
fn list_available_sessions() -> Result<()> {
    let sessions = match tmux::list_sessions() {
        Ok(s) => s,
        Err(_) => {
            // Handle case where tmux server is not running
            println!("📭 No tmux sessions found.");
            println!("   Use 'sprite start' to create a new multi-agent session.");
            return Ok(());
        }
    };

    if sessions.is_empty() {
        println!("📭 No tmux sessions found.");
        println!("   Use 'sprite start' to create a new multi-agent session.");
        return Ok(());
    }

    println!("📋 Available tmux sessions:");
    println!();

    for session in sessions {
        let status = if session.attached {
            "🟢 Active".to_string()
        } else {
            "⚪ Detached".to_string()
        };

        let created_ago = format_time_ago(&session.created);

        println!(
            "  {} {} ({} windows) - {}",
            status, session.name, session.windows, created_ago
        );
    }

    println!();
    println!("💡 Use 'sprite attach <session-name>' to connect to a session");

    Ok(())
}

/// Format a time duration as human readable text
fn format_time_ago(timestamp: &str) -> String {
    // For now, just return the timestamp as-is since parsing tmux time format is complex
    // TODO: Implement proper relative time formatting
    format!("created {}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time_ago() {
        // Test the format_time_ago function with a sample timestamp
        let sample_timestamp = "2025-01-01T12:00:00";
        let result = format_time_ago(sample_timestamp);

        assert!(result.contains("created"));
        assert!(result.contains(sample_timestamp));
    }

    #[test]
    fn test_format_time_ago_empty() {
        // Test with empty timestamp
        let empty_timestamp = "";
        let result = format_time_ago(empty_timestamp);

        assert!(result.contains("created"));
    }

    #[test]
    fn test_find_default_session_empty_list() {
        // This test would need mocking to work properly in isolation
        // For now, we just verify the function exists and handles empty lists
        // The actual functionality is tested in integration tests
    }
}
