# Sprite Workflow Guide

Understanding how to use Sprite effectively with tmux.

## Understanding the Components

### What is Tmux?

Tmux (Terminal Multiplexer) creates **virtual terminals** inside your terminal. It lets you:
- Run multiple terminal sessions in one window
- Detach and reattach to sessions
- Keep processes running in the background

### What is Sprite?

Sprite **uses tmux** to manage multiple AI agents. Each agent gets its own tmux pane/window.

## The Two Main Workflows

### Workflow 1: Interactive (Recommended for Daily Use)

**Best for:** Working with agents directly

```bash
# 1. Open a REAL terminal
#    - macOS: Terminal.app or iTerm2
#    - Linux: GNOME Terminal, Konsole, etc.
#    - NOT VS Code terminal, NOT IDE terminal

# 2. Navigate to your project
cd /path/to/your/project

# 3. Start sprite (auto-attaches)
sprite start

# 4. You're now inside tmux!
#    - See all your agent panes
#    - Switch between agents
#    - Work with multiple agents simultaneously
```

**Inside tmux:**
- `Ctrl+B D` - Detach (go back to normal terminal)
- `Ctrl+B [arrow]` - Switch panes
- `Ctrl+B c` - New window
- `Ctrl+B n` - Next window

### Workflow 2: Background (For Automation)

**Best for:** CI/CD, scripts, running in background

```bash
# 1. Create session in background (from anywhere)
sprite start --force --detach

# 2. Session runs independently
#    - Agents are working
#    - No terminal needed
#    - Survives terminal close

# 3. Later, attach from real terminal
#    Open Terminal.app, then:
tmux attach -t sprite-session
# OR
sprite attach
```

## Common Issues & Solutions

### Issue 1: "not a terminal" Error

**Error:**
```
sprite attach
Error: open terminal failed: not a terminal
```

**Cause:** You're in an IDE terminal (VS Code, etc.)

**Solution:**
```bash
# Option A: Use real terminal
# 1. Open Terminal.app (macOS) or equivalent
# 2. Then run:
sprite attach

# Option B: Skip attach, just start
sprite start  # Auto-attaches
```

### Issue 2: "Session already exists"

**Error:**
```
sprite start
Error: Session 'sprite-session' already exists
```

**Solutions:**
```bash
# Quick: Replace existing
sprite start --force

# Or: Attach to existing
tmux attach -t sprite-session

# Or: Kill old, start new
sprite kill
sprite start
```

### Issue 3: Lost Session After Terminal Close

**Not an issue!** Tmux keeps running.

**Solution:**
```bash
# Just reattach
tmux attach -t sprite-session
```

## Step-by-Step Examples

### Example 1: First Time Setup

```bash
# 1. Install sprite
cargo install --git https://github.com/hotaq/Sprit-mutil

# 2. Open Terminal.app (real terminal!)

# 3. Go to your git project
cd ~/projects/my-app

# 4. Initialize sprite
sprite init --agents 3

# 5. Start working
sprite start
# You're now in tmux with 3 agents!

# 6. When done, detach
Ctrl+B D

# 7. Later, reattach
sprite attach
# (must be from real terminal)
```

### Example 2: Quick Daily Workflow

```bash
# Morning:
cd ~/projects/my-app
sprite start --force  # Kills old, starts fresh

# Work with agents...

# Evening:
Ctrl+B D  # Detach, agents keep running

# Next day:
sprite attach  # Continue where you left off
```

### Example 3: CI/CD Workflow

```bash
#!/bin/bash
# ci-test.sh

cd /project
sprite init --agents 2
sprite start --force --detach

# Run your tests...
# Agents work in background

sprite kill  # Clean up
```

## Terminal Compatibility

### ✅ Works (Real Terminals)
- Terminal.app (macOS)
- iTerm2 (macOS)
- GNOME Terminal (Linux)
- Konsole (KDE)
- xterm, rxvt
- Any terminal that supports tmux

### ❌ Doesn't Work (Pseudo Terminals)
- VS Code integrated terminal
- IntelliJ/PyCharm terminal
- Emacs terminal
- Jupyter notebook terminal
- Most IDE terminals

**Workaround:** Use `sprite start --detach` from IDE, then attach from real terminal.

## Understanding Detach vs Attach

### Detached Session
```bash
sprite start --detach
# Creates session in background
# Terminal is free to use
# Agents are running
```

**When to use:**
- Automation
- Want to keep terminal available
- Running in scripts

### Attached Session
```bash
sprite start
# OR
sprite attach
# You're inside tmux
# See all agents
# Interact directly
```

**When to use:**
- Daily interactive work
- Monitoring agents
- Debugging

## Best Practices

### 1. Always Use Real Terminals

```bash
# ❌ Don't do this
# (from VS Code terminal)
sprite attach

# ✅ Do this
# 1. Open Terminal.app
# 2. Then:
sprite attach
```

### 2. Use --force for Fresh Starts

```bash
# Instead of:
sprite kill
sprite start

# Just do:
sprite start --force
```

### 3. Detach, Don't Close

```bash
# ❌ Don't close terminal while in tmux
# Agents will still run, but you lose visual context

# ✅ Detach properly
Ctrl+B D
# Now you can close terminal safely
```

### 4. Check Session Status

```bash
# Before starting
tmux list-sessions
# OR
sprite status

# If session exists
sprite attach
# OR
sprite start --force
```

## Quick Reference

| Action | Command |
|--------|---------|
| Start new session | `sprite start` |
| Start in background | `sprite start --detach` |
| Force restart | `sprite start --force` |
| Attach to session | `sprite attach` |
| Detach from session | `Ctrl+B D` |
| Kill session | `sprite kill` |
| List sessions | `tmux list-sessions` |
| Check status | `sprite status` |

## Tmux Basics

Once inside tmux (after `sprite start`):

| Action | Keys |
|--------|------|
| Detach | `Ctrl+B D` |
| Switch pane | `Ctrl+B [arrow]` |
| New window | `Ctrl+B C` |
| Next window | `Ctrl+B N` |
| Previous window | `Ctrl+B P` |
| List windows | `Ctrl+B W` |
| Kill pane | `Ctrl+B X` |

## Troubleshooting

### Can't attach from VS Code?

**Normal!** Use Terminal.app instead:
```bash
# In VS Code terminal:
sprite start --detach

# Open Terminal.app:
sprite attach
```

### Session keeps existing?

Use `--force`:
```bash
sprite start --force
```

### How to see agents without attaching?

```bash
# List panes
tmux list-panes -t sprite-session

# Check status
sprite status
```

### Tmux not installed?

```bash
# macOS
brew install tmux

# Ubuntu/Debian
sudo apt install tmux

# Then try again
sprite start
```

---

**Remember:** Sprite is a tmux manager. You need a real terminal to attach to tmux sessions!
