# Quickstart Guide: Sprite - Multi-Agent Workflow Toolkit

**Date**: 2025-10-18
**Purpose**: Get started with Sprite multi-agent workflow toolkit in under 10 minutes

## Prerequisites

Before you begin, ensure you have the following installed:

- **Git** (version 2.5 or later)
- **Tmux** (version 3.2 or later)
- **yq** (YAML processor)
- **Rust toolchain** (if installing from source)

**Installation Commands**:
```bash
# macOS
brew install git tmux yq

# Ubuntu/Debian
sudo apt-get update
sudo apt-get install git tmux yq

# Verify installations
git --version
tmux -V
yq --version
```

## Installation

### Option 1: Pre-compiled Binary (Recommended)

Download the latest release for your platform:

```bash
# Linux (x86_64)
curl -L https://github.com/your-org/sprite/releases/latest/download/sprite-linux-x86_64.tar.gz | tar xz
sudo mv sprite /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/your-org/sprite/releases/latest/download/sprite-darwin-x86_64.tar.gz | tar xz
sudo mv sprite /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/your-org/sprite/releases/latest/download/sprite-darwin-aarch64.tar.gz | tar xz
sudo mv sprite /usr/local/bin/
```

### Option 2: Install from Source

```bash
cargo install --git https://github.com/your-org/sprite.git
```

### Option 3: Cargo Install (Developers)

```bash
cargo install sprite
```

## 5-Minute Quickstart

### Step 1: Initialize Your Project

Navigate to your git repository and initialize the multi-agent environment:

```bash
cd your-project
sprite init
```

This creates:
- `agents/agents.yaml` - Agent configuration file
- `agents/scripts/` - Management scripts
- `agents/profiles/` - Tmux layout profiles
- `.envrc` - Environment configuration (if direnv is installed)

### Step 2: Configure Your Agents

Edit the configuration file to define your agents:

```bash
# Open the configuration file
sprite config edit
```

Example configuration:
```yaml
agents:
  1:
    branch: agents/1
    model: claude-sonnet-4
    description: "Frontend development"
  2:
    branch: agents/2
    model: claude-sonnet-4
    description: "Backend development"
  3:
    branch: agents/3
    model: claude-sonnet-4
    description: "Testing & QA"

session:
  name: "my-project-session"
  profile: profile0

sync:
  auto_sync: false
  conflict_resolution: manual
```

### Step 3: Create Agent Branches

Create the branches for your agents:

```bash
git checkout main
git checkout -b feature/frontend-work
git checkout main
git checkout -b feature/backend-work
git checkout main
git checkout -b feature/testing-work
git checkout main
```

### Step 4: Start the Session

Launch the multi-agent supervision session:

```bash
sprite start
```

This opens a tmux session with:
- A supervisor control pane
- Individual panes for each agent
- Real-time observability of all agent activities

### Step 5: Assign Work to Agents

Send tasks to specific agents:

```bash
# Send task to agent 1 (frontend)
sprite hey 1 "implement user authentication UI"

# Send task to agent 2 (backend)
sprite hey 2 "create user authentication API"

# Send command to all agents
sprite send "git status"
```

### Step 6: Monitor Progress

Check the status of all agents:

```bash
sprite status
```

Output:
```
SESSION: my-project-session (Active)
LAYOUT: tiled

AGENT           BRANCH                STATUS      LAST ACTIVITY
1               agents/1               Active      30 seconds ago
2               agents/2               Active      1 minute ago
3               agents/3               Inactive    --
```

### Step 7: Sync Work

When agents complete work, sync changes:

```bash
# In main worktree - pull latest changes
sprite sync

# In agent worktree - merge main changes
sprite warp frontend-agent
sprite sync
```

## Common Workflows

### Workflow 1: Feature Development

1. **Setup**: Initialize project and configure agents
2. **Assign**: Distribute feature work across agents
3. **Monitor**: Observe progress in real-time
4. **Sync**: Periodically synchronize work
5. **Integrate**: Merge completed features
6. **Cleanup**: Remove agent workspaces when done

### Workflow 2: Bug Fixing

```bash
# Create bug fix agent
sprite add bugfix-agent --branch hotfix/security-patch

# Start session
sprite start

# Assign bug fix
sprite hey bugfix-agent "fix security vulnerability in auth module"

# Monitor and sync
sprite status
sprite sync

# Clean up when done
sprite remove bugfix-agent --merge-branch
```

### Workflow 3: Code Review

