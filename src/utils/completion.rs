//! Command completion and suggestion system for Sprite.
//!
//! This module provides intelligent command completion, argument suggestions,
//! and auto-completion helpers for enhanced CLI usability.

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Command completion and suggestion system.
#[derive(Debug, Clone)]
pub struct CompletionSystem {
    /// Command registry
    pub commands: HashMap<String, CommandInfo>,
    /// Context-aware suggestions
    pub context_suggestions: Vec<ContextSuggestion>,
    /// Common argument patterns
    pub argument_patterns: HashMap<String, Vec<ArgumentPattern>>,
    /// File system completions
    pub file_completions: FileCompletionOptions,
}

/// Information about a command for completion.
#[derive(Debug, Clone)]
pub struct CommandInfo {
    /// Command name
    pub name: String,
    /// Subcommands
    pub subcommands: Vec<String>,
    /// Required arguments
    pub required_args: Vec<ArgumentInfo>,
    /// Optional arguments
    pub optional_args: Vec<ArgumentInfo>,
    /// Command aliases
    pub aliases: Vec<String>,
    /// Command description
    pub description: String,
    /// Usage examples
    pub examples: Vec<String>,
}

/// Information about a command argument.
#[derive(Debug, Clone)]
pub struct ArgumentInfo {
    /// Argument name
    pub name: String,
    /// Short flag (e.g., "f" for --force)
    pub short_flag: Option<String>,
    /// Long flag (e.g., "force")
    pub long_flag: String,
    /// Argument description
    pub description: String,
    /// Expected values
    pub values: Vec<String>,
    /// Takes a value
    pub takes_value: bool,
    /// Whether it's a boolean flag
    pub is_flag: bool,
    /// Completion provider
    pub completion_provider: Option<CompletionProvider>,
}

/// Provider for generating completions.
#[derive(Debug, Clone)]
pub enum CompletionProvider {
    /// File system completion
    Files(FileCompletionType),
    /// Directory completion
    Directories,
    /// Command completion
    Commands,
    /// Agent completion
    Agents,
    /// Workspace completion
    Workspaces,
    /// Custom completion function
    Custom(String),
    /// Fixed values
    Values(Vec<String>),
}

/// Type of file system completion.
#[derive(Debug, Clone)]
pub enum FileCompletionType {
    /// All files
    All,
    /// Specific extensions
    Extensions(Vec<String>),
    /// Directories only
    Directories,
    /// Executable files
    Executables,
}

/// File system completion options.
#[derive(Debug, Clone, Default)]
pub struct FileCompletionOptions {
    /// Include hidden files
    pub include_hidden: bool,
    /// Follow symbolic links
    pub follow_symlinks: bool,
    /// Maximum depth
    pub max_depth: Option<usize>,
    /// File extensions filter
    pub extensions: Vec<String>,
}

/// Context-aware suggestion.
#[derive(Debug, Clone)]
pub struct ContextSuggestion {
    /// Suggestion pattern
    pub pattern: String,
    /// Suggestion text
    pub suggestion: String,
    /// Description
    pub description: String,
    /// Context where this applies
    pub context: String,
    /// Priority (lower = higher priority)
    pub priority: u8,
}

/// Argument completion pattern.
#[derive(Debug, Clone)]
pub struct ArgumentPattern {
    /// Argument name
    pub argument: String,
    /// Pattern to match
    pub pattern: String,
    /// Completions for this pattern
    pub completions: Vec<String>,
    /// Description
    pub description: String,
}

/// Completion result.
#[derive(Debug, Clone)]
pub struct CompletionResult {
    /// Suggested completions
    pub suggestions: Vec<String>,
    /// Current input
    pub input: String,
    /// Cursor position
    pub cursor_pos: usize,
    /// Command context
    pub command_context: Option<String>,
}

/// Smart suggestion based on context.
#[derive(Debug, Clone)]
pub struct SmartSuggestion {
    /// Suggestion text
    pub text: String,
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Description
    pub description: String,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Context
    pub context: String,
}

/// Type of suggestion.
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    /// Command suggestion
    Command,
    /// Argument suggestion
    Argument,
    /// Value suggestion
    Value,
    /// File suggestion
    File,
    /// Pattern suggestion
    Pattern,
    /// Correction suggestion
    Correction,
}

impl CompletionSystem {
    /// Create a new completion system.
    pub fn new() -> Self {
        let mut system = Self {
            commands: HashMap::new(),
            context_suggestions: Vec::new(),
            argument_patterns: HashMap::new(),
            file_completions: FileCompletionOptions::default(),
        };

        system.load_command_info();
        system.load_context_suggestions();
        system.load_argument_patterns();

        system
    }

