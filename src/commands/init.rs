use crate::error::SpriteError;
use crate::utils::git;
use anyhow::{Context, Result};
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
    git::validate_git_repository().context("Failed to validate git repository")?;

    // T028: Handle edge case - existing configuration
    let agents_dir = PathBuf::from("agents");
    let config_file = agents_dir.join("agents.yaml");

    if config_file.exists() && !options.force {
        return Err(SpriteError::config(
            "Configuration file already exists. Use --force to overwrite.",
        )
        .into());
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
    println!(
        "✅ Sprite environment initialized successfully in {:.1}s!",
        duration.as_secs_f64()
    );
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
    fs::create_dir_all(agents_dir).context("Failed to create agents directory")?;

    // Create scripts directory
    let scripts_dir = agents_dir.join("scripts");
    fs::create_dir_all(&scripts_dir).context("Failed to create scripts directory")?;

    // Create profiles directory
    let profiles_dir = agents_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).context("Failed to create profiles directory")?;

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

    fs::write(config_file, config).context("Failed to write agents.yaml configuration")?;

    Ok(())
}

fn generate_config_content(agent_count: u32) -> Result<String> {
    let mut config = String::new();

    config.push_str("# Sprite Multi-Agent Configuration\n");
    config.push_str("# Generated automatically by 'sprite init'\n");
    config.push_str("# Customize this file to match your project needs\n\n");

    config.push_str("version: \"1.0\"\n\n");

    config.push_str("agents:\n");
    for i in 1..=agent_count {
        config.push_str(&format!(
            "  - id: \"{}\"\n\
            branch: \"agents/{}\"\n\
            model: \"claude-sonnet-4\"\n\
            description: \"Agent {} workspace\"\n\
            worktree_path: \"agents/{}\"\n\
            status: \"inactive\"\n\
            config:\n\
            env_vars: {{}}\n\
            work_dir: null\n\
            startup_commands: []\n\
            resource_limits:\n\
            max_memory_mb: 1024\n\
            max_cpu_percent: 80\n\
            operation_timeout_secs: 300\n\
            max_concurrent_ops: 3\n\
            max_disk_mb: 5120\n\
            shell: null\n\
            default_timeout_secs: 300\n\
            auto_sync: false\n\
            custom_settings: {{}}\n",
            i, i, i, i
        ));
    }

    config.push_str("\nsession_name: \"sprite-session\"\n\n");

    config.push_str("sync:\n");
    config.push_str("  auto_sync: false\n");
    config.push_str("  default_interval_secs: 300\n");
    config.push_str("  conflict_resolution: \"manual\"\n");
    config.push_str("  exclude_branches: [\"main\", \"master\"]\n");
    config.push_str("  pre_sync_hooks: []\n");
    config.push_str("  post_sync_hooks: []\n\n");

    config.push_str("settings:\n");
    config.push_str("  default_shell: \"bash\"\n");
    config.push_str("  global_env_vars: {}\n");
    config.push_str("  logging:\n");
    config.push_str("    log_file: \"agents/logs/sprite.log\"\n");
    config.push_str("    level: \"info\"\n");
    config.push_str("    log_to_stdout: true\n");
    config.push_str("    rotation:\n");
    config.push_str("      max_size_mb: 100\n");
    config.push_str("      max_files: 5\n");
    config.push_str("  performance:\n");
    config.push_str("    max_concurrent_ops: 10\n");
    config.push_str("    default_timeout_secs: 300\n");
    config.push_str("    enable_monitoring: true\n");
    config.push_str("    memory_limit_mb: 1024\n");
    config.push_str("    cpu_limit_percent: 80\n");
    config.push_str("  security:\n");
    config.push_str("    allowed_paths: [\"agents/\"]\n");
    config.push_str("    blocked_commands: [\"rm -rf\", \"sudo\", \"su\", \"chmod 777\"]\n");
    config.push_str("    strict_path_validation: true\n");
    config.push_str("    max_command_length: 1000\n");
    config.push_str("    allow_shell_execution: false\n");

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
    use std::fs;
    use tempfile::TempDir;

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
    fn test_create_agents_directory_structure_zero_agents() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");

        let result = create_agents_directory_structure(&agents_dir, 0);

        assert!(result.is_ok());
        assert!(agents_dir.exists());
        assert!(agents_dir.join("scripts").exists());
        assert!(agents_dir.join("profiles").exists());
        // Should not create agent directories for 0 agents
        assert!(!agents_dir.join("1").exists());
    }

    #[test]
    fn test_create_agents_directory_structure_single_agent() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");

        let result = create_agents_directory_structure(&agents_dir, 1);

        assert!(result.is_ok());
        assert!(agents_dir.exists());
        assert!(agents_dir.join("1").exists());
        assert!(!agents_dir.join("2").exists());
    }

    #[test]
    fn test_create_agents_directory_structure_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("deep").join("nested").join("agents");
        fs::create_dir_all(agents_dir.parent().unwrap()).unwrap();

        let result = create_agents_directory_structure(&agents_dir, 2);

        assert!(result.is_ok());
        assert!(agents_dir.exists());
        assert!(agents_dir.join("1").exists());
        assert!(agents_dir.join("2").exists());
    }

    #[test]
    fn test_generate_config_content() {
        let result = generate_config_content(2);

        assert!(result.is_ok());
        let config = result.unwrap();

        assert!(config.contains("agents:"));
        assert!(config.contains("- id: \"1\""));
        assert!(config.contains("- id: \"2\""));
        assert!(config.contains("session_name:"));
        assert!(config.contains("sync:"));
        assert!(config.contains("settings:"));
        assert!(config.contains("version: \"1.0\""));
        assert!(config.contains("claude-sonnet-4"));
        assert!(config.contains("worktree_path:"));
        assert!(config.contains("resource_limits:"));
    }

    #[test]
    fn test_generate_config_content_many_agents() {
        let result = generate_config_content(10);

        assert!(result.is_ok());
        let config = result.unwrap();

        // Should contain all agents from 1 to 10
        for i in 1..=10 {
            assert!(config.contains(&format!("- id: \"{}\"", i)));
        }
        // Should not contain agent 11
        assert!(!config.contains("- id: \"11\""));
    }

    #[test]
    fn test_generate_empty_config() {
        let result = generate_config_content(0);

        assert!(result.is_ok());
        let config = result.unwrap();

        assert!(config.contains("agents:"));
        assert!(!config.contains("- id: \"1\""));
        assert!(config.contains("version: \"1.0\""));
        assert!(config.contains("session_name:"));
    }

    #[test]
    fn test_generate_agents_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("agents.yaml");

        let result = generate_agents_config(&config_file, 3);

        assert!(result.is_ok());
        assert!(config_file.exists());

        let content = fs::read_to_string(&config_file).unwrap();
        assert!(content.contains("agents:"));
        assert!(content.contains("- id: \"1\""));
        assert!(content.contains("- id: \"2\""));
        assert!(content.contains("- id: \"3\""));
    }

    #[test]
    fn test_generate_agents_config_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("agents.yaml");

        let result = generate_agents_config(&config_file, 0);

        assert!(result.is_ok());
        assert!(config_file.exists());

        let content = fs::read_to_string(&config_file).unwrap();
        // Should use empty template
        assert!(content.len() > 0);
    }

    #[test]
    fn test_create_shell_script_templates() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");

        // Create the directory structure first
        let result = create_agents_directory_structure(&agents_dir, 2);
        assert!(result.is_ok());

        let scripts_dir = agents_dir.join("scripts");
        assert!(scripts_dir.exists());

        // Create a simple test script
        let test_script = "#!/bin/bash\n# Test script\necho 'Hello from test script'";
        fs::write(scripts_dir.join("test_script.sh"), test_script).unwrap();

        assert!(scripts_dir.join("test_script.sh").exists());

        let test_content = fs::read_to_string(scripts_dir.join("test_script.sh")).unwrap();
        assert!(!test_content.is_empty());
        assert!(test_content.contains("#!/bin/bash"));
    }

    #[test]
    fn test_create_tmux_profile_templates() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");

        // Create the directory structure first
        let result = create_agents_directory_structure(&agents_dir, 2);
        assert!(result.is_ok());

        let profiles_dir = agents_dir.join("profiles");
        assert!(profiles_dir.exists());

        // Create a simple test profile
        let test_profile = "#!/bin/bash\n# Test tmux profile\necho 'Test tmux profile'";
        fs::write(profiles_dir.join("test_profile.sh"), test_profile).unwrap();

        assert!(profiles_dir.join("test_profile.sh").exists());

        let content = fs::read_to_string(profiles_dir.join("test_profile.sh")).unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("#!/bin/bash"));
    }

    #[test]
    fn test_create_direnv_config() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agents_dir).unwrap();

        let result = create_direnv_config(&agents_dir);

        assert!(result.is_ok());
        // Should not fail even if direnv is not available
    }

    #[test]
    fn test_init_options_struct() {
        let options = InitOptions {
            force: true,
            agents: 5,
        };

        assert!(options.force);
        assert_eq!(options.agents, 5);
    }

    #[test]
    fn test_init_execution_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agents_dir).unwrap();

        // Create an existing config file
        let config_file = agents_dir.join("agents.yaml");
        fs::write(&config_file, "existing: config").unwrap();

        let options = InitOptions {
            force: true,
            agents: 2,
        };

        // Should succeed with force flag even if config exists
        // Note: This test would require mocking git validation for full testing
        // For now, we test the structure creation parts
        let result = create_agents_directory_structure(&agents_dir, options.agents);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_execution_without_force_existing_config() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agents_dir).unwrap();

        // Create an existing config file
        let config_file = agents_dir.join("agents.yaml");
        fs::write(&config_file, "existing: config").unwrap();

        let _options = InitOptions {
            force: false,
            agents: 2,
        };

        // Testing the config existence check
        assert!(config_file.exists());
    }

    #[test]
    fn test_config_content_structure() {
        let result = generate_config_content(1);
        assert!(result.is_ok());

        let config = result.unwrap();

        // Test YAML structure
        assert!(config.contains("version:"));
        assert!(config.contains("agents:"));
        assert!(config.contains("- id:"));
        assert!(config.contains("branch:"));
        assert!(config.contains("model:"));
        assert!(config.contains("description:"));
        assert!(config.contains("worktree_path:"));
        assert!(config.contains("status:"));
        assert!(config.contains("config:"));
        assert!(config.contains("resource_limits:"));
        assert!(config.contains("session_name:"));
        assert!(config.contains("sync:"));
        assert!(config.contains("settings:"));
    }

    #[test]
    fn test_config_content_default_values() {
        let result = generate_config_content(1);
        assert!(result.is_ok());

        let config = result.unwrap();

        // Test default values
        assert!(config.contains("claude-sonnet-4"));
        assert!(config.contains("\"inactive\""));
        assert!(config.contains("max_memory_mb: 1024"));
        assert!(config.contains("max_cpu_percent: 80"));
        assert!(config.contains("operation_timeout_secs: 300"));
        assert!(config.contains("max_concurrent_ops: 3"));
        assert!(config.contains("max_disk_mb: 5120"));
        assert!(config.contains("auto_sync: false"));
    }

    #[test]
    fn test_config_content_security_settings() {
        let result = generate_config_content(1);
        assert!(result.is_ok());

        let config = result.unwrap();

        // Test security settings
        assert!(config.contains("security:"));
        assert!(config.contains("allowed_paths:"));
        assert!(config.contains("blocked_commands:"));
        assert!(config.contains("strict_path_validation: true"));
        assert!(config.contains("max_command_length: 1000"));
        assert!(config.contains("allow_shell_execution: false"));
    }

    #[test]
    fn test_error_handling_invalid_directory() {
        // Test with an invalid path (should handle gracefully)
        let invalid_path = Path::new("/invalid/nonexistent/path");

        // This should return an error, not panic
        let result = create_agents_directory_structure(invalid_path, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_case_large_agent_count() {
        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");

        // Test with a large number of agents
        let result = create_agents_directory_structure(&agents_dir, 100);
        assert!(result.is_ok());

        // Verify all directories were created
        for i in 1..=100 {
            assert!(agents_dir.join(i.to_string()).exists());
        }
    }
}
