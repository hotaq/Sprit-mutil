//! Unit tests for configuration management

use anyhow::Result;
use assert_fs::prelude::*;
use serde_yaml;
use sprite::config::SpriteConfig;
use sprite::commands::config::AgentConfig;
use tempfile::TempDir;

#[cfg(test)]
mod config_tests {
    use super::*;

    fn create_test_config() -> SpriteConfig {
        SpriteConfig {
            version: "1.0".to_string(),
            session_name: "test-session".to_string(),
            agents: vec![
                AgentConfig {
                    id: "1".to_string(),
                    branch: "agents/1".to_string(),
                    worktree_path: "agents/1".to_string(),
                    model: "claude-sonnet-4".to_string(),
                    description: "Test agent 1".to_string(),
                    status: "active".to_string(),
                    config: std::collections::HashMap::new(),
                },
            ],
        }
    }

    #[test]
    fn test_save_and_load_config() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("agents.yaml");

        // Create and save config
        let original_config = create_test_config();
        let yaml_content = serde_yaml::to_string(&original_config)?;
        std::fs::write(&config_path, yaml_content)?;

        // Load config
        let loaded_config = SpriteConfig::load_from_path(&config_path)?;

        assert_eq!(loaded_config.version, original_config.version);
        assert_eq!(loaded_config.session_name, original_config.session_name);
        assert_eq!(loaded_config.agents.len(), original_config.agents.len());
        assert_eq!(loaded_config.agents[0].id, original_config.agents[0].id);

