# Sprite Configuration Schema

This document describes the current configuration schema for Sprite multi-agent toolkit.

## Version

Current configuration version: `"1.0"`

## Top-Level Structure

```yaml
version: "1.0"
agents: [...]
session_name: string
sync: {...}
settings: {...}
```

## Agent Configuration

Each agent in the `agents` array follows this structure:

```yaml
- id: 'string'                    # Unique identifier (alphanumeric, hyphens, underscores)
  branch: 'string'                # Git branch name for this agent
  worktree_path: 'string'         # Local workspace path (default: "agents/{id}")
  model: 'string'                 # AI model (e.g., "claude-sonnet-4", "gpt-4")
  description: 'string'           # Role or specialization description
  status: Inactive|Active|Paused|Error|Initializing  # Current agent status
  config:                         # Agent-specific configuration
    env_vars: {}                  # Environment variables (key: value)
    work_dir: null                 # Default working directory within workspace
    startup_commands: []           # Commands to run when agent starts
    resource_limits:               # Resource constraints
      max_memory_mb: 1024         # Maximum memory usage in MB
      max_cpu_percent: 80         # Maximum CPU percentage
      operation_timeout_secs: 300  # Timeout for operations in seconds
      max_concurrent_ops: 3        # Maximum concurrent operations
      max_disk_mb: 5120           # Maximum disk usage in MB
    shell: null                    # Preferred shell (default: bash)
    default_timeout_secs: 300      # Default timeout for commands
    auto_sync: false               # Whether to auto-sync changes
    custom_settings: {}            # Custom agent settings
```

## Session Configuration

```yaml
session_name: 'string'            # tmux session name
```

## Sync Configuration

```yaml
sync:
  auto_sync: false                # Enable automatic synchronization
  default_interval_secs: 300       # Sync interval in seconds
  conflict_resolution: Manual       # Manual|AutoTheirs|AutoOurs
  exclude_branches:                # Branches to exclude from sync
  - main
  - master
  pre_sync_hooks: []               # Commands to run before sync
  post_sync_hooks: []              # Commands to run after sync
```

## Settings Configuration

```yaml
settings:
  default_shell: bash              # Default shell for agents
  global_env_vars: {}              # Global environment variables
  logging:                         # Logging configuration
    log_file: 'agents/logs/sprite.log'
    level: Info                    # Error|Warn|Info|Debug|Trace
    log_to_stdout: true
    rotation:
      max_size_mb: 100            # Maximum log file size
      max_files: 5                 # Number of log files to keep
  performance:                     # Performance settings
    max_concurrent_ops: 10         # Global concurrent operations limit
    default_timeout_secs: 300      # Default timeout
    enable_monitoring: true        # Enable performance monitoring
    memory_limit_mb: 1024         # Memory limit for sprite itself
    cpu_limit_percent: 80          # CPU limit for sprite itself
  security:                        # Security settings
    allowed_paths:                 # Paths agents can access
    - agents/
    blocked_commands:              # Commands agents cannot run
    - rm -rf
    - sudo
    - su
    - chmod 777
    strict_path_validation: true    # Enforce path validation
    max_command_length: 1000       # Maximum command length
    allow_shell_execution: false    # Allow shell execution
```

## Resource Limits

### Agent Resource Limits
- `max_memory_mb`: Optional<u64> - Maximum memory per agent
- `max_cpu_percent`: Option<u8> - Maximum CPU per agent (0-100)
- `operation_timeout_secs`: Option<u64> - Timeout per operation
- `max_concurrent_ops`: Option<u32> - Max concurrent operations per agent
- `max_disk_mb`: Option<u64> - Maximum disk usage per agent

## Agent Status Values

- `Inactive`: Configured but not running
- `Active`: Currently running and responsive
- `Paused`: Temporarily suspended
- `Error(string)`: Encountered an error (string contains error message)
- `Initializing`: Starting up

## Conflict Resolution Strategies

- `Manual`: User must resolve conflicts manually
- `AutoTheirs`: Automatically accept their version of conflicts
- `AutoOurs`: Automatically accept our version of conflicts

## Log Levels

- `Error`: Only errors
- `Warn`: Warnings and errors
- `Info`: General information (default)
- `Debug`: Debug information
- `Trace`: Detailed trace information

## Example Configuration

```yaml
version: "1.0"

agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4'
  description: 'Frontend development'
  status: 'Inactive'
  config:
    env_vars:
      NODE_ENV: development
      DEBUG: true
    work_dir: null
    startup_commands:
    - 'npm install'
    - 'npm run dev'
    resource_limits:
      max_memory_mb: 2048
      max_cpu_percent: 75
      operation_timeout_secs: 600
      max_concurrent_ops: 5
      max_disk_mb: 10240
    shell: null
    default_timeout_secs: 300
    auto_sync: true
    custom_settings:
      editor: vim
      language: typescript

- id: '2'
  branch: 'agent-2'
  worktree_path: 'agents/2'
  model: 'claude-sonnet-4'
  description: 'Backend development'
  status: 'Inactive'
  config:
    env_vars:
      RAILS_ENV: development
    work_dir: null
    startup_commands:
    - 'bundle install'
    - 'rails server'
    resource_limits:
      max_memory_mb: 1536
      max_cpu_percent: 80
      operation_timeout_secs: 300
      max_concurrent_ops: 3
      max_disk_mb: 5120
    shell: null
    default_timeout_secs: 300
    auto_sync: true
    custom_settings:
      database: postgresql
      language: ruby

session_name: 'my-project'

sync:
  auto_sync: true
  default_interval_secs: 180
  conflict_resolution: 'Manual'
  exclude_branches:
  - main
  - develop
  pre_sync_hooks: []
  post_sync_hooks:
  - 'echo "Sync completed"'

settings:
  default_shell: bash
  global_env_vars:
    PROJECT_NAME: my_project
    DEBUG: true
  logging:
    log_file: 'agents/logs/sprite.log'
    level: 'Info'
    log_to_stdout: true
    rotation:
      max_size_mb: 50
      max_files: 10
      compress: true
  performance:
    max_concurrent_ops: 8
    default_timeout_secs: 300
    enable_monitoring: true
    memory_limit_mb: 2048
    cpu_limit_percent: 85
  security:
    allowed_paths:
    - agents/
    - shared/
    - config/
    blocked_commands:
    - rm -rf /
    - sudo rm
    - dd if=/dev/zero
    - format
    strict_path_validation: true
    max_command_length: 2000
    allow_shell_execution: true
```

## Migration from Older Versions

If you have an older configuration, note these changes:

### Version Changes
- `0.2.x` → `1.0`: Major schema restructure
  - Moved from `session.name` to `session_name`
  - Added comprehensive `settings` section
  - Updated `resource_limits` with `operation_timeout_secs` field
  - Added `performance` and `security` subsections

### Field Changes
- `agents[].config.timeout_secs` → `agents[].config.default_timeout_secs`
- `agents[].config.timeout` → `agents[].config.resource_limits.operation_timeout_secs`
- `session.name` → `session_name`
- Added `agents[].config.startup_commands` (was missing)
- Added `agents[].config.custom_settings` (was missing)

### Validation Rules
- Agent IDs must be unique and non-empty
- Branch names must be valid git branch names
- At least one agent must be configured (validation requirement)
- Resource limits must be positive numbers where applicable
- Log levels must be valid values
- SECURITY: blocked_commands helps prevent destructive operations

## Best Practices

1. **Memory Allocation**: Set `max_memory_mb` based on your system resources
2. **CPU Limits**: Keep `max_cpu_percent` below 90 to maintain system responsiveness
3. **Timeout Values**: Set higher timeouts for complex operations, lower for quick tasks
4. **Security**: Use `blocked_commands` to prevent destructive operations
5. **Monitoring**: Enable `enable_monitoring` for production environments
6. **Log Rotation**: Configure appropriate log rotation to prevent disk filling

## Troubleshooting

### Common Validation Errors

1. **"At least one agent must be configured"**
   - Ensure the `agents` array contains at least one agent configuration

2. **"Missing required field: operation_timeout_secs"**
   - Add `operation_timeout_secs` to each agent's `resource_limits`

3. **"Agent ID already exists"**
   - Ensure each agent has a unique `id` value

4. **"Invalid log level"**
   - Use one of: Error, Warn, Info, Debug, Trace

5. **"Invalid agent status"**
   - Use one of: Inactive, Active, Paused, Error, Initializing
