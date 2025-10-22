# Research Findings: Multi-Agent Workflow Toolkit

**Date**: 2025-10-18
**Feature**: Multi-Agent Workflow Toolkit
**Purpose**: Research findings to resolve technical decisions and constitution requirements

## 1. CLI Accessibility Standards

### Decision: Implement comprehensive CLI accessibility following WCAG principles

**Rationale**: Based on research into CLI accessibility best practices, we will implement a fully accessible CLI tool that works with screen readers and assistive technologies.

**Key Requirements**:
- Use standard terminal output that screen readers can interpret
- Provide text alternatives for all visual indicators
- Ensure color is never the sole information carrier
- Implement comprehensive help system with examples
- Support NO_COLOR environment variable
- Provide verbose/quiet output modes
- Use clear, actionable error messages with suggestions

**Implementation Strategy**:
```rust
// Error handling with thiserror for clear messages
use thiserror::Error;

#[derive(Error, Debug)]
enum CliError {
    #[error("Configuration file not found at path: {path}")]
    ConfigNotFound { path: String },

    #[error("Invalid configuration: {details}. Suggestion: {suggestion}")]
    InvalidConfig {
        details: String,
        suggestion: String
    },
}

// Color handling respecting accessibility
fn should_use_color() -> bool {
    env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout)
}
```

**Alternatives Considered**: Minimal accessibility approach (rejected - would violate constitution requirements)

## 2. Rust CLI Distribution Patterns

### Decision: Use dual distribution strategy (cargo install + pre-compiled binaries)

**Rationale**: To reach both developers and general users, we need multiple installation methods.

**Chosen Approach**:
1. **Developer Installation**: `cargo install --git <repo_url>`
2. **General User Installation**: Pre-compiled binaries via GitHub Actions releases
3. **Package Managers**: Consider homebrew, apt, chocolatey in future

**Implementation Details**:
- Set up GitHub Actions workflow for automatic binary builds
- Support Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Windows (x86_64)
- Use GitHub releases for distribution
- Provide installation scripts for convenience

**Benefits**:
- Low barrier to entry for all users
- Automatic updates with cargo
- No Rust toolchain required for general users
- Cross-platform compatibility

