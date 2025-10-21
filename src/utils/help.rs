//! Comprehensive help system for the Sprite multi-agent workflow toolkit.
//!
//! This module provides enhanced help functionality including command documentation,
//! usage examples, troubleshooting guides, and contextual help suggestions.

use anyhow::Result;
use std::collections::HashMap;
use std::fmt;

/// Comprehensive help system for Sprite commands and features.
#[derive(Debug, Clone)]
pub struct HelpSystem {
    /// Command documentation
    pub commands: HashMap<String, CommandHelp>,
    /// Common usage patterns
    pub usage_patterns: Vec<UsagePattern>,
    /// Troubleshooting guides
    pub troubleshooting: Vec<TroubleshootingGuide>,
    /// Quick reference
    pub quick_reference: QuickReference,
}

/// Help information for a specific command.
#[derive(Debug, Clone)]
pub struct CommandHelp {
    /// Command name
    pub name: String,
    /// Brief description
    pub description: String,
    /// Detailed usage information
    pub usage: String,
    /// Examples
    pub examples: Vec<String>,
    /// Common mistakes and solutions
    pub common_mistakes: Vec<String>,
    /// Related commands
    pub related_commands: Vec<String>,
    /// Prerequisites
    pub prerequisites: Vec<String>,
    /// Tips and tricks
    pub tips: Vec<String>,
}

/// Usage pattern for common workflows.
#[derive(Debug, Clone)]
pub struct UsagePattern {
    /// Pattern name
    pub name: String,
    /// Description
    pub description: String,
    /// Steps to follow
    pub steps: Vec<String>,
    /// Commands involved
    pub commands: Vec<String>,
    /// Use case category
    pub category: UsageCategory,
}

/// Category of usage pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum UsageCategory {
    /// Getting started
    GettingStarted,
    /// Daily workflow
    DailyWorkflow,
    /// Troubleshooting
    Troubleshooting,
    /// Advanced usage
    Advanced,
    /// Collaboration
    Collaboration,
}

/// Troubleshooting guide.
#[derive(Debug, Clone)]
pub struct TroubleshootingGuide {
    /// Problem description
    pub problem: String,
    /// Symptoms
    pub symptoms: Vec<String>,
    /// Possible causes
    pub causes: Vec<String>,
    /// Solutions
    pub solutions: Vec<String>,
    /// Prevention tips
    pub prevention: Vec<String>,
    /// Related commands
    pub related_commands: Vec<String>,
}

/// Quick reference information.
#[derive(Debug, Clone)]
pub struct QuickReference {
    /// Most commonly used commands
    pub common_commands: Vec<String>,
    /// Keyboard shortcuts
    pub shortcuts: Vec<KeyboardShortcut>,
    /// Frequently asked questions
    pub faq: Vec<Faq>,
}

/// Keyboard shortcut.
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    /// Key combination
    pub keys: String,
    /// Description
    pub description: String,
    /// Context where it applies
    pub context: String,
}

/// Frequently asked question.
#[derive(Debug, Clone)]
pub struct Faq {
    /// Question
    pub question: String,
    /// Answer
    pub answer: String,
    /// Related commands
    pub related_commands: Vec<String>,
}

impl HelpSystem {
    /// Create a new help system with all documentation.
    pub fn new() -> Self {
        let mut help_system = Self {
            commands: HashMap::new(),
            usage_patterns: Vec::new(),
            troubleshooting: Vec::new(),
            quick_reference: QuickReference {
                common_commands: Vec::new(),
                shortcuts: Vec::new(),
                faq: Vec::new(),
            },
        };

        help_system.load_command_help();
        help_system.load_usage_patterns();
        help_system.load_troubleshooting();
        help_system.load_quick_reference();

        help_system
    }

