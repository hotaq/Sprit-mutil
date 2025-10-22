//! Unit tests for CLI command argument parsing

use crate::cli::{Commands, AgentsCommands, ConfigCommands};
use clap::Parser;
use sprite::cli::Cli;

#[cfg(test)]
mod cli_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_agents_list_command() {
        let args = vec!["sprite", "agents", "list"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Agents { command: AgentsCommands::List } => {
                // Test passed
            }
            _ => panic!("Expected Agents::List command"),
        }
    }

    #[test]
    fn test_parse_agents_create_command_with_all_options() {
        let args = vec![
            "sprite", "agents", "create", "5",
            "--branch", "feature/test",
            "--workspace", "custom/path",
            "--model", "claude-sonnet-4",
            "--description", "Test agent",
            "--no-workspace"
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Agents {
                command: AgentsCommands::Create {
                    agent_id,
                    branch,
                    workspace,
                    model,
                    description,
                    no_workspace,
                },
            } => {
                assert_eq!(agent_id, "5");
                assert_eq!(branch, Some("feature/test".to_string()));
                assert_eq!(workspace, Some("custom/path".to_string()));
                assert_eq!(model, "claude-sonnet-4");
                assert_eq!(description, Some("Test agent".to_string()));
                assert!(no_workspace);
            }
            _ => panic!("Expected Agents::Create command"),
        }
    }

    #[test]
    fn test_parse_invalid_command() {
        let args = vec!["sprite", "invalid-command"];
        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn test_parse_config_commands() {
        let test_cases = vec![
            (vec!["sprite", "config", "show"], "show"),
            (vec!["sprite", "config", "set", "key", "value"], "set"),
            (vec!["sprite", "config", "get", "key"], "get"),
        ];

        for (args, expected_subcommand) in test_cases {
            let cli = Cli::try_parse_from(args).unwrap();

            match cli.command {
                Commands::Config { command } => {
                    match expected_subcommand {
                        "show" => assert!(matches!(command, ConfigCommands::Show)),
                        "set" => assert!(matches!(command, ConfigCommands::Set { .. })),
                        "get" => assert!(matches!(command, ConfigCommands::Get { .. })),
                        _ => panic!("Unexpected subcommand: {}", expected_subcommand),
                    }
                }
                _ => panic!("Expected Config command"),
            }
        }
    }

    #[test]
    fn test_parse_init_command() {
        let args = vec!["sprite", "init", "--agents", "5", "--force"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Init { force, agents } => {
                assert!(force);
                assert_eq!(agents, 5);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_parse_start_command() {
        let args = vec![
            "sprite", "start",
            "--session-name", "test-session",
            "--layout", "tiled",
            "--detach",
            "--force"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Start {
                session_name,
                layout,
                detach,
                force,
            } => {
                assert_eq!(session_name, Some("test-session".to_string()));
                assert_eq!(layout, "tiled");
                assert!(detach);
                assert!(force);
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_parse_send_command() {
        let args = vec![
            "sprite", "send", "echo", "hello", "world",
            "--timeout", "60",
            "--work-dir", "/tmp",
            "--env", "KEY=VALUE",
            "--sequential"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Send {
                command,
                args,
                timeout,
                work_dir,
                env_vars,
                sequential,
            } => {
                assert_eq!(command, "echo");
                assert_eq!(args, vec!["hello", "world"]);
                assert_eq!(timeout, 60);
                assert_eq!(work_dir, Some("/tmp".to_string()));
                assert_eq!(env_vars, vec!["KEY=VALUE"]);
                assert!(sequential);
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_parse_hey_command() {
        let args = vec![
            "sprite", "hey", "1",
            "pwd",
            "--timeout", "30",
            "--interactive"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Hey {
                agent,
                command,
                args,
                timeout,
                work_dir,
                env_vars,
                interactive,
            } => {
                assert_eq!(agent, "1");
                assert_eq!(command, "pwd");
                assert!(args.is_empty());
                assert_eq!(timeout, 30);
                assert!(work_dir.is_none());
                assert!(env_vars.is_empty());
                assert!(interactive);
            }
            _ => panic!("Expected Hey command"),
        }
    }

    #[test]
    fn test_parse_status_command() {
        let args = vec![
            "sprite", "status",
            "--cleanup",
            "--detailed"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Status {
                session_name,
                cleanup,
                detailed,
            } => {
                assert!(session_name.is_none());
                assert!(cleanup);
                assert!(detailed);
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_parse_kill_command() {
        let args = vec![
            "sprite", "kill",
            "--session-name", "test-session",
            "--force",
            "--all"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Kill {
                session_name,
                force,
                all,
            } => {
                assert_eq!(session_name, Some("test-session".to_string()));
                assert!(force);
                assert!(all);
            }
            _ => panic!("Expected Kill command"),
        }
    }

    #[test]
    fn test_parse_sync_command() {
        let args = vec![
            "sprite", "sync",
            "--agent", "1",
            "--force",
            "--strategy", "auto",
            "--dry-run"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Sync {
                agent,
                force,
                strategy,
                dry_run,
            } => {
                assert_eq!(agent, Some("1".to_string()));
                assert!(force);
                assert_eq!(strategy, "auto");
                assert!(dry_run);
            }
            _ => panic!("Expected Sync command"),
        }
    }

    #[test]
    fn test_parse_warp_command() {
        let args = vec![
            "sprite", "warp",
            "--workspace", "1",
            "--list",
            "--print",
            "--relative"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Warp {
                workspace,
                list,
                print,
                relative,
            } => {
                assert_eq!(workspace, Some("1".to_string()));
                assert!(list);
                assert!(print);
                assert!(relative);
            }
            _ => panic!("Expected Warp command"),
        }
    }

    #[test]
    fn test_parse_zoom_command() {
        let args = vec![
            "sprite", "zoom",
            "--agent", "1",
            "--unzoom",
            "--list"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Zoom {
                agent,
                unzoom,
                list,
            } => {
                assert_eq!(agent, Some("1".to_string()));
                assert!(unzoom);
                assert!(list);
            }
            _ => panic!("Expected Zoom command"),
        }
    }

    #[test]
    fn test_parse_guide_command() {
        let args = vec![
            "sprite", "guide",
            "--command", "init",
            "--search", "agent",
            "--patterns",
            "--troubleshooting",
            "--quick",
            "--accessible",
            "--category", "GettingStarted"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Guide {
                command,
                search,
                patterns,
                troubleshooting,
                quick,
                accessible,
                category,
            } => {
                assert_eq!(command, Some("init".to_string()));
                assert_eq!(search, Some("agent".to_string()));
                assert!(patterns);
                assert!(troubleshooting);
                assert!(quick);
                assert!(accessible);
                assert!(category.is_some());
            }
            _ => panic!("Expected Guide command"),
        }
    }

    #[test]
    fn test_parse_update_command() {
        let args = vec![
            "sprite", "update",
            "--check",
            "--yes",
            "--version", "1.0.0"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Update {
                check,
                yes,
                version,
            } => {
                assert!(check);
                assert!(yes);
                assert_eq!(version, Some("1.0.0".to_string()));
            }
            _ => panic!("Expected Update command"),
        }
    }

    #[test]
    fn test_parse_remove_command() {
        let args = vec![
            "sprite", "remove", "1",
            "--force",
            "--keep-workspace",
            "--merge-branch"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Remove {
                agent,
                force,
                keep_workspace,
                merge_branch,
            } => {
                assert_eq!(agent, "1");
                assert!(force);
                assert!(keep_workspace);
                assert!(merge_branch);
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_parse_attach_command() {
        let args = vec![
            "sprite", "attach",
            "--session-name", "test-session",
            "--list"
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Attach {
                session_name,
                list,
            } => {
                assert_eq!(session_name, Some("test-session".to_string()));
                assert!(list);
            }
            _ => panic!("Expected Attach command"),
        }
    }
}