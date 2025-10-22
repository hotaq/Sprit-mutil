#!/bin/bash

# Comprehensive test script for Sprite fixes
# Tests both configuration parsing and workspace provisioning fixes

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_test() {
    echo -e "${BLUE}🧪 Testing: $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Count tests and results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

run_test() {
    local test_name="$1"
    local test_command="$2"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    print_test "$test_name"

    if eval "$test_command" > /dev/null 2>&1; then
        print_success "$test_name - PASSED"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        print_error "$test_name - FAILED"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Run a command with a timeout that works across platforms
run_with_timeout() {
    local duration="$1"
    shift

    if command -v timeout >/dev/null 2>&1; then
        timeout "$duration" "$@"
    elif command -v gtimeout >/dev/null 2>&1; then
        gtimeout "$duration" "$@"
    else
        python3 - "$duration" "$@" <<'PY'
import subprocess
import sys

def parse_duration(raw):
    raw = raw.strip()
    if raw.lower().endswith('s'):
        raw = raw[:-1]
    try:
        return float(raw)
    except ValueError:
        raise SystemExit(f"Invalid timeout duration: {raw}")

duration = parse_duration(sys.argv[1])
command = sys.argv[2:]

try:
    completed = subprocess.run(command, timeout=duration, check=False)
    raise SystemExit(completed.returncode)
except subprocess.TimeoutExpired:
    raise SystemExit(124)
PY
    fi
}

# Cleanup function
cleanup() {
    print_info "Cleaning up test environments..."

    # Kill any tmux sessions
    if tmux has-session -t sprite-test 2>/dev/null; then
        tmux kill-session -t sprite-test
    fi

    # Remove test directories
    rm -rf /tmp/sprite-test-*

    print_success "Cleanup completed"
}

# Setup test environment
setup_test_env() {
    local test_name="$1"
    local test_dir="/tmp/sprite-test-$test_name"

    # Create test directory
    rm -rf "$test_dir"
    mkdir -p "$test_dir"
    cd "$test_dir"

    # Initialize git repo
    git init > /dev/null 2>&1
    git config user.email "test@example.com"
    git config user.name "Test User"

    # Create initial commit
    echo "# Test Project: $test_name" > README.md
    git add README.md > /dev/null 2>&1
    git commit -m "Initial commit" > /dev/null 2>&1

    # Copy sprite binary
    cp /Users/chinnaphat/Desktop/cedt/mutilagent/target/debug/sprite ./sprite
    chmod +x ./sprite

    echo "$test_dir"
}

# Test 1: Configuration parsing with correct case
test_config_parsing() {
    local test_dir=$(setup_test_env "config-parsing")
    cd "$test_dir"

    # Initialize with agents
    if ! ./sprite init --agents 2 > /dev/null 2>&1; then
        return 1
    fi

    # Test zoom command (should not fail with parsing errors)
    if ./sprite zoom --list > /dev/null 2>&1; then
        return 1  # Should fail because no tmux session, but not due to parsing
    fi

    # Check if error contains parsing error
    if ./sprite zoom --list 2>&1 | grep -q "Failed to parse configuration"; then
        return 1  # Should not have parsing errors
    fi

    return 0
}

# Test 2: Workspace provisioning during init
test_workspace_provisioning_init() {
    local test_dir=$(setup_test_env "workspace-init")
    cd "$test_dir"

    # Initialize with agents
    if ! ./sprite init --agents 3 > /dev/null 2>&1; then
        return 1
    fi

    # Check if workspaces were created
    if [[ ! -d "agents/1" ]] || [[ ! -d "agents/2" ]] || [[ ! -d "agents/3" ]]; then
        return 1
    fi

    # Check if branches exist
    if ! git rev-parse --verify agents/1 > /dev/null 2>&1; then
        return 1
    fi

    if ! git rev-parse --verify agents/2 > /dev/null 2>&1; then
        return 1
    fi

    if ! git rev-parse --verify agents/3 > /dev/null 2>&1; then
        return 1
    fi

    return 0
}

# Test 3: Auto-provisioning during start (fix for missing workspaces)
test_auto_provisioning_start() {
    local test_dir=$(setup_test_env "auto-provision")
    cd "$test_dir"

    # Initialize configuration but manually remove workspaces
    ./sprite init --agents 2 > /dev/null 2>&1

    # Remove workspaces to simulate the issue
    rm -rf agents/1 agents/2
    git worktree prune > /dev/null 2>&1

    # Try to start in background and check if it auto-provisions missing workspaces
    ./sprite start --force > start_output.log 2>&1 &
    local sprite_pid=$!

    # Wait a bit for provisioning to complete
    sleep 5

    # Check if workspaces were recreated
    if [[ ! -d "agents/1" ]] || [[ ! -d "agents/2" ]]; then
        kill $sprite_pid 2>/dev/null || true
        return 1
    fi

    # Check if provisioning messages appear in output
    if ! grep -q "Attempting to provision missing workspaces" start_output.log; then
        kill $sprite_pid 2>/dev/null || true
        return 1
    fi

    # Check if provisioning completed successfully
    if ! grep -q "Worktree provisioning complete" start_output.log; then
        kill $sprite_pid 2>/dev/null || true
        return 1
    fi

    # Clean up
    kill $sprite_pid 2>/dev/null || true
    tmux kill-session -t sprite-session 2>/dev/null || true
    return 0
}

# Test 4: Configuration file structure validation
test_config_structure() {
    local test_dir=$(setup_test_env "config-structure")
    cd "$test_dir"

    # Initialize with agents
    if ! ./sprite init --agents 1 > /dev/null 2>&1; then
        return 1
    fi

    # Check configuration file contains proper case
    if ! grep -q "status: Inactive" agents/agents.yaml; then
        return 1
    fi

    # Check configuration file contains proper conflict resolution
    if ! grep -q "conflict_resolution: Manual" agents/agents.yaml; then
        return 1
    fi

    # Check configuration file contains proper log level
    if ! grep -q "level: Info" agents/agents.yaml; then
        return 1
    fi

    return 0
}

# Test 5: Agents list command
test_agents_list() {
    local test_dir=$(setup_test_env "agents-list")
    cd "$test_dir"

    # Initialize with agents
    if ! ./sprite init --agents 2 > /dev/null 2>&1; then
        return 1
    fi

    # Test agents list command
    if ! ./sprite agents list > /dev/null 2>&1; then
        return 1
    fi

    # Check if output contains agent information
    if ! ./sprite agents list | grep -q "Agent 1"; then
        return 1
    fi

    return 0
}

# Test 6: Start command with different layouts
test_start_layouts() {
    local test_dir=$(setup_test_env "start-layouts")
    cd "$test_dir"

    # Initialize with agents
    if ! ./sprite init --agents 2 > /dev/null 2>&1; then
        return 1
    fi

    # Test start with tiled layout
    if run_with_timeout 5 ./sprite start --layout tiled --force > /dev/null 2>&1; then
        tmux kill-session -t sprite-session 2>/dev/null || true

        # Test start with vertical layout
        if run_with_timeout 5 ./sprite start --layout vertical --force > /dev/null 2>&1; then
            tmux kill-session -t sprite-session 2>/dev/null || true
            return 0
        fi
    fi

    return 1
}

# Main test execution
main() {
    echo -e "${BLUE}🚀 Starting Comprehensive Sprite Tests${NC}"
    echo "================================================"

    # Build sprite first
    print_info "Building sprite..."
    cd /Users/chinnaphat/Desktop/cedt/mutilagent
    if ! cargo build > /dev/null 2>&1; then
        print_error "Failed to build sprite"
        exit 1
    fi
    print_success "Sprite built successfully"

    # Run tests
    echo
    print_info "Running configuration and workspace tests..."

    run_test "Configuration parsing (case sensitivity)" test_config_parsing
    run_test "Workspace provisioning during init" test_workspace_provisioning_init
    run_test "Auto-provisioning during start" test_auto_provisioning_start
    run_test "Configuration file structure" test_config_structure
    run_test "Agents list command" test_agents_list
    run_test "Start command with different layouts" test_start_layouts

    # Print results
    echo
    echo "================================================"
    echo -e "${BLUE}🏁 Test Results${NC}"
    echo "================================================"
    echo "Total tests: $TOTAL_TESTS"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo
        print_success "🎉 All tests passed! The fixes are working correctly."
        echo
        echo "✅ Configuration parsing issue: FIXED"
        echo "✅ Workspace provisioning issue: FIXED"
        echo "✅ Case sensitivity in enum values: FIXED"
        echo "✅ Auto-provisioning during start: WORKING"
        echo
        exit 0
    else
        echo
        print_error "❌ Some tests failed. Please review the fixes."
        exit 1
    fi
}

# Set up cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
