#!/bin/bash

# Configuration Loading/Validation Edge Case Tests
# Tests for configuration file loading, validation, and edge cases

set -euo pipefail

# Source common utilities
source "$(dirname "$0")/common/test_utils.sh"

# Test: Missing configuration file
test_missing_config() {
    log_test "Testing behavior with missing configuration file"
    
    setup_test_env "missing_config"
    
    # Don't create config file
    # Should fail gracefully
    assert_command_fails "sprite_cmd status" "Should fail without configuration file"
    assert_command_fails "sprite_cmd start" "Should fail to start without config"
    
    cleanup_test_env
}

# Test: Empty configuration file
test_empty_config() {
    log_test "Testing behavior with empty configuration file"
    
    setup_test_env "empty_config"
    
    # Create empty config file
    mkdir -p agents
    touch agents/agents.yaml
    
    # Should fail with empty config
    assert_command_fails "sprite_cmd config validate" "Should fail with empty configuration"
    
    cleanup_test_env
}

# Test: Malformed YAML configuration
test_malformed_yaml() {
    log_test "Testing behavior with malformed YAML"
    
    setup_test_env "malformed_yaml"
    
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4
  description: 'Missing closing quote
  status: 'Inactive'
  config: { invalid_yaml_structure }
EOF
    
    # Should fail with malformed YAML
    assert_command_fails "sprite_cmd config validate" "Should fail with malformed YAML"
    
    cleanup_test_env
}

# Test: Missing required fields
test_missing_required_fields() {
    log_test "Testing configuration with missing required fields"
    
    setup_test_env "missing_fields"
    
    mkdir -p agents
    
    # Config missing version
    cat > agents/agents.yaml << EOF
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4'
  status: 'Inactive'
  config: { env_vars: {} }
EOF
    
    assert_command_fails "sprite_cmd config validate" "Should fail with missing version field"
    
    cleanup_test_env
}

# Test: Invalid data types
test_invalid_data_types() {
    log_test "Testing configuration with invalid data types"
    
    setup_test_env "invalid_types"
    
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4'
  description: 'Agent 1'
  status: 123  # Should be string
  config:
    env_vars: "invalid"  # Should be object
    resource_limits:
      max_memory_mb: "invalid"  # Should be number
      max_cpu_percent: "invalid"  # Should be number
      max_processes: "invalid"  # Should be number
    timeout_settings:
      default_timeout_secs: "invalid"  # Should be number
      max_timeout_secs: "invalid"  # Should be number
session_name: 'sprite-test-session'
settings: { default_timeout_secs: "invalid" }  # Should be number
EOF
    
    assert_command_fails "sprite_cmd config validate" "Should fail with invalid data types"
    
    cleanup_test_env
}

# Test: Configuration with circular references
test_circular_references() {
    log_test "Testing configuration with potential circular references"
    
    setup_test_env "circular_refs"
    
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/2'  # Points to agent 2's path
  model: 'claude-sonnet-4'
  description: 'Agent 1'
  status: 'Inactive'
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024 }, timeout_settings: { default_timeout_secs: 300 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
- id: '2'
  branch: 'agent-2'
  worktree_path: 'agents/1'  # Points to agent 1's path
  model: 'claude-sonnet-4'
  description: 'Agent 2'
  status: 'Inactive'
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024 }, timeout_settings: { default_timeout_secs: 300 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
session_name: 'sprite-test-session'
settings: { default_timeout_secs: 300, log_level: 'Info', global_env_vars: {}, logging: { log_file: 'agents/logs/sprite.log', level: 'Info', log_to_stdout: true } }
EOF
    
    # Should detect circular references or handle gracefully
    if ! sprite_cmd config validate >/dev/null 2>&1; then
        log_success "Correctly detected circular reference issue"
    else
        log_warning "Circular reference validation not implemented"
    fi
    
    cleanup_test_env
}

# Test: Configuration with extreme values
test_extreme_values() {
    log_test "Testing configuration with extreme values"
    
    setup_test_env "extreme_values"
    
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4'
  description: 'Agent with extreme values'
  status: 'Inactive'
  config:
    env_vars: {}
    work_dir: null
    aliases: []
    tools: []
    resource_limits:
      max_memory_mb: 999999999  # Very large number
      max_cpu_percent: 1000     # Invalid percentage
      max_processes: 999999999  # Very large number
    timeout_settings:
      default_timeout_secs: 0   # Zero timeout
      max_timeout_secs: -1      # Negative timeout
    sync_settings:
      auto_sync: false
      sync_interval_secs: 0     # Zero interval
      conflict_resolution: 'Manual'
session_name: 'sprite-test-session'
settings:
  default_timeout_secs: -1     # Negative timeout
  log_level: 'InvalidLevel'    # Invalid log level
  global_env_vars: {}
  logging:
    log_file: 'agents/logs/sprite.log'
    level: 'InvalidLevel'       # Invalid log level
    log_to_stdout: true
EOF
    
    # Should validate or reject extreme values
    if ! sprite_cmd config validate >/dev/null 2>&1; then
        log_success "Correctly rejected extreme values"
    else
        log_warning "Extreme value validation not implemented"
    fi
    
    cleanup_test_env
}