    /// Load command help documentation.
    fn load_command_help(&mut self) {
        // Core commands
        self.commands.insert(
            "init".to_string(),
            CommandHelp {
                name: "init".to_string(),
                description: "Initialize a new multi-agent environment in the current directory"
                    .to_string(),
                usage: "sprite init [OPTIONS]".to_string(),
                examples: vec![
                    "sprite init                                    # Basic initialization"
                        .to_string(),
                    "sprite init --agents 5                        # Create 5 agents".to_string(),
                    "sprite init --force                           # Overwrite existing config"
                        .to_string(),
                ],
                common_mistakes: vec![
                    "Running from outside a git repository - sprite requires a git repository"
                        .to_string(),
                    "Not having tmux installed - required for session management".to_string(),
                ],
                related_commands: vec![
                    "config".to_string(),
                    "agents".to_string(),
                    "start".to_string(),
                ],
                prerequisites: vec![
                    "Git repository initialized".to_string(),
                    "Tmux installed and available".to_string(),
                ],
                tips: vec![
                    "Use --force if you want to overwrite an existing configuration".to_string(),
                    "The default creates 3 agents - adjust with --agents if needed".to_string(),
                ],
            },
        );

        self.commands.insert(
            "start".to_string(),
            CommandHelp {
                name: "start".to_string(),
                description: "Start a new multi-agent supervision session".to_string(),
                usage: "sprite start [OPTIONS]".to_string(),
                examples: vec![
                    "sprite start                                    # Start with default settings"
                        .to_string(),
                    "sprite start --session-name my-session          # Custom session name"
                        .to_string(),
                    "sprite start --layout main-vertical            # Use vertical layout"
                        .to_string(),
                    "sprite start --detach                          # Start and detach immediately"
                        .to_string(),
                ],
                common_mistakes: vec![
                    "Starting without agent configuration - run 'sprite init' first".to_string(),
                    "Session already exists - use 'sprite attach' or kill existing session"
                        .to_string(),
                ],
                related_commands: vec![
                    "attach".to_string(),
                    "kill".to_string(),
                    "agents".to_string(),
                ],
                prerequisites: vec![
                    "Sprite configuration exists".to_string(),
                    "Agent workspaces created".to_string(),
                ],
                tips: vec![
                    "Use --detach to start in background and return to terminal".to_string(),
                    "Different layouts work better for different screen sizes".to_string(),
                ],
            },
        );

        self.commands.insert(
            "agents".to_string(),
            CommandHelp {
                name: "agents".to_string(),
                description: "Manage agents (list, create, remove, show details)".to_string(),
                usage: "sprite agents <SUBCOMMAND> [OPTIONS]".to_string(),
                examples: vec![
                    "sprite agents list                              # List all agents".to_string(),
                    "sprite agents create 4                          # Create agent 4".to_string(),
                    "sprite agents create 5 --branch feature/auth   # Custom branch".to_string(),
                    "sprite agents show 1                           # Show agent 1 details"
                        .to_string(),
                    "sprite agents remove 2 --force                 # Remove agent 2".to_string(),
                ],
                common_mistakes: vec![
                    "Creating agents with duplicate IDs - each agent must have unique ID"
                        .to_string(),
                    "Removing agents that are currently running - stop session first".to_string(),
                ],
                related_commands: vec![
                    "remove".to_string(),
                    "start".to_string(),
                    "warp".to_string(),
                ],
                prerequisites: vec!["Sprite configuration exists".to_string()],
                tips: vec![
                    "Use 'show' to see detailed agent information".to_string(),
                    "Agent IDs are typically numbers but can be any identifier".to_string(),
                ],
            },
        );

        self.commands.insert(
            "warp".to_string(),
            CommandHelp {
                name: "warp".to_string(),
                description: "Navigate between agent workspaces".to_string(),
                usage: "sprite warp [WORKSPACE] [OPTIONS]".to_string(),
                examples: vec![
                    "sprite warp                                    # Show current workspace"
                        .to_string(),
                    "sprite warp --list                            # List all workspaces"
                        .to_string(),
                    "sprite warp 1                                 # Navigate to agent 1"
                        .to_string(),
                    "sprite warp main                              # Navigate to main directory"
                        .to_string(),
                    "sprite warp --print 2                         # Print path without changing"
                        .to_string(),
                ],
                common_mistakes: vec![
                    "Navigating to workspace that doesn't exist - check with --list first"
                        .to_string(),
                    "Forgetting that sprite warp doesn't actually change directory".to_string(),
                ],
                related_commands: vec!["agents".to_string(), "status".to_string()],
                prerequisites: vec!["Git repository".to_string()],
                tips: vec![
                    "Use 'cd $(sprite warp --print <name>)' to actually change directory"
                        .to_string(),
                    "Workspace navigation works with relative paths and tab completion".to_string(),
                ],
            },
        );

        self.commands.insert(
            "hey".to_string(),
            CommandHelp {
                name: "hey".to_string(),
                description: "Send a command to a specific agent".to_string(),
                usage: "sprite hey <AGENT> <COMMAND> [ARGS...] [OPTIONS]".to_string(),
                examples: vec![
                    "sprite hey 1 'npm test'                       # Run tests in agent 1"
                        .to_string(),
                    "sprite hey 2 'git status' --timeout 60       # Custom timeout".to_string(),
                    "sprite hey 3 'npm run dev' --interactive      # Interactive mode".to_string(),
                ],
                common_mistakes: vec![
                    "Sending commands to non-existent agent - check with 'sprite agents list'"
                        .to_string(),
                    "Commands not found - agents have different environments".to_string(),
                    "Timeout too short for long-running operations".to_string(),
                ],
                related_commands: vec![
                    "send".to_string(),
                    "agents".to_string(),
                    "start".to_string(),
                ],
                prerequisites: vec!["Active session with running agents".to_string()],
                tips: vec![
                    "Use --interactive for commands that require user input".to_string(),
                    "Adjust timeout based on command complexity".to_string(),
                    "Use 'send' to broadcast to all agents".to_string(),
                ],
            },
        );

        self.commands.insert(
            "send".to_string(),
            CommandHelp {
                name: "send".to_string(),
                description: "Send a command to all agents (broadcast)".to_string(),
                usage: "sprite send <COMMAND> [ARGS...] [OPTIONS]".to_string(),
                examples: vec![
                    "sprite send 'git pull'                        # Pull changes in all agents"
                        .to_string(),
                    "sprite send 'npm install' --sequential        # Run one by one".to_string(),
                    "sprite send 'cargo build' --timeout 120       # Longer timeout".to_string(),
                ],
                common_mistakes: vec![
                    "Broadcasting expensive operations - consider using --sequential".to_string(),
                    "Not setting appropriate timeout for complex commands".to_string(),
                ],
                related_commands: vec!["hey".to_string(), "sync".to_string()],
                prerequisites: vec!["Active session with running agents".to_string()],
                tips: vec![
                    "Use --sequential for operations that shouldn't run in parallel".to_string(),
                    "Broadcast is great for maintenance tasks across all agents".to_string(),
                ],
            },
        );

        self.commands.insert(
            "sync".to_string(),
            CommandHelp {
                name: "sync".to_string(),
                description: "Synchronize workspaces context-aware".to_string(),
                usage: "sprite sync [OPTIONS]".to_string(),
                examples: vec![
                    "sprite sync                                    # Sync current context"
                        .to_string(),
                    "sprite sync --agent 2                          # Sync specific agent"
                        .to_string(),
                    "sprite sync --force                            # Force sync with conflicts"
                        .to_string(),
                    "sprite sync --dry-run                          # Preview changes".to_string(),
                ],
                common_mistakes: vec![
                    "Syncing with uncommitted changes - commit or stash first".to_string(),
                    "Merge conflicts - resolve manually or use --force strategy".to_string(),
                ],
                related_commands: vec!["hey".to_string(), "send".to_string(), "status".to_string()],
                prerequisites: vec!["Git repository with changes to sync".to_string()],
                tips: vec![
                    "Use --dry-run to preview what will be synced".to_string(),
                    "Different behavior in main vs agent workspaces".to_string(),
                ],
            },
        );

        self.commands.insert("remove".to_string(), CommandHelp {
            name: "remove".to_string(),
            description: "Remove agent workspace with comprehensive cleanup".to_string(),
            usage: "sprite remove <AGENT> [OPTIONS]".to_string(),
            examples: vec![
                "sprite remove 1                               # Remove agent 1 (with confirmation)".to_string(),
                "sprite remove 2 --force                      # Remove without confirmation".to_string(),
                "sprite remove 3 --merge-branch               # Merge branch before removal".to_string(),
                "sprite remove 4 --keep-workspace             # Keep workspace files".to_string(),
            ],
            common_mistakes: vec![
                "Removing running agent - stop session first".to_string(),
                "Losing work with uncommitted changes - commit first".to_string(),
            ],
            related_commands: vec!["agents".to_string(), "kill".to_string()],
            prerequisites: vec![
                "Agent not currently running".to_string(),
            ],
            tips: vec![
                "Use --merge-branch to preserve agent work".to_string(),
                "--force bypasses all safety checks - use with caution".to_string(),
            ],
        });

        self.commands.insert(
            "zoom".to_string(),
            CommandHelp {
                name: "zoom".to_string(),
                description: "Focus on a specific agent pane in tmux".to_string(),
                usage: "sprite zoom [AGENT] [OPTIONS]".to_string(),
                examples: vec![
                    "sprite zoom 1                                 # Zoom on agent 1".to_string(),
                    "sprite zoom --unzoom                         # Return to normal layout"
                        .to_string(),
                    "sprite zoom --list                           # List available panes"
                        .to_string(),
                ],
                common_mistakes: vec![
                    "Zooming when no session is active - start session first".to_string(),
                    "Agent not found in current session".to_string(),
                ],
                related_commands: vec![
                    "start".to_string(),
                    "attach".to_string(),
                    "hey".to_string(),
                ],
                prerequisites: vec!["Active tmux session".to_string()],
                tips: vec![
                    "Use --unzoom or run again to return to normal layout".to_string(),
                    "Great for focusing on a specific agent's work".to_string(),
                ],
            },
        );

        self.commands.insert(
            "status".to_string(),
            CommandHelp {
                name: "status".to_string(),
                description: "Check session status and health".to_string(),
                usage: "sprite status [OPTIONS]".to_string(),
                examples: vec![
                    "sprite status                                    # Show status".to_string(),
                    "sprite status --detailed                        # Detailed information"
                        .to_string(),
                    "sprite status --cleanup                         # Clean up old sessions"
                        .to_string(),
                    "sprite status --session-name my-session         # Check specific session"
                        .to_string(),
                ],
                common_mistakes: vec![
                    "Not understanding status indicators - check help for symbols".to_string(),
                ],
                related_commands: vec![
                    "start".to_string(),
                    "kill".to_string(),
                    "attach".to_string(),
                ],
                prerequisites: vec!["None (works with or without active sessions)".to_string()],
                tips: vec![
                    "Use --detailed for comprehensive health information".to_string(),
                    "Regular status checks help maintain system health".to_string(),
                ],
            },
        );

        self.commands.insert(
            "config".to_string(),
            CommandHelp {
                name: "config".to_string(),
                description: "Manage configuration settings".to_string(),
                usage: "sprite config <SUBCOMMAND> [OPTIONS]".to_string(),
                examples: vec![
                    "sprite config show                              # Show current config"
                        .to_string(),
                    "sprite config validate                          # Validate configuration"
                        .to_string(),
                    "sprite config set timeout 60                   # Set configuration value"
                        .to_string(),
                    "sprite config edit                              # Open config in editor"
                        .to_string(),
                ],
                common_mistakes: vec![
                    "Invalid configuration values - check validation output".to_string(),
                    "Editing config directly - use 'sprite config edit'".to_string(),
                ],
                related_commands: vec!["init".to_string(), "agents".to_string()],
                prerequisites: vec!["Sprite configuration file exists".to_string()],
                tips: vec![
                    "Always validate config after making changes".to_string(),
                    "Use 'show' to review current settings".to_string(),
                ],
            },
        );
    }