    /// Load command information.
    fn load_command_info(&mut self) {
        // Core commands
        self.commands.insert("init".to_string(), CommandInfo {
            name: "init".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "agents".to_string(),
                    short_flag: Some("a".to_string()),
                    long_flag: "agents".to_string(),
                    description: "Number of agents to create".to_string(),
                    values: vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string()],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Values(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string()])),
                },
                ArgumentInfo {
                    name: "force".to_string(),
                    short_flag: Some("f".to_string()),
                    long_flag: "force".to_string(),
                    description: "Force overwrite existing configuration".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Initialize a new multi-agent environment".to_string(),
            examples: vec![
                "sprite init".to_string(),
                "sprite init --agents 5".to_string(),
                "sprite init --force".to_string(),
            ],
        });

        self.commands.insert("start".to_string(), CommandInfo {
            name: "start".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "session-name".to_string(),
                    short_flag: Some("s".to_string()),
                    long_flag: "session-name".to_string(),
                    description: "Custom session name".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("session_name".to_string())),
                },
                ArgumentInfo {
                    name: "layout".to_string(),
                    short_flag: Some("l".to_string()),
                    long_flag: "layout".to_string(),
                    description: "Layout type".to_string(),
                    values: vec!["tiled".to_string(), "main-vertical".to_string(), "main-horizontal".to_string(), "even-horizontal".to_string()],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Values(vec!["tiled".to_string(), "main-vertical".to_string(), "main-horizontal".to_string(), "even-horizontal".to_string()])),
                },
                ArgumentInfo {
                    name: "detach".to_string(),
                    short_flag: Some("d".to_string()),
                    long_flag: "detach".to_string(),
                    description: "Start and detach immediately".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Start a new multi-agent supervision session".to_string(),
            examples: vec![
                "sprite start".to_string(),
                "sprite start --session-name dev".to_string(),
                "sprite start --layout main-vertical".to_string(),
                "sprite start --detach".to_string(),
            ],
        });

        self.commands.insert("agents".to_string(), CommandInfo {
            name: "agents".to_string(),
            subcommands: vec!["list".to_string(), "create".to_string(), "remove".to_string(), "show".to_string(), "validate".to_string(), "provision".to_string()],
            required_args: vec![],
            optional_args: vec![],
            aliases: vec![],
            description: "Manage agents".to_string(),
            examples: vec![
                "sprite agents list".to_string(),
                "sprite agents create 4".to_string(),
                "sprite agents remove 2".to_string(),
            ],
        });

        self.commands.insert("warp".to_string(), CommandInfo {
            name: "warp".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "list".to_string(),
                    short_flag: Some("l".to_string()),
                    long_flag: "list".to_string(),
                    description: "List available workspaces".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "print".to_string(),
                    long_flag: "print".to_string(),
                    description: "Print workspace path without changing".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "relative".to_string(),
                    long_flag: "relative".to_string(),
                    description: "Show relative path".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec!["cd".to_string()],
            description: "Navigate between agent workspaces".to_string(),
            examples: vec![
                "sprite warp".to_string(),
                "sprite warp --list".to_string(),
                "sprite warp 1".to_string(),
                "sprite warp main".to_string(),
            ],
        });

        self.commands.insert("hey".to_string(), CommandInfo {
            name: "hey".to_string(),
            subcommands: vec![],
            required_args: vec![
                ArgumentInfo {
                    name: "agent".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Agent number".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Agents),
                },
                ArgumentInfo {
                    name: "command".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Command to execute".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Commands),
                },
            ],
            optional_args: vec![
                ArgumentInfo {
                    name: "args".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Command arguments".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("command_args".to_string())),
                },
                ArgumentInfo {
                    name: "timeout".to_string(),
                    short_flag: Some("t".to_string()),
                    long_flag: "timeout".to_string(),
                    description: "Command timeout in seconds".to_string(),
                    values: vec!["10".to_string(), "30".to_string(), "60".to_string(), "120".to_string(), "300".to_string()],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Values(vec!["10".to_string(), "30".to_string(), "60".to_string(), "120".to_string(), "300".to_string()])),
                },
                ArgumentInfo {
                    name: "work-dir".to_string(),
                    short_flag: Some("w".to_string()),
                    long_flag: "work-dir".to_string(),
                    description: "Working directory for command".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Directories),
                },
                ArgumentInfo {
                    name: "env".to_string(),
                    short_flag: None,
                    long_flag: "env".to_string(),
                    description: "Environment variable (KEY=VALUE)".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("env_var".to_string())),
                },
                ArgumentInfo {
                    name: "interactive".to_string(),
                    short_flag: Some("i".to_string()),
                    long_flag: "interactive".to_string(),
                    description: "Interactive mode".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Send a command to a specific agent".to_string(),
            examples: vec![
                "sprite hey 1 npm test".to_string(),
                "sprite hey 2 'git status'".to_string(),
                "sprite hey 3 'npm run dev'".to_string(),
            ],
        });

        self.commands.insert("send".to_string(), CommandInfo {
            name: "send".to_string(),
            subcommands: vec![],
            required_args: vec![
                ArgumentInfo {
                    name: "command".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Command to execute".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Commands),
                },
            ],
            optional_args: vec![
                ArgumentInfo {
                    name: "args".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Command arguments".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("command_args".to_string())),
                },
                ArgumentInfo {
                    name: "timeout".to_string(),
                    short_flag: Some("t".to_string()),
                    long_flag: "timeout".to_string(),
                    description: "Command timeout in seconds".to_string(),
                    values: vec!["10".to_string(), "30".to_string(), "60".to_string(), "120".to_string(), "300".to_string()],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Values(vec!["10".to_string(), "30".to_string(), "60".to_string(), "120".to_string(), "300".to_string()])),
                },
                ArgumentInfo {
                    name: "work-dir".to_string(),
                    short_flag: Some("w".to_string()),
                    long_flag: "work-dir".to_string(),
                    description: "Working directory for command".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Directories),
                },
                ArgumentInfo {
                    name: "env".to_string(),
                    short_flag: None,
                    long_flag: "env".to_string(),
                    description: "Environment variable (KEY=VALUE)".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("env_var".to_string())),
                },
                ArgumentInfo {
                    name: "sequential".to_string(),
                    long_flag: "sequential".to_string(),
                    description: "Execute sequentially".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Send a command to all agents".to_string(),
            examples: vec![
                "sprite send 'git pull'".to_string(),
                "sprite send 'npm install' --sequential".to_string(),
                "sprite send 'cargo build'".to_string(),
            ],
        });

        self.commands.insert("sync".to_string(), CommandInfo {
            name: "sync".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "agent".to_string(),
                    short_flag: Some("a".to_string()),
                    long_flag: "agent".to_string(),
                    description: "Sync specific agent".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Agents),
                },
                ArgumentInfo {
                    name: "force".to_string(),
                    short_flag: Some("f".to_string()),
                    long_flag: "force".to_string(),
                    description: "Force sync even if conflicts".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "strategy".to_string(),
                    short_flag: Some("s".to_string()),
                    long_flag: "strategy".to_string(),
                    description: "Conflict resolution strategy".to_string(),
                    values: vec!["manual".to_string(), "auto-theirs".to_string(), "auto-ours".to_string(), "auto-merge".to_string()],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Values(vec!["manual".to_string(), "auto-theirs".to_string(), "auto-ours".to_string(), "auto-merge".to_string()])),
                },
                ArgumentInfo {
                    name: "dry-run".to_string(),
                    long_flag: "dry-run".to_string(),
                    description: "Show what would be synced".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Synchronize workspaces context-aware".to_string(),
            examples: vec![
                "sprite sync".to_string(),
                "sprite sync --agent 2".to_string(),
                "sprite sync --force".to_string(),
            ],
        });

        self.commands.insert("remove".to_string(), CommandInfo {
            name: "remove".to_string(),
            subcommands: vec![],
            required_args: vec![
                ArgumentInfo {
                    name: "agent".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Agent number to remove".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Agents),
                },
            ],
            optional_args: vec![
                ArgumentInfo {
                    name: "force".to_string(),
                    short_flag: Some("f".to_string()),
                    long_flag: "force".to_string(),
                    description: "Force removal without confirmation".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "keep-workspace".to_string(),
                    long_flag: "keep-workspace".to_string(),
                    description: "Keep workspace files".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "merge-branch".to_string(),
                    long_flag: "merge-branch".to_string(),
                    description: "Merge branch before removal".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Remove agent workspace".to_string(),
            examples: vec![
                "sprite remove 1".to_string(),
                "sprite remove 2 --force".to_string(),
                "sprite remove 3 --merge-branch".to_string(),
            ],
        });

        self.commands.insert("zoom".to_string(), CommandInfo {
            name: "zoom".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "agent".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Agent number to zoom on".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Agents),
                },
                ArgumentInfo {
                    name: "unzoom".to_string(),
                    short_flag: Some("u".to_string()),
                    long_flag: "unzoom".to_string(),
                    description: "Return to normal layout".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "list".to_string(),
                    short_flag: Some("l".to_string()),
                    long_flag: "list".to_string(),
                    description: "List available panes".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Focus on agent pane".to_string(),
            examples: vec![
                "sprite zoom 1".to_string(),
                "sprite zoom --unzoom".to_string(),
                "sprite zoom --list".to_string(),
            ],
        });

        self.commands.insert("status".to_string(), CommandInfo {
            name: "status".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "session_name".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Check specific session".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("session_name".to_string())),
                },
                ArgumentInfo {
                    name: "cleanup".to_string(),
                    long_flag: "cleanup".to_string(),
                    description: "Clean up old sessions and temporary files".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "detailed".to_string(),
                    long_flag: "detailed".to_string(),
                    description: "Show detailed health information".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Check session status and health".to_string(),
            examples: vec![
                "sprite status".to_string(),
                "sprite status --detailed".to_string(),
                "sprite status --cleanup".to_string(),
            ],
        });

        self.commands.insert("help".to_string(), CommandInfo {
            name: "help".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "command".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Command to get help for".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Commands),
                },
                ArgumentInfo {
                    name: "search".to_string(),
                    long_flag: "search".to_string(),
                    description: "Search help content".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("search_term".to_string())),
                },
                ArgumentInfo {
                    name: "patterns".to_string(),
                    long_flag: "patterns".to_string(),
                    description: "Show usage patterns".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "troubleshooting".to_string(),
                    long_flag: "troubleshooting".to_string(),
                    description: "Show troubleshooting guides".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "quick".to_string(),
                    long_flag: "quick".to_string(),
                    description: "Show quick reference".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "accessible".to_string(),
                    long_flag: "accessible".to_string(),
                    description: "Show accessible help (screen reader friendly)".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "category".to_string(),
                    long_flag: "category".to_string(),
                    description: "Show usage patterns for specific category".to_string(),
                    values: vec!["getting-started".to_string(), "daily-workflow".to_string(), "troubleshooting".to_string(), "advanced".to_string(), "collaboration".to_string()],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Values(vec!["getting-started".to_string(), "daily-workflow".to_string(), "troubleshooting".to_string(), "advanced".to_string(), "collaboration".to_string()])),
                },
            ],
            aliases: vec![],
            description: "Show help and documentation".to_string(),
            examples: vec![
                "sprite help".to_string(),
                "sprite help init".to_string(),
                "sprite help --search sync".to_string(),
                "sprite help --patterns".to_string(),
            ],
        });

        self.commands.insert("config".to_string(), CommandInfo {
            name: "config".to_string(),
            subcommands: vec!["show".to_string(), "validate".to_string(), "set".to_string(), "get".to_string(), "edit".to_string(), "status".to_string()],
            required_args: vec![],
            optional_args: vec![],
            aliases: vec![],
            description: "Manage configuration settings".to_string(),
            examples: vec![
                "sprite config show".to_string(),
                "sprite config validate".to_string(),
                "sprite config set timeout 60".to_string(),
            ],
        });

        self.commands.insert("kill".to_string(), CommandInfo {
            name: "kill".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "session_name".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Session name to kill".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("session_name".to_string())),
                },
                ArgumentInfo {
                    name: "force".to_string(),
                    short_flag: Some("f".to_string()),
                    long_flag: "force".to_string(),
                    description: "Force kill without confirmation".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
                ArgumentInfo {
                    name: "all".to_string(),
                    long_flag: "all".to_string(),
                    description: "Kill all sessions".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Terminate session".to_string(),
            examples: vec![
                "sprite kill".to_string(),
                "sprite kill --force".to_string(),
                "sprite kill --all".to_string(),
            ],
        });

        self.commands.insert("attach".to_string(), CommandInfo {
            name: "attach".to_string(),
            subcommands: vec![],
            required_args: vec![],
            optional_args: vec![
                ArgumentInfo {
                    name: "session_name".to_string(),
                    short_flag: None,
                    long_flag: "".to_string(),
                    description: "Session name to attach to".to_string(),
                    values: vec![],
                    takes_value: true,
                    is_flag: false,
                    completion_provider: Some(CompletionProvider::Custom("session_name".to_string())),
                },
                ArgumentInfo {
                    name: "list".to_string(),
                    short_flag: Some("l".to_string()),
                    long_flag: "list".to_string(),
                    description: "List available sessions".to_string(),
                    values: vec![],
                    takes_value: false,
                    is_flag: true,
                    completion_provider: None,
                },
            ],
            aliases: vec![],
            description: "Attach to existing session".to_string(),
            examples: vec![
                "sprite attach".to_string(),
                "sprite attach my-session".to_string(),
                "sprite attach --list".to_string(),
            ],
        });
    }

    /// Load context-aware suggestions.
    fn load_context_suggestions(&mut self) {
        self.context_suggestions = vec![
            ContextSuggestion {
                pattern: "init.*".to_string(),
                suggestion: " --agents 3".to_string(),
                description: "Create 3 agents (default)".to_string(),
                context: "sprite init".to_string(),
                priority: 10,
            },
            ContextSuggestion {
                pattern: "start.*".to_string(),
                suggestion: " --session-name dev".to_string(),
                description: "Use descriptive session name".to_string(),
                context: "sprite start".to_string(),
                priority: 8,
            },
            ContextSuggestion {
                pattern: "hey.*".to_string(),
                suggestion: " 'npm test'".to_string(),
                description: "Common test command".to_string(),
                context: "sprite hey".to_string(),
                priority: 7,
            },
            ContextSuggestion {
                pattern: "send.*".to_string(),
                suggestion: " 'git pull'".to_string(),
                description: "Sync all agents".to_string(),
                context: "sprite send".to_string(),
                priority: 7,
            },
            ContextSuggestion {
                pattern: "warp.*".to_string(),
                suggestion: " --list".to_string(),
                description: "List available workspaces".to_string(),
                context: "sprite warp".to_string(),
                priority: 6,
            },
        ];
    }

    /// Load argument completion patterns.
    fn load_argument_patterns(&mut self) {
        self.argument_patterns.insert("timeout".to_string(), vec![
            ArgumentPattern {
                argument: "timeout".to_string(),
                pattern: r"\d+$".to_string(),
                completions: vec!["30".to_string(), "60".to_string(), "120".to_string()],
                description: "Common timeout values".to_string(),
            },
        ]);

        self.argument_patterns.insert("layout".to_string(), vec![
            ArgumentPattern {
                argument: "layout".to_string(),
                pattern: r".+$".to_string(),
                completions: vec!["tiled".to_string(), "main-vertical".to_string(), "main-horizontal".to_string()],
                description: "Common layout types".to_string(),
            },
        ]);

        self.argument_patterns.insert("env".to_string(), vec![
            ArgumentPattern {
                argument: "env".to_string(),
                pattern: r"^[A-Z_][A-Z0-9_]*=".to_string(),
                completions: vec![
                    "NODE_ENV=development".to_string(),
                    "NODE_ENV=production".to_string(),
                    "RUST_LOG=debug".to_string(),
                    "RUST_LOG=info".to_string(),
                ],
                description: "Common environment variables".to_string(),
            },
        ]);
    }

    /// Get command completions for a given input.
    pub fn get_command_completions(&self, input: &str) -> Vec<String> {
        let input_lower = input.to_lowercase();

        self.commands
            .keys()
            .filter(|cmd| cmd.starts_with(&input_lower))
            .cloned()
            .collect()
    }

    /// Get argument completions for a command.
    pub fn get_argument_completions(&self, command: &str, arg_input: &str) -> Vec<String> {
        if let Some(cmd_info) = self.commands.get(command) {
            let mut completions = Vec::new();

            // Check exact argument matches
            for arg in &cmd_info.optional_args {
                if arg.long_flag.starts_with(arg_input) {
                    completions.push(arg.long_flag.clone());
                } else if let Some(short) = &arg.short_flag {
                    if *short == arg_input {
                        completions.push(format!("-{}", short));
                    }
                }
            }

            // Add values for arguments that take values
            for arg in &cmd_info.optional_args {
                if arg.takes_value && (arg.long_flag == arg_input || arg.short_flag.as_ref().map_or(false, |s| s == arg_input)) {
                    if let Some(provider) = &arg.completion_provider {
                        completions.extend(self.get_provider_completions(provider, arg_input));
                    }
                }
            }

            // Check pattern-based completions
            if let Some(patterns) = self.argument_patterns.get(arg_input) {
                for pattern in patterns {
                    completions.extend(pattern.completions.clone());
                }
            }

            completions.sort();
            completions.dedup();
            completions
        } else {
            Vec::new()
        }
    }

    /// Get completions from a completion provider.
    pub fn get_provider_completions(&self, provider: &CompletionProvider, input: &str) -> Vec<String> {
        match provider {
            CompletionProvider::Values(values) => values.clone(),
            CompletionProvider::Files(file_type) => self.get_file_completions(file_type, input),
            CompletionProvider::Directories => self.get_directory_completions(input),
            CompletionProvider::Commands => self.get_command_completions(input),
            CompletionProvider::Agents => self.get_agent_completions(),
            CompletionProvider::Workspaces => self.get_workspace_completions(),
            CompletionProvider::Custom(custom) => self.get_custom_completions(custom, input),
        }
    }

    /// Get file completions.
    fn get_file_completions(&self, file_type: &FileCompletionType, input: &str) -> Vec<String> {
        // This is a simplified implementation
        // In a real implementation, this would use the file system
        match file_type {
            FileCompletionType::All => vec!["*.rs".to_string(), "*.md".to_string(), "*.txt".to_string(), "*.yaml".to_string()],
            FileCompletionType::Extensions(exts) => exts.iter().map(|e| format!("*.{}", e)).collect(),
            FileCompletionType::Directories => vec![],
            FileCompletionType::Executables => vec![],
        }
    }

    /// Get directory completions.
    fn get_directory_completions(&self, _input: &str) -> Vec<String> {
        // This would scan directories in the current path
        vec!["agents/".to_string(), "src/".to_string(), "tests/".to_string()]
    }

    /// Get command completions.
    fn get_command_completions(&self, input: &str) -> Vec<String> {
        self.get_command_completions(input)
    }

    /// Get agent completions.
    fn get_agent_completions(&self) -> Vec<String> {
        // This would query the current configuration for agent IDs
        vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string()]
    }

    /// Get workspace completions.
    fn get_workspace_completions(&self) -> Vec<String> {
        // This would query available workspaces
        vec!["main".to_string(), "1".to_string(), "2".to_string(), "3".to_string()]
    }

    /// Get custom completions.
    fn get_custom_completions(&self, custom: &str, input: &str) -> Vec<String> {
        match custom {
            "session_name" => self.get_session_name_completions(),
            "agent" => self.get_agent_completions(),
            "workspace" => self.get_workspace_completions(),
            "command" => self.get_command_completions(input),
            "command_args" => self.get_command_args_completions(input),
            "env_var" => self.get_env_var_completions(),
            "search_term" => self.get_search_term_completions(input),
            _ => Vec::new(),
        }
    }

    /// Get session name completions.
    fn get_session_name_completions(&self) -> Vec<String> {
        // This would query tmux for existing sessions
        vec!["sprite-session".to_string(), "dev-session".to_string(), "test-session".to_string()]
    }

    /// Get command argument completions.
    fn get_command_args_completions(&self, _input: &str) -> Vec<String> {
        vec!["'npm test'".to_string(), "'git status'".to_string(), "'cargo build'".to_string()]
    }

    /// Get environment variable completions.
    fn get_env_var_completions(&self) -> Vec<String> {
        vec!["NODE_ENV=".to_string(), "RUST_LOG=".to_string(), "PATH=".to_string()]
    }

    /// Get search term completions.
    fn get_search_term_completions(&self, input: &str) -> Vec<String> {
        // This would provide suggestions based on common search terms
        vec!["sync".to_string(), "agent".to_string(), "workspace".to_string(), "troubleshoot".to_string()]
    }

    /// Get smart suggestions based on context.
    pub fn get_smart_suggestions(&self, input: &str, context: &str) -> Vec<SmartSuggestion> {
        let mut suggestions = Vec::new();

        // Check for typos in command names
        for cmd in self.commands.keys() {
            let distance = levenshtein_distance(input, cmd);
            if distance > 0 && distance <= 2 && distance < cmd.len() / 2 {
                suggestions.push(SmartSuggestion {
                    text: cmd.clone(),
                    suggestion_type: SuggestionType::Correction,
                    description: format!("Did you mean '{}'?", cmd),
                    confidence: (100 - (distance * 30)) as u8,
                    context: context.to_string(),
                });
            }
        }

        // Add context-aware suggestions
        for context_suggestion in &self.context_suggestions {
            if context_suggestion.context == context && input.matches(&context_suggestion.pattern) {
                suggestions.push(SmartSuggestion {
                    text: context_suggestion.suggestion.clone(),
                    suggestion_type: SuggestionType::Pattern,
                    description: context_suggestion.description.clone(),
                    confidence: context_suggestion.priority,
                    context: context_suggestion.context.clone(),
                });
            }
        }

        suggestions.sort_by(|a, b| b.confidence.cmp(&a.confidence));
        suggestions
    }

    /// Generate bash completion script.
    pub fn generate_bash_completion(&self) -> String {
        let commands: Vec<&String> = self.commands.keys().collect();
        let command_list = commands.join(" ");

        format!(
            r#"#!/bin/bash
# Sprite bash completion script

_sprite_completion() {{
    local cur prev words cword
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}"
    words=("${{COMP_WORDS[@]}}")

    case "$prev" in
        sprite)
            COMPREPLY=($(compgen -W "{}" -- "$cur"))
            ;;
        agents)
            COMPREPLY=($(compgen -W "list create remove show validate provision" -- "$cur"))
            ;;
        config)
            COMPREPLY=($(compgen -W "show validate set get edit status" -- "$cur"))
            ;;
        init|start|kill|attach|send|hey|sync|remove|warp|zoom|status|help)
            # Command-specific completions would go here
            ;;
        warp)
            if [[ $cur == -* ]]; then
                COMPREPLY=($(compgen -W "list print relative" -- "${{cur#-}}"))
            else
                COMPREPLY=($(compgen -W "$(sprite warp --list 2>/dev/null | grep '📁' | awk '{{print $3}}' | sed 's/://')" -- "$cur"))
            fi
            ;;
        hey|send)
            if [[ $cur == -* ]]; then
                COMPREPLY=($(compgen -W "timeout work-dir env" -- "${{cur#-}}"))
            else
                # Agent completion
                COMPREPLY=($(compgen -W "$(sprite agents list 2>/dev/null | grep -E '^[0-9]+' | awk '{{print $1}}')" -- "$cur"))
            fi
            ;;
        sync)
            if [[ $cur == -* ]]; then
                COMPREPLY=($(compgen -W "agent force strategy dry-run" -- "${{cur#-}}"))
            else
                COMPREPLY=($(compgen -W "$(sprite agents list 2>/dev/null | grep -E '^[0-9]+' | awk '{{print $1}}')" -- "$cur"))
            fi
            ;;
    esac
}}

