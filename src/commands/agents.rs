use crate::cli::AgentsCommands;
use crate::commands::config::{SimpleAgentConfig, SpriteConfig};
use crate::error::SpriteError;
use crate::utils::git;
use anyhow::{Context, Result};
use std::io::{self, Write};
use std::path::PathBuf;

/// Execute agents command with subcommands
pub fn execute(command: AgentsCommands) -> Result<()> {
    match command {
        AgentsCommands::List => list_agents(),
        AgentsCommands::Create {
            agent_id,
            branch,
            workspace,
            model,
            description,
            no_workspace,
        } => create_agent(
            agent_id,
            branch,
            workspace,
            model,
            description,
            no_workspace,
        ),
        AgentsCommands::Remove {
            agent_id,
            force,
            keep_workspace,
        } => remove_agent(agent_id, force, keep_workspace),
        AgentsCommands::Show { agent_id } => show_agent(agent_id),
        AgentsCommands::Validate => validate_agents(),
        AgentsCommands::Provision { agent_id } => provision_agents(agent_id),
    }
}

/// List all configured agents
fn list_agents() -> Result<()> {
    let config = SpriteConfig::load().context("Failed to load configuration")?;

    if config.agents.is_empty() {
        println!("📝 No agents configured.");
        println!("Use 'sprite agents create <id>' to add your first agent.");
        return Ok(());
    }

    println!("🤖 Configured Agents ({}) :", config.agents.len());
    println!();

    // Sort agent IDs for consistent display
    let mut agent_ids: Vec<_> = config.agents.keys().collect();
    agent_ids.sort();

    for agent_id in agent_ids {
        let agent_config = &config.agents[agent_id];
        let workspace_path = PathBuf::from(&agent_config.worktree_path);
        let workspace_exists = workspace_path.exists();

        // Get git status if workspace exists
        let git_status = if workspace_exists {
            match git::get_current_branch_at(&workspace_path) {
                Ok(branch) => {
                    if branch == agent_config.branch {
                        format!("✅ Ready (branch: {})", branch)
                    } else {
                        format!("⚠️  Wrong branch ({} ≠ {})", branch, agent_config.branch)
                    }
                }
                Err(_) => "❌ Not a git repository".to_string(),
            }
        } else {
            "❌ Workspace missing".to_string()
        };

        println!("  Agent {} :", agent_id);
        println!(
            "    📁 Workspace: {} {}",
            agent_config.worktree_path, git_status
        );
        println!("    🌿 Branch: {}", agent_config.branch);
        println!("    🤖 Model: {}", agent_config.model);
        println!("    📝 Description: {}", agent_config.description);
        println!();
    }

    Ok(())
}

/// Create a new agent
fn create_agent(
    agent_id: String,
    branch: Option<String>,
    workspace: Option<String>,
    model: String,
    description: Option<String>,
    no_workspace: bool,
) -> Result<()> {
    println!("🚀 Creating new agent: {}", agent_id);

    // Load existing configuration
    let mut config = SpriteConfig::load().context("Failed to load configuration")?;

    // Check if agent already exists
    if config.agents.contains_key(&agent_id) {
        return Err(SpriteError::config(format!(
            "Agent {} already exists. Use a different ID or remove the existing agent first.",
            agent_id
        ))
        .into());
    }

    // Validate agent ID format
    if agent_id.is_empty() || agent_id.contains('/') || agent_id.contains("..") {
        return Err(SpriteError::config(
            "Invalid agent ID. Must be a simple identifier without slashes or '..'.",
        )
        .into());
    }

    // Set default values
    let branch_name = branch.unwrap_or_else(|| format!("agents/{}", agent_id));
    let workspace_path = workspace.unwrap_or_else(|| format!("agents/{}", agent_id));
    let agent_description = description.unwrap_or_else(|| format!("Agent {} workspace", agent_id));

    // Create agent configuration
    let agent_config = SimpleAgentConfig {
        branch: branch_name.clone(),
        worktree_path: workspace_path.clone(),
        model,
        description: agent_description,
    };

    // Add agent to configuration
    config.agents.insert(agent_id.clone(), agent_config);

    // Save configuration
    config
        .save()
        .context("Failed to save updated configuration")?;

    println!("  ✅ Agent configuration saved");

    // Create workspace if requested
    if !no_workspace {
        println!("  🔧 Creating workspace...");

        // Validate git repository first
        git::validate_git_repository().context("Failed to validate git repository")?;

        // Create branch if it doesn't exist
        if !git::branch_exists(&branch_name)? {
            git::create_branch_from(&branch_name, "main") // Assuming main is the base branch
                .with_context(|| format!("Failed to create branch: {}", branch_name))?;
            println!("    🌿 Created branch: {}", branch_name);
        } else {
            println!("    🌿 Using existing branch: {}", branch_name);
        }

        // Create worktree
        let workspace_path_buf = PathBuf::from(&workspace_path);
        git::create_worktree(&workspace_path_buf, &branch_name)
            .with_context(|| format!("Failed to create worktree: {}", workspace_path))?;

        println!("    ✅ Created workspace: {}", workspace_path);
    } else {
        println!("  ⏭️  Skipping workspace creation (use --no-workspace)");
    }

    println!("✅ Agent {} created successfully!", agent_id);
    println!("   Configuration: agents/agents.yaml");
    if !no_workspace {
        println!("   Workspace: {}", workspace_path);
        println!("   Branch: {}", branch_name);
    }

    Ok(())
}