    /// Load usage patterns.
    fn load_usage_patterns(&mut self) {
        self.usage_patterns = vec![
            UsagePattern {
                name: "First Time Setup".to_string(),
                description: "Set up Sprite for a new project".to_string(),
                steps: vec![
                    "Initialize git repository if not already done".to_string(),
                    "Run 'sprite init' to create initial configuration".to_string(),
                    "Review agent configuration with 'sprite config show'".to_string(),
                    "Start first session with 'sprite start'".to_string(),
                    "Navigate workspaces with 'sprite warp --list'".to_string(),
                ],
                commands: vec![
                    "init".to_string(),
                    "config".to_string(),
                    "start".to_string(),
                    "warp".to_string(),
                ],
                category: UsageCategory::GettingStarted,
            },
            UsagePattern {
                name: "Daily Workflow".to_string(),
                description: "Typical daily work with Sprite agents".to_string(),
                steps: vec![
                    "Check current status with 'sprite status'".to_string(),
                    "Start session if not running: 'sprite start'".to_string(),
                    "Sync any changes: 'sprite sync'".to_string(),
                    "Navigate to workspace: 'sprite warp <agent>'".to_string(),
                    "Send commands to agents: 'sprite hey <agent> <command>'".to_string(),
                    "Check progress and coordinate between agents".to_string(),
                    "End session: 'sprite kill'".to_string(),
                ],
                commands: vec![
                    "status".to_string(),
                    "start".to_string(),
                    "sync".to_string(),
                    "warp".to_string(),
                    "hey".to_string(),
                    "kill".to_string(),
                ],
                category: UsageCategory::DailyWorkflow,
            },
            UsagePattern {
                name: "Collaborative Development".to_string(),
                description: "Working with multiple agents on the same codebase".to_string(),
                steps: vec![
                    "Ensure all agents are on latest code: 'sprite send git pull'".to_string(),
                    "Assign tasks to different agents based on expertise".to_string(),
                    "Coordinate changes to avoid conflicts".to_string(),
                    "Regular sync to merge work: 'sprite sync'".to_string(),
                    "Resolve conflicts if they arise".to_string(),
                    "Review and test combined changes".to_string(),
                ],
                commands: vec![
                    "send".to_string(),
                    "sync".to_string(),
                    "hey".to_string(),
                    "status".to_string(),
                ],
                category: UsageCategory::Collaboration,
            },
        ];
    }

