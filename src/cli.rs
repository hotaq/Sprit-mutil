use clap::{Subcommand, ValueEnum};

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new multi-agent environment
    Init {
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
        /// Number of agents to create
        #[arg(short, long, default_value = "3")]
        agents: u32,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Start supervision session
    Start {
        /// Custom session name
        #[arg(short, long)]
        session_name: Option<String>,
        /// Layout type
        #[arg(short, long, default_value = "tiled")]
        layout: String,
        /// Start and detach immediately
        #[arg(short, long)]
        detach: bool,
    },
    /// Manage agents
    Agents {
        #[command(subcommand)]
        command: AgentsCommands,
    },
    /// Attach to existing session
    Attach {
        /// Session name to attach to
        session_name: Option<String>,
        /// List available sessions
        #[arg(short, long)]
        list: bool,
    },
    /// Terminate session
    Kill {
        /// Session name to kill
        session_name: Option<String>,
        /// Force kill without confirmation
        #[arg(short, long)]
        force: bool,
        /// Kill all sessions
        #[arg(long)]
        all: bool,
    },
    /// Send command to all agents
    Send {
        /// Command to execute
        command: String,
        /// Command arguments
        args: Vec<String>,
        /// Command timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Working directory for command
        #[arg(short, long)]
        work_dir: Option<String>,
        /// Environment variable (KEY=VALUE)
        #[arg(short, long = "env")]
        env_vars: Vec<String>,
        /// Execute sequentially
        #[arg(long)]
        sequential: bool,
    },
    /// Send command to specific agent
    Hey {
        /// Agent number (1, 2, 3, etc.)
        agent: String,
        /// Command to execute
        command: String,
        /// Command arguments
        args: Vec<String>,
        /// Command timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Working directory for command
        #[arg(short, long)]
        work_dir: Option<String>,
        /// Environment variable (KEY=VALUE)
        #[arg(short, long = "env")]
        env_vars: Vec<String>,
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },
    /// Synchronize workspaces
    Sync {
        /// Sync specific agent
        #[arg(short, long)]
        agent: Option<String>,
        /// Force sync even if conflicts
        #[arg(short, long)]
        force: bool,
        /// Conflict resolution strategy
        #[arg(short, long, default_value = "manual")]
        strategy: String,
        /// Show what would be synced
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove agent workspace
    Remove {
        /// Agent number to remove
        agent: String,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
        /// Keep workspace files
        #[arg(long)]
        keep_workspace: bool,
        /// Merge branch before removal
        #[arg(long)]
        merge_branch: bool,
    },
    /// Navigate to agent workspace
    Warp {
        /// Agent number or workspace name
        workspace: Option<String>,
        /// List available workspaces
        #[arg(short, long)]
        list: bool,
        /// Print workspace path without changing
        #[arg(long)]
        print: bool,
        /// Show relative path
        #[arg(long)]
        relative: bool,
    },
    /// Focus on agent pane
    Zoom {
        /// Agent number to zoom on
        agent: Option<String>,
        /// Return to normal layout
        #[arg(short = 'u', long)]
        unzoom: bool,
        /// List available panes
        #[arg(short, long)]
        list: bool,
    },
    /// Check session status and health
    Status {
        /// Check specific session
        session_name: Option<String>,
        /// Clean up old sessions and temporary files
        #[arg(long)]
        cleanup: bool,
        /// Show detailed health information
        #[arg(long)]
        detailed: bool,
    },
    /// Show extended help and documentation
    Guide {
        /// Command to get help for
        command: Option<String>,
        /// Search help content
        #[arg(long)]
        search: Option<String>,
        /// Show usage patterns
        #[arg(long)]
        patterns: bool,
        /// Show troubleshooting guides
        #[arg(long)]
        troubleshooting: bool,
        /// Show quick reference
        #[arg(long)]
        quick: bool,
        /// Show accessible help (screen reader friendly)
        #[arg(long)]
        accessible: bool,
        /// Show usage patterns for specific category
        #[arg(long, value_enum)]
        category: Option<HelpCategory>,
    },
    /// Update sprite to the latest version
    Update {
        /// Only check for updates without installing
        #[arg(long)]
        check: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
        /// Update to specific version
        #[arg(long)]
        version: Option<String>,
    },
}

/// Help system command categories.
#[derive(ValueEnum, Debug, Clone)]
pub enum HelpCategory {
    /// Getting started guides
    GettingStarted,
    /// Daily workflow patterns
    DailyWorkflow,
    /// Troubleshooting guides
    Troubleshooting,
    /// Advanced usage
    Advanced,
    /// Collaboration workflows
    Collaboration,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Validate configuration
    Validate,
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Edit configuration file
    Edit,
    /// Check for configuration changes
    Status,
}

#[derive(Subcommand)]
pub enum AgentsCommands {
    /// List all configured agents
    List,
    /// Create a new agent
    Create {
        /// Agent ID (e.g., "3", "4")
        agent_id: String,
        /// Branch name
        #[arg(short, long)]
        branch: Option<String>,
        /// Workspace path
        #[arg(short, long)]
        workspace: Option<String>,
        /// Model type
        #[arg(short, long, default_value = "claude-sonnet-4")]
        model: String,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Skip workspace creation
        #[arg(long)]
        no_workspace: bool,
    },
    /// Remove an agent
    Remove {
        /// Agent ID to remove
        agent_id: String,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
        /// Keep workspace files
        #[arg(long)]
        keep_workspace: bool,
    },
    /// Show agent details
    Show {
        /// Agent ID (or leave blank for all agents)
        agent_id: Option<String>,
    },
    /// Validate agent workspaces
    Validate,
    /// Provision workspaces for agents
    Provision {
        /// Specific agent ID (or leave blank for all agents)
        agent_id: Option<String>,
    },
}
