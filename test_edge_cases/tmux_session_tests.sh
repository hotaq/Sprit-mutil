#!/bin/bash

# Tmux Session Management Edge Case Tests
# Tests for tmux session creation, attachment, and cleanup edge cases

set -euo pipefail

# Source common utilities
source "$(dirname "$0")/common/test_utils.sh"

# Test: Start without tmux installed
test_no_tmux() {
    log_test "Testing behavior when tmux is not available"
    
    setup_test_env "no_tmux"
    
    create_test_config 2
    
    # Temporarily hide tmux
    local original_path="$PATH"
    export PATH="/nonexistent:$PATH"
    
    # Should fail gracefully without tmux
    assert_command_fails "sprite_cmd start" "Should fail without tmux"
    
    # Restore path
    export PATH="$original_path"
    
    cleanup_test_env
}

# Test: Start with existing tmux session
test_existing_tmux_session() {
    log_test "Testing sprite with existing tmux sessions"
    
    setup_test_env "existing_tmux"
    
    create_test_config 2
    
    # Create a tmux session manually
    tmux new-session -d -s existing-session
    assert_command_success "tmux has-session -t existing-session" "Manual tmux session should exist"
    
    # Should not interfere with existing sessions
    assert_command_success "sprite_cmd start --force" "Should not interfere with existing sessions"
    
    # Cleanup
    tmux kill-session -t existing-session 2>/dev/null || true
    
    cleanup_test_env
}

# Test: Multiple sprite sessions
test_multiple_sprite_sessions() {
    log_test "Testing multiple sprite session attempts"
    
    setup_test_env "multiple_sessions"
    
    create_test_config 2
    
    # Start first session
    sprite_cmd start &
    local pid1=$!
    wait_for_tmux
    
    # Try to start second session (should fail)
    assert_command_fails "sprite_cmd start" "Should fail with existing sprite session"
    
    # Force start should work
    assert_command_success "sprite_cmd start --force" "Should force start new session"
    
    # Clean up
    kill $pid1 2>/dev/null || true
    wait $pid1 2>/dev/null || true
    
    cleanup_test_env
}

# Test: Tmux session with special characters
test_tmux_special_characters() {
    log_test "Testing tmux session names with special characters"
    
    setup_test_env "tmux_special"
    
    # Create config with special session name
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4'
  description: 'Agent 1'
  status: 'Inactive'
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024 }, timeout_settings: { default_timeout_secs: 300 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
session_name: 'sprite-test-@#$%^&*()'
settings: { default_timeout_secs: 300, log_level: 'Info', global_env_vars: {}, logging: { log_file: 'agents/logs/sprite.log', level: 'Info', log_to_stdout: true } }
EOF
    
    # Should handle special characters in session names
    assert_command_success "sprite_cmd start --force" "Should handle special session names"
    
    cleanup_test_env
}

# Test: Tmux layout edge cases
test_tmux_layouts() {
    log_test "Testing different tmux layouts"
    
    setup_test_env "tmux_layouts"
    
    create_test_config 4  # More agents for layout testing
    
    # Test tiled layout
    assert_command_success "sprite_cmd start --force --layout tiled" "Should handle tiled layout"
    sprite_cmd kill 2>/dev/null || true
    sleep 1
    
    # Test vertical layout
    assert_command_success "sprite_cmd start --force --layout vertical" "Should handle vertical layout"
    sprite_cmd kill 2>/dev/null || true
    sleep 1
    
    # Test dashboard layout (if supported)
    if sprite_cmd start --force --layout dashboard >/dev/null 2>&1; then
        log_success "Dashboard layout works"
        sprite_cmd kill 2>/dev/null || true
    else
        log_warning "Dashboard layout not supported"
    fi
    
    cleanup_test_env
}

# Test: Tmux session with very long session name
test_long_session_name() {
    log_test "Testing tmux with very long session names"
    
    setup_test_env "long_session"
    
    local long_name="this-is-a-very-long-session-name-that-might-exceed-tmux-limits-and-various-terminal-constraints"
    
    # Create config with long session name
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4'
  description: 'Agent 1'
  status: 'Inactive'
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024 }, timeout_settings: { default_timeout_secs: 300 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
session_name: '$long_name'
settings: { default_timeout_secs: 300, log_level: 'Info', global_env_vars: {}, logging: { log_file: 'agents/logs/sprite.log', level: 'Info', log_to_stdout: true } }
EOF
    
    # Should handle long session names or fail gracefully
    if sprite_cmd start --force >/dev/null 2>&1; then
        log_success "Long session names work"
        sprite_cmd kill 2>/dev/null || true
    else
        log_warning "Long session names not supported"
    fi
    
    cleanup_test_env
}

