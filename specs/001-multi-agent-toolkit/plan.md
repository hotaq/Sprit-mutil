# Implementation Plan: Sprite - Multi-Agent Workflow Toolkit

**Branch**: `001-multi-agent-toolkit` | **Date**: 2025-10-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-multi-agent-toolkit/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build **Sprite**, a robust command-line toolkit for managing multiple AI coding agents in parallel development workflows. The solution combines a Rust-based CLI interface with shell script orchestration to provide fast performance, reliability, and direct system integration. Key features include single-command initialization, YAML-driven agent configuration, tmux profile-based layouts, individual/broadcast agent communication, context-aware synchronization, and workspace navigation tools. Inspired by proven patterns from Soul-Brews-Studio/multi-agent-workflow-kit.

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: Clap (CLI framework), serde_yaml (configuration parsing), anyhow (error handling)
**Storage**: YAML configuration files (.agents/agents.yaml), Git worktrees for agent isolation
**Testing**: cargo test (Rust built-in testing framework) with 80% minimum code coverage requirement
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool)
**Project Type**: Single project with CLI interface and shell script orchestration
**Performance Goals**: CLI commands complete in under 3 seconds (95% of interactions), toolkit initialization under 60 seconds, support up to 10 simultaneous agents without performance degradation
**Constraints**: <3 seconds response time for user interactions, <100MB memory footprint, requires git v2.5+, tmux v3.2+, yq, uvx dependencies
**Scale/Scope**: Supports up to 10 simultaneous agents, manages entire git repository with multiple worktrees, handles typical 8-hour work sessions with 99% uptime

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Code Quality Compliance
✅ **Clarity and Readability**: Rust's strong typing and Clap's declarative approach provide self-documenting code structure
✅ **Maintainability**: Modular design with separate modules for CLI, config, and commands; small, single-responsibility functions
✅ **Documentation**: All public APIs will be documented with Rust doc comments; README will be maintained
✅ **Peer Review**: All code will require review before merging; focus on correctness, quality, and constitution adherence

### Testing Standards Compliance
✅ **Unit Testing**: Every Rust function will have unit tests with edge cases and error conditions; 80% minimum coverage
✅ **Integration Testing**: Critical workflows (agent setup, session management, sync) will have integration tests
✅ **End-to-End Testing**: Key user journeys (init → configure → start → assign task → sync → cleanup) will have E2E tests
✅ **Regression Prevention**: Bug fixes will include reproducing tests to prevent regressions

### User Experience (UX) Consistency Compliance
✅ **Predictable Patterns**: All CLI commands will follow consistent naming and argument patterns
✅ **User-Centric Feedback**: Clear error messages and success notifications using anyhow for error handling
⚠️ **Accessibility**: CLI accessibility will be addressed through clear error messages and help text; further investigation needed for screen reader compatibility

### Performance Requirements Compliance
✅ **Response Times**: CLI commands designed to complete in under 3 seconds; initialization under 60 seconds
✅ **Resource Efficiency**: Rust's memory efficiency; <100MB memory footprint target
✅ **Scalability**: Support for up to 10 simultaneous agents; horizontal scaling design
✅ **Measurement and Monitoring**: Performance metrics will be tracked; alerts for degradation

### GATE STATUS: ✅ PASSED (Post-Design)
All constitution requirements are fully addressed in the technical approach and design.

**Resolved Accessibility Concern**: Research completed comprehensive CLI accessibility strategy including:
- WCAG principles applied to CLI tools
- Screen reader compatibility with standard terminal output
- Color accessibility with NO_COLOR support and text alternatives
- Clear, actionable error messages with suggestions
- Comprehensive help system with examples
- Keyboard-only operation support

**Constitution Compliance Achieved**:
✅ **Code Quality**: Rust strong typing, modular design, comprehensive documentation
✅ **Testing Standards**: 80% coverage requirement, unit/integration/E2E testing strategy
✅ **UX Consistency**: Predictable CLI patterns, user-centric feedback, full accessibility
✅ **Performance Requirements**: Sub-3s response times, <100MB memory, 10-agent scalability

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── main.rs               # CLI entry point, command routing
├── cli.rs                # Definition of CLI commands using Clap
├── config.rs             # Logic for loading and validating agents.yaml
└── commands/             # Modules for each command's logic
    ├── mod.rs
    ├── init.rs           # Initialization command logic
    ├── install.rs        # Installation command logic
    ├── start.rs          # Session start command logic
    ├── attach.rs         # Session attach command logic
    ├── kill.rs           # Session kill command logic
    ├── send.rs           # Command broadcasting logic
    ├── hey.rs            # Individual agent communication logic
    ├── sync.rs           # Smart synchronization logic
    ├── remove.rs         # Agent removal logic
    ├── warp.rs           # Workspace navigation logic
    ├── zoom.rs           # Tmux pane zoom logic
    └── profiles.rs       # Tmux profile management logic

tests/
├── unit/                 # Unit tests for individual functions
├── integration/          # Integration tests for workflows
└── e2e/                  # End-to-end tests for user journeys

