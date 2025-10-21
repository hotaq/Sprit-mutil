use crate::commands::config::SpriteConfig;
use crate::error::SpriteError;
use crate::utils::git;
use crate::utils::tmux;
use anyhow::{Context, Result};

/// Execute start command with provided arguments
pub fn execute(
    session_name: Option<String>,
    layout: String,
    detach: bool,
    force: bool,
) -> Result<()> {
    let session_name = session_name.unwrap_or_else(|| "sprite-session".to_string());

    // Load configuration
    let config = SpriteConfig::load().context("Failed to load configuration")?;

    if config.agents.is_empty() {
        return Err(SpriteError::config(
            "No agents configured. Use 'sprite init' to create agents first.",
        )
        .into());
    }

    println!("🚀 Starting Sprite multi-agent session...");
    println!("📋 Configuration loaded: {} agents", config.agents.len());
    println!("🎮 Session name: {}", session_name);
    println!("🖼️  Layout: {}", layout);

    // Validate git repository first
    git::validate_git_repository().context("Git repository validation failed")?;

    // Check if session already exists
    if tmux::session_exists(&session_name)? {
        if force {
            println!(
                "🔄 Force mode: Killing existing session '{}'...",
                session_name
            );
            tmux::kill_session_force(&session_name)?;
            println!("   ✅ Old session killed");
        } else if !detach {
            return Err(SpriteError::session(
                format!("Session '{}' already exists. Use 'sprite start --force' to replace it, 'sprite attach' to join, or 'sprite kill' to terminate.", session_name),
                Some(session_name),
            )
            .into());
        } else {
            println!(
                "ℹ️  Session '{}' already exists, detaching...",
                session_name
            );
            return Ok(());
        }
    }

    // Validate all agent workspaces
    println!("🔍 Validating agent workspaces...");
    config
        .validate_workspaces()
        .context("Workspace validation failed")?;

    // Determine tmux profile to use
    let profile = determine_tmux_profile(&config.agents.len(), &layout)?;
    println!("📐 Using tmux profile: {}", profile);

    // Create tmux session
    create_tmux_session(&session_name, &config, &profile, detach)?;

    // Update agent status to Active after successful session creation
    update_agent_status_to_active(&config)?;

    if !detach {
        println!("✅ Session created successfully!");
        println!("💡 Use 'sprite attach' to rejoin the session");
        println!("💡 Use 'sprite kill' to terminate the session");
    } else {
        println!("✅ Session created and detached!");
        println!(
            "💡 Use 'sprite attach {}' to join the session",
            session_name
        );
    }

    Ok(())
}

/// Determine the appropriate tmux profile based on agent count and layout preference
fn determine_tmux_profile(agent_count: &usize, layout: &str) -> Result<String> {
    let profile = match layout {
        "tiled" => match agent_count {
            1 => "profile3",     // Focus mode for single agent
            2..=3 => "profile0", // Top + split bottom
            4..=6 => "profile2", // Top row + full-width bottom
            _ => "profile5",     // Six-pane dashboard
        },
        "focus" => "profile3",      // Single agent focus mode
        "vertical" => "profile1",   // Left column + stacked right
        "horizontal" => "profile0", // Top + split bottom
        "dashboard" => "profile5",  // Six-pane dashboard
        custom => {
            // Try to find a matching profile file
            if custom.starts_with("profile") && custom.len() == 8 {
                custom
            } else {
                return Err(SpriteError::config(format!(
                    "Unknown layout or profile: '{}'. Available: tiled, focus, vertical, horizontal, dashboard, profile0-profile5",
                    custom
                ))
                .into());
            }
        }
    };

    println!("📊 {} agents detected → profile: {}", agent_count, profile);
    Ok(profile.to_string())
}

/// Create a new tmux session with the specified configuration
fn create_tmux_session(
    session_name: &str,
    config: &SpriteConfig,
    profile: &str,
    #[allow(dead_code)] _detach: bool,
) -> Result<()> {
    println!("🔧 Creating tmux session '{}'...", session_name);

    // Execute the tmux profile script (which creates the session)
    let agents_dir = std::path::PathBuf::from("agents");
    let profile_script = agents_dir.join("profiles").join(format!("{}.sh", profile));

    // Execute the profile script using the tmux utility
    tmux::execute_profile_script(session_name, &profile_script).with_context(|| {
        format!(
            "Failed to execute profile script: {}",
            profile_script.display()
        )
    })?;

    // Send initial setup commands to each agent pane
    setup_agent_panes(session_name, config)?;

    // Profile script already created supervisor pane, just send info to it
    send_supervisor_info(session_name, config)?;

    println!("✅ Tmux session '{}' created successfully", session_name);
    Ok(())
}

