#!/bin/bash

# Hey Command Integration Tests
# Tests the /hey command functionality with multiple agents and scenarios

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR=$(mktemp -d)
PROJECT_NAME="hey-test-$(date +%s)"
LOG_FILE="$TEST_DIR/hey_test.log"
SPRITE_SESSION_PREFIX="test-hey"

# Safety flags
SAFE_MODE=true
CLEANUP_ON_EXIT=true

# Cleanup function
cleanup() {
    if [ "$CLEANUP_ON_EXIT" = true ]; then
        echo -e "${BLUE}Cleaning up test environment...${NC}"
        cd /Users/chinnaphat/Desktop/cedt/mutilagent
        
        # Clean up tmux sessions
        tmux list-sessions 2>/dev/null | grep "$SPRITE_SESSION_PREFIX" | cut -d: -f1 | xargs -I {} tmux kill-session -t {} 2>/dev/null || true
        
        # Remove test directory
        rm -rf "$TEST_DIR" 2>/dev/null || true
        
        echo -e "${GREEN}✅ Cleanup completed${NC}"
    fi
}

# Set up cleanup on exit
trap cleanup EXIT INT TERM

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

# Utility functions
print_test_header() {
    echo
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
    log "SUCCESS: $1"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
    log "ERROR: $1"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
    log "INFO: $1"
}

# Test setup
setup_test_environment() {
    print_test_header "Setting Up Test Environment"
    
    cd /Users/chinnaphat/Desktop/cedt/mutilagent
    
    # Create test project directory
    mkdir -p "$TEST_DIR/$PROJECT_NAME"
    cd "$TEST_DIR/$PROJECT_NAME"
    
    print_info "Test project created at: $(pwd)"
    
    # Initialize git repository
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    # Create initial commit
    echo "# $PROJECT_NAME" > README.md
    git add README.md
    git commit -m "Initial commit"
    
    print_success "Git repository initialized"
    
    # Initialize sprite environment
    log "Initializing sprite environment..."
    if cargo run init --force > /dev/null 2>&1; then
        print_success "Sprite environment initialized"
    else
        print_error "Failed to initialize sprite environment"
        exit 1
    fi
}

# Create test agent configuration
create_agent_config() {
    log "Creating test agent configuration..."
    
    cat > agents/agents.yaml << EOF
agents:
  frontend:
    name: "Frontend Agent"
    profile: "profile0.sh"
    workspace: "frontend"
    active: true
    commands: ["echo", "npm", "node", "ls"]
  
  backend:
    name: "Backend Agent"
    profile: "profile1.sh"
    workspace: "backend"
    active: true
    commands: ["echo", "cargo", "rustc", "ls"]
  
  devops:
    name: "DevOps Agent"
    profile: "profile2.sh"
    workspace: "infrastructure"
    active: true
    commands: ["echo", "docker", "kubectl", "ls"]

sprite:
  name: "sprite-test"
  description: "Test sprite for hey command functionality"
  agents: ["frontend", "backend", "devops"]
  tmux:
    session_name: "$SPRITE_SESSION_PREFIX-$PROJECT_NAME"
    layout: "main-vertical"
EOF

    # Create workspaces
    mkdir -p frontend backend infrastructure
    
    # Create fake content in each workspace
    echo "console.log('Hello from frontend');" > frontend/app.js
    echo "fn main() { println!(\"Hello from backend\"); }" > backend/main.rs
    echo "version: '3.8'" > infrastructure/docker-compose.yml
    
    print_success "Agent configuration created"
}

# Test 1: Basic hey command functionality
test_basic_hey_command() {
    print_test_header "Test 1: Basic Hey Command Functionality"
    
    cd "/Users/chinnaphat/Desktop/cedt/mutilagent"
    cd "$TEST_DIR/$PROJECT_NAME"
    
    # Start sprite session
    log "Starting sprite session..."
    if cargo run start --agents 3 > /dev/null 2>&1; then
        print_success "Sprite session started"
    else
        print_error "Failed to start sprite session"
        return 1
    fi
    
    # Wait for session to initialize
    sleep 5
    
    # Test hey command to frontend agent
    log "Testing hey command to frontend agent..."
    if cargo run hey frontend "echo 'Hello from frontend test'" > /dev/null 2>&1; then
        print_success "Hey command to frontend agent succeeded"
    else
        print_error "Hey command to frontend agent failed"
        return 1
    fi
    
    sleep 2
    
    # Test hey command to backend agent
    log "Testing hey command to backend agent..."
    if cargo run hey backend "echo 'Hello from backend test'" > /dev/null 2>&1; then
        print_success "Hey command to backend agent succeeded"
    else
        print_error "Hey command to backend agent failed"
        return 1
    fi
    
    sleep 2
    
    # Test hey command to devops agent
    log "Testing hey command to devops agent..."
    if cargo run hey devops "echo 'Hello from devops test'" > /dev/null 2>&1; then
        print_success "Hey command to devops agent succeeded"
    else
        print_error "Hey command to devops agent failed"
        return 1
    fi
    
    print_success "Basic hey command tests completed"
}

