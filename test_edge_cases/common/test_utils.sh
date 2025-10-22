#!/bin/bash

# Common utilities for Sprite edge case testing

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0

# Global test directory
TEST_DIR=""
ORIGINAL_DIR=""
SPRITE_BIN=""

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

# Test assertion functions
assert_true() {
    local condition=$1
    local message=${2:-"Assertion failed"}
    
    ((TESTS_TOTAL++))
    
    if eval "$condition"; then
        ((TESTS_PASSED++))
        log_success "✓ $message"
        return 0
    else
        ((TESTS_FAILED++))
        log_error "✗ $message"
        return 1
    fi
}

assert_false() {
    local condition=$1
    local message=${2:-"Assertion should be false"}
    
    ((TESTS_TOTAL++))
    
    if ! eval "$condition"; then
        ((TESTS_PASSED++))
        log_success "✓ $message"
        return 0
    else
        ((TESTS_FAILED++))
        log_error "✗ $message"
        return 1
    fi
}

assert_file_exists() {
    local file=$1
    local message=${2:-"File should exist: $file"}
    
    ((TESTS_TOTAL++))
    
    if [[ -f "$file" ]]; then
        ((TESTS_PASSED++))
        log_success "✓ $message"
        return 0
    else
        ((TESTS_FAILED++))
        log_error "✗ $message"
        return 1
    fi
}

assert_file_not_exists() {
    local file=$1
    local message=${2:-"File should not exist: $file"}
    
    ((TESTS_TOTAL++))
    
    if [[ ! -f "$file" ]]; then
        ((TESTS_PASSED++))
        log_success "✓ $message"
        return 0
    else
        ((TESTS_FAILED++))
        log_error "✗ $message"
        return 1
    fi
}

assert_dir_exists() {
    local dir=$1
    local message=${2:-"Directory should exist: $dir"}
    
    ((TESTS_TOTAL++))
    
    if [[ -d "$dir" ]]; then
        ((TESTS_PASSED++))
        log_success "✓ $message"
        return 0
    else
        ((TESTS_FAILED++))
        log_error "✗ $message"
        return 1
    fi
}

assert_dir_not_exists() {
    local dir=$1
    local message=${2:-"Directory should not exist: $dir"}
    
    ((TESTS_TOTAL++))
    
    if [[ ! -d "$dir" ]]; then
        ((TESTS_PASSED++))
        log_success "✓ $message"
        return 0
    else
        ((TESTS_FAILED++))
        log_error "✗ $message"
        return 1
    fi
}

assert_command_success() {
    local command=$1
    local message=${2:-"Command should succeed: $command"}
    
    ((TESTS_TOTAL++))
    
    if eval "$command" >/dev/null 2>&1; then
        ((TESTS_PASSED++))
        log_success "✓ $message"
        return 0
    else
        ((TESTS_FAILED++))
        log_error "✗ $message"
        return 1
    fi
}

assert_command_fails() {
    local command=$1
    local message=${2:-"Command should fail: $command"}
    
    ((TESTS_TOTAL++))
    
    if ! eval "$command" >/dev/null 2>&1; then
        ((TESTS_PASSED++))
        log_success "✓ $message"
        return 0
    else
        ((TESTS_FAILED++))
        log_error "✗ $message"
        return 1
    fi
}

# Setup functions
setup_test_env() {
    local test_name=$1
    
    # Store original directory
    ORIGINAL_DIR=$(pwd)
    
    # Create temporary test directory
    TEST_DIR=$(mktemp -d "/tmp/sprite_test_${test_name}_XXXXXX")
    
    # Create sprite binary if not exists
    if [[ -z "$SPRITE_BIN" ]]; then
        SPRITE_BIN=$(mktemp "/tmp/sprite_bin_XXXXXX")
        cargo build --release >/dev/null 2>&1
        cp target/release/sprite "$SPRITE_BIN"
        chmod +x "$SPRITE_BIN"
    fi
    
    # Change to test directory
    cd "$TEST_DIR"
    
    # Initialize git repository
    git init >/dev/null 2>&1
    git config user.name "Test User" >/dev/null 2>&1
    git config user.email "test@example.com" >/dev/null 2>&1
    
    # Create initial commit
    echo "Initial commit" > README.md
    git add README.md >/dev/null 2>&1
    git commit -m "Initial commit" >/dev/null 2>&1
    
    log_info "Test environment setup: $TEST_DIR"
}