/// Remove an agent
fn remove_agent(agent_id: String, force: bool, keep_workspace: bool) -> Result<()> {
    println!("🗑️  Removing agent: {}", agent_id);

    // Load configuration
    let mut config = SpriteConfig::load().context("Failed to load configuration")?;

    // Check if agent exists
    if !config.agents.contains_key(&agent_id) {
        return Err(SpriteError::config(format!("Agent {} does not exist.", agent_id)).into());
    }

    let agent_config = &config.agents[&agent_id];

    // Confirmation prompt (unless force is true)
    if !force {
        println!(
            "This will remove agent {} with the following configuration:",
            agent_id
        );
        println!("  Workspace: {}", agent_config.worktree_path);
        println!("  Branch: {}", agent_config.branch);
        println!("  Model: {}", agent_config.model);
        if !keep_workspace {
            println!("  ⚠️  Workspace directory will be REMOVED");
        } else {
            println!("  📁 Workspace directory will be kept");
        }
        println!();
        print!("Are you sure you want to continue? [y/N] ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("❌ Agent removal cancelled.");
            return Ok(());
        }
    }

    // Remove workspace directory (unless keep_workspace is true)
    if !keep_workspace {
        let workspace_path = PathBuf::from(&agent_config.worktree_path);
        if workspace_path.exists() {
            // First, remove the git worktree
            if let Err(e) = git::remove_worktree(&workspace_path) {
                println!("⚠️  Warning: Failed to remove git worktree: {}", e);
            }

            // Then remove the directory if it still exists
            if workspace_path.exists() {
                std::fs::remove_dir_all(&workspace_path).with_context(|| {
                    format!(
                        "Failed to remove workspace directory: {}",
                        workspace_path.display()
                    )
                })?;
                println!(
                    "  ✅ Removed workspace directory: {}",
                    agent_config.worktree_path
                );
            }
        } else {
            println!(
                "  ℹ️  Workspace directory does not exist: {}",
                agent_config.worktree_path
            );
        }
    } else {
        println!(
            "  📁 Keeping workspace directory: {}",
            agent_config.worktree_path
        );
    }

    // Remove agent from configuration
    config.agents.remove(&agent_id);

    // Save configuration
    config
        .save()
        .context("Failed to save updated configuration")?;

    println!("  ✅ Agent configuration removed");
    println!("✅ Agent {} removed successfully!", agent_id);

    Ok(())
}

/// Show agent details
fn show_agent(agent_id: Option<String>) -> Result<()> {
    let config = SpriteConfig::load().context("Failed to load configuration")?;

    match agent_id {
        Some(id) => {
            // Show specific agent
            if let Some(agent_config) = config.agents.get(&id) {
                println!("🤖 Agent {} :", id);
                println!("  📁 Workspace: {}", agent_config.worktree_path);
                println!("  🌿 Branch: {}", agent_config.branch);
                println!("  🤖 Model: {}", agent_config.model);
                println!("  📝 Description: {}", agent_config.description);

                // Show workspace status
                let workspace_path = PathBuf::from(&agent_config.worktree_path);
                if workspace_path.exists() {
                    println!();
                    println!("  📊 Workspace Status:");

                    if let Ok(branch) = git::get_current_branch_at(&workspace_path) {
                        println!("    Current branch: {}", branch);
                        if branch == agent_config.branch {
                            println!("    Status: ✅ On correct branch");
                        } else {
                            println!(
                                "    Status: ⚠️  Wrong branch (should be {})",
                                agent_config.branch
                            );
                        }
                    }

                    if let Ok(status) = git::get_status_string_at(&workspace_path) {
                        if status.is_empty() {
                            println!("    Working tree: ✅ Clean");
                        } else {
                            println!("    Working tree: ⚠️  Has changes");
                            for line in status.lines().take(5) {
                                println!("      {}", line);
                            }
                        }
                    }
                } else {
                    println!();
                    println!("  ❌ Workspace does not exist");
                    println!("  💡 Run 'sprite agents provision {}' to create it", id);
                }
            } else {
                return Err(SpriteError::config(format!("Agent {} does not exist.", id)).into());
            }
        }
        None => {
            // Show all agents (same as list)
            list_agents()?;
        }
    }

    Ok(())
}

