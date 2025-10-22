#!/bin/bash

# Comprehensive Test Runner for Sprite Edge Cases
# This script runs all edge case tests and provides detailed reporting

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Global counters
TOTAL_SUITES=0
TOTAL_TESTS=0
TOTAL_PASSED=0
TOTAL_FAILED=0
FAILED_SUITES=()

# Test configuration
VERBOSE=false
CLEANUP=true
PARALLEL=false
MAX_PARALLEL=4
OUTPUT_FILE=""
TEST_TIMEOUT=300

# List of test suites
TEST_SUITES=(
    "agent_workspace_tests.sh"
    "git_worktree_tests.sh"
    "config_loading_tests.sh"
    "tmux_session_tests.sh"
    "integration_tests.sh"
)

# Utility functions
log_banner() {
    echo
    echo -e "${BOLD}${BLUE}$1${NC}"
    echo -e "${BLUE}$(printf '=%.0s' {1..50})${NC}"
    echo
}

log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
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

# Usage information
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Comprehensive test runner for Sprite edge case tests.

OPTIONS:
    -v, --verbose          Enable verbose output
    -q, --quiet            Suppress non-error output
    -c, --no-cleanup       Disable automatic cleanup (keep test directories)
    -p, --parallel         Run tests in parallel (experimental)
    -j, --jobs N           Maximum number of parallel jobs (default: 4)
    -t, --timeout SECONDS  Timeout for each test suite (default: 300)
    -o, --output FILE      Save test results to file
    -s, --suite SUITE      Run specific test suite only
    -l, --list             List available test suites
    -h, --help             Show this help message

EXAMPLES:
    $0                           # Run all tests
    $0 -v                        # Run with verbose output
    $0 -s agent_workspace_tests.sh  # Run specific suite
    $0 -p -j 8                   # Run in parallel with 8 jobs
    $0 -o results.txt            # Save results to file

AVAILABLE TEST SUITES:
EOF
    for suite in "${TEST_SUITES[@]}"; do
        echo "    - $suite"
    done
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -q|--quiet)
                # Redirect non-error output to /dev/null
                exec 1>/dev/null
                shift
                ;;
            -c|--no-cleanup)
                CLEANUP=false
                shift
                ;;
            -p|--parallel)
                PARALLEL=true
                shift
                ;;
            -j|--jobs)
                MAX_PARALLEL="$2"
                shift 2
                ;;
            -t|--timeout)
                TEST_TIMEOUT="$2"
                shift 2
                ;;
            -o|--output)
                OUTPUT_FILE="$2"
                shift 2
                ;;
            -s|--suite)
                TEST_SUITES=("$2")
                shift 2
                ;;
            -l|--list)
                echo "Available test suites:"
                for suite in "${TEST_SUITES[@]}"; do
                    echo "  - $suite"
                done
                exit 0
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing=()
    
    # Check if we're in sprite project directory
    if [[ ! -f "Cargo.toml" ]] || ! grep -q "sprite" Cargo.toml; then
        missing+=("Not in Sprite project directory")
    fi
    
    # Check required commands
    for cmd in git tmux cargo; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing+=("$cmd")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing prerequisites:"
        for item in "${missing[@]}"; do
            echo "  - $item"
        done
        exit 1
    fi
    
    log_success "All prerequisites satisfied"
}

# Build sprite binary
build_sprite() {
    log_info "Building Sprite binary..."
    
    if [[ ! -f "target/release/sprite" ]]; then
        if ! cargo build --release; then
            log_error "Failed to build sprite binary"
            exit 1
        fi
    fi
    
    log_success "Sprite binary built successfully"
}

# Run a single test suite
run_test_suite() {
    local suite=$1
    local suite_dir=$(dirname "$0")
    local suite_path="$suite_dir/$suite"
    
    if [[ ! -f "$suite_path" ]]; then
        log_error "Test suite not found: $suite"
        return 1
    fi
    
    if [[ "$VERBOSE" == true ]]; then
        log_info "Running test suite: $suite"
    fi
    
    # Set up environment for test suite
    export TEST_VERBOSE="$VERBOSE"
    export TEST_CLEANUP="$CLEANUP"
    export TEST_TIMEOUT="$TEST_TIMEOUT"
    
    # Run test suite with timeout (using gtimeout if available, or skip timeout)
    local start_time=$(date +%s)
    local exit_code=0
    
    # Check for timeout command (not available on macOS by default)
    local timeout_cmd="timeout"
    if command -v gtimeout >/dev/null 2>&1; then
        timeout_cmd="gtimeout"
    elif ! command -v timeout >/dev/null 2>&1; then
        timeout_cmd=""  # No timeout available
    fi
    
    if [[ -n "$timeout_cmd" ]]; then
        if "$timeout_cmd" "$TEST_TIMEOUT" bash "$suite_path" 2>&1; then
            exit_code=0
        else
            exit_code=$?
            if [[ $exit_code -eq 124 ]]; then
                log_warning "Test suite $suite timed out after ${TEST_TIMEOUT}s"
            else
                log_error "Test suite $suite failed with exit code $exit_code"
            fi
        fi
    else
        # Run without timeout
        if bash "$suite_path" 2>&1; then
            exit_code=0
        else
            exit_code=$?
            log_error "Test suite $suite failed with exit code $exit_code"
        fi
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    # Report results
    if [[ $exit_code -eq 0 ]]; then
        log_success "✓ $suite (${duration}s)"
        ((TOTAL_PASSED++))
    else
        log_error "✗ $suite (${duration}s)"
        ((TOTAL_FAILED++))
        FAILED_SUITES+=("$suite")
    fi
    
    ((TOTAL_TESTS++))
    ((TOTAL_SUITES++))
    
    return $exit_code
}