# Test: Tmux session during tmux server issues
test_tmux_server_issues() {
    log_test "Testing tmux behavior during server issues"
    
    setup_test_env "tmux_server_issues"
    
    create_test_config 2
    
    # Kill tmux server to simulate issues
    tmux kill-server 2>/dev/null || true
    
    # Should recreate tmux server
    assert_command_success "sprite_cmd start --force" "Should handle tmux server restart"
    
    cleanup_test_env
}

# Test: Tmux session with different terminal sizes
test_terminal_sizes() {
    log_test "Testing tmux behavior with different terminal sizes"
    
    setup_test_env "terminal_sizes"
    
    create_test_config 2
    
    # Test with small terminal
    export LINES=10
    export COLUMNS=40
    
    assert_command_success "sprite_cmd start --force" "Should handle small terminal"
    sprite_cmd kill 2>/dev/null || true
    sleep 1
    
    # Test with large terminal
    export LINES=200
    export COLUMNS=500
    
    assert_command_success "sprite_cmd start --force" "Should handle large terminal"
    
    # Reset terminal size
    unset LINES COLUMNS
    
    cleanup_test_env
}

# Test: Tmux session with unicode content
test_tmux_unicode() {
    log_test "Testing tmux session with unicode content"
    
    setup_test_env "tmux_unicode"
    
    create_test_config 1
    
    # Start sprite session
    sprite_cmd start --force &
    local pid=$!
    wait_for_tmux
    
    # Send unicode content to agent session
    tmux send-keys -t sprite-test-session:1 "echo '测试Unicode 🤖 café'" Enter
    sleep 1
    
    # Clean up
    kill $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    
    cleanup_test_env
}

# Test: Tmux session persistence
test_tmux_persistence() {
    log_test "Testing tmux session persistence across restarts"
    
    setup_test_env "tmux_persistence"
    
    create_test_config 2
    
    # Start session
    sprite_cmd start --force &
    local pid1=$!
    wait_for_tmux
    sleep 2
    
    # Kill sprite process but not tmux session
    kill $pid1 2>/dev/null || true
    wait $pid1 2>/dev/null || true
    
    # Should detect existing session
    assert_command_fails "sprite_cmd start" "Should detect existing session"
    
    # Force start should clean up
    assert_command_success "sprite_cmd start --force" "Should force restart"
    
    cleanup_test_env
}

# Test: Tmux session cleanup on crash
test_tmux_cleanup_on_crash() {
    log_test "Testing tmux session cleanup during crashes"
    
    setup_test_env "tmux_crash"
    
    create_test_config 2
    
    # Start session
    sprite_cmd start --force &
    local pid=$!
    wait_for_tmux
    sleep 2
    
    # Kill session abruptly (simulate crash)
    tmux kill-session -t sprite-test-session 2>/dev/null || true
    kill -9 $pid 2>/dev/null || true
    
    # Should handle orphaned state
    assert_command_success "sprite_cmd start --force" "Should handle orphaned tmux state"
    
    cleanup_test_env
}

# Test: Tmux session with detached mode
test_detached_mode() {
    log_test "Testing sprite start with detached mode"
    
    setup_test_env "detached_mode"
    
    create_test_config 2
    
    # Start in detached mode
    assert_command_success "sprite_cmd start --force --detach" "Should start in detached mode"
    
    # Verify session exists but we're not attached
    assert_command_success "tmux has-session -t sprite-test-session" "Session should exist"
    
    # Should be able to attach later
    assert_command_success "sprite_cmd attach" "Should be able to attach to detached session"
    sleep 1
    
    cleanup_test_env
}

# Test: Tmux zoom functionality
test_tmux_zoom() {
    log_test "Testing tmux zoom functionality"
    
    setup_test_env "tmux_zoom"
    
    create_test_config 3
    
    # Start session
    sprite_cmd start --force &
    local pid=$!
    wait_for_tmux
    sleep 2
    
    # Test zoom commands
    assert_command_success "sprite_cmd zoom --agent 1" "Should zoom on agent 1"
    sleep 1
    
    assert_command_success "sprite_cmd zoom --unzoom" "Should unzoom"
    sleep 1
    
    # Clean up
    kill $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    
    cleanup_test_env
}

# Main test runner
main() {
    echo "================================="
    echo "Tmux Session Edge Case Tests"
    echo "================================="
    
    # Run all tests
    test_no_tmux
    test_existing_tmux_session
    test_multiple_sprite_sessions
    test_tmux_special_characters
    test_tmux_layouts
    test_long_session_name
    test_tmux_server_issues
    test_terminal_sizes
    test_tmux_unicode
    test_tmux_persistence
    test_tmux_cleanup_on_crash
    test_detached_mode
    test_tmux_zoom
    
    # Print results
    print_test_results
}

# Run tests if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