**Alternatives Considered**: Docker distribution (rejected - adds complexity), Python wheels (rejected - doesn't match our Rust stack)

## 3. Error Handling Best Practices

### Decision: Use anyhow for error propagation with thiserror for custom error types

**Rationale**: Provides the best balance of developer experience and user-friendly error messages.

**Implementation Strategy**:
```rust
use anyhow::{Context, Result};
use thiserror::Error;

// Custom error types for domain-specific errors
#[derive(Error, Debug)]
enum AgentError {
    #[error("Agent '{name}' not found")]
    AgentNotFound { name: String },

    #[error("Git worktree creation failed: {source}")]
    WorktreeCreationFailed { source: String },
}

// Use anyhow for error propagation in application logic
fn create_agent_workspace(name: &str) -> Result<()> {
    std::fs::create_dir(format!(".agents/{}", name))
        .with_context(|| format!("Failed to create workspace for agent: {}", name))?;
    Ok(())
}
```

**Error Message Guidelines**:
- Be specific and actionable
- Include file paths and line numbers when relevant
- Provide suggested solutions
- Use consistent formatting
- Include exit codes for scripting

**Alternatives Considered**: Pure anyhow (rejected - less structured), custom error handling (rejected - too much boilerplate)

## 4. Tmux Integration Patterns

### Decision: Use tmux control mode for programmatic session management

**Rationale**: Tmux control mode provides the most reliable way to manage sessions programmatically while maintaining compatibility.

**Implementation Approach**:
```rust
use std::process::Command;

fn create_tmux_session(session_name: &str) -> Result<()> {
    let output = Command::new("tmux")
        .args(&["new-session", "-d", "-s", session_name])
        .output()
        .context("Failed to create tmux session")?;

    if !output.status.success() {
        anyhow::bail!("Tmux session creation failed: {}",
                     String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

fn send_command_to_pane(session: &str, pane: &str, command: &str) -> Result<()> {
    Command::new("tmux")
        .args(&["send-keys", "-t", &format!("{}:{}", session, pane), command, "C-m"])
        .status()
        .context("Failed to send command to tmux pane")?;

    Ok(())
}
```

**Session Management Strategy**:
- Create dedicated session for multi-agent supervision
- Use naming convention: `mutilagent-[timestamp]`
- Implement session cleanup on exit
- Support session reconnection

**Alternatives Considered**: Direct terminal manipulation (rejected - too complex and fragile), expect script (rejected - less reliable)

## 5. Git Worktree Management

### Decision: Use libgit2 bindings for Rust with shell script fallbacks

**Rationale**: Provides type safety while maintaining compatibility with complex git operations.

**Implementation Strategy**:
```rust
use git2::{Repository, Worktree, WorktreeAddOptions};

fn create_agent_worktree(repo: &Repository, agent_name: &str, branch: &str) -> Result<Worktree> {
    let worktree_path = format!(".agents/{}", agent_name);

    let mut opts = WorktreeAddOptions::new();
    opts.branch(repo.find_branch(branch, git2::BranchType::Local)?);

    repo.worktree(&agent_name, &worktree_path, Some(&mut opts))
        .context("Failed to create git worktree for agent")
}
```

**Worktree Management Rules**:
- Each agent gets isolated worktree in `.agents/{agent-name}`
- Worktree names match agent names for consistency
- Implement proper cleanup on agent removal
- Validate worktree integrity before operations

**Fallback Strategy**: For complex operations, use shell scripts to leverage git CLI directly

**Alternatives Considered**: Pure shell script approach (rejected - less type safety), pure libgit2 (rejected - may miss edge cases)

## 6. Configuration Management

### Decision: Use serde_yaml with strict validation and schema

**Rationale**: Provides type safety, validation, and good error messages for configuration files.

**Configuration Schema**:
```yaml
# .agents/agents.yaml
version: "1.0"
agents:
  - name: "agent-1"
    branch: "feature/agent-1-work"
    role: "frontend"
    workspace: ".agents/agent-1"
  - name: "agent-2"
    branch: "feature/agent-2-work"
    role: "backend"
    workspace: ".agents/agent-2"

session:
  name: "mutilagent-session"
  layout: "tiled"  # tiled, main-vertical, main-horizontal

sync:
  auto_sync: false
  conflict_resolution: "manual"  # manual, auto-theirs, auto-ours
```

**Rust Structures**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct AgentConfig {
    name: String,
    branch: String,
    role: Option<String>,
    workspace: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SessionConfig {
    name: String,
    layout: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    version: String,
    agents: Vec<AgentConfig>,
    session: SessionConfig,
    sync: SyncConfig,
}
```

**Validation Strategy**:
- Schema validation on load
- Required field validation
- Git branch existence validation
- Worktree path validation

## 7. Performance Considerations

### Decision: Implement asynchronous operations for I/O-heavy tasks

**Rationale**: Ensures responsive CLI even during lengthy operations.

**Implementation Strategy**:
```rust
use tokio::process::Command;
use futures::future::join_all;

async fn create_all_agent_worktrees(agents: Vec<AgentConfig>) -> Result<()> {
    let tasks: Vec<_> = agents.into_iter()
        .map(|agent| tokio::spawn(async move {
            create_single_agent_worktree(agent).await
        }))
        .collect();

    let results = join_all(tasks).await;

    for result in results {
        result??; // Handle both task and operation errors
    }

    Ok(())
}
```

**Performance Targets**:
- CLI commands: <3 seconds (95th percentile)
- Initialization: <60 seconds
- Session start: <10 seconds
- Agent communication: <1 second

**Resource Management**:
- Memory usage: <100MB
- CPU usage: Minimal during idle
- Disk usage: Efficient worktree management

## 8. Security Considerations

### Decision: Implement principle of least privilege for all operations

**Security Measures**:
- Validate all file paths to prevent directory traversal
- Use absolute paths for all git operations
- Implement proper cleanup for temporary files
- Validate tmux session names to prevent injection
- Restrict shell command execution to predefined scripts

**Path Validation**:
```rust
use std::path::{Path, PathBuf};

fn validate_agent_path(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path);

    // Ensure path is within .agents directory
    if !path.starts_with(".agents") {
        anyhow::bail!("Agent path must be within .agents directory");
    }

    // Prevent directory traversal
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        anyhow::bail!("Agent path cannot contain parent directory references");
    }

    Ok(path)
}
```

## Summary of Decisions

All research areas have been resolved with clear technical decisions:

1. ✅ **CLI Accessibility**: Comprehensive accessibility implementation with WCAG principles
2. ✅ **Distribution Strategy**: Dual approach with cargo install + pre-compiled binaries
3. ✅ **Error Handling**: anyhow + thiserror combination for optimal UX
4. ✅ **Tmux Integration**: Control mode with programmatic session management
5. ✅ **Git Worktrees**: libgit2 with shell script fallbacks
6. ✅ **Configuration**: serde_yaml with strict validation
7. ✅ **Performance**: Async operations with clear targets
8. ✅ **Security**: Principle of least privilege with path validation

The research phase is complete and all technical decisions support the constitution requirements, particularly addressing the previously identified accessibility concern.