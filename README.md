# Sprite Multi-Agent Workflow Toolkit

A robust command-line toolkit for managing multiple AI coding agents with isolated workspaces, git branch management, and tmux session orchestration.

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd sprit-mutil

# Build the project
cargo build --release

# Add to PATH (optional)
export PATH=$PWD/target/release:$PATH
```

### Initial Setup

```bash
# Initialize a new Sprite environment with 3 agents
sprite init --agents 3

# Or with custom configuration
sprite init --force --agents 5
```

## 📋 Current Status

### ✅ Phase 1: Project Foundation (Complete)
- Core CLI structure with Clap
- Error handling system
- Logging infrastructure
- Configuration models

### ✅ Phase 2: Core Infrastructure (Complete)
- Git utilities and worktree management
- Tmux session management
- Security validation
- File system utilities

### ✅ Phase 3: User Story 1 - Quick Environment Setup (Complete)
- **T021**: Complete init command logic with progress indicators
- **T022**: Template system with embedded compile-time constants
- **T023**: Directory structure creation with proper permissions
- **T024**: Git repository validation and edge case handling
- **T025**: Template creation (shell scripts, tmux profiles, configuration)
- **T026**: Git repository validation
- **T027**: Template system for configuration, scripts, and profiles
- **T028**: Edge case handling for existing configurations

### ✅ Phase 4: User Story 2 - Agent Configuration Management (Complete)
- **T031**: Complete config command logic in `src/commands/config.rs`
- **T032**: YAML parsing and validation for agent configurations
- **T033**: Git worktree creation in `src/utils/git.rs`
- **T034**: Branch creation and management logic
- **T035**: Workspace validation logic
- **T037**: Agents subcommands (list, create, remove) in `src/commands/agents.rs`
- **T038**: Comprehensive error handling for git operations

## 🛠️ Available Commands

### Environment Management

```bash
# Initialize Sprite environment
sprite init [--force] [--agents <N>]

# Configuration management
sprite config show                    # Show current configuration
sprite config validate                 # Validate configuration
sprite config get <key>               # Get configuration value
sprite config set <key> <value>       # Set configuration value
sprite config edit                    # Edit configuration file

# Agent management
sprite agents list                     # List all configured agents
sprite agents create <id> [options]    # Create new agent
sprite agents remove <id> [options]    # Remove agent
sprite agents show [id]                # Show agent details
sprite agents validate                 # Validate agent workspaces
sprite agents provision [id]           # Provision workspaces
```

### Session Management (Planned)

```bash
# Session control
sprite start [options]                 # Start supervision session
sprite attach [session]                # Attach to existing session
sprite kill [session]                  # Terminate session

# Agent communication
sprite send <command>                  # Send command to all agents
sprite hey <agent> <command>           # Send command to specific agent
```

## 🧪 Testing Current Phase (Phase 4)

### Prerequisites
- Rust 1.75+
- Git repository (Sprite must be run from within a git repo)
- Basic understanding of git worktrees

### Test Suite

#### 1. Environment Setup Testing

```bash
# Test 1: Basic initialization
cargo run -- init --agents 2

# Expected: Creates agents/ directory with:
# - agents/agents.yaml (configuration)
# - agents/1/ and agents/2/ directories
# - agents/scripts/ and agents/profiles/ directories

# Test 2: Configuration validation
cargo run -- config validate

# Expected: Shows configuration validation results

# Test 3: Agent listing
cargo run -- agents list

# Expected: Lists configured agents with workspace status
```

#### 2. Agent Management Testing

```bash
# Test 4: Create new agent
cargo run -- agents create 3 --description "Test agent 3"

# Expected: Creates agent 3 with workspace and branch

# Test 5: Show agent details
cargo run -- agents show 3

# Expected: Shows detailed information about agent 3

# Test 6: Validate workspaces
cargo run -- agents validate

# Expected: Validates all agent workspaces and reports status

# Test 7: Provision workspaces
cargo run -- agents provision

# Expected: Creates missing workspaces and fixes branch issues
```

#### 3. Configuration Management Testing

```bash
# Test 8: Show configuration
cargo run -- config show

# Expected: Displays complete configuration structure

# Test 9: Get configuration values
cargo run -- config get agents.1.model

# Expected: Shows model setting for agent 1

# Test 10: Set configuration values
cargo run -- config set session.name "my-session"

# Expected: Updates session name in configuration

# Test 11: Configuration validation
cargo run -- config validate

# Expected: Validates configuration structure and workspaces
```

#### 4. Edge Case Testing

```bash
# Test 12: Duplicate agent creation
cargo run -- agents create 1

# Expected: Error - Agent 1 already exists

# Test 13: Invalid agent ID
cargo run -- agents create "../invalid"

# Expected: Error - Invalid agent ID format

# Test 14: Force reinitialization
cargo run -- init --force --agents 1

# Expected: Overwrites existing configuration
```

### Expected Test Results

#### ✅ Successful Test Indicators:
- Commands execute without errors
- Configuration files are properly created/updated
- Git worktrees are correctly established
- Agent status reports are accurate
- Error handling provides clear feedback

#### ⚠️ Expected Issues:
- Agent workspaces may show "wrong branch" status initially
- Git worktree operations require clean git state
- Some tmux-related commands are not yet implemented

## 📁 Project Structure

```
sprit-mutil/
├── src/
│   ├── commands/
│   │   ├── init.rs          # Environment initialization
│   │   ├── config.rs        # Configuration management
│   │   ├── agents.rs        # Agent management
│   │   └── mod.rs           # Command exports
│   ├── utils/
│   │   ├── git.rs           # Git utilities
│   │   ├── tmux.rs          # Tmux session management
│   │   └── mod.rs           # Utility exports
│   ├── models/
│   │   ├── agent.rs         # Agent models
│   │   ├── config.rs        # Configuration models
│   │   └── mod.rs           # Model exports
│   ├── cli.rs               # CLI structure
│   ├── error.rs             # Error types
│   └── main.rs              # Application entry
├── agents/                  # Generated workspace
│   ├── agents.yaml          # Agent configuration
│   ├── 1/, 2/, 3/           # Agent workspaces
│   ├── scripts/             # Helper scripts
│   └── profiles/            # Tmux profiles
└── Cargo.toml               # Project dependencies
```

## 🔧 Configuration

### Agent Configuration (agents/agents.yaml)

```yaml
agents:
  '1':
    branch: agents/1
    worktree_path: agents/1
    model: claude-sonnet-4
    description: Agent 1 workspace
  '2':
    branch: agents/2
    worktree_path: agents/2
    model: claude-sonnet-4
    description: Agent 2 workspace

session:
  name: sprite-session
  profile: profile0

sync:
  auto_sync: false
  conflict_resolution: manual
  exclude_branches: []
```

## 🚧 Upcoming Features

### Phase 5: Session Management (Planned)
- Tmux session creation and management
- Multi-agent supervision
- Command broadcasting
- Session persistence

### Phase 6: Agent Communication (Planned)
- Command distribution system
- Result collection
- Error handling and retries
- Performance monitoring

## 🐛 Troubleshooting

### Common Issues

1. **"Not a git repository"**
   - Solution: Run sprite commands from within a git repository

2. **"Workspace already exists"**
   - Solution: Use `--force` flag or remove existing workspace

3. **"Agent already exists"**
   - Solution: Use a different agent ID or remove existing agent first

4. **Git worktree errors**
   - Solution: Ensure git repository is in a clean state

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- <command>

# Build with debug symbols
cargo build

# Run tests
cargo test
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

Built with Rust, Clap, and a passion for improving developer workflows.
