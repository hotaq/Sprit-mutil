# Sprite Multi-Agent Workflow Toolkit 🤖

A powerful CLI tool that lets you manage multiple AI coding agents simultaneously, each working in isolated environments. Perfect for collaborative AI development, parallel task processing, and organized workflow management.

## 🎯 What Problems Does Sprite Solve?

**Tired of managing multiple AI sessions manually?** Sprite automates the entire workflow:

- **🔄 Parallel Development**: Run multiple AI agents simultaneously, each with their own workspace
- **🏠 Isolated Environments**: Each agent works in a separate git worktree with dedicated branches
- **📊 Session Management**: Organize and monitor all your AI sessions from one interface
- **🔧 Auto-Provisioning**: Automatically sets up workspaces, branches, and tmux sessions
- **💾 Smart Recovery**: Detects and fixes issues with orphaned sessions or broken workspaces

## 🚀 Quick Start

### Installation Comparison

| Method | Command | Speed | Dependencies | Best For |
|--------|---------|-------|--------------|----------|
| **UV-Style** | `uv tool install .` | ⚡ Fast | Python + uv | Python developers |
| **Binary Install** | `cargo binstall hotaq/sprit-mutil` | ⚡ Fast | Rust + cargo-binstall | Quick setup |
| **Traditional** | `cargo install --git https://github.com/hotaq/Sprit-mutil.git` | 🐢 Slow | Rust + cargo | Full control |
| **Smart Script** | `curl ... install-uv.sh \| bash` | 🚀 Auto | Varies | Most users |

### One-Command Installation (Modern)

```bash
# Method 1: UV-Style (like 'uv tool install specify-cli')
git clone https://github.com/hotaq/Sprit-mutil.git && cd Sprit-mutil
uv tool install .

# Method 2: Smart Installer (detects best method)
curl -fsSL https://raw.githubusercontent.com/hotaq/Sprit-mutil/main/scripts/install-uv.sh | bash

# Method 3: Binary Installation (no compilation)
cargo binstall hotaq/sprit-mutil

# Method 4: Traditional Cargo
cargo install --git https://github.com/hotaq/Sprit-mutil.git

# Verify installation
sprite --version
```

### Your First Multi-Agent Project

```bash
# 1. Navigate to your project (must be a git repository)
cd /path/to/your/project

# 2. Initialize with 3 AI agents
sprite init --agents 3

# 3. Start your multi-agent session
sprite start

# 4. Attach to the session
sprite attach sprite-session
```

**That's it!** You now have 3 AI agents working in parallel, each in their own isolated workspace.

## 🛠️ How It Works: The Solution

### The Problem Sprite Solves

Before Sprite:
```bash
# Manual setup for each agent
git worktree add agent1 main
cd agent1
tmux new-session -d -s agent1
# Repeat for agent2, agent3...
```

After Sprite:
```bash
sprite init --agents 3
sprite start
# Everything is automated!
```

### Architecture Overview

```
Your Project/
├── agents/
│   ├── agents.yaml      # Configuration file
│   ├── 1/               # Agent 1 workspace (git worktree)
│   │   └── (your project files)
│   ├── 2/               # Agent 2 workspace
│   ├── 3/               # Agent 3 workspace
│   ├── scripts/         # Helper scripts
│   └── profiles/        # Tmux layouts
├── src/                 # Your main project
└── README.md
```

### Each Agent Gets:
- **Isolated Workspace**: Separate git worktree with dedicated branch
- **Independent Session**: Individual tmux pane/window
- **Resource Limits**: Configurable memory and CPU constraints
- **Task Isolation**: Changes don't affect other agents until you merge

## 📋 Essential Commands

### Daily Workflow Commands

```bash
# Start a new session with all agents
sprite start

# List active sessions
sprite attach --list

# Attach to an existing session
sprite attach sprite-session

# Check system health
sprite status

# End a session safely
sprite kill sprite-session
```

### Agent Management

```bash
# List all agents and their status
sprite agents list

# Create a new agent
sprite agents create 4 --description "Documentation agent"

# Show detailed agent information
sprite agents show 1

# Validate all agent workspaces
sprite agents validate

# Fix workspace issues
sprite agents provision
```

### Configuration Management

```bash
# View complete configuration
sprite config show

# Get specific setting
sprite config get agents.1.model

# Change a setting
sprite config set session_name "my-session"

# Validate configuration
sprite config validate
```

## 🎮 Session Layouts

Choose the perfect layout for your workflow:

```bash
# Tiled layout (default) - best for monitoring all agents
sprite start --layout tiled

# Focus mode - concentrate on one agent at a time
sprite start --layout focus

# Vertical split - agents on left, supervisor on right
sprite start --layout vertical

# Dashboard - 6-panel overview
sprite start --layout dashboard
```

## 💡 Real-World Use Cases

### Use Case 1: Collaborative Code Development
```bash
# Agent 1: Frontend development
# Agent 2: Backend API development
# Agent 3: Testing and documentation

sprite init --agents 3
sprite start --layout tiled
# All agents work simultaneously on different aspects
```

