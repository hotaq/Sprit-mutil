# Sprite Multi-Agent Workflow Toolkit 🤖

A robust command-line toolkit for managing multiple AI coding agents in isolated tmux sessions with comprehensive workspace management and health monitoring.

## 🌟 Features

- **Multi-Agent Management**: Create, configure, and manage multiple AI agents simultaneously
- **Workspace Provisioning**: Automatic git worktree setup for each agent with branch isolation
- **Session Management**: Robust tmux-based session handling with health monitoring
- **Configuration Management**: YAML-based configuration with real-time change detection
- **Health Monitoring**: Comprehensive session health analysis with automatic recovery
- **Resource Cleanup**: Automatic cleanup of orphaned sessions and temporary files
- **CLI Integration**: Intuitive command-line interface with helpful progress indicators

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- Git
- Tmux
- Bash shell

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd mutilagent

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Initialize a New Project

```bash
# Initialize a new multi-agent environment
sprite init

# Start a session with all agents
sprite start

# List available sessions
sprite attach --list

# Attach to a session
sprite attach sprite-session
```

## 📋 Command Reference

### Configuration Management

```bash
# Show complete configuration
cargo run -- config show

# Get specific configuration values
cargo run -- config get agents.1.model
cargo run -- config get session_name

# Set configuration values
cargo run -- config set session_name "my-session"

# Validate configuration and workspaces
cargo run -- config validate

# Check configuration status and changes
cargo run -- config status

# Expected: Shows configuration metadata and recent changes
```

### Agent Management

```bash
# List all configured agents
cargo run -- agents list

# Expected: Lists agents with workspace status and health indicators

# Create new agent
cargo run -- agents create 4 --description "Specialized agent" --model claude-sonnet-4

# Expected: Creates agent 4 with workspace, branch, and configuration

# Show agent details
cargo run -- agents show 1

# Expected: Detailed agent information including workspace status and git state

# Validate all agent workspaces
cargo run -- agents validate

# Expected: Comprehensive workspace validation with health reports

# Provision missing workspaces
cargo run -- agents provision

# Expected: Creates missing workspaces and fixes branch synchronization

# Remove agent
cargo run -- agents remove 4 --force

# Expected: Removes agent configuration and optionally workspace
```

### Session Management

```bash
# Start new multi-agent session
cargo run -- start

# Expected: Creates tmux session with all configured agents

# Start with custom layout
cargo run -- start --layout tiled --session my-session

# Expected: Creates session with specified layout profile

# List active sessions
cargo run -- attach --list

# Expected: Shows all active tmux sessions with details

# Attach to session
cargo run -- attach sprite-session

# Expected: Connects to the specified tmux session

# Kill session
cargo run -- kill sprite-session

# Expected: Terminates session and performs cleanup
```

#### Status and Health Monitoring

```bash
# Check overall system status
cargo run -- status

# Expected: Comprehensive health report including:
# - Session state analysis
# - Agent workspace validation
# - Resource utilization monitoring
# - Configuration consistency checks
# - Recovery recommendations

# Check session-specific status
cargo run -- status --session sprite-session

# Expected: Focused status for specific session

# Continuous monitoring mode
cargo run -- status --watch

# Expected: Real-time status updates with automatic refresh
```

#### Session Recovery and Cleanup

```bash
# Recover orphaned sessions
cargo run -- status --recover

# Expected: Automatic recovery of orphaned tmux sessions
# - Detects orphaned sessions
# - Attempts graceful reconnection
# - Provides recovery options

# Cleanup resources
cargo run -- status --cleanup

# Expected: Cleanup of temporary files and orphaned resources
# - Removes temporary worktrees
# - Cleans up log files
# - Recovers disk space
```

## 🏗️ Architecture and System Design

### Multi-Agent Architecture

Sprite implements a sophisticated multi-agent architecture with the following components:

#### 1. Configuration Management Layer
- **YAML-based Configuration**: Human-readable configuration with comprehensive validation
- **Dynamic Configuration Detection**: Real-time monitoring of configuration changes
- **Version Control Integration**: Seamless integration with git worktrees for branch isolation

#### 2. Workspace Management System
- **Isolated Workspaces**: Each agent operates in a dedicated git worktree
- **Branch Isolation**: Agents work on separate branches to prevent conflicts
- **Resource Allocation**: Configurable resource limits and constraints per agent

#### 3. Session Management Framework
- **Tmux Integration**: Robust session handling using tmux terminal multiplexer
- **Health Monitoring**: Continuous monitoring of session health and agent status
- **Recovery Mechanisms**: Automatic recovery from session failures and orphaned resources

#### 4. Communication Infrastructure
- **Command Distribution**: Efficient command broadcasting to individual agents
- **Result Collection**: Centralized collection of agent execution results
- **Error Handling**: Comprehensive error reporting and recovery strategies

### Data Flow Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Commands  │───▶│  Configuration  │───▶│  Workspace      │
│                 │    │   Manager       │    │  Provisioner    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                       │
                                ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Session       │◀───│   Health        │◀───│   Agent         │
│   Manager       │    │   Monitor       │    │   Workspaces    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### State Management

