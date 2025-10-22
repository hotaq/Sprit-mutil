# Sprite Edge Case Test Suite

This directory contains comprehensive edge case tests for the Sprite multi-agent toolkit.

## Test Categories

### 1. Agent Management Edge Cases
- `agent_workspace_tests.sh` - Agent workspace creation, deletion, and management edge cases
- `agent_config_tests.sh` - Agent configuration validation and edge cases

### 2. Git Operations Edge Cases
- `git_worktree_tests.sh` - Git worktree creation, deletion, and conflict resolution
- `git_branch_tests.sh` - Git branch management edge cases
- `git_sync_tests.sh` - Git synchronization and conflict resolution tests

### 3. Configuration Edge Cases
- `config_loading_tests.sh` - Configuration file loading and validation edge cases
- `config_corruption_tests.sh` - Handling corrupted or malformed configuration files

### 4. Tmux Session Management
- `tmux_session_tests.sh` - Tmux session creation, attachment, and cleanup edge cases
- `tmux_layout_tests.sh` - Layout management and agent pane handling

### 5. Error Recovery Tests
- `recovery_tests.sh` - System recovery from various failure scenarios
- `cleanup_tests.sh` - Cleanup and resource management tests

### 6. Integration Tests
- `integration_tests.sh` - End-to-end workflow testing with edge cases
- `performance_tests.sh` - Performance and resource limit testing

## Usage

```bash
# Run all edge case tests
./run_all_tests.sh

# Run specific test category
./agent_workspace_tests.sh

# Run with verbose output
./run_all_tests.sh -v

# Run with cleanup after tests
./run_all_tests.sh --cleanup
```

## Test Environment Setup

All tests create isolated temporary environments and clean up after themselves. Tests require:
- Git
- Tmux
- Rust (for building sprite)

## Safety Features

- Tests run in isolated temporary directories
- Automatic cleanup of test artifacts
- Safety checks before destructive operations
- Rollback mechanisms for failed tests