complete -F _sprite_completion sprite
"#,
            command_list
        )
    }

    /// Generate fish completion script.
    pub fn generate_fish_completion(&self) -> String {
        format!(
            r#"# Fish completion for Sprite

complete -c sprite -f
complete -c sprite -n sprite -a -x 'sprite --help'

# Command completions
complete -c sprite -n '__fish_sprite_complete'

function __fish_sprite_complete
    set -l cmd (commandline -poc)
    set -l cmd (commandline -opc)

    switch $cmd[1]
        case init
            complete -c sprite -n '__fish_sprite_complete_init'
            ;;
        case agents
            complete -c sprite -n '__fish_sprite_complete_agents'
            ;;
        case config
            complete -c sprite -n '__fish_sprite_complete_config'
            ;;
        case start
            complete -c sprite -n '__fish_complete_start'
            ;;
        case warp
            complete -c sprite -n '__fish_complete_warp'
            ;;
        case hey
            complete -c sprite -n '__fish_complete_hey'
            ;;
        case send
            complete -c sprite -n '__fish_complete_send'
            ;;
        case sync
            complete -c sprite -n '__fish_complete_sync'
            ;;
        case remove
            complete -c sprite -n '__fish_complete_remove'
            ;;
        case zoom
            complete -c sprite -n '__fish_complete_zoom'
            ;;
        case status
            complete -c sprite -n '__fish_complete_status'
            ;;
        case help
            complete -c sprite -n '__fish_complete_help'
            ;;
    end
end

function __fish_sprite_complete_init
    complete -c sprite -f -a 'force' -d 'Description: Force overwrite existing configuration'
end

function __fish_complete_agents
    complete -c sprite -f -a 'list' -d 'Description: List all configured agents'
    complete -c sprite -f -a 'create' -d 'Description: Create a new agent'
    complete -c sprite -f -a 'remove' -d 'Description: Remove an agent'
    complete -c sprite -f -a 'show' -d 'Description: Show agent details'
    complete -c sprite -f -a 'validate' -d 'Description: Validate agent workspaces'
    complete -c sprite -f -a 'provision' -d 'Description: Provision workspaces for agents'
end

function __fish_complete_config
    complete -c sprite -f -a 'show' -d 'Description: Show current configuration'
    complete -c sprite -f -a 'validate' -d 'Description: Validate configuration'
    complete -c sprite -f -a 'set' -d 'Description: Set configuration value'
    complete -c sprite -f -a 'get' -d 'Description: Get configuration value'
    complete -c sprite -f -a 'edit' -d 'Description: Open configuration in editor'
    complete -c sprite -f -a 'status' -d 'Description: Check for configuration changes'
end

function __fish_complete_start
    complete -c sprite -f -a 'session-name' -d 'Description: Custom session name'
    complete -c sprite -f -a 'layout' -d 'Description: Layout type'
    complete -c sprite -f -a 'detach' -d 'Description: Start and detach immediately'
end

function __fish_complete_warp
    complete -c sprite -f -a 'list' -d 'Description: List available workspaces'
    complete -c sprite -f -a 'print' -d 'Description: Print path without changing directory'
    complete -c sprite -f -a 'relative' -d 'Description: Show relative path'
    complete -c sprite -f -a '(command)' -d 'Description: Workspace to navigate to'
end

function __fish_complete_hey
    complete -c sprite -f -a 'timeout' -d 'Description: Command timeout in seconds'
    complete -c sprite -f -a 'work-dir' -d 'Description: Working directory for command'
    complete -c sprite -f -a 'env' -d 'Description: Environment variable (KEY=VALUE)'
    complete -c sprite -f -a 'interactive' -d 'Description: Interactive mode'
    complete -c sprite -f -a '(agent)' -d 'Description: Agent number'
    complete -c sprite -f -a '(command)' -d 'Description: Command to execute'
end

function __fish_complete_send
    complete -c sprite -f -a 'timeout' -d 'Description: Command timeout in seconds'
    complete -c sprite -f -a 'work-dir' -d 'Description: Working directory for command'
    complete -c sprite -f -a 'env' -d 'Description: Environment variable (KEY=VALUE)'
    complete -c sprite -f -a 'sequential' -d 'Description: Execute sequentially'
    complete -c sprite -f -a '(command)' -d 'Description: Command to execute'
end

function __fish_complete_sync
    complete -c sprite -f -a 'agent' -d 'Description: Sync specific agent'
    complete -c sprite -f -a 'force' -d 'Description: Force sync even if conflicts'
    complete -c sprite -f -a 'strategy' -d 'Description: Conflict resolution strategy'
    complete -c sprite -f -a 'dry-run' -d 'Description: Show what would be synced'
end

function __fish_complete_remove
    complete -c sprite -f -a 'force' -d 'Description: Force removal without confirmation'
    complete -c sprite -f -a 'keep-workspace' -d 'Description: Keep workspace files'
    complete -c sprite -f -a 'merge-branch' -d 'Description: Merge branch before removal'
    complete -c sprite -f -a '(agent)' -d 'Description: Agent number to remove'
end

function __fish_complete_zoom
    complete -c sprite -f -a 'unzoom' -d 'Description: Return to normal layout'
    complete -c sprite -f -a 'list' -d 'Description: List available panes'
    complete -c sprite -f -a '(agent)' -d 'Description: Agent number to zoom on'
end

function __fish_complete_status
    complete -c sprite -f -a 'cleanup' -d 'Description: Clean up old sessions and temporary files'
    complete -c sprite -f -a 'detailed' -d 'Description: Show detailed health information'
    complete -c sprite -f -a '(session-name)' -d 'Description: Check specific session'
end

function __fish_complete_help
    complete -c sprite -f -a 'command' -d 'Description: Command to get help for'
    complete -c sprite -f -a 'search' -d 'Description: Search help content'
    complete -c sprite -f -a 'patterns' -d 'Description: Show usage patterns'
    complete -c sprite -f -a 'troubleshooting' -d 'Description: Show troubleshooting guides'
    complete -c sprite -f -a 'quick' -d 'Description: Show quick reference'
    complete -c sprite -f -a 'accessible' -d 'Description: Show accessible help'
    complete -c sprite -f -a 'category' -d 'Description: Show usage patterns for specific category'
end
"#
        )
    }

    /// Generate zsh completion script.
    pub fn generate_zsh_completion(&self) -> String {
        format!(
            r#"#compdef sprite
# Sprite zsh completion script

#autoload -U compinit
compinit

_sprite() {{
  local line state
  _arguments -C $line

  case $state in
    (init)
      _arguments -s \
        '(-a --agents)' \
        '(-f --force)' \
        '--agents=-' \
        '--force=-' \
        ':description:Initialize a new multi-agent environment' \
        && return 0
      ;;
    (start)
      _arguments -s \
        '(-s --session-name)' \
        '(-l --layout)' \
        '(-d --detach)' \
        '--session-name=-' \
        '--layout=-' \
        '--detach=-' \
        ':description:Start a new multi-agent supervision session' \
        && return 0
      ;;
    (agents)
      _arguments -C \
        '(list:create:remove:show:validate:provision)' \
        ':description:Manage agents' \
        && return 0
      ;;
    (config)
      _arguments -C \
        '(show:validate:set:get:edit:status)' \
        ':description:Manage configuration settings' \
        && return 0
      ;;
    (warp)
      _arguments -s \
        '(-l --list)' \
        '(-p --print)' \
        '(-r --relative)' \
        '--list=-' \
        '--print=-' \
        '--relative=-' \
        ':description:Navigate between agent workspaces' \
        && return 0
      ;;
    (hey)
      _arguments -s \
        '(-t --timeout)' \
        '(-w --work-dir)' \
        '(-i --interactive)' \
        '--timeout=-' \
        '--work-dir=-' \
        '--interactive=-' \
        '1: :command: :*:command:args' \
        ':description:Send a command to a specific agent' \
        && return 0
      ;;
    (send)
      _arguments -s \
        '(-t --timeout)' \
        '(-w --work-dir)' \
        '(-e --env)' \
        '--timeout=-' \
        '--work-dir=-' \
        '--env=-' \
        '--sequential' \
        '1: :command: :*:command:args' \
        ':description:Send a command to all agents (broadcast)' \
        && return 0
      ;;
    (sync)
      _arguments -s \
        '(-a --agent)' \
        '(-f --force)' \
        '(-s --strategy)' \
        '--agent=-' \
        '--force=-' \
        '--strategy=-' \
        '--dry-run' \
        ':description:Synchronize workspaces context-aware' \
        && return 0
      ;;
    (remove)
      _arguments -s \
        '(-f --force)' \
        '--force=-' \
        '--keep-workspace' \
        '--merge-branch' \
        '1: :agent)' \
        ':description:Remove agent workspace' \
        && return 0
      ;;
    (zoom)
      _arguments -s \
        '(-u --unzoom)' \
        '--unzoom=-' \
        '(-l --list)' \
        '--unzoom=-' \
        '--list=-' \
        '1: :agent)' \
        ':description:Focus on agent pane' \
        && return 0
      ;;
    (status)
      _arguments -s \
        '(-c --cleanup)' \
        '(-d --detailed)' \
        '--cleanup=-' \
        '--detailed=-' \
        ':description:Check session status and health' \
        && return 0
      ;;
    (help)
      _arguments -s \
        '(--command)' \
        '(--search)' \
        '(--patterns)' \
        '(--troubleshooting)' \
        '(--quick)' \
        '(--accessible)' \
        '--command=-' \
        '--search=-' \
        '--patterns=-' \
        '--troubleshooting=-' \
        '--quick=-' \
        '--accessible=-' \
        '--category=-' \
        ':description:Show help and documentation' \
        && return 0
      ;;
    (*)
      _message 'Unknown command: $words[1]'
      _sprite_help
      return 1
      ;;
  esac
}

_sprite_help() {{
  echo "🚀 Sprite - Multi-Agent Workflow Toolkit"
  echo
  echo "Core Commands:"
  echo "  init     - Initialize project with agents"
  echo "  start    - Start agent supervision session"
  echo "  agents   - Manage agents (list, create, remove)"
  echo "  warp     - Navigate workspaces"
  echo "  hey      - Send commands to agents"
  echo "  send     - Broadcast commands to all agents"
  echo "  sync     - Synchronize workspaces"
  echo "  remove   - Remove agent workspaces"
  echo "  zoom     - Focus on agent panes"
  echo "  status   - Check session status"
  echo "  help     - Show this help"
  echo
  echo "Get detailed help: sprite help <command>"
  echo "Search help: sprite help --search <query>"
}}

compdef _sprite
{
  local line state
  _arguments -C $line

  _call_function "_sprite" $state
}}
"#
        )
    }

    /// Generate PowerShell completion script.
    pub fn generate_powershell_completion(&self) -> String {
        format!(
            r#"# PowerShell completion for Sprite

Register-ArgumentCompleter -Native -CommandName sprite -ScriptBlock {{ {{
    param($commandName, $wordToComplete, $commandAst, $cursorPosition)

    $completions = @(
        switch ($commandName) {{
            "init" {{
                @("agents", "force") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "start" {{
                @("session-name", "layout", "detach") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "agents" {{
                @("list", "create", "remove", "show", "validate", "provision") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "config" {{
                @("show", "validate", "set", "get", "edit", "status") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "warp" {{
                @("list", "print", "relative") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "hey" {{
                @("timeout", "work-dir", "env", "interactive") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "send" {{
                @("timeout", "work-dir", "env", "sequential") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "sync" {{
                @("agent", "force", "strategy", "dry-run") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "remove" {{
                @("force", "keep-workspace", "merge-branch") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "zoom" {{
                @("unzoom", "list") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "status" {{
                @("cleanup", "detailed") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
            "help" {{
                @("command", "search", "patterns", "troubleshooting", "quick", "accessible", "category") | ForEach-Object {{
                    Complete-Argument -Variable cmdword -ParameterName "Argument" -WordToComplete $wordToComplete
                }}
            }}
        }}
    )

    $completions | Where-Object {{ $_.CompletionText }}
    $completions | Sort-Object Property CompletionText
    $completions | Select-Object -Unique
    $completions | ForEach-Object {{ Complete-Argument -Variable cmdword -ParameterValue $_.CompletionText }}
}}

Register-ArgumentCompleter -Native -ScriptBlock {{ __sprite_completion }}

# Add to your PowerShell profile:
# . $PROFILE.CurrentUserAllHosts
# Add to end of profile
#
# . $PROFILE.CurrentUserAllHosts = $PROFILE.CurrentUserAllHosts + @"
# source C:\path\to\sprite-completion.ps1
# ""
#
"#
        )
    }
    }
}

/// Calculate Levenshtein distance between two strings.
fn levenshtein_distance(s1: &str, s2: &s1) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 { return len2; }
    if len2 == 0 { return len1; }

    let mut prev = 0;
    let mut cost = if len1 < len2 { len2 - len1 } else { len1 - len2 };

    for (i, c1) in s1.chars().enumerate() {
        let mut min_cost = cost + 1;

        for (j, c2) in s2.chars().enumerate() {
            let cost = if i == 0 && j == 0 {
                if c1 != c2 {{ prev as i } + j } else {{ prev as i + 1 }
            } else {
                prev + 1 + if c1 == c2 {{ 0 }} else {{ 1 }}
            };

            if cost < min_cost {{
                min_cost = cost;
            }}
        }

        prev = min_cost;
    }

    cost
}
}

impl Default for CompletionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CompletionSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sprite Completion System ({} commands)", self.commands.len())
    }
}

/// Get the global completion system instance.
pub fn get_completion_system() -> CompletionSystem {
    CompletionSystem::new()
}

/// Generate shell completion scripts.
pub fn generate_shell_completions(shell: &str) -> Result<String> {
    let system = get_completion_system();

    match shell {
        "bash" => Ok(system.generate_bash_completion()),
        "fish" => Ok(system.generate_fish_completion()),
        "zsh" => Ok(system.generate_zsh_completion()),
        "powershell" => Ok(system.generate_powershell_completion()),
        _ => Err(anyhow::anyhow!("Unsupported shell: {}", shell)),
    }
}

/// Get completions for a specific context.
pub fn get_completions(input: &str, context: Option<&str>) -> Vec<CompletionResult> {
    let system = get_completion_system();
    let mut results = Vec::new();

    // If context is provided, try command completion
    if let Some(ctx) = context {
        if let Some(cmd_info) = system.commands.get(ctx) {
            let completions = system.get_argument_completions(ctx, input);
            if !completions.is_empty() {
                results.push(CompletionResult {
                    suggestions: completions,
                    input: input.to_string(),
                    cursor_pos: input.len(),
                    command_context: Some(ctx.to_string()),
                });
            }
        }
    } else {
        // Try command completions
        let completions = system.get_command_completions(input);
        if !completions.is_empty() {
            results.push(CompletionResult {
                suggestions: completions,
                input: input.to_string(),
                cursor_pos: input.len(),
                command_context: None,
            });
        }
    }

    results
}

/// Get smart suggestions for input correction.
pub fn get_smart_suggestions(input: &str, context: &str) -> Vec<SmartSuggestion> {
    let system = get_completion_system();
    system.get_smart_suggestions(input, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_system_creation() {
        let system = CompletionSystem::new();
        assert!(!system.commands.is_empty());
        assert!(system.commands.contains_key("init"));
    }

    #[test]
    fn test_command_completions() {
        let system = CompletionSystem::new();
        let completions = system.get_command_completions("in");
        assert!(completions.contains(&"init".to_string()));
        assert!(completions.contains(&"init".to_string()));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", "test"), 4);
        assert_eq!(levenshtein_distance("init", "init"), 0);
        assert_eq!(levenshtein_distance("init", "ink"), 1);
        assert_eq!(levenshtein_distance("start", "stat"), 4);
    }

    #[test]
    test_smart_suggestions() {
        let system = CompletionSystem::new();
        let suggestions = system.get_smart_suggestions("stat", "sprite");
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::Correction);
    }
}