Sprite maintains comprehensive state tracking across multiple dimensions:

- **Configuration State**: Tracks configuration file changes and updates
- **Workspace State**: Monitors git repository status and branch synchronization
- **Session State**: Maintains tmux session health and agent connectivity
- **Resource State**: Tracks system resource utilization and limits

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

## 🧪 Development and Testing

### Development Environment Setup

```bash
# Clone the repository
git clone <repository-url>
cd mutilagent

# Install development dependencies
rustup update stable
cargo install cargo-watch cargo-test

# Run development server with hot reload
cargo watch -x run

# Run tests with coverage
cargo test --all-features
```

### Testing Strategy

Sprite implements a comprehensive testing strategy across multiple levels:

#### 1. Unit Testing
- **Configuration Management Tests**: Validate YAML parsing and configuration detection
- **Git Operations Tests**: Test worktree creation and branch management
- **Session Management Tests**: Validate tmux integration and health monitoring
- **Error Handling Tests**: Comprehensive error scenario testing

#### 2. Integration Testing
- **End-to-End Workflow Tests**: Complete workflow validation from init to cleanup
- **Multi-Agent Session Tests**: Multi-agent collaboration and isolation testing
- **Resource Management Tests**: Resource limit enforcement and cleanup validation
- **Recovery Testing**: Session recovery and cleanup mechanism testing

#### 3. Performance Testing
- **Concurrent Operations**: Multi-agent concurrent operation performance
- **Resource Utilization**: Memory and CPU usage under various loads
- **Scalability Testing**: Performance with increasing numbers of agents

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test config
cargo test session_management
cargo test workspace_provisioning

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release

# Generate test coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Code Quality and Style

```bash
# Format code
cargo fmt

# Run clippy for linting
cargo clippy -- -D warnings

# Run security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

### Configuration File Structure

The project uses a comprehensive YAML configuration system located at `agents/agents.yaml`:

```yaml
version: '1.0'
agents:
- id: '1'
  branch: agents/1
  worktree_path: agents/1
  model: claude-sonnet-4
  description: Agent 1 workspace
  status: inactive
  config:
    env_vars: {}
    work_dir: null
    startup_commands: []
    resource_limits:
      max_memory_mb: 1024
      max_cpu_percent: 80
      operation_timeout_secs: 300
      max_concurrent_ops: 3
      max_disk_mb: 5120
    shell: null
    default_timeout_secs: 300
    auto_sync: false
    custom_settings: {}

session_name: sprite-session
sync:
  auto_sync: false
  default_interval_secs: 300
  conflict_resolution: manual
  exclude_branches:
  - main
  - master
  pre_sync_hooks: []
  post_sync_hooks: []

settings:
  default_shell: bash
  global_env_vars: {}
  logging:
    log_file: agents/logs/sprite.log
    level: info
    log_to_stdout: true
    rotation:
      max_size_mb: 100
      max_files: 5
  performance:
    max_concurrent_ops: 10
    default_timeout_secs: 300
    enable_monitoring: true
    memory_limit_mb: 1024
    cpu_limit_percent: 80
  security:
    allowed_paths:
    - agents/
    blocked_commands:
    - rm -rf
    - sudo
    - su
    - chmod 777
    strict_path_validation: true
    max_command_length: 1000
    allow_shell_execution: false
```

## 🔍 Monitoring and Observability

### Health Monitoring System

Sprite includes a comprehensive health monitoring system that tracks:

#### Session Health Metrics
- **Session Status**: Active, inactive, orphaned, or recovering
- **Agent Connectivity**: Real-time agent connection status
- **Resource Utilization**: Memory, CPU, and disk usage per agent
- **Error Rates**: Tracking of errors and recovery actions

#### Workspace Health Metrics
- **Git Repository Status**: Clean, modified, or conflicted state
- **Branch Synchronization**: Status of branch synchronization across agents
- **Workspace Integrity**: Validation of workspace structure and permissions

#### System Health Metrics
- **Configuration Consistency**: Validation of configuration across components
- **Resource Availability**: System resource availability and limits
- **Performance Metrics**: Response times and throughput measurements

### Logging and Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- <command>

# Enable trace logging for detailed troubleshooting
RUST_LOG=trace cargo run -- <command>

# Log to file
RUST_LOG=info cargo run -- <command> 2> sprite.log

# View real-time logs
tail -f agents/logs/sprite.log
```

### Performance Profiling

```bash
# Install profiling tools
cargo install cargo-flamegraph

# Generate flame graph
cargo flamegraph --bin sprite -- <command>

# Memory profiling
valgrind --tool=massif cargo run -- <command>
```

## 🚨 Troubleshooting and Support

### Common Issues and Solutions

#### 1. Initialization Issues

**Problem**: `Not a git repository` error
```bash
# Solution: Initialize from within a git repository
cd /path/to/your/git/repo
cargo run -- init
```

**Problem**: `Workspace already exists` error
```bash
# Solution: Use force flag or clean up existing workspace
cargo run -- init --force
# OR
rm -rf agents/
cargo run -- init
```

#### 2. Session Management Issues

