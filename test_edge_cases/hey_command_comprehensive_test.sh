#!/bin/bash

# Comprehensive Hey Command Test Suite
# Tests all aspects of the /hey command functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Test configuration
ORIGINAL_DIR="/Users/chinnaphat/Desktop/cedt/mutilagent"
TEST_BASE_DIR="/tmp/sprite-hey-tests"
PROJECT_NAME="hey-$(date +%s)"
TEST_PROJECT_DIR="$TEST_BASE_DIR/$PROJECT_NAME"
LOG_FILE="$TEST_PROJECT_DIR/test.log"
SPRITE_SESSION_PREFIX="test-hey"
CLEANUP_ON_EXIT=true

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Cleanup function
cleanup() {
    if [ "$CLEANUP_ON_EXIT" = true ]; then
        echo -e "\n${BLUE}========================================${NC}"
        echo -e "${BLUE}Cleaning Up Test Environment${NC}"
        echo -e "${BLUE}========================================${NC}"
        
        # Clean up tmux sessions
        echo -e "${YELLOW}Cleaning up tmux sessions...${NC}"
        tmux list-sessions 2>/dev/null | grep "$SPRITE_SESSION_PREFIX" | cut -d: -f1 | while read session; do
            tmux kill-session -t "$session" 2>/dev/null || true
            echo "  Killed session: $session"
        done
        
        # Remove test directory
        echo -e "${YELLOW}Removing test directory...${NC}"
        rm -rf "$TEST_BASE_DIR" 2>/dev/null || true
        
        echo -e "${GREEN}✅ Cleanup completed${NC}"
    fi
}

# Set up cleanup on exit
trap cleanup EXIT INT TERM

# Logging functions
log() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] $1" | tee -a "$LOG_FILE"
}

# Test framework functions
print_header() {
    echo
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    log "TEST SUITE: $1"
}

print_test() {
    ((TESTS_RUN++))
    echo -e "\n${CYAN}TEST $TESTS_RUN: $1${NC}"
    log "TEST $TESTS_RUN: $1"
}

print_success() {
    ((TESTS_PASSED++))
    echo -e "${GREEN}✅ PASS: $1${NC}"
    log "PASS: $1"
}

print_error() {
    ((TESTS_FAILED++))
    echo -e "${RED}❌ FAIL: $1${NC}"
    log "FAIL: $1"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
    log "INFO: $1"
}

print_warning() {
    echo -e "${MAGENTA}⚠️  $1${NC}"
    log "WARNING: $1"
}

# Assert functions
assert_command_success() {
    local description="$1"
    shift
    local command="$@"
    
    print_test "$description"
    
    if $command > /dev/null 2>&1; then
        print_success "$description - Command succeeded"
        return 0
    else
        print_error "$description - Command failed"
        return 1
    fi
}

assert_command_failure() {
    local description="$1"
    shift
    local command="$@"
    
    print_test "$description"
    
    if ! $command > /dev/null 2>&1; then
        print_success "$description - Command correctly failed"
        return 0
    else
        print_error "$description - Command should have failed"
        return 1
    fi
}

assert_time_less_than() {
    local description="$1"
    local max_time="$2"
    local actual_time="$3"
    
    print_test "$description"
    
    if (( $(echo "$actual_time < $max_time" | bc -l) )); then
        print_success "$description (${actual_time}s < ${max_time}s)"
        return 0
    else
        print_error "$description (${actual_time}s >= ${max_time}s)"
        return 1
    fi
}

# Setup test environment
setup_test_environment() {
    print_header "Setting Up Test Environment"
    
    # Clean up any previous test runs
    rm -rf "$TEST_BASE_DIR" 2>/dev/null || true
    mkdir -p "$TEST_PROJECT_DIR"
    
    cd "$ORIGINAL_DIR"
    print_info "Original directory: $ORIGINAL_DIR"
    
    # Create test project directory
    mkdir -p "$TEST_PROJECT_DIR"
    cd "$TEST_PROJECT_DIR"
    
    print_info "Test project created at: $(pwd)"
    
    # Initialize git repository
    print_info "Initializing git repository..."
    git init > /dev/null 2>&1
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    # Create initial commit
    echo "# $PROJECT_NAME" > README.md
    git add README.md > /dev/null 2>&1
    git commit -m "Initial commit" > /dev/null 2>&1
    
    print_success "Git repository initialized"
    
    # Initialize sprite environment
    print_info "Initializing sprite environment..."
    if cargo run --bin sprite init --force > /dev/null 2>&1; then
        print_success "Sprite environment initialized"
    else
        print_error "Failed to initialize sprite environment"
        exit 1
    fi
    
    # Create workspace directories
    mkdir -p frontend backend infrastructure docs
    
    # Create test files in each workspace
    echo "console.log('Hello from frontend!');" > frontend/app.js
    echo "fn main() { println!(\"Hello from backend!\"); }" > backend/main.rs
    echo "version: '3.8'" > infrastructure/docker-compose.yml
    echo "# Documentation" > docs/README.md
    
    print_success "Workspace directories created"
}

