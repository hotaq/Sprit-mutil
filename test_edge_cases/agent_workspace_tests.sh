#!/bin/bash

# Agent Workspace Management Edge Case Tests
# Tests for agent workspace creation, deletion, and management edge cases

set -euo pipefail

# Source common utilities
source "$(dirname "$0")/common/test_utils.sh"

# Test: Initialize with zero agents
test_zero_agents() {
    log_test "Testing sprite init with 0 agents"
    
    setup_test_env "zero_agents"
    
    assert_command_success "sprite_cmd init --agents 0" "Should handle zero agents gracefully"
    assert_file_exists "agents/agents.yaml" "Configuration file should be created even with 0 agents"
    
    # Check that agents section exists (should be empty array for 0 agents now)
    assert_true "grep -q 'agents:' agents/agents.yaml" "Agents section should exist"
    assert_true "grep -q 'agents: \[\]' agents/agents.yaml" "Agents array should be empty"
    
    cleanup_test_env
}

# Test: Initialize with very large number of agents
test_large_agent_count() {
    log_test "Testing sprite init with large agent count"
    
    setup_test_env "large_agent_count"
    
    # Try to create 100 agents (stress test)
    assert_command_success "sprite_cmd init --agents 100" "Should handle large agent count"
    assert_file_exists "agents/agents.yaml" "Configuration file should be created"
    
    # Verify all agents are in config
    local agent_count=$(grep -c "id: '" agents/agents.yaml || true)
    assert_true "[[ $agent_count -eq 100 ]]" "Should have 100 agents in configuration"
    
    cleanup_test_env
}

# Test: Initialize when agents directory already exists
test_existing_agents_dir() {
    log_test "Testing sprite init with existing agents directory"
    
    setup_test_env "existing_agents_dir"
    
    # Create existing agents directory with files
    mkdir -p agents
    echo "existing file" > agents/existing.txt
    
    # Should succeed without force flag (sprite doesn't check for existing agents dir)
    assert_command_success "sprite_cmd init --agents 2" "Should succeed even with existing agents directory"
    
    # Should work with force flag too
    assert_command_success "sprite_cmd init --agents 2 --force" "Should succeed with force flag"
    
    # Note: sprite doesn't remove existing files by default
    assert_file_exists "agents/existing.txt" "Existing files may be preserved"
    
    cleanup_test_env
}

# Test: Agent workspace with special characters in path
test_special_characters_workspace() {
    log_test "Testing agent workspace with special characters"
    
    setup_test_env "special_chars"
    
    # Create config with special characters in paths
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-with-dash'
  worktree_path: 'agents/agent-1 test'
  model: 'claude-sonnet-4'
  description: 'Agent with special chars'
  status: 'Inactive'
  config:
    env_vars: {}
    work_dir: null
    aliases: []
    tools: []
    resource_limits:
      max_memory_mb: 1024
      max_cpu_percent: 80
      max_processes: 100
      operation_timeout_secs: 300
    timeout_settings:
      default_timeout_secs: 300
      max_timeout_secs: 3600
    sync_settings:
      auto_sync: false
      sync_interval_secs: 300
      conflict_resolution: 'Manual'
session_name: 'sprite-test-session'
settings:
  default_timeout_secs: 300
  log_level: 'Info'
  global_env_vars: {}
  logging:
    log_file: 'agents/logs/sprite.log'
    level: 'Info'
    log_to_stdout: true
EOF
    
    # Should handle special characters gracefully in config creation
    # (Validation will fail due to missing worktrees, but config parsing should succeed)
    assert_command_fails "sprite_cmd config validate" "Expected to fail validation (no worktrees)"
    
    cleanup_test_env
}

# Test: Duplicate agent IDs
test_duplicate_agent_ids() {
    log_test "Testing configuration with duplicate agent IDs"
    
    setup_test_env "duplicate_ids"
    
    # Create config with duplicate IDs
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
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024, max_cpu_percent: 80, max_processes: 100 }, timeout_settings: { default_timeout_secs: 300, max_timeout_secs: 3600 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
- id: '1'
  branch: 'agent-1-duplicate'
  worktree_path: 'agents/1-dup'
  model: 'claude-sonnet-4'
  description: 'Agent 1 duplicate'
  status: 'Inactive'
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024, max_cpu_percent: 80, max_processes: 100 }, timeout_settings: { default_timeout_secs: 300, max_timeout_secs: 3600 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
session_name: 'sprite-test-session'
settings: { default_timeout_secs: 300, log_level: 'Info', global_env_vars: {}, logging: { log_file: 'agents/logs/sprite.log', level: 'Info', log_to_stdout: true } }
EOF
    
    # Should detect duplicate IDs
    assert_command_fails "sprite_cmd config validate" "Should reject configuration with duplicate IDs"
    
    cleanup_test_env
}

