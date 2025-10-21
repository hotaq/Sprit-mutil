# Sprite - Multi-Agent Workflow Tool 🚀

> Manage multiple AI coding agents in parallel, each in their own workspace.

## What is Sprite?

Sprite lets you run multiple AI agents at once, each working in isolation. Think of it as having 3+ AI assistants working on different tasks simultaneously.

## Quick Start (3 Steps)

### 1. Install

```bash
cargo install --git https://github.com/hotaq/Sprit-mutil
```

### 2. Setup Your Project

```bash
cd your-project
sprite init --agents 3
```

This creates 3 AI agent workspaces automatically.

### 3. Start Working

```bash
sprite start
```

Done! You now have 3 AI agents ready to work.

---

## Common Commands

### Daily Use

```bash
sprite start              # Start all agents
sprite start --force      # Replace existing session
sprite attach             # Join the session
sprite kill               # Stop all agents
sprite status             # Check system health
```

### Managing Agents

```bash
sprite agents list        # See all agents
sprite agents create 4    # Add a 4th agent
sprite agents validate    # Check if everything is ok
```

### Updates

```bash
sprite update --check     # Check for new version
sprite update             # Update to latest
```

---

## How It Works

When you run `sprite init --agents 3`, Sprite creates:

```
your-project/
├── agents/
│   ├── 1/              ← Agent 1 workspace
│   ├── 2/              ← Agent 2 workspace
│   ├── 3/              ← Agent 3 workspace
│   └── agents.yaml     ← Configuration
└── (your files)
```

Each agent works in their own folder, with their own git branch.

---

## Layouts

Choose how agents are displayed:

```bash
sprite start --layout tiled      # Grid layout (default)
sprite start --layout vertical   # Side by side
sprite start --layout dashboard  # 6-panel view
```

---

## Troubleshooting

**"command not found: sprite"**
```bash
# Restart terminal, or:
export PATH="$HOME/.cargo/bin:$PATH"
```

**"Not a git repository"**
```bash
# Make sure you're in a git project:
cd your-project
git init  # if needed
sprite init --agents 3
```

**"Session already exists"**
```bash
sprite start --force        # Quick: Kill old + start new
# OR
sprite kill sprite-session  # Stop the old one
sprite start                # Start fresh
```

---

## Requirements

- **Git** - for version control
- **Tmux** - for session management
- **Rust** - for installation

Install on macOS:
```bash
brew install git tmux rust
```

Install on Ubuntu:
```bash
sudo apt install git tmux cargo
```

---

## Configuration

Edit `agents/agents.yaml` to customize:

```yaml
agents:
- id: '1'
  model: claude-sonnet-4
  description: Frontend development
  resource_limits:
    max_memory_mb: 1024
    max_cpu_percent: 80

session_name: sprite-session
```

---

## Example Workflows

### Scenario 1: Parallel Development
```bash
# Agent 1: Frontend
# Agent 2: Backend
# Agent 3: Tests

sprite init --agents 3
sprite start
# Each agent works independently
```

### Scenario 2: Code Review
```bash
# Agent 1: Find bugs
# Agent 2: Suggest improvements
# Agent 3: Refactor code

sprite init --agents 3
# Review code from 3 perspectives
```

---

## Tips

✅ **Start small** - Use 2-3 agents first  
✅ **Name agents** - Add descriptions in config  
✅ **Check status** - Run `sprite status` regularly  
✅ **Clean up** - Always `sprite kill` when done  
✅ **Stay updated** - Run `sprite update` monthly  

---

## Getting Help

- **Issues**: https://github.com/hotaq/Sprit-mutil/issues
- **Documentation**: See `docs/` folder
- **Updates**: https://github.com/hotaq/Sprit-mutil/releases

---

## Quick Reference Card

| Command | What it does |
|---------|-------------|
| `sprite init --agents 3` | Setup 3 agents |
| `sprite start` | Start session |
| `sprite attach` | Join session |
| `sprite status` | Check health |
| `sprite kill` | Stop session |
| `sprite agents list` | Show agents |
| `sprite update` | Update Sprite |

---

**That's it!** Start simple, learn as you go. 🎯

## License

MIT License - See LICENSE file