    /// Load troubleshooting guides.
    fn load_troubleshooting(&mut self) {
        self.troubleshooting = vec![
            TroubleshootingGuide {
                problem: "Session won't start".to_string(),
                symptoms: vec![
                    "'sprite start' fails with error".to_string(),
                    "Tmux session not created".to_string(),
                    "Agents don't appear in session".to_string(),
                ],
                causes: vec![
                    "Tmux not installed or not in PATH".to_string(),
                    "Configuration file is corrupted or invalid".to_string(),
                    "Git repository issues".to_string(),
                    "Agent workspace creation failed".to_string(),
                ],
                solutions: vec![
                    "Check tmux installation: 'tmux -V'".to_string(),
                    "Validate configuration: 'sprite config validate'".to_string(),
                    "Check git status: 'git status'".to_string(),
                    "Reinitialize: 'sprite init --force'".to_string(),
                ],
                prevention: vec![
                    "Regularly validate configuration".to_string(),
                    "Keep tmux updated".to_string(),
                    "Maintain clean git repository".to_string(),
                ],
                related_commands: vec![
                    "config".to_string(),
                    "init".to_string(),
                    "status".to_string(),
                ],
            },
            TroubleshootingGuide {
                problem: "Agent commands not working".to_string(),
                symptoms: vec![
                    "Commands sent with 'hey' fail".to_string(),
                    "No output from agents".to_string(),
                    "Timeout errors".to_string(),
                ],
                causes: vec![
                    "Session not active".to_string(),
                    "Agent pane not responding".to_string(),
                    "Command not found in agent environment".to_string(),
                    "Network or permission issues".to_string(),
                ],
                solutions: vec![
                    "Check session status: 'sprite status'".to_string(),
                    "Increase timeout: '--timeout 120'".to_string(),
                    "Restart session: 'sprite kill && sprite start'".to_string(),
                    "Check agent environment with simple commands".to_string(),
                ],
                prevention: vec![
                    "Use appropriate timeouts for commands".to_string(),
                    "Test agent environments regularly".to_string(),
                    "Monitor agent health with status command".to_string(),
                ],
                related_commands: vec![
                    "status".to_string(),
                    "hey".to_string(),
                    "start".to_string(),
                ],
            },
            TroubleshootingGuide {
                problem: "Sync conflicts".to_string(),
                symptoms: vec![
                    "'sprite sync' reports conflicts".to_string(),
                    "Git merge conflicts appear".to_string(),
                    "Agents can't share changes".to_string(),
                ],
                causes: vec![
                    "Multiple agents modified same files".to_string(),
                    "Uncommitted changes during sync".to_string(),
                    "Branch divergence".to_string(),
                ],
                solutions: vec![
                    "Commit changes before syncing".to_string(),
                    "Use conflict resolution: --strategy auto-theirs".to_string(),
                    "Manual conflict resolution in main workspace".to_string(),
                    "Coordinate agent work to avoid overlap".to_string(),
                ],
                prevention: vec![
                    "Regular syncs to prevent divergence".to_string(),
                    "Coordinate agent tasks carefully".to_string(),
                    "Commit frequently when working with agents".to_string(),
                ],
                related_commands: vec!["sync".to_string(), "hey".to_string(), "warp".to_string()],
            },
        ];
    }

