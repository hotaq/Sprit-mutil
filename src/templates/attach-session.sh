#!/bin/bash

# Sprite Multi-Agent Attach Script
# Generated automatically by 'sprite init'

SPRITE_SESSION="${SPRITE_SESSION:-sprite-session}"

echo "🔗 Connecting to tmux session '$SPRITE_SESSION'..."

# Check if session exists
if ! tmux has-session -t "$SPRITE_SESSION" 2>/dev/null; then
    echo "❌ Session '$SPRITE_SESSION' not found"
    echo "💡 Use 'sprite start' to create a new session"
    exit 1
fi

# Show session info
echo "📊 Session Information:"
tmux list-windows -t "$SPRITE_SESSION"

echo ""
echo "🎮 Attaching to session..."
echo "💡 Use Ctrl+B then D to detach"
echo "💡 Use Ctrl+B then W to list windows"
echo "💡 Use Ctrl+B then <window number> to switch windows"

# Attach to session
tmux attach -t "$SPRITE_SESSION"