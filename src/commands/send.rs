//! Send command - Broadcast command to all agents

use crate::models::ProjectConfig;
use crate::error::SpriteError;
use crate::utils::{tmux, accessibility::AccessibilityConfig};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Execute the send command with the given parameters.
pub fn execute(
    command: &str,
    args: &[String],
    _timeout: u64,
    work_dir: Option<&str>,
    env_vars: &[String],
    sequential: bool,
) -> Result<()> {
    // Load current configuration
    let config_path = ProjectConfig::config_path();
    if !config_path.exists() {
        return Err(SpriteError::config_not_found(config_path.display().to_string()).into());
    }

    let config = ProjectConfig::load_from_file(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    // Get active agents
    let active_agents: Vec<_> = config.active_agents();
    if active_agents.is_empty() {
        println!("ℹ️  No active agents found. Command broadcast skipped.");
        return Ok(());
    }

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

    // Prepare the command with arguments
    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    // Set up environment variables if provided
    let env_map = parse_env_vars(env_vars)?;

    let accessibility_config = AccessibilityConfig::default();
    crate::utils::accessibility::print_info(
        &format!("Broadcasting command to {} agents: {}", active_agents.len(), full_command),
        &accessibility_config
    );

    // Execute the command on all agents
    if sequential {
        execute_sequential(&active_session.name, &panes, &active_agents, &full_command, work_dir, &env_map)?;
    } else {
        execute_parallel(&active_session.name, &panes, active_agents.into_iter().cloned().collect(), &full_command, work_dir, &env_map)?;
    }

    crate::utils::accessibility::print_success("Command broadcast completed", &accessibility_config);
    Ok(())
}

/// Execute command sequentially on all agents.
fn execute_sequential(
    session_name: &str,
    panes: &[tmux::PaneInfo],
    agents: &[&crate::models::Agent],
    command: &str,
    work_dir: Option<&str>,
    env_vars: &HashMap<String, String>,
) -> Result<()> {
    for agent in agents {
        println!("  📤 Sending to agent {}...", agent.id);

        match send_to_agent(session_name, panes, agent, command, work_dir, env_vars) {
            Ok(_) => println!("    ✓ Sent to agent {}", agent.id),
            Err(e) => {
                eprintln!("    ✗ Failed to send to agent {}: {}", agent.id, e);
                // Continue with other agents
            }
        }

        // Small delay between commands
        thread::sleep(Duration::from_millis(200));
    }

    Ok(())
}

/// Execute command in parallel on all agents.
fn execute_parallel(
    session_name: &str,
    panes: &[tmux::PaneInfo],
    agents: Vec<crate::models::Agent>,
    command: &str,
    work_dir: Option<&str>,
    env_vars: &HashMap<String, String>,
) -> Result<()> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for agent in agents {
        let session_name = session_name.to_string();
        let panes = panes.to_vec();
        let agent_id = agent.id.clone();
        let command = command.to_string();
        let work_dir = work_dir.map(|s| s.to_string());
        let env_vars = env_vars.clone();
        let results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            let start_time = Instant::now();
            let result = send_to_agent(&session_name, &panes, &agent, &command, work_dir.as_deref(), &env_vars);
            let duration = start_time.elapsed();

            let mut results = results.lock().unwrap();
            results.push((agent_id, result, duration));
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Display results
    let results = results.lock().unwrap();
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut total_duration = Duration::ZERO;

    for (agent_id, result, duration) in &*results {
        match result {
            Ok(_) => {
                println!("    ✓ Sent to agent {} (took {:?})", agent_id, duration);
                success_count += 1;
            }
            Err(e) => {
                eprintln!("    ✗ Failed to send to agent {}: {}", agent_id, e);
                failure_count += 1;
            }
        }
        total_duration += *duration;
    }

    if !results.is_empty() {
        let avg_duration = total_duration / results.len() as u32;
        println!("  📊 Parallel execution: {} successful, {} failed, avg time: {:?}",
                success_count, failure_count, avg_duration);
    }

    Ok(())
}

/// Send command to a specific agent.
fn send_to_agent(
    session_name: &str,
    panes: &[tmux::PaneInfo],
    agent: &crate::models::Agent,
    command: &str,
    work_dir: Option<&str>,
    env_vars: &HashMap<String, String>,
) -> Result<()> {
    // Find the pane for this agent
    let agent_pane = find_agent_pane(panes, agent, session_name)?;

    // Change to working directory if specified
    if let Some(work_dir) = work_dir {
        let work_command = format!("cd {}", work_dir);
        tmux::send_keys(session_name, &agent_pane.to_string(), &work_command)
            .with_context(|| format!("Failed to change to working directory '{}'", work_dir))?;

        // Small delay to allow directory change
        thread::sleep(Duration::from_millis(100));
    }

    // Set environment variables
    for (key, value) in env_vars {
        let env_command = format!("export {}='{}'", key, value.replace('\'', "'\"'\"'"));
        tmux::send_keys(session_name, &agent_pane.to_string(), &env_command)
            .with_context(|| format!("Failed to set environment variable '{}'", key))?;

        thread::sleep(Duration::from_millis(50));
    }

    // Send the command to the agent pane
    tmux::send_keys(session_name, &agent_pane.to_string(), command)
        .with_context(|| format!("Failed to send command to agent '{}'", agent.id))?;

    Ok(())
}

/// Find the pane that corresponds to the given agent.
fn find_agent_pane(
    panes: &[tmux::PaneInfo],
    agent: &crate::models::Agent,
    session_name: &str
) -> Result<usize> {
    // Try to find the pane by matching the workspace path
    if let Some(workspace) = &agent.worktree_path {
        for pane in panes {
            if let Some(current_path) = &pane.current_path {
                if current_path.contains(&*workspace.to_string_lossy()) || current_path.ends_with(&agent.id) {
                    return Ok(pane.index);
                }
            }
        }
    }

    // Fallback: try to find by agent ID in pane title or current command
    for pane in panes {
        if let Some(current_cmd) = &pane.current_command {
            if current_cmd.contains(&agent.id) {
                return Ok(pane.index);
            }
        }
    }

    // Another fallback: assume agent 1 is pane 0, agent 2 is pane 1, etc.
    if let Ok(agent_num) = agent.id.parse::<usize>() {
        let expected_pane = agent_num - 1; // 0-indexed
        if expected_pane < panes.len() {
            return Ok(expected_pane);
        }
    }

    Err(SpriteError::pane_not_found(
        format!("Could not find tmux pane for agent '{}' in session '{}'", agent.id, session_name)
    ).into())
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
                    None::<String>
                ).into());
            }
            env_map.insert(key.to_string(), value.to_string());
        } else {
            return Err(SpriteError::validation(
                format!("Invalid environment variable format: '{}'. Expected KEY=VALUE", env_var),
                Some("env_var".to_string()),
                Some(env_var.clone() as String)
            ).into());
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
            "NODE_ENV=production".to_string(),
            "PORT=3000".to_string(),
            "DEBUG=".to_string(),
        ];

        let result = parse_env_vars(&env_vars).unwrap();
        assert_eq!(result.get("NODE_ENV"), Some(&"production".to_string()));
        assert_eq!(result.get("PORT"), Some(&"3000".to_string()));
        assert_eq!(result.get("DEBUG"), Some(&"".to_string()));

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