    /// Load quick reference information.
    fn load_quick_reference(&mut self) {
        self.quick_reference.common_commands = vec![
            "sprite init".to_string(),
            "sprite start".to_string(),
            "sprite agents list".to_string(),
            "sprite warp --list".to_string(),
            "sprite hey <agent> <command>".to_string(),
            "sprite send <command>".to_string(),
            "sprite sync".to_string(),
            "sprite status".to_string(),
            "sprite kill".to_string(),
        ];

        self.quick_reference.shortcuts = vec![
            KeyboardShortcut {
                keys: "Ctrl+C".to_string(),
                description: "Cancel current command or operation".to_string(),
                context: "All contexts".to_string(),
            },
            KeyboardShortcut {
                keys: "Ctrl+D".to_string(),
                description: "Exit current session or shell".to_string(),
                context: "Tmux sessions".to_string(),
            },
            KeyboardShortcut {
                keys: "Ctrl+B".to_string(),
                description: "Tmux prefix key".to_string(),
                context: "Tmux sessions".to_string(),
            },
            KeyboardShortcut {
                keys: "Ctrl+B then d".to_string(),
                description: "Detach from tmux session".to_string(),
                context: "Tmux sessions".to_string(),
            },
        ];

        self.quick_reference.faq = vec![
            Faq {
                question: "How do I get started with Sprite?".to_string(),
                answer: "Run 'sprite init' in your git repository, then 'sprite start' to begin your first session. Use 'sprite warp --list' to see available workspaces.".to_string(),
                related_commands: vec!["init".to_string(), "start".to_string(), "warp".to_string()],
            },
            Faq {
                question: "Can I use Sprite without tmux?".to_string(),
                answer: "No, Sprite requires tmux for session management and multi-agent coordination. Install tmux first, then use Sprite.".to_string(),
                related_commands: vec![],
            },
            Faq {
                question: "How do I actually change directories with warp?".to_string(),
                answer: "Use 'cd $(sprite warp --print <name>)' to change directory. Sprite warp shows the path but doesn't change directories itself.".to_string(),
                related_commands: vec!["warp".to_string()],
            },
            Faq {
                question: "What's the difference between 'hey' and 'send'?".to_string(),
                answer: "'hey' sends commands to a specific agent, while 'send' broadcasts to all agents. Use 'hey' for targeted tasks, 'send' for maintenance or coordination.".to_string(),
                related_commands: vec!["hey".to_string(), "send".to_string()],
            },
        ];
    }