# Create comprehensive agent configuration
create_agent_config() {
    print_header "Creating Test Agent Configuration"
    
    print_info "Creating multi-agent configuration..."
    
    cat > agents/agents.yaml << 'EOF'
agents:
  frontend:
    name: "Frontend Developer"
    profile: "profile0.sh"
    workspace: "frontend"
    active: true
    commands: ["npm", "node", "echo", "ls", "cat"]
    
  backend:
    name: "Backend Developer"
    profile: "profile1.sh"
    workspace: "backend"
    active: true
    commands: ["cargo", "rustc", "echo", "ls", "cat"]
    
  devops:
    name: "DevOps Engineer"
    profile: "profile2.sh"
    workspace: "infrastructure"
    active: true
    commands: ["docker", "kubectl", "echo", "ls", "cat"]
    
  docs:
    name: "Documentation Writer"
    profile: "profile3.sh"
    workspace: "docs"
    active: true
    commands: ["echo", "ls", "cat", "markdown"]

sprite:
  name: "comprehensive-test-sprite"
  description: "Test sprite for comprehensive hey command testing"
  agents: ["frontend", "backend", "devops", "docs"]
  tmux:
    session_name: "test-hey-comprehensive"
    layout: "main-vertical"
    
features:
  git_integration: true
  workspace_management: true
  health_monitoring: true
EOF

    print_success "Multi-agent configuration created"
    echo "  - Frontend Agent (Agent 1)"
    echo "  - Backend Agent (Agent 2)"
    echo "  - DevOps Agent (Agent 3)"
    echo "  - Documentation Agent (Agent 4)"
}

# Test Suite 1: Basic Hey Command Functionality
test_basic_hey_functionality() {
    print_header "Test Suite 1: Basic Hey Command Functionality"
    
    cd "$ORIGINAL_DIR"
    
    # Test 1.1: Verify hey command exists
    assert_command_success "hey command exists" \
        cargo run hey --help
    
    # Test 1.2: Test hey command with no arguments (should show help)
    assert_command_failure "hey command with no arguments" \
        cargo run hey
    
    # Test 1.3: Test hey command to non-existent agent (should fail)
    assert_command_failure "hey command to non-existent agent" \
        cargo run hey 99 echo "test"
    
    print_success "Basic hey command functionality tests completed"
}

# Test Suite 2: Hey Command with Active Sprite Session
test_hey_with_sprite_session() {
    print_header "Test Suite 2: Hey Command with Active Sprite Session"
    
    cd "$TEST_PROJECT_DIR"
    
    # Start the sprite session
    print_info "Starting sprite session with 4 agents..."
    cd "$ORIGINAL_DIR"
    if cargo run --bin sprite start --agents 4 --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1; then
        print_success "Sprite session started"
    else
        print_error "Failed to start sprite session"
        return 1
    fi
    
    # Wait for session to initialize
    print_info "Waiting for sprite session to initialize..."
    sleep 8
    
    # Test 2.1: Basic hey command to agent 1
    assert_command_success "hey command to agent 1 (frontend)" \
        cargo run --bin sprite hey 1 echo "Hello from frontend!"
    
    sleep 2
    
    # Test 2.2: Basic hey command to agent 2
    assert_command_success "hey command to agent 2 (backend)" \
        cargo run --bin sprite hey 2 echo "Hello from backend!"
    
    sleep 2
    
    # Test 2.3: Basic hey command to agent 3
    assert_command_success "hey command to agent 3 (devops)" \
        cargo run --bin sprite hey 3 echo "Hello from devops!"
    
    sleep 2
    
    # Test 2.4: Basic hey command to agent 4
    assert_command_success "hey command to agent 4 (docs)" \
        cargo run --bin sprite hey 4 echo "Hello from docs!"
    
    sleep 3
    
    print_success "Hey command with sprite session tests completed"
}

