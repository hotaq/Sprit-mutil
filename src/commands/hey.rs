//! Hey command - Send command to specific agent

use crate::commands::config::SpriteConfig;
use crate::error::SpriteError;
use crate::utils::{accessibility::AccessibilityConfig, tmux};
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Execute the hey command with the given parameters.
pub fn execute(
    agent: &str,
    command: &str,
    args: &[String],
    _timeout: u64,
    work_dir: Option<&str>,
    env_vars: &[String],
    interactive: bool,
) -> Result<()> {
    // Load current configuration using project root detection
    let config =
        SpriteConfig::load().map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    // Validate agent exists
    let agent_config = config
        .get_agent(agent)
        .ok_or_else(|| SpriteError::agent_not_found(agent.to_string()))?;

    // Check if agent is active
    if agent_config.status.to_lowercase() != "active" {
        return Err(SpriteError::agent_not_active(agent.to_string()).into());
    }

    // Find the active session
    let sessions = tmux::list_sessions().context("Failed to list tmux sessions")?;

    let active_session = sessions
        .iter()
        .find(|s| s.name.starts_with("sprite-") && s.attached)
        .or_else(|| sessions.iter().find(|s| s.name.starts_with("sprite-")))
        .ok_or_else(|| SpriteError::session_not_found("No active sprite session found"))?;

    // Get session panes to find the agent pane
    let panes = tmux::get_session_panes(&active_session.name)
        .with_context(|| format!("Failed to get panes for session '{}'", active_session.name))?;

    // Find the pane for this agent (based on agent workspace)
    let agent_pane = find_agent_pane(&panes, agent_config, &active_session.name)?;

    // Prepare the command with arguments
    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    // Set up environment variables if provided
    let env_map = parse_env_vars(env_vars)?;

    // Change to working directory if specified
    if let Some(work_dir) = work_dir {
        let work_command = format!("cd {}", work_dir);
        tmux::send_keys(&active_session.name, &agent_pane, &work_command)
            .with_context(|| format!("Failed to change to working directory '{}'", work_dir))?;

        // Small delay to allow directory change
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Set environment variables
    for (key, value) in &env_map {
        let env_command = format!("export {}='{}'", key, value.replace('\'', "'\"'\"'"));
        tmux::send_keys(&active_session.name, &agent_pane, &env_command)
            .with_context(|| format!("Failed to set environment variable '{}'", key))?;

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Send the command to the agent pane
    tmux::send_keys(&active_session.name, &agent_pane, &full_command)
        .with_context(|| format!("Failed to send command to agent '{}'", agent))?;

    let accessibility_config = AccessibilityConfig::default();
    crate::utils::accessibility::print_success(
        &format!("Command sent to agent {}: {}", agent, full_command),
        &accessibility_config,
    );

    // If interactive mode, attach to the session
    if interactive {
        crate::utils::accessibility::print_info(
            "Attaching to session for interactive mode...",
            &accessibility_config,
        );
        tmux::attach_session(&active_session.name)?;
    }

    Ok(())
}

/// Find the pane that corresponds to the given agent.
fn find_agent_pane(
    panes: &[tmux::PaneInfo],
    agent_config: &crate::commands::config::AgentConfig,
    session_name: &str,
) -> Result<String> {
    // Try to find the pane by matching the workspace path
    for pane in panes {
        if let Some(current_path) = &pane.current_path {
            if current_path.contains(&agent_config.worktree_path)
                || current_path.ends_with(&agent_config.id)
            {
                return Ok(pane.pane_id.clone());
            }
        }
    }

    // Fallback: try to find by agent ID in pane title or current command
    for pane in panes {
        if let Some(current_cmd) = &pane.current_command {
            if current_cmd.contains(&agent_config.id) {
                return Ok(pane.pane_id.clone());
            }
        }
    }

    // Another fallback: assume agent 1 is pane 0, agent 2 is pane 1, etc.
    if let Ok(agent_num) = agent_config.id.parse::<usize>() {
        let expected_index = agent_num.saturating_sub(1);
        if let Some(pane) = panes.iter().find(|p| p.index == expected_index) {
            return Ok(pane.pane_id.clone());
        }
    }

    Err(SpriteError::pane_not_found(format!(
        "Could not find tmux pane for agent '{}' in session '{}'",
        agent_config.id, session_name
    ))
    .into())
}

/// Parse environment variables from KEY=VALUE format.
fn parse_env_vars(env_vars: &[String]) -> Result<HashMap<String, String>> {
    let mut env_map = HashMap::new();

    for env_var in env_vars {
        if let Some((key, value)) = env_var.split_once('=') {
            if key.is_empty() {
                return Err(SpriteError::validation(
                    "Environment variable key cannot be empty".to_string(),
                    Some("env_var".to_string()),
                    None::<String>,
                )
                .into());
            }
            env_map.insert(key.to_string(), value.to_string());
        } else {
            return Err(SpriteError::validation(
                format!(
                    "Invalid environment variable format: '{}'. Expected KEY=VALUE",
                    env_var
                ),
                Some("env_var".to_string()),
                Some(env_var.clone() as String),
            )
            .into());
        }
    }

    Ok(env_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_vars() {
        let env_vars = vec![
            "DEBUG=true".to_string(),
            "PATH=/usr/bin".to_string(),
            "EMPTY=".to_string(),
        ];

        let result = parse_env_vars(&env_vars).unwrap();
        assert_eq!(result.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(result.get("PATH"), Some(&"/usr/bin".to_string()));
        assert_eq!(result.get("EMPTY"), Some(&"".to_string()));

        // Test invalid format
        let invalid_vars = vec!["INVALID_FORMAT".to_string()];
        assert!(parse_env_vars(&invalid_vars).is_err());

        // Test empty key
        let empty_key = vec!["=value".to_string()];
        assert!(parse_env_vars(&empty_key).is_err());
    }

    #[test]
    fn test_find_agent_pane() {
        // This test would require mocking tmux panes, which is complex
        // For now, we'll just ensure the function compiles
        // In a real test environment, you might create a temporary tmux session
        // and test pane finding logic
    }
}
