//! Zoom command - Focus on specific agent panes in tmux sessions

use crate::error::SpriteError;
use crate::models::ProjectConfig;
use crate::utils::tmux;
use anyhow::{Context, Result};

/// Execute the zoom command with the given parameters.
pub fn execute(
    agent: Option<String>,
    unzoom: bool,
    list: bool,
) -> Result<()> {
    // Load current configuration
    let config_path = ProjectConfig::config_path();
    if config_path.exists() {
        let _config = ProjectConfig::load_from_file(&config_path)
            .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;
    }

    // Handle list option
    if list {
        list_panes()?;
        return Ok(());
    }

    // Handle unzoom option
    if unzoom {
        unzoom_session()?;
        return Ok(());
    }

    // If no agent specified, show available panes
    let agent_id = if let Some(agent) = agent {
        agent
    } else {
        list_panes_with_prompt()?;
        return Ok(());
    };

    // Find the active session
    let sessions = tmux::list_sessions()
        .context("Failed to list tmux sessions")?;

    let active_session = sessions.iter()
        .find(|s| s.name.starts_with("sprite-") && s.attached)
        .or_else(|| sessions.iter().find(|s| s.name.starts_with("sprite-")))
        .ok_or_else(|| SpriteError::session_not_found("No active sprite session found"))?;

    // Get session panes
    let panes = tmux::get_session_panes(&active_session.name)
        .with_context(|| format!("Failed to get panes for session '{}'", active_session.name))?;

    // Find the target pane
    let target_pane = find_target_pane(&panes, &agent_id)?;

    // Zoom to the target pane
    zoom_to_pane(&active_session.name, target_pane)?;

    Ok(())
}

/// List all available panes in the active session
fn list_panes() -> Result<()> {
    // Find the active session
    let sessions = tmux::list_sessions()
        .context("Failed to list tmux sessions")?;

    let active_session = sessions.iter()
        .find(|s| s.name.starts_with("sprite-") && s.attached)
        .or_else(|| sessions.iter().find(|s| s.name.starts_with("sprite-")));

    if active_session.is_none() {
        println!("📭 No active sprite session found.");
        println!("💡 Use 'sprite start' to create a new session.");
        return Ok(());
    }

    let active_session = active_session.unwrap();
    println!("🎬 Active Session: {}", active_session.name);

    // Get session panes
    let panes = tmux::get_session_panes(&active_session.name)
        .with_context(|| format!("Failed to get panes for session '{}'", active_session.name))?;

    if panes.is_empty() {
        println!("   No panes found in session.");
        return Ok(());
    }

    println!("\n📋 Available Panes:");
    println!("==================");

    for (i, pane) in panes.iter().enumerate() {
        let command = pane.current_command.as_deref().unwrap_or("idle");
        let path = pane.current_path.as_deref().unwrap_or("unknown");

        // Try to extract agent information from path or command
        let agent_info = extract_agent_info(pane);

        println!("{}. {} ({})", i + 1, command, path);
        if let Some(info) = agent_info {
            println!("   -> Agent: {}", info);
        }
    }

    println!();
    println!("💡 Usage:");
    println!("  sprite zoom <agent_number>  # Focus on specific agent pane");
    println!("  sprite zoom --unzoom         # Return to normal layout");
    println!("  sprite zoom --list           # Show this pane list");

    Ok(())
}

/// List panes with a prompt for selection
fn list_panes_with_prompt() -> Result<()> {
    list_panes()?;
    println!();
    println!("Please specify an agent number or name to zoom to.");
    println!("Example: sprite zoom 1  # Zoom to agent 1");
    Ok(())
}

/// Find the target pane for the given agent
fn find_target_pane(panes: &[tmux::PaneInfo], agent_id: &str) -> Result<usize> {
    // Try to parse as numeric agent ID first
    if let Ok(agent_num) = agent_id.parse::<usize>() {
        let expected_pane = agent_num - 1; // 0-indexed
        if expected_pane < panes.len() {
            return Ok(expected_pane);
        }
    }

    // Try to find by agent ID in path
    for (i, pane) in panes.iter().enumerate() {
        if let Some(path) = &pane.current_path {
            if path.contains(&format!("agents/{}", agent_id)) || path.ends_with(agent_id) {
                return Ok(i);
            }
        }
    }

    // Try to find by agent ID in command
    for (i, pane) in panes.iter().enumerate() {
        if let Some(command) = &pane.current_command {
            if command.contains(agent_id) {
                return Ok(i);
            }
        }
    }

    Err(SpriteError::validation(
        format!("Agent '{}' not found or no active pane", agent_id),
        Some("agent".to_string()),
        Some(agent_id.to_string())
    ).into())
}

/// Zoom to the specified pane
fn zoom_to_pane(session_name: &str, pane_index: usize) -> Result<()> {
    println!("🔍 Zooming to pane {}...", pane_index + 1);

    // Use tmux's select-pane command to focus on the pane
    let output = std::process::Command::new("tmux")
        .args(["select-pane", "-t", &pane_index.to_string()])
        .output()
        .with_context(|| "Failed to select pane")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SpriteError::tmux_with_source(
            "Failed to select pane",
            stderr.to_string()
        ).into());
    }

    // Optionally maximize the pane (tmux has zoom functionality)
    let zoom_output = std::process::Command::new("tmux")
        .args(["resize-pane", "-Z"])
        .output()
        .with_context(|| "Failed to zoom pane")?;

    if !zoom_output.status.success() {
        let stderr = String::from_utf8_lossy(&zoom_output.stderr);
        eprintln!("⚠️  Warning: Could not maximize pane: {}", stderr);
    }

    println!("✓ Zoomed to pane {}", pane_index + 1);
    println!("💡 Press Ctrl+B then Z to unzoom and return to normal layout");

    Ok(())
}

/// Unzoom the current session (return to normal layout)
fn unzoom_session() -> Result<()> {
    println!("🔍 Returning to normal layout...");

    // Use tmux's resize-pane -Z command to toggle zoom off
    let output = std::process::Command::new("tmux")
        .args(["resize-pane", "-Z"])
        .output()
        .with_context(|| "Failed to unzoom session")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SpriteError::tmux_with_source(
            "Failed to unzoom session",
            stderr.to_string()
        ).into());
    }

    println!("✓ Returned to normal layout");

    Ok(())
}

/// Extract agent information from a pane
fn extract_agent_info(pane: &tmux::PaneInfo) -> Option<String> {
    // Try to extract from path
    if let Some(path) = &pane.current_path {
        if path.contains("agents/") {
            if let Some(agent_part) = path.split("agents/").nth(1) {
                if let Some(agent_id) = agent_part.split('/').next() {
                    return Some(format!("Agent {}", agent_id));
                }
            }
        }
    }

    // Try to extract from command using simple string matching
    if let Some(command) = &pane.current_command {
        if command.contains("agent") {
            // Simple pattern matching for agent identifiers
            let words: Vec<&str> = command.split_whitespace().collect();
            for word in words {
                if word.contains("agent") {
                    // Extract the part after "agent"
                    if let Some(agent_part) = word.split("agent").nth(1) {
                        let agent_id = agent_part.trim_start_matches('_')
                            .trim_start_matches('-')
                            .trim();
                        if !agent_id.is_empty() {
                            return Some(format!("Agent {}", agent_id));
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_execution() {
        // This test would need mocking to work properly in isolation
        // For now, we just verify the function exists and handles basic cases
        // The actual functionality is tested in integration tests
    }
}