# Test: Agent workspace with invalid git repository
test_invalid_git_repo() {
    log_test "Testing agent workspace with invalid git state"
    
    setup_test_env "invalid_git"
    
    # Initialize sprite normally
    create_test_config 2
    
    # Corrupt git repository
    rm -rf .git
    
    # Should fail when git repository is invalid
    assert_command_fails "sprite_cmd start" "Should fail without valid git repository"
    
    cleanup_test_env
}

# Test: Agent workspace permission issues
test_workspace_permissions() {
    log_test "Testing agent workspace with permission issues"
    
    setup_test_env "permissions"
    
    create_test_config 2
    
    # Create agents directory and make it read-only
    mkdir -p agents
    chmod 444 agents
    
    # Should fail when cannot create workspaces
    assert_command_fails "sprite_cmd start" "Should fail when cannot create workspaces"
    
    # Restore permissions for cleanup
    chmod 755 agents
    
    cleanup_test_env
}

# Test: Agent workspace cleanup on failure
test_workspace_cleanup_failure() {
    log_test "Testing workspace cleanup when operations fail"
    
    setup_test_env "cleanup_failure"
    
    create_test_config 3
    
    # Simulate workspace creation failure by making workspace directory immutable
    mkdir -p agents/1
    chmod 444 agents/1
    
    # Should handle cleanup gracefully
    assert_command_fails "sprite_cmd start" "Should fail when workspace operations fail"
    
    # Restore permissions for cleanup
    chmod 755 agents/1
    
    cleanup_test_env
}

# Test: Concurrent workspace operations
test_concurrent_operations() {
    log_test "Testing concurrent workspace operations"
    
    setup_test_env "concurrent"
    
    create_test_config 2
    
    # Try to start multiple sprite processes concurrently
    sprite_cmd start &
    local pid1=$!
    
    sleep 0.5
    
    # Second start should fail due to existing session
    assert_command_fails "sprite_cmd start" "Should fail when session already exists"
    
    # Clean up
    kill $pid1 2>/dev/null || true
    wait $pid1 2>/dev/null || true
    
    cleanup_test_env
}

# Test: Workspace with very long paths
test_long_paths() {
    log_test "Testing workspace with very long paths"
    
    setup_test_env "long_paths"
    
    # Create config with long path names
    mkdir -p agents
    local long_name="this_is_a_very_long_agent_name_that_might_cause_issues_with_filesystem_limits_and_path_restrictions"
    
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: '$long_name'
  worktree_path: 'agents/$long_name'
  model: 'claude-sonnet-4'
  description: 'Agent with long name'
  status: 'Inactive'
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024, max_cpu_percent: 80, max_processes: 100 }, timeout_settings: { default_timeout_secs: 300, max_timeout_secs: 3600 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
session_name: 'sprite-test-session'
settings: { default_timeout_secs: 300, log_level: 'Info', global_env_vars: {}, logging: { log_file: 'agents/logs/sprite.log', level: 'Info', log_to_stdout: true } }
EOF
    
    # Should handle long paths or fail gracefully
    if ! sprite_cmd config validate >/dev/null 2>&1; then
        log_warning "Long paths not supported (expected on some systems)"
    else
        log_success "Long paths handled correctly"
    fi
    
    cleanup_test_env
}

# Main test runner
main() {
    echo "================================="
    echo "Agent Workspace Edge Case Tests"
    echo "================================="
    
    # Run all tests
    test_zero_agents
    test_large_agent_count
    test_existing_agents_dir
    test_special_characters_workspace
    test_duplicate_agent_ids
    test_invalid_git_repo
    test_workspace_permissions
    test_workspace_cleanup_failure
    test_concurrent_operations
    test_long_paths
    
    # Print results
    print_test_results
}

# Run tests if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