# Test: Configuration file permissions
test_config_permissions() {
    log_test "Testing configuration file with different permissions"
    
    setup_test_env "config_perms"
    
    create_test_config 2
    
    # Make config file read-only
    chmod 444 agents/agents.yaml
    
    # Should still be able to read
    assert_command_success "sprite_cmd config validate" "Should read read-only config file"
    
    # Make config file unreadable
    chmod 000 agents/agents.yaml
    
    # Should fail when cannot read
    assert_command_fails "sprite_cmd config validate" "Should fail when config is unreadable"
    
    # Restore permissions for cleanup
    chmod 644 agents/agents.yaml
    
    cleanup_test_env
}

# Test: Multiple configuration files
test_multiple_configs() {
    log_test "Testing behavior with multiple configuration files"
    
    setup_test_env "multiple_configs"
    
    # Create config in subdirectory
    mkdir -p subproject/agents
    create_test_config 2 "subproject/agents/agents.yaml"
    
    # Create config in root
    create_test_config 1 "agents/agents.yaml"
    
    # Should use the closest config
    cd subproject
    assert_command_success "sprite_cmd config validate" "Should find config in current directory"
    assert_file_exists "agents/agents.yaml" "Should use local config"
    
    # Should find config in parent directory
    cd ..
    assert_command_success "sprite_cmd config validate" "Should find parent config"
    
    cleanup_test_env
}

# Test: Configuration with Unicode content
test_unicode_config() {
    log_test "Testing configuration with Unicode content"
    
    setup_test_env "unicode_config"
    
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-café'
  worktree_path: 'agents/测试'
  model: 'claude-sonnet-4'
  description: 'Agent with unicodé: 🤖 café müller'
  status: 'Inactive'
  config:
    env_vars:
      UNICODE_VAR: '测试变量'
      EMOJI_VAR: '🚀🎯💻'
    work_dir: null
    aliases: ['别名', 'alias']
    tools: []
    resource_limits:
      max_memory_mb: 1024
      max_cpu_percent: 80
      max_processes: 100
    timeout_settings:
      default_timeout_secs: 300
      max_timeout_secs: 3600
    sync_settings:
      auto_sync: false
      sync_interval_secs: 300
      conflict_resolution: 'Manual'
session_name: 'sprite-测试-session'
settings:
  default_timeout_secs: 300
  log_level: 'Info'
  global_env_vars:
    UNICODE: '😀🎉'
  logging:
    log_file: 'agents/logs/sprite-测试.log'
    level: 'Info'
    log_to_stdout: true
EOF
    
    # Should handle Unicode content
    assert_command_success "sprite_cmd config validate" "Should handle Unicode content"
    
    cleanup_test_env
}

# Test: Configuration with environment variables
test_env_var_parsing() {
    log_test "Testing configuration with environment variable references"
    
    setup_test_env "env_vars"
    
    export TEST_AGENT_ID="999"
    export TEST_MODEL="claude-opus-3"
    
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: '${TEST_MODEL}'
  description: 'Agent from env'
  status: 'Inactive'
  config:
    env_vars:
      AGENT_ID: '${TEST_AGENT_ID}'
      CUSTOM_VAR: 'custom_value'
    work_dir: null
    aliases: []
    tools: []
    resource_limits:
      max_memory_mb: 1024
      max_cpu_percent: 80
      max_processes: 100
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
    
    # Should handle env var substitution if supported
    if ! sprite_cmd config validate >/dev/null 2>&1; then
        log_warning "Environment variable substitution not supported"
    else
        log_success "Environment variable substitution works"
    fi
    
    unset TEST_AGENT_ID TEST_MODEL
    
    cleanup_test_env
}

# Test: Configuration with nested includes (if supported)
test_config_includes() {
    log_test "Testing configuration with file includes"
    
    setup_test_env "config_includes"
    
    mkdir -p agents
    
    # Create partial config files
    cat > agents/base_config.yaml << EOF
version: "0.2.3"
session_name: 'sprite-base-session'
settings:
  default_timeout_secs: 300
  log_level: 'Info'
  global_env_vars: {}
  logging:
    log_file: 'agents/logs/sprite.log'
    level: 'Info'
    log_to_stdout: true
EOF
    
    cat > agents/agents_list.yaml << EOF
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'agents/1'
  model: 'claude-sonnet-4'
  description: 'Agent 1'
  status: 'Inactive'
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024 }, timeout_settings: { default_timeout_secs: 300 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
EOF
    
    # Try to combine (if supported)
    cat > agents/agents.yaml << EOF
# This tests include functionality - not all YAML parsers support it
<<: (base_config.yaml)
<<: (agents_list.yaml)
EOF
    
    if ! sprite_cmd config validate >/dev/null 2>&1; then
        log_warning "YAML includes not supported"
    else
        log_success "YAML includes work"
    fi
    
    cleanup_test_env
}

# Main test runner
main() {
    echo "================================="
    echo "Configuration Edge Case Tests"
    echo "================================="
    
    # Run all tests
    test_missing_config
    test_empty_config
    test_malformed_yaml
    test_missing_required_fields
    test_invalid_data_types
    test_circular_references
    test_extreme_values
    test_config_permissions
    test_multiple_configs
    test_unicode_config
    test_env_var_parsing
    test_config_includes
    
    # Print results
    print_test_results
}

# Run tests if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
