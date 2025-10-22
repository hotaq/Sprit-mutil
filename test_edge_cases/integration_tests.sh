#!/bin/bash

# Integration Tests - End-to-End Workflow Edge Cases
# Tests for complete Sprite workflows with various edge cases

set -euo pipefail

# Source common utilities
source "$(dirname "$0")/common/test_utils.sh"

# Test: Complete workflow from init to cleanup
test_complete_workflow() {
    log_test "Testing complete workflow initialization to cleanup"
    
    setup_test_env "complete_workflow"
    
    # Initialize
    assert_command_success "sprite_cmd init --agents 3" "Should initialize project"
    assert_file_exists "agents/agents.yaml" "Config should exist"
    
    # Start
    assert_command_success "sprite_cmd start --force" "Should start session"
    wait_for_tmux
    
    # Check status
    assert_command_success "sprite_cmd status" "Should check status"
    
    # Stop
    assert_command_success "sprite_cmd kill" "Should stop session"
    
    cleanup_test_env
}

# Test: Workflow with git integration
test_git_workflow() {
    log_test "Testing workflow with git integration"
    
    setup_test_env "git_workflow"
    
    # Create some files first
    echo "initial content" > main.txt
    git add main.txt >/dev/null 2>&1
    git commit -m "Initial commit" >/dev/null 2>&1
    
    # Initialize sprite
    assert_command_success "sprite_cmd init --agents 2" "Should initialize with git"
    
    # Make changes in worktrees
    assert_command_success "sprite_cmd start --force" "Should start with git"
    wait_for_tmux
    sleep 2
    
    # Send commands to agent 1
    assert_command_success "sprite_cmd send --agent 1 --command 'echo \"agent1 change\" > agent1.txt'" "Should send command to agent 1"
    sleep 1
    
    # Sync changes
    assert_command_success "sprite_cmd sync --agent 1" "Should sync agent 1 changes"
    
    # Clean up
    assert_command_success "sprite_cmd kill" "Should stop session"
    
    cleanup_test_env
}

# Test: Workflow with concurrent operations
test_concurrent_workflow() {
    log_test "Testing workflow with concurrent operations"
    
    setup_test_env "concurrent_workflow"
    
    create_test_config 3
    
    # Start session
    sprite_cmd start --force &
    local pid=$!
    wait_for_tmux
    sleep 2
    
    # Send multiple concurrent commands
    sprite_cmd send --agent 1 --command "sleep 1" &
    sprite_cmd send --agent 2 --command "sleep 1" &
    sprite_cmd send --agent 3 --command "sleep 1" &
    
    # Wait for commands
    wait
    sleep 1
    
    # Clean up
    kill $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    
    cleanup_test_env
}

# Test: Workflow with resource limits
test_resource_limits() {
    log_test "Testing workflow with resource limits"
    
    setup_test_env "resource_limits"
    
    # Create config with strict limits
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4'
  description: 'Resource-limited agent'
  status: 'Inactive'
  config:
    env_vars: {}
    work_dir: null
    aliases: []
    tools: []
    resource_limits:
      max_memory_mb: 64
      max_cpu_percent: 50
      max_processes: 5
    timeout_settings:
      default_timeout_secs: 5
      max_timeout_secs: 10
    sync_settings:
      auto_sync: false
      sync_interval_secs: 300
      conflict_resolution: 'Manual'
session_name: 'sprite-test-session'
settings:
  default_timeout_secs: 5
  log_level: 'Info'
  global_env_vars: {}
  logging:
    log_file: 'agents/logs/sprite.log'
    level: 'Info'
    log_to_stdout: true
EOF
    
    # Should respect resource limits
    assert_command_success "sprite_cmd start --force" "Should start with resource limits"
    wait_for_tmux
    
    # Send command that might exceed limits
    if sprite_cmd send --agent 1 --command "sleep 10" --timeout 3 2>/dev/null; then
        log_warning "Timeout not enforced"
    else
        log_success "Timeout enforced correctly"
    fi
    
    assert_command_success "sprite_cmd kill" "Should stop session"
    
    cleanup_test_env
}

# Test: Workflow with configuration changes
test_config_changes() {
    log_test "Testing workflow with mid-session configuration changes"
    
    setup_test_env "config_changes"
    
    create_test_config 2
    
    # Start session
    sprite_cmd start --force &
    local pid=$!
    wait_for_tmux
    sleep 1
    
    # Modify configuration while session is running
    sed -i.bak 's/claude-sonnet-4/claude-opus-3/g' agents/agents.yaml
    
    # Should detect changes
    assert_command_success "sprite_cmd config validate" "Should validate modified config"
    
    # Clean up
    kill $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    
    cleanup_test_env
}

# Test: Workflow with network issues
test_network_issues() {
    log_test "Testing workflow with simulated network issues"
    
    setup_test_env "network_issues"
    
    create_test_config 2
    
    # Simulate network issues for update checks
    export HTTP_PROXY="invalid-proxy-url:9999"
    
    # Should handle network timeouts gracefully
    if sprite_cmd update --check 2>/dev/null; then
        log_warning "Network failure not simulated properly"
    else
        log_success "Network failures handled gracefully"
    fi
    
    # Normal operations should still work
    assert_command_success "sprite_cmd start --force" "Should start despite network issues"
    assert_command_success "sprite_cmd kill" "Should stop normally"
    
    unset HTTP_PROXY
    
    cleanup_test_env
}