```bash
# Send review tasks to agents
sprite send "git diff main..HEAD"

# Get detailed review from specific agent
sprite hey senior-agent "review changes in feature/auth-branch"

# Collect all reviews
sprite send "git add . && git commit -m 'review feedback'"
```

## Navigation Tips

### Workspace Navigation

```bash
# Jump to agent workspace
sprite warp frontend-agent

# List all workspaces
sprite warp --list

# Return to main directory
cd ../../  # or sprite warp main
```

### Session Navigation

```bash
# Attach to session
sprite attach

# Focus on specific agent pane
sprite zoom frontend-agent

# Return to normal layout
sprite zoom --unzoom

# Detach from session (keep it running)
# Press Ctrl+B then D in tmux
```

## Configuration Examples

### Basic Web Project

```yaml
version: "1.0"
agents:
  - name: "ui-dev"
    branch: "feature/ui-improvements"
    role: "frontend"
  - name: "api-dev"
    branch: "feature/api-endpoints"
    role: "backend"

session:
  name: "web-dev-session"
  layout: "main-vertical"

sync:
  auto_sync: true
  conflict_resolution: "manual"
```

### Large Team Setup

```yaml
version: "1.0"
agents:
  - name: "feature-lead"
    branch: "feature/new-feature"
    role: "coordination"
  - name: "frontend-dev-1"
    branch: "feature/frontend-component"
    role: "frontend"
  - name: "frontend-dev-2"
    branch: "feature/frontend-styles"
    role: "frontend"
  - name: "backend-dev-1"
    branch: "feature/api-changes"
    role: "backend"
  - name: "backend-dev-2"
    branch: "feature/database-schema"
    role: "backend"
  - name: "qa-agent"
    branch: "feature/qa-testing"
    role: "testing"

session:
  name: "team-session"
  layout: "tiled"

sync:
  auto_sync: false
  conflict_resolution: "auto-theirs"
```

## Troubleshooting

### Common Issues

**Issue**: `Error: Not a git repository`
```bash
# Solution: Initialize git repository
git init
git add .
git commit -m "Initial commit"
sprite init
```

**Issue**: `Error: tmux session not found`
```bash
# Solution: Start a new session
sprite start

# Or list existing sessions
sprite attach --list
```

**Issue**: Agent workspace doesn't exist
```bash
# Solution: Re-initialize agents
sprite install

# Or manually create workspace
git worktree add .agents/agent-name feature/branch-name
```

**Issue**: Merge conflicts during sync
```bash
# Solution: Resolve conflicts manually
sprite warp agent-name
# Resolve conflicts in files
git add .
git commit
sprite sync
```

### Getting Help

```bash
# General help
sprite --help

# Command-specific help
sprite start --help
sprite sync --help

# Show configuration
sprite config show

# Validate setup
sprite config validate
```

### Debug Mode

Enable verbose output for troubleshooting:

```bash
sprite --verbose start
sprite -v sync
```

## Best Practices

### 1. Agent Organization

- Use descriptive agent names (`frontend-auth`, `api-users`)
- Assign clear roles to agents
- Keep agent workspaces focused on specific features

### 2. Branch Management

- Use consistent naming conventions (`feature/description`, `hotfix/description`)
- Clean up branches when agents complete work
- Use descriptive commit messages

### 3. Session Management

- Use meaningful session names
- Clean up sessions when work is complete
- Save important session layouts as templates

### 4. Synchronization

- Sync regularly to avoid conflicts
- Resolve conflicts promptly
- Use appropriate conflict resolution strategies

### 5. Performance

- Limit active agents to 10 or fewer
- Clean up unused agent workspaces
- Monitor resource usage with `sprite status`

## Next Steps

Now that you're familiar with the basics:

1. **Explore Advanced Features**: Learn about custom layouts, hooks, and automation
2. **Integrate with IDE**: Configure your editor to work with agent workspaces
3. **Team Collaboration**: Set up shared configurations for your team
4. **Automation**: Create scripts for common workflows
5. **Monitoring**: Set up logging and alerting for production use

For more detailed information, see:
- [CLI API Contracts](contracts/cli-api.md)
- [Data Model](data-model.md)
- [Configuration Reference](docs/configuration.md)

## Support

- **Documentation**: https://docs.sprite.dev
- **Issues**: https://github.com/your-org/sprite/issues
- **Discussions**: https://github.com/your-org/sprite/discussions
- **Community**: https://discord.gg/sprite