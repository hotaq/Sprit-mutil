# Data Model: Sprite - Multi-Agent Workflow Toolkit

**Date**: 2025-10-18
**Feature**: Sprite Multi-Agent Workflow Toolkit
**Purpose**: Define core data entities and their relationships for the Sprite multi-agent management system

## Core Entities

### 1. Agent Configuration

Represents an AI coding agent with its workspace configuration and operational parameters.

**Fields**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier for the agent (number or name)
    pub id: String,

    /// Git branch name for this agent's work
    pub branch: String,

    /// AI model to use (e.g., "claude-sonnet-4", "gpt-4")
    pub model: Option<String>,

    /// Role or specialization description
    pub description: Option<String>,

    /// Local workspace path (defaults to agents/{id})
    pub worktree_path: Option<PathBuf>,

    /// Current status of the agent
    pub status: AgentStatus,

    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,

    /// tmux pane identifier
    pub tmux_pane: Option<String>,

    /// Agent-specific configuration
    pub config: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is configured but not running
    Inactive,
    /// Agent is running and active
    Active,
    /// Agent is paused/suspended
    Paused,
    /// Agent encountered an error
    Error(String),
    /// Agent is being setup/initialized
    Initializing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Shell environment variables
    pub env_vars: HashMap<String, String>,

    /// Default working directory within workspace
    pub work_dir: Option<PathBuf>,

    /// Startup commands to run when agent starts
    pub startup_commands: Vec<String>,

    /// Resource limits
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,

    /// Maximum CPU percentage
    pub max_cpu_percent: Option<u8>,

    /// Timeout for operations in seconds
    pub operation_timeout_secs: Option<u64>,
}
```

**Validation Rules**:
- `name`: Must be unique, alphanumeric with hyphens/underscores, 1-50 characters
- `branch`: Must be valid git branch name, must exist in repository
- `workspace`: Must be within `.agents/` directory, cannot escape project root
- `role`: Optional, if present must be from predefined list or free text

**State Transitions**:
```
Initializing → Active
Initializing → Error
Active → Paused
Active → Error
Active → Inactive
Paused → Active
Paused → Inactive
Error → Initializing (retry)
```

### 2. Tmux Profile Configuration

Represents pre-configured tmux layout profiles for different agent arrangements.

**Fields**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxProfile {
    /// Profile identifier (profile0 - profile5)
    pub name: String,

    /// Profile description and use case
    pub description: String,

    /// Best for number of agents
    pub best_for_agent_count: Option<u32>,

    /// Layout type and configuration
    pub layout: ProfileLayout,

    /// Shell script path for this profile
    pub script_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileLayout {
    /// Layout pattern: main-horizontal, main-vertical, even-horizontal, tiled
    pub layout_type: String,

    /// Pane size specifications
    pub pane_sizes: Vec<PaneSize>,

    /// Root pane configuration (main repository)
    pub root_pane: Option<RootPaneConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneSize {
    /// Agent identifier
    pub agent_id: String,

    /// Percentage or size specification
    pub size_spec: String,

    /// Position in layout
    pub position: LayoutPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutPosition {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootPaneConfig {
    /// Whether root pane is included
    pub enabled: bool,

    /// Root pane position
    pub position: LayoutPosition,

    /// Root pane size
    pub size: String,
}

### 3. Session Configuration

Represents the tmux session that hosts all agent panes and provides supervision interface.

**Fields**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub name: String,

    /// Current session status
    pub status: SessionStatus,

    /// Tmux profile being used
    pub profile: String,

    /// Active profile configuration
    pub profile_config: TmuxProfile,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,

    /// Session-specific settings
    pub settings: SessionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is being created
    Creating,
    /// Session is active and running
    Active,
    /// Session is detached but still running
    Detached,
    /// Session is being terminated
    Terminating,
    /// Session encountered an error
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLayout {
    /// Layout type: tiled, main-vertical, main-horizontal, custom
    pub layout_type: LayoutType,

    /// Pane arrangement specifications
    pub pane_config: Vec<PaneConfig>,

    /// Session dimensions
    pub dimensions: Option<SessionDimensions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    /// All panes arranged in a grid
    Tiled,
    /// One main vertical pane with smaller panes on the side
    MainVertical,
    /// One main horizontal pane with smaller panes below
    MainHorizontal,
    /// Custom layout configuration
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneConfig {
    /// Agent name for this pane
    pub agent_name: String,

    /// Pane size/position specifications
    pub size: PaneSize,

    /// Pane index in layout
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    /// Supervisor pane position
    pub position: SupervisorPosition,

    /// Supervisor pane size
    pub size: SupervisorSize,

    /// Default working directory for supervisor
    pub work_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisorPosition {
    Top,
    Bottom,
    Left,
    Right,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    /// Whether to auto-sync agents
    pub auto_sync: bool,

    /// Sync interval in seconds
    pub sync_interval_secs: Option<u64>,

    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,

    /// Log level
    pub log_level: LogLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    Manual,
    AutoTheirs,
    AutoOurs,
}
```

**Validation Rules**:
- `name`: Must be unique, valid tmux session name
- `layout_type`: Must be supported layout type
- `pane_config`: All agents must have corresponding pane configuration
- `conflict_resolution`: Must be valid resolution strategy

### 3. Project Configuration

Top-level configuration that defines the entire multi-agent setup.

