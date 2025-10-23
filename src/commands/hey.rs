//! Hey command - Send command to specific agent(s)

use crate::commands::config::SpriteConfig;
use crate::error::SpriteError;
use crate::utils::{accessibility::AccessibilityConfig, tmux};
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Execute the hey command with the given parameters.
/// 
/// Target formats:
/// - "all" = broadcast to all active agents
/// - "1,2,3" = send to specific agents
/// - "1" = send to single agent
pub fn execute(
    agents: &str,
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

    // Parse agent targets
    let target_agents = parse_agent_targets(agents)?;

    if target_agents.is_empty() {
        println!("ℹ️  No active agents found. Command cancelled.");
        return Ok(());
    }

    // Validate all target agents exist and are active
    for agent_id in &target_agents {
        let agent_config = config
            .get_agent(agent_id)
            .ok_or_else(|| SpriteError::agent_not_found(agent_id.to_string()))?;

        if agent_config.status.to_lowercase() != "active" {
            return Err(SpriteError::agent_not_active(agent_id.to_string()).into());
        }
    }

    // Find the active session
    let sessions = tmux::list_sessions().context("Failed to list tmux sessions")?;

    let active_session = sessions
        .iter()
        .find(|s| s.name.starts_with("sprite-") && s.attached)
        .or_else(|| sessions.iter().find(|s| s.name.starts_with("sprite-")))
        .ok_or_else(|| SpriteError::session_not_found("No active sprite session found"))?;

    // Get session panes to find the agent panes
    let panes = tmux::get_session_panes(&active_session.name)
        .with_context(|| format!("Failed to get panes for session '{}'", active_session.name))?;

    // Prepare the command with arguments
    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    // Set up environment variables if provided
    let env_map = parse_env_vars(env_vars)?;

    // Send command to all target agents
    for agent_id in &target_agents {
        let agent_config = config
            .get_agent(agent_id)
            .expect("Agent config should exist after validation");

        // Find the pane for this agent
        let agent_pane = find_agent_pane(&panes, agent_config, &active_session.name)?;

        // Change to working directory if specified
        if let Some(work_dir) = work_dir {
            let work_command = format!("cd {}", work_dir);
            std::process::Command::new("tmux")
                .args(["send-keys", "-t", &format!("{}.{}", active_session.name, agent_pane), &work_command])
                .output()
                .with_context(|| format!("Failed to change to working directory '{}' for agent '{}'", work_dir, agent_id))?;

            // Small delay then send Enter to execute
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            std::process::Command::new("tmux")
                .args(["send-keys", "-t", &format!("{}.{}", active_session.name, agent_pane), "C-m"])
                .output()
                .with_context(|| format!("Failed to send Enter key to agent '{}'", agent_id))?;
        }

        // Set environment variables
        for (key, value) in &env_map {
            let env_command = format!("export {}='{}'", key, value.replace('\'', "'\"'\"'"));
            std::process::Command::new("tmux")
                .args(["send-keys", "-t", &format!("{}.{}", active_session.name, agent_pane), &env_command])
                .output()
                .with_context(|| format!("Failed to set environment variable '{}' for agent '{}'", key, agent_id))?;
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Send the command to the agent pane using two-step approach
        std::process::Command::new("tmux")
            .args(["send-keys", "-t", &format!("{}.{}", active_session.name, agent_pane), &full_command])
            .output()
            .with_context(|| format!("Failed to send command text to agent '{}'", agent_id))?;

        // Small delay then send Enter to execute
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        std::process::Command::new("tmux")
            .args(["send-keys", "-t", &format!("{}.{}", active_session.name, agent_pane), "C-m"])
            .output()
            .with_context(|| format!("Failed to send Enter key to agent '{}'", agent_id))?;
    }

    let accessibility_config = AccessibilityConfig::default();
    crate::utils::accessibility::print_success(
        &format!("Command sent to agents {}: {}", target_agents.join(", "), full_command),
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

/// Parse agent targets from string 
/// "all" -> returns all active agent IDs
/// "1,2,3" -> returns ["1", "2", "3"]
/// "1" -> returns ["1"]
fn parse_agent_targets(agent_target: &str) -> Result<Vec<String>> {
    let config = SpriteConfig::load()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;
    
    let active_agents: Vec<String> = config
        .agents
        .iter()
        .filter(|agent| agent.status.to_lowercase() == "active")
        .map(|agent| agent.id.clone())
        .collect();

    if agent_target.trim().to_lowercase() == "all" {
        return Ok(active_agents);
    }

    // Parse comma-separated agent IDs
    let agent_ids: Vec<String> = agent_target
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Validate each agent ID
    for agent_id in &agent_ids {
        if !active_agents.contains(agent_id) {
            return Err(SpriteError::agent_not_found(format!("Agent '{}' not found or not active", agent_id)).into());
        }
    }

    Ok(agent_ids)
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

    // Fallback: try to find by agent description
    for pane in panes {
        let agent_desc = &agent_config.description;
        if let Some(current_cmd) = &pane.current_command {
            if current_cmd.contains(agent_desc) || current_cmd.contains(&format!("agent-{}", agent_config.id)) {
                return Ok(pane.pane_id.clone());
            }
        }
    }

    // As a last resort, just return the first available pane if matches agent count
    if panes.len() >= agent_config.id.parse::<usize>().unwrap_or(0) {
        return Ok(panes[agent_config.id.parse::<usize>().unwrap_or(0)].pane_id.clone());
    }

    Err(SpriteError::agent(
        format!("Agent '{}' not found in any tmux pane", agent_config.id),
        Some(agent_config.id.clone()),
    ).into())
}

/// Parse environment variables in KEY=VALUE format.
fn parse_env_vars(env_vars: &[String]) -> Result<HashMap<String, String>> {
    let mut env_map = HashMap::new();
    
    for env_var in env_vars {
        if let Some((key, value)) = env_var.split_once('=') {
            env_map.insert(key.trim().to_string(), value.trim().to_string());
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
    fn test_parse_agent_targets() {
        // Test single agent
        let result = parse_agent_targets("1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["1"]);
        
        // Test multiple agents
        let result = parse_agent_targets("1,2,3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["1", "2", "3"]);
        
        // Test all agents
        let result = parse_agent_targets("all");
        assert!(result.is_ok());
        // Can't assert exact result as it depends on available agents
        assert!(!result.unwrap().is_empty());
        
        // Test invalid agent
        let result = parse_agent_targets("99");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_env_vars() {
        let env_vars = vec![
            "DEBUG=true".to_string(),
            "PATH=/usr/bin".to_string(),
            "NODE_ENV=production".to_string(),
        ];

        let result = parse_env_vars(&env_vars);
        assert!(result.is_ok());
        let env_map = result.unwrap();
        
        assert_eq!(env_map.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(env_map.get("PATH"), Some(&"/usr/bin".to_string()));
        assert_eq!(env_map.get("NODE_ENV"), Some(&"production".to_string()));
    }

    #[test]
    fn test_find_agent_pane() {
        // This test would require mocking tmux panes, which is complex
        // For now, we'll just ensure the function compiles
        // In a real test environment, you might create a temporary tmux session
        // and test pane finding logic
    }
}