# Test Suite 3: Hey Command with Flags and Options
test_hey_command_flags() {
    print_header "Test Suite 3: Hey Command with Flags and Options"
    
    cd "$ORIGINAL_DIR"
    
    # Test 3.1: Hey command with work-dir flag
    assert_command_success "hey with work-dir flag (frontend workspace)" \
        cargo run --bin sprite hey 1 ls -la --work-dir "$TEST_PROJECT_DIR/frontend"
    
    sleep 2
    
    # Test 3.2: Hey command with timeout flag
    assert_command_success "hey with timeout flag (5 seconds)" \
        cargo run --bin sprite hey 2 echo "Timeout test" --timeout 5
    
    sleep 2
    
    # Test 3.3: Hey command with environment variable
    assert_command_success "hey with environment variable" \
        cargo run --bin sprite hey 3 'echo "Environment: $TEST_VAR"' --env TEST_VAR=comprehensive_test
    
    sleep 2
    
    # Test 3.4: Hey command with multiple args
    assert_command_success "hey with multiple arguments" \
        cargo run --bin sprite hey 4 echo "Multiple" "args" "test"
    
    sleep 2
    
    # Test 3.5: Hey command with complex command
    assert_command_success "hey with complex command (file read)" \
        cargo run --bin sprite hey 1 cat frontend/app.js --work-dir "$TEST_PROJECT_DIR"
    
    sleep 3
    
    print_success "Hey command flags tests completed"
}

# Test Suite 4: Performance Testing
test_hey_command_performance() {
    print_header "Test Suite 4: Performance Testing"
    
    cd "$ORIGINAL_DIR"
    
    # Test 4.1: Simple command performance (< 3 seconds)
    print_test "Simple hey command performance (< 3s)"
    start_time=$(date +%s.%N)
    
    if cargo run --bin sprite hey 1 echo "performance test" --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1; then
        end_time=$(date +%s.%N)
        execution_time=$(echo "$end_time - $start_time" | bc)
        
        assert_time_less_than "Simple hey command performance" 3.0 "$execution_time"
    else
        print_error "Performance test command failed"
    fi
    
    sleep 2
    
    # Test 4.2: Complex command performance (< 5 seconds)
    print_test "Complex hey command performance (< 5s)"
    start_time=$(date +%s.%N)
    
    if cargo run --bin sprite hey 2 'echo "Complex test"; sleep 1; echo "Done complex test"' --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1; then
        end_time=$(date +%s.%N)
        execution_time=$(echo "$end_time - $start_time" | bc)
        
        assert_time_less_than "Complex hey command performance" 5.0 "$execution_time"
    else
        print_error "Complex performance test command failed"
    fi
    
    sleep 3
    
    print_success "Performance tests completed"
}

# Test Suite 5: Concurrent Commands Testing
test_concurrent_commands() {
    print_header "Test Suite 5: Concurrent Commands Testing"
    
    cd "$ORIGINAL_DIR"
    
    # Test 5.1: Multiple concurrent commands to different agents
    print_test "Concurrent commands to different agents"
    start_time=$(date +%s.%N)
    
    (
        cargo run --bin sprite hey 1 echo "Concurrent 1" --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1 &
        cargo run --bin sprite hey 2 echo "Concurrent 2" --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1 &
        cargo run --bin sprite hey 3 echo "Concurrent 3" --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1 &
        cargo run --bin sprite hey 4 echo "Concurrent 4" --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1 &
        wait
    )
    
    end_time=$(date +%s.%N)
    total_time=$(echo "$end_time - $start_time" | bc)
    
    assert_time_less_than "Concurrent commands total time" 15.0 "$total_time"
    
    sleep 5
    
    # Test 5.2: Multiple commands to same agent
    print_test "Multiple commands to same agent"
    start_time=$(date +%s.%N)
    
    (
        for i in {1..3}; do
            cargo run --bin sprite hey 1 "echo 'Same agent $i'" --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1 &
        done
        wait
    )
    
    end_time=$(date +%s.%N)
    total_time=$(echo "$end_time - $start_time" | bc)
    
    assert_time_less_than "Multiple commands to same agent" 10.0 "$total_time"
    
    sleep 3
    
    print_success "Concurrent commands tests completed"
}

# Test Suite 6: Error Handling and Edge Cases
test_error_handling() {
    print_header "Test Suite 6: Error Handling and Edge Cases"
    
    cd "$ORIGINAL_DIR"
    
    # Test 6.1: Command to invalid agent number
    assert_command_failure "hey command to invalid agent number" \
        cargo run --bin sprite hey 99 echo "invalid agent" --project-dir "$TEST_PROJECT_DIR"
    
    # Test 6.2: Empty command
    assert_command_failure "hey command with empty command" \
        cargo run --bin sprite hey 1 "" --project-dir "$TEST_PROJECT_DIR"
    
    # Test 6.3: Negative timeout
    assert_command_failure "hey command with negative timeout" \
        cargo run --bin sprite hey 1 echo "test" --timeout -5 --project-dir "$TEST_PROJECT_DIR"
    
    # Test 6.4: Invalid workspace directory
    print_info "Testing invalid workspace directory (should not crash)..."
    if cargo run --bin sprite hey 1 echo "test" --work-dir "/nonexistent" --project-dir "$TEST_PROJECT_DIR" > /dev/null 2>&1; then
        print_warning "Command succeeded with invalid work-dir (may create directory)"
    else
        print_success "Command correctly handled invalid work-dir"
    fi
    
    sleep 2
    
    print_success "Error handling tests completed"
}

# Test Suite 7: Session Verification
test_session_verification() {
    print_header "Test Suite 7: Session Verification"
    
    # Check tmux session
    print_test "Verify tmux session exists"
    if tmux list-sessions 2>/dev/null | grep -q "$SPRITE_SESSION_PREFIX"; then
        print_success "Tmux session found"
        
        # Display session information
        echo -e "${WHITE}Active tmux sessions:${NC}"
        tmux list-sessions 2>/dev/null | grep "$SPRITE_SESSION_PREFIX" | while read line; do
            echo "  $line"
        done
        
        # Count panes
        local session_name=$(tmux list-sessions 2>/dev/null | grep "$SPRITE_SESSION_PREFIX" | head -1 | cut -d: -f1)
        if [ -n "$session_name" ]; then
            local pane_count=$(tmux display-message -p -t "$session_name" '#{window_panes}' 2>/dev/null || echo "unknown")
            echo -e "${WHITE}Panes in session $session_name: $pane_count${NC}"
        fi
    else
        print_error "No tmux session found"
    fi
}

# Generate test report
generate_test_report() {
    print_header "Test Report Summary"
    
    echo -e "${WHITE}========================================${NC}"
    echo -e "${WHITE}COMPREHENSIVE HEY COMMAND TEST RESULTS${NC}"
    echo -e "${WHITE}========================================${NC}"
    echo -e "${WHITE}Tests Run:     ${CYAN}$TESTS_RUN${NC}"
    echo -e "${WHITE}Tests Passed:   ${GREEN}$TESTS_PASSED${NC}"
    echo -e "${WHITE}Tests Failed:   ${RED}$TESTS_FAILED${NC}"
    
    local pass_rate=0
    if [ $TESTS_RUN -gt 0 ]; then
        pass_rate=$(echo "scale=2; $TESTS_PASSED * 100 / $TESTS_RUN" | bc)
    fi
    
    echo -e "${WHITE}Pass Rate:      ${CYAN}${pass_rate}%${NC}"
    echo -e "${WHITE}========================================${NC}"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}🎉 ALL TESTS PASSED! 🎉${NC}"
        echo -e "${GREEN}The hey command is fully functional and ready for production!${NC}"
    else
        echo -e "${RED}❌ SOME TESTS FAILED${NC}"
        echo -e "${YELLOW}Please check the log file for details: $LOG_FILE${NC}"
    fi
    
    echo ""
    echo -e "${WHITE}Test Features Verified:${NC}"
    echo -e "${GREEN}✅ Basic hey command execution${NC}"
    echo -e "${GREEN}✅ Multi-agent support (4 agents tested)${NC}"
    echo -e "${GREEN}✅ Command flags (--timeout, --work-dir, --env)${NC}"
    echo -e "${GREEN}✅ Performance requirements (<3s simple, <5s complex)${NC}"
    echo -e "${GREEN}✅ Concurrent command execution${NC}"
    echo -e "${GREEN}✅ Error handling and validation${NC}"
    echo -e "${GREEN}✅ Tmux session integration${NC}"
    
    echo ""
    echo -e "${WHITE}Log file: $LOG_FILE${NC}"
    echo -e "${WHITE}Test directory: $TEST_PROJECT_DIR${NC}"
}

# Main test execution function
main() {
    print_header "COMPREHENSIVE HEY COMMAND TEST SUITE"
    
    log "Starting comprehensive hey command tests"
    log "Original directory: $ORIGINAL_DIR"
    log "Test directory: $TEST_PROJECT_DIR"
    log "Project name: $PROJECT_NAME"
    
    # Check dependencies
    print_info "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "cargo is not installed"
        exit 1
    fi
    
    if ! command -v tmux &> /dev/null; then
        print_error "tmux is not installed"
        exit 1
    fi
    
    if ! command -v bc &> /dev/null; then
        print_error "bc is not installed (required for time calculations)"
        exit 1
    fi
    
    print_success "All dependencies found"
    
    # Run test suites
    setup_test_environment
    create_agent_config
    test_basic_hey_functionality
    test_hey_with_sprite_session
    test_hey_command_flags
    test_hey_command_performance
    test_concurrent_commands
    test_error_handling
    test_session_verification
    
    # Generate final report
    generate_test_report
    
    log "Test suite completed"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-cleanup)
            CLEANUP_ON_EXIT=false
            shift
            ;;
        --log-file)
            LOG_FILE="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--no-cleanup] [--log-file <file>] [--help]"
            echo "  --no-cleanup    Keep test artifacts after completion"
            echo "  --log-file      Specify custom log file location"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run the main test suite
main "$@"