**Problem**: `Session already exists` error
```bash
# Solution: Kill existing session or use different name
cargo run -- kill sprite-session
# OR
cargo run -- start --session new-session-name
```

**Problem**: Cannot attach to session
```bash
# Solution: Check session status and recover if needed
cargo run -- status --recover
cargo run -- attach sprite-session
```

#### 3. Workspace Issues

**Problem**: Agent workspace on wrong branch
```bash
# Solution: Provision workspaces to fix branch issues
cargo run -- agents provision
# OR
cargo run -- agents provision <agent-id>
```

**Problem**: Git worktree errors
```bash
# Solution: Ensure clean git state
git status
git stash  # If needed
cargo run -- agents provision
```

#### 4. Performance Issues

**Problem**: High memory usage
```bash
# Solution: Check resource limits and cleanup
cargo run -- status --cleanup
# Adjust agent resource limits in agents/agents.yaml
```

**Problem**: Slow session startup
```bash
# Solution: Check system resources and workspace status
cargo run -- status
free -h
df -h
```

### Debug Mode and Diagnostics

```bash
# Comprehensive system diagnostics
RUST_LOG=debug cargo run -- status --diagnostics

# Session-specific diagnostics
RUST_LOG=debug cargo run -- status --session sprite-session --verbose

# Workspace validation diagnostics
cargo run -- agents validate --verbose
```

### Getting Help

```bash
# General help
cargo run -- --help

# Command-specific help
cargo run -- agents --help
cargo run -- config --help
cargo run -- status --help

# Version information
cargo run -- --version
```

### Community Support

- **Issues**: Report bugs and feature requests on GitHub Issues
- **Discussions**: Join community discussions on GitHub Discussions
- **Documentation**: Check the wiki for detailed guides and tutorials
- **Examples**: Explore the examples/ directory for use cases

## 🤝 Contributing

We welcome contributions from the community! Here's how you can help:

### Contribution Guidelines

1. **Fork the Repository**: Create a fork of the project on GitHub
2. **Create a Feature Branch**: Use descriptive branch names
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make Your Changes**: Follow the coding standards and write tests
4. **Run Tests**: Ensure all tests pass
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
5. **Commit Your Changes**: Use descriptive commit messages
   ```bash
   git commit -m "feat: add your feature description"
   ```
6. **Push to Your Fork**: Push changes to your fork
   ```bash
   git push origin feature/your-feature-name
   ```
7. **Create a Pull Request**: Submit a pull request with a clear description

### Development Standards

- **Code Style**: Follow `cargo fmt` and `cargo clippy` recommendations
- **Testing**: Maintain test coverage above 80%
- **Documentation**: Update documentation for new features
- **Commits**: Use conventional commit messages (feat:, fix:, docs:, etc.)

### Areas for Contribution

- **New Agent Models**: Support for additional AI models and services
- **Session Layouts**: Additional tmux layout profiles and configurations
- **Monitoring Tools**: Enhanced monitoring and alerting capabilities
- **Performance**: Performance optimizations and resource usage improvements
- **Documentation**: Documentation improvements and examples
- **Testing**: Additional test cases and edge case handling

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### License Summary

- ✅ **Commercial use**: You can use this software in commercial products
- ✅ **Modification**: You can modify the source code
- ✅ **Distribution**: You can distribute modified and original versions
- ✅ **Private use**: You can use this software privately
- ⚠️ **Liability**: The software is provided "as-is" without warranty
- ⚠️ **Copyright**: You must include the original copyright notice

## 🙏 Acknowledgments

Built with passion for improving developer workflows and multi-agent AI collaboration.

### Core Technologies

- **[Rust](https://www.rust-lang.org/)**: Systems programming language for performance and safety
- **[Clap](https://clap.rs/)**: Command-line argument parsing framework
- **[Serde](https://serde.rs/)**: Serialization and deserialization framework
- **[Tmux](https://github.com/tmux/tmux)**: Terminal multiplexer for session management
- **[Git](https://git-scm.com/)**: Version control system for workspace management

### Inspiration and References

- Multi-agent coordination patterns and distributed systems research
- Modern CLI design principles and user experience best practices
- Container orchestration concepts applied to development environments
- Workspace isolation techniques from virtualization and sandboxing

---

## 🚀 Quick Reference

### Essential Commands
```bash
# Initialize project
sprite init --agents 3

# Start session
sprite start

# List sessions
sprite attach --list

# Attach to session
sprite attach sprite-session

# Check status
sprite status

# Stop session
sprite kill sprite-session
```

### Configuration File Location
- **Main Config**: `agents/agents.yaml`
- **Logs**: `agents/logs/sprite.log`
- **Workspaces**: `agents/1/`, `agents/2/`, `agents/3/`

### Getting Started Checklist
- [ ] Install Rust 1.75+
- [ ] Install Git and Tmux
- [ ] Clone repository
- [ ] Run `cargo build --release`
- [ ] Initialize with `sprite init`
- [ ] Start session with `sprite start`
- [ ] Explore agent workspaces
- [ ] Check status with `sprite status`

---

*Sprite Multi-Agent Workflow Toolkit - Empowering developers with intelligent multi-agent collaboration tools.* 🤖