agents/
├── 1/                    # Agent 1 workspace (git worktree)
├── 2/                    # Agent 2 workspace (git worktree)
├── 3/                    # Agent 3 workspace (git worktree)
├── agents.yaml           # User-facing agent configuration
├── profiles/             # Tmux layout profiles
│   ├── profile0.sh       # Top + split bottom (2-3 agents)
│   ├── profile1.sh       # Left column + stacked right
│   ├── profile2.sh       # Top row + full-width bottom
│   ├── profile3.sh       # Single full-width top (focus mode)
│   ├── profile4.sh       # Three-pane layout
│   └── profile5.sh       # Six-pane dashboard
└── scripts/              # Core orchestration shell scripts
    ├── setup.sh
    ├── start-agents.sh
    ├── attach-session.sh
    ├── kill-session.sh
    ├── send-commands.sh
    ├── remove-agent.sh
    └── sync-workspace.sh

Cargo.toml                # Rust project manifest and dependencies
README.md                 # Project documentation
.gitignore                # Git ignore patterns
.envrc                    # Direnv environment configuration
```

**Structure Decision**: Single Rust project with modular command structure. CLI logic in Rust for performance and type safety, system orchestration via shell scripts for direct git/tmux integration. Uses proven agents/ workspace organization and tmux profile system from reference implementation.

## Phase 0: Research & Decision Making

### Research Tasks

Based on the Constitution Check and Technical Context, the following research areas need to be investigated:

1. **CLI Accessibility Standards**: Research best practices for making CLI tools accessible to users with screen readers and other assistive technologies
2. **Rust CLI Distribution Patterns**: Investigate optimal strategies for distributing Rust binaries to users without Rust toolchain
3. **Error Handling Best Practices**: Research Rust error handling patterns that provide clear, actionable feedback for CLI users
4. **Tmux Integration Patterns**: Research best practices for Rust applications integrating with tmux sessions
5. **Git Worktree Management**: Investigate robust patterns for managing multiple git worktrees from a CLI application

### Research Output

✅ **COMPLETED**: All research findings documented in `research.md` with clear decisions, rationale, and alternatives considered.

**Resolved Research Areas**:
1. ✅ **CLI Accessibility Standards**: Comprehensive accessibility implementation plan
2. ✅ **Rust CLI Distribution Patterns**: Dual distribution strategy decided
3. ✅ **Error Handling Best Practices**: anyhow + thiserror approach chosen
4. ✅ **Tmux Integration Patterns**: Control mode with programmatic management
5. ✅ **Git Worktree Management**: libgit2 with shell script fallbacks

## Phase 1: Design & Contracts

### Data Model Design

✅ **COMPLETED**: Comprehensive data model defined in `data-model.md`

**Key Entities Designed**:
- **Agent Configuration**: Complete agent definition with status, workspace, and resource limits
- **Session Configuration**: Tmux session management with layouts and settings
- **Project Configuration**: Top-level configuration with agents, session, sync, and settings
- **Command Message & Execution Result**: Communication and execution tracking
- **Validation Rules & State Transitions**: Comprehensive data integrity constraints

### API Contracts

✅ **COMPLETED**: CLI API contracts defined in `contracts/cli-api.md`

**Contract Coverage**:
- **Core Command Structure**: Hierarchical CLI with global options
- **Initialization Commands**: `init`, `install` with preconditions and contracts
- **Session Management**: `start`, `attach`, `kill` with tmux integration
- **Agent Management**: `add`, `remove`, `list` with workspace management
- **Communication Commands**: `send`, `hey` with broadcast and individual messaging
- **Synchronization Commands**: `sync` with context-aware behavior
- **Navigation Commands**: `warp`, `zoom` for workspace and pane navigation
- **Configuration Commands**: `config` subcommands for settings management
- **Error Handling Contracts**: Standardized error codes and message formats
- **Performance Contracts**: Response time requirements and resource limits
- **Accessibility Contracts**: Screen reader support and visual accessibility

### User Documentation

✅ **COMPLETED**: Quickstart guide created in `quickstart.md`

**Documentation Coverage**:
- **5-Minute Quickstart**: Fast path to productive usage
- **Installation Instructions**: Multiple installation methods for different users
- **Common Workflows**: Feature development, bug fixing, code review scenarios
- **Configuration Examples**: Basic and large-team setup templates
- **Troubleshooting Guide**: Common issues and solutions
- **Best Practices**: Agent organization, branch management, performance tips

### Agent Context Update

✅ **COMPLETED**: Agent context files updated with Rust technology stack

**Technology Added to Agent Context**:
- **Language**: Rust 1.75+
- **Frameworks**: Clap (CLI framework), serde_yaml (configuration parsing), anyhow (error handling)
- **Storage**: YAML configuration files, Git worktrees for agent isolation
- **Project Type**: Single project with CLI interface and shell script orchestration

### Constitution Compliance Re-evaluation

✅ **COMPLETED**: All constitution requirements fully addressed

**Final Compliance Status**:
- ✅ **Code Quality**: Achieved through Rust strong typing, modular design
- ✅ **Testing Standards**: 80% coverage requirement with comprehensive testing strategy
- ✅ **UX Consistency**: Achieved through accessibility research and CLI design patterns
- ✅ **Performance Requirements**: Met through technical choices and performance contracts

## Phase 1 Summary

**All Phase 1 design artifacts completed successfully**:

- ✅ **Technical decisions resolved** through comprehensive research
- ✅ **Data model designed** with full entity relationships and validation
- ✅ **API contracts defined** covering all CLI functionality
- ✅ **User documentation created** with quickstart and troubleshooting
- ✅ **Agent context updated** with new technology information
- ✅ **Constitution compliance achieved** with all requirements addressed

**Ready for Phase 2**: Implementation planning and task generation.

