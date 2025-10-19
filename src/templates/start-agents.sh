#!/bin/bash

# Sprite Multi-Agent Start Script
# Generated automatically by 'sprite init'

set -e

SPRITE_SESSION="${SPRITE_SESSION:-sprite-session}"
AGENT_COUNT="${AGENT_COUNT:-3}"

echo "🚀 Starting $AGENT_COUNT agents in tmux session '$SPRITE_SESSION'..."

# Check if session already exists
if tmux has-session -t "$SPRITE_SESSION" 2>/dev/null; then
    echo "⚠️  Session '$SPRITE_SESSION' already exists"
    echo "   Use 'sprite attach' to connect or 'sprite kill' to terminate"
    exit 1
fi

# Create new session
tmux new-session -d -s "$SPRITE_SESSION"

# Create windows for each agent
for i in $(seq 1 $AGENT_COUNT); do
    if [ $i -eq 1 ]; then
        # First agent uses the initial window
        tmux rename-window -t "$SPRITE_SESSION:0" "agent-$i"
        tmux send-keys -t "$SPRITE_SESSION:agent-$i" "cd agents/$i && clear" C-m
    else
        # Create new windows for additional agents
        tmux new-window -t "$SPRITE_SESSION" -n "agent-$i"
        tmux send-keys -t "$SPRITE_SESSION:agent-$i" "cd agents/$i && clear" C-m
    fi

    echo "✅ Started agent $i in window 'agent-$i'"
done

# Create supervisor window
tmux new-window -t "$SPRITE_SESSION" -n "supervisor"
tmux send-keys -t "$SPRITE_SESSION:supervisor" "echo '🎯 Supervisor Control Panel' && echo 'Use sprite hey <agent> <command> to send commands'" C-m

echo "✅ All agents started successfully!"
echo "🎮 Use 'sprite attach' to connect to the session"
echo "📋 Use 'tmux list-windows -t $SPRITE_SESSION' to see all windows"