    /// Get help for a specific command.
    pub fn get_command_help(&self, command: &str) -> Option<&CommandHelp> {
        self.commands.get(command)
    }

    /// Get all available commands.
    pub fn list_commands(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// Get usage patterns by category.
    pub fn get_usage_patterns(&self, category: Option<UsageCategory>) -> Vec<&UsagePattern> {
        match category {
            Some(cat) => self
                .usage_patterns
                .iter()
                .filter(|p| p.category == cat)
                .collect(),
            None => self.usage_patterns.iter().collect(),
        }
    }

    /// Get troubleshooting guides.
    pub fn get_troubleshooting(&self) -> &[TroubleshootingGuide] {
        &self.troubleshooting
    }

    /// Get quick reference.
    pub fn get_quick_reference(&self) -> &QuickReference {
        &self.quick_reference
    }

    /// Search help content.
    pub fn search(&self, query: &str) -> Vec<HelpResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Search commands
        for (name, command) in &self.commands {
            if name.contains(&query_lower)
                || command.description.to_lowercase().contains(&query_lower)
            {
                results.push(HelpResult {
                    result_type: HelpResultType::Command,
                    title: format!("Command: {}", name),
                    description: command.description.clone(),
                    action: format!("sprite {} --help", name),
                });
            }

            // Search examples
            for example in &command.examples {
                if example.to_lowercase().contains(&query_lower) {
                    results.push(HelpResult {
                        result_type: HelpResultType::Example,
                        title: format!("Example: {}", name),
                        description: example.clone(),
                        action: example.clone(),
                    });
                }
            }

            // Search tips
            for tip in &command.tips {
                if tip.to_lowercase().contains(&query_lower) {
                    results.push(HelpResult {
                        result_type: HelpResultType::Tip,
                        title: format!("Tip: {}", name),
                        description: tip.clone(),
                        action: format!("sprite {} --help", name),
                    });
                }
            }
        }

        // Search troubleshooting
        for guide in &self.troubleshooting {
            if guide.problem.to_lowercase().contains(&query_lower) {
                results.push(HelpResult {
                    result_type: HelpResultType::Troubleshooting,
                    title: format!("Problem: {}", guide.problem),
                    description: guide.symptoms.join("; "),
                    action: "Check troubleshooting guide".to_string(),
                });
            }
        }

        results
    }

    /// Format help output for display.
    pub fn format_help(&self, command: Option<&str>) -> String {
        match command {
            Some(cmd) => {
                if let Some(help) = self.get_command_help(cmd) {
                    self.format_command_help(help)
                } else {
                    format!(
                        "Command '{}' not found. Available commands:\n{}",
                        cmd,
                        self.list_commands().join(", ")
                    )
                }
            }
            None => self.format_general_help(),
        }
    }

    /// Format command help.
    fn format_command_help(&self, help: &CommandHelp) -> String {
        let mut output = String::new();

        output.push_str(&format!("🔧 {} - {}\n\n", help.name, help.description));
        output.push_str(&format!("Usage: {}\n\n", help.usage));

        if !help.examples.is_empty() {
            output.push_str("Examples:\n");
            for example in &help.examples {
                output.push_str(&format!("  {}\n", example));
            }
            output.push('\n');
        }

        if !help.prerequisites.is_empty() {
            output.push_str("Prerequisites:\n");
            for prereq in &help.prerequisites {
                output.push_str(&format!("  • {}\n", prereq));
            }
            output.push('\n');
        }

        if !help.tips.is_empty() {
            output.push_str("Tips:\n");
            for tip in &help.tips {
                output.push_str(&format!("  💡 {}\n", tip));
            }
            output.push('\n');
        }

        if !help.common_mistakes.is_empty() {
            output.push_str("Common Mistakes:\n");
            for mistake in &help.common_mistakes {
                output.push_str(&format!("  ⚠️  {}\n", mistake));
            }
            output.push('\n');
        }

        if !help.related_commands.is_empty() {
            output.push_str(&format!(
                "Related Commands: {}\n",
                help.related_commands.join(", ")
            ));
        }

        output
    }

    /// Format general help.
    fn format_general_help(&self) -> String {
        let mut output = String::new();

        output.push_str("🚀 Sprite - Multi-Agent Workflow Toolkit\n\n");
        output.push_str(
            "Sprite helps you manage multiple AI coding agents in isolated tmux sessions\n",
        );
        output.push_str("with comprehensive workspace management and health monitoring.\n\n");

        output.push_str("📋 Quick Start:\n");
        output.push_str("  1. sprite init          # Initialize project\n");
        output.push_str("  2. sprite start         # Start session\n");
        output.push_str("  3. sprite hey 1 <cmd>   # Send command to agent 1\n");
        output.push_str("  4. sprite status        # Check status\n\n");

        output.push_str("🔧 Core Commands:\n");
        for cmd in &[
            "init", "start", "agents", "warp", "hey", "send", "sync", "status",
        ] {
            if let Some(help) = self.get_command_help(cmd) {
                output.push_str(&format!("  {:12} - {}\n", cmd, help.description));
            }
        }

        output.push_str("\n💡 Get Help:\n");
        output.push_str("  sprite help <command>  # Get detailed help for a command\n");
        output.push_str("  sprite help --search  # Search help content\n");
        output.push_str("  sprite help --patterns # Show usage patterns\n");

        output
    }