cleanup_test_env() {
    log_info "Cleaning up test environment: $TEST_DIR"
    
    # Return to original directory
    cd "$ORIGINAL_DIR" 2>/dev/null || cd /
    
    # Kill any tmux sessions created during tests
    if command -v tmux >/dev/null 2>&1; then
        tmux list-sessions 2>/dev/null | grep -E "sprite-test|test-" | cut -d: -f1 | xargs -I {} tmux kill-session -t {} 2>/dev/null || true
    fi
    
    # Remove test directory
    if [[ -n "$TEST_DIR" && -d "$TEST_DIR" ]]; then
        rm -rf "$TEST_DIR"
    fi
}

# Sprite command wrapper
sprite_cmd() {
    "$SPRITE_BIN" "$@" 2>&1
}

# Create test configuration
create_test_config() {
    local agent_count=${1:-3}
    local config_file=${2:-"agents/agents.yaml"}
    
    mkdir -p agents
    
    cat > "$config_file" << EOF
version: "0.2.3"
agents:
EOF
    
    for ((i=1; i<=agent_count; i++)); do
        cat >> "$config_file" << EOF
- id: '$i'
  branch: 'agent-$i'
  worktree_path: 'agents/$i'
  model: 'claude-sonnet-4'
  description: 'Agent $i workspace'
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
    timeout_settings:
      default_timeout_secs: 300
      max_timeout_secs: 3600
    sync_settings:
      auto_sync: false
      sync_interval_secs: 300
      conflict_resolution: 'Manual'

EOF
    done
    
    cat >> "$config_file" << EOF
session_name: 'sprite-test-session'
session_config:
  layout: 'tiled'
  sync_status: false
  always_active: true
settings:
  default_timeout_secs: 300
  log_level: 'Info'
  auto_cleanup: true
  persist_sessions: false
  default_layout: 'tiled'
  workspace_retention_days: 7
  health_check_interval_secs: 60
  session_configs: {}
  global_env_vars: {}
  logging:
    log_file: 'agents/logs/sprite.log'
    level: 'Info'
    log_to_stdout: true
    rotation:
      max_size_mb: 100
      max_files: 5
      compress: true
sync:
  auto_sync: false
  default_interval_secs: 300
  conflict_resolution: 'Manual'
  exclude_branches:
  - 'main'
  - 'master'
  merge_strategy: 'merge'
  sync_on_start: false
  sync_on_exit: false
EOF
}

# Print test results
print_test_results() {
    echo
    echo "================================="
    echo "Test Results Summary"
    echo "================================="
    echo "Total Tests: $TESTS_TOTAL"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}Some tests failed!${NC}"
        return 1
    fi
}

# Wait for tmux session to stabilize
wait_for_tmux() {
    local retries=10
    local delay=0.5
    
    for ((i=1; i<=retries; i++)); do
        if tmux list-sessions 2>/dev/null | grep -q "sprite"; then
            sleep "$delay"
            return 0
        fi
        sleep "$delay"
    done
    
    log_warning "Tmux session did not start properly"
    return 1
}

# Generate random string for unique naming
random_string() {
    local length=${1:-8}
    openssl rand -hex "$length" 2>/dev/null || date +%s | sha256sum | head -c "$length"
}

# Trap cleanup
trap cleanup_test_env EXIT

# Export functions for use in test scripts
export -f log_info log_success log_warning log_error log_test
export -f assert_true assert_false assert_file_exists assert_file_not_exists
export -f assert_dir_exists assert_dir_not_exists assert_command_success assert_command_fails
export -f setup_test_env cleanup_test_env sprite_cmd create_test_config
export -f print_test_results wait_for_tmux random_string
export TEST_DIR ORIGINAL_DIR SPRITE_BIN TESTS_TOTAL TESTS_PASSED TESTS_FAILED
