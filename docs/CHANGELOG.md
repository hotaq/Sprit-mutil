# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive installation script with multiple methods
- GitHub Actions workflow for automatic releases
- Support for pre-compiled binary distribution

## [0.1.0] - 2024-10-20

### Added
- Initial release of Sprite Multi-Agent Workflow Toolkit
- Multi-agent management system with isolated tmux sessions
- Comprehensive workspace provisioning with git worktrees
- YAML-based configuration management with real-time change detection
- Session management with health monitoring and automatic recovery
- Complete CLI with commands: init, start, attach, kill, status, agents, config
- Integration test suite covering end-to-end workflows
- Accessibility support and comprehensive session health monitoring
- Resource cleanup and session state management
- Professional documentation with troubleshooting guides

### Features
- **Multi-Agent Management**: Create, configure, and manage multiple AI agents simultaneously
- **Workspace Provisioning**: Automatic git worktree setup for each agent with branch isolation
- **Session Management**: Robust tmux-based session handling with health monitoring
- **Configuration Management**: YAML-based configuration with real-time change detection
- **Health Monitoring**: Comprehensive session health analysis with automatic recovery
- **Resource Cleanup**: Automatic cleanup of orphaned sessions and temporary files
- **CLI Integration**: Intuitive command-line interface with helpful progress indicators

### Technical Implementation
- Rust-based CLI tool with Clap framework
- YAML configuration with serde_yaml
- Git worktree management for workspace isolation
- Tmux session management for multi-agent environments
- Health monitoring and automatic recovery systems
- Resource cleanup and session state management

### Testing
- Unit tests for all core components
- Integration tests for complete workflows
- Session management and recovery testing
- Workspace provisioning and validation tests