/// Validate all agent workspaces
fn validate_agents() -> Result<()> {
    let config = SpriteConfig::load().context("Failed to load configuration")?;

    if config.agents.is_empty() {
        println!("📝 No agents configured.");
        return Ok(());
    }

    println!("🔍 Validating agent workspaces...");
    println!();

    let mut valid_count = 0;
    let total_count = config.agents.len();

    for (agent_id, agent_config) in &config.agents {
        let workspace_path = PathBuf::from(&agent_config.worktree_path);

        print!("  Agent {}: ", agent_id);

        // Check workspace exists
        if !workspace_path.exists() {
            println!("❌ Workspace does not exist");
            continue;
        }

        // Check it's a git repository
        if !git::is_git_repository_at(&workspace_path)? {
            println!("❌ Not a git repository");
            continue;
        }

        // Check correct branch
        match git::get_current_branch_at(&workspace_path) {
            Ok(branch) => {
                if branch == agent_config.branch {
                    print!("✅ Valid");
                } else {
                    print!("⚠️  Wrong branch ({} ≠ {})", branch, agent_config.branch);
                }
            }
            Err(_) => {
                print!("❌ Cannot determine current branch");
            }
        }

        // Check if branch exists
        if git::branch_exists(&agent_config.branch)? {
            println!(" (branch exists)");
        } else {
            println!(" (branch missing)");
        }

        valid_count += 1;
    }

    println!();
    if valid_count == total_count {
        println!("✅ All {} agent workspaces are valid!", total_count);
    } else {
        println!(
            "⚠️  {}/{} agent workspaces are valid",
            valid_count, total_count
        );
        println!("💡 Use 'sprite agents provision' to fix missing workspaces");
    }

    Ok(())
}

/// Provision workspaces for agents
fn provision_agents(agent_id: Option<String>) -> Result<()> {
    let config = SpriteConfig::load().context("Failed to load configuration")?;

    // Validate git repository first
    git::validate_git_repository().context("Failed to validate git repository")?;

    let agents_to_provision: Vec<_> = if let Some(id) = agent_id {
        // Provision specific agent
        if let Some(agent_config) = config.agents.get(&id) {
            vec![(id.clone(), agent_config.clone())]
        } else {
            return Err(SpriteError::config(format!("Agent {} does not exist.", id)).into());
        }
    } else {
        // Provision all agents
        config
            .agents
            .iter()
            .map(|(id, config)| (id.clone(), config.clone()))
            .collect()
    };

    if agents_to_provision.is_empty() {
        println!("📝 No agents to provision.");
        return Ok(());
    }

    println!(
        "🔧 Provisioning workspaces for {} agent(s)...",
        agents_to_provision.len()
    );

    for (id, agent_config) in agents_to_provision {
        println!("  🤖 Agent {}:", id);

        let workspace_path = PathBuf::from(&agent_config.worktree_path);

        // Check if workspace already exists
        if workspace_path.exists() {
            if git::is_git_repository_at(&workspace_path)? {
                println!("    ✅ Workspace already exists");

                // Check if it's on the correct branch
                if let Ok(current_branch) = git::get_current_branch_at(&workspace_path) {
                    if current_branch != agent_config.branch {
                        println!("    🔄 Switching to branch: {}", agent_config.branch);
                        git::checkout_branch(&workspace_path, &agent_config.branch)?;
                        println!("    ✅ Switched to correct branch");
                    }
                }
            } else {
                return Err(SpriteError::config(format!(
                    "Workspace path exists but is not a git repository: {}",
                    agent_config.worktree_path
                ))
                .into());
            }
        } else {
            // Create branch if it doesn't exist
            if !git::branch_exists(&agent_config.branch)? {
                println!("    🌿 Creating branch: {}", agent_config.branch);
                git::create_branch_from(&agent_config.branch, "main")?;
            }

            // Create worktree
            println!("    📁 Creating workspace: {}", agent_config.worktree_path);
            git::create_worktree(&workspace_path, &agent_config.branch)?;
            println!("    ✅ Workspace created");
        }
    }

    println!("✅ Workspace provisioning complete!");

    Ok(())
}
