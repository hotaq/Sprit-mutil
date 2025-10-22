#!/bin/bash

# Git Worktree Operation Edge Case Tests
# Tests for git worktree creation, deletion, and conflict resolution edge cases

set -euo pipefail

# Source common utilities
source "$(dirname "$0")/common/test_utils.sh"

# Test: Worktree creation with existing branch
test_worktree_existing_branch() {
    log_test "Testing worktree creation with existing branch"
    
    setup_test_env "existing_branch"
    
    create_test_config 2
    
    # Create a branch first
    git checkout -b agent-1 >/dev/null 2>&1
    git checkout main >/dev/null 2>&1
    
    # Should handle existing branch gracefully
    assert_command_success "sprite_cmd start --force" "Should handle existing branch"
    assert_dir_exists "agents/1" "Worktree directory should be created"
    
    cleanup_test_env
}

# Test: Worktree creation with conflicting branch names
test_worktree_conflicting_branches() {
    log_test "Testing worktree creation with conflicting branch names"
    
    setup_test_env "conflicting_branches"
    
    create_test_config 2
    
    # Create a file and commit on main
    echo "test content" > test.txt
    git add test.txt >/dev/null 2>&1
    git commit -m "Add test file" >/dev/null 2>&1
    
    # Create branch with same name but different content
    git checkout -b agent-1 >/dev/null 2>&1
    echo "different content" > test.txt
    git add test.txt >/dev/null 2>&1
    git commit -m "Different content" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1
    
    # Should handle branch conflicts
    assert_command_success "sprite_cmd start --force" "Should handle branch conflicts"
    
    cleanup_test_env
}

# Test: Worktree with detached HEAD
test_worktree_detached_head() {
    log_test "Testing worktree creation from detached HEAD"
    
    setup_test_env "detached_head"
    
    # Create commits and go to detached HEAD
    echo "content" > file.txt
    git add file.txt >/dev/null 2>&1
    git commit -m "Commit 1" >/dev/null 2>&1
    echo "content2" > file.txt
    git add file.txt >/dev/null 2>&1
    git commit -m "Commit 2" >/dev/null 2>&1
    
    git checkout HEAD~1 >/dev/null 2>&1
    
    create_test_config 2
    
    # Should handle detached HEAD state
    assert_command_success "sprite_cmd start --force" "Should work from detached HEAD"
    
    cleanup_test_env
}

# Test: Stale worktree references
test_stale_worktree_references() {
    log_test "Testing cleanup of stale worktree references"
    
    setup_test_env "stale_references"
    
    create_test_config 2
    
    # Create worktrees manually
    git worktree add agents/1 agent-1 >/dev/null 2>&1
    git worktree add agents/2 agent-2 >/dev/null 2>&1
    
    # Remove directories but keep references
    rm -rf agents/1 agents/2
    
    # Should cleanup stale references
    assert_command_success "sprite_cmd start --force" "Should cleanup stale worktree references"
    assert_dir_exists "agents/1" "Worktree should be recreated"
    assert_dir_exists "agents/2" "Worktree should be recreated"
    
    cleanup_test_env
}

# Test: Worktree in nested subdirectory
test_nested_worktree() {
    log_test "Testing worktree creation in nested directories"
    
    setup_test_env "nested_worktree"
    
    # Create config with nested paths
    mkdir -p agents
    cat > agents/agents.yaml << EOF
version: "0.2.3"
agents:
- id: '1'
  branch: 'agent-1'
  worktree_path: 'deep/nested/path/agent-1'
  model: 'claude-sonnet-4'
  description: 'Nested agent'
  status: 'Inactive'
  config: { env_vars: {}, work_dir: null, aliases: [], tools: [], resource_limits: { max_memory_mb: 1024, max_cpu_percent: 80, max_processes: 100 }, timeout_settings: { default_timeout_secs: 300, max_timeout_secs: 3600 }, sync_settings: { auto_sync: false, sync_interval_secs: 300, conflict_resolution: 'Manual' } }
session_name: 'sprite-test-session'
settings: { default_timeout_secs: 300, log_level: 'Info', global_env_vars: {}, logging: { log_file: 'agents/logs/sprite.log', level: 'Info', log_to_stdout: true } }
EOF
    
    # Should create nested directories
    assert_command_success "sprite_cmd start --force" "Should create nested worktree paths"
    assert_dir_exists "deep/nested/path/agent-1" "Nested worktree directory should exist"
    
    cleanup_test_env
}

# Test: Worktree with merge conflicts
test_worktree_merge_conflicts() {
    log_test "Testing worktree with potential merge conflicts"
    
    setup_test_env "merge_conflicts"
    
    create_test_config 2
    
    # Create conflicting changes on main and agent branch
    echo "main content" > shared.txt
    git add shared.txt >/dev/null 2>&1
    git commit -m "Add shared file" >/dev/null 2>&1
    
    # Create agent branch with different content
    git checkout -b agent-1 >/dev/null 2>&1
    echo "agent content" > shared.txt
    git add shared.txt >/dev/null 2>&1
    git commit -m "Agent changes" >/dev/null 2>&1
    git checkout main >/dev/null 2>&1
    
    # Update main with different content
    echo "updated main content" > shared.txt
    git add shared.txt >/dev/null 2>&1
    git commit -m "Main updates" >/dev/null 2>&1
    
    # Should handle potential conflicts
    assert_command_success "sprite_cmd start --force" "Should handle potential merge conflicts"
    
    cleanup_test_env
}

# Test: Worktree with untracked files
test_worktree_untracked_files() {
    log_test "Testing worktree with untracked files"
    
    setup_test_env "untracked_files"
    
    create_test_config 1
    
    # Create untracked files in main repo
    echo "untracked" > untracked.txt
    mkdir -p temp
    echo "temp file" > temp/temp.txt
    
    # Should handle untracked files
    assert_command_success "sprite_cmd start --force" "Should handle untracked files"
    
    cleanup_test_env
}

# Test: Worktree with submodules
test_worktree_submodules() {
    log_test "Testing worktree with git submodules"
    
    setup_test_env "submodules"
    
    # Add a submodule if possible
    mkdir -p submodule
    cd submodule
    git init >/dev/null 2>&1
    git config user.name "Test" >/dev/null 2>&1
    git config user.email "test@example.com" >/dev/null 2>&1
    echo "submodule content" > sub.txt
    git add sub.txt >/dev/null 2>&1
    git commit -m "Initial submodule commit" >/dev/null 2>&1
    cd ..
    
    git submodule add ./submodule >/dev/null 2>&1 || {
        log_warning "Skipping submodule test - not supported in this git version"
        cleanup_test_env
        return 0
    }
    
    git commit -m "Add submodule" >/dev/null 2>&1
    
    create_test_config 1
    
    # Should handle submodules
    assert_command_success "sprite_cmd start --force" "Should handle git submodules"
    
    cleanup_test_env
}

# Test: Worktree with large files
test_worktree_large_files() {
    log_test "Testing worktree with large files"
    
    setup_test_env "large_files"
    
    # Create a large file
    dd if=/dev/zero of=large_file.bin bs=1M count=10 >/dev/null 2>&1
    git add large_file.bin >/dev/null 2>&1
    git commit -m "Add large file" >/dev/null 2>&1
    
    create_test_config 1
    
    # Should handle large files
    assert_command_success "sprite_cmd start --force" "Should handle large files in worktrees"
    
    cleanup_test_env
}

# Test: Worktree with binary files
test_worktree_binary_files() {
    log_test "Testing worktree with binary files"
    
    setup_test_env "binary_files"
    
    # Create binary files
    printf '\x00\x01\x02\x03' > binary.bin
    git add binary.bin >/dev/null 2>&1
    git commit -m "Add binary file" >/dev/null 2>&1
    
    create_test_config 1
    
    # Should handle binary files
    assert_command_success "sprite_cmd start --force" "Should handle binary files"
    
    cleanup_test_env
}

# Test: Worktree with symbolic links
test_worktree_symlinks() {
    log_test "Testing worktree with symbolic links"
    
    setup_test_env "symlinks"
    
    # Create symbolic links (if supported)
    ln -s /tmp/test_link link.txt 2>/dev/null || {
        log_warning "Skipping symlink test - not supported on this filesystem"
        cleanup_test_env
        return 0
    }
    git add link.txt >/dev/null 2>&1
    git commit -m "Add symlink" >/dev/null 2>&1
    
    create_test_config 1
    
    # Should handle symbolic links
    assert_command_success "sprite_cmd start --force" "Should handle symbolic links"
    
    cleanup_test_env
}

# Test: Worktree permission handling
test_worktree_permissions() {
    log_test "Testing worktree permission handling"
    
    setup_test_env "worktree_permissions"
    
    create_test_config 1
    
    # Start sprite to create worktree
    sprite_cmd start --force >/dev/null 2>&1 &
    local pid=$!
    sleep 2
    kill $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    
    # Make worktree read-only
    chmod 444 agents/1
    
    # Should handle permission issues
    assert_command_fails "sprite_cmd start --force" "Should fail with read-only worktree"
    
    # Restore permissions for cleanup
    chmod 755 agents/1
    
    cleanup_test_env
}

# Main test runner
main() {
    echo "================================="
    echo "Git Worktree Edge Case Tests"
    echo "================================="
    
    # Run all tests
    test_worktree_existing_branch
    test_worktree_conflicting_branches
    test_worktree_detached_head
    test_stale_worktree_references
    test_nested_worktree
    test_worktree_merge_conflicts
    test_worktree_untracked_files
    test_worktree_submodules
    test_worktree_large_files
    test_worktree_binary_files
    test_worktree_symlinks
    test_worktree_permissions
    
    # Print results
    print_test_results
}

# Run tests if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
