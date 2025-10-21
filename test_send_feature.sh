#!/bin/bash

# Test script to verify the send feature works correctly
# Usage: ./test_send_feature.sh

set -e

echo "📤 Testing sprite send feature..."
echo "==============================="

# Create a temporary directory for testing
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "📁 Working in: $TEMP_DIR"

# Initialize git repo
echo "📦 Initializing git repository..."
git init > /dev/null 2>&1
git config user.name "Test User"
git config user.email "test@example.com"
echo "# Test Repository for Sprite Send Feature" > README.md
git add README.md > /dev/null 2>&1
git commit -m "Initial commit" > /dev/null 2>&1

echo "✅ Git repository initialized"

# Initialize sprite with 2 agents
echo "🚀 Initializing sprite with 2 agents..."
/Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite init --agents 2 --force

echo "🔍 Checking sprite status..."
/Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite status

echo ""
echo "🎮 Starting sprite session..."
/Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite start --detach

echo ""
echo "⏳ Waiting for session to be ready..."
sleep 3

echo ""
echo "📤 Testing send feature..."

# Test 1: Send a simple echo command
echo "Test 1: Sending 'echo Hello from send feature' to all agents"
if /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send "echo Hello from send feature"; then
    echo "✅ Send command completed successfully"
else
    echo "❌ Send command failed"
fi

# Test 2: Send a pwd command to check working directory
echo ""
echo "Test 2: Sending 'pwd' to check working directories"
if /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send "pwd"; then
    echo "✅ PWD command completed successfully"
else
    echo "❌ PWD command failed"
fi

# Test 3: Send a git status command
echo ""
echo "Test 3: Sending 'git status --porcelain' to check git status"
if /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send "git status --porcelain"; then
    echo "✅ Git status command completed successfully"
else
    echo "❌ Git status command failed"
fi

# Test 4: Send with timeout option
echo ""
echo "Test 4: Testing with timeout option (5 seconds)"
if /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send --timeout 5 "echo 'Test with 5 second timeout'"; then
    echo "✅ Timeout option test completed successfully"
else
    echo "❌ Timeout option test failed"
fi

echo ""
echo "📊 Checking final session status..."
/Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite status

echo ""
echo "🧹 Cleaning up session..."
/Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite kill --force

echo ""
echo "✅ Send feature test completed!"
echo ""
echo "🎯 Manual verification steps:"
echo "   1. cd $TEMP_DIR"
echo "   2. /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite start"
echo "   3. /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite attach"
echo "   4. In different tmux panes, check if commands were executed"
echo "   5. Try: /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send 'ls -la'"
echo ""
echo "🧹 Test directory preserved at: $TEMP_DIR"
echo "   You can continue manual testing there if needed."