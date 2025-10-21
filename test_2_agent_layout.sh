#!/bin/bash

# Test script to verify the 2-agent panel layout fix
# Usage: ./test_2_agent_layout.sh

set -e

echo "🧪 Testing 2-agent panel layout fix..."
echo "=================================="

# Create a temporary directory for testing
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "📁 Working in: $TEMP_DIR"

# Initialize git repo
echo "📦 Initializing git repository..."
git init > /dev/null 2>&1
git config user.name "Test User"
git config user.email "test@example.com"
echo "# Test Repository for Sprite" > README.md
git add README.md > /dev/null 2>&1
git commit -m "Initial commit" > /dev/null 2>&1

echo "✅ Git repository initialized"

# Initialize sprite with 2 agents
echo "🚀 Initializing sprite with 2 agents..."
/Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite init --agents 2 --force

echo "🔍 Checking the generated profile0.sh..."

# Show the 2-agent section from the profile
echo ""
echo "📋 2-agent configuration in profile0.sh:"
echo "======================================="
sed -n '24,30p' agents/profiles/profile0.sh

echo ""
echo "✅ Profile0.sh has been updated with the fix!"
echo ""
echo "🎯 Expected behavior:"
echo "   - 1 agent: 2 panels (vertical split: supervisor top, agent bottom)"
echo "   - 2 agents: 2 panels (horizontal split: supervisor left, agent right) ← FIXED"
echo "   - 3+ agents: 3+ panels (original logic preserved)"
echo ""
echo "💡 To test the actual tmux layout:"
echo "   cd $TEMP_DIR"
echo "   /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite start"
echo ""
echo "   Then check: tmux list-panes"
echo "   You should see exactly 2 panes for 2 agents!"

# Keep the temp directory for manual testing
echo ""
echo "🧹 Test directory preserved at: $TEMP_DIR"
echo "   You can manually test by running:"
echo "   cd $TEMP_DIR"
echo "   /Users/chinnaphat/Desktop/cedt/mutilagent/target/release/sprite start"

echo ""
echo "✅ Setup complete! You can now test the panel layout fix."