# Test: Workflow with filesystem full simulation
test_filesystem_full() {
    log_test "Testing behavior when filesystem is full"
    
    setup_test_env "filesystem_full"
    
    create_test_config 1
    
    # Create a small temporary filesystem limit using df
    # This is a simplified test - real disk full is harder to simulate
    local space_available=$(df . | tail -1 | awk '{print $4}')
    
    if [[ $space_available -gt 1048576 ]]; then  # More than 1GB available
        log_info " Plenty of disk space available, simulating full filesystem"
        
        # Create a large file to consume space partially
        dd if=/dev/zero of=big_file.tmp bs=1M count=100 >/dev/null 2>&1
        
        # Should still work with limited space
        assert_command_success "sprite_cmd start --force" "Should start with limited space"
        assert_command_success "sprite_cmd kill" "Should stop normally"
        
        # Cleanup
        rm -f big_file.tmp
    else
        log_warning "Skipping disk full test - insufficient space"
    fi
    
    cleanup_test_env
}

# Test: Workflow with signal handling
test_signal_handling() {
    log_test "Testing signal handling during sprite operations"
    
    setup_test_env "signal_handling"
    
    create_test_config 2
    
    # Start session
    sprite_cmd start --force &
    local pid=$!
    wait_for_tmux
    sleep 2
    
    # Send various signals
    kill -HUP $pid 2>/dev/null || true  # Hangup
    sleep 1
    
    kill -USR1 $pid 2>/dev/null || true  # User signal 1
    sleep 1
    
    # Session should still be running
    assert_command_success "tmux has-session -t sprite-test-session" "Session should survive signals"
    
    # Clean shutdown
    assert_command_success "sprite_cmd kill" "Should stop normally"
    
    cleanup_test_env
}

# Test: Workflow with environment variable changes
test_env_changes() {
    log_test "Testing workflow with environment variable changes"
    
    setup_test_env "env_changes"
    
    create_test_config 2
    
    # Set custom environment variables
    export SPRITE_TEST="true"
    export CUSTOM_AGENT_VAR="test_value"
    
    # Should use environment variables in config
    assert_command_success "sprite_cmd start --force" "Should start with custom env vars"
    wait_for_tmux
    
    # Verify environment variables are available
    if tmux capture-pane -t sprite-test-session:1 -p | grep -q "SPRITE_TEST"; then
        log_success "Environment variables propagated"
    else
        log_warning "Environment variable propagation not verified"
    fi
    
    assert_command_success "sprite_cmd kill" "Should stop normally"
    
    unset SPRITE_TEST CUSTOM_AGENT_VAR
    
    cleanup_test_env
}

# Test: Workflow with different git states
test_git_states() {
    log_test "Testing workflow with various git repository states"
    
    setup_test_env "git_states"
    
    # Create complex git state
    echo "content" > file1.txt
    git add file1.txt >/dev/null 2>&1
    git commit -m "Add file1" >/dev/null 2>&1
    
    # Create uncommitted changes
    echo "changes" > file2.txt
    git add file2.txt >/dev/null 2>&1
    echo "more changes" >> file1.txt
    
    # Create stashed changes
    echo "stashed content" > stashed.txt
    git add stashed.txt >/dev/null 2>&1
    git stash >/dev/null 2>&1
    
    create_test_config 2
    
    # Should handle complex git state
    assert_command_success "sprite_cmd start --force" "Should handle complex git state"
    wait_for_tmux
    
    assert_command_success "sprite_cmd kill" "Should stop normally"
    
    cleanup_test_env
}

# Test: Workflow with user interruption
test_user_interruption() {
    log_test "Testing workflow with user interruption (Ctrl+C)"
    
    setup_test_env "user_interruption"
    
    create_test_config 2
    
    # Start session in background
    sprite_cmd start --force &
    local pid=$!
    wait_for_tmux
    sleep 2
    
    # Send interrupt signal (simulate Ctrl+C)
    kill -INT $pid 2>/dev/null || true
    sleep 2
    
    # Session should survive interrupt or be cleaned up properly
    if tmux has-session -t sprite-test-session 2>/dev/null; then
        log_success "Session survived interrupt"
        assert_command_success "sprite_cmd kill" "Should stop normally"
    else
        log_success "Session cleaned up after interrupt"
    fi
    
    cleanup_test_env
}

# Main test runner
main() {
    echo "================================="
    echo "Integration Edge Case Tests"
    echo "================================="
    
    # Run all tests
    test_complete_workflow
    test_git_workflow
    test_concurrent_workflow
    test_resource_limits
    test_config_changes
    test_network_issues
    test_filesystem_full
    test_signal_handling
    test_env_changes
    test_git_states
    test_user_interruption
    
    # Print results
    print_test_results
}

# Run tests if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