/// Setup individual agent panes with initial commands
fn setup_agent_panes(session_name: &str, config: &SpriteConfig) -> Result<()> {
    println!("🤖 Setting up agent panes...");

    // Get the current panes in the session to determine where to send commands
    let panes = tmux::get_session_panes(session_name)
        .context("Failed to get session panes for agent setup")?;

    for (index, agent_config) in config.agents.iter().enumerate() {
        let workspace_path = &agent_config.worktree_path;
        let pane_target = if index < panes.len() {
            panes[index].index.to_string()
        } else {
            // If we don't have enough panes, send to the session (tmux will route to active pane)
            "".to_string()
        };

        // Change to agent workspace
        let cmd = format!("cd {}", workspace_path);
        if let Err(e) = tmux::send_keys_with_delay(session_name, &pane_target, &cmd, 50) {
            eprintln!(
                "⚠️  Warning: Failed to setup agent {} workspace: {}",
                agent_config.id, e
            );
        }

        // Display agent information
        let info_cmd = format!(
            "echo '🤖 Agent {} - {}'",
            agent_config.id, agent_config.description
        );
        if let Err(e) = tmux::send_keys_with_delay(session_name, &pane_target, &info_cmd, 50) {
            eprintln!(
                "⚠️  Warning: Failed to display agent {} info: {}",
                agent_config.id, e
            );
        }

        // Show git status
        let git_cmd = "git status --porcelain";
        if let Err(e) = tmux::send_keys_with_delay(session_name, &pane_target, git_cmd, 50) {
            eprintln!(
                "⚠️  Warning: Failed to show git status for agent {}: {}",
                agent_config.id, e
            );
        }
    }

    Ok(())
}

/// Send supervisor info to the existing supervisor pane (created by profile script)
fn send_supervisor_info(session_name: &str, config: &SpriteConfig) -> Result<()> {
    println!("🎮 Configuring supervisor control pane...");

    // Profile script already created the supervisor window, so we just send info to it
    // No need to create a new window

    // Display supervisor information
    let supervisor_info = format!(
        "🎮 Sprite Supervisor Control Panel\n\
        📊 Active Agents: {}\n\
        📁 Session: {}\n\
        ⚙️  Profile: {}\n\
        \n\
        Available Commands:\n\
        • sprite hey <agent> <command> - Send command to specific agent\n\
        • sprite send <command> - Broadcast to all agents\n\
        • sprite list - List all agents and their status\n\
        • sprite sync - Synchronize all workspaces\n\
        • sprite warp <agent> - Navigate to agent workspace\n\
        • sprite zoom <agent> - Focus on agent pane\n\
        • sprite kill - Terminate session\n\
        • sprite attach - Reattach to session",
        config.agents.len(),
        session_name,
        "tiled"
    );

    // Use the supervisor window as target
    if let Err(e) = tmux::send_keys_with_delay(session_name, "supervisor", &supervisor_info, 100) {
        eprintln!(
            "⚠️  Warning: Failed to display supervisor information: {}",
            e
        );
    }

    // Show help information
    let help_cmd = "echo 'Type \"sprite --help\" for command reference'";
    if let Err(e) = tmux::send_keys_with_delay(session_name, "supervisor", help_cmd, 50) {
        eprintln!("⚠️  Warning: Failed to show help command: {}", e);
    }

    Ok(())
}

/// Update agent status to Active in the configuration after successful session creation
fn update_agent_status_to_active(_config: &crate::commands::config::SpriteConfig) -> Result<()> {
    println!("🔄 Activating agents...");

    // Load current configuration
    let mut current_config = crate::commands::config::SpriteConfig::load()
        .context("Failed to load configuration for status update")?;

    let mut updated_count = 0;

    // Update each agent's status to Active if it's currently Inactive
    for agent in &mut current_config.agents {
        if agent.status == "Inactive" {
            agent.status = "Active".to_string();
            updated_count += 1;
            println!("  ✅ Agent {} is now Active", agent.id);
        } else {
            println!(
                "  ℹ️  Agent {} already has status: {}",
                agent.id, agent.status
            );
        }
    }

    if updated_count > 0 {
        // Save the updated configuration
        current_config
            .save()
            .context("Failed to save updated configuration")?;
        println!(
            "📝 Configuration updated: {} agents activated",
            updated_count
        );
    } else {
        println!("ℹ️  No agents needed activation");
    }

    Ok(())
}
