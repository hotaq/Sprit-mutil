#!/bin/bash

# Sprite Multi-Agent Environment Setup Script
# Generated automatically by 'sprite init'

set -e

echo "🔧 Setting up Sprite multi-agent environment..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Install dependencies if needed
if command -v cargo > /dev/null 2>&1; then
    echo "📦 Building Sprite..."
    cargo build --release
else
    echo "❌ Error: Rust/Cargo not found"
    exit 1
fi

# Create symlink for global access (optional)
read -p "🔗 Create global 'sprite' command? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -w "/usr/local/bin" ]; then
        ln -sf "$(pwd)/target/release/sprite" /usr/local/bin/sprite
        echo "✅ Global 'sprite' command created"
    else
        echo "⚠️  Cannot create global command (no permission to /usr/local/bin)"
        echo "   You can manually add $(pwd)/target/release to your PATH"
    fi
fi

echo "✅ Setup complete!"
echo "🚀 Run 'sprite init' to create your first multi-agent environment"
