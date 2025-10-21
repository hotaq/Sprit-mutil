#!/bin/bash

# Test script to verify the send feature works correctly
# Usage: ./test_send_working.sh

set -e

echo "📤 Testing sprite send feature (Fixed Version)..."
echo "========================================="

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
sleep 5

echo ""
echo "📤 Testing send feature..."

# Test 1: Send a simple echo command
echo "Test 1: Sending 'echo Hello from agent 1 & 2' to all agents"
if /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send "echo 'Hello from agent'"; then
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

# Test 3: Send a ls command
echo ""
echo "Test 3: Sending 'ls -la' to list files"
if /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send "ls -la"; then
    echo "✅ LS command completed successfully"
else
    echo "❌ LS command failed"
fi

echo ""
echo "📊 Checking final session status..."
/Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite status

echo ""
echo "🧹 Keeping session alive for manual testing..."
echo ""
echo "🎯 Manual testing steps:"
echo "   1. cd $TEMP_DIR"
echo "   2. /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite attach"
echo "   3. Check the output of the commands in different panes"
echo "   4. Try more send commands:"
echo "      - /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send 'echo Testing again'"
echo "      - /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send 'date'"
echo "      - /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite send 'git log --oneline -1'"
echo ""
echo "🖱️  Mouse navigation:"
echo "   - Click on different panes to switch between them"
echo "   - You should see exactly 2 panes (supervisor + agent) for 2 agents"
echo ""
echo "⌨️  Keyboard navigation:"
echo "   - Ctrl+B then Arrow keys to switch panes"
echo "   - Ctrl+B then O to rotate through panes"
echo ""
echo "🧹 When done testing:"
echo "   /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite kill --force"
echo ""
echo "🧹 Test directory preserved at: $TEMP_DIR"