### Use Case 2: Code Review and Refactoring
```bash
# Agent 1: Reviews code for bugs
# Agent 2: Suggests improvements
# Agent 3: Implements refactoring

sprite agents create reviewer --description "Code reviewer"
sprite agents create improver --description "Code improver"
sprite agents create refactor --description "Refactoring agent"
sprite start
```

### Use Case 3: Research and Documentation
```bash
# Agent 1: Research task
# Agent 2: Documentation writing
# Agent 3: Example code generation

sprite init --agents 3
# Each agent researches and documents different topics
```

## 🔧 Configuration File

Sprite uses a simple YAML file (`agents/agents.yaml`):

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
    resource_limits:
      max_memory_mb: 1024
      max_cpu_percent: 80
      operation_timeout_secs: 300

session_name: sprite-session
settings:
  logging:
    level: info
    log_to_stdout: true
  security:
    allowed_paths:
    - agents/
```

## 🚨 Troubleshooting Common Issues

### "Not a git repository"
```bash
# Solution: Always run from within a git repository
cd /path/to/your/git/project
sprite init
```

### "Session already exists"
```bash
# Solution: Kill existing session or use different name
sprite kill sprite-session
# OR
sprite start --session new-session
```

### "Agent workspace on wrong branch"
```bash
# Solution: Fix workspace provisioning
sprite agents provision
```

### High memory usage
```bash
# Solution: Check status and clean up
sprite status --cleanup
# Adjust resource limits in agents/agents.yaml
```

## 📦 Installation Methods

### Method 1: UV-Style Install (Recommended)
```bash
# Like 'uv tool install' - most modern approach
git clone https://github.com/hotaq/Sprit-mutil.git && cd Sprit-mutil
uv tool install .

# Or using our smart installer
curl -fsSL https://raw.githubusercontent.com/hotaq/Sprit-mutil/main/scripts/install-uv.sh | bash
```

### Method 2: Using Cargo
```bash
# Binary installation (no compilation)
cargo binstall hotaq/sprit-mutil

# From GitHub (latest)
cargo install --git https://github.com/hotaq/Sprit-mutil.git

# Note: There's a name conflict on crates.io, so we use GitHub
```

### Method 3: Package Managers
```bash
# macOS with Homebrew
brew install sprite

# Arch Linux with AUR
paru -S sprite-git

# Nix/NixOS
nix-env -iA nixpkgs.sprite

# Windows with Scoop
scoop install sprite
```

### Method 4: Pre-compiled Binary
```bash
# Linux
curl -L https://github.com/hotaq/Sprit-mutil/releases/latest/download/sprite-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv sprite /usr/local/bin/

# macOS
curl -L https://github.com/hotaq/Sprit-mutil/releases/latest/download/sprite-x86_64-apple-darwin.tar.gz | tar xz
sudo mv sprite /usr/local/bin/
```

## 🏗️ System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Git**: Required for workspace management
- **Tmux**: Required for session management
- **Rust 1.75+**: Only needed for building from source
- **Bash**: Required for installation scripts

## 🎯 Tips and Best Practices

### For Best Performance
1. **Start with 2-3 agents** and add more as needed
2. **Use appropriate layouts** for your workflow
3. **Regular status checks** with `sprite status`
4. **Clean up sessions** when done with `sprite kill`

### For Team Collaboration
1. **Share configuration** via `agents/agents.yaml`
2. **Use descriptive agent names** and purposes
3. **Set resource limits** to prevent system overload
4. **Regular workspace validation** with `sprite agents validate`

### For Development Workflow
1. **Integrate with CI/CD** for automated testing
2. **Use git branches** effectively for parallel development
3. **Monitor system resources** during long sessions
4. **Backup configurations** for team consistency

## 🤝 Contributing

We welcome contributions! Here's how to get started:

```bash
# Clone and set up for development
git clone https://github.com/hotaq/Sprit-mutil.git
cd Sprit-mutil
cargo build
cargo test

# Make your changes
# ... code changes ...

# Run tests and linting
cargo test
cargo clippy
cargo fmt

# Submit a pull request
```

### Areas to Contribute:
- 🆕 New agent models and integrations
- 🎨 Additional session layouts and themes
- 📊 Enhanced monitoring and metrics
- 🐛 Bug fixes and performance improvements
- 📚 Documentation improvements

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Documentation**: <https://docs.rs/sprite>
- **Repository**: <https://github.com/hotaq/Sprit-mutil>
- **Issues**: <https://github.com/hotaq/Sprit-mutil/issues>
- **Releases**: <https://github.com/hotaq/Sprit-mutil/releases>

## 🚀 Quick Reference

```bash
# Essential workflow
sprite init --agents 3    # Setup
sprite start              # Start session
sprite attach sprite-session  # Join session
sprite status             # Check health
sprite kill sprite-session  # End session

# Agent management
sprite agents list         # See agents
sprite agents create 4     # Add agent
sprite agents show 1       # Agent details

# Configuration
sprite config show         # View config
sprite config get key      # Get setting
sprite config set key val  # Change setting
```

---

**Sprite transforms how you work with AI agents** - from manual session management to automated, organized, and efficient multi-agent workflows.

*Start your multi-agent journey today! 🚀*