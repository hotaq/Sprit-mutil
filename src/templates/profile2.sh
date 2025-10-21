#!/bin/bash

# Sprite Tmux Profile 2: Top Row + Full-Width Bottom
# Layout: Agents in top row, supervisor on full-width bottom
# Best for: 4-6 agents with overview supervision

SPRITE_SESSION="${SPRITE_SESSION:-sprite-session}"
AGENT_COUNT="${AGENT_COUNT:-4}"

echo "🖼️  Applying tmux Profile 2: Top Row + Full-Width Bottom"

# Kill existing session if it exists
tmux kill-session -t "$SPRITE_SESSION" 2>/dev/null || true

# Create new session
tmux new-session -d -s "$SPRITE_SESSION"

# Enable mouse support for better navigation
tmux set-option -g mouse on
tmux set-option -t "$SPRITE_SESSION" mouse on

# Create agent row at top
if [ "$AGENT_COUNT" -ge 1 ]; then
    tmux send-keys -t "$SPRITE_SESSION:0" "cd agents/1 && echo '🤖 Agent 1 Workspace'" C-m
    tmux rename-window -t "$SPRITE_SESSION:0" "agent-1"
fi

# Create additional agent windows
for i in $(seq 2 $AGENT_COUNT); do
    tmux new-window -t "$SPRITE_SESSION" -n "agent-$i"
    tmux send-keys -t "$SPRITE_SESSION:agent-$i" "cd agents/$i && echo '🤖 Agent $i Workspace'" C-m
done

# Create supervisor window at bottom
tmux new-window -t "$SPRITE_SESSION" -n "supervisor"
tmux send-keys -t "$SPRITE_SESSION:supervisor" "echo '🎯 Supervisor Control Panel - Overview Mode'" C-m

# Arrange windows with even layout
tmux select-layout -t "$SPRITE_SESSION" even-horizontal

echo "✅ profile2 applied successfully!"
echo "🖱️  Mouse support enabled - Click to switch between panels"
echo "⌨️  Keyboard shortcuts: Ctrl+B then Arrow keys to navigate"
echo "🎮 Use 'sprite attach' to connect to the session"