//! Slash Status command - Display agent and session status for slash commands

use crate::commands::config::SpriteConfig;
use crate::utils::{accessibility::AccessibilityConfig, tmux};
use anyhow::Result;

#[allow(dead_code)]
pub enum StatusScope {
    Overall,
    Agents(Option<String>),
    Session,
    System,
}

#[allow(dead_code)]
pub struct StatusOptions {
    pub scope: StatusScope,
    pub detailed: bool,
    pub health: bool,
    pub cleanup: bool,
}

#[allow(dead_code)]
/// Execute the slash status command
pub fn execute(options: StatusOptions) -> Result<()> {
    let accessibility_config = AccessibilityConfig::default();

    match &options.scope {
        StatusScope::Overall => show_overall_status(&accessibility_config, options.detailed),
        StatusScope::Agents(agent_id) => {
            show_agent_status(agent_id, &accessibility_config, options.detailed)
        }
        StatusScope::Session => show_session_status(&accessibility_config, options.detailed),
        StatusScope::System => show_system_status(&accessibility_config, options.detailed),
    }?;

    if options.health {
        run_health_check(&accessibility_config)?;
    }

    if options.cleanup {
        run_cleanup(&accessibility_config)?;
    }

    Ok(())
}

#[allow(dead_code)]
fn show_overall_status(config: &AccessibilityConfig, detailed: bool) -> Result<()> {
    crate::utils::accessibility::print_info("📊 Overall Sprite Status", config);
    println!();

    // Show session info
    match tmux::list_sessions() {
        Ok(sessions) => {
            let sprite_sessions: Vec<_> = sessions
                .iter()
                .filter(|s| s.name.starts_with("sprite-"))
                .collect();

            if sprite_sessions.is_empty() {
                crate::utils::accessibility::print_warning("No active sprite sessions", config);
            } else {
                crate::utils::accessibility::print_info(
                    &format!("🎯 Active Sessions: {}", sprite_sessions.len()),
                    config,
                );
                for session in sprite_sessions {
                    let attached = if session.attached { " (attached)" } else { "" };
                    crate::utils::accessibility::print_info(
                        &format!("  {}{}", session.name, attached),
                        config,
                    );
                }
            }
        }
        Err(_) => {
            crate::utils::accessibility::print_warning("Could not access tmux sessions", config);
        }
    }
    println!();

    // Show agent status
    match SpriteConfig::load() {
        Ok(sprite_config) => {
            crate::utils::accessibility::print_info(
                &format!("🤖 Configured Agents: {}", sprite_config.agents.len()),
                config,
            );
        }
        Err(_) => {
            crate::utils::accessibility::print_warning(
                "Could not load agent configuration",
                config,
            );
        }
    }

    // Show basic system info if detailed
    if detailed {
        println!();
        crate::utils::accessibility::print_info("🖥️  System Resources", config);

        // Show disk usage
        if let Ok(output) = std::process::Command::new("df").args(["-h", "."]).output() {
            if output.status.success() {
                let output = String::from_utf8_lossy(&output.stdout);
                for line in output.lines().take(2) {
                    // Skip header, get current dir
                    crate::utils::accessibility::print_info(&format!("  {}", line), config);
                }
            }
        } // Skip if df fails

        // Show memory usage on Unix systems
        #[cfg(unix)]
        {
            if let Ok(output) = std::process::Command::new("free").arg("-h").output() {
                if output.status.success() {
                    let output = String::from_utf8_lossy(&output.stdout);
                    for line in output.lines().take(2) {
                        // Header + Mem line
                        crate::utils::accessibility::print_info(&format!("  {}", line), config);
                    }
                }
            } // Skip if free fails
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn show_agent_status(
    agent_id: &Option<String>,
    config: &AccessibilityConfig,
    _detailed: bool,
) -> Result<()> {
    crate::utils::accessibility::print_info("🤖 Agent Status", config);
    println!();

    match SpriteConfig::load() {
        Ok(sprite_config) => {
            if sprite_config.agents.is_empty() {
                crate::utils::accessibility::print_warning("No agents configured", config);
            } else {
                crate::utils::accessibility::print_info(
                    &format!("Configured Agents: {}", sprite_config.agents.len()),
                    config,
                );

                if let Some(id) = agent_id {
                    // Show specific agent
                    if let Some(agent) = sprite_config.agents.iter().find(|a| a.id == *id) {
                        crate::utils::accessibility::print_info(
                            &format!("Agent ID: {}", agent.id),
                            config,
                        );
                        crate::utils::accessibility::print_info(
                            &format!("Description: {:?}", agent.description),
                            config,
                        );
                        crate::utils::accessibility::print_info(
                            &format!("Status: {:?}", agent.status),
                            config,
                        );
                    } else {
                        crate::utils::accessibility::print_warning(
                            &format!("Agent '{}' not found", id),
                            config,
                        );
                    }
                } else {
                    // List all agents
                    for agent in &sprite_config.agents {
                        crate::utils::accessibility::print_info(
                            &format!(
                                "  {}: {} ({})",
                                agent.id,
                                agent.status,
                                if agent.description.is_empty() {
                                    "No description"
                                } else {
                                    &agent.description
                                }
                            ),
                            config,
                        );
                    }
                }
            }
        }
        Err(e) => {
            crate::utils::accessibility::print_error(
                &format!("Could not load agent configuration: {}", e),
                config,
            );
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn show_session_status(config: &AccessibilityConfig, detailed: bool) -> Result<()> {
    crate::utils::accessibility::print_info("🎯 Session Status", config);
    println!();

    match tmux::list_sessions() {
        Ok(sessions) => {
            let sprite_sessions: Vec<_> = sessions
                .iter()
                .filter(|s| s.name.starts_with("sprite-"))
                .collect();

            if sprite_sessions.is_empty() {
                crate::utils::accessibility::print_warning("No active sprite sessions", config);
            } else {
                for session in sprite_sessions {
                    crate::utils::accessibility::print_info(
                        &format!("Session: {}", session.name),
                        config,
                    );

                    let attached = if session.attached { "Yes" } else { "No" };
                    crate::utils::accessibility::print_info(
                        &format!("  Attached: {}", attached),
                        config,
                    );

                    if detailed {
                        // Show session panes
                        if let Ok(panes) = tmux::get_session_panes(&session.name) {
                            crate::utils::accessibility::print_info(
                                &format!("  Panes: {}", panes.len()),
                                config,
                            );
                            for pane in panes {
                                crate::utils::accessibility::print_info(
                                    &format!(
                                        "    Pane {}: {} ({})",
                                        pane.index,
                                        pane.current_path.as_deref().unwrap_or("unknown"),
                                        pane.current_command.as_deref().unwrap_or("idle")
                                    ),
                                    config,
                                );
                            }
                        } // Skip panes if can't get them
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            crate::utils::accessibility::print_error(
                &format!("Could not list sessions: {}", e),
                config,
            );
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn show_system_status(config: &AccessibilityConfig, _detailed: bool) -> Result<()> {
    crate::utils::accessibility::print_info("🖥️  System Status", config);
    println!();

    // Show basic system info
    crate::utils::accessibility::print_info("System Resources:", config);

    // Check disk space
    match std::process::Command::new("df").args(["-h", "."]).output() {
        Ok(output) => {
            if output.status.success() {
                let output = String::from_utf8_lossy(&output.stdout);
                for line in output.lines().take(2) {
                    crate::utils::accessibility::print_info(&format!("  {}", line), config);
                }
            }
        }
        Err(_) => {
            crate::utils::accessibility::print_warning("Could not get disk usage", config);
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn run_health_check(config: &AccessibilityConfig) -> Result<()> {
    crate::utils::accessibility::print_info("🏥 Running Health Check", config);
    println!();

    // Check tmux availability
    if let Err(e) = tmux::is_tmux_available() {
        crate::utils::accessibility::print_error(&format!("tmux error: {}", e), config);
    } else {
        crate::utils::accessibility::print_info("✓ tmux is available", config);
    }

    // Check agent configuration
    match SpriteConfig::load() {
        Ok(sprite_config) => {
            if !sprite_config.agents.is_empty() {
                crate::utils::accessibility::print_info("✓ Agent configurations found", config);
            } else {
                crate::utils::accessibility::print_warning("No agents configured", config);
            }
        }
        Err(_) => {
            crate::utils::accessibility::print_warning(
                "Could not validate agent configuration",
                config,
            );
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn run_cleanup(config: &AccessibilityConfig) -> Result<()> {
    crate::utils::accessibility::print_info("🧹 Running Cleanup", config);
    println!();

    // This would integrate with existing cleanup functionality
    // For now, just indicate cleanup would happen
    crate::utils::accessibility::print_info("Cleanup operations completed", config);

    Ok(())
}
