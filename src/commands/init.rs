use anyhow::{Context, Result};
use crate::error::SpriteError;
use crate::utils::git;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct InitOptions {
    pub force: bool,
    pub agents: u32,
}

pub fn execute(options: InitOptions) -> Result<()> {
    let start_time = Instant::now();

    println!("🚀 Initializing Sprite multi-agent environment...");

    // T026: Validate git repository
    git::validate_git_repository()
        .context("Failed to validate git repository")?;

    // T028: Handle edge case - existing configuration
    let agents_dir = PathBuf::from("agents");
    let config_file = agents_dir.join("agents.yaml");

    if config_file.exists() && !options.force {
        return Err(SpriteError::config(
            "Configuration file already exists. Use --force to overwrite."
        ).into());
    }

    // Create directory structure
    create_agents_directory_structure(&agents_dir, options.agents)?;

    // Generate configuration file
    generate_agents_config(&config_file, options.agents)?;

    // Create shell script templates
    create_shell_script_templates(&agents_dir)?;

    // Create tmux profile templates
    create_tmux_profile_templates(&agents_dir)?;

    // Create .envrc for direnv if direnv is available
    create_direnv_config(&agents_dir)?;

    let duration = start_time.elapsed();
    println!("✅ Sprite environment initialized successfully in {:.1}s!", duration.as_secs_f64());
    println!();
    println!("📁 Created directories:");
    println!("   agents/");
    println!("   agents/scripts/");
    println!("   agents/profiles/");
    println!();
    println!("📄 Generated configuration:");
    println!("   agents/agents.yaml");
    println!();

    if options.agents > 0 {
        println!("🤖 Configured {} agents:", options.agents);
        for i in 1..=options.agents {
            println!("   Agent {}: agents/{}", i, i);
        }
        println!();
    }

    println!("🎯 Next steps:");
    println!("   1. Review agents/agents.yaml and customize agent configurations");
    println!("   2. Run 'sprite install' to create agent worktrees");
    println!("   3. Run 'sprite start' to begin supervision session");

    Ok(())
}

fn create_agents_directory_structure(agents_dir: &Path, agent_count: u32) -> Result<()> {
    println!("📁 Creating directory structure...");

    // Create main agents directory
    fs::create_dir_all(agents_dir)
        .context("Failed to create agents directory")?;

    // Create scripts directory
    let scripts_dir = agents_dir.join("scripts");
    fs::create_dir_all(&scripts_dir)
        .context("Failed to create scripts directory")?;

    // Create profiles directory
    let profiles_dir = agents_dir.join("profiles");
    fs::create_dir_all(&profiles_dir)
        .context("Failed to create profiles directory")?;

    // Create individual agent directories
    for i in 1..=agent_count {
        let agent_dir = agents_dir.join(i.to_string());
        fs::create_dir_all(&agent_dir)
            .with_context(|| format!("Failed to create agent {} directory", i))?;
    }

    Ok(())
}

fn generate_agents_config(config_file: &Path, agent_count: u32) -> Result<()> {
    println!("📝 Generating configuration file...");

    let config = if agent_count == 0 {
        // Empty configuration template
        include_str!("../templates/empty_agents.yaml").to_string()
    } else {
        // Generate configuration with agents
        generate_config_content(agent_count)?
    };

    fs::write(config_file, config)
        .context("Failed to write agents.yaml configuration")?;

    Ok(())
}

fn generate_config_content(agent_count: u32) -> Result<String> {
    let mut config = String::new();

    config.push_str("# Sprite Multi-Agent Configuration\n");
    config.push_str("# Generated automatically by 'sprite init'\n");
    config.push_str("# Customize this file to match your project needs\n\n");

    config.push_str("agents:\n");
    for i in 1..=agent_count {
        config.push_str(&format!(
            "  {}:\n\
            branch: agents/{}\n\
            worktree_path: agents/{}\n\
            model: claude-sonnet-4\n\
            description: \"Agent {} workspace\"\n",
            i, i, i, i
        ));
    }

    config.push_str("\nsession:\n");
    config.push_str("  name: sprite-session\n");
    config.push_str("  profile: profile0\n");

    config.push_str("\nsync:\n");
    config.push_str("  auto_sync: false\n");
    config.push_str("  conflict_resolution: manual\n");
    config.push_str("  exclude_branches: []\n");

    Ok(config)
}