# Run test suites in parallel
run_parallel_tests() {
    log_info "Running tests in parallel (max jobs: $MAX_PARALLEL)"
    
    local pids=()
    local suite_files=()
    
    # Start test suites in background
    for suite in "${TEST_SUITES[@]}"; do
        if [[ "$VERBOSE" == true ]]; then
            log_info "Starting: $suite"
        fi
        
        (
            run_test_suite "$suite"
            echo $? > "/tmp/sprite_test_${suite}.exit"
        ) &
        
        pids+=($!)
        suite_files+=("$suite")
        
        # Limit parallel jobs
        while [[ ${#pids[@]} -ge $MAX_PARALLEL ]]; do
            for i in "${!pids[@]}"; do
                if ! kill -0 "${pids[$i]}" 2>/dev/null; then
                    wait "${pids[$i]}"
                    unset pids[$i]
                    break
                fi
            done
            sleep 0.1
        done
    done
    
    # Wait for all remaining jobs
    for pid in "${pids[@]}"; do
        wait "$pid"
    done
    
    # Process results
    for suite in "${suite_files[@]}"; do
        local exit_file="/tmp/sprite_test_${suite}.exit"
        if [[ -f "$exit_file" ]]; then
            local exit_code=$(cat "$exit_file")
            rm -f "$exit_file"
            
            if [[ $exit_code -eq 0 ]]; then
                log_success "✓ $suite"
                ((TOTAL_PASSED++))
            else
                log_error "✗ $suite"
                ((TOTAL_FAILED++))
                FAILED_SUITES+=("$suite")
            fi
            ((TOTAL_TESTS++))
        fi
    done
}

# Generate final report
generate_report() {
    local report_file="${1:-}"
    
    if [[ -n "$report_file" ]]; then
        log_info "Generating report: $report_file"
        {
            echo "Sprite Edge Case Test Report"
            echo "============================="
            echo "Generated: $(date)"
            echo
            echo "Summary:"
            echo "  Total Test Suites: $TOTAL_SUITES"
            echo "  Total Tests Run: $TOTAL_TESTS"
            echo "  Passed: $TOTAL_PASSED"
            echo "  Failed: $TOTAL_FAILED"
            echo "  Success Rate: $(( TOTAL_PASSED * 100 / TOTAL_SUITES ))%"
            echo
            
            if [[ ${#FAILED_SUITES[@]} -gt 0 ]]; then
                echo "Failed Test Suites:"
                for suite in "${FAILED_SUITES[@]}"; do
                    echo "  - $suite"
                done
                echo
            fi
            
            echo "Configuration:"
            echo "  Verbose: $VERBOSE"
            echo "  Cleanup: $CLEANUP"
            echo "  Parallel: $PARALLEL"
            echo "  Timeout: ${TEST_TIMEOUT}s"
            echo
        } > "$report_file"
    fi
    
    # Console report
    echo
    log_banner "FINAL REPORT"
    echo "Total Test Suites: $TOTAL_SUITES"
    echo "Total Tests Run: $TOTAL_TESTS"
    echo -e "Passed: ${GREEN}$TOTAL_PASSED${NC}"
    if [[ $TOTAL_FAILED -gt 0 ]]; then
        echo -e "Failed: ${RED}$TOTAL_FAILED${NC}"
    else
        echo "Failed: $TOTAL_FAILED"
    fi
    echo "Success Rate: $(( TOTAL_PASSED * 100 / TOTAL_SUITES ))%"
    
    if [[ ${#FAILED_SUITES[@]} -gt 0 ]]; then
        echo
        echo -e "${RED}Failed Test Suites:${NC}"
        for suite in "${FAILED_SUITES[@]}"; do
            echo "  - $suite"
        done
    fi
    
    echo
    if [[ $TOTAL_FAILED -eq 0 ]]; then
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}❌ Some tests failed!${NC}"
        return 1
    fi
}

# Cleanup function
cleanup() {
    if [[ "$CLEANUP" == true ]]; then
        log_info "Cleaning up temporary files..."
        rm -f /tmp/sprite_test_*.exit 2>/dev/null || true
        
        # Kill any orphaned tmux sessions
        tmux list-sessions 2>/dev/null | grep -E "sprite-test|test-" | cut -d: -f1 | xargs -I {} tmux kill-session -t {} 2>/dev/null || true
    fi
}

# Main function
main() {
    parse_args "$@"
    
    log_banner "SPRITE EDGE CASE TEST SUITE"
    
    # Setup
    check_prerequisites
    build_sprite
    trap cleanup EXIT
    
    local start_time=$(date +%s)
    
    # Run tests
    if [[ "$PARALLEL" == true ]]; then
        run_parallel_tests
    else
        for suite in "${TEST_SUITES[@]}"; do
            run_test_suite "$suite"
        done
    fi
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    echo
    log_info "Total test duration: ${total_duration}s"
    
    # Generate report
    generate_report "$OUTPUT_FILE"
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