# Test 2: Hey command with different flags
test_hey_command_flags() {
    print_test_header "Test 2: Hey Command with Flags"
    
    cd "/Users/chinnaphat/Desktop/cedt/mutilagent"
    cd "$TEST_DIR/$PROJECT_NAME"
    
    # Test with work-dir flag
    log "Testing hey command with work-dir flag..."
    if cargo run hey frontend "ls" --work-dir "frontend" > /dev/null 2>&1; then
        print_success "Hey command with work-dir flag succeeded"
    else
        print_error "Hey command with work-dir flag failed"
        return 1
    fi
    
    sleep 2
    
    # Test with timeout flag
    log "Testing hey command with timeout flag..."
    if cargo run hey backend "echo 'timeout test'" --timeout 10 > /dev/null 2>&1; then
        print_success "Hey command with timeout flag succeeded"
    else
        print_error "Hey command with timeout flag failed"
        return 1
    fi
    
    sleep 2
    
    print_success "Hey command flags tests completed"
}

# Test 3: Error scenarios
test_hey_command_errors() {
    print_test_header "Test 3: Hey Command Error Scenarios"
    
    cd "/Users/chinnaphat/Desktop/cedt/mutilagent"
    
    # Test command to non-existent agent
    log "Testing hey command to non-existent agent..."
    cd "$TEST_DIR/$PROJECT_NAME"
    if cargo run hey nonexistent-agent "echo test" 2>/dev/null; then
        print_error "Hey command to non-existent agent should have failed"
        return 1
    else
        print_success "Hey command correctly rejected non-existent agent"
    fi
    
    # Test empty agent name
    log "Testing hey command with empty agent name..."
    if cargo run hey "" "echo test" 2>/dev/null; then
        print_error "Hey command with empty agent name should have failed"
        return 1
    else
        print_success "Hey command correctly rejected empty agent name"
    fi
    
    print_success "Error scenario tests completed"
}

# Test 4: Performance test
test_hey_command_performance() {
    print_test_header "Test 4: Hey Command Performance Test"
    
    cd "/Users/chinnaphat/Desktop/cedt/mutilagent"
    cd "$TEST_DIR/$PROJECT_NAME"
    
    # Measure execution time
    start_time=$(date +%s.%N)
    
    log "Running performance test..."
    if cargo run hey frontend "echo 'performance test'" > /dev/null 2>&1; then
        end_time=$(date +%s.%N)
        execution_time=$(echo "$end_time - $start_time" | bc)
        
        log "Hey command execution time: ${execution_time}s"
        
        # Check if execution time is less than 3 seconds
        if (( $(echo "$execution_time < 3.0" | bc -l) )); then
            print_success "Hey command meets performance requirement (${execution_time}s < 3s)"
        else
            print_error "Hey command exceeds performance requirement (${execution_time}s >= 3s)"
            return 1
        fi
    else
        print_error "Performance test failed"
        return 1
    fi
    
    print_success "Performance test completed"
}

# Test 5: Concurrent commands
test_concurrent_commands() {
    print_test_header "Test 5: Concurrent Hey Commands"
    
    cd "/Users/chinnaphat/Desktop/cedt/mutilagent"
    cd "$TEST_DIR/$PROJECT_NAME"
    
    start_time=$(date +%s.%N)
    
    # Send concurrent commands to different agents
    (
        cargo run hey frontend "echo 'Concurrent test 1'" > /dev/null 2>&1 &
        cargo run hey backend "echo 'Concurrent test 2'" > /dev/null 2>&1 &
        cargo run hey devops "echo 'Concurrent test 3'" > /dev/null 2>&1 &
        wait
    )
    
    end_time=$(date +%s.%N)
    total_time=$(echo "$end_time - $start_time" | bc)
    
    log "Concurrent commands total time: ${total_time}s"
    
    # Check if total time is reasonable (< 10 seconds for 3 concurrent commands)
    if (( $(echo "$total_time < 10.0" | bc -l) )); then
        print_success "Concurrent commands completed efficiently (${total_time}s < 10s)"
    else
        print_error "Concurrent commands took too long (${total_time}s >= 10s)"
        return 1
    fi
    
    print_success "Concurrent commands test completed"
}

# Main test execution
main() {
    print_test_header "Hey Command Integration Tests"
    
    log "Starting hey command integration tests"
    log "Test directory: $TEST_DIR"
    log "Project name: $PROJECT_NAME"
    
    # Run tests
    setup_test_environment
    create_agent_config
    test_basic_hey_command
    test_hey_command_flags
    test_hey_command_errors
    test_hey_command_performance
    test_concurrent_commands
    
    print_test_header "Test Summary"
    print_success "All hey command integration tests passed! ✅"
    
    log "All tests completed successfully"
    log "Log file: $LOG_FILE"
}

# Check dependencies
if ! command -v cargo &> /dev/null; then
    print_error "cargo is not installed"
    exit 1
fi

if ! command -v tmux &> /dev/null; then
    print_error "tmux is not installed"
    exit 1
fi

if ! command -v bc &> /dev/null; then
    print_error "bc is not installed (required for performance tests)"
    exit 1
fi

# Run the tests
main "$@"