    /// Generate accessible help output.
    pub fn format_accessible_help(&self, command: Option<&str>) -> String {
        // Similar to format_help but with enhanced accessibility features
        let mut output = "SCREEN READER HELP FOR SPRITE\n\n".to_string();

        if let Some(cmd) = command {
            if let Some(help) = self.get_command_help(cmd) {
                output.push_str(&format!("COMMAND: {}\n", help.name.to_uppercase()));
                output.push_str(&format!("DESCRIPTION: {}\n", help.description));
                output.push_str(&format!("USAGE: {}\n", help.usage));

                output.push_str("\nEXAMPLES:\n");
                for (i, example) in help.examples.iter().enumerate() {
                    output.push_str(&format!("EXAMPLE {}: {}\n", i + 1, example));
                }

                if !help.prerequisites.is_empty() {
                    output.push_str("\nPREREQUISITES:\n");
                    for (i, prereq) in help.prerequisites.iter().enumerate() {
                        output.push_str(&format!("{}. {}\n", i + 1, prereq));
                    }
                }

                output.push_str("\nRELATED COMMANDS:\n");
                output.push_str(&help.related_commands.join(", "));
            } else {
                output.push_str(&format!("COMMAND '{}' NOT FOUND\n", cmd.to_uppercase()));
                output.push_str("AVAILABLE COMMANDS:\n");
                for cmd_name in self.list_commands() {
                    output.push_str(&format!("• {}\n", cmd_name));
                }
            }
        } else {
            output.push_str("SPRITE OVERVIEW\n");
            output.push_str(
                "Sprite is a command-line toolkit for managing multiple AI coding agents.\n\n",
            );

            output.push_str("AVAILABLE COMMANDS:\n");
            for cmd_name in self.list_commands() {
                if let Some(help) = self.get_command_help(cmd_name) {
                    output.push_str(&format!("• {}: {}\n", cmd_name, help.description));
                }
            }

            output.push_str("\nFor detailed help on any command, use: sprite help <command>\n");
        }

        output
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a help search.
#[derive(Debug, Clone)]
pub struct HelpResult {
    /// Type of result
    pub result_type: HelpResultType,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Suggested action
    pub action: String,
}

/// Type of help result.
#[derive(Debug, Clone, PartialEq)]
pub enum HelpResultType {
    /// Command help
    Command,
    /// Usage example
    Example,
    /// Tip or trick
    Tip,
    /// Troubleshooting guide
    Troubleshooting,
}

impl fmt::Display for HelpSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sprite Help System with {} commands",
            self.commands.len()
        )
    }
}

/// Get the global help system instance.
pub fn get_help_system() -> HelpSystem {
    HelpSystem::new()
}

/// Show help for a specific command.
pub fn show_help(command: Option<&str>) -> Result<()> {
    let help_system = get_help_system();
    let help_text = help_system.format_help(command);
    println!("{}", help_text);
    Ok(())
}

/// Show accessible help.
pub fn show_accessible_help(command: Option<&str>) -> Result<()> {
    let help_system = get_help_system();
    let help_text = help_system.format_accessible_help(command);
    println!("{}", help_text);
    Ok(())
}

/// Search help content.
pub fn search_help(query: &str) -> Result<()> {
    let help_system = get_help_system();
    let results = help_system.search(query);

    if results.is_empty() {
        println!("No results found for: {}", query);
        return Ok(());
    }

    println!("🔍 Search Results for: {}\n", query);
    for result in results {
        match result.result_type {
            HelpResultType::Command => println!("📋 Command:"),
            HelpResultType::Example => println!("💡 Example:"),
            HelpResultType::Tip => println!("✨ Tip:"),
            HelpResultType::Troubleshooting => println!("🔧 Troubleshooting:"),
        }

        println!("  {}", result.title);
        println!("  {}", result.description);

        if !result.action.is_empty() {
            println!("  Action: {}", result.action);
        }

        println!();
    }

    Ok(())
}

/// Show usage patterns.
pub fn show_usage_patterns(category: Option<UsageCategory>) -> Result<()> {
    let help_system = get_help_system();
    let patterns = help_system.get_usage_patterns(category);

    if patterns.is_empty() {
        println!("No usage patterns found.");
        return Ok(());
    }

    println!("📚 Usage Patterns\n");
    for pattern in patterns {
        println!("🎯 {}\n", pattern.name);
        println!("{}\n", pattern.description);

        println!("Steps:");
        for (i, step) in pattern.steps.iter().enumerate() {
            println!("  {}. {}", i + 1, step);
        }

        println!("Commands: {}\n", pattern.commands.join(", "));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_creation() {
        let help_system = HelpSystem::new();
        assert!(!help_system.commands.is_empty());
        assert!(help_system.get_command_help("init").is_some());
    }

    #[test]
    fn test_command_help() {
        let help_system = HelpSystem::new();
        let init_help = help_system.get_command_help("init").unwrap();
        assert_eq!(init_help.name, "init");
        assert!(!init_help.description.is_empty());
        assert!(!init_help.examples.is_empty());
    }

    #[test]
    fn test_search_functionality() {
        let help_system = HelpSystem::new();
        let results = help_system.search("init");
        assert!(!results.is_empty());

        let results = help_system.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_usage_patterns() {
        let help_system = HelpSystem::new();
        let patterns = help_system.get_usage_patterns(Some(UsageCategory::GettingStarted));
        assert!(!patterns.is_empty());

        for pattern in patterns {
            assert_eq!(pattern.category, UsageCategory::GettingStarted);
        }
    }
}