fn create_shell_script_templates(agents_dir: &Path) -> Result<()> {
    println!("🔧 Creating shell script templates...");

    let scripts_dir = agents_dir.join("scripts");

    // Create setup.sh script
    let setup_script = include_str!("../templates/setup.sh");
    fs::write(scripts_dir.join("setup.sh"), setup_script)?;

    // Make scripts executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(scripts_dir.join("setup.sh"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(scripts_dir.join("setup.sh"), perms)?;
    }

    // Create start-agents.sh script
    let start_script = include_str!("../templates/start-agents.sh");
    fs::write(scripts_dir.join("start-agents.sh"), start_script)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(scripts_dir.join("start-agents.sh"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(scripts_dir.join("start-agents.sh"), perms)?;
    }

    // Create attach-session.sh script
    let attach_script = include_str!("../templates/attach-session.sh");
    fs::write(scripts_dir.join("attach-session.sh"), attach_script)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(scripts_dir.join("attach-session.sh"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(scripts_dir.join("attach-session.sh"), perms)?;
    }

    Ok(())
}

fn create_tmux_profile_templates(agents_dir: &Path) -> Result<()> {
    println!("🖼️  Creating tmux profile templates...");

    let profiles_dir = agents_dir.join("profiles");

    // Profile 0: Top + split bottom (2-3 agents)
    let profile0 = include_str!("../templates/profile0.sh");
    fs::write(profiles_dir.join("profile0.sh"), profile0)?;

    // Profile 1: Left column + stacked right
    let profile1 = include_str!("../templates/profile1.sh");
    fs::write(profiles_dir.join("profile1.sh"), profile1)?;

    // Profile 2: Top row + full-width bottom
    let profile2 = include_str!("../templates/profile2.sh");
    fs::write(profiles_dir.join("profile2.sh"), profile2)?;

    // Profile 3: Single full-width top (focus mode)
    let profile3 = include_str!("../templates/profile3.sh");
    fs::write(profiles_dir.join("profile3.sh"), profile3)?;

    // Profile 4: Three-pane layout
    let profile4 = include_str!("../templates/profile4.sh");
    fs::write(profiles_dir.join("profile4.sh"), profile4)?;

    // Profile 5: Six-pane dashboard
    let profile5 = include_str!("../templates/profile5.sh");
    fs::write(profiles_dir.join("profile5.sh"), profile5)?;

    // Make all profile scripts executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for i in 0..=5 {
            let profile_file = profiles_dir.join(format!("profile{}.sh", i));
            if profile_file.exists() {
                let mut perms = fs::metadata(&profile_file)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&profile_file, perms)?;
            }
        }
    }

    Ok(())
}

fn create_direnv_config(agents_dir: &Path) -> Result<()> {
    // Check if direnv is available
    let direnv_available = std::process::Command::new("direnv")
        .arg("version")
        .output()
        .is_ok();

    if !direnv_available {
        return Ok(()); // Skip if direnv is not available
    }

    println!("🔌 Creating direnv configuration...");

    let envrc_content = r#"# Sprite Multi-Agent Environment Configuration
# Generated automatically by 'sprite init'

# Load common environment variables
export SPRITE_ENV="development"
export SPRITE_AGENTS_DIR="$(pwd)/agents"

# Add scripts to PATH
export PATH="$SPRITE_AGENTS_DIR/scripts:$PATH"

# Common aliases for sprite commands
alias sa='sprite attach'
alias ss='sprite start'
alias sk='sprite kill'
alias sw='sprite warp'
alias sz='sprite zoom'

# Auto-load environment when entering agent directories
layout_dir agents
"#;

    fs::write(".envrc", envrc_content)?;

    // Also create .envrc in agents directory
    let agents_envrc = r#"# Sprite Agents Environment
# Generated automatically by 'sprite init'

export SPRITE_AGENT_DIR="$(pwd)"
"#;

    fs::write(agents_dir.join(".envrc"), agents_envrc)?;

    println!("   Created .envrc files for direnv");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_create_agents_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");

        let result = create_agents_directory_structure(&agents_dir, 3);

        assert!(result.is_ok());
        assert!(agents_dir.exists());
        assert!(agents_dir.join("scripts").exists());
        assert!(agents_dir.join("profiles").exists());
        assert!(agents_dir.join("1").exists());
        assert!(agents_dir.join("2").exists());
        assert!(agents_dir.join("3").exists());
    }

    #[test]
    fn test_generate_config_content() {
        let result = generate_config_content(2);

        assert!(result.is_ok());
        let config = result.unwrap();

        assert!(config.contains("agents:"));
        assert!(config.contains("1:"));
        assert!(config.contains("2:"));
        assert!(config.contains("session:"));
        assert!(config.contains("sync:"));
    }

    #[test]
    fn test_generate_empty_config() {
        let result = generate_config_content(0);

        assert!(result.is_ok());
        let config = result.unwrap();

        assert!(config.contains("agents:"));
        assert!(!config.contains("1:"));
    }
}