#!/bin/bash

# Test script to verify mouse support in tmux sessions
# Usage: ./test_mouse_support.sh

set -e

echo "🖱️  Testing mouse support in tmux sessions..."
echo "=========================================="

# Create a temporary directory for testing
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "📁 Working in: $TEMP_DIR"

# Initialize git repo
echo "📦 Initializing git repository..."
git init > /dev/null 2>&1
git config user.name "Test User"
git config user.email "test@example.com"
echo "# Test Repository for Sprite Mouse Support" > README.md
git add README.md > /dev/null 2>&1
git commit -m "Initial commit" > /dev/null 2>&1

echo "✅ Git repository initialized"

# Initialize sprite with 2 agents
echo "🚀 Initializing sprite with 2 agents (profile0)..."
/Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite init --agents 2 --force

echo "🔍 Checking the updated profile0.sh for mouse support..."

# Show mouse support configuration
echo ""
echo "📋 Mouse support in profile0.sh:"
echo "================================"
grep -A 2 -B 2 "mouse on" agents/profiles/profile0.sh

echo ""
echo "✅ Mouse support configuration found!"
echo ""
echo "🎯 How to test mouse functionality:"
echo "   1. cd $TEMP_DIR"
echo "   2. /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite start"
echo "   3. /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite attach"
echo ""
echo "🖱️  In tmux, you should now be able to:"
echo "   • Click on any panel to switch to it"
echo "   • Click and drag to resize panels"
echo "   • Use mouse wheel to scroll within panels"
echo "   • Right-click for tmux context menu"
echo ""
echo "⌨️  Alternative navigation methods:"
echo "   • Ctrl+B then Arrow keys (←→↑↓) to switch panels"
echo "   • Ctrl+B then O to rotate through panes"
echo "   • Ctrl+B then { or } to swap panes"
echo ""
echo "🧹 Test directory preserved at: $TEMP_DIR"
echo "   You can manually test the mouse functionality there."

echo ""
echo "✅ Mouse support test setup complete!"
echo "🎉 Your tmux sessions should now respond to mouse clicks!"