**Fields**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Configuration version for migration support
    pub version: String,

    /// List of configured agents
    pub agents: Vec<Agent>,

    /// Session configuration
    pub session: Session,

    /// Synchronization settings
    pub sync: SyncConfig,

    /// Global project settings
    pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Whether to automatically sync agents
    pub auto_sync: bool,

    /// Default sync interval in seconds
    pub default_interval_secs: u64,

    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,

    /// Branches to exclude from auto-sync
    pub exclude_branches: Vec<String>,

    /// Pre-sync hooks
    pub pre_sync_hooks: Vec<SyncHook>,

    /// Post-sync hooks
    pub post_sync_hooks: Vec<SyncHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHook {
    /// Hook command to execute
    pub command: String,

    /// Arguments for the command
    pub args: Vec<String>,

    /// Working directory for the hook
    pub work_dir: Option<PathBuf>,

    /// Whether hook failure should stop sync
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Default shell for agent operations
    pub default_shell: String,

    /// Global environment variables
    pub global_env_vars: HashMap<String, String>,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Performance settings
    pub performance: PerformanceSettings,

    /// Security settings
    pub security: SecuritySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log file location
    pub log_file: Option<PathBuf>,

    /// Log level
    pub level: LogLevel,

    /// Whether to log to stdout
    pub log_to_stdout: bool,

    /// Log rotation settings
    pub rotation: Option<LogRotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    /// Maximum log file size in MB
    pub max_size_mb: u64,

    /// Maximum number of log files to keep
    pub max_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Maximum concurrent operations
    pub max_concurrent_ops: u32,

    /// Timeout for operations in seconds
    pub default_timeout_secs: u64,

    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Allowed workspace paths
    pub allowed_paths: Vec<PathBuf>,

    /// Blocked commands
    pub blocked_commands: Vec<String>,

    /// Whether to validate paths strictly
    pub strict_path_validation: bool,
}
```

### 4. Command Message

Represents commands sent from supervisor to agents.

**Fields**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMessage {
    /// Unique message identifier
    pub id: String,

    /// Message timestamp
    pub timestamp: DateTime<Utc>,

    /// Target agent name (empty for broadcast)
    pub target_agent: Option<String>,

    /// Command to execute
    pub command: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Working directory for command
    pub work_dir: Option<PathBuf>,

    /// Environment variables for this command
    pub env_vars: HashMap<String, String>,

    /// Timeout for this command
    pub timeout_secs: Option<u64>,

    /// Message priority
    pub priority: MessagePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}
```

### 5. Execution Result

Represents the result of command execution.

**Fields**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Corresponding command message ID
    pub command_id: String,

    /// Agent that executed the command
    pub agent_name: String,

    /// Execution status
    pub status: ExecutionStatus,

    /// Command output (stdout)
    pub output: String,

    /// Error output (stderr)
    pub error_output: String,

    /// Exit code
    pub exit_code: i32,

    /// Execution start time
    pub start_time: DateTime<Utc>,

    /// Execution end time
    pub end_time: DateTime<Utc>,

    /// Resource usage during execution
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,

    /// Memory usage in MB
    pub memory_mb: u64,

    /// Disk usage in MB
    pub disk_mb: u64,
}
```

## Entity Relationships

```
ProjectConfig
├── Agent[] (1..*)
│   ├── AgentConfig
│   └── AgentStatus
├── Session
│   ├── SessionLayout
│   │   └── PaneConfig[] (1..*)
│   └── SupervisorConfig
├── SyncConfig
│   ├── SyncHook[] (0..*)
│   └── ConflictResolution
└── ProjectSettings
    ├── LoggingConfig
    ├── PerformanceSettings
    └── SecuritySettings

CommandMessage → ExecutionResult (1:1)
Agent → CommandMessage[] (1:*)
Agent → ExecutionResult[] (1:*)
```

## Data Integrity Constraints

1. **Uniqueness Constraints**:
   - Agent names must be unique within project
   - Session names must be unique globally
   - Command message IDs must be unique

2. **Referential Integrity**:
   - All pane_config.agent_name must reference existing agents
   - All command messages must reference valid target agents
   - All execution results must reference valid command messages

3. **Path Constraints**:
   - All workspace paths must be within `agents/` directory
   - No path traversal allowed (`../` components)
   - Absolute paths required for workspaces

4. **State Consistency**:
   - Agent status transitions must follow defined state machine
   - Session status must be consistent with tmux session state
   - Active agents must have valid tmux pane assignments

## Data Persistence

**Primary Storage**: `agents/agents.yaml` - Human-readable configuration file

### Configuration File Format (agents.yaml)

The configuration follows the proven format from the reference implementation:

```yaml
# agents/agents.yaml
agents:
  1:
    branch: agents/1
    worktree_path: agents/1
    model: claude-sonnet-4
    description: "Core backend"

  2:
    branch: agents/2
    worktree_path: agents/2
    model: claude-sonnet-4
    description: "Frontend UI"

  3:
    branch: agents/3
    worktree_path: agents/3
    model: claude-sonnet-4
    description: "Testing & QA"

session:
  name: sprite-session
  profile: profile0

sync:
  auto_sync: false
  conflict_resolution: manual
```

**Runtime State**: `.agents/state/` directory containing:
- `session.json` - Current session state
- `agents/` - Per-agent state files
- `logs/` - Execution logs and command history
- `cache/` - Cached information for performance

**Backup Strategy**:
- Configuration files are version-controlled in git
- Runtime state is regenerated from configuration
- Command history is rotated and compressed

## Migration Strategy

**Version Support**: Configuration includes version field for migrations

**Backward Compatibility**:
- Unknown fields are ignored during parsing
- Default values provided for new fields
- Migration functions handle version upgrades

**Validation**: All configuration changes are validated before application