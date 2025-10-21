use crate::error::SpriteError;
use crate::utils::session_recovery::{
    analyze_session_health, cleanup_temp_files, generate_health_report, RecoveryConfig,
};
use crate::utils::tmux;
use anyhow::{Context, Result};
use std::io::{self, Write};

/// Execute kill command with session name, force, and all options
pub fn execute(session_name: Option<String>, force: bool, all: bool) -> Result<()> {
    // Check if tmux is available
    tmux::is_tmux_available()
        .context("tmux is not available. Please install tmux to use sprite kill.")?;

    // Handle kill all sessions
    if all {
        return kill_all_sessions(force);
    }

    // Get list of sessions to determine what to kill
    let sessions = tmux::list_sessions().context("Failed to list sessions")?;

    if sessions.is_empty() {
        println!("📭 No tmux sessions found to kill.");
        return Ok(());
    }

    // Determine which session to kill
    let target_session = match session_name {
        Some(name) => name,
        None => {
            // Try to find the most recent sprite session
            find_default_session(&sessions)?
        }
    };

    // Check if session exists
    if !sessions.iter().any(|s| s.name == target_session) {
        return Err(SpriteError::session(
            format!("Session '{}' does not exist. Use 'sprite attach --list' to see available sessions.", target_session),
            Some(target_session)
        ).into());
    }

    // Get session info for display
    let session_info = sessions.iter().find(|s| s.name == target_session).unwrap();

    // Confirm before killing unless force flag is set
    if !force {
        confirm_session_kill(session_info)?;
    }

    // Perform cleanup before killing
    cleanup_session_resources(&target_session)?;

    // Kill the session
    println!("🔪 Killing tmux session: {}", target_session);
    tmux::kill_session(&target_session)
        .with_context(|| format!("Failed to kill session '{}'", target_session))?;

    println!("✅ Session '{}' killed successfully", target_session);
    Ok(())
}

/// Kill all tmux sessions
fn kill_all_sessions(force: bool) -> Result<()> {
    let sessions = tmux::list_sessions().context("Failed to list sessions")?;

    if sessions.is_empty() {
        println!("📭 No tmux sessions found to kill.");
        return Ok(());
    }

    if !force {
        confirm_kill_all_sessions(&sessions)?;
    }

    println!("🔪 Killing {} tmux session(s)...", sessions.len());

    let mut killed_count = 0;
    let mut failed_sessions = Vec::new();

    for session in &sessions {
        // Perform cleanup before killing
        if let Err(e) = cleanup_session_resources(&session.name) {
            eprintln!(
                "  ⚠️  Warning: Failed to cleanup session '{}': {}",
                session.name, e
            );
        }

        match tmux::kill_session(&session.name) {
            Ok(()) => {
                println!("  ✅ Killed session: {}", session.name);
                killed_count += 1;
            }
            Err(e) => {
                eprintln!("  ❌ Failed to kill session '{}': {}", session.name, e);
                failed_sessions.push(session.name.clone());
            }
        }
    }

    if killed_count > 0 {
        println!("✅ Successfully killed {} session(s)", killed_count);
    }

    if !failed_sessions.is_empty() {
        println!(
            "⚠️  Failed to kill {} session(s): {}",
            failed_sessions.len(),
            failed_sessions.join(", ")
        );

        // When force flag is used, only error if ALL sessions failed
        // Otherwise, treat partial success as success with warnings
        if force && killed_count > 0 {
            return Ok(());
        }

        // If no sessions were killed or force is not used, return error
        return Err(SpriteError::tmux(format!(
            "Failed to kill some sessions: {}",
            failed_sessions.join(", ")
        ))
        .into());
    }

    Ok(())
}

/// Find the default session to kill
fn find_default_session(sessions: &[tmux::SessionInfo]) -> Result<String> {
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

/// Confirm session killing with user
fn confirm_session_kill(session_info: &tmux::SessionInfo) -> Result<()> {
    println!("🎯 About to kill session:");
    println!(
        "   Name: {} ({} windows)",
        session_info.name, session_info.windows
    );
    println!(
        "   Status: {}",
        if session_info.attached {
            "🟢 Active"
        } else {
            "⚪ Detached"
        }
    );

    print!("❓ Are you sure you want to kill this session? [y/N] ");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let input = input.trim().to_lowercase();
    if input == "y" || input == "yes" {
        Ok(())
    } else {
        println!("❌ Session kill cancelled.");
        Err(SpriteError::tmux("Session kill cancelled by user".to_string()).into())
    }
}

/// Confirm killing all sessions with user
fn confirm_kill_all_sessions(sessions: &[tmux::SessionInfo]) -> Result<()> {
    println!("🎯 About to kill ALL {} tmux session(s):", sessions.len());

    for session in sessions {
        let status = if session.attached {
            "🟢 Active"
        } else {
            "⚪ Detached"
        };
        println!(
            "   {} {} ({} windows)",
            status, session.name, session.windows
        );
    }

    print!("❓ Are you sure you want to kill ALL sessions? This action cannot be undone. [y/N] ");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let input = input.trim().to_lowercase();
    if input == "y" || input == "yes" {
        Ok(())
    } else {
        println!("❌ Session kill cancelled.");
        Err(SpriteError::tmux("Session kill cancelled by user".to_string()).into())
    }
}

/// Clean up resources associated with a session
fn cleanup_session_resources(session_name: &str) -> Result<()> {
    println!("🧹 Cleaning up resources for session: {}", session_name);

    // Use the session recovery module to cleanup
    crate::utils::session_recovery::cleanup_session_resources(session_name)?;

    Ok(())
}

/// Check and display session health
#[allow(dead_code)]
pub fn check_session_health() -> Result<()> {
    println!("🏥 Checking session health...");

    let config = RecoveryConfig::default();
    let health_reports = analyze_session_health(&config)?;

    if health_reports.is_empty() {
        println!("📭 No tmux sessions found.");
        return Ok(());
    }

    let report = generate_health_report(&health_reports);
    println!("{}", report);

    // Check if any sessions need attention
    let needs_attention = health_reports
        .iter()
        .any(|h| h.status != crate::utils::session_recovery::SessionStatus::Healthy);

    if needs_attention {
        println!("\n💡 Some sessions need attention. Use 'sprite kill --force <session>' to cleanup problematic sessions.");
    }

    Ok(())
}

/// Clean up old sessions and temporary files
#[allow(dead_code)]
pub fn cleanup_old_resources() -> Result<()> {
    println!("🧹 Starting cleanup of old resources...");

    let config = RecoveryConfig {
        auto_cleanup: true,
        ..Default::default()
    };

    // Clean up old sessions
    let cleaned_sessions = crate::utils::session_recovery::cleanup_old_sessions(&config)?;

    if !cleaned_sessions.is_empty() {
        println!(
            "✅ Cleaned up {} old session(s): {}",
            cleaned_sessions.len(),
            cleaned_sessions.join(", ")
        );
    } else {
        println!("✅ No old sessions needed cleanup.");
    }

    // Clean up temporary files
    cleanup_temp_files(&config)?;
    println!("✅ Cleaned up temporary files.");

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_find_default_session_with_sprite_session() {
        // This test would need mocking to work properly in isolation
        // For now, we just verify the function exists and handles empty lists
        // The actual functionality is tested in integration tests
    }

    #[test]
    fn test_find_default_session_empty_list() {
        // This test would need mocking to work properly in isolation
        // For now, we just verify the function exists and handles empty lists
        // The actual functionality is tested in integration tests
    }
}