        Ok(())
    }

    #[test]
    fn test_config_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("invalid.yaml");

        // Create invalid config (missing required fields)
        let invalid_yaml = r#"
        agents:
          - id: "1"
            # Missing required fields
        "#;

        std::fs::write(&config_path, invalid_yaml)?;

        // Should handle invalid config gracefully
        let result = SpriteConfig::load_from_path(&config_path);
        assert!(result.is_err() || result.unwrap().agents.is_empty());

        Ok(())
    }

    #[test]
    fn test_config_update_agent() -> Result<()> {
        let mut config = create_test_config();

        // Add new agent
        let new_agent = AgentConfig {
            id: "2".to_string(),
            branch: "agents/2".to_string(),
            worktree_path: "agents/2".to_string(),
            model: "gpt-4".to_string(),
            description: "Test agent 2".to_string(),
            status: "inactive".to_string(),
            config: std::collections::HashMap::new(),
        };

        config.agents.push(new_agent);
        assert_eq!(config.agents.len(), 2);
        assert_eq!(config.agents[1].id, "2");

        // Update existing agent
        config.agents[0].status = "updated".to_string();
        assert_eq!(config.agents[0].status, "updated");

        Ok(())
    }

    #[test]
    fn test_config_with_multiple_agents() -> Result<()> {
        let mut config = SpriteConfig {
            version: "1.0".to_string(),
            session_name: "multi-agent-session".to_string(),
            agents: Vec::new(),
        };

        // Add multiple agents
        for i in 1..=5 {
            let agent = AgentConfig {
                id: i.to_string(),
                branch: format!("agents/{}", i),
                worktree_path: format!("agents/{}", i),
                model: match i {
                    1 => "claude-sonnet-4".to_string(),
                    2 => "gpt-4".to_string(),
                    3 => "claude-haiku".to_string(),
                    4 => "gpt-3.5-turbo".to_string(),
                    _ => "claude-sonnet-4".to_string(),
                },
                description: format!("Test agent {}", i),
                status: "active".to_string(),
                config: std::collections::HashMap::new(),
            };
            config.agents.push(agent);
        }

        assert_eq!(config.agents.len(), 5);

        // Test serialization and deserialization
        let yaml_content = serde_yaml::to_string(&config)?;
        let loaded_config: SpriteConfig = serde_yaml::from_str(&yaml_content)?;

        assert_eq!(loaded_config.agents.len(), 5);
        assert_eq!(loaded_config.version, config.version);
        assert_eq!(loaded_config.session_name, config.session_name);

        for i in 0..5 {
            assert_eq!(loaded_config.agents[i].id, config.agents[i].id);
            assert_eq!(loaded_config.agents[i].branch, config.agents[i].branch);
            assert_eq!(loaded_config.agents[i].model, config.agents[i].model);
        }

        Ok(())
    }

    #[test]
    fn test_config_agent_properties() -> Result<()> {
        let mut config = std::collections::HashMap::new();
        config.insert("timeout".to_string(), "30".to_string());
        config.insert("retries".to_string(), "3".to_string());

        let agent = AgentConfig {
            id: "test-agent".to_string(),
            branch: "feature/test".to_string(),
            worktree_path: "/tmp/test-agent".to_string(),
            model: "claude-sonnet-4".to_string(),
            description: "Test agent with custom config".to_string(),
            status: "active".to_string(),
            config: config.clone(),
        };

        assert_eq!(agent.id, "test-agent");
        assert_eq!(agent.branch, "feature/test");
        assert_eq!(agent.worktree_path, "/tmp/test-agent");
        assert_eq!(agent.model, "claude-sonnet-4");
        assert_eq!(agent.description, "Test agent with custom config");
        assert_eq!(agent.status, "active");
        assert_eq!(agent.config.len(), 2);
        assert_eq!(agent.config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(agent.config.get("retries"), Some(&"3".to_string()));

        // Test in full config
        let full_config = SpriteConfig {
            version: "1.0".to_string(),
            session_name: "test-session".to_string(),
            agents: vec![agent],
        };

        let yaml_content = serde_yaml::to_string(&full_config)?;
        let loaded_config: SpriteConfig = serde_yaml::from_str(&yaml_content)?;

        assert_eq!(loaded_config.agents.len(), 1);
        let loaded_agent = &loaded_config.agents[0];
        assert_eq!(loaded_agent.config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(loaded_agent.config.get("retries"), Some(&"3".to_string()));

        Ok(())
    }

    #[test]
    fn test_config_empty_agents() -> Result<()> {
        let config = SpriteConfig {
            version: "1.0".to_string(),
            session_name: "empty-session".to_string(),
            agents: Vec::new(),
        };

        assert_eq!(config.agents.len(), 0);

        // Test serialization
        let yaml_content = serde_yaml::to_string(&config)?;
        let loaded_config: SpriteConfig = serde_yaml::from_str(&yaml_content)?;

        assert_eq!(loaded_config.agents.len(), 0);
        assert_eq!(loaded_config.version, "1.0");
        assert_eq!(loaded_config.session_name, "empty-session");

        Ok(())
    }

    #[test]
    fn test_config_edge_cases() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("edge_case.yaml");

        // Test config with special characters in description
        let config_with_special_chars = SpriteConfig {
            version: "1.0".to_string(),
            session_name: "test-session-with-dashes".to_string(),
            agents: vec![
                AgentConfig {
                    id: "agent-1".to_string(),
                    branch: "feature/test-with-special_chars-@#$".to_string(),
                    worktree_path: "/path/with spaces and symbols/test".to_string(),
                    model: "claude-sonnet-4".to_string(),
                    description: "Agent with special chars: @#$%^&*()_+-=[]{}|;':\",./<>?".to_string(),
                    status: "active".to_string(),
                    config: std::collections::HashMap::new(),
                },
            ],
        };

        let yaml_content = serde_yaml::to_string(&config_with_special_chars)?;
        std::fs::write(&config_path, yaml_content)?;

        let loaded_config = SpriteConfig::load_from_path(&config_path)?;
        assert_eq!(loaded_config.agents.len(), 1);
        assert_eq!(loaded_config.agents[0].id, "agent-1");
        assert!(loaded_config.agents[0].description.contains("@#$%^&*()"));

        Ok(())
    }

    #[test]
    fn test_config_file_not_found() {
        let non_existent_path = "/tmp/non/existent/path/config.yaml";
        let result = SpriteConfig::load_from_path(non_existent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_malformed_yaml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("malformed.yaml");

        // Create malformed YAML
        let malformed_yaml = r#"
        version: "1.0"
        session_name: "test"
        agents:
          - id: "1"
            branch: "test"
        # Missing closing bracket for agents array
        "#;

        std::fs::write(&config_path, malformed_yaml)?;

        let result = SpriteConfig::load_from_path(&config_path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_config_default_values() {
        // Test creating config with default values
        let config = SpriteConfig::default();

        // Assuming default implementation provides sensible defaults
        // This test should be adapted based on the actual default implementation
        assert!(!config.version.is_empty());
        assert!(!config.session_name.is_empty());
        // agents should probably be empty by default
        assert_eq!(config.agents.len(), 0);
    }
}