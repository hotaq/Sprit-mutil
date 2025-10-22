# CLI API Contracts: Sprite - Multi-Agent Workflow Toolkit

**Date**: 2025-10-18
**Purpose**: Define the command-line interface contracts for the Sprite multi-agent management system

## Core Command Structure

The CLI tool will be named `sprite` and follow the proven command structure from the reference implementation:

```
sprite [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

### Global Options

```bash
--config <PATH>          # Path to configuration file (default: .agents/agents.yaml)
--verbose, -v            # Enable verbose output
--quiet, -q              # Suppress non-error output
--no-color               # Disable colored output
--help, -h               # Show help message
--version                # Show version information
```

## Command Contracts

### 1. Initialization Commands

#### `sprite init`

Initialize a new multi-agent environment in the current directory.

**Usage**:
```bash
sprite init [OPTIONS]
```

**Options**:
```bash
--force, -f              # Overwrite existing configuration
--template <TEMPLATE>    # Use specific template (default: basic)
--agents <COUNT>         # Number of agents to create (default: 3)
```

**Contract**:
- **Preconditions**: Current directory must be a git repository
- **Success**: Creates `agents/` directory with configuration and scripts
- **Failure**: Returns appropriate error code and message
- **Timeout**: 60 seconds maximum
- **Output**: Success message with next steps

#### `sprite install`

Install dependencies and set up the environment (provisions worktrees).

**Usage**:
```bash
sprite install [OPTIONS]
```

**Options**:
```bash
--skip-deps              # Skip dependency checking
--dry-run               # Show what would be installed without installing
```

**Contract**:
- **Preconditions**: `sprite init` must have been run
- **Success**: All dependencies verified, worktrees created, environment configured
- **Failure**: Clear error message with installation instructions
- **Output**: Progress indicators and final status

#### `sprite direnv`

Enable direnv in all worktrees for automatic environment loading.

**Usage**:
```bash
sprite direnv [OPTIONS]
```

**Options**:
```bash
--force, -f              # Overwrite existing .envrc files
```

**Contract**:
- **Preconditions**: `sprite install` must have been run, direnv must be installed
- **Success**: .envrc files created and enabled in all worktrees
- **Output**: direnv status and instructions

### 2. Session Management Commands

#### `sprite start`

Start a new multi-agent supervision session.

**Usage**:
```bash
sprite start [OPTIONS]
```

**Options**:
```bash
--session-name <NAME>    # Custom session name (default: auto-generated)
--layout <LAYOUT>        # Layout type: tiled, main-vertical, main-horizontal
--detach, -d            # Start session and detach immediately
--agents <NAMES>         # Specific agents to start (default: all configured)
```

**Contract**:
- **Preconditions**: Valid configuration, agents configured, dependencies installed
- **Success**: tmux session created with agent panes, returns session info
- **Failure**: Detailed error with troubleshooting steps
- **Output**: Session name, layout info, agent status summary

#### `sprite attach`

Attach to an existing session.

**Usage**:
```bash
sprite attach [SESSION_NAME]
```

**Options**:
```bash
--list, -l              # List available sessions
--force                 # Force attach even if already attached
```

**Contract**:
- **Preconditions**: Session must exist and be running
- **Success**: Attaches to tmux session
- **Failure**: Lists available sessions if attachment fails
- **Output**: Session connection status

#### `sprite kill`

Terminate a running session.

**Usage**:
```bash
sprite kill [SESSION_NAME] [OPTIONS]
```

**Options**:
```bash
--force, -f             # Force kill without confirmation
--all                   # Kill all sessions
```

**Contract**:
- **Preconditions**: Session must exist
- **Success**: Gracefully terminates session, cleans up resources
- **Failure**: Warning if session doesn't exist
- **Output**: Termination confirmation

### 3. Agent Management Commands

#### `sprite agents`

Manage agents (subcommands for agent operations).

**Usage**:
```bash
sprite agents <SUBCOMMAND> [OPTIONS]
```

**Subcommands**:
```bash
sprite agents list          # List all configured agents
sprite agents create N       # Create new agent with number N
sprite agents remove <ID>    # Delete agent worktree
```

**sprite agents list**:
```bash
sprite agents list [OPTIONS]
```
**Options**:
```bash
--status, -s            # Include current status
--json                  # Output in JSON format
--active-only           # Show only active agents
```
**Contract**:
- **Success**: Table or JSON of agent information
- **Output**: Agent ID, branch, status, workspace, last activity

**sprite agents create N**:
```bash
sprite agents create <N> [OPTIONS]
```
**Options**:
```bash
--branch <BRANCH>       # Git branch for agent (default: agents/N)
--model <MODEL>         # AI model to use
--description <DESC>    # Agent description
```
**Contract**:
- **Preconditions**: Valid agent number, git repository
- **Success**: Agent added to configuration, worktree created
- **Failure**: Validation error with specific issues
- **Output**: Agent creation confirmation and next steps

**sprite agents remove <ID>**:
```bash
sprite agents remove <AGENT_ID> [OPTIONS]
```
**Options**:
```bash
--force, -f             # Remove without confirmation
--keep-workspace        # Keep workspace files
--merge-branch          # Merge branch before removal
```
**Contract**:
- **Preconditions**: Agent must exist, not be running
- **Success**: Agent removed, resources cleaned up
- **Failure**: Warning about running agent or uncommitted changes
- **Output**: Removal confirmation

#### `sprite remove <ID>`

Legacy command for removing agent worktree.

**Usage**:
```bash
sprite remove <AGENT_ID> [OPTIONS]
```

**Options**:
```bash
--force, -f             # Remove without confirmation
--keep-workspace        # Keep workspace files
--merge-branch          # Merge branch before removal
```

**Contract**:
- **Preconditions**: Agent must exist, not be running
- **Success**: Agent removed, resources cleaned up
- **Failure**: Warning about running agent or uncommitted changes
- **Output**: Removal confirmation

### 4. Communication Commands

#### `sprite send`

Send a command to all agents (broadcast).

**Usage**:
```bash
sprite send <COMMAND> [ARGS...] [OPTIONS]
```

**Options**:
```bash
--timeout <SECONDS>     # Command timeout (default: 30)
--work-dir <PATH>       # Working directory for command
--env <KEY=VALUE>       # Environment variable (can be used multiple times)
--parallel              # Execute in parallel (default)
--sequential            # Execute sequentially
```

**Contract**:
- **Preconditions**: Active session with agents
- **Success**: Command sent to all agents, results collected
- **Failure**: Error details for failed agents
- **Output**: Per-agent execution results

#### `sprite hey`

Send a command to a specific agent.

**Usage**:
```bash
sprite hey <AGENT_NAME> <COMMAND> [ARGS...] [OPTIONS]
```

**Options**:
```bash
--timeout <SECONDS>     # Command timeout (default: 30)
--work-dir <PATH>       # Working directory for command
--env <KEY=VALUE>       # Environment variable
--interactive, -i       # Interactive mode
```

**Contract**:
- **Preconditions**: Agent must exist and be active
- **Success**: Command executed by specific agent
- **Failure**: Agent not found or command failed
- **Output**: Execution result and output

### 5. Synchronization Commands

#### `sprite sync`

Synchronize workspaces context-aware.

**Usage**:
```bash
sprite sync [OPTIONS]
```

**Options**:
```bash
--agent <NAME>          # Sync specific agent (default: current context)
--force, -f             # Force sync even if conflicts
--strategy <STRATEGY>   # Conflict resolution: manual, auto-theirs, auto-ours
--dry-run               # Show what would be synced
```

**Contract**:
- **Preconditions**: Valid git repository
- **Success**: Workspaces synchronized according to context
- **Failure**: Merge conflicts (unless strategy specified)
- **Output**: Sync status and any conflicts

**Context Behavior**:
- **In main worktree**: `git pull --ff-only origin main`
- **In agent worktree**: `git merge main`
- **Elsewhere**: Error with context guidance

### 6. Navigation Commands

#### `sprite warp`

Navigate between agent workspaces.

**Usage**:
```bash
sprite warp [AGENT_NAME] [OPTIONS]
```

**Options**:
```bash
--list, -l              # List available workspaces
--print                 # Print workspace path without changing
--relative              # Show relative path from current location
```

**Contract**:
- **Success**: Changes to agent workspace directory
- **Failure**: Agent not found or workspace doesn't exist
- **Output**: New current directory path

#### `sprite zoom`

Focus on a specific agent pane in tmux.

**Usage**:
```bash
sprite zoom [AGENT_NAME] [OPTIONS]
```

**Options**:
```bash
--unzoom, -u            # Return to normal layout
--list, -l              # List available panes
```

**Contract**:
- **Preconditions**: Active tmux session
- **Success**: Tmux layout adjusted to focus on agent pane
- **Failure**: Session not active or agent not found
- **Output**: Layout change confirmation

### 7. Configuration Commands

#### `sprite config`

Manage configuration settings.

**Usage**:
```bash
sprite config <SUBCOMMAND> [OPTIONS]
```

**Subcommands**:
```bash
sprite config show           # Show current configuration
sprite config validate       # Validate configuration
sprite config set <KEY> <VALUE>  # Set configuration value
sprite config get <KEY>      # Get configuration value
sprite config edit           # Open configuration in editor
```

**Contract**:
- **Success**: Configuration operation completed
- **Failure**: Validation errors or missing configuration
- **Output**: Configuration details or validation results

#### `sprite status`

Show overall system status.

**Usage**:
```bash
sprite status [OPTIONS]
```

**Options**:
```bash
--detailed, -d         # Include detailed information
--json                  # Output in JSON format
--watch, -w             # Watch mode (updates every 5 seconds)
```

**Contract**:
- **Success**: System status overview
- **Output**: Session status, agent statuses, git status, resource usage

### 8. Utility Commands

#### `sprite version`

Show version information.

**Usage**:
```bash
sprite version [OPTIONS]
```

**Options**:
```bash
--verbose, -v            # Show detailed version information
```

**Contract**:
- **Success**: Version information displayed
- **Output**: Version number, build information, Rust version

#### `sprite uninstall`

Remove sprite from repository (clean up all files and configurations).

**Usage**:
```bash
sprite uninstall [OPTIONS]
```

**Options**:
```bash
--force, -f             # Remove without confirmation
--keep-workspaces       # Keep agent workspaces
```

**Contract**:
- **Preconditions**: Must be run from repository root
- **Success**: All sprite files and configurations removed
- **Failure**: Warning if running session or files can't be removed
- **Output**: Cleanup confirmation

#### `sprite catlab [gist-url]`

Download CLAUDE.md guidelines for AI agent collaboration.

**Usage**:
```bash
sprite catlab [GIST_URL] [OPTIONS]
```

**Options**:
```bash
--force, -f             # Overwrite existing CLAUDE.md files
--agent <ID>            # Download only for specific agent
```

**Contract**:
- **Success**: CLAUDE.md guidelines downloaded to agent workspaces
- **Failure**: Network error or invalid gist URL
- **Output**: Download status and file locations

### 9. Claude Slash Commands

In Claude Code, these slash commands are available:

#### `/sprite.sync`

Sync current worktree with main branch.

**Usage**: `/sprite.sync`

**Contract**:
- **Success**: Current workspace synchronized according to context
- **Output**: Sync status and any conflicts

#### `/sprite.hey <agent> <message>`

Send message to specific agent.

**Usage**: `/sprite.hey 1 "implement user authentication"`

**Contract**:
- **Preconditions**: Agent must exist and session must be active
- **Success**: Message sent to specified agent
- **Output**: Message delivery confirmation

#### `/sprite.zoom <agent>`

Toggle zoom for agent pane.

**Usage**: `/sprite.zoom 1` or `/sprite.zoom root`

**Contract**:
- **Preconditions**: Active tmux session
- **Success**: Tmux layout adjusted to focus on agent pane
- **Output**: Layout change confirmation

#### `/sprite.issue` (alias `/sprite-issue`)

Create a GitHub issue via gh command.

**Usage**: `/sprite.issue "Bug: authentication not working"`

**Contract**:
- **Preconditions**: gh CLI installed and authenticated
- **Success**: GitHub issue created
- **Output**: Issue URL and confirmation

## Error Handling Contracts

### Error Codes

| Code | Meaning | Example |
|------|---------|---------|
| 0    | Success | Command completed successfully |
| 1    | General Error | Invalid arguments, configuration error |
| 2    | Dependency Error | Git/tmux not found |
| 3    | Permission Error | Cannot create files/directories |
| 4    | Network Error | Git operations failed |
| 5    | Session Error | Tmux session operations failed |
| 6    | Agent Error | Agent operations failed |
| 7    | Timeout Error | Command timed out |
| 8    | Conflict Error | Merge conflicts during sync |
| 130  | Interrupted | User pressed Ctrl+C |

### Error Message Format

```
Error: <ERROR_TYPE>

<DESCRIPTION>

Suggestion: <SUGGESTED_RESOLUTION>

Context: <ADDITIONAL_CONTEXT>
```

### Standard Error Scenarios

1. **Git Repository Not Found**:
   ```
   Error: Not a git repository

   This command must be run from within a git repository.

   Suggestion: Run 'git init' to initialize a repository, or navigate to an existing one.

   Context: Current directory: /path/to/directory
   ```

2. **Configuration Not Found**:
   ```
   Error: Configuration file not found

   No .agents/agents.yaml file found in current directory.

   Suggestion: Run 'sprite init' to create a new configuration.

   Context: Expected path: /current/path/.agents/agents.yaml
   ```

3. **Session Not Active**:
   ```
   Error: No active session found

   No tmux session is currently running for this project.

   Suggestion: Run 'sprite start' to create a new session.

   Context: Available sessions: none
   ```

## Output Format Contracts

### Progress Indicators

For long-running operations, show progress:

```
[████████████████████████████████] 100% Creating agent worktrees...
[████████████████████          ] 80% Syncing repositories...
[█████████                      ] 40% Starting tmux session...
```

### Table Format

For list commands, use consistent table format:

```
AGENT     BRANCH           STATUS      LAST ACTIVITY
agent-1   feature/auth     Active      2 minutes ago
agent-2   feature/api      Inactive    1 hour ago
agent-3   main             Error       5 minutes ago
```

### JSON Format

When `--json` flag is used, output structured JSON:

```json
{
  "agents": [
    {
      "name": "agent-1",
      "branch": "feature/auth",
      "status": "Active",
      "last_activity": "2025-10-18T14:30:00Z"
    }
  ],
  "session": {
    "name": "sprite-session-1729274400",
    "status": "Active",
    "layout": "tiled"
  }
}
```

## Performance Contracts

### Response Time Requirements

- **Configuration Commands**: <1 second
- **Status/List Commands**: <2 seconds
- **Session Management**: <5 seconds
- **Agent Communication**: <3 seconds per agent
- **Synchronization**: <30 seconds for typical repositories

### Resource Limits

- **Memory Usage**: <100MB for CLI process
- **Disk Usage**: Minimal, temporary files only
- **CPU Usage**: Low impact during normal operations
- **Network Usage**: Only for git operations

### Concurrency Constraints

- **Maximum Concurrent Agents**: 10
- **Maximum Concurrent Commands**: 5
- **Session Limit**: 1 active session per project
- **Command Queue**: FIFO with priority support

## Accessibility Contracts

### Screen Reader Support

- All output must be readable by screen readers
- Progress indicators must have text alternatives
- Error messages must be clear and actionable
- Use semantic markup in help output

### Keyboard Navigation

- All functionality accessible via keyboard
- Tab completion for commands and arguments
- Standard keyboard shortcuts (Ctrl+C, etc.)
- Arrow key navigation where applicable

### Visual Accessibility

- Respect NO_COLOR environment variable
- Provide high contrast output
- Use text indicators alongside color
- Allow font size adjustment